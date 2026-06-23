# FR TRACEABILITY SNAPSHOT 3 — Civis Engine Crate

Generated: 2026-06-16 (post-N5/N6/N7/N8 audit)
Scope: `crates/engine/src/engine.rs` only (lines 1–8318)
Method: read-only grep/classify per task; **NO cargo, NO git commit, NO source edits**
Classification: `phase_*` / helper fn = impl; `#[cfg(test)]` + `/// Covers` = test

## BASELINE (FR_TRACE_SNAPSHOT_2)

| Bucket | Count |
|--------|-------|
| FULL | 45 |
| IMPL-NO-TEST | 1 |
| SPEC-ONLY | 4 |
| **Total distinct FR-IDs** | **50** |

## CLASSIFICATION LEGEND

- **FULL** — FR-ID referenced in engine.rs comment AND has matching `phase_*` (or supporting) impl AND has `#[test]` reference in the same file
- **Family coupling FULL** — bare family-level FR-ID (no numbered suffix) with impl + test in engine.rs; tracks coupling work under a family umbrella without specific numbered sub-IDs
- **IMPL-NO-TEST** — FR-ID referenced in engine.rs comment AND has impl, but NO test reference in this file
- **SPEC-ONLY** — FR-ID (or family wildcard) referenced in engine.rs comment but NO impl, NO test in this file

## TOTALS BY BUCKET (engine.rs scope)

| Bucket | Count | vs Snapshot 2 |
|--------|-------|---------------|
| FULL (numbered IDs) | 48 | **+3** |
| Family coupling FULL (bare family refs) | 2 | **+2** (new bucket) |
| IMPL-NO-TEST | 1 | 0 |
| SPEC-ONLY | 4 | 0 |
| **Total** | **55** | **+5** |

<response>
FULL numbered: 48  (+3)
Family coupling FULL: 2  (+2)
IMPL-NO-TEST: 1  (0)
SPEC-ONLY: 4  (0)
</response>

## FULL TABLE — Numbered IDs (48 total)

