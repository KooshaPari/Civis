# Sim-LOD Cohorts — How Simulation Detail Degrades with Distance

> **Status:** Design (Wave 2 slice). Companion to
> [`streaming-window.md`](streaming-window.md) and
> [`EMERGENCE_WIRING_PATCHPLAN.md`](EMERGENCE_WIRING_PATCHPLAN.md).
>
> **Scope:** `crates/voxel/src/window/{mod,plan,ring_iter,io}.rs`,
> `crates/voxel/src/scale_budget.rs`,
> `crates/engine/src/{engine,emergence,emergence_metrics,replay,replay_format}.rs`.
>
> **Requirements addressed:**
> - `FR-CIV-SCALE-004` — Sim-LOD SHALL run full agent/CA fidelity near
>   interest and statistical gestalt far away without state divergence
>   explosions.
> - `FR-CIV-SCALE-005` — Prefetch SHALL queue disk chunk loads from
>   camera/sim velocity vectors.
> - `FR-CIV-SCALE-008` — Scale tests SHALL report max resident chunks
>   and P99 tick time on reference hardware without silent degradation.
>
> **Authority contracts (do not duplicate):**
> - `crates/voxel/src/window/mod.rs:103-118` — `SimCohort` enum.
> - `crates/voxel/src/window/mod.rs:256-268` — `WindowPolicy::sim_cohort`.
> - `crates/voxel/src/scale_budget.rs:668-890` — `SimLodAggregator`,
>   `CohortTotals`, `Gestalt` (the deterministic-fold contract for
>   far-ring stats).
> - `crates/voxel/src/window/plan.rs:144-180` — `prefetch_set`
>   (the velocity-driven prefetch planner).
> - `crates/agents/src/lib.rs:239-247` — `LodTier` (Hot/Warm/Cold
>   per-agent cadence, warm=4, cold=16 ticks).
> - `docs/adr/ADR-020-wire-dormant-emergence-phases.md` — the 11
>   dormant phases (`phase_life` … `phase_emergence`).
> - `docs/design/EMERGENCE_WIRING_PATCHPLAN.md` — per-phase
>   implementation; this doc specifies the **LOD partition** that
>   patches into it.

---

## 0. Thesis

The scale doctrine (`streaming-window.md` §3.3, `scale_budget.rs`
`SimCohort`) commits to a **three-cohort ring model**: `FullSim`
inside the inner mesh ring, `CoarseSim` in the middle ring,
`Frozen` past the coarse ring. That model is sound for **the voxel
CA** (mass conservation, diffusive agents), but `Simulation::tick`
runs **12 phases today and 23 phases after ADR-020 wires the 11
dormant emergence phases** — and the emergence phases have wildly
different cost profiles and data scopes. A blanket "this ring
cohort × this tick rate" rule is wrong:

- Some phases are **O(1) / O(factions)** and are immune to the
  cohort model — running them at full fidelity every tick is free.
- Some phases are **O(agents)** in the resident window — these
  are the ones the cohort model is *for*, and they need a
  per-cohort rule.
- Some phases touch **macro scalars that aggregate over the whole
  world** (belief, unrest, cohesion) — these read the per-cohort
  totals and produce a single global value, so the cohort
  *is* the input filter, not a tick-rate gate.
- One phase (`phase_emergence` orchestrator at
  `crates/engine/src/emergence.rs:159-171`) is the **heavy hitter**:
  it runs genetics, culture, social, psyche, legends, civ-ai in
  sequence, all O(agents). This is the only phase where the LOD
  partition is **load-bearing**.

The design: keep `SimCohort` as the ring-derived classifier
(no change), but layer a **per-phase cohort rule** on top that
says *which cohorts each phase touches, and how*. Macro phases
(1-10 in ADR-020) ignore the cohort model and read macro scalars
that are *derived from* the per-cohort agents. The orchestrator
(`phase_emergence`) is the only phase that **forks by cohort**:
inside the inner ring every sub-phase runs at full fidelity; in
the middle ring each sub-phase produces a per-cohort gestalt
folded by `SimLodAggregator`; in the outer ring every sub-phase
is skipped and the macro scalars carry only the gestalt summary.

---

## 1. Goals & non-goals

### Goals

- **HW-bounded sim cost.** Per-tick sim cost is a function of
  `(active_cohort_size, sim_ring, coarse_ring)`, not `(world_extent,
  agent_count)`. Adding world extent does not change the budget.
- **Three-tier fidelity per ring.** `FullSim` (ring ≤ `sim_ring`),
  `CoarseSim` (ring ≤ `coarse_ring`), `Frozen` (ring > `coarse_ring`)
  with a deterministic per-cohort rule for every emergence sub-phase.
