# Civis TraceLinks

Requirement-to-code-to-test-to-commit links for the active FR-CIV families in this workspace.

Format:

`FR ID` | `Code` | `Test` | `Commit`

## LIFE

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-LIFE-000 | `crates/needs/src/lib.rs` | `schema_version_is_semver` | `8cd14203` |
| FR-CIV-LIFE-001 | `crates/needs/src/lib.rs` | `needs_decay_toward_zero`, `decay_clamps_at_zero`, `decay_preserves_need_bounds`, `most_pressing_picks_lowest`, `satisfy_raises_and_clamps` | `8cd14203`, `98273179` |
| FR-CIV-LIFE-002 | `crates/needs/src/lib.rs` | `sickness_onset_respects_threshold`, `sickness_recovery_requires_high_integrity`, `deprivation_triggers_sickness`, `pipeline_is_deterministic` | `8cd14203`, `98273179` |
| FR-CIV-LIFE-003 | `crates/needs/src/lib.rs` | `unmet_needs_cause_death`, `critical_boundary_triggers_damage`, `sated_agent_survives`, `dead_agent_is_a_noop`, `health_stays_bounded_and_dead_is_terminal` | `8cd14203`, `98273179` |
| FR-CIV-LIFE-010..016 | `crates/agents/src/daily_path.rs` | `nearest_of_kind_returns_the_closest_matching_poi`, `pick_target_chooses_the_highest_pressure_need`, `path_step_moves_toward_the_target_and_never_overshoots`, `scoring_is_deterministic_for_the_same_inputs`, `empty_registry_yields_no_target`, `satisfied_needs_prefer_idle_wander_over_seek`, `wander_anchors_remain_local_and_deterministic` | `f7e212cf`, `2babe84a` |
| FR-CIV-LIFE-020..025 | `crates/economy/src/stocks.rs` | `arbitrary_stock_updates_never_make_inventories_negative`, `surplus_and_deficit_signs_reflect_net_flow_against_stock`, `comparative_advantage_must_select_a_maximum_net_flow_good`, `trade_gain_is_positive_when_advantages_differ_and_zero_when_identical`, `valid_trades_conserve_the_combined_stock_total`, `trade_proposals_are_rejected_when_there_is_no_mutual_benefit` | `98273179` |
| FR-CIV-LIFE-030 | `crates/engine/src/engine.rs` | `phase_life_attaches_needs_and_exposes_settlements` | `d9699bf9` |

## INFRA

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-INFRA-001 | `crates/civ-traffic/src/lib.rs` | `graph_round_trips_through_ron` | `d9699bf9` |
| FR-CIV-INFRA-010 | `crates/civ-traffic/src/lib.rs` | `traffic_promotes_desire_path_ladder` | `1c570f1c` |
| FR-CIV-INFRA-011 | `crates/civ-traffic/src/lib.rs` | `edges_are_undirected` | `1c570f1c` |
| FR-CIV-INFRA-020 | `crates/civ-traffic/src/lib.rs` | `roads_speed_up_pathing` | `d9699bf9` |
| FR-CIV-INFRA-021 | `crates/civ-traffic/src/lib.rs` | `user_and_emergent_share_one_graph` | `d9699bf9` |
| FR-CIV-INFRA-022 | `crates/civ-traffic/src/lib.rs` | `user_road_upgrades_under_heavy_use_never_downgrades` | `d9699bf9` |
| FR-CIV-INFRA-030 | `crates/civ-traffic/src/lib.rs` | `emergent_growth_is_deterministic` | `1c570f1c` |
| FR-CIV-INFRA-040 | `crates/civ-traffic/src/lib.rs` | `vehicles_gate_on_tech_era` | `d9699bf9` |
| FR-CIV-INFRA-050 | `crates/civ-traffic/src/lib.rs` | `place_path_lays_connected_segments` | `d9699bf9` |
| FR-CIV-INFRA-060 | `crates/civ-traffic/src/lib.rs` | `bridges_place_and_price_like_roads` | `d9699bf9` |