| FR-ID | impl? | test? | bucket | delta vs SNAP2 |
|-------|-------|-------|--------|----------------|
| FR-CIV-0100 | yes (many `phase_*` fns: production, citizen_lifecycle, military, economy, planet, diplomacy, tactics, voxel, buildings, diffusion, research, tech, belief, unrest, faction_unrest, cohesion, social_mood, stratification, institutions, economic_focus, chronicle) | yes (24+ tests cover §3 downward/upward causation) | FULL | 0 |
| FR-CIV-0200 | yes (`phase_research`, `research_progress`, `research_tier`) | yes (phase_research_accrues_from_population, phase_research_quiescent_without_population, research_tier_divides_progress, research_tier_and_capacity_grow_with_progress) | FULL | 0 |
| FR-CIV-CA-009 | yes (`phase_voxel_ca`, `last_tick_abiogenesis_sites`) | yes (phase_voxel_ca_none_is_noop, phase_voxel_ca_warm_water_is_viable_stone_is_not) | FULL | 0 |
| FR-CIV-EMERGENCE | yes (`phase_belief`, `try_invoke_divine_power`, `add_belief`) | yes (try_invoke_divine_power_spends_belief, try_invoke_divine_power_gates_on_belief, phase_belief_accrues_from_population, belief_decays_toward_equilibrium) | FULL | 0 |
| FR-CIV-ENGINE-INT-001 | yes (phase_planet) | yes (climate_recomputes_every_tick) | FULL | 0 |
| FR-CIV-ENGINE-INT-002 | yes (phase_tactics drains pending_damage) | yes (pending_damage_drains_and_reduces_chunk_count) | FULL | 0 |
| FR-CIV-ENGINE-INT-003 | yes (phase_compact modulo 64) | yes (compact_runs_every_64_ticks) | FULL | 0 |
| FR-CIV-ENGINE-INT-005 | yes (phase_planet uses is_daytime) | yes (daytime_cycles_across_one_full_day) | FULL | 0 |
| FR-CIV-ENGINE-INT-010 | yes (`spawn_faction_civilians` + `spawn_initial_entities`) | yes (startup_spawns_128_civilians) | FULL | 0 |
| FR-CIV-ENGINE-INT-011 | yes (phase_buildings, building_graph) | yes (phase_buildings_allocates_over_time_when_signals_are_high) | FULL | 0 |
| FR-CIV-ENGINE-INT-012 | yes (phase_diffusion, propagate_wardrobe) | yes (phase_diffusion_bumps_wardrobe_eras) | FULL | 0 |
| FR-CIV-ENGINE-INT-013 | yes (tick determinism via seeded RNG) | yes (determinism_holds_with_all_phases_enabled) | FULL | 0 |
| FR-CIV-ENGINE-INT-014 | yes (phase_diffusion cohort_stats) | yes (last_cohort_stats_reflects_population) | FULL | 0 |
| FR-CIV-ENGINE-INT-015 | yes (phase_diffusion + should_tick_entity_with_policy) | yes (cold_tier_diffusion_only_on_cadence_boundaries) | FULL | 0 |
| FR-CIV-ENGINE-REPLAY-001 | yes (ReplayLog save/load) | yes (replay_log_round_trips_through_save_load) | FULL | 0 |
| FR-CIV-ENGINE-REPLAY-002 | yes (tick writes ReplayEvent::Tick) | yes (simulation_tick_produces_replay_tick_event) | FULL | 0 |
| FR-CIV-ENGINE-REPLAY-003 | yes (`push_damage` records event) | yes (push_damage_records_damage_event) | FULL | 0 |
| FR-CIV-ENGINE-REPLAY-004 | yes (load_replay_from_file) | yes (replay_reproduces_final_voxel_chunk_count_and_tick) | FULL | 0 |
| FR-CIV-ENGINE-REPLAY-005 | yes (load_replay_from_file convergence) | yes (replay_logs_converge_to_identical_voxel_state) | FULL | 0 |
| **FR-CIV-LANG-001** | **yes (`language_trade_factor` at line 3933, `faction_language_centroids` at line 4062)** | **yes (`language_trade_factor_scales_with_distance` at 5828, `faction_language_centroids_member_weighted` at 5842, `language_barrier_reduces_trade_route_flow` at 5875)** | **FULL** | **+1 (N5)** |
| **FR-CIV-LEGENDS-001** | **yes (`apply_saga_belief_gain` at line 2061, `saga_belief_gain` at line 4435)** | **yes (`saga_belief_gain_bounded_by_cap` at 6894, `saga_belief_gain_scales_with_promotions` at 6905, `saga_promotions_increase_belief_over_ticks` at 6919)** | **FULL** | **+1 (N6)** |
| FR-CIV-LIFE-001 | yes (phase_life needs_tick pipeline) | yes (phase_life_attaches_needs_and_exposes_settlements) | FULL | 0 |
| FR-CIV-LIFE-003 | yes (phase_life death via needs) | yes (phase_life_attaches_needs_and_exposes_settlements, last_life_deaths) | FULL | 0 |
| FR-CIV-LIFE-010 | yes (phase_life utility-AI daily path) | yes (phase_life_attaches_needs_and_exposes_settlements) | FULL | 0 |
| FR-CIV-LIFE-020 | yes (phase_life cluster stocks, phase_settlement_consumption) | yes (cluster_stocks_food_stays_bounded_over_populated_cluster_ticks) | FULL | 0 |
| FR-CIV-LIFE-030 | yes (phase_life cluster_by_colocation) | yes (phase_life_attaches_needs_and_exposes_settlements, phase_life_clustering_is_deterministic, phase_life_clustering_skip_matches_full_recompute_on_movement, phase_life_clustering_skipped_when_population_stationary) | FULL | 0 |
| FR-CIV-PLANET-010 | yes (`phase_planet`, compute_climate) | yes (climate_recomputes_every_tick, engine_tick_includes_climate_in_snapshot) | FULL | 0 |
| FR-CIV-PLANET-020 | yes (`apply_tide_offset`, `register_coastal_water_column`, `coastal_columns`) | yes (tide_offset_shifts_coastal_voxel_height) | FULL | 0 |
| FR-CIV-PLANET-030 | yes (phase_planet calls compute_weather; `weather_grid()`) | yes (weather_grid_temperature_varies_with_year_phase) | FULL | 0 |
| FR-CIV-PLANET-040 | yes (SimulationSnapshot.geology_map, GeologyMap::seed) | yes (weather_grid_temperature_varies_with_year_phase — Covers) | FULL | 0 |
| FR-CIV-PLANET-060 | yes (replay_log hash chain integration) | yes (combat_events_extend_replay_hash_chain — Covers) | FULL | 0 |
| **FR-CIV-PSYCHE-912** | **yes (`language_trade_factor` at line 3933, `faction_language_centroids` at line 4062 — paired with FR-CIV-LANG-001)** | **yes (same 3 tests as LANG-001: `language_trade_factor_scales_with_distance`, `faction_language_centroids_member_weighted`, `language_barrier_reduces_trade_route_flow`)** | **FULL** | **+1 (N5)** |
| FR-CIV-TACTICS-010 | yes (phase_tactics, `default_faction_doctrines`, `evolve_doctrine` block) | yes (phase_tactics_evolve_doctrine_on_cadence) | FULL | 0 |
| FR-CIV-TACTICS-024 | yes (`last_tick_combat_pulses`, CombatDamagePulse, phase_military/phase_tactics writes) | yes (fr_civ_tactics_024_snapshot_damage_events_reflect_last_tick_pulses) | FULL | 0 |
| FR-CIV-TACTICS-025 | yes (phase_military combat recording, replay_log.record_combat) | yes (replay_combat_events_restore_pending_damage, replay_round_trip_preserves_combat_events, war_bridge_records_combat_replay_events) | FULL | 0 |
| FR-CIV-TACTICS-032 | yes (MilitaryUnit.hp, phase_military hp_loss block) | yes (war_bridge_records_combat_replay_events — Covers) | FULL | 0 |
| FR-CIV-TACTICS-035 | yes (`military_phase: MilitaryPhaseConfig`, phase_military cadence) | yes (war_bridge_records_combat_replay_events — Covers) | FULL | 0 |
| FR-CIV-TACTICS-041 | yes (replay_log.record_combat hash chain) | yes (combat_events_extend_replay_hash_chain — Covers) | FULL | 0 |
| FR-CIV-TACTICS-045 | yes (`configure_military_fog`) | yes (configure_military_fog_sets_radius_and_clamps_grid) | FULL | 0 |
| FR-CIV-TACTICS-050 | yes (`apply_scenario_military`) | yes (apply_scenario_military_wires_overrides_and_clamps_range) | FULL | 0 |
| FR-CIV-VOXEL-002 | yes (`phase_voxel`, `drain_dirty`) | yes (tide_offset_shifts_coastal_voxel_height — Covers) | FULL | 0 |
| FR-CIV-VOXEL-006 | yes (phase_voxel, last_tick_voxel_events) | yes (voxel_phase_drains_dirty_events_each_tick) | FULL | 0 |
| FR-CIV-VOXEL-007 | yes (phase_voxel, replay-safe writes) | yes (voxel_phase_replay_is_bit_identical) | FULL | 0 |
| FR-CIV-WAR-020 | yes (replay_log combat events shared with sim state) | yes (war_bridge_records_combat_replay_events) | FULL | 0 |
| FR-CORE-001 | yes (tick → replay_log.record_tick) | yes (fr_core_001_single_tick_event_per_tick) | FULL | 0 |
| FR-ECON-001 | yes (phase_economy `effective_consumption`) | yes (phase_economy_conserves_non_negative_budget, phase_economy_uses_capitalist_allocator, phase_economy_updates_economy_state, phase_economy_steps_market_prices) | FULL | 0 |
| FR-REPLAY-001 | yes (`save_replay`, `load_replay_from_file`) | yes (civreplay_save_load_restores_tick_after_ticks — Covers) | FULL | 0 |