- **Macro phases cheap and global.** The 10 macro phases in
  ADR-020 (life, research, tech, belief, unrest, cohesion,
  social_mood, stratification, institutions, economic_focus) are
  O(1) / O(factions). They run every tick at full fidelity — they
  read the per-cohort macro inputs but do not themselves cohort.
- **Gestalt without divergence.** A far-ring `CoarseSim` chunk
  contributes its `CohortTotals` (mass, agent count) to the
  `SimLodAggregator`; the orchestrator's totals match the sum
  over cohort totals within `f32` summation error (per
  `SimLodAggregator::mass_divergence_bound`, `scale_budget.rs:882-889`).
- **Prefetch warms the rings before the camera arrives.** Velocity
  vectors (camera + sim-interest anchor) drive `prefetch_set` so
  chunks warm into the inner ring before the camera crosses into
  them. Two cones are unioned.
- **Replay-safe.** Two clients with the same seed, anchor trace,
  and policy produce the same per-cohort assignment, the same
  per-tick `should_tick_phase_for_chunk` decisions, and the same
  gestalt bit-pattern. Replay reads `(camera_anchor, sim_interest,
  tick, policy)` and reconstructs the ring layout bit-identically.
- **Pluggable per-phase rules.** The default rule table is in
  `WindowPolicy::phase_cohort_rule(phase_id)`. Custom scenarios
  (e.g. "debug mode = full fidelity everywhere") override the
  table without touching the per-phase call sites.

### Non-goals (this slice)

- **Per-agent LOD assignment.** `civ-agents::LodTier`
  (`crates/agents/src/lib.rs:239-247`) already exists and drives
  per-civilian prop cadence (wardrobe/tools). This doc treats it
  as the **derived per-agent view** of the chunk-level cohort:
  a `CoarseSim` chunk's agents default to `Cold` (16-tick cadence),
  a `FullSim` chunk's agents default to `Hot`. A future slice may
  decouple them (e.g. a Hot agent in a CoarseSim chunk for a
  selected character); out of scope here.
- **Multi-camera / split-screen.** A single anchor (with optional
  sim-interest secondary anchor). Multi-anchor is a follow-up
  (already noted as `WindowPolicy::anchors()` in `streaming-window.md`
  §7).
- **The actual mesh-blend shader.** Already deferred to the renderer
  in `streaming-window.md` §2.
- **Save-format changes for cohort state.** The existing
  `MaterializedSnapshot` (`window/io.rs:114-202`) records the policy
  + IoContract list; the cohort rule is part of the policy, so the
  save format needs **no** change. A reload with a different
  `WindowPolicy` would produce a different cohort layout — that is
  expected and not a save-format regression.

---

## 2. The three sim cohorts (recap)

`WindowPolicy::sim_cohort(coord, anchor)` returns one of three
variants (defined at `crates/voxel/src/window/mod.rs:103-118`):

| Ring distance | Cohort       | Tick behaviour (default policy)              |
|---------------|--------------|----------------------------------------------|
| `≤ sim_ring`  | `FullSim`    | Every tick, per-voxel CA + per-agent ECS     |
| `≤ coarse_ring` | `CoarseSim` | Every `sim_lod_step`-th tick (default = 2), per-cohort gestalt only |
| `> coarse_ring` | `Frozen`   | No sim tick; mass conserved by construction (no writes, no decay) |

Defaults (`WindowPolicy::default()` at `crates/voxel/src/window/mod.rs:162-181`):
`sim_ring = 1, coarse_ring = 2, sim_lod_step = 2, seam_chunks = 1`.
Inner mesh band is `(2*sim_ring + 1)³ = 27` chunks worst case;
middle band is `(2*coarse_ring + 1)³ − 27 = 98` chunks; outer
band is unbounded (gated only by `ExtentBudget::Unbounded`,
`scale_budget.rs:285-302`).

The 3-cohort rule is the **chunk-level substrate**. The design
below layers a per-phase rule on top.

---

## 3. Per-phase cohort rules

The 23 phases after ADR-020's wiring split into three buckets
based on their cost profile and data scope:

### 3.1 Macro phases (10 phases) — always full-fidelity, O(1)/O(factions)

