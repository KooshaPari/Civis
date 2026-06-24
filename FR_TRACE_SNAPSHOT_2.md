# FR TRACEABILITY SNAPSHOT 2 — Civis Engine Crate

Generated: 2026-06-16
Scope: `crates/engine/src/engine.rs` only
Method: read-only grep/classify; **NO cargo, NO git commit, NO source edits** (per task constraint)
Spec sources cross-referenced: `FUNCTIONAL_REQUIREMENTS.md`, `docs/traceability/**`, `agileplus-specs/**`
Comment patterns: `/// FR-...`, `// FR-...`, `/// Covers FR-...`
Impl patterns: `fn phase_*(&mut self)`, supporting impl methods (`apply_tide_offset`, `try_invoke_divine_power`, `push_damage`, `push_voxel_write`, `save_replay`, `load_replay_from_file`, `register_coastal_water_column`, `configure_military_fog`, `apply_scenario_military`, `install_mod_path`, `mod_loaded_bus_events`, `rebuild_agent_id_index`, `agent_entity`, `install_mod_path`, etc.)
Test patterns: `#[test]` functions inside `mod tests` (line 4308+), with `/// Covers FR-...` references

## CLASSIFICATION LEGEND

- **FULL** — FR-ID referenced in engine.rs comment AND has matching `phase_*` (or supporting) impl AND has a `#[test]` (or `/// Covers`) reference in the same file
- **IMPL-NO-TEST** — FR-ID referenced in engine.rs comment AND has impl, but NO test reference in this file
- **SPEC-ONLY** — FR-ID (or family wildcard) referenced in engine.rs comment but NO impl, NO test in this file

## TOTALS BY BUCKET (engine.rs scope)

| Bucket | Count |
|--------|-------|
| FULL | 45 |
| IMPL-NO-TEST | 1 |
| SPEC-ONLY | 4 |
| **Total distinct FR-IDs in engine.rs** | **50** |

## FULL TABLE

