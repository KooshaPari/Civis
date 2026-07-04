# ADR-018: Emergence Systems Bidirectional Coupling via Shared Gradients + Conserved Resources

## Status

Proposed

## Date

2026-06-23

## Context

Civis is built as a stack of emergent layers — **language**, **faction**,
**religion**, **trade**, **architecture**, and **climate** — each documented by
its own substrate decision (ADR-014 phoneme-drift language, ADR-015 k-means
ideology clustering for factions, ADR-016 needs-vector religion; trade routes
in `civ-economy`; building emergence in `civ-build`; climate in `civ-climate`).
ADR-011 ("N-Series Emergence Coupling Architecture") laid down the **contract**
that every coupling must follow — shared gradient, bidirectional, bounded,
3-test minimum, FR-traceable, dashboard-observable — and enumerated N5 / N6 /
N7 / N8 / N9 / N10 / N11 / N12 in `crates/engine/src/engine.rs`.

What ADR-011 did **not** record is the actual mesh: which emergence layer
**causes** which other layer to change, in which direction, via which conserved
budget, and in which crate the coupling call site lives. Without that mesh, the
N-series reads as a numbered list rather than a coupled system, and a new
contributor cannot tell whether a proposed N-coupling is duplicating an
existing pathway or whether a missing one is actually a design gap.

This ADR closes that gap. It is a **cross-cutting, top-down** companion to
ADR-011, ADR-014, ADR-015, ADR-016 and the various substrate ADRs — it does
not replace them. Where ADR-011 defines the *contract* for an N-coupling,
this ADR enumerates the *instances* of the contract that the codebase
currently exposes, grouped by emergence layer and the gradients / conserved
resources they share.

The two architectural commitments are:

1. **Shared gradient, not API call.** Layers couple by reading/writing the
   same value on `Simulation` state (`belief`, `cohesion`, `unrest`,
   `wealth`, `energy_budget_joules`, `prices`, `culture_traits`,
   `language_centroid`, `climate`) — not by importing each other as
   dependencies. This avoids crate-graph cycles and keeps every coupling
   tick-local.
2. **Downward causation is real, upward causation is real, and every
   direction is bounded by a conserved resource.** Downward paths
   (macro → micro) include belief → cohesion, culture → trade friction,
   climate → biome → spawn success, faction language → trade friction,
   species aggression → diplomacy threshold. Upward paths (micro → macro)
   include mean agent misery → unrest, mean kinship → cohesion, mean trust →
   trade boost, mean affinity → diplomacy threshold, sentience crossings →
   bounded belief pulse. Every upward path is clamped to a per-tick cap
   (e.g. `MAX_AWAKENING_COHESION_PER_TICK`, `MAX_MISERY_UNREST`,
   `MICRO_BIND_CAP`); every downward path is clamped to a fixed factor
   range (e.g. `unrest_trade_factor ∈ [0.5, 1.0]`, `language_trade_factor
   ∈ [0.5, 1.0]`, `cohesion_trade_factor ∈ [1.0, 1.5]`). The cap is the
   invariant that keeps the system on the edge of chaos rather than
   collapsing or exploding.

The two **conserved resources** that anchor almost every coupling are:

- **Joule/energy budget** (`energy_budget_joules`, `civ-economy::EconomyState`)
  — the macro economic substrate, clamped to non-negative every tick
  (`drain_energy_budget` + `step` in `phase_economy`).
- **Macro belief / cohesion / unrest** triplet on `Simulation` (`belief`,
  `cohesion`, `unrest`) — the social-state conservation law, with all
  deltas bounded by per-tick caps defined in `engine.rs`.

A secondary conserved resource — **faith/legends significance** — is
captured in the saga graph but, by design, is *not* a coupling gradient:
the saga graph is a measured record of what the sim already produced, not
a generator of outcomes (see `crates/legends/src/lib.rs:13-14`).

## Decision

The six emergence layers couple through a fixed set of shared gradients.
The table below enumerates the **current** coupling surface, with the
crate / file / function that owns each direction, the gradient it
propagates along, and the conserved resource that bounds it. The N-series
column references ADR-011. Every row is implemented in the codebase as of
Snapshot 3 (2026-06-21) and is exercised by at least one `#[test]` in the
referenced file unless explicitly marked `partial`.