## TRAFFIC

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-TRAFFIC-LANE-001 | `crates/civ-traffic/src/lane.rs` | `lanes_follow_road_kind_ladder` | `1c570f1c` |
| FR-CIV-TRAFFIC-LANE-002 | `crates/civ-traffic/src/lane.rs` | `route_lanes_crosses_shared_node` | `1c570f1c` |
| FR-CIV-TRAFFIC-LANE-003 | `crates/civ-traffic/src/lane.rs` | `lane_speed_matches_road_speed` | `1c570f1c` |
| FR-CIV-TRAFFIC-LANE-004 | `crates/civ-traffic/src/lane.rs` | `scalar_speed_graph_stays_intact` | `1c570f1c` |

## INSPECT

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-INSPECT-900 | `clients/bevy-ref/src/inspect.rs` | `cell_readout_reflects_terrain`, `submerged_flag_tracks_water_level`, `inspect_kind_labels` | `ac4642ff` |
| FR-CIV-INSPECT-910 | `clients/bevy-ref/src/inspect.rs` | `tooltip_is_informative`, `temperature_bands_span_range` | `ac4642ff` |

## INFOVIEW

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-INFOVIEW-900 | `clients/bevy-ref/src/info_views.rs` | `registry_ships_high_value_overlays`, `cycle_wraps_through_off`, `activate_by_id_selects_overlay` | `ac4642ff` |
| FR-CIV-INFOVIEW-901 | `clients/bevy-ref/src/info_views.rs` | `ramp_color_interpolates_and_clamps` | `ac4642ff` |
| FR-CIV-INFOVIEW-910 | `clients/bevy-ref/src/info_views.rs` | `elevation_overlay_always_returns_color`, `population_overlay_skips_empty_cells`, `water_overlay_flags_submerged`, `cluster_colors_distinct_and_stable`, `terrain_sample_populates_height` | `ac4642ff` |

## SAVE

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-SAVE-001 | `crates/engine/src/engine.rs`, `crates/server/src/jsonrpc.rs` | `replay_log_round_trips_through_save_load`, `civreplay_save_load_restores_tick_after_ticks`, `dispatch_sim_save_replay_plans_save_effect` | `808eb4c9`, `30ab1a8f` |
| FR-CIV-SAVE-002 | `crates/server/src/jsonrpc.rs` | `dispatch_save_slot_plans_save_effect`, `dispatch_save_slot_rejects_invalid_slot`, `parse_sim_save_replay_request` | `30ab1a8f` |
| FR-CIV-SAVE-003 | `crates/server/src/jsonrpc.rs` | `dispatch_sim_snapshot_includes_snapshot_fields`, `dispatch_sim_status_omits_population_without_snapshot` | `30ab1a8f`, `17cd7fa6` |
| FR-CIV-SAVE-004 | `crates/watch/src/saves_api.rs`, `crates/watch/src/api_tests.rs` | `post_control_save_and_load_round_trip`, `post_save_slot_round_trip`, `autosave_ring_evicts_oldest_beyond_max`, `get_events_streams_snapshot_within_timeout` | `7a6461f4`, `808eb4c9`, `abb6fee9` |

## VOXEL

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-VOXEL-000 | `crates/voxel/src/lib.rs` | `schema_version_stub`, `kernel_reexports_resolve` | `71752001` |
| FR-CIV-VOXEL-001 | `crates/voxel/src/lib.rs` | `adaptive_storage` | `ae76e455` |
| FR-CIV-VOXEL-002 | `crates/voxel/src/lib.rs` | `dirty_queue_deterministic`, `dirty_events_sort_deterministically_through_reexport` | `ae76e455`, `71752001` |
| FR-CIV-VOXEL-003 | `crates/voxel/src/lib.rs` | `fixed_point_api` | `fa951238` |
| FR-CIV-VOXEL-004 | `crates/voxel/src/lib.rs` | `scale_multiplier_lod` | `71752001` |
| FR-CIV-VOXEL-005 | `crates/voxel/src/lib.rs` | `voxel_world_replay_is_bit_identical_through_reexport` | `d800e400` |
| FR-CIV-VOXEL-010 | `crates/voxel/src/lib.rs` | `mesher_watertight`, `voxel_world_to_cubic_mesh_end_to_end` | `ae76e455`, `71752001` |

