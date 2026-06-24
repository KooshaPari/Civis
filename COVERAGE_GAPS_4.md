# COVERAGE_GAPS_4.md

Read-only coverage audit of `crates/engine/src/engine.rs` (v3 line 7022).
Focus: **pure policy functions** (factor / threshold / centroid helpers — pure
scalar functions whose entire behaviour is captured by their return value
given well-typed arguments, with no `&mut self` and no engine-internal
side effects beyond a parameter or read-only `&hecs::World` scan).

Methodology:

1. Enumerated every top-level `fn` in `engine.rs` (326 definitions).
2. Filtered to pure policy helpers (factor / threshold / centroid /
   step / selector / mapping) — 44 candidates.
3. For each candidate, searched the entire `crates/engine` test corpus
   (`#[cfg(test)]` module in `engine.rs` at L4308–7021, plus
   `crates/engine/tests/*`) for the function name with call-paren
   `fn_name(` to count **direct** unit tests, not indirect coverage via
   phase methods.
4. Searched production call sites to confirm "untested" means
   "no direct unit test", not "no callers".

All audited functions are below. "Tested?" = **dedicated** `#[cfg(test)]`
unit test exists in `engine.rs` or any `crates/engine/tests/*` file.

## 1. Full inventory — every pure policy helper in `engine.rs`