| # | From layer | To layer | Direction | Gradient | Conserved resource / cap | Crate / file / symbol | N-series | FR |
|---|------------|----------|-----------|----------|--------------------------|------------------------|----------|----|
| 1 | **Climate** | Language / Faction / Religion / Trade | Downward | `climate.mean_temp_c`, `weather_grid`, `geology_map`, `tide_offset` | `ClimateParams::co2_sensitivity` × `feedback_factor`; sea-level sensitivity bounded by `SEA_LEVEL_SENSITIVITY_M_PER_C` | `crates/climate/src/lib.rs:ClimateState::step`; `crates/engine/src/engine.rs:phase_planet` (`compute_climate`, `compute_weather`, `apply_tide_offset`); `crates/engine/src/emergence.rs:select_seed_for_position` (biome → seed choice) | (substrate) | FR-CIV-PLANET-020 / -030; FR-CIV-014 map-seed determinism |
| 2 | **Climate** | Diplomacy (war eligibility) | Downward | `tide_offset`, climate-mediated food scarcity | `faction_wealth_scarcity_shadow` clamped to `FOOD_SCARCITY_BASELINE` + shortfall/4; `unrest_delta` clamp `[-DECAY, MAX_RISE]` | `crates/engine/src/engine.rs:faction_wealth_scarcity_shadow` (2039), `unrest_delta` (2000) | (substrate) | FR-CIV-0100 §3 |
| 3 | **Climate** | Architecture (spawn biome) | Downward | `geology_map.biome_at_normalized` → `SeedDefinition::spawn_biome_affinity` | (selection is deterministic, no amplification) | `crates/engine/src/emergence.rs:select_seed_for_position` (126) | (substrate) | FR-CIV-014 |
| 4 | **Language** | Trade (friction) | Downward | `language_trade_factor(distance)` ∈ [0.5, 1.0] | `LANGUAGE_TRADE_PENALTY_PERMILLE = 500`; cap-floor 0.5 | `crates/engine/src/engine.rs:language_trade_factor` (2736); `crates/engine/src/language.rs:LanguageState` | N5 | FR-CIV-LANG-001 |
| 5 | **Language** | Faction (centroid) | Upward (carries) / Downward (projected) | `faction_language_centroids` (member-weighted `[f32; 4]`) | cluster membership ≥ 2 required to anchor a centroid (lone wanderer ignored) | `crates/engine/src/engine.rs:faction_language_centroids` (2875) | N5 | FR-CIV-LANG-001 / FR-CIV-PSYCHE-912 |
| 6 | **Faction** | Trade (volume + relation) | Downward | `relation_trade_factor(relation)` ∈ [0.5, 1.5]; `cohesion_trade_factor` ∈ [1.0, 1.5]; `society_trade_factor` ∈ [1.0, 1.75] | `COHESION_TRADE_CAP_PERMILLE = 500`; `SOCIETY_TRADE_BOOST_CAP_PERMILLE = 750`; `UNREST_TRADE_FLOOR_PERMILLE = 500` | `crates/engine/src/engine.rs:relation_trade_factor` (2725), `cohesion_trade_factor` (2719), `society_trade_factor` (2708), `unrest_trade_factor` (2691) | N5 (continues) | FR-CIV-ECON-001 |
| 7 | **Faction** | Diplomacy (conflict threshold) | Downward | `diplomacy_conflict_threshold(belief, unrest)`, `diplomacy_culture_threshold_bias`, `diplomacy_relation_threshold_bias` | `DIPLOMACY_BASE_CONFLICT_THRESHOLD = 10_000`; `DIPLOMACY_MIN_CONFLICT_THRESHOLD = 2_000` (floor); `BELIEF_PEACE_CAP = 10_000`; `UNREST_WAR_CAP = 8_000`; `CULTURE_PEACE_SPAN = 3_000` | `crates/engine/src/engine.rs:diplomacy_conflict_threshold` (2784), `diplomacy_culture_threshold_bias` (2809), `diplomacy_relation_threshold_bias` (2801) | (substrate of N5) | FR-CIV-0100 §3 |
| 8 | **Faction** | Architecture (dominant alignment per settlement) | Downward | `settlement_dominant_factions` (per-cluster) → building graph demands | tie-broken by `faction_id` for determinism | `crates/engine/src/engine.rs:settlement_dominant_factions` (2826) | N3 | FR-CIV-LIFE-030 |
| 9 | **Religion / Belief** | Cohesion | Downward | `cohesion + peace` (peace is the *belief* contribution to the diplomacy threshold; cohesion is a macro state that belief also mints directly) | `BELIEF_PEACE_DIVISOR = 50`; `BELIEF_PEACE_CAP = 10_000` | `crates/engine/src/engine.rs:diplomacy_conflict_threshold` (peace leg, 2784) | (N5 family) | FR-CIV-0100 §3 |
| 10 | **Religion** | Cohesion (ritual) | Downward (within religion) / Upward (to macro) | `spread_religion` `cohesion_gain = shared_ritual_performance * 0.2`; `shared_ritual_performance = ritual_load * nearby_factor`, clamped to [0, 1] | religion.member_count ≥ 1; cohesion clamped per-tick | `crates/engine/src/religion.rs:spread_religion` (69) | (N16 family) | FR-CIV-016 / FR-CIV-LEGENDS-001 |
| 11 | **Religion / Belief** | Sentience crossings (bounded pulse) | Downward | `awakening_belief_gain(awakenings)`; `awakening_cohesion_gain(awakenings)` | `MAX_AWAKENING_BELIEF_PER_TICK`, `MAX_AWAKENING_COHESION_PER_TICK = 10`; `COHESION_PER_AWAKENING = 2`; per-tick cap; no double-counting (additive only) | `crates/engine/src/engine.rs:awakening_cohesion_gain` (2650); `crates/engine/src/emergence.rs:apply_awakening_coupling` (539) | N7 | FR-CIV-GENETICS / FR-CIV-LEGENDS-001 |
| 12 | **Sentience / Genetics** | Saga graph (record) | Downward (record) | `last_sentience` → `RawSimEvent{SpeciationEvent, SourceCrate::Genetics}` → `SagaGraph::ingest` | (recording only; no amplification; saga is measured record, not generator) | `crates/engine/src/emergence.rs:emergence_legends` (548), `emergence_genetics_sentience` (472) | N6 / N7 | FR-CIV-LEGENDS-INGEST-02 |
| 13 | **Trade** | Faction treasury (conservation) | Downward | `tick_trade_routes`: `quantity * (1 + margin/100)` profit | (per-route volume > 0; per-route from ≠ to; conservation via `adjust_resource` zero-sum) | `crates/engine/src/engine.rs:tick_trade_routes` (1781) | (substrate) | FR-CIV-ECON-001 |
| 14 | **Trade** | Diplomacy (relation drift) | Downward | `apply_signal` with `DIPLOMACY_TRADE_DRIFT = 0.08`; `FACTION_TRADE_RELATION_SIGNAL = 0.05/0.08` | `DIPLOMACY_TRADE_DRIFT` is a per-step bounded increment | `crates/engine/src/engine.rs:diplomacy_conflict_threshold` (relation leg); `crates/diplomacy/src/lib.rs:Relation` (`apply_signal`); `crates/engine/src/engine.rs:phase_diplomacy` (1695) | (N5 family) | FR-CIV-DIPLO-001 |
| 15 | **Trade** | Saga graph (record) | Downward (record) | `DiplomacyKind::TradeAgreement` → `EventKind::EconomicBoom` → `SagaGraph::ingest` | (recording only) | `crates/engine/src/emergence.rs:emergence_legends` (548) | N6 | FR-CIV-LEGENDS-INGEST-02 |
| 16 | **Architecture** | Saga graph (record) | Downward (record) | building / structure events flow into `civ-legends` when the producer emits them | (recording only) | `crates/legends/src/lib.rs:RawSimEvent` producers; `crates/engine/src/emergence.rs:emergence_legends` (548) | N6 | FR-CIV-LEGENDS-INGEST-02 |
| 17 | **Psyche / Agent mood** | Unrest (mean misery) | Upward | `agent_misery_unrest` = mean of `-psyche.mood.valence` mapped to `[0, MAX_MISERY_UNREST = 30]` | `MAX_MISERY_UNREST = 30` (hard cap) | `crates/engine/src/engine.rs:agent_misery_unrest` (2078) | N11 (family) | FR-CIV-0100 §3 |
| 18 | **Psyche / Agent mood** | Cohesion (consensus) | Upward | `micro_cohesion_delta` = consensus-based bind/fray in `[-MICRO_FRAY_CAP, MICRO_BIND_CAP]` = `[-18, +12]` | `MICRO_BIND_CAP = 12`, `MICRO_FRAY_CAP = 18`; `MIN_AGENTS = 2` (need a population) | `crates/engine/src/engine.rs:micro_cohesion_delta` (2093) | N10 (family) | FR-CIV-0100 §3 |
| 19 | **Psyche / Social tie trust** | Trade (volume boost) | Upward | `micro_social_trust_permille` cached and read by `society_trade_factor` | `MICRO_TRUST_CAP = 250`; combined with cohesion capped at `SOCIETY_TRADE_BOOST_CAP_PERMILLE = 750` | `crates/engine/src/engine.rs:micro_social_trust_permille` (2124), `society_trade_factor` (2708) | (N10 family) | FR-CIV-0100 §3 |
| 20 | **Psyche / Social tie affinity** | Diplomacy (threshold bias) | Upward | `avg_social_affinity` ∈ [-1, 1] → `N12_AFFINITY_BIAS_SCALE = 5_000` permille bias | bias clamped to `[-5_000, 5_000]`; combined threshold floored at `DIPLOMACY_MIN_CONFLICT_THRESHOLD` | `crates/engine/src/engine.rs:avg_social_affinity` (2179), `N12_AFFINITY_BIAS_SCALE` (2200) | N12 | FR-CIV-EMERGENCE-N12 |
| 21 | **Psyche maturity** | Belief (stability) | Upward (implied) | `avg_psyche_maturity` (engine hook; mature populations damp belief noise) | (planned, partial — used as a stabilizing input) | `crates/engine/src/engine.rs:avg_psyche_maturity` (2148) | N11 | FR-CIV-EMERGENCE-N11 |
| 22 | **Social tie kinship** | Cohesion (family bonds) | Upward | `avg_faction_kinship` (engine hook; family ties boost macro cohesion) | (planned, partial) | `crates/engine/src/engine.rs:avg_faction_kinship` (2160) | N10 | FR-CIV-EMERGENCE-N10 |
| 23 | **Species aggression** | Diplomacy (conflict threshold) | Downward | `aggression_threshold_reduction` reduces the conflict threshold by `mean_aggression * AGGRESSION_CONFLICT_BOOST` | `AGGRESSION_CONFLICT_BOOST = 3_000`; floored at `DIPLOMACY_MIN_CONFLICT_THRESHOLD` | `crates/engine/src/engine.rs:aggression_threshold_reduction` (2796); `crates/engine/src/emergence.rs:faction_aggression` rebuilt in `emergence_genetics_sentience` (502) | N9 | FR-CIV-EMERGENCE-N9 |
| 24 | **Economy / Non-food scarcity** | Unrest (cost-of-living) | Downward | `commodity_unrest_delta` over non-food prices, food skipped (unrest_delta owns food) | `MAX_RISE = 15`; `DECAY = 5`; bounded `[-DECAY, MAX_RISE]` | `crates/engine/src/engine.rs:commodity_unrest_delta` (2018) | N8 | FR-ECON-001 / FR-CIV-ECON |
| 25 | **Energy blackout** | Unrest (acute shock) | Downward | `energy_scarcity_unrest` adds `BLACKOUT_UNREST = 15` on full drain | flat 0/15; `phase_economy` clamps budget to non-negative | `crates/engine/src/engine.rs:energy_scarcity_unrest` (2066) | (N8 family) | FR-CIV-0100 §3 |
| 26 | **Culture drift** | Faction (centroid traits) | Upward | `drift_populations` over per-cluster `CultureProfile.traits`; stable sort, no HashMap iteration order | drift `0.02`, `0.85` retention; cadence-gated feed event every 128 ticks | `crates/engine/src/emergence.rs:emergence_culture` (191); `civ-agents::culture::drift_populations` | (substrate) | FR-CIV-PSYCHE / FR-CIV-LEGENDS-INGEST-02 |
| 27 | **Cohesion** | Unrest (damping) | Downward (within macro) | `cohesion_unrest_damp` divides unrest rise by `1 + cohesion/200` (capped at 10) | divisor floor 1; pass-through for decay | `crates/engine/src/engine.rs:cohesion_unrest_damp` (2657) | (substrate) | FR-CIV-0100 §3 |
| 28 | **Cohesion** | Trade (volume boost) | Downward (within macro) | `cohesion_trade_factor` ∈ [1.0, 1.5] | `COHESION_TRADE_CAP_PERMILLE = 500` | `crates/engine/src/engine.rs:cohesion_trade_factor` (2719) | (substrate of N5) | FR-CIV-0100 §3 |
| 29 | **Births / Deaths** | Saga graph (record) | Downward (record) | `RawSimEvent{Birth, Death}` → `SagaGraph::ingest`; `mark_died` on death | (recording only) | `crates/engine/src/emergence.rs:emergence_legends` (548); `crates/legends/src/lib.rs:SagaGraph::mark_died` (47) | N6 | FR-CIV-LEGENDS-INGEST-02 |
| 30 | **Legend promotion** | Civ-AI naming (record) | Downward (record) | `civ_ai_sync_generate` produces deterministic name for `legend_promotion` events | (recording only; deterministic, no LLM) | `crates/engine/src/emergence.rs:emergence_civ_ai` (633) | (N6 family) | FR-CIV-AI-006 |