These phases are scalar or small-set aggregations. Their cost is
bounded by the number of factions (typically 2-10) or is constant.
Running them at every tick is cheap and **they need to run at
full fidelity** because their outputs (`belief`, `unrest`,
`cohesion`, `society_mood`, `dispossessed_permille`,
`temple_level`, `garrison_level`, `economic_focus`,
`research_progress`, `tech_unlocks`) are the macro state the
cohort gestalt reads. If we ran them on a coarse cadence, the
gestalt would carry stale macro state.

| # | Phase | Reads | Writes | Cohort rule |
|---|-------|-------|--------|-------------|
| 1 | `phase_life` | `state.population`, `ClusterMember` (post-`citizen_lifecycle`), ECS | `cluster_stocks`, `last_settlement_count`, `last_life_deaths` | **Always runs**, every tick. Cluster commit is global; per-agent cost is O(1) per cluster (reconcile membership). |
| 2 | `phase_research` | `population`, `belief`, `cohesion`, `economy_state` | `state.research_progress` | **Always runs**, every tick. O(1). |
| 3 | `phase_tech` | `state.research_progress` | `state.tech_unlocks` (bitmask) | **Always runs**, every tick. O(1). |
| 4 | `phase_belief` | `last_sentience` (post-`phase_emergence`), `unrest`, `population`, disasters | `state.belief` via `add_belief` | **Always runs**, every tick. O(1). |
| 5 | `phase_unrest` | `food_price`, `energy_budget_joules`, mean `-Psyche.mood.valence` | `state.unrest` | **Always runs**, every tick. O(factions). |
| 6 | `phase_cohesion` | `state.belief`, `state.unrest`, `avg_faction_kinship` | `state.cohesion` | **Always runs**, every tick. O(factions). |
| 7 | `phase_social_mood` | mean `Psyche.mood.valence`, `state.cohesion` | `state.society_mood` | **Always runs**, every tick. O(factions). |
| 8 | `phase_stratification` | treasury spread, `cohesion`, `unrest`, `economic_focus` | `state.dispossessed_permille` | **Always runs**, every tick. O(factions). |
| 9 | `phase_institutions` | `belief`, `unrest`, `dispossessed_permille`, `population` | `state.temple_level`, `state.garrison_level` | **Always runs**, every tick. O(1). |
| 10 | `phase_economic_focus` | `state.resources.food`, `research_tier()`, `belief`, treasury | `state.economic_focus` | **Always runs**, every tick. O(1). |

**Why no LOD:** these phases' costs are O(1) or O(factions), not
O(agents). They produce macro scalars that the cohort gestalt
*reads* — they don't read the world scan themselves. The only
phase in this group that has a non-trivial data dependency on
agents is `phase_unrest` (mean `Psyche.mood.valence`) and
`phase_social_mood` (mean valence/arousal). Those reads are
**over the per-cohort gestalts**, not the per-agent scan:
`phase_emergence`'s orchestrator produces one `CohortTotals` per
cohort per tick, and the macro phases sum them with
`SimLodAggregator::fold` (`scale_budget.rs:822-835`).

### 3.2 The orchestrator — `phase_emergence` (1 phase) — cohort-forked

`phase_emergence` (`crates/engine/src/emergence.rs:159-171`) is the
only phase whose cost scales with the **resident agent set**. It
runs 7 sub-phases in sequence:

```text
phase_emergence:
    1. emergence_ensure_genomes   (O(agents) — attach Dna if missing)
    2. emergence_culture          (O(clusters) — drift_populations)
    3. emergence_social           (O(agents) — apply social events)
    4. emergence_psyche           (O(agents) — update_mood / beliefs)
    5. emergence_genetics_sentience (O(agents) — evaluate_sentience)
    6. emergence_legends          (O(events) — saga graph ingest)
    7. emergence_civ_ai           (O(events) — name promotion)
```

Sub-phases 1, 3, 4, 5 are O(agents); 2 is O(clusters); 6, 7 are
O(events). The orchestrator's per-phase cohort rule partitions
the agent set by ring and runs each sub-phase over the relevant
cohort only:

