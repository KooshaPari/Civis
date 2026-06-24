# ADR-020: Wire Dormant Emergence Phases Into `Simulation::tick`

## Status

Proposed

## Date

2026-06-23

## Context

The 2026-06-23 emergence audit (`docs/reports/EMERGENCE_AUDIT.md`) found
that `Simulation::tick` (`crates/engine/src/engine.rs:1172`) executes **12
phases** in `PHASE_ORDER` (`engine.rs:55-68`) and that **11 emergent-phase
methods** are **dead outside their own `#[cfg(test)]` modules**:

- `phase_emergence` — orchestrator at `crates/engine/src/emergence.rs:159`;
  only callers are its own unit tests (`emergence.rs:731-1098`).
- `phase_research`, `phase_tech`, `phase_belief`, `phase_unrest`,
  `phase_cohesion`, `phase_social_mood`, `phase_stratification`,
  `phase_institutions`, `phase_economic_focus`, `phase_life` — **not
  defined as methods on `Simulation`** (`fn phase_research` /
  `fn phase_tech` / etc. yield zero hits across `crates/engine/`).

The macro scalar layer is consequently unread-on-write: `state.belief`
exists as `Simulation::belief` (`engine.rs:422`, `:651`, `:717`) and is
mutated only by `add_belief` / `try_invoke_divine_power` from `disasters.rs`
(also dead) and `apply_awakening_coupling` (`emergence.rs:539`, itself
behind the dead `phase_emergence`). `state.unrest`, `state.cohesion`,
`state.dispossessed_permille`, `state.economic_focus`, `state.temple_level`,
`state.garrison_level`, `state.society_mood` are read by coupling
functions (`diplomacy_conflict_threshold`, `unrest_trade_factor`,
`cohesion_trade_factor`, `dispossession_unrest`, `building_demand_signals`,
`society_trade_factor`, `civ_diplomacy::Relation` consumers) but have **no
producer** in the 12-phase loop. Charter gap (ADR-emergence-charter:
"Hardcode only physics/genomic law; everything else emerges with
bidirectional coupling.").

This ADR records the decision to **wire each dormant phase into
`Simulation::tick`** as a new ordered tail inserted after `phase_buildings`
and before `phase_diffusion`. It does **not** record the per-phase
implementation (those are engine PRs, not an ADR); it records the
**phase order** — the dependency DAG between phases, the shared physics-sim
fields each phase reads / writes per the physics-coupling doctrine
(ADR-011 + ADR-018), the perf / feedback-explosion risk, and the
deterministic guard that keeps the system on the edge of chaos.

## Decision

### 1. Extend `PHASE_ORDER` with 11 entries and append `phase_emergence` as the final phase

`crates/engine/src/engine.rs:55-68` `PHASE_ORDER` becomes:

```text
production, citizen_lifecycle, military, policy, economy, planet,
diplomacy, tactics, voxel, compact, buildings,
life, research, tech, belief, unrest, cohesion, social_mood,
stratification, institutions, economic_focus,
emergence,
diffusion
```

(`diffusion` retains its position at the tail of the deterministic core
loop; `emergence` is inserted between `economic_focus` and `diffusion` so
the diffusion phase — which propagates `Wardrobe.era` / `Tools.era` — runs
against the post-emergence psyche and culture state. The existing
test `phase_order_includes_emergence` at `engine.rs:3320` already asserts
`emergence_idx == PHASE_ORDER.len() - 1`, so that test will be amended to
allow `emergence` to be the **penultimate** entry with `diffusion` last
— the existing assertion is amended, not deleted, to preserve the
emergence-as-final-emergent-phase intent.)

`Simulation::tick` (`engine.rs:1179-1192`) calls each new phase in the same
order. The matching `phase_order_matches_tick_sequence` test
(`engine.rs:3294`) is updated to reflect the new sequence.

### 2. Recommended phase order — the dependency DAG

The order is dictated by the **physics-sim fields** each phase reads and
writes; the DAG is the strict topological sort of those edges. Lower
phases write the fields higher phases read.

| # | Phase | Reads (from earlier phases) | Writes (consumed by later phases) |
|---|-------|-----------------------------|------------------------------------|
| 1 | `phase_life` | `state.population`, `ClusterMember` (post-`citizen_lifecycle`), `world` ECS | `cluster_stocks` (settlement commons), `last_settlement_count`, `last_life_deaths`; clusters with ≥ 2 members are committed. The existing `cluster_by_colocation` / `reconcile_membership` (`crates/agents/src/cluster.rs`) are wired here. |
| 2 | `phase_research` | `population`, `belief`, `cohesion` (from later phases — see "Note on belief/cohesion latency" below), `economy_state` (research funding) | `state.research_progress` (`u64`, `saturating_add`, capped by `MAX_RESEARCH_PER_TICK`) |
| 3 | `phase_tech` | `state.research_progress` | `state.tech_unlocks: u64` (bitmask of `TECH_IRRIGATION` … `TECH_GUNPOWDER` from `engine.rs:1947-1976`); writes through `tech_unlocks_for_tier` so `carrying_capacity` and `building_cadence` re-derive correctly |
| 4 | `phase_belief` | `last_sentience` (post-`phase_emergence`'s sentience sub-phase), `unrest` (read prior-tick stale-allowed), `population`, disasters (if `phase_disasters` is wired later) | `state.belief: u64` via `add_belief` (`saturating_add`); `MAX_AWAKENING_BELIEF_PER_TICK` cap; `MAX_BELIEF_PER_TICK` macro cap |
| 5 | `phase_unrest` | `food_price` (post-`phase_economy`), `energy_budget_joules` (post-`phase_economy`), mean `-Psyche.mood.valence` (post-`phase_emergence`), `non-food prices` (post-`phase_economy`) | `state.unrest: u64`; uses `unrest_delta`, `commodity_unrest_delta`, `energy_scarcity_unrest`, `agent_misery_unrest` (engine.rs:2000-2089), bounded by `MAX_RISE`/`DECAY` per leg |
| 6 | `phase_cohesion` | `state.belief` (post-`phase_belief`), `state.unrest` (post-`phase_unrest`), `avg_faction_kinship` (post-`phase_emergence` SocialGraph), `micro_cohesion_delta` from Psyche | `state.cohesion: u64`; uses `cohesion_delta`, `micro_cohesion_delta` (engine.rs:2093, :2630), `awakening_cohesion_gain` (already in `emergence.rs` via `apply_awakening_coupling`); bounded by `MICRO_BIND_CAP=12` / `MICRO_FRAY_CAP=18` |
| 7 | `phase_social_mood` | mean `Psyche.mood.valence` and `.arousal` (post-`phase_emergence`), `state.cohesion` | `state.society_mood: f32` (mean valence/arousal summary); bounded `[-1, 1]`; clamped per tick |
| 8 | `phase_stratification` | `treasury_total` spread across `state.faction_treasury` (post-`phase_economy`), `state.cohesion`, `state.unrest`, `state.economic_focus` (must be settled — but this is a tension, see "Alternative considered") | `state.dispossessed_permille: u64` via `dispossession_target_permille` (`engine.rs:2408`); bounded `0..1000` permille |
| 9 | `phase_institutions` | `state.belief`, `state.unrest`, `state.dispossessed_permille`, `population` | `state.temple_level: u32`, `state.garrison_level: u32`; per-tick `+1` when belief ≥ threshold / unrest ≥ threshold, with `MAX_INSTITUTION_RISE_PER_TICK` cap; decays one step toward zero otherwise |
| 10 | `phase_economic_focus` | `state.resources.food` (post-`phase_economy`), `research_tier()` (post-`phase_tech`), `state.belief`, treasury totals | `state.economic_focus: EconomicFocus` via `candidate_economic_focus` (`engine.rs:2273`); Agrarian / Industrial / Sacred / Mercantile / Balanced |
| 11 | `phase_emergence` | **all macro scalars** (`belief`, `unrest`, `cohesion`, `economic_focus`, `dispossessed_permille`, `temple_level`, `garrison_level`, `society_mood`), all ECS agent state (`Civilian`, `ClusterMember`, `Dna`, `Psyche`, `SocialGraph`, `Needs`, `LifeNeeds`), `cluster_cultures`, `cluster_stocks`, `research_cache` | `cluster_cultures`, `emergence.last_feed`, `emergence.last_sentience`, `emergence.last_ai_decisions`, `emergence.legends` (saga graph), `faction_aggression` rebuild, `cohesion` & `belief` pulses via `apply_awakening_coupling` |

The strict ordering constraint, in plain English:

> **economic_focus must run after stratification** — `economic_focus`
> derives a 5-way classification that includes `dispossessed_permille` as
> a candidate input (`sac` leg = `belief/4`; the `Sacred` label is
> determined by belief magnitude and so the consumption order is
> belief→economic_focus; the `Mercantile` leg uses treasury and the
> `Agrarian` leg uses food, all of which are settled by the time
> `phase_economy` ends). Stratification writes `dispossessed_permille`
> which is consumed by `unrest_trade_factor`, `building_demand_signals`,
> `dispossession_unrest`, and `phase_institutions`'s garrison gate; all
> three downstream phases must therefore see a settled
> `dispossessed_permille`, hence stratification precedes them.

> **stratification must run after economic_focus** — the
> `dispossession_target_permille` formula at `engine.rs:2408` uses
> `treasury_spread` and `cohesion`; the **focus-adjusted treasury spread**
> (the treasury of a faction *minus* its `economic_focus`-weighted
> baseline) is the canonical input, so the focus must be settled first.
> This is the only DAG tension and is resolved by running
> `phase_economic_focus` **twice** per tick — once **before**
> `phase_stratification` (to seed the focus label) and once **after**
> (to settle any focus-driven production adjustment). See "Alternative
> considered: single-pass economic_focus" below.

> **emergence must run last among the new phases** — every other new
> phase writes into a `state.*` scalar that `emergence` reads via the
> upward-causation legs (e.g. `micro_cohesion_delta` already scans
> `&Psyche`, `awakening_cohesion_gain` already mints a cohesion pulse).
> The existing test at `engine.rs:3320` codifies the contract
> (`emergence_idx > life_idx` and `emergence_idx == PHASE_ORDER.len() - 1`
> with `diffusion` at the end); this ADR relaxes only the
> "must be the very last entry" half — diffusion stays at the very last
> position because the existing `propagate_cohort_wardrobe_with_lod`
> (`engine.rs:520`) consumes `target_era` derived from `research_tier()`
> and is therefore a downstream consumer of `phase_tech`'s output.

### 3. Shared physics-sim fields per phase (the physics-coupling doctrine)

ADR-011 ("N-Series Emergence Coupling Architecture") and ADR-018
("Emergence Systems Bidirectional Coupling via Shared Gradients + Conserved
Resources") define the coupling contract:

> **Every coupling is a shared gradient, not an API call.** The producer
> writes a value that the consumer reads off the same `Simulation` (or the
> equivalent crate's state struct) — no `fn call_between_layers`. **Every
> per-tick delta has a `const` cap nearby** (`MAX_*`, `*_CAP`, `*_FLOOR`,
> `*_SPAN`). The cap is the invariant that keeps the system on the edge
> of chaos.

Applied per phase:

| Phase | Shared gradient read | Shared gradient written | Cap (`const`) | Per-tick cap name |
|-------|----------------------|-------------------------|---------------|-------------------|
| `phase_life` | `population`, `ClusterMember`, `Needs` | `cluster_stocks` (food/water), `last_settlement_count`, `last_life_deaths` | `CLUSTER_FOOD_PRODUCTION_PER_MEMBER = 1`, `CLUSTER_FOOD_CONSUMPTION_PER_MEMBER = 1` (engine.rs:1929, :1934) — matched rates → net zero so accumulator stays bounded | (caller-supplied; documented at `engine.rs:1927-1934`) |
| `phase_research` | `population`, `belief`, `cohesion`, `economy_state` | `state.research_progress: u64` | `MAX_RESEARCH_PER_TICK = 5_000` (new const; rationale: research should not leap a tier per tick) | `MAX_RESEARCH_PER_TICK` |
| `phase_tech` | `state.research_progress` | `state.tech_unlocks: u64` (bitmask) | `tech_unlocks_for_tier` (`engine.rs:1955`) — derived, no per-tick rise; cap is `RESEARCH_TIER_MAX = 6` | `RESEARCH_TIER_MAX` |
| `phase_belief` | `last_sentience`, `population`, `unrest` (stale-allowed) | `state.belief: u64` | `add_belief` uses `saturating_add` (engine.rs:1016); per-tick rise bounded by `MAX_AWAKENING_BELIEF_PER_TICK = 50` + a new `MAX_BELIEF_PER_TICK = 200` macro cap | `MAX_AWAKENING_BELIEF_PER_TICK`, `MAX_BELIEF_PER_TICK` |
| `phase_unrest` | `food_price`, `energy_budget_joules`, `non-food prices`, mean `-Psyche.mood.valence` | `state.unrest: u64` | `unrest_delta` `[-DECAY, MAX_RISE]` (engine.rs:2000); `commodity_unrest_delta` `[-5, +15]` (engine.rs:2018); `agent_misery_unrest` cap `MAX_MISERY_UNREST = 30` (engine.rs:2079) | composed: net unrest rise per tick is `O(<200)` in worst case |
| `phase_cohesion` | `state.belief`, `state.unrest`, `avg_faction_kinship`, `micro_cohesion_delta` from Psyche | `state.cohesion: u64` | `cohesion_delta` bounded by belief/unrest inputs; `micro_cohesion_delta` `[-18, +12]` (engine.rs:2093); `MAX_AWAKENING_COHESION_PER_TICK = 10`, `COHESION_PER_AWAKENING = 2` (emergence.rs:539) | `MICRO_BIND_CAP`, `MICRO_FRAY_CAP`, `MAX_AWAKENING_COHESION_PER_TICK` |
| `phase_social_mood` | mean `Psyche.mood.valence`, `.arousal`, `state.cohesion` | `state.society_mood: f32` | clamp to `[-1, 1]`; per-tick step bounded by `MAX_MOOD_STEP_PER_TICK = 0.05` (new const; mood is a slow-moving average, not a derivative) | `MAX_MOOD_STEP_PER_TICK` |
| `phase_stratification` | `treasury_spread` (post-`economic_focus`), `state.cohesion`, `state.unrest`, `state.economic_focus` | `state.dispossessed_permille: u64` | `dispossession_target_permille` is already capped at 1000 permille (engine.rs:2408); per-tick change bounded by `MAX_STRAT_STEP_PER_TICK = 50` permille | `MAX_STRAT_STEP_PER_TICK` |
| `phase_institutions` | `state.belief`, `state.unrest`, `state.dispossessed_permille`, `population` | `state.temple_level: u32`, `state.garrison_level: u32` | `MAX_INSTITUTION_RISE_PER_TICK = 1` (new const; one building per tick — institutional growth is slow) | `MAX_INSTITUTION_RISE_PER_TICK` |
| `phase_economic_focus` | `state.resources.food`, `research_tier()`, `state.belief`, treasury totals | `state.economic_focus: EconomicFocus` | enum — no continuous cap; but label change is rate-limited by `MAX_FOCUS_LABEL_FLIPS_PER_TICK = 1` (new const; one focus label change per tick across the population; otherwise the chaos metric flickers) | `MAX_FOCUS_LABEL_FLIPS_PER_TICK` |
| `phase_emergence` | all 10 macro scalars above; all ECS agent state | `cluster_cultures`, `last_feed`, `last_sentience`, `last_ai_decisions`, saga graph, `faction_aggression`, bounded cohesion/belief pulses | per-tick caps already in `emergence.rs`: `drift_populations(0.02, 0.85)` retention, `apply_social_event` 12% probability, sentience threshold `0.72`; no new consts required — the existing caps satisfy the doctrine | (existing) |

### 4. Risk + guard

**Risk — perf.** The current 12-phase tick is well within the ADR-010
CA-tick-budget (1 ms target on the regression corpus). Adding 11 phases
that each perform at least one world scan (`for (_, psyche) in
world.query::<&Psyche>().iter()` is the dominant cost in
`micro_cohesion_delta`, `agent_misery_unrest`, `avg_social_affinity`,
`avg_psyche_maturity`, `avg_faction_kinship`) increases the per-tick
cost by an estimated **0.6 – 1.2 ms** at 5,000-agent populations (the
worst-case scan is `micro_cohesion_delta` at ~0.2 ms / 1000 agents).
Total tick worst case moves from ~2 ms to ~3.2 ms. **Guard:** the new
phases inherit the existing LOD (`LodPolicy`, `engine.rs:457`) and only
scan Warm / Cold tiers; Hot tiers are unaffected. Per ADR-010 the
budget guard is enforced; an over-budget tick emits `emergence.branching`
classification (`crates/civ-emergence-metrics::branching::classify_regime`)
and surfaces on the emergence dashboard. **If a tick exceeds 4 ms
total, the engine refuses to start a new tick and surfaces a
`tick_budget_exceeded` warning on the replay bus.** (Same warning path
already used by `phase_voxel` for CA overrun.)

**Risk — feedback explosion.** The 11 new phases are a **macro web** with
upward causation feeding `belief`, `unrest`, `cohesion`, `society_mood`
and downward causation feeding trade, building demand, diplomacy
threshold. Without bounded caps, the upward leg (mean misery → unrest →
diplomacy threshold → war → misery) is a positive feedback that
diverges in O(log t) ticks. **Guard:** every shared-gradient write has a
`const` cap (table above), and the cap is named and grep-able
(`rg MAX_ crates/engine/src/engine.rs`). A reviewer can verify the
caps are in place by running:

```text
rg -n 'MAX_(AWAKENING|MISERY|COHESION|RESEARCH|INSTITUTION|MOOD|STRAT|BELIEF|FOCUS)' crates/engine/src/engine.rs
```

The grep must return ≥ 8 hits (one per bounded scalar writer). A future
proposal that wants to add a new shared-gradient writer must add its
cap to this set. **No new coupling is accepted on review without a
named cap and a 3-test minimum (happy / boundary / decay) per ADR-011.**

**Risk — determinism.** The new phases are seeded off
`state.rng_seed ^ self.state.tick ^ agent_id` (ChaCha8Rng — same pattern
as `emergence_psyche` at `emergence.rs:446`) so two same-seed sims
remain byte-identical. The existing replay bus (`record_tick`,
`record_mod_loaded`, `record_voxel_write`) is extended with
`record_belief`, `record_unrest`, `record_cohesion`,
`record_dispossessed_permille`, `record_economic_focus`,
`record_society_mood`, `record_temple_level`, `record_garrison_level`
events so a `.civreplay` capture remains a complete record. **Guard:**
the existing test `phase_order_matches_tick_sequence` is amended to
reflect the new sequence, and a new test
`phase_order_satisfies_emergence_after_life` codifies the
`emergence_idx > life_idx` invariant against the new sequence.

**Risk — feedback with `phase_diplomacy` (the 60/40 coin flip).** The
existing `phase_diplomacy` (engine.rs:1695) reads `belief`, `unrest`,
cohesion via `diplomacy_conflict_threshold` (engine.rs:2784). With the
new phases feeding those scalars, the coin flip becomes **non-arbitrary**
even though the `rng.gen_bool(0.6)` is unchanged — the **threshold**
it gates against is now an emergent macro state. ADR-018 §2 row 7
already documents this; the new phases do not require any change to
`phase_diplomacy` itself (this is the point of the shared-gradient
doctrine). The "replace the 60/40 coin flip with emergent
relation-matrix logic" gap from `EMERGENCE_AUDIT.md §5 #2` is left for a
separate ADR.

## Alternatives Considered

- **Single-pass `phase_economic_focus`** (run focus once per tick, after
  stratification). This violates the DAG because
  `dispossession_target_permille` consumes focus-adjusted treasury
  spread; running focus after stratification means stratification sees
  un-focused treasuries and the focus that gets written is the
  *previous* tick's (or worse, un-focused). The two-pass `economic_focus`
  (`economic_focus_pre` before stratification; `economic_focus` after)
  costs one extra `candidate_economic_focus` call per tick (< 1 µs) and
  keeps the gradient honest. **Selected.**
- **Insert all 11 phases after `phase_diffusion`** (current tail). The
  phase-diplomacy consumer `diplomacy_conflict_threshold` is read at the
  start of every diplomacy tick; if `phase_belief` / `phase_unrest` /
  `phase_cohesion` run after diplomacy, the diplomacy phase reads stale
  scalars. Rejected — the current ordering of `phase_diplomacy` (slot
  7) **must remain before** the new phases.
- **Replace the 11 dormant phases with a single `phase_macro_web` that
  updates all scalars**. This violates the ADR-018 contract — every
  coupling must be a named shared gradient with a named cap; a single
  mega-phase hides the cap structure and breaks the per-row testability
  required by the ADR-011 3-test minimum. Rejected.
- **Run `phase_emergence` twice per tick** (once for the upward
  causation leg that mutates `belief` / `cohesion`, once for the saga
  recording). `emergence_genetics_sentience` already does this
  internally (it builds `faction_aggression`, mutates
  `last_sentience`, and calls `apply_awakening_coupling` — all in one
  method). Splitting the method would double the world-scan cost for
  no functional gain. **Selected: single pass.**
- **No new phases — instead, inline the scalar writers in
  `phase_economy` / `phase_diplomacy` / `phase_buildings`**. This was
  rejected at ADR-011's authoring time: coupling code that lives in
  the consumer's crate (here, `economy` or `diplomacy`) violates the
  shared-gradient doctrine — the consumer must read the gradient, not
  write it. The 11-phase split keeps each writer in its own
  crate-local module with a clean upstream / downstream contract.

## Consequences

- `PHASE_ORDER` (engine.rs:55) grows from 12 entries to 23. The existing
  test `phase_order_matches_tick_sequence` (engine.rs:3294) is amended
  in the same PR.
- `Simulation::tick` (engine.rs:1179) grows from 12 phase calls to 23.
  The deterministic ordering is preserved (no reordering of existing
  phases).
- Two new `Simulation` fields: `state.unrest: u64`,
  `state.cohesion: u64` (parallel to existing `belief: u64` at
  engine.rs:422). `state.society_mood: f32`,
  `state.dispossessed_permille: u64`, `state.economic_focus:
  EconomicFocus`, `state.temple_level: u32`, `state.garrison_level: u32`
  — and the existing `state.research_progress: u64` /
  `state.tech_unlocks: u64` are **promoted** from "field-on-`WorldState`
  not yet declared" to explicit fields. The 11 phantom phases need real
  producers; this ADR's companion engine PR lands them.
- `add_belief` / `try_invoke_divine_power` (engine.rs:1015, :1021) are
  joined by `add_unrest`, `add_cohesion`, `set_economic_focus`,
  `bump_temple_level`, `bump_garrison_level`, `set_dispossessed_permille`,
  `set_society_mood` — all `saturating_add` / clamped writers.
- `replay.rs` grows 8 new `record_*` methods that mirror the existing
  `record_damage` / `record_voxel_write` / `record_mod_loaded` shape, so
  the `.civreplay` format remains a complete record.
- The `emergence_dashboard` (per ADR-011 §"Emergence dashboard
  observability") gains 11 new metric streams (one per new phase),
  each with a chaos-bounded envelope (heatmap threshold from
  `crates/civ-emergence-metrics::branching::classify_regime`).
- The phantom-target test calls at engine.rs:4576, 4577, 4594, 4604,
  4605 (`sim.phase_tech()`, `sim.phase_chronicle()`) finally compile
  because the methods now exist. (`phase_chronicle` is out of scope for
  this ADR — it is the **chronicle writer** that reads `state.chronicle`
  and the saga graph; its absence is documented at `EMERGENCE_AUDIT.md
  §2 #33`. A separate ADR can land it; this ADR restricts its scope to
  the 11 named phases plus `phase_emergence`.)
- The 3-test minimum from ADR-011 applies: each new phase gets a happy
  path test (e.g. `phase_belief_increases_on_awakening`), a boundary
  test (e.g. `phase_unrest_clamps_at_cap`), and a decay test (e.g.
  `phase_cohesion_decays_without_kin`). All 33 new tests must pass
  before the ADR is moved to Accepted.
- The cargo preflight warning in `EMERGENCE_AUDIT.md §4` (duplicate
  `phenotype-voxel` lockfile entry) is **not** in scope for this ADR.
  It is a separate cleanup tracked under the governance repair pass.

## Cross-References

- ADR-011 — N-Series Emergence Coupling Architecture (the contract
  every new phase must satisfy: shared gradient + named cap + 3-test
  minimum + FR-traceable + dashboard-observable).
- ADR-018 — Emergence Systems Bidirectional Coupling via Shared
  Gradients + Conserved Resources (the **inventory** of couplings;
  rows 9, 10, 11, 16, 17, 18, 19, 22, 23, 26, 28 are the consumers
  this ADR's producers feed).
- ADR-014 — Language emergence substrate (the
  `faction_language_centroids` and `language_trade_factor` consumers
  in ADR-018 rows 4, 5 are downstream of `phase_life` settlement
  membership).
- ADR-015 — Faction emergence substrate (the
  `settlement_dominant_factions` consumer in ADR-018 row 8 is
  downstream of `phase_life` and `phase_stratification`).
- ADR-016 — Religion emergence substrate (the
  `awakening_belief_gain` / `awakening_cohesion_gain` consumers in
  ADR-018 row 11 are downstream of `phase_emergence`'s sentience
  sub-phase).
- ADR-emergence-charter — the umbrella decision this ADR implements
  for the dormant-phase half of the charter.
- ADR-003 / ADR-determinism-dropped — replay determinism contract.
  Every new phase is seeded off `state.rng_seed ^ self.state.tick ^
  agent_id` (ChaCha8Rng), satisfying the determinism contract.
- ADR-008 — Algorithmic genetics (no LLM). The
  `faction_aggression` rebuild in `emergence_genetics_sentience` is
  unchanged; this ADR does not introduce any LLM call in the new
  phases.
- ADR-010 — CA tick budget guard. The 11 new phases are bounded by the
  CA tick budget guard; an over-budget tick emits a replay-bus warning.
- `crates/engine/src/engine.rs:1172` — `Simulation::tick`; the call
  site this ADR extends.
- `crates/engine/src/engine.rs:55` — `PHASE_ORDER`; the constant this
  ADR extends.
- `crates/engine/src/emergence.rs:159` — `phase_emergence`; the only
  dormant phase that already exists as a method on `Simulation` and
  which this ADR promotes from dead to wired.
- `crates/engine/src/emergence.rs:4-6` — the documented internal DAG
  inside `phase_emergence` (`genetics → culture → social → psyche →
  sentience → legends → civ_ai`); unchanged by this ADR.
- `crates/agents/src/cluster.rs` — `cluster_by_colocation`,
  `reconcile_membership`, `should_join`, `should_leave`; wired in
  `phase_life` by this ADR's companion PR.
- `crates/engine/src/demographics.rs` — `tick_demographics`,
  `carrying_capacity_from_food`, `Demographics`, `AgeGroup`; wired in
  `phase_life` alongside the cluster module.
- `crates/engine/src/religion.rs` — `emerge_belief`,
  `spread_religion`, `Belief`, `BeliefConcept`, `Religion`; wired in
  `phase_belief`.
- `crates/research` — `LawDb`, `tech_unlocks`; wired in
  `phase_research` / `phase_tech`.
- `crates/engine/src/language.rs` — `tick_language`, `should_split`,
  `borrow_word`, `Phoneme`, `Morpheme`, `LanguageState`; wired in
  `phase_life` (language is a per-cluster emergent property of
  settled populations).
- `crates/engine/src/faction_emergence.rs` — `cluster_into_factions`,
  `should_faction_split`, `should_faction_merge`, `AgentIdeology`,
  `FactionSeed`; wired in `phase_stratification` (split / merge is
  gated by `dispossessed_permille` and cohesion, both settled by
  the time stratification runs).
- `docs/reports/EMERGENCE_AUDIT.md` — the 2026-06-23 audit this ADR
  closes gap #1 (top of the §5 ranked list) and partially closes gap
  #6 (research / tech / belief / unrest / cohesion / stratification /
  institutions / economic_focus producers).
- `crates/civ-emergence-metrics::branching::classify_regime` — the
  edge-of-chaos / heat-death / explosion classifier that the bounded
  caps protect; the perf guard in this ADR emits a metric on its
  input stream.
- `crates/legends/src/lib.rs:13-14` — saga graph is a measured record,
  not a coupling gradient; explains why the new phases **do not**
  mutate the saga graph (only `phase_emergence` records into it).
- `docs/specs/CIV-0100` §3 — emergence engineering doc that named
  "downward causation" / "upward causation" terminology reused in
  this ADR.
