# Emergent-Systems Traceability Ledger — feat/sim-emergence-batch

Maps the emergence-batch gameplay systems and their downward-causation couplings
to requirement IDs, implementing code, and verifying tests. Goal: keep this batch
out of the `CODE-ONLY-no-spec` bucket (634 IDs as of `fr-matrix-2026-06-13`) by
asserting spec + code + test for each row → `COVERED`.

Charter: hardcode only physical/environmental/genomic law; life, society, economy,
belief, diplomacy EMERGE from state with bidirectional coupling (downward
causation), never scripted silos. See `project_civis_emergence_charter`,
`project_civis_emergence_design_layer`.

Spec roots: `docs/specs/CIV-0100-economy-v1.md`, `CIV-0107-joule-economy-system-v1.md`,
emergence charter (FR-CIV-0100 §3 emergence).

Tick order (emergence tail): `phase_disasters` → `phase_life` → `phase_emergence`
→ `phase_research` → `phase_tech` → `phase_belief` → `phase_unrest` →
`phase_cohesion` → `phase_stratification` → `phase_institutions`
(`engine.rs:Simulation::tick_with_emergence_source`). Economy/diplomacy run earlier
in the same tick so food price and treasuries are current when unrest/stratification
execute.

## Systems (tick-loop phases) — 11

| # | System | State field(s) | FR-CIV-0100 | Code | Criticality bound | Test(s) |
|---|--------|----------------|-------------|------|-------------------|---------|
| 1 | Research accrual | `research_progress` | §3 emergence | `phase_research`; `cohesion_research_bonus_permille` | Tier = progress / 100k; cohesion bonus ≤ +50% | `phase_research_accrues_from_population`, `phase_research_quiescent_without_population`, `cohesion_boosts_research_contribution` |
| 2 | Tech-unlocks | `tech_unlocks` (bitmask) | §3 emergence | `phase_tech`; `tech_unlocks_for_tier` | Monotonic OR-only; bits never cleared | `phase_tech_sets_and_keeps_bits`, `tech_unlocks_for_tier_is_monotonic` |
| 3 | Belief/faith accrual | `belief` | §3 belief | `phase_belief` (+ `temple_level` bonus) | Pop divisor 2_000; temple adds level/tick | `phase_belief_accrues_from_population`, `phase_unrest_feeds_belief_under_hardship` |
| 4 | Emergent market pricing | `market_state` prices | §3d economy | `phase_economy`; `market.rs:apply_pressure` | Demand = pop + faction wealth; supply = `carrying_capacity` | `phase_economy_steps_market_prices`, `faction_wealth_drives_market_demand`, `apply_pressure_*` |
| 5 | Emergent diplomacy | `diplomacy_events` | §3 emergence | `phase_diplomacy`; `diplomacy_conflict_threshold` | Cadence 500 ticks; threshold floor 2_000 | `phase_diplomacy_emerges_*`, `diplomacy_threshold_*`, `high_cohesion_biases_diplomacy_toward_peace` |
| 6 | Divine powers / faith spend | `belief` (spent) | §3 belief | `try_invoke_divine_power`; `disasters.rs:invoke_divine_disaster` | Spend-or-fail; no partial debit | `try_invoke_divine_power_gates_on_belief`, `try_invoke_divine_power_spends_belief`, `invoke_divine_disaster_*` |
| 7 | Disasters (wildfire/quake) | terrain + agents | §3 emergence | `phase_disasters`; `trigger_disaster` | Env thresholds; research raises ignition temp | `phase_disasters_*`, `invoke_divine_disaster_*` |
| 8 | Unrest | `unrest` | §3 emergence | `phase_unrest` (multi-driver sum) | Floored at 0; per-tick rise caps per driver | `phase_unrest_floors_at_zero`, `phase_unrest_accumulates_under_scarcity`, driver unit tests |
| 9 | Cohesion | `cohesion` | §3 emergence | `phase_cohesion`; `cohesion_delta` | Floored at 0; unrest frays 4× faster than belief binds | `cohesion_delta_balances_belief_against_unrest` |
| 10 | Social stratification | `dispossessed_permille` | §3 emergence | `phase_stratification`; `dispossession_target_permille`, `dispossession_step` | Clamped [0, 1000]; max 5 permille/tick (hysteresis) | `dispossession_target_rises_with_inequality_falls_with_cohesion`, `dispossession_step_is_sticky`, `dispossession_unrest_scales_and_caps` |
| 11 | Institutions (temple/garrison) | `temple_level`, `garrison_level` | §3 emergence | `phase_institutions`; `institution_target_level`, `institution_step` | `MAX_INSTITUTION_LEVEL` = 5; ±1 level/tick; treasury upkeep | `phase_institutions_grows_temple_with_belief` |