| FR-ID | impl? | test? | bucket |
|-------|-------|-------|--------|
| FR-CIV-0100 | yes (many `phase_*` fns: production, citizen_lifecycle, military, economy, planet, diplomacy, tactics, voxel, buildings, diffusion, research, tech, belief, unrest, faction_unrest, cohesion, social_mood, stratification, institutions, economic_focus, chronicle) | yes (24+ tests cover §3 downward/upward causation) | FULL |
| FR-CIV-0200 | yes (`phase_research`, `research_progress`, `research_tier`) | yes (phase_research_accrues_from_population, phase_research_quiescent_without_population, research_tier_divides_progress, research_tier_and_capacity_grow_with_progress) | FULL |
| FR-CIV-EMERGENCE | yes (`phase_belief`, `try_invoke_divine_power`, `add_belief`) | yes (try_invoke_divine_power_spends_belief, try_invoke_divine_power_gates_on_belief, phase_belief_accrues_from_population, belief_decays_toward_equilibrium) | FULL |
| FR-CIV-TACTICS-010 | yes (phase_tactics, `default_faction_doctrines`, `evolve_doctrine` block) | yes (phase_tactics_evolve_doctrine_on_cadence) | FULL |
| FR-CIV-TACTICS-024 | yes (`last_tick_combat_pulses`, CombatDamagePulse, phase_military/phase_tactics writes) | yes (fr_civ_tactics_024_snapshot_damage_events_reflect_last_tick_pulses) | FULL |
| FR-CIV-TACTICS-025 | yes (phase_military combat recording, replay_log.record_combat) | yes (replay_combat_events_restore_pending_damage, replay_round_trip_preserves_combat_events, war_bridge_records_combat_replay_events) | FULL |
| FR-CIV-TACTICS-025- | yes (apply_replay_combat) | yes (replay_combat_drains_to_same_voxel_state_as_live, replay_combat_log_deterministic_for_seed_rerun) | FULL |
| FR-CIV-TACTICS-032 | yes (MilitaryUnit.hp, phase_military hp_loss block) | yes (war_bridge_records_combat_replay_events — Covers) | FULL |
| FR-CIV-TACTICS-035 | yes (`military_phase: MilitaryPhaseConfig`, phase_military cadence) | yes (war_bridge_records_combat_replay_events — Covers) | FULL |
| FR-CIV-TACTICS-041 | yes (replay_log.record_combat hash chain) | yes (combat_events_extend_replay_hash_chain — Covers) | FULL |
| FR-CIV-TACTICS-045 | yes (`configure_military_fog`) | yes (configure_military_fog_sets_radius_and_clamps_grid) | FULL |
| FR-CIV-TACTICS-050 | yes (`apply_scenario_military`) | yes (apply_scenario_military_wires_overrides_and_clamps_range) | FULL |
| FR-CIV-PLANET-010 | yes (`phase_planet`, compute_climate) | yes (climate_recomputes_every_tick, engine_tick_includes_climate_in_snapshot) | FULL |
| FR-CIV-PLANET-020 | yes (`apply_tide_offset`, `register_coastal_water_column`, `coastal_columns`) | yes (tide_offset_shifts_coastal_voxel_height) | FULL |
| FR-CIV-PLANET-030 | yes (phase_planet calls compute_weather; `weather_grid()`) | yes (weather_grid_temperature_varies_with_year_phase) | FULL |
| FR-CIV-PLANET-040 | yes (SimulationSnapshot.geology_map, GeologyMap::seed) | yes (weather_grid_temperature_varies_with_year_phase — Covers) | FULL |
| FR-CIV-PLANET-060 | yes (replay_log hash chain integration) | yes (combat_events_extend_replay_hash_chain — Covers) | FULL |
| FR-CIV-VOXEL-002 | yes (`phase_voxel`, `drain_dirty`) | yes (tide_offset_shifts_coastal_voxel_height — Covers) | FULL |
| FR-CIV-VOXEL-006 | yes (phase_voxel, last_tick_voxel_events) | yes (voxel_phase_drains_dirty_events_each_tick) | FULL |
| FR-CIV-VOXEL-007 | yes (phase_voxel, replay-safe writes) | yes (voxel_phase_replay_is_bit_identical) | FULL |
| FR-CIV-LIFE-001 | yes (phase_life needs_tick pipeline) | yes (phase_life_attaches_needs_and_exposes_settlements — Covers) | FULL |
| FR-CIV-LIFE-003 | yes (phase_life death via needs) | yes (phase_life_attaches_needs_and_exposes_settlements — Covers, last_life_deaths) | FULL |
| FR-CIV-LIFE-010 | yes (phase_life utility-AI daily path) | yes (phase_life_attaches_needs_and_exposes_settlements — Covers) | FULL |
| FR-CIV-LIFE-020 | yes (phase_life cluster stocks, phase_settlement_consumption) | yes (cluster_stocks_food_stays_bounded_over_populated_cluster_ticks) | FULL |
| FR-CIV-LIFE-030 | yes (phase_life cluster_by_colocation) | yes (phase_life_attaches_needs_and_exposes_settlements, phase_life_clustering_is_deterministic, phase_life_clustering_skip_matches_full_recompute_on_movement, phase_life_clustering_skipped_when_population_stationary) | FULL |
| FR-CIV-CA-009 | yes (`phase_voxel_ca`, `last_tick_abiogenesis_sites`) | yes (phase_voxel_ca_none_is_noop, phase_voxel_ca_warm_water_is_viable_stone_is_not) | FULL |
| FR-CIV-WAR-020 | yes (replay_log combat events shared with sim state) | yes (war_bridge_records_combat_replay_events) | FULL |
| FR-CIV-ENGINE-INT-001 | yes (phase_planet) | yes (climate_recomputes_every_tick — Covers) | FULL |
| FR-CIV-ENGINE-INT-002 | yes (phase_tactics drains pending_damage) | yes (pending_damage_drains_and_reduces_chunk_count) | FULL |
| FR-CIV-ENGINE-INT-003 | yes (phase_compact modulo 64) | yes (compact_runs_every_64_ticks) | FULL |
| FR-CIV-ENGINE-INT-005 | yes (phase_planet uses is_daytime) | yes (daytime_cycles_across_one_full_day) | FULL |
| FR-CIV-ENGINE-INT-010 | yes (`spawn_faction_civilians` + `spawn_initial_entities`) | yes (startup_spawns_128_civilians) | FULL |
| FR-CIV-ENGINE-INT-011 | yes (phase_buildings, building_graph) | yes (phase_buildings_allocates_over_time_when_signals_are_high) | FULL |
| FR-CIV-ENGINE-INT-012 | yes (phase_diffusion, propagate_wardrobe) | yes (phase_diffusion_bumps_wardrobe_eras) | FULL |
| FR-CIV-ENGINE-INT-013 | yes (tick determinism via seeded RNG) | yes (determinism_holds_with_all_phases_enabled) | FULL |
| FR-CIV-ENGINE-INT-014 | yes (phase_diffusion cohort_stats) | yes (last_cohort_stats_reflects_population) | FULL |
| FR-CIV-ENGINE-INT-015 | yes (phase_diffusion + should_tick_entity_with_policy) | yes (cold_tier_diffusion_only_on_cadence_boundaries) | FULL |
| FR-CIV-ENGINE-REPLAY-001 | yes (ReplayLog save/load) | yes (replay_log_round_trips_through_save_load) | FULL |
| FR-CIV-ENGINE-REPLAY-002 | yes (tick writes ReplayEvent::Tick) | yes (simulation_tick_produces_replay_tick_event) | FULL |
| FR-CIV-ENGINE-REPLAY-003 | yes (`push_damage` records event) | yes (push_damage_records_damage_event) | FULL |
| FR-CIV-ENGINE-REPLAY-004 | yes (load_replay_from_file) | yes (replay_reproduces_final_voxel_chunk_count_and_tick) | FULL |
| FR-CIV-ENGINE-REPLAY-005 | yes (load_replay_from_file convergence) | yes (replay_logs_converge_to_identical_voxel_state) | FULL |
| FR-CORE-001 | yes (tick → replay_log.record_tick) | yes (fr_core_001_single_tick_event_per_tick) | FULL |
| FR-ECON-001 | yes (phase_economy `effective_consumption`) | yes (phase_economy_conserves_non_negative_budget, phase_economy_uses_capitalist_allocator, phase_economy_updates_economy_state, phase_economy_steps_market_prices) | FULL |
| FR-REPLAY-001 | yes (`save_replay`, `load_replay_from_file`) | yes (civreplay_save_load_restores_tick_after_ticks — Covers) | FULL |