| Sub-phase | `FullSim` (inner ring) | `CoarseSim` (middle ring) | `Frozen` (outer ring) |
|-----------|------------------------|---------------------------|------------------------|
| `emergence_ensure_genomes` | every agent, every tick | **skipped** (already cached on FullSim pass) | **skipped** |
| `emergence_culture` | full `drift_populations` over resident clusters | **aggregated gestalt** per cluster: drift a reduced representation (centroid + variance) at 1/`sim_lod_step` cadence | **skipped** — frozen cluster profiles carry over from last warm tick |
| `emergence_social` | per-agent `apply_social_event` every tick | per-cluster edge count + mean affinity only, every `sim_lod_step`-th tick; no per-agent event log | **skipped** |
| `emergence_psyche` | per-agent `update_mood` / `update_beliefs` / `nudge_temperament` every tick | **gestalt only**: mean valence, arousal, belief distribution per cluster at 1/`sim_lod_step` cadence | **skipped** |
| `emergence_genetics_sentience` | per-agent `evaluate_sentience` every tick | **edge-of-threshold check only**: agents near the sentience threshold are *promoted* to FullSim cohort for one tick so the sentience event is not lost (cohort escalation, see §3.3) | **skipped** (no per-agent check) |
| `emergence_legends` | birth/death/sentience events from the FullSim ring feed the saga graph | **birth/death rollups** (counts + cluster ids) at 1/`sim_lod_step` cadence feed the saga graph as `RawSimEvent` with reduced salience | **skipped** — last known gestalt carries |
| `emergence_civ_ai` | `civ_ai_sync_generate` for promotions in the FullSim ring | **skipped** — civ-ai names are only minted for FullSim-cohort events | **skipped** |

The partition is **deterministic** — it is a function of
`(coord, anchor, policy, tick)`. The orchestrator's entry point
becomes:

```rust
pub(crate) fn phase_emergence(&mut self) {
    self.emergence.last_feed.clear();
    self.emergence.last_ai_decisions.clear();
    self.emergence.last_sentience.clear();

    // Collect per-cohort agent lists in one pass (O(agents) once).
    let (full_sim_agents, coarse_sim_agents, frozen_agents) =
        self.partition_agents_by_sim_cohort();

    // Sub-phases 1, 3, 4, 5 run on the FullSim cohort at full fidelity.
    self.emergence_ensure_genomes(&full_sim_agents);
    self.emergence_culture_full(&full_sim_agents);
    self.emergence_social_full(&full_sim_agents);
    self.emergence_psyche_full(&full_sim_agents);
    self.emergence_genetics_sentience_full(&full_sim_agents);

    // Sub-phases 2, 3, 4 also produce per-cohort gestalt on the
    // coarse cohort, on the sim_lod_step cadence.
    if self.state.tick % self.lod_policy.sim_lod_step as u64 == 0 {
        self.emergence_culture_gestalt(&coarse_sim_agents);
        self.emergence_social_gestalt(&coarse_sim_agents);
        self.emergence_psyche_gestalt(&coarse_sim_agents);
    }

    // Sub-phases 6, 7 read the events from BOTH cohorts (FullSim +
    // CoarseSim gestalt) but only mint saga nodes / civ-ai names for
    // FullSim events.
    self.emergence_legends();   // unchanged signature; reads self.last_births/deaths
                               //   + a new `self.emergence.last_coarse_events`
                               //   from the gestalt pass.
    self.emergence_civ_ai();

    // Frozen cohort contributes nothing — mass conservation is
    // automatic (no writes).
}
```

The orchestrator's per-tick cost is now
`O(full_sim_agents) + O(coarse_sim_agents / sim_lod_step)` —
which is `O(hot_cohort)` regardless of world extent.

### 3.3 Cohort escalation — sentience threshold crossing

A subtle case: `emergence_genetics_sentience` checks each agent
against the sentience threshold. If we run it only on the FullSim
ring, an agent in the CoarseSim ring that crosses the threshold
on the coarse-cadence tick would **miss the crossing** (it runs
once every 4 ticks on the coarse cohort) and the legends graph
would miss the `SpeciationEvent` for that lineage.

**Rule:** when the coarse-cohort `evaluate_sentience` pass detects
an agent at `cognition_score ∈ [threshold - ε, threshold + ε]`
(where `ε = 0.05`, the sentience threshold tolerance), the agent
is **escalated** to the FullSim cohort for **one tick** — its
next per-agent `evaluate_sentience` runs at full fidelity on the
next tick, and the sentience event is captured if it crosses.

The escalation is recorded as a `CohortEscalation { agent_id,
from_cohort: CoarseSim, to_cohort: FullSim, ticks: 1 }` entry on
the per-agent view (a new optional component on
`Civilian` — `CohortOverride`). The orchestrator's
`partition_agents_by_sim_cohort` consults the override list before
the ring-derived cohort, so the escalated agent appears in
`full_sim_agents` for one tick.

Cost: the override list is small (O(escalations-per-tick), bounded
by `MAX_COHORT_ESCALATIONS_PER_TICK = 16`); the per-tick partition
cost is unaffected. Replay determinism is preserved because the
escalation list is a function of `(cohort_totals, tick, seed)`.

### 3.4 Frozen cohort — mass conservation