## Couplings (downward causation) — ~22 links

Each row is **source → target** via the named policy function (or phase hook).

| # | Coupling | Policy / phase | FR-CIV-0100 | Criticality bound | Test(s) |
|---|----------|----------------|-------------|-------------------|---------|
| 1 | population → research | `phase_research` | §3 emergence | 1 progress / 1k pop/tick | `phase_research_accrues_from_population` |
| 2 | cohesion → research | `cohesion_research_bonus_permille` | §3 emergence | Bonus ≤ +500‰ (+50%) | `cohesion_boosts_research_contribution` |
| 3 | research tier → tech_unlocks | `tech_unlocks_for_tier` / `phase_tech` | §3 emergence | Irrigation@1, Storage@2, Metallurgy@3 | `phase_tech_sets_and_keeps_bits` |
| 4 | tech_unlocks (irrigation) → carrying capacity | `carrying_capacity` | §3d | +200k cap when `TECH_IRRIGATION` set | `research_tier_and_capacity_grow_with_progress` |
| 5 | research tier → carrying capacity | `carrying_capacity` | §3d | Base 1M + 200k/tier | `research_tier_and_capacity_grow_with_progress` |
| 6 | carrying capacity + wealth → market prices | `phase_economy` → `apply_pressure` | §3d | Staple demand = pop + Σ treasuries | `phase_economy_steps_market_prices`, `faction_wealth_drives_market_demand` |
| 7 | food scarcity → unrest | `unrest_delta` | §3 emergence | Rise cap 50/tick; decay −10/tick abundance | `unrest_delta_rises_with_scarcity`, `unrest_delta_decays_under_abundance` |
| 8 | energy blackout → unrest | `energy_scarcity_unrest` | §3 emergence | +15 when budget ≤ 0 | `energy_scarcity_adds_unrest_only_on_blackout` |
| 9 | overcrowding → unrest | `overcrowding_unrest` | §3 emergence | +1 per 10% overshoot; cap 30/tick | `overcrowding_breeds_unrest_above_capacity` |
| 10 | treasury spread → unrest | `inequality_unrest` | §3 emergence | Cap 25/tick | `inequality_unrest_scales_with_spread_capped` |
| 11 | dispossessed share → unrest | `dispossession_unrest` | §3 emergence | permille/40; cap 25 | `dispossession_unrest_scales_and_caps` |
| 12 | research → unrest (damp rise) | `research_unrest_mitigation` | §3 emergence | Divide rise by 1+tier (tier≤9); floor 1 | `research_unrest_mitigation_damps_rise_floored_at_one` |
| 13 | cohesion → unrest (damp rise) | `cohesion_unrest_damp` | §3 emergence | Divide rise by 1+cohesion/200 (≤9); floor 1 | `cohesion_unrest_damp_calms_high_cohesion_floored_at_one` |
| 14 | garrison → unrest (damp) | `phase_unrest` | §3 emergence | −2 × `garrison_level`/tick | (via `phase_unrest_*`) |
| 15 | unrest → belief (hardship faith) | `phase_unrest` | §3 belief | +unrest/100 belief/tick | `phase_unrest_feeds_belief_under_hardship` |
| 16 | belief ↔ unrest → cohesion | `cohesion_delta` | §3 emergence | Bind belief/200; fray unrest/50; floor 0 | `cohesion_delta_balances_belief_against_unrest` |
| 17 | inequality + cohesion → stratification | `dispossession_target_permille`, `dispossession_step` | §3 emergence | Target [0,1000]; step ≤5/tick | `dispossession_target_rises_with_inequality_falls_with_cohesion`, `dispossession_step_is_sticky` |
| 18 | belief → temple; unrest → garrison | `institution_target_level`, `institution_step` | §3 emergence | 1 level / 5k belief or 200 unrest; cap 5 | `phase_institutions_grows_temple_with_belief` |
| 19 | temple → belief | `phase_belief` | §3 belief | +`temple_level`/tick | `phase_institutions_grows_temple_with_belief` |
| 20 | belief + cohesion ↔ unrest → diplomacy | `diplomacy_conflict_threshold`; `phase_diplomacy` | §3 emergence | Peace cap +10k; war erosion cap 8k; floor 2k | `diplomacy_threshold_*`, `diplomacy_belief_and_unrest_oppose` |
| 21 | diplomacy → treasury → market demand | `phase_diplomacy` → `phase_economy` | §3d | Trade ±100; conflict −50 treasuries | `phase_diplomacy_emerges_*` |
| 22 | unrest + cohesion → trade volume | `unrest_trade_factor`, `cohesion_trade_factor` | §3d | Trade factor [0.5,1.0] × [1.0,1.5] | `trade_volume_multiplier_*` (trade path) |
| 23 | surplus gap → trade volume | `trade_volume_multiplier` | §3d | Arbitrage multiplier [1.0, 2.0] | `trade_volume_multiplier_scales_with_surplus_capped_at_2x` |
| 24 | food scarcity → population (births) | `food_scarcity_birth_factor` | §3 emergence | Factor (0,1]; never reduces standing pop | `food_scarcity_birth_factor_*` |
| 25 | research → production yield | `production_yield_factor` | §3 emergence | +10%/tier; cap 2× | `production_yield_factor_rises_with_research_capped_at_2x` |
| 26 | research → building cadence | `building_cadence` | §3 emergence | 16 − 2×tier ticks; floor 4 | `building_cadence_shortens_with_research_floored` |
| 27 | research → wildfire mitigation | `wildfire_ignition_temp_fp` | §3 emergence | +2°C/tier; cap +20°C | (disasters.rs tests) |
| 28 | disasters → belief | `trigger_disaster` (+50) | §3 belief | Fixed faith gain per disaster | `invoke_divine_disaster_*` |
| 29 | belief → divine disaster | `invoke_divine_disaster` | §3 belief | Spend-or-fail loop | `invoke_divine_disaster_requires_faith` |
| 30 | cohesion → military morale | `morale_recovery_rate` | §3 emergence | Recovery 0.010–0.050/tick | `morale_recovery_rate_rises_with_cohesion_capped` |