## IMPL-NO-TEST

| FR-ID | impl? | test? | bucket | note |
|-------|-------|-------|--------|------|
| FR-MOD-004 partial | yes (`mod_loaded_bus_events`, `last_tick_mod_lifecycle`, `install_mod_path` writes bus JSON) | NO in engine.rs (covered by `crates/mod-host` tests outside this scope) | IMPL-NO-TEST | engine.rs docstring claims "partial"; mod-host crate owns the lifecycle tests |

## SPEC-ONLY (in engine.rs comments — wildcards/MOAT placeholders)

| FR-ID | impl? | test? | bucket | note |
|-------|-------|-------|--------|------|
| FR-CIV-LEGENDS-* | NO (only MOAT comment at line 70; legends_graph accessor exists in `emergence.rs` but no numbered ID wired in engine.rs) | NO | SPEC-ONLY | MOAT placeholder; SagaGraph scaffold is in `crates/agents/src/legends.rs` (per FUNCTIONAL_REQUIREMENTS.md spec FR-CIV-LEGENDS-001..008) |
| FR-CIV-PSYCHE-* | NO (only MOAT comment at line 70; Psyche component used via `agent_psyche()` accessor in `emergence.rs` but no numbered ID wired in engine.rs) | NO | SPEC-ONLY | MOAT placeholder; Psyche is wired through ECS components (per FUNCTIONAL_REQUIREMENTS.md FR-CIV-PSYCHE-001..008) |
| FR-CIV-GENETICS-* | NO (only MOAT comment at line 70; DNA/sentience scaffolding via `sentient_agents`/`sentience_events` in `emergence.rs` but no numbered ID wired in engine.rs) | NO | SPEC-ONLY | MOAT placeholder; FR-CIV-GENETICS-001 referenced as dependency in `agileplus-specs/civ-019-emergence-metrics-dashboard/spec.md` but full FR family not in FUNCTIONAL_REQUIREMENTS.md |
| FR-CIV-AI-* | NO (only MOAT comment at line 70; `civ_ai_decisions()` accessor in `emergence.rs` but no numbered ID wired in engine.rs) | NO | SPEC-ONLY | MOAT placeholder; AI decision surface not yet numbered in FUNCTIONAL_REQUIREMENTS.md |