| # | fn | file:line | Tested? | Edge-cases-missing |
|---|----|-----------|---------|---------------------|
| 1  | `job_type_for_civilian_id` | engine.rs:128 | ✗ | id→JobType deterministic split (e.g. mod buckets, sparse ids, u64::MAX) |
| 2  | `tech_unlocks_for_tier` | engine.rs:3271 | ✓ (L6657, L6695) | tier u64::MAX bit-saturation |
| 3  | `food_scarcity_birth_factor` | engine.rs:3306 | ✓ (L4500, L4507, L4515, L4526) | price = 0 (divide-by-zero risk) |
| 4  | `unrest_delta` | engine.rs:3316 | ✓ (L4817, L4824) | price = 0, very large price, sign of delta |
| 5  | `faction_wealth_scarcity_shadow` | engine.rs:3334 | ✗ | comfort threshold, empty `Resources`, large `treasury`, deep scarcity saturation at `SCARCITY_BASELINE` |
| 6  | `faction_unrest_delta_from_shadow` | engine.rs:3353 | ✗ | sign of `shadow`, clamp at 0, large shadow |
| 7  | `energy_scarcity_unrest` | engine.rs:3361 | ✓ (L4834) | `energy_budget < 0` (defensive), exactly zero, large positive |
| 8  | `agent_misery_unrest` | engine.rs:3373 | ✓ (L4843) | empty world, no agents, max agents |
| 9  | `micro_cohesion_delta` | engine.rs:3388 | ✓ (L4872) | empty world, only positive-cohesion agents, sign of delta |
| 10 | `micro_social_trust_permille` | engine.rs:3419 | ✓ (L5568) | empty world, single agent, mixed signs |
| 11 | `sentience_research_bonus` | engine.rs:3443 | ✓ (L4974) | empty world, no sentient DNA, multiple sentient agents (capped) |
| 12 | `candidate_economic_focus` | engine.rs:3463 | ✓ (L5071) | empty inputs, exact-equal weights, tie-break |
| 13 | `production_yield_factor` | engine.rs:3491 | ✓ (L5137) | tier > 9 (saturate at 2.0), tier u64::MAX |
| 14 | `morale_recovery_rate` | engine.rs:3500 | ✓ (L5148) | cohesion = 0, cohesion u64::MAX, slope at midpoint |
| 15 | `overcrowding_unrest` | engine.rs:3511 | ✓ (L5003) | population = capacity (zero overshoot), `capacity = 0` (divide-by-zero risk), very large population |
| 16 | `cohesion_research_bonus_permille` | engine.rs:3524 | ✓ (L5015) | cohesion = 0, saturate at +500, very large |
| 17 | `faction_treasury_spread` | engine.rs:3530 | ✓ (L5127) | empty `treasury` map, single faction, equal treasuries, sign of spread |
| 18 | `inequality_unrest` | engine.rs:3548 | ✓ (L5023) | spread = 0, negative spread, large spread, cap |
| 19 | `dispossession_target_permille` | engine.rs:3557 | ✓ (L5031) | spread = 0, cohesion u64::MAX, target saturate at 1000 |
| 20 | `institution_target_level` | engine.rs:3569 | ✓ (L5044) | `signal < per_level` → 0, large signal, `per_level = 0` (divide-by-zero) |
| 21 | `institution_step` | engine.rs:3575 | ✓ (L5054) | `current = target` (no-op), `current < target`, `current > target` |
| 22 | `dispossession_step` | engine.rs:3587 | ✓ (L5111) | `current = target`, `current < target`, step size edge |
| 23 | `dispossession_unrest` | engine.rs:3597 | ✓ (L5119) | dispossessed = 0, large dispossessed, cap |
| 24 | `research_unrest_mitigation` | engine.rs:3607 | ✓ (L5186) | `rise ≤ 0` (pass-through), tier u64::MAX, large rise |
| 25 | `building_cadence` | engine.rs:3619 | ✓ (L5206) | tier = 0 (max cadence), tier u64::MAX (min cadence), floor at 1 |
| 26 | `building_demand_signals` | engine.rs:3630 | ✓ (L5217) | empty inputs, all-zero multipliers, very large multipliers |
| 27 | `building_material_headroom_permille` | engine.rs:3670 | ✗ | `stock ≤ reserve` → 0, `stock ≥ gate` → 1000, quadratic rolloff, `gate ≤ reserve` (inverted) |
| 28 | `building_affordable_parcel_count` | engine.rs:3682 | ✗ | `wood = 0` or `metal = 0` → 0, exact `WOOD_PER_PARCEL` boundary, overflow at u64::MAX |
| 29 | `building_signals_limited` | engine.rs:3699 | ✗ | 0/1/2/3/4 saturated signals, `max_parcels = 0`, sort stability, ties on strength |
| 30 | `fc3_commercial_metal_steady_ceiling_i64` | engine.rs:3731 | ✓ (L5247) | cohesion = 0 (zero ceiling), cohesion u64::MAX, mid-cohesion |
| 31 | `building_parcel_count` | engine.rs:3742 | ✗ | empty `signals`, all-zero strengths, overflow on `usize` sum |
| 32 | `building_material_cost` | engine.rs:3755 | ✗ | `parcel_count = 0` (zero cost), large `parcel_count`, `Fixed` overflow |
| 33 | `building_materials_affordable` | engine.rs:3765 | ✗ | exact threshold, `parcel_count = 0`, negative `Fixed` inputs |
| 34 | `cohesion_delta` | engine.rs:3779 | ✓ (L5338) | `belief = 0`, `unrest = 0`, both equal, sign of delta, large inputs |
| 35 | `cohesion_unrest_damp` | engine.rs:3787 | ✓ (L5348) | `rise ≤ 0` (pass-through), cohesion = 0, large rise, damp saturate |
| 36 | `trade_volume_multiplier` | engine.rs:3805 | ✓ (L5672, L5683) | `from_stock = to_stock` (mid), `to_stock = 0`, `from_stock` very large, overflow, cap at 2.0 |
| 37 | `unrest_trade_factor` | engine.rs:3821 | ✓ (L5524) | `unrest = 0` (unity), large unrest, floor at 0.5 |
| 38 | `society_trade_factor` | engine.rs:3838 | ✓ (L5549) | both inputs = 0 (unity), saturate, mixed signs |
| 39 | `cohesion_trade_factor` | engine.rs:3849 | ✓ (L5539) | cohesion = 0 (unity), large cohesion, cap at 1.5 |
| 40 | `relation_trade_factor` | engine.rs:3855 | ✓ (L5660) | `relation = 0` (unity), `relation = ±1` (cap), `relation > 1` (clamp), `relation < -1` (clamp) |
| 41 | `diplomacy_conflict_threshold` | engine.rs:3904 | ✓ (L4541–4591) | base case, belief saturate, unrest saturate, min-floor, max-cap at 2x |
| 42 | `diplomacy_relation_threshold_bias` | engine.rs:3911 | ✗ | `relation = 0` → 0, `relation = ±1` → ±SPAN, `\|relation\| > 1` (clamp), NaN guard |
| 43 | `diplomacy_culture_threshold_bias` | engine.rs:3919 | ✓ (L4601) | identical traits (positive bias), divergent traits, no overlap, cap |
| 44 | `settlement_dominant_factions` | engine.rs:3936 | ✗ | empty world, no clusters, single-faction cluster, ties (lower id wins), sub-threshold cluster dropped |
| 45 | `settlement_contact_pairs` | engine.rs:3978 | ✗ | empty `clusters`, in-contact pair (≥ min), out-of-contact pair, canonical `(a, b)` ordering |
| 46 | `diplomacy_faction_pairs_from_settlement_contact` | engine.rs:4025 | ✗ | same-faction contact filtered, canonical ordering, empty `contacts` |
| 47 | `diplomacy_pair_from_settlement_overlap` | engine.rs:4045 | ✓ (L4647) | same-faction overlap, no overlap, empty `dominant` |
| 48 | `decay_faction_relations` | engine.rs:4086 | ✗ | `factor = 0` (no decay), `factor = 1` (full step toward 0), `factor > 1` (clamp), `score = 0`, sign of `score`, overshoot guard |
| 49 | `canonical_faction_pair` | engine.rs:4127 | ✗ | `a < b`, `a = b`, `a > b`, large ids |
| 50 | `emergent_route_goods` | engine.rs:4136 | ✗ | out-of-range `from`, determinism for stable id |
| 51 | `carrying_capacity` (inherent method) | engine.rs:1410 | ✓ (L6685, L6709, L6720) | zero population, large population, capacity vs. births |
| 52 | `last_tick_damage_centers` | engine.rs:1315 | ✗ | empty `pulses`, weighted centroid, ties |
| 53 | `life_cluster_position_fingerprint` | engine.rs:2373 | ✗ | empty world, single entity, hash stability across runs |