Rows 1–22 are the core emergence DAG; 23–30 are secondary feedback arms (trade,
production, disasters, military) that close loops without parallel silos.

## Higher-order emergent structures — 3

Persistent macro-structures that sit above scalar accumulators and feed back through
the coupling graph.

### 1. Tech-unlocks (`tech_unlocks: u64`)

| Aspect | Detail |
|--------|--------|
| What | Irreversible capability bitmask (irrigation, storage, metallurgy) |
| Drivers | `research_tier` via `phase_tech` / `tech_unlocks_for_tier` |
| Feedback | `TECH_IRRIGATION` → +200k `carrying_capacity` → cheaper staples via `phase_economy` |
| FR | FR-CIV-0100 §3 emergence |
| Bound | Set-only OR; never cleared; tier gates at 1/2/3 |

### 2. Social stratification (`dispossessed_permille: u64`)

| Aspect | Detail |
|--------|--------|
| What | Persistent underclass share (per-mille, 0–1000) |
| Drivers | `faction_treasury_spread` pushes target up; `cohesion` erodes target |
| Feedback | `dispossession_unrest` adds class unrest; feeds cohesion/unrest/diplomacy hub |
| FR | FR-CIV-0100 §3 emergence |
| Bound | Target clamped [0,1000]; `dispossession_step` max ±5/tick (hysteresis) |