The `Frozen` cohort (ring > `coarse_ring`) carries **no sim tick**:
no CA diffusion, no agent evolution, no decay. Mass is conserved
by construction because no writes happen — `last_cohort_stats`
(`crates/engine/src/engine.rs:415, :651, :1005, :4027-4036`) for
the frozen cohort is the `CohortTotals` snapshot from the last
warm tick, and `SimLodAggregator` folds it into the gestalt on
the coarse-cadence tick.

**What this means for replay:** the gestalt at tick `T` for a
frozen chunk is **identical** to the gestalt at tick `T-1` (no
change, no decay). The replay bus carries `FrozenChunkStaleness`
events for HUD inspection ("this chunk has been frozen for 4,096
ticks") but the sim state is unchanged.

---

## 4. Prefetch model — velocity-driven, two-cone

The prefetch planner (`crates/voxel/src/window/plan.rs:144-180`)
takes a `VelocityChunksPerTick` and a `WindowPolicy` and returns
the set of chunk coords the streaming layer should warm over the
next `ticks` ticks. The default is a single anchor (the camera).

**Two-cone extension:** the camera is not the only sim-interest
source. A sim-interest anchor (e.g. a selected agent, a faction
HQ, the diplomacy target) drives a **second prefetch cone** that
is unioned with the camera cone.

```rust
pub struct SimInterest {
    /// Anchor chunk for the sim-interest cone (e.g. selected agent).
    pub anchor: ChunkCoord,
    /// Velocity vector for the sim-interest anchor.
    pub velocity: VelocityChunksPerTick,
    /// Forward-cone threshold (Q0.7), same shape as WindowPolicy's.
    pub forward_cone_cos_theta: i8,
}

pub fn prefetch_set_with_interest(
    camera_anchor: ChunkCoord,
    camera_velocity: VelocityChunksPerTick,
    interest: Option<SimInterest>,
    policy: &WindowPolicy,
    ticks: u32,
) -> Vec<ChunkCoord> {
    let mut set = prefetch_set(camera_anchor, camera_velocity, policy, ticks);
    if let Some(interest) = interest {
        let interest_set = prefetch_set(
            interest.anchor,
            interest.velocity,
            policy,
            ticks,
        );
        // Union; the streaming layer dedupes by ChunkCoord.
        set.extend(interest_set);
        set.sort_unstable_by_key(|c| (c.cx, c.cy, c.cz));
        set.dedup();
    }
    set
}
```

**Cone coalescing rule:** if a chunk is in **both** cones, the
streaming layer prefers the **outer** entry — i.e. the entry
with the larger `mesh_ring + prefetch_ring` radius. This avoids
duplicated load requests and lets the **furthest-out** prefetch
warm the path between the two anchors (a common case: the camera
is moving toward a faction HQ the player has selected, and the
chunks between them are in both cones).

**Sim-interest sources:**

1. **Selected agent.** The agent's `CohortOverride` is the
   `Hot` tier (full fidelity), and its `Position3d` is the
   `SimInterest::anchor`. The agent's velocity is derived from
   the last two `Position3d` samples (or zero if stationary).
2. **Faction HQ.** The faction's capital chunk is the anchor;
   velocity is zero (faction HQs don't move — but the camera
   moving toward them produces a non-zero effective velocity in
   the camera cone, so the prefetch still fires).
3. **Diplomacy target.** Same as faction HQ for the active
   diplomacy engagement (trade route, war).

**Velocity-driven prefetch ticks:** the default is
`DEFAULT_PREFETCH_TICKS = 4` (`window/plan.rs:35`). This matches
`sim_lod_step = 2` × 2 — a chunk the camera is approaching in
2-4 ticks is exactly the band the coarse sim cohort is about to
read, so warming it ahead of the camera keeps the cohort gestalt
fresh.

**Empty-set conditions** (no prefetch needed, in priority order):

1. `prefetch_ring == 0` — policy disables prefetch entirely.
2. `camera_velocity.is_zero() && interest_velocity.is_zero()` —
   neither anchor is moving.
3. `ticks == 0` — caller wants no prefetch.
4. The desired set is fully inside the current inner ring
   (already warm).

---

## 5. The `PhaseCohortRule` table — composition with `LodTier`

The per-phase rule table is a small `const fn` lookup at
`WindowPolicy::phase_cohort_rule(phase_id) -> PhaseCohortRule`.
This is the **single source of truth** for "which cohorts does
this phase touch?".

```rust
/// FR-CIV-SCALE-004 — the per-phase cohort rule. Each phase is
/// tagged with one of three rules: Always (macro phases 1-10,
/// O(1)/O(factions)), Forked (`phase_emergence` orchestrator only),
/// or Inherited (a future phase that delegates to a parent rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhaseCohortRule {
    /// Run every tick, full fidelity, no cohort gate.
    Always,
    /// `phase_emergence` orchestrator: fork by cohort.
    Forked,
}

impl WindowPolicy {
    pub const fn phase_cohort_rule(&self, phase_id: PhaseId) -> PhaseCohortRule {
        match phase_id {
            PhaseId::Life             => PhaseCohortRule::Always,
            PhaseId::Research         => PhaseCohortRule::Always,
            PhaseId::Tech             => PhaseCohortRule::Always,
            PhaseId::Belief           => PhaseCohortRule::Always,
            PhaseId::Unrest           => PhaseCohortRule::Always,
            PhaseId::Cohesion         => PhaseCohortRule::Always,
            PhaseId::SocialMood       => PhaseCohortRule::Always,
            PhaseId::Stratification   => PhaseCohortRule::Always,
            PhaseId::Institutions     => PhaseCohortRule::Always,
            PhaseId::EconomicFocus    => PhaseCohortRule::Always,
            PhaseId::Emergence        => PhaseCohortRule::Forked,
            // Production, citizen_lifecycle, military, policy, economy,
            // planet, diplomacy, tactics, voxel, compact, buildings,
            // diffusion — the original 12 phases — are Always (they
            // either don't touch agents or are O(1)).
            _ => PhaseCohortRule::Always,
        }
    }
}
```

**Composition with `LodTier`:** a chunk's cohort (`SimCohort`)
translates to a default `LodTier` for the agents in that chunk:

| Sim cohort     | Default `LodTier` | Cadence |
|----------------|-------------------|---------|
| `FullSim`      | `Hot`             | every tick (1) |
| `CoarseSim`    | `Cold`            | every 16 ticks |
| `Frozen`       | `Hot` (no work happens, but the prop is consistent — see note) | n/a |

**Note on Frozen tier:** the `LodTier` for frozen agents is
**moot** — no per-agent phase runs on them, so the cadence
doesn't matter. The default mapping table sets it to `Hot` to
keep the `LodTier` field consistent with the cohort (a frozen
agent's `LodTier` is "Hot" in name, but no `should_tick_entity`
call ever fires for it). This avoids a separate `Frozen` variant
on `LodTier` (which would be a breaking change to
`civ-agents::LodTier`'s public API).

**Override path:** an agent's `CohortOverride` (one of:
`AlwaysHot`, `AlwaysCold`, `EscalatedForOneTick`) takes precedence
over the cohort-derived `LodTier`. The escalation list for
sentience threshold crossings is a special case of `EscalatedForOneTick`.

---

## 6. Per-tick orchestration — `Simulation::tick` integration

The existing `PHASE_ORDER` at `crates/engine/src/engine.rs:55-68`
lists 12 phases today; after ADR-020 wiring it lists 23. The LOD
partition sits **inside** `Simulation::tick` and is keyed off
`WindowPolicy::phase_cohort_rule(phase_id)`:

```rust
fn tick(&mut self) {
    self.state.tick = self.state.tick.wrapping_add(1);
    let t = self.state.tick;

    // Phases 1-12 (the original loop): unchanged. The macro
    // phases that ADR-020 adds (life, research, ..., economic_focus)
    // are also unchanged — they all carry PhaseCohortRule::Always.

    // The orchestrator: PhaseCohortRule::Forked.
    if self.window_policy.phase_cohort_rule(PhaseId::Emergence)
        == PhaseCohortRule::Forked
    {
        // The partition by cohort happens INSIDE phase_emergence.
        // (No change to PHASE_ORDER; the orchestrator owns the fork.)
        self.phase_emergence();
    } else {
        // Debug / mod-dev: always full fidelity, no fork.
        self.phase_emergence_full();
    }

    // Diffusion: always runs (it's the deterministic core tail).
}
```

The orchestrator's partition by cohort is the **only place** in
the tick loop that reads the chunk-level cohort. The 22 other
phases are cohort-agnostic.

**Tick-budget implications:**

- The orchestrator's per-tick cost is now bounded by
  `O(full_sim_agents) + O(coarse_sim_agents / sim_lod_step)`.
  With defaults (`sim_ring=1, coarse_ring=2, sim_lod_step=2`),
  this is `~27 chunks × hot_agents + ~98 chunks × cold_agents / 2`
  — roughly `~80%` of the full-fidelity cost in the worst case
  (a 27-chunk hot cube + a 98-chunk cold cube). In practice the
  coarse-cadence coarse-cohort pass is ~`1/8` of the full pass
  (gestalt only, no per-agent mutation), so the total is closer
  to `~35%` of the full-fidelity cost.
- The 10 macro phases cost `~O(1)` each — adding ~`1 µs` per
  phase on reference hardware, dominated by the orchestrator's
  `~ms`-scale cost.
- The `ScaleReport` (`window/plan.rs:202-285`) tracks the
  P99 tick time and `max_resident_chunks`. The orchestrator's
  per-tick partition cost is reported as
  `last_emergence_partition_us` (new field on `ScaleReport`) so
  the perf HUD can distinguish "orchestrator hot path cost" from
  "macro phase cost".

---

## 7. Determinism contract

Two clients with the same `(seed, anchor_trace, policy)` MUST
produce the same per-cohort assignment, the same
`should_tick_phase_for_chunk` decisions, and the same gestalt
bit-pattern.

**Per-cohort assignment:** a pure function of
`(coord, anchor, policy.vy_weight, policy.sim_ring, policy.coarse_ring)`.
`WindowPolicy::sim_cohort` (`crates/voxel/src/window/mod.rs:256-268`)
is `const fn` and pure. Translation-invariance is asserted by
`fr_civ_scale_002_no_fixed_world_cap_allows_arbitrary_anchor_coords`
(`window/mod.rs:702-726`).

**Per-tick partition:** the orchestrator's `partition_agents_by_sim_cohort`
is `O(agents)` once per tick, with a deterministic agent sort key
(`(CohortOverride, ring, agent_id)`). The escalation list is
derived from `(cohort_totals, tick, seed)` — the same seed
produces the same escalation list, so replay reconstructs the
FullSim cohort bit-identically.

**Gestalt fold:** `SimLodAggregator::fold`
(`crates/voxel/src/scale_budget.rs:822-835`) canonicalises input
order by `(mass, agents, chunks)` so two clients with the same
input set in different per-cohort order produce the same gestalt
bit-pattern. `mass_divergence_bound` (`scale_budget.rs:882-889`)
bounds the `f32` summation error at
`len * f32::EPSILON * max_mass`.

**Replay bus:** `Simulation::last_emergence_partition_us` and the
per-cohort gestalt summary are added to the replay log
(`crates/engine/src/replay.rs`) so a `.civreplay` carries the
cohort assignment + gestalt bit-pattern. The replay reader
asserts the gestalt against the per-cohort totals at the end of
each tick; mismatch is a determinism regression.

---

## 8. Data shapes (slice)

```rust
// crates/voxel/src/window/mod.rs (additions)

/// Per-phase cohort rule (FR-CIV-SCALE-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhaseCohortRule {
    /// Run every tick at full fidelity (no cohort gate).
    Always,
    /// Fork by cohort (the orchestrator only).
    Forked,
}

/// Stable identifiers for the 23 phases. Used by
/// [`WindowPolicy::phase_cohort_rule`]. The mapping from
/// `PhaseId` to the actual `Simulation::phase_*` method is the
/// engine's `PHASE_ORDER` (single source of truth — see
/// `crates/engine/src/engine.rs:55-68`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PhaseId {
    Production,
    CitizenLifecycle,
    Military,
    Policy,
    Economy,
    Planet,
    Diplomacy,
    Tactics,
    Voxel,
    Compact,
    Buildings,
    Life,
    Research,
    Tech,
    Belief,
    Unrest,
    Cohesion,
    SocialMood,
    Stratification,
    Institutions,
    EconomicFocus,
    Emergence,
    Diffusion,
}

impl WindowPolicy {
    /// FR-CIV-SCALE-004 — return the cohort rule for `phase_id`.
    /// The default table is the spec in §3 of this doc; custom
    /// policies can override via `WindowPolicy::with_phase_rule`.
    pub const fn phase_cohort_rule(&self, phase_id: PhaseId) -> PhaseCohortRule {
        match phase_id {
            PhaseId::Emergence => PhaseCohortRule::Forked,
            // All other phases (including the 10 macro phases ADR-020
            // adds) carry the default `Always` rule.
            _ => PhaseCohortRule::Always,
        }
    }
}
```

```rust
// crates/voxel/src/window/plan.rs (additions)

/// Sim-interest anchor for the second prefetch cone. See
/// `docs/design/SIM_LOD_COHORTS.md` §4 for the union / coalesce rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SimInterest {
    pub anchor: ChunkCoord,
    pub velocity: VelocityChunksPerTick,
    pub forward_cone_cos_theta: i8,
}

/// Compute the prefetch set with a second sim-interest cone unioned
/// with the camera cone.
pub fn prefetch_set_with_interest(
    camera_anchor: ChunkCoord,
    camera_velocity: VelocityChunksPerTick,
    interest: Option<SimInterest>,
    policy: &WindowPolicy,
    ticks: u32,
) -> Vec<ChunkCoord> { /* ... */ }
```

```rust
// crates/engine/src/emergence.rs (additions)

/// Per-tick cohort assignment for the orchestrator. The `Hot` and
/// `Cold` slices are the agent lists; the gestalt is the
/// per-cohort fold. Carried on the replay bus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmergenceCohortSplit {
    pub tick: u64,
    pub full_sim_agents: u32,
    pub coarse_sim_agents: u32,
    pub frozen_agents: u32,
    pub cohort_gestalt: Gestalt,
    pub escalation_count: u32,
}
```

The full types land in `crates/voxel/src/window/mod.rs`,
`crates/voxel/src/window/plan.rs`, and
`crates/engine/src/emergence.rs` in a follow-up slice. This doc
is the design — the implementation slice is gated on ADR-020
wiring landing first.

---

## 9. Open questions (follow-up slices)

1. **Cohort escalation for non-sentience triggers.** Sentience is
   the only trigger today. A future slice may escalate for
   "near-death" (a `CohortEscalation::LifeThreat` if a faction's
   mean `Needs.safety` drops below 0.2), "near-promotion" (a
   culture trait near the drift threshold), or "near-war" (a
   `DiplomacyEvent::Conflict` 5 ticks out). The same one-tick
   override rule applies.
2. **Coarse-cadence gestalt write-through.** The coarse cohort's
   per-tick gestalt is currently computed **only** on the
   `tick % sim_lod_step == 0` boundary. A future slice may
   write-through the partial gestalt on the off-tick (a
   "running gestalt" that interpolates) for HUD responsiveness.
3. **Per-faction cohort.** The default policy treats all factions
   uniformly. A future slice may override per-faction — e.g. the
   player faction is always `FullSim` regardless of ring, and AI
   factions follow the default rule. The `WindowPolicy` would
   carry a `faction_overrides: HashMap<u32, SimCohort>` field.
4. **Multi-anchor.** `streaming-window.md` §7 calls out
   split-screen / multi-camera. The sim-interest anchor is the
   single-anchor specialisation; a multi-anchor slice would
   union N cones (not just 2).
5. **Cohort-aware save.** A save under policy A with
   `coarse_ring = 2` has gestalts for rings 1-2. A reload under
   policy B with `coarse_ring = 4` would expect gestalts for
   rings 1-4 — the new rings' gestalts would be empty until the
   first tick. A future slice may carry a "minimum cohort size"
   hint in the `MaterializedSnapshot` so a reload under a wider
   policy degrades gracefully.

---

## 10. Cross-references

- `docs/design/streaming-window.md` — the rings, prefetch cones,
  and sim-cohort taxonomy this doc layers on top of.
- `docs/design/EMERGENCE_WIRING_PATCHPLAN.md` — the per-phase
  implementation plan; the LOD partition is the "LOD fork" in
  the orchestrator's pseudocode.
- `docs/design/EMERGENCE_TESTS_PLAN.md` — the 57-test plan that
  proves the 11 phases *emerge*. The cohort-gated tests are a
  subset of that plan: each sub-phase gets a "FullSim → gestalt
  → Frozen" assertion that proves the LOD partition is wired.
- `docs/adr/ADR-020-wire-dormant-emergence-phases.md` — the
  per-phase DAG and the 23-phase `PHASE_ORDER`.
- `docs/specs/requirements/NFR-CIV-SCALE-PERF.md` — the perf
  budget this design must meet on reference hardware.
- `crates/voxel/src/window/mod.rs:103-268` — `SimCohort`,
  `WindowPolicy::sim_cohort`.
- `crates/voxel/src/window/plan.rs:144-180` — `prefetch_set`.
- `crates/voxel/src/scale_budget.rs:668-890` — `SimLodAggregator`,
  `CohortTotals`, `Gestalt`.
- `crates/agents/src/lib.rs:239-247` — `LodTier` (Hot/Warm/Cold
  per-agent cadence, derived from the cohort rule).
- `crates/engine/src/emergence.rs:159-171` — `phase_emergence`
  orchestrator (the fork site).
- `crates/engine/src/engine.rs:55-68` — `PHASE_ORDER` (the
  ordered phase list).