## FAMILY COUPLING FULL TABLE (bare family refs with impl + test, 2 total)

| FR-ID | impl? | test? | bucket | origin |
|-------|-------|-------|--------|--------|
| FR-CIV-ECON (family) | yes (`commodity_unrest_delta` at line 3383, wired at 2102) | yes (3 tests: `commodity_unrest_skips_food` at 5014, `commodity_unrest_caps_rise` at 5031, `commodity_unrest_decay_when_cheap` at 5042) | Family coupling FULL | N8 — non-food commodity unrest delta |
| FR-CIV-GENETICS (family) | yes (`awakening_belief_gain` at line 4453, `awakening_cohesion_gain` at line 3847) | yes (3 tests: `awakening_belief_gain_bounded` at 6957, `awakening_increases_belief` at 6982, `awakening_cohesion_pulse_bounded` at 7030) | Family coupling FULL | N7 — sentience awakening → belief + cohesion coupling |

FR-CIV-GENETICS (family) pairs with FR-CIV-LEGENDS in all 7 comment references (lines 3833, 3843, 4441, 4449, 6953, 6977, 7026) and bears the "N7" label in the test block. The numbered GENETICS-001..006 sub-IDs remain unwired in engine.rs (handled in `crates/genetics/`).

FR-CIV-ECON (family) is the N8 commodity-unrest coupling. `FR-ECON-001` is already a separate numbered FULL entry; `FR-CIV-ECON` is the broader family umbrella referenced in the new coupling.