## TOP 8 SPEC-ONLY FRs CLOSEST TO IMPLEMENTABLE (ranked by existing emergence scaffolding leverage)

Existing scaffolding (in `crates/engine/src/emergence.rs`, `crates/agents/src/`, `crates/genetics/src/`, `crates/diplomacy/src/`):
- `EmergenceState::legends_graph()` → SagaGraph with culture/cluster metadata
- `EmergenceState::cluster_cultures()` → `BTreeMap<u64, CultureProfile>` (N2 culture)
- `EmergenceState::civ_ai_decisions()` → `Vec<CivAiDecision>` (FR-CIV-AI surface)
- `EmergenceState::sentience_events()`, `sentient_agents` → DNA cognition (FR-CIV-GENETICS surface)
- `EmergenceState::agent_psyche(agent_id)` → Psyche component (OCEAN, PAD mood, Maslow needs)
- `EmergenceState::agent_social_graph(agent_id)` → SocialGraph (affinity, trust, familiarity)
- `EmergenceState::seed_library()` → SeedSet (canon/primitive mode seeds)
- `EmergenceState::emergence_feed()` → `Vec<EmergenceFeedEvent>` on the watch bus
- `Simulation::phase_emergence()` runs every tick (line 1668)
- `phase_diplomacy` already reads `faction_relations`; can hook into Psyche relationship floats