## CA

Material cellular-automaton fields shipped for the dense `CaGrid` (per-cell
saturation, TPT thermal properties, percolation/capillary-lock, evaporation,
heat/phase). See `crates/voxel/src/fluid_ca.rs` and
`crates/voxel/src/material.rs`.

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-CA-001 | `crates/voxel/src/fluid_ca.rs` | `ca_grid_saturation_defaults_zero_and_round_trips`, `water_falls`, `step_n_hot_water_loses_saturation_cold_preserved`, `saturation_evaporation_pass_hot_water_dries` | `53a882d8` |
| FR-CIV-CA-002 | `crates/voxel/src/material.rs`, `crates/voxel/src/fluid_ca.rs` | `water_spreads_laterally`, `sand_piles`, `gas_rises`, `ice_melts_above_melting_point`, `oil_evaporation_ignored_when_cold`, `salt_water_boils_to_steam`, `molten_metal_boils_to_fire` | `afd591ce` |
| FR-CIV-CA-003 | `crates/voxel/src/fluid_ca.rs` | `percolation_into_dry_sand`, `capillary_lock_holds_water_below_field_capacity`, `gravity_excess_drains_above_field_capacity`, `capillary_rise_lifts_water_upward_even_below_capacity`, `percolation_oversaturates_and_drains` | `0599c609` |
| FR-CIV-CA-004 | `crates/voxel/src/fluid_ca.rs` | `steam_condenses_near_cold`, `water_evaporates_to_steam_when_hot`, `saturation_evaporation_pass_hot_water_dries`, `saturation_evaporation_pass_cold_water_unchanged`, `saturation_evaporation_pass_zero_saturation_no_op`, `step_n_hot_water_loses_saturation_cold_preserved` | `7ff3b772` |
| FR-CIV-CA-005 | `crates/voxel/src/fluid_ca.rs` | `heat_conduction_between_hot_and_cold`, `phase_change_smoke`, `fused_fluid_thermo_melts_after_heat_transfer`, `ice_melts_to_fluid_when_hot`, `static_world_step_no_change`, `snow_melts_above_freezing`, `lava_boils_to_fire_when_hot`, `acid_boils_to_steam`, `hot_water_evaporates_with_adjacent_air`, `steam_drifts_away_when_warm`, `heat_conduction_air_high_conduct`, `phase_transition_unknown_material` | — |

## SCALE

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-SCALE-001 | `crates/voxel/src/residency.rs`, `crates/voxel/src/lib.rs` | `fr_civ_scale_001_residency_mvp_defaults_match_fr`, `fr_civ_scale_001_residency_passes_at_exact_mvp_boundary`, `fr_civ_scale_001_residency_rejects_area_over_limit`, `fr_civ_scale_001_residency_rejects_chunk_below_minimum`, `fr_civ_scale_001_residency_error_display_mentions_fr`, `fr_civ_scale_001_residency_nan_and_inf_rejected`, `fr_civ_scale_001_residency_negative_area_clamps_and_passes_area_check`, `fr_civ_scale_001_residency_tuned_limits_round_trip` | `beb1d3c9` |

## PBR

3D physical-based-rendering baseline closure: SSAO + SSR + volumetric fog +
tonemapping + color grading for the main `bevy-ref` 3D camera.

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-PBR-001 | `clients/bevy-ref/src/post_fx.rs` | `post_fx_settings_default_all_true`, `post_fx_settings_partial_override` | `b9d1daab` |
| FR-CIV-PBR-002 | `clients/bevy-ref/src/post_fx.rs` | `post_fx_settings_default_all_true`, `post_fx_settings_partial_override` | `d4292e5b` |
| FR-CIV-PBR-003 | `clients/bevy-ref/src/post_fx.rs` | `post_fx_settings_default_all_true`, `post_fx_settings_partial_override` | `0d097b9c` |
| FR-CIV-PBR-004 | `clients/bevy-ref/src/post_fx.rs` | `post_fx_settings_default_all_true`, `post_fx_settings_partial_override` | `8c4b4d97` |