## IMPL-NO-TEST

| FR-ID | impl? | test? | bucket | note |
|-------|-------|-------|--------|------|
| FR-MOD-004 partial | yes (`mod_loaded_bus_events`, `last_tick_mod_lifecycle`, `install_mod_path` writes bus JSON) | NO in engine.rs (covered by `crates/mod-host` tests) | IMPL-NO-TEST | unchanged from SNAP2 |

## SPEC-ONLY (in engine.rs comments — MOAT family wildcards, 4 total)

| FR-ID | impl? | test? | note |
|-------|-------|-------|------|
| FR-CIV-LEGENDS-* | NO (MOAT comment at line 70; numbered LEGENDS-001 now wired as FULL, but 002–008 remain unwired) | NO | MOAT placeholder for 002–008; 001 promoted to FULL via N6 |
| FR-CIV-PSYCHE-* | NO (MOOT comment at line 70; PSYCHE-912 now wired as FULL for language-coupling, but 001–008 remain unwired) | NO | MOAT placeholder for 001–008; 912 is a coupling ID, not core psyche |
| FR-CIV-GENETICS-* | NO (MOAT comment at line 70; bare family FR-CIV-GENETICS is now wired for coupling code, but numbered 001–006 remain unwired in engine.rs) | NO | MOAT placeholder for numbered genetics sub-IDs |
| FR-CIV-AI-* | NO (only MOAT comment at line 70; civ_ai_decisions accessor in emergence.rs but no numbered ID wired in engine.rs) | NO | MOAT placeholder |

## DELTA SUMMARY vs SNAPSHOT 2 (45/1/4 baseline)

| Metric | Snapshot 2 | Snapshot 3 | Delta |
|--------|------------|------------|-------|
| FULL (numbered IDs) | 45 | 48 | **+3** |
| Family coupling FULL | — | 2 | **+2** (new bucket) |
| IMPL-NO-TEST | 1 | 1 | 0 |
| SPEC-ONLY | 4 | 4 | 0 |
| **Total** | **50** | **55** | **+5** |

