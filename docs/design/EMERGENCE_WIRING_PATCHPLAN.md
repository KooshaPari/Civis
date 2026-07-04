# Emergence Wiring Patch Plan

> Companion implementation playbook for [ADR-020](../adr/ADR-020-wire-dormant-emergence-phases.md).
> This file is the **mechanical recipe** an implementer follows inside
> `crates/engine/src/engine.rs:1183` `Simulation::tick` to land all 11 dormant
> phases plus the `phase_emergence` orchestrator in the correct dependency
> order. It does not introduce new design decisions; it operationalises the
> DAG, cap table, and risk guards from ADR-020.

**Scope.** Read-only patch plan (no code changes in this commit). The
implementer lands the corresponding code in a follow-up engine PR whose
build is GREEN.

**Source of truth (citations).**

- `crates/engine/src/engine.rs:55-68` — `PHASE_ORDER` (current 12 entries).
- `crates/engine/src/engine.rs:1178-1211` — `Simulation::tick` body.
- `crates/engine/src/engine.rs:1281-1850` — existing `phase_*` method
  signatures (`phase_planet`, `phase_tactics`, `phase_voxel`,
  `phase_compact`, `phase_buildings`, `phase_diffusion`,
  `phase_production`, `phase_citizen_lifecycle`, `phase_military`,
  `phase_diplomacy`, `phase_policy`, `phase_economy`).
- `crates/engine/src/emergence.rs:151-171` — existing `phase_emergence`
  orchestrator (dead outside `#[cfg(test)]` until this PR).
- `crates/engine/src/engine.rs:300-326` — `WorldState` struct (the four
  fields `belief`, `unrest`, `cohesion`, etc. referenced by the existing
  tests at `engine.rs:4670, 5038-5040, 5057-5059, 5322-5324, 5354-5356` are
  **not** declared on `WorldState` yet; see §2 below).
- `crates/engine/src/engine.rs:3294-3410` — `phase_order_matches_tick_sequence`
  + `phase_order_includes_emergence` tests (both must be amended in the
  same engine PR).
- `crates/engine/src/engine.rs:4571, 4582, 4644, 4662, 4672, 5051, 5065, 5348, 5371`
  — phantom-target test calls that compile only once the new phase
  methods exist (`phase_tech`, `phase_chronicle`, `phase_diplomacy`
  callers).

---

## 1. Final `PHASE_ORDER` (line `engine.rs:55-68`)

Replace the existing 12-entry constant with the 23-entry sequence below.
The order is the strict topological sort of the shared-gradient edges
documented in ADR-020 §2; the rationale for the **two-pass economic_focus**
is reproduced in §3.10.

```rust
pub(crate) const PHASE_ORDER: &[&str] = &[
    "production",
    "citizen_lifecycle",
    "military",
    "policy",
    "economy",
    "planet",
    "diplomacy",
    "tactics",
    "voxel",
    "compact",
    "buildings",
    "life",                 // NEW (#1)
    "research",             // NEW (#2)
    "tech",                 // NEW (#3)
    "belief",               // NEW (#4)
    "unrest",               // NEW (#5)
    "cohesion",             // NEW (#6)
    "social_mood",          // NEW (#7)
    "stratification",       // NEW (#8)
    "institutions",         // NEW (#9)
    "economic_focus",       // NEW (#10 — second pass; see §3.10)
    "emergence",            // NEW (orchestrator — pre-existing at emergence.rs:159)
    "diffusion",            // TAIL — unchanged, still last
];
```

**Test amendments (same engine PR).**

- `phase_order_matches_tick_sequence` (`engine.rs:3362`) → assert against
  the 23-entry list above.
- `phase_order_includes_emergence` (`engine.rs:3388`) → relax the
  `emergence_idx == PHASE_ORDER.len() - 1` assertion to
  `emergence_idx == PHASE_ORDER.len() - 2` (diffusion now sits at the very
  end). The `emergence_idx > life_idx` half of the test is unchanged.

---

## 2. `WorldState` field promotion (line `engine.rs:306-326`)