### How a coupling earns its place

For each row above, the answer to **all three** of these must be "yes":

1. **Is the gradient shared (not API-called)?** The producer writes a
   value that the consumer reads off the same `Simulation` (or the
   equivalent crate's state struct) — no `fn call_between_layers`.
2. **Is the cap real and named?** Every per-tick delta has a `const`
   cap nearby (`MAX_*`, `*_CAP`, `*_FLOOR`, `*_SPAN`). The cap is the
   invariant that keeps the system on the edge of chaos.
3. **Is there a downward path AND an upward path?** Either direction
   is the trivial one; the interesting couplings are those where the
   upward path is also a real feedback (e.g. `agent_misery_unrest`
   scanning the `hecs::World` for `Psyche.mood.valence`).

A row that fails any of these is **theatre** by the test in
ADR-011 §"Emergence dashboard observability" and is not accepted as an
N-coupling.

### What "downward causation" means here

Downward causation is the macro-to-micro direction: the social / climate
/ economy layer changes a per-tick scalar (e.g. `unrest_trade_factor`)
that the micro layer (a single trade route) reads. It is **not** the
same as "macro writes to a single agent's field" — that would be
authored drama. Downward causation in Civis is **always** a **factor**
or **bias** on a probability / rate, never a hard override of a single
agent's state.

Examples:

- `unrest_trade_factor(unrest)` — macro unrest scales the trade-route
  volume factor in `[0.5, 1.0]`. No single route is killed; the *whole*
  flow is damped, with a floor of 50 % so trade never stops.
- `diplomacy_conflict_threshold(belief, unrest)` — macro belief and
  unrest shift the wealth-disparity a faction pair must reach before
  conflict, bounded by `DIPLOMACY_MIN_CONFLICT_THRESHOLD` so conflict
  always needs *some* disparity.
- `cohesion_trade_factor(cohesion)` — macro cohesion raises trade
  volume in `[1.0, 1.5]`, capped at 50 % above baseline.
- `aggression_threshold_reduction(mean_aggression)` — species-wide
  aggression lowers the diplomacy threshold up to `AGGRESSION_CONFLICT_BOOST = 3_000`.

### What "upward causation" means here

Upward causation is the micro-to-macro direction: a per-tick aggregate
over the live `hecs::World` (or `civ_planet::GeologyMap`,
`civ_voxel::VoxelWorld`) is **read** by a macro function and
**added** to a macro budget. It is always:

- A **mean / sum / fraction** over the world, not a single agent's
  value.
- **Bounded** by a per-tick cap, so a million grieving agents cannot
  push unrest to infinity.
- **Causally upstream** of the macro state it modifies — i.e. the
  micro state would not be where it is without the macro state that
  produced it (otherwise it is theatre).

Examples:

- `agent_misery_unrest(world)` scans `&Psyche` for `-mood.valence`,
  returns `(mean * MAX_MISERY_UNREST) as i64`, capped at 30.
- `micro_cohesion_delta(world)` reads `psyche.beliefs[0]`, computes
  consensus, returns bind − fray in `[-18, +12]`.
- `avg_social_affinity(world)` reads `SocialGraph::ties[*].affinity`,
  returns mean in `[-1, 1]`, which `diplomacy_conflict_threshold` later
  scales into a threshold bias.
- `emergence_genetics_sentience` rebuilds `faction_aggression` as a
  per-faction mean of `express(dna).behavior.aggression` each tick
  (ephemeral; see
  `crates/engine/src/engine.rs:4947`), consumed the same tick by
  `aggression_threshold_reduction`.

### Why every coupling is bounded

The cap is the invariant that keeps the system on the edge of chaos.
Without caps:

- `agent_misery_unrest` would let a population of sad agents push
  unrest to a value the diplomacy layer cannot reason about, leading
  to runaway conflict.
- `unrest_trade_factor` would let unrest stall all trade, leading to
  starvation, leading to more unrest, leading to heat-death.
- `awakening_belief_gain` would let a single tick of mass sentience
  mint infinite belief, breaking the conservation law that
  `try_invoke_divine_power` depends on.

Caps are the engineering expression of the **edge-of-chaos** property
the emergence dashboard is designed to detect (see
`crates/civ-emergence-metrics/src/lib.rs:branching::classify_regime`,
`power_law::PowerLawFit`).

### Why the saga graph is NOT a coupling gradient

A common reviewer question is "is the saga graph a coupling?" The
answer is **no**: `civ-legends::SagaGraph::ingest` is a *recording* of
events that other layers already produced, scored for historical
significance, and exposed for query. It does not feed back into the
tick state. This is a deliberate charter decision
(`crates/legends/src/lib.rs:13-14`):

> A measured record of what the sim already produced, never a
> generator of outcomes.

If a future proposal wants the saga graph to influence gameplay, the
right move is to (a) read the saga graph as a *gradient source* in the
way a language or faction layer does — e.g. "promoted entities bias
diplomacy by +X" — and (b) record the new coupling as a row in this
table, with its own cap and its own upward path. The saga graph should
not become a side-channel that mutates the tick state directly.

## Alternatives Considered

- **Event bus between crates.** Cleaner crate boundaries, but each
  coupling is then asynchronous (one tick minimum lag) and the
  downward / upward symmetry is hard to enforce. Deferred to
  couplings that genuinely cross process boundaries (the
  `civ-mod-host` event bus is the existing example). The synchronous
  shared-gradient approach stays in-tree for in-tick couplings.
- **Per-coupling crate.** Maximum isolation, but crate-explosion
  (`civ-language-trade`, `civ-faction-religion`, `civ-climate-arch`,
  …) and the engine would import all of them. Rejected: the
  `engine.rs` shared-state approach keeps coupling code co-located and
  the 3-test minimum auditable in one place.
- **Shared-memory ECS components.** Bevy-style components would give
  O(1) random access, but require migrating the engine off `hecs`.
  Deferred — `hecs` is the current ECS and the existing coupling
  scan patterns (`for (_, graph) in world.query::<&SocialGraph>().iter()`)
  are deterministic and fast enough at the populations we run.
- **Parallel simulation silos.** Each layer runs independently and
  outputs are aggregated at display time. This is the
  **#1 anti-pattern** (theatre emergence) and is rejected
  categorically — see ADR-011 §"Alternatives considered".
- **Saga graph as a coupling layer.** See "Why the saga graph is NOT
  a coupling gradient" above. Recording ≠ coupling. If a future
  reviewer wants the saga to feed back, the proposal must take the
  same form as every other row in this table: shared gradient, cap,
  downward AND upward path.

## Consequences

- The N-series enumerations in ADR-011 (§"Known N-series couplings")
  remain the **contract** for adding a new coupling; this ADR is the
  **inventory** of couplings that already satisfy the contract.
  New N-series rows in ADR-011 should be cross-referenced from this
  table.
- Every row in this table is testable via the existing `#[test]`
  functions in the referenced file (the ADR-011 3-test minimum
  applies). Some rows (e.g. `avg_psyche_maturity`, `avg_faction_kinship`)
  are **partial** at the time of writing and should be completed
  before the next coupling that depends on them lands.
- A future N-coupling that wants to use a **new** shared gradient
  (e.g. "saga significance → diplomacy trust") must:
  1. Define a new bounded const in `engine.rs` (e.g.
     `MAX_SAGA_TRUST_PER_TICK`).
  2. Add a row to this table.
  3. Pass the 3-test minimum (happy / boundary / decay).
  4. Add an FR-CIV-EMERGENCE-Nxx row in ADR-011.
  5. Wire at least one emergence-dashboard metric
     (`crates/civ-emergence-metrics`) so the coupling is observable
     and not theatre.
- The conservation law on `belief` (`saturating_add` in
  `add_belief`), on `cohesion` (clamped at zero, capped per-tick), and
  on `energy_budget_joules` (clamped to zero in
  `phase_economy`) remains the **invariant layer** that all 30 rows
  inherit. A row that violates one of these conservation laws is
  rejected on review.
- The "downward causation" / "upward causation" terminology in this
  ADR is the same as in `crates/engine/src/engine.rs:2062-2200` and
  in `docs/specs/CIV-0100` §3 — the words are not new, the table is.
- The saga graph rows (12, 15, 16, 29, 30) are explicitly **recording
  only** and do not constitute bidirectional coupling in the sense
  of the other rows. They are listed for completeness so a future
  reviewer does not mistake them for missed coupling rows.

## Cross-References

- ADR-011 — N-series coupling **contract** (shared gradient,
  bidirectional, bounded, 3-test, FR-traceable, dashboard-observable).
  This ADR is its **inventory**; together they form the spec for
  adding a new N-coupling.
- ADR-014 — Language emergence substrate (phoneme drift).
  Implements the language row above.
- ADR-015 — Faction emergence substrate (k-means ideology clustering).
  Implements the faction centroid row above.
- ADR-016 — Religion emergence substrate (needs-vector).
  Implements the religion / belief rows above.
- ADR-002 — Joule economy as pluggable resource allocator.
  Anchors the `energy_budget_joules` conserved resource.
- ADR-003 / `ADR-determinism-dropped` — Replay determinism contract.
  Every coupling in the table is **deterministic** by construction
  (ChaCha8Rng, BTreeMap iteration, `MIN_AGENTS` guards) so the
  coupling surface can be replayed.
- ADR-008 — Algorithmic genetics (no LLM). Anchors the
  `faction_aggression` rebuild and the
  `emergence_legends` / `civ_ai_sync_generate` recording chain.
- ADR-010 — CA tick budget guard. Caps the per-tick cost of the
  world scans in the upward-causation rows (e.g.
  `agent_misery_unrest`, `avg_social_affinity`).
- `crates/legends/src/lib.rs:13-14` — Saga graph is a measured
  record, not a generator; explains why rows 12, 15, 16, 29, 30 are
  recording-only.
- `crates/civ-emergence-metrics/src/lib.rs:branching::classify_regime` —
  edge-of-chaos / heat-death / explosion classification; the
  observable the bounded caps protect.
- `docs/specs/CIV-0100` §3 — emergence engineering doc that
  originally named "downward causation" and "upward causation" in
  this codebase.
- `crates/emergence-coupling-audit.txt` — coupling audit snapshot
  this ADR is the durable record of.
- `crates/engine/src/engine.rs:1172` — `Simulation::tick` phase
  order; every coupling in the table is invoked by exactly one of
  these phases.
- `crates/engine/src/emergence.rs:159` — `phase_emergence` phase
  order; this is where the N7 / N9 / N11 / N12 / N15 couplings land.