**Numbered FULL additions:**
1. **FR-CIV-LANG-001** (N5) — language→trade friction coupling: `language_trade_factor`, `faction_language_centroids`, 3 tests
2. **FR-CIV-PSYCHE-912** (N5, paired) — same impl+tests as LANG-001; language barrier on psyche → economic cost
3. **FR-CIV-LEGENDS-001** (N6) — saga significance→belief coupling: `apply_saga_belief_gain`, `saga_belief_gain`, 3 tests

**Family coupling FULL additions:**
4. **FR-CIV-GENETICS** (N7) — sentience awakening→belief+cohesion: `awakening_belief_gain`, `awakening_cohesion_gain`, 3 tests
5. **FR-CIV-ECON** (N8) — non-food commodity→unrest: `commodity_unrest_delta`, 3 tests

## SPEC-ONLY REMAINING (4 wildcards)

| Wildcard | Remaining unwired numbered sub-IDs | Next effort |
|----------|-----------------------------------|-------------|
| FR-CIV-LEGENDS-* | 002 (rumor retelling), 005 (saga-query API), 006 (missing producer gap warn), 007 (cultural register), 003 (mutation hop), 004, 008 | N6 deferred |
| FR-CIV-PSYCHE-* | 001–008 (core psyche: OCEAN moods, Maslow needs, relationship persistence) | serde-psyche save/load (FR-CIV-PSYCHE-005) |
| FR-CIV-GENETICS-* | 001–006 (mutation determinism, DNA cognition scoring, sentience thresholds, lineage tracing) | handled in `crates/genetics/` |
| FR-CIV-AI-* | 001–008 (civ-ai decision surface, flavor actions, MOAT wiring) | handled in `crates/agents/src/` |

## NOTES

- N5 (LANG) and N8 (ECON) delivered their full coupling in a single batch with 3 tests each — closed as FULL
- N6 (LEGENDS→belief) closed LEGENDS-001 as FULL with 3 engine.rs tests; the saga-side helpers (`top_significance` on `SagaGraph`, `last_tick_promotions` on `LegendsWorker`) live in `crates/legends/src/`
- N7 (GENETICS→awakening coupling) closed bare family FR-CIV-GENETICS with 3 engine.rs tests; the main `apply_awakening_coupling` entry point lives in `crates/engine/src/emergence.rs:504`
- Engine.rs is **8318 lines** (+779 vs snapshot 2's 7639). 197 FR-* comment/doc occurrences across the file
- The 4 MOAT wildcards (LEGENDS-*, PSYCHE-*, GENETICS-*, AI-*) remain at line 70 — unchanged. The PSYCHE and GENETICS wildcards now have **one** numbered child each promoted to FULL (912 and bare family), but the core numbered sub-IDs (001–008 and 001–006 respectively) remain unwired in engine.rs
- FR-CIV-LIFE-* at line 2366 is a doc umbrella over numbered LIFE-001/003/010/020/030 (all FULL); not counted as SPEC-ONLY

## AUDIT METHOD (read-only)

1. `crates/engine/src/engine.rs` (8318 lines) scanned via `fs_search` + PowerShell unique extraction for all FR-ID patterns
2. Each FR-ID line number verified against surrounding code context (impl fn vs test fn vs doc-only)
3. Snapshot 2 FULL/IMPL-NO-TEST/SPEC-ONLY entries re-verified against current file content
4. N5/N6/N7/N8 spec docs (`N5_LANGUAGE_SPEC.md`, `N6_LEGEND_BELIEF_SPEC.md`, `docs/design/species-sentience.md` for N7, `N8_DESIGN.md`) cross-referenced for new FR-IDs
5. Did NOT execute cargo build / cargo test (per "NO cargo" constraint)
6. Did NOT edit any source files (per "NO edits" constraint)
7. Did NOT commit (per "NO commit" constraint)