The 11 phantom phases need real producers. The 7 new `state.*` fields
below are referenced by the existing test suite (`engine.rs:4670, 5038-5040,
5057-5059, 5322-5324, 5354-5356`) and by the new `phase_*` methods, so
they must be **promoted** to the `WorldState` struct (not kept on
`Simulation`). All fields use `#[serde(default)]` so older `.civreplay`
files load cleanly.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    pub tick: u64,
    pub population: u64,
    pub energy_budget_joules: Fixed,
    pub rng_seed: u64,
    pub factions: HashMap<u32, String>,
    pub faction_treasury: HashMap<u32, Fixed>,
    pub faction_resources: HashMap<u32, Resources>,
    pub trade_routes: Vec<TradeRoute>,
    #[serde(default)] pub emergent_trade_route_keys: BTreeSet<(u32, u32, String)>,
    #[serde(default)] pub trade_route_idle_ticks: BTreeMap<(u32, u32, String), u32>,
    pub resources: Resources,

    // ---- NEW: macro-web state promoted from phantom to producer ----
    /// Faith / belief reserve. Read by `phase_diplomacy` (N-religion) and
    /// `phase_cohesion`. Written by `phase_belief` and
    /// `phase_emergence::apply_awakening_coupling`. Saturating.
    #[serde(default)] pub belief: u64,
    /// Social unrest (FR-CIV-0100 §3). Read by `phase_diplomacy` (N12
    /// threshold) and `phase_cohesion` (fray divisor). Written by
    /// `phase_unrest`. Saturating; floor 0.
    #[serde(default)] pub unrest: u64,
    /// Social cohesion (FR-CIV-0100 §3). Read by `phase_stratification`
    /// (erosion term) and downstream coupling fns. Written by
    /// `phase_cohesion` and `phase_emergence::apply_awakening_coupling`.
    /// Saturating; floor 0.
    #[serde(default)] pub cohesion: u64,
    /// Mean social mood in [-1, 1] (clamped). Slow-moving aggregate of
    /// per-agent `Psyche.mood.valence`. Written by `phase_social_mood`;
    /// per-tick step capped at `MAX_MOOD_STEP_PER_TICK = 0.05`.
    #[serde(default)] pub society_mood: f32,
    /// Dispossessed share in permille [0, 1000]. Read by `phase_unrest`
    /// (downward causation) and `phase_institutions` (garrison gate).
    /// Written by `phase_stratification`.
    #[serde(default)] pub dispossessed_permille: u64,
    /// 5-way economic focus label. Written by `phase_economic_focus`
    /// (twice per tick; see §3.10).
    #[serde(default)] pub economic_focus: EconomicFocus,
    /// Temple / civic-institution level in [0, 5]. One step per tick max.
    #[serde(default)] pub temple_level: u32,
    /// Garrison / military-institution level in [0, 5]. One step per tick max.
    #[serde(default)] pub garrison_level: u32,
    /// Research progress accumulator (u64, saturating). Written by
    /// `phase_research` (capped by `MAX_RESEARCH_PER_TICK = 5_000`).
    #[serde(default)] pub research_progress: u64,
    /// Research tier derived from number of completed techs in
    /// `research_cache.researched`; computed, not stored, but exposed
    /// for replay-log readability.
    /// Tech-unlock bitmask (set-only; promoted from phantom to producer
    /// in `phase_tech` via `tech_unlocks_for_tier(research_tier)`).
    #[serde(default)] pub tech_unlocks: u64,
    /// Bounded human-readable chronicle (CHRONICLE_MAX_LEN = 200). Written
    /// by `phase_chronicle` (out of scope for ADR-020; see §5).
    #[serde(default)] pub chronicle: Vec<String>,
    /// Used by `phase_chronicle` to dedupe repeated golden-age lines.
    #[serde(default)] pub chronicle_age: u64,
}
```

The `Default` impl (`engine.rs:328-399`) gets the same 11-field
additions, all initialised to zero / `EconomicFocus::Balanced` /
`Vec::new()`. `tech_unlocks = 0`, `society_mood = 0.0`,
`economic_focus = EconomicFocus::Balanced`.

**Side effect.** The existing `Simulation::belief: u64` field at
`engine.rs:422` must be **moved to** `WorldState::belief`; all
`self.belief` / `self.add_belief` call sites in
`emergence.rs:38, 544, 545` become `self.state.belief` /
`self.state.belief = self.state.belief.saturating_add(...)`. The
`Simulation::belief()` accessor (`engine.rs:1019-1023`) and
`Simulation::add_belief()` (`engine.rs:1025-1028`) become thin shims
over `self.state.belief`. The 11 new accessors listed in §4 are
siblings, not replacements.

---

## 3. Exact call-site additions inside `Simulation::tick`

`engine.rs:1183-1211` currently reads:

```rust
pub fn tick(&mut self) {
    self.state.tick += 1;
    self.last_tick_combat_pulses.clear();
    self.last_tick_engagements.clear();
    self.last_tick_mod_lifecycle.clear();
    self.last_tick_construction_events.clear();

    // Phases in PHASE_ORDER (CIV-0001 partial)
    self.phase_production();
    self.phase_citizen_lifecycle();
    self.phase_military();
    self.phase_policy();
    self.phase_economy();
    self.phase_planet();
    self.diplomacy_events.clear();
    self.phase_diplomacy();
    self.phase_tactics();
    self.phase_voxel();
    self.phase_compact();
    self.phase_buildings();
    self.phase_diffusion();
    self.replay_log.record_tick(self.state.tick);

    #[cfg(debug_assertions)]
    debug_assert!(
        crate::integrity::check_integrity(self).is_ok(),
        "simulation integrity violated"
    );
}
```

After the patch, the body reads (added lines marked `// +NEW`):

```rust
pub fn tick(&mut self) {
    self.state.tick += 1;
    self.last_tick_combat_pulses.clear();
    self.last_tick_engagements.clear();
    self.last_tick_mod_lifecycle.clear();
    self.last_tick_construction_events.clear();

    // Phases in PHASE_ORDER (CIV-0001 partial)
    self.phase_production();
    self.phase_citizen_lifecycle();
    self.phase_military();
    self.phase_policy();
    self.phase_economy();
    self.phase_planet();
    self.diplomacy_events.clear();
    self.phase_diplomacy();
    self.phase_tactics();
    self.phase_voxel();
    self.phase_compact();
    self.phase_buildings();
    self.phase_life();                    // +NEW (#1) — §3.1
    self.phase_research();                // +NEW (#2) — §3.2
    self.phase_tech();                    // +NEW (#3) — §3.3
    self.phase_belief();                  // +NEW (#4) — §3.4
    self.phase_unrest();                  // +NEW (#5) — §3.5
    self.phase_cohesion();                // +NEW (#6) — §3.6
    self.phase_social_mood();             // +NEW (#7) — §3.7
    self.phase_economic_focus_pre();      // +NEW (#10a first pass) — §3.10
    self.phase_stratification();          // +NEW (#8) — §3.8
    self.phase_institutions();            // +NEW (#9) — §3.9
    self.phase_economic_focus();          // +NEW (#10 settle pass) — §3.10
    self.phase_emergence();               // +NEW (orchestrator) — §3.11
    self.phase_diffusion();               // tail — unchanged
    self.replay_log.record_tick(self.state.tick);

    #[cfg(debug_assertions)]
    debug_assert!(
        crate::integrity::check_integrity(self).is_ok(),
        "simulation integrity violated"
    );
}
```