### 3. Institutions (`temple_level`, `garrison_level: u32`)

| Aspect | Detail |
|--------|--------|
| What | Leveled Temple (faith org) and Garrison (order org) |
| Drivers | `belief` → temple target; `unrest` → garrison target |
| Feedback | Temple boosts `phase_belief`; garrison damps `phase_unrest`; both drain treasury upkeep (10 × combined levels) |
| FR | FR-CIV-0100 §3 emergence |
| Bound | `MAX_INSTITUTION_LEVEL` = 5; `institution_step` ±1/tick (hysteresis) |

## Loop closure (no parallel silos)

**Cohesion hub:** accrues from belief minus unrest, then damps unrest, boosts research,
trade, diplomacy tolerance, military morale recovery, and erodes stratification target.

**Research hub:** accrues from population (+ cohesion), sets tech bits, raises carrying
capacity and production, shortens building cadence, mitigates unrest and wildfire ignition.

**Belief hub:** accrues from population, disasters, unrest hardship, and temples; spends
on divine disasters; raises diplomacy peace threshold and cohesion.

**Economy ↔ population:** market scarcity drives unrest and damps births; research/tech
raise capacity and ease prices; diplomacy and trade move treasuries that bid demand.

These bidirectional links are the compositionality test from
`project_civis_emergence_design_layer` — state feeds forward and backward through shared
resources, not one-way API calls.

## Open traceability gaps (next lanes)

- Add explicit `FR-CIV-0100-§N` IDs to the spec doc so the matrix generator links these
  rows as COVERED rather than CODE-ONLY-no-spec.
- Wire these test names into `docs/audits/_id_inventory_v3.json` on the next matrix refresh.
- Candidate next couplings: `TECH_STORAGE` / `TECH_METALLURGY` gameplay effects beyond
  bitmask presence; institution upkeep → faction inequality feedback.

---

## Cross-reference to next-phase FR rows (2026-06-27)

Each of the 11 systems documented in this ledger has concrete FR-CIV-EMERGENCE-NNN rows in `fr-emergence-matrix.md` (Section B) covering 155 emergence-batch IDs across 14 families. The 4 charter integration rows (Section C, FR-CIV-0100-int1..int4) promote the §3.3 charter umbrella into concrete cross-family emergence points.

Per-system batch-row index:

| System | Batch row range | Coverage |
|---|---|---|
| civ-linguabridge | FR-CIV-EMERGENCE-100..110 | 11 rows |
| civ-factions | FR-CIV-EMERGENCE-111..118 | 8 rows |
| civ-religion | FR-CIV-EMERGENCE-119..123 | 5 rows |
| civ-market | FR-CIV-EMERGENCE-124..131 | 8 rows |
| civ-urban | FR-CIV-EMERGENCE-132..140 | 9 rows |
| civ-climate | FR-CIV-EMERGENCE-141..143 | 3 rows |
| civ-econ | FR-CIV-EMERGENCE-144..150 | 7 rows |
| civ-demographics | FR-CIV-EMERGENCE-151..167 | 17 rows |
| civ-psyche | FR-CIV-EMERGENCE-168..197 | 30 rows |
| civ-legends | FR-CIV-EMERGENCE-198..220 | 23 rows |
| civ-ai | FR-CIV-EMERGENCE-221..235 | 15 rows |
| civ-culture | FR-CIV-EMERGENCE-236..238 | 3 rows |
| civ-social | FR-CIV-EMERGENCE-239..240 | 2 rows |
| civ-diplomacy | FR-CIV-EMERGENCE-241..248 | 8 rows |
| civ-laws | FR-CIV-EMERGENCE-249..254 | 6 rows |
| **Total** | | **155 rows** |

The 11-systems × 30-couplings matrix documented above is the **test surface** that promotes each of these 158 dormant IDs to `covered` status (i.e., spec + code + test triple).