| Rank | FR-ID | Closest scaffolding | Why close | Estimated effort |
|------|-------|---------------------|-----------|------------------|
| 1 | **FR-CIV-LEGENDS-001** (Sim SHALL emit structured `HistoricalEvent` records on the watch bus; legends layer SHALL NOT author outcomes) | `emergence_feed()` + watch bus + `emergence.rs:39` `EmergenceFeedEvent` | `EmergenceFeedEvent` is the seed struct; just need to give it a `HistoricalEvent` shape and ensure it serializes on `sim.emergence` bus. | S — rename/wrap struct + 1 test |
| 2 | **FR-CIV-LEGENDS-006** (Missing producer events SHALL log `legends: gap` warnings; show empty saga with reason, never silent omission) | `legends_graph()` (SagaGraph) | SagaGraph already exposed; just add a `verify_legend_provenance()` pass on the emergence sample and wire `tracing::warn!`. | S — single new pass + 1 test |
| 3 | **FR-CIV-PSYCHE-005** (Psyche state SHALL persist in snapshots; reload restores moods/relationships without re-rolling traits) | `agent_psyche()` accessor + serde derive on `Psyche` | Psyche struct exists in `crates/agents`; needs `#[derive(Serialize, Deserialize)]` and inclusion in `save_bundle` (CIV-1000). | S — serde + save/load test |
| 4 | **FR-CIV-LEGENDS-002** (Historian agents SHALL re-emit `Rumor`/`Chronicle` from witnessed event subsets only) | `legends_graph()` + `agent_psyche()` | Re-emit transformer is a pure function `(EventSet, &SagaGraph) -> Vec<Rumor>`; needs a new `Rumor` type in `crates/legends`. | M — new type + transformer fn + 1 test |
| 5 | **FR-CIV-PSYCHE-006** (Diplomacy reservation utility SHALL read relationship floats from this component; ties to FR-CIV-DIPLO-003) | `phase_diplomacy` already reads `faction_relation`; Psyche has `pairwise (affinity, trust, familiarity)` | Add a `diplomacy_reservation_utility(psyche_pair) -> f32` helper; call from `phase_diplomacy` threshold computation. | S — helper + 1 test |
| 6 | **FR-CIV-LEGENDS-005** (Saga-graph ingest SHALL stay compatible with `docs/design/legends-engine.md` query API) | `legends_graph()` (SagaGraph) | SagaGraph exists; just expose a `query_saga(filter)` matching the legends-engine.md API surface. | S — query API surface + 1 test |
| 7 | **FR-CIV-LEGENDS-007** (Cultural register output SHALL feed literature/historian UI; formal register SHALL remain separate from treaty text) | `cluster_cultures()` (CultureProfile) | CultureProfile already has traits vec; needs a `register_formality(register: RegisterKind)` enum and UI feed payload. | M — enum + register classifier + 1 test |
| 8 | **FR-CIV-LEGENDS-003** (Each retelling hop SHALL mutate rumors (actor swap, amplification, teller psyche/culture tags) with gates from OCEAN traits) | `legends_graph()` + `agent_psyche()` OCEAN | Mutation function is pure: `(Rumor, OCEAN, CultureProfile) -> Rumor`; needs OCEAN-gated amplification/actor-swap logic. | M — mutation fn + OCEAN gate test |

## NOTES

- Engine.rs is **dense in traceability** for the FR-CIV-0100 emergence web (24+ test references) and the engine internals (FR-CIV-ENGINE-INT-001..015, FR-CIV-ENGINE-REPLAY-001..005).
- The MOAT wildcards (LEGENDS/PSYCHE/GENETICS/AI) are deliberately **family-level placeholders** — no specific numbered ID is wired into engine.rs. Per AGENTS.md and FUNCTIONAL_REQUIREMENTS.md, those FR families live in other crates (`crates/legends`, `crates/agents`, `crates/genetics`, `crates/ai`), not in `crates/engine`.
- The single IMPL-NO-TEST entry (FR-MOD-004 partial) reflects that mod-host lifecycle is owned by the dedicated `crates/mod-host` crate; engine.rs is the integration point, not the test surface.
- No cargo build / cargo test was executed (per task constraint). All counts derive from grep on the file, comments, and `mod tests` block (line 4308+).

## AUDIT METHOD (read-only)

1. `crates/engine/src/engine.rs` read in 4 chunks (lines 1-2000, 2000-4000, 4000-6550, 6550-7639)
2. `fs_search` regex `FR-[A-Z]+-[A-Z0-9-]+` → 178 hits; truncated results read in full via follow-up reads
3. `fs_search` regex `fn phase_\w+` → 28 `phase_*` functions identified
4. `fs_search` regex `#\[test\]|fn [a-z_]+\(\)` → 96 `#[test]` functions in the `mod tests` block
5. Cross-referenced `Covers FR-...` and `/// FR-...` comments to impl/test functions in the same file
6. Cross-referenced `crates/mod-host` ownership of FR-MOD-004 partial
7. Did NOT execute cargo build / cargo test (per "NO cargo" constraint)
8. Did NOT edit any source files (per "NO edits" constraint)
9. Did NOT commit (per "NO commit" constraint)