The line `self.diplomacy_events.clear();` stays immediately before
`self.phase_diplomacy()` — `phase_diplomacy` already appends to it, and
the new phases must see a clean buffer.

### 3.1 `phase_life` — settlement commons (slot 12 in PHASE_ORDER)

**Signature (new on `Simulation`).**

```rust
fn phase_life(&mut self);
```

**Inputs.** `state.population` (post-`citizen_lifecycle`),
`world: &mut World` (ECS), `ClusterMember` components, `Needs` components,
`state.faction_resources` (read-only).

**Outputs / writes.** `self.cluster_stocks: BTreeMap<u64, ClusterStocks>`,
`self.last_settlement_count: u32`, `self.last_life_deaths: u32`,
`self.cluster_cultures` (cleared at start; repopulated by
`emergence_culture` from the cluster id space).

**DAG rationale.** Runs after `citizen_lifecycle` (population settled)
and before `research` (so `population` is the canonical input), but
strictly before `phase_emergence` (so cluster membership and
`last_settlement_count` are stable when `emergence_culture` re-derives
`cluster_cultures` at `emergence.rs:194-213`). Const
`CLUSTER_FOOD_PRODUCTION_PER_MEMBER = 1` /
`CLUSTER_FOOD_CONSUMPTION_PER_MEMBER = 1` (defined at
`engine.rs:1997-2002`) keep the food commons bounded at matched rates
(net zero).

**Checklist.**

- [ ] `cluster_by_colocation` (`crates/agents/src/cluster.rs`) is
      called to derive per-tick `ClusterMember` membership.
- [ ] `reconcile_membership` handles agents leaving/joining existing
      clusters (`should_join` / `should_leave`).
- [ ] Clusters with `< 2` members are **not** committed to
      `cluster_stocks` (matches `emergence_culture` filter at
      `emergence.rs:198`).
- [ ] `cluster_stocks` keys are sorted (BTreeMap iteration is
      deterministic so this is the canonical contract).
- [ ] `last_settlement_count` is the count of multi-member clusters
      committed this tick.
- [ ] No new `const` caps required — the matched
      production/consumption rates keep the food commons bounded
      (ADR-020 §3 table).
- [ ] 3-test minimum (ADR-011): `phase_life_commits_2plus_clusters`,
      `phase_life_drops_singleton_clusters`, `phase_life_food_commons_bounded`.

### 3.2 `phase_research` — research progress (slot 13)

**Signature.**

```rust
fn phase_research(&mut self);
```

**Inputs.** `state.population` (post-`life`),
`state.belief` (stale-allowed: this phase runs **before** `phase_belief`,
so the cap on belief-flavoured input is small),
`state.cohesion` (stale-allowed: this phase runs **before**
`phase_cohesion`, so cohesion contribution is
`cohesion_research_bonus_permille(stale) / 1000` × base),
`economy_state.research_funding` (post-`phase_economy`).

**Outputs / writes.** `state.research_progress: u64` (saturating;
capped at `MAX_RESEARCH_PER_TICK = 5_000` per tick). Also pushes
`research_cache.in_progress` toward completion and updates
`research_cache.researched` when a tech completes.

**DAG rationale.** `cohesion_research_bonus_permille` is small (cap
+50%) so running before `phase_cohesion` introduces a single-tick
lag in the upward-cohesion→research loop; the lag is bounded and
named (see ADR-020 §2 "Note on belief/cohesion latency"). No
downstream consumer reads `research_progress` before `phase_tech`,
so the slot is safe.

**Checklist.**

- [ ] `MAX_RESEARCH_PER_TICK = 5_000` declared in the
      `const`-block alongside `CLUSTER_FOOD_*` (search
      `engine.rs:1993-2012`).
- [ ] Per-tick rise = `min(MAX_RESEARCH_PER_TICK,
      base_research + belief_contribution + cohesion_contribution
      + sentience_research_bonus(&self.world))`.