## SAVESLOT

Engine-side named save-slot management API + in-game save-slot browser UI.
The FR ID `FR-CIV-SAVESLOT` (per #1216) covers both the engine substrate
(`crates/engine/src/save_bundle.rs`) and the client browser panel
(`clients/bevy-ref/src/save_load_ui.rs`); the existing JSON-RPC-side
`FR-CIV-SAVE-*` rows above describe the server API surface for the same
feature family.

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-SAVESLOT | `crates/engine/src/save_bundle.rs`, `clients/bevy-ref/src/save_load_ui.rs` | `fr_civ_save_001_save_category_for_name_classifies_three_buckets`, `fr_civ_save_001_age_label_partitions_five_buckets`, `fr_civ_save_001_mtime_unix_seconds_clamps_pre_epoch`, `fr_civ_save_001_age_label_for_mtime_classifies_zero_age_as_just_now`, `fr_civ_save_001_list_saves_browser_returns_rich_rows`, `fr_civ_save_001_browser_agrees_with_list_saves`, `fr_civ_save_001_list_saves_browser_errors_on_missing_dir` | `a7f7088b`, `3f192abd` |

## TACTICS

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-TACTICS (umbrella) | `crates/engine/src/engine.rs` | `fr_civ_tactics_tick_resolves_in_range_combat` | `d02eff1a` |

## TUTORIAL

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-TUTORIAL | `crates/engine/src/engine.rs`, `crates/engine/src/tutorial.rs` | `fr_civ_tutorial_advances_from_tick_progress` | `b6c37d73` |

## EMERGENCE

The nine emergence systems wired into the sim tick per PR #1213. Per-system
sub-IDs (e.g. `FR-CIV-LEGENDS-001..008`, `FR-CIV-PSYCHE-900..921`,
`FR-CIV-RELIGION-001..008`) are tracked in
`docs/traceability/fr-emergence-matrix.md` and
`docs/traceability/emergent-systems-tracelinks.md`; the rows below capture
the umbrella IDs landed by PR #1213.

| FR ID | Code | Test | Commit |
|---|---|---|---|
| FR-CIV-LEGENDS | `crates/engine/src/emergence.rs`, `crates/legends/src/chronicle.rs`, `crates/legends/src/query.rs` | `fr_legends_auto_generation.rs`, `fr_legends_completion.rs` (Covers FR-CIV-LEGENDS-001..008 + LEGENDS-005/GRAPH-01/INGEST-02/RESOLVE-04 etc.) | `3ad9a650`, `6de7cf40` |
| FR-CIV-LANGUAGE | `crates/engine/src/engine.rs`, `crates/agents/src/language.rs` | `n13_coverage.rs` (FR-CIV-LANG-001..010 + PSYCHE-912 alias) | `3ad9a650` |
| FR-CIV-CULTURE | `crates/engine/src/engine.rs`, `crates/agents/src/culture.rs` | `culture_phase_drifts_cluster_profiles`, `n_series_coverage.rs` (FR-CIV-CULT-* via diffusion crate) | `3ad9a650` |
| FR-CIV-ECON | `crates/engine/src/engine.rs`, `crates/economy/src/market.rs`, `crates/economy/src/lib.rs` | `fr_matrix_batch1.rs`, `fr_matrix_batch3.rs`, `crates/economy/src/market.rs` market mod tests (FR-CIV-ECON-001..015) | `3ad9a650` |
| FR-CIV-PSYCHE | `crates/engine/src/engine.rs`, `crates/engine/src/era.rs`, `crates/legends/src/chronicle.rs`, `crates/ai/src/psyche_lod.rs`, `crates/engine/tests/fr_psyche_behavior.rs` | `fr_psyche_behavior.rs`, `fr_civ_psyche_920_appending_preserves_order_and_query_by_agent_is_exact`, `fr_civ_psyche_920_deterministic_under_same_input_stream`, `fr_civ_psyche_921_acceptance_demote_aggregates_promote_reconstructs`, `fr_civ_psyche_921_promote_is_deterministic`, `fr_civ_psyche_921_demote_is_a_projection_of_the_mean`, `fr_civ_psyche_921_event_log_caps_at_event_log_cap`, `fr_civ_psyche_921_round_trip_preserves_mood_within_tolerance`, `fr_civ_psyche_921_lod_tags_are_correct` | `3ad9a650` |
| FR-CIV-RELIGION | `crates/engine/src/engine.rs`, `crates/engine/src/religion.rs`, `crates/engine/tests/fr_civ_religion_007_phase_belief.rs` | `fr_civ_religion_007_phase_belief.rs`, `fr_civ_religion_001_big_gods_response_clamp_unit_interval`, `fr_civ_religion_002_rel_invariant_2_cap_monitor`, `fr_civ_religion_003_rel_invariant_3_cap_coherence`, `fr_civ_religion_004_rel_invariant_4_cap_uncertainty_reduction`, `fr_civ_religion_005_norenzayan_dominance_monitoring`, `fr_civ_religion_006_kin_density_decays_monitoring`, `fr_civ_religion_007_min_agents_guard`, `fr_civ_religion_008_rel_invariant_6_coherence_le_monitoring_plus_0_2` | `3ad9a650` |
| FR-CIV-DIPLOMACY | `crates/engine/src/engine.rs`, `crates/engine/tests/diplomacy_behavior.rs` | `war_drives_relation_score_negative`, `scarcity_raises_rivalry`, `exhaustion_lowers_engagement` | `3ad9a650` |
| FR-CIV-TECH | `crates/engine/src/engine.rs`, `crates/engine/src/era.rs` | `fr_civ_tech_ticks_unlock_faction_tech` | `3ad9a650` |
| FR-CIV-VICTORY | `crates/engine/src/engine.rs`, `crates/engine/src/conditions.rs` | `n_series_coverage.rs` (TURN-010 victory conditions + TURN-065..080 wrap-up), `fr_matrix_batch1.rs` | `3ad9a650` |

## Commit log

- `feat(trace): add FR-CIV-LIFE row to full-traceability-matrix.md (matrix-2026-06-26)` — adds `### Life-Sim — P-LS` section to `docs/traceability/full-traceability-matrix.md` covering FR-CIV-LIFE-001/002/003 (needs decay / sickness / death in `crates/needs/src/lib.rs`), FR-CIV-LIFE-010..014 (utility-AI daily path / POI in `crates/agents/src/daily_path.rs`), FR-CIV-LIFE-020..025 (mercantile resource stocks in `crates/economy/src/stocks.rs`), and FR-CIV-LIFE-030..035 (emergent clusters in `crates/agents/src/cluster.rs`); updates coverage summary + headline metrics; matrix snapshot updated to reflect 6 new `Partial` rows (unit-tested primitives, no engine-tick integration test yet).
- `docs(traceability): sync FR-CIV-CA/SCALE/PBR/SAVE/TUTORIAL/TACTICS/emergence entries (traceability sync)` — appends new sections to `docs/traceability/civis-tracelinks.md` covering: FR-CIV-CA-001..005 (`crates/voxel/src/fluid_ca.rs` + `material.rs`, PRs #1248/#1249/#1252/#1253 — CA-005 has no PR yet so commit is `—`); FR-CIV-SCALE-001 (`crates/voxel/src/residency.rs`, PR #1250); FR-CIV-PBR-001..004 (`clients/bevy-ref/src/post_fx.rs`, PRs #1218/#1241/#1244/#1245 — SSAO/SSR/fog/tonemap); FR-CIV-SAVESLOT (engine `save_bundle.rs` + bevy-ref `save_load_ui.rs`, PRs #1216/#1217); FR-CIV-TUTORIAL / FR-CIV-TACTICS (engine `engine.rs` + `tutorial.rs`, PRs #1215/#1214); and 9 emergence systems (LEGENDS/LANGUAGE/CULTURE/ECON/PSYCHE/RELIGION/DIPLOMACY/TECH/VICTORY, PR #1213 + `6de7cf40` for LEGENDS). Per-system sub-IDs continue to live in `fr-emergence-matrix.md` and `emergent-systems-tracelinks.md`; rows here capture the umbrella IDs from this session's merges.
