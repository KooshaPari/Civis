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