- [ ] `saturating_add` everywhere (no panic on overflow).
- [ ] When `research_progress` crosses a tier boundary,
      `research_cache.researched` is updated; `last_births` /
      `last_deaths` are **not** touched (population is the
      `citizen_lifecycle` writer's contract).
- [ ] 3-test minimum: `phase_research_increments_progress`,
      `phase_research_clamps_at_max`, `phase_research_no_op_with_zero_inputs`.

### 3.3 `phase_tech` — tier-derived tech unlocks (slot 14)

**Signature.**

```rust
fn phase_tech(&mut self);
```

**Inputs.** `state.research_progress` (post-`phase_research`),
`research_cache.researched` (history of completed tech names).

**Outputs / writes.** `state.tech_unlocks: u64` bitmask via
`tech_unlocks_for_tier(research_tier())` (`engine.rs:2023-2044`).
`research_tier()` is a computed accessor at `engine.rs:1043-1045`
(`researched.len() as u64`), not a stored field.

**DAG rationale.** Pure-function derivable from
`research_cache.researched.len()`. Re-derived each tick so the bitmask
is always coherent with the cache; no per-tick cap required (the
underlying `researched` list grows at most by 1 per tick due to the
research cap in §3.2).

**Checklist.**

- [ ] Call `tech_unlocks_for_tier(self.research_tier())` and assign
      to `state.tech_unlocks` (no-op if the value is unchanged).
- [ ] Optional: when the bitmask grows, push a `chronicle` entry
      (deferred to the `phase_chronicle` PR — see §5).
- [ ] The phantom-target test at `engine.rs:4644`
      (`sim.phase_tech()`) compiles.
- [ ] 3-test minimum: `phase_tech_sets_irrigation_at_tier_1`,
      `phase_tech_idempotent`, `phase_tech_no_progress_no_unlocks`.

### 3.4 `phase_belief` — faith reserve (slot 15)

**Signature.**

```rust
fn phase_belief(&mut self);
```

**Inputs.** `emergence.last_sentience` (post-`phase_emergence` on the
**previous** tick — first few ticks see an empty list, which is the
intended behaviour since `last_sentience` is cleared at the start of
every `phase_emergence` call at `emergence.rs:162`), `state.unrest`
(stale-allowed: `phase_unrest` runs after this phase), `state.population`
(post-`phase_life`), `religion::spread_religion` (callable hook, no
default), `disasters` (not wired — TODO per ADR-020).

**Outputs / writes.** `state.belief: u64` via the existing
`Simulation::add_belief` shim (which now mutates `state.belief`).
Per-tick rise = `awakening_belief_gain(awakenings) +
spread_religion_signal(...)` capped by
`MAX_AWAKENING_BELIEF_PER_TICK = 50` (existing, in
`emergence.rs`/`apply_awakening_coupling`) and the new macro cap
`MAX_BELIEF_PER_TICK = 200`.

**DAG rationale.** First writer of `state.belief` *after*
`phase_emergence` on the previous tick. The stale-allowed read of
`state.unrest` is intentional: faith rises fastest when unrest is
high (a tension / piety coupling); the per-tick cap stops the
positive feedback from running away.

**Checklist.**

- [ ] `MAX_BELIEF_PER_TICK = 200` declared in the `const`-block.
- [ ] `awakening_belief_gain` imported from `crate::engine` (already
      in `emergence.rs:38`); reuse the existing pure fn.
- [ ] If `religion::spread_religion` is wired in this PR, gate on a
      `cfg` flag — leave the call site as a TODO marker otherwise.
- [ ] Record on replay bus: `replay_log.record_belief(self.state.tick,
      self.state.belief, delta)` (new `record_belief` method on
      `ReplayLog`, see §6).
- [ ] 3-test minimum: `phase_belief_increases_on_awakening`,
      `phase_belief_clamps_at_cap`, `phase_belief_zero_awakenings_no_op`.

### 3.5 `phase_unrest` — societal unrest (slot 16)

**Signature.**

```rust
fn phase_unrest(&mut self);
```

**Inputs.** `market_state.prices()` (`food` + non-food `BTreeMap`),
`economy_state.energy_budget_joules` (post-`phase_economy`),
mean `(-Psyche.mood.valence)` from the world ECS, mean kinship from
the world ECS (post-`phase_emergence` on the previous tick —
`agent_misery_unrest` uses the same scan pattern, no extra
overhead), `faction_treasury_spread(&state.faction_treasury)`,
`state.dispossessed_permille` (stale-allowed on the first tick after
`phase_stratification` lands).

**Outputs / writes.** `state.unrest: u64` (saturating; floor 0).
Per-tick delta = `unrest_delta(food_price) +
commodity_unrest_delta(non_food_prices) + energy_scarcity_unrest(...)
+ agent_misery_unrest(&self.world) +
overcrowding_unrest(population, capacity) +
inequality_unrest(faction_treasury_spread) +
dispossession_unrest(dispossessed_permille)`, then passed through
`cohesion_unrest_damp(rise, cohesion)` (rises only) and
`research_unrest_mitigation(rise, research_tier())` (rises only).

**DAG rationale.** Reads `food_price` (post-`phase_economy`) and
`energy_budget_joules` (post-`phase_economy`); reads
`dispossessed_permille` stale-allowed (single-tick lag); reads
`cohesion` from the previous tick via the `cohesion_unrest_damp` damp
(single-tick lag). The mean `(-Psyche.mood.valence)` is from the ECS
post-`phase_emergence` on the previous tick (also single-tick lag);
acceptable because the mean is a slow-moving aggregate.

**Checklist.**

- [ ] All seven delta terms composed with explicit `clamp(..., MAX_RISE)`
      or `min(MAX_*)` — the composed net rise is `O(<200)` per tick
      (ADR-020 §3 table).
- [ ] `state.unrest` floored at 0 after each add/sub (no negative
      unrest — the "calm" is encoded as zero, not -X).
- [ ] `cohesion_unrest_damp` only damps rises (passes decay
      through), per the existing `engine.rs:2725-2731` contract.
- [ ] `research_unrest_mitigation` only damps rises (passes decay
      through), per `engine.rs:2526-2532`.
- [ ] Record on replay bus: `record_unrest(tick, value, delta)`.
- [ ] 3-test minimum: `phase_unrest_rises_on_food_scarcity`,
      `phase_unrest_clamps_at_cap`, `phase_unrest_decays_when_abundant`.

### 3.6 `phase_cohesion` — social fabric (slot 17)

**Signature.**

```rust
fn phase_cohesion(&mut self);
```

**Inputs.** `state.belief` (post-`phase_belief`),
`state.unrest` (post-`phase_unrest`), `avg_faction_kinship(&self.world)`
(post-`phase_emergence` on previous tick; this is the N10 upward
causation), `micro_cohesion_delta(&self.world)` (re-scanned here;
`engine.rs:2161-2188`).

**Outputs / writes.** `state.cohesion: u64` (saturating; floor 0).
Per-tick delta = `cohesion_delta(belief, unrest) +
micro_cohesion_delta(&self.world) + (kinship_boost * cohesion_tier)
+ awakening_cohesion_gain(awakenings_from_last_sentience)`, all summed
with explicit caps:
`MICRO_BIND_CAP = 12` / `MICRO_FRAY_CAP = 18` (existing, in
`micro_cohesion_delta`),
`MAX_AWAKENING_COHESION_PER_TICK = 10` (existing, in
`emergence.rs:2713`).

**DAG rationale.** Runs after `phase_unrest` so unrest is fresh;
runs after `phase_belief` so belief is fresh. The `kinship_boost` is
read from the post-`phase_emergence` previous-tick world scan
(acceptable lag). `phase_emergence` on the **current** tick
re-applies the awakening pulse via `apply_awakening_coupling` at
`emergence.rs:539-546` *after* this phase, which means
`apply_awakening_coupling` reads `state.cohesion` and adds a bounded
pulse — the bookkeeping is clean.

**Checklist.**

- [ ] `cohesion_delta` is the existing pure fn at `engine.rs:2698-2702`
      (no changes; `belief / 200 - unrest / 50`).
- [ ] `micro_cohesion_delta(&self.world)` returns signed i64
      `[-18, +12]`.
- [ ] `awakening_cohesion_gain` reads `emergence.last_sentience.len()`
      (a previous-tick value, capped at 10 per tick).
- [ ] `state.cohesion` floored at 0 after each add/sub.
- [ ] Record on replay bus: `record_cohesion(tick, value, delta)`.
- [ ] 3-test minimum: `phase_cohesion_rises_with_belief`,
      `phase_cohesion_frays_with_unrest`, `phase_cohesion_binds_with_consensus`.

### 3.7 `phase_social_mood` — mean mood aggregate (slot 18)

**Signature.**

```rust
fn phase_social_mood(&mut self);
```

**Inputs.** Mean `Psyche.mood.valence` and `.arousal` from the world
ECS (post-`phase_emergence` on the previous tick — same single-tick
lag as `agent_misery_unrest`; acceptable because the mean is a
slow-moving aggregate), `state.cohesion` (post-`phase_cohesion`).

**Outputs / writes.** `state.society_mood: f32` clamped to `[-1, 1]`;
per-tick step bounded by `MAX_MOOD_STEP_PER_TICK = 0.05` (new const;
mood is a slow-moving average, not a derivative).

**DAG rationale.** Last writer that consumes a post-`phase_cohesion`
input before `phase_stratification`. The new const cap is named and
grep-able (ADR-020 §4 "feedback explosion" guard).

**Checklist.**

- [ ] `MAX_MOOD_STEP_PER_TICK = 0.05` declared in the `const`-block.
- [ ] `state.society_mood = (state.society_mood + delta).clamp(-1.0, 1.0)`
      where `delta = (mean_valence - state.society_mood).clamp(-MAX, +MAX)`.
- [ ] `mean_valence` re-derived from a single world scan
      (`for (_, p) in world.query::<&Psyche>()`); do **not** store
      mean_valence on `Simulation` (it is a per-tick derived value).
- [ ] Record on replay bus: `record_society_mood(tick, value)`.
- [ ] 3-test minimum: `phase_social_mood_rises_on_positive_mean`,
      `phase_social_mood_clamps_at_one`, `phase_social_mood_step_bounded`.

### 3.8 `phase_stratification` — dispossessed share (slot 19)

**Signature.**

```rust
fn phase_stratification(&mut self);
```

**Inputs.** `treasury_spread = faction_treasury_spread(...)`
(post-`phase_economy`; uses `state.faction_treasury` directly),
`state.cohesion` (post-`phase_cohesion`),
`state.economic_focus` (must be **settled** — see §3.10 for the
two-pass justification),
`state.unrest` (post-`phase_unrest`).

**Outputs / writes.** `state.dispossessed_permille: u64` via
`dispossession_step(state.dispossessed_permille, target)` where
`target = dispossession_target_permille(spread, cohesion)` (existing
fn at `engine.rs:2476-2481`); bounded at 1000 permille and capped
at `MAX_STRAT_STEP_PER_TICK = 50` permille per tick (new const;
the existing `dispossession_step` has a `MAX_STEP = 5` per-tick
internal cap at `engine.rs:2506-2513`; the new outer cap of 50 is a
defensive double-guard).

**DAG rationale.** The only DAG tension in the new phases.
`dispossession_target_permille` consumes the focus-adjusted treasury
spread; running after `phase_economic_focus_pre` ensures the focus
label is available even though the per-tick treasury spread is not
yet adjusted. The two-pass (§3.10) handles the chicken-and-egg.

**Checklist.**

- [ ] `MAX_STRAT_STEP_PER_TICK = 50` declared in the `const`-block.
- [ ] Defensive: if `state.economic_focus` is
      `EconomicFocus::default()` (`Balanced`) on the very first
      tick, treat it as neutral; do not panic.
- [ ] `state.dispossessed_permille` clamped to `0..=1000` after
      each step.
- [ ] Record on replay bus: `record_dispossessed_permille(tick, value)`.
- [ ] 3-test minimum: `phase_stratification_rises_on_inequality`,
      `phase_stratification_decays_with_cohesion`, `phase_stratification_clamps_at_1000`.

### 3.9 `phase_institutions` — temple + garrison levels (slot 20)

**Signature.**

```rust
fn phase_institutions(&mut self);
```

**Inputs.** `state.belief` (post-`phase_belief`),
`state.unrest` (post-`phase_unrest`),
`state.dispossessed_permille` (post-`phase_stratification`),
`state.population` (post-`phase_life`).

**Outputs / writes.** `state.temple_level: u32` and
`state.garrison_level: u32`, both via `institution_step(current,
target)` where
`temple_target = institution_target_level(belief, per_level)` and
`garrison_target = institution_target_level(unrest +
dispossessed_permille, per_level)`. Per-tick rise capped at
`MAX_INSTITUTION_RISE_PER_TICK = 1` (already enforced by
`institution_step` at `engine.rs:2494-2502`; the `MAX_INSTITUTION_LEVEL = 5`
const at `engine.rs:2484` is the absolute cap). The new const name
documents the per-tick step cap explicitly for grep-ability
(ADR-020 §4 guard).

**DAG rationale.** Reads only post-`phase_stratification` state and
the two earlier macro scalars (belief, unrest). Runs before
`phase_economic_focus` (the settle pass) so the focus sees the
institutional picture.

**Checklist.**

- [ ] `MAX_INSTITUTION_RISE_PER_TICK = 1` declared in the
      `const`-block (mirrors the existing
      `institution_step` contract).
- [ ] Both `temple_level` and `garrison_level` clamped to
      `0..=MAX_INSTITUTION_LEVEL = 5`.
- [ ] Record on replay bus: `record_temple_level(tick, value)` and
      `record_garrison_level(tick, value)`.
- [ ] 3-test minimum: `phase_institutions_temple_rises_with_belief`,
      `phase_institutions_garrison_rises_with_unrest`,
      `phase_institutions_step_bounded`.

### 3.10 `phase_economic_focus` — two passes (slots 21a + 21b)

The DAG has a single chicken-and-egg edge: `phase_stratification`
reads `state.economic_focus`, but the focus label is in turn
influenced by the *settled* cohesion (which `phase_stratification`
consumes). The resolution is **two passes per tick**:
`phase_economic_focus_pre` (slot 21a) seeds the focus label from
the previous tick's settled state; `phase_stratification` (slot 19)
runs against the seeded focus; `phase_economic_focus` (slot 21b)
settles the focus for the current tick. The two-pass costs one
extra `candidate_economic_focus` call per tick (< 1 µs) and keeps
the gradient honest (ADR-020 §"Alternatives considered").

**Signature (both passes).**

```rust
fn phase_economic_focus_pre(&mut self);
fn phase_economic_focus(&mut self);
```

**Shared inputs.** `state.resources.food` (post-`phase_economy`),
`research_tier()` (post-`phase_tech`),
`state.belief` (post-`phase_belief`), `treasury_total =
sum(faction_treasury.values())`.

**Shared outputs.** `state.economic_focus: EconomicFocus` via
`candidate_economic_focus(food, research_tier, belief,
treasury_total)` (existing pure fn at `engine.rs:2341-2364`).
Per-tick change bounded by `MAX_FOCUS_LABEL_FLIPS_PER_TICK = 1` (new
const; one focus label change per tick across the population;
otherwise the chaos metric flickers).

**DAG rationale.** The first pass uses *previous-tick* settled
cohesion (the only stale read in the new phases; acceptable because
the focus is a 5-way enum, not a continuous scalar). The second
pass uses *current-tick* settled state.

**Checklist.**

- [ ] `MAX_FOCUS_LABEL_FLIPS_PER_TICK = 1` declared in the
      `const`-block.
- [ ] Both passes share a single `candidate_economic_focus` call
      signature; the second pass overwrites the first (idempotent
      when no inputs change).
- [ ] Record on replay bus: `record_economic_focus(tick, label)`
      (only from the second pass to avoid double-recording).
- [ ] 3-test minimum: `phase_economic_focus_picks_agrarian_on_food`,
      `phase_economic_focus_picks_industrial_on_tier`,
      `phase_economic_focus_picks_sacred_on_belief`.

### 3.11 `phase_emergence` — orchestrator (slot 22)

**Signature.** Pre-existing at `crates/engine/src/emergence.rs:159`:

```rust
pub(crate) fn phase_emergence(&mut self);
```

**No signature change.** The body is unchanged; the only wiring work
is the call from `Simulation::tick`. The orchestrator already
clears `last_feed` / `last_ai_decisions` / `last_sentience` at the
top (`emergence.rs:160-162`), runs the seven sub-phases
(`emergence_ensure_genomes`, `emergence_culture`, `emergence_social`,
`emergence_psyche`, `emergence_genetics_sentience`,
`emergence_legends`, `emergence_civ_ai`), and calls
`apply_awakening_coupling` at the end
(`emergence.rs:530` → `emergence.rs:539-546`).

**DAG rationale.** Runs after every new phase, so the world scans
it performs (`emergence_culture` for `cluster_cultures`,
`emergence_social` for `SocialGraph` ties, `emergence_psyche` for
`Psyche.mood`) operate on the post-everyone-else ECS. The
`apply_awakening_coupling` call at the end mints a bounded
`belief` + `cohesion` pulse that closes the upward-causation loop
back to `phase_cohesion` (next tick) and `phase_diplomacy` (same
tick, via `religion`-flavoured threshold — see ADR-020 §"Risk —
feedback with `phase_diplomacy`").

**Checklist (existing test set must pass).**

- [ ] `tick_invokes_emergence_phase` at `engine.rs:3418` passes
      (deterministic saga graph + emergence feed across same-seed
      sims).
- [ ] `phase_order_includes_emergence` test (amended, §1) passes.
- [ ] `emergence_apply_awakening_coupling_*` tests at
      `emergence.rs:731-1098` pass.
- [ ] 3-test minimum (ADR-011) per sub-phase:
      `emergence_culture_drift_deterministic`,
      `emergence_social_pair_applies_event`,
      `emergence_psyche_mood_updates_on_needs`.

### 3.12 `phase_diffusion` — tail (slot 23)

**Signature.** Pre-existing at `engine.rs:1538`. **No change.**

**DAG rationale.** Remains the very last phase (per the amended
`phase_order_includes_emergence` test). Consumes `target_era`
derived from `research_tier()` (post-`phase_tech`) and
`emergence.last_sentience` via the `target_era` lineage derivation.

**Checklist.**

- [ ] No changes; the only verification is that
      `phase_diffusion_bumps_wardrobe_eras` (`engine.rs:3929`)
      still passes after the `PHASE_ORDER` amendment.

---

## 4. New `Simulation` accessors (shims over `state.*`)

The phantom-target tests at `engine.rs:5038-5040, 5057-5059, 5322-5324,
5354-5356` already call `sim.state.belief = 0` / `sim.state.cohesion = 0`
/ `sim.state.unrest = 0`, and the new phases need read accessors. The
following thin shims sit alongside the existing `Simulation::belief()` /
`Simulation::add_belief()` (which themselves become shims over
`state.belief`):

| Accessor | Signature | Field |
|----------|-----------|-------|
| `belief` / `add_belief` / `try_invoke_divine_power` | existing (`engine.rs:1019-1039`) | `state.belief: u64` |
| `cohesion` (new) | `pub fn cohesion(&self) -> u64` | `state.cohesion: u64` |
| `add_cohesion` (new) | `pub fn add_cohesion(&mut self, amount: i64)` (signed; negative decays; floored at 0) | `state.cohesion: u64` |
| `unrest` (new) | `pub fn unrest(&self) -> u64` | `state.unrest: u64` |
| `add_unrest` (new) | `pub fn add_unrest(&mut self, amount: i64)` (signed; floored at 0) | `state.unrest: u64` |
| `set_economic_focus` (new) | `pub fn set_economic_focus(&mut self, f: EconomicFocus)` | `state.economic_focus` |
| `set_dispossessed_permille` (new) | `pub fn set_dispossessed_permille(&mut self, p: u64)` (clamped 0..=1000) | `state.dispossessed_permille` |
| `set_society_mood` (new) | `pub fn set_society_mood(&mut self, m: f32)` (clamped -1..=1) | `state.society_mood` |
| `bump_temple_level` (new) | `pub fn bump_temple_level(&mut self, delta: i32)` (clamped 0..=5) | `state.temple_level` |
| `bump_garrison_level` (new) | `pub fn bump_garrison_level(&mut self, delta: i32)` (clamped 0..=5) | `state.garrison_level` |
| `economic_focus` (new) | `pub fn economic_focus(&self) -> EconomicFocus` | `state.economic_focus` |
| `society_mood` (new) | `pub fn society_mood(&self) -> f32` | `state.society_mood` |
| `dispossessed_permille` (new) | `pub fn dispossessed_permille(&self) -> u64` | `state.dispossessed_permille` |
| `temple_level` (new) | `pub fn temple_level(&self) -> u32` | `state.temple_level` |
| `garrison_level` (new) | `pub fn garrison_level(&self) -> u32` | `state.garrison_level` |

The pre-existing `add_cohesion` call at `emergence.rs:545`
(`self.add_cohesion(awakening_cohesion_gain(awakenings));`) requires
this shim to be in place before the engine PR lands.

---

## 5. Out-of-scope phases (deferred to follow-up PRs)

- **`phase_chronicle`** — referenced by the phantom-target tests at
  `engine.rs:4645, 4662, 4672`. The chronicle writer reads
  `state.chronicle` (now a real field, §2) and emits deduped
  golden-age / tech-breakthrough / awakening / faction-formation
  lines. Per ADR-020 §"Consequences" this is **not** in scope for
  the dormant-phase PR; it is documented in
  `EMERGENCE_AUDIT.md §2 #33` and will land in a separate ADR. The
  `phase_order_matches_tick_sequence` amendment above does not
  include `chronicle`; the phantom-target tests compile only once
  this follow-up PR lands.
- **`phase_disasters`** — referenced in ADR-020 §3.4 inputs but
  flagged as "if `phase_disasters` is wired later". Out of scope.
- **`phase_religion`** — the `religion::spread_religion` input to
  `phase_belief` is left as a TODO marker; the `phase_belief`
  per-tick cap is intact so the missing input is observationally a
  no-op (the cap absorbs the zero).

---

## 6. Replay-bus `record_*` methods (new on `ReplayLog`)

Per ADR-020 §"Consequences" the `.civreplay` format remains a complete
record. Add 8 new methods on `ReplayLog` (sibling to
`record_damage` / `record_voxel_write` / `record_mod_loaded`):

| Method | Tick stamp | Payload |
|--------|-----------|---------|
| `record_belief(tick, value, delta)` | yes | (value: u64, delta: i64) |
| `record_unrest(tick, value, delta)` | yes | (value: u64, delta: i64) |
| `record_cohesion(tick, value, delta)` | yes | (value: u64, delta: i64) |
| `record_dispossessed_permille(tick, value)` | yes | (value: u64) |
| `record_economic_focus(tick, label)` | yes | (label: &str) |
| `record_society_mood(tick, value)` | yes | (value: f32) |
| `record_temple_level(tick, value)` | yes | (value: u32) |
| `record_garrison_level(tick, value)` | yes | (value: u32) |

Each new method appends a typed `ReplayEvent` variant. The replay
loader (`apply_replay_*` in `engine.rs:926-958`) is extended with
matching `state.*` setters that route through the same shims as §4,
so determinism is preserved.

---

## 7. Cap-const grep guard (ADR-020 §4 risk)

The review-time grep is:

```text
rg -n 'MAX_(AWAKENING|MISERY|COHESION|RESEARCH|INSTITUTION|MOOD|STRAT|BELIEF|FOCUS)' crates/engine/src/engine.rs
```

After the engine PR lands, this grep must return **≥ 8 hits** (one
per bounded scalar writer). The implementer must add **3 new
consts** to make the count match the ADR-020 table:

- `MAX_BELIEF_PER_TICK = 200` (§3.4)
- `MAX_MOOD_STEP_PER_TICK = 0.05` (§3.7)
- `MAX_STRAT_STEP_PER_TICK = 50` (§3.8)

(`MAX_INSTITUTION_RISE_PER_TICK = 1`, `MAX_FOCUS_LABEL_FLIPS_PER_TICK = 1`,
`MAX_RESEARCH_PER_TICK = 5_000` round out the table; some are
re-declared with explicit names even when the existing fns enforce the
cap inline, so the grep result is grep-able.)

---

## 8. Determinism + perf guards

**Determinism.** Each new phase is seeded off
`state.rng_seed ^ self.state.tick ^ agent_id` (ChaCha8Rng — same
pattern as `emergence_psyche` at `emergence.rs:446`). The existing
replay bus is extended per §6. Two same-seed sims must remain
byte-identical after N ticks for all N (verify with the existing
`test_determinism` regression).

**Perf.** Per ADR-020 §4, the 11 new phases add **0.6 – 1.2 ms** at
5,000-agent populations; total tick moves from ~2 ms to ~3.2 ms. The
LOD policy (`LodPolicy`, `engine.rs:457`) is reused — Warm / Cold
tiers are scanned, Hot tiers are unaffected. The 4 ms tick-budget
guard is enforced; an over-budget tick emits
`emergence.branching` and surfaces a `tick_budget_exceeded` warning
on the replay bus (same path as `phase_voxel` for CA overrun).

**3-test minimum per phase (ADR-011).** Happy / boundary / decay.
Total of **33 new tests** across the 11 phases; all must pass before
ADR-020 is moved to Accepted.

---

## 9. Mechanical recipe summary

1. **Amend `PHASE_ORDER`** (`engine.rs:55-68`) to the 23-entry list in §1.
2. **Promote 11 fields** to `WorldState` (§2) + update `Default`.
3. **Move `Simulation::belief` → `WorldState::belief`**, update all
   `self.belief` call sites in `emergence.rs` and `engine.rs:422, 651, 717`.
4. **Add 11 new `phase_*` method skeletons** (§3.1–§3.10) — empty
   bodies first to satisfy the call sites and get the build GREEN.
5. **Wire the 12 new call sites** in `Simulation::tick` (§3).
6. **Add 11 `Simulation` accessors** (§4).
7. **Add 8 `ReplayLog::record_*` methods** (§6) + matching
   `apply_replay_*` setters.
8. **Amend 2 existing tests** in `engine.rs:3362, 3388` (§1).
9. **Add the 3 new consts** (§7) for the grep guard.
10. **Implement each phase body** in dependency order (§3.1 → §3.10),
    one PR per phase group, each PR keeping the build GREEN and
    adding the per-phase 3-test minimum.
11. **Once all 11 phases are GREEN**, land the `phase_emergence` call
    site (§3.11) and verify `tick_invokes_emergence_phase` passes.
12. **Re-run the full `civis-3d-verify` gate** to confirm the FR
    matrix and replay determinism still hold.

---

## 10. Cross-references

- ADR-020 — the design decision this patch plan operationalises.
- ADR-011 — N-Series Emergence Coupling Architecture (3-test minimum,
  shared gradient + named cap + FR-traceable + dashboard-observable).
- ADR-018 — Emergence Systems Bidirectional Coupling (the inventory of
  consumers this patch plan's producers feed).
- ADR-003 / ADR-determinism-dropped — replay determinism contract.
- ADR-010 — CA tick budget guard (4 ms cap; warning path).
- `docs/reports/EMERGENCE_AUDIT.md` — the audit this PR closes gap #1
  + partially closes gap #6.
- `docs/traceability/fr-3d-matrix.md` — the FR matrix this PR unblocks.