## 2. Statistics

- **Pure policy helpers audited:** 53
- **With dedicated unit test:** 32 (60%)
- **Without dedicated unit test:** 21 (40%)

The 21 untested helpers split into three risk tiers:

- **High** (coupling primitives with N1–N4 or FC-3 criticality and
  no test coverage at all): 9
- **Medium** (selector / step / cost helpers with subtle
  edge cases): 8
- **Low** (trivial wrapper / mapping): 4

## 3. Highest-value 8 untested or under-tested pure fns

Each entry is one **policy fn** with a **one-line test spec** describing the
smallest `#[cfg(test)]` block that would close the gap. These are
ordered from the most criticality-bearing / coupling-primitive gap
to the most edge-case-heavy.

1. **`faction_wealth_scarcity_shadow`** (engine.rs:3334) — N1
   downward-causation shadow for faction unrest. Test: build a `Resources`
   with food=0, treasury=0 → shadow saturates at `SCARCITY_BASELINE`;
   with food=large, treasury=large → shadow = 0; with food below
   `COMFORT_THRESHOLD` but treasury above → shadow = 0 (treasury hedges).

2. **`settlement_dominant_factions`** (engine.rs:3936) — N3 centroid
   helper that drives the N3 settlement→diplomacy bridge. Test: empty
   `WorldState` → empty `BTreeMap`; single settlement with one faction at
   population X → `(faction, X)`; ties (two factions at X) → lower
   `faction_id` wins; sub-`MIN_CLUSTER_POPULATION` cluster omitted.

3. **`settlement_contact_pairs`** (engine.rs:3978) — N3 centroid
   helper that produces `(a, b)` pairs in canonical order. Test: empty
   `clusters` → empty set; in-contact pair (centroid distance ≤
   `CONTACT_DISTANCE`) returned once as `(min, max)`; out-of-contact
   pair omitted; multiple-contacted pair returned exactly once.

4. **`diplomacy_faction_pairs_from_settlement_contact`** (engine.rs:4025) —
   N3 selector that dedupes self-pairs and canonicalises. Test: contacts
   `[(0,0), (1,2), (2,1)]` → `[(1,2)]` (self-filtered, canonical
   ordering, deduped); empty → empty; `(0,1)` and `(1,0)` collapse.

5. **`building_material_headroom_permille`** (engine.rs:3670) — FC-3
   criticality: this number is the building throttle that closes the
   N1→FC-3 construction loop. Test: `stock = reserve` → 0;
   `stock = gate` → 1000 (saturated); `stock = (reserve+gate)/2` →
   quadratic midpoint; `stock < reserve` → 0; `gate ≤ reserve` → clamps
   to 0 (no negative headroom).

6. **`building_signals_limited`** (engine.rs:3699) — FC-3 selector
   that picks the top-N strongest demand channels for this tick.
   Test: 4 saturated signals + `max_parcels=2` → only top 2 by
   `strength` retained, in original `kind` slots; `max_parcels=0` →
   all zeros; `max_parcels ≥ saturated` → untouched; ties on
   `strength` keep lower-index slot.

7. **`building_material_cost` + `building_materials_affordable`**
   (engine.rs:3755, 3765) — FC-3 material gate. Test:
   `material_cost(0) = (0, 0)`; `material_cost(1) = (WOOD_PER_PARCEL,
   METAL_PER_PARCEL)`; `affordable(WOOD_PER_PARCEL, METAL_PER_PARCEL, 1)
   = true` (exact); `affordable(WOOD_PER_PARCEL - 1, METAL_PER_PARCEL,
   1) = false` (off-by-one); large `parcel_count` does not panic
   (overflow guard).

8. **`diplomacy_relation_threshold_bias`** (engine.rs:3911) — N2
   threshold-bias primitive. Test: `score = 0.0` → 0; `score = 1.0` →
   `+FACTION_RELATION_THRESHOLD_SPAN`; `score = -1.0` → `-SPAN`;
   `score = 2.0` → `+SPAN` (clamped); `score = -2.0` → `-SPAN`
   (clamped); sign monotonic in `score`.

Runners-up (next 8 if the top 8 close cleanly): `decay_faction_relations`
(criticality decay primitive, sign + clamp edges), `faction_unrest_delta_from_shadow`
(trivial wrapper but ties the N1 shadow into the unrest channel),
`canonical_faction_pair` (deterministic ordering helper used by the
trade-emergence memory), `emergent_route_goods` (resource→string
mapping for the trade bus), `building_parcel_count` (saturating count,
no test for `usize` overflow), `building_affordable_parcel_count`
(min-of-two integer division edges), `job_type_for_civilian_id` (the
deterministic id→JobType split feeding the spawn palette), and
`life_cluster_position_fingerprint` (the cluster identity hash used by
the diff between consecutive ticks).

## 4. Notes

- This audit is **read-only** — no `cargo` runs, no edits, no commits.
- "Tested?" requires a **direct** `fn_name(` invocation in a test body,
  matching the convention used by the existing `phase_*`, `food_scarcity_*`,
  `diplomacy_conflict_threshold`, etc. families. Indirect coverage
  through `phase_*` methods does not count.
- The same audit pattern applies to other v3 coupling files
  (`crates/coupling/*`, `crates/policy/*`) but they are out of scope
  for this gap report.
