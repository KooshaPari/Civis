# 3D Extension Traceability Matrix

**Status:** Active — tracks `FR-CIV-*` IDs from `docs/development-guide/fr-3d-additions.md`.
**Format:** FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status

Status values: `planned` | `in_progress` | `implemented`

> Strategic `FR-CORE-*` / `FR-ECON-*` rows remain in
> [`TRACEABILITY_MATRIX.md`](TRACEABILITY_MATRIX.md). This file is the traceability
> home for the 3D workspace extension until rows are merged upstream.

---

## Voxel (FR-CIV-VOXEL-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-VOXEL-000 | Crate compiles; exposes `SCHEMA_VERSION` (stub). | `crates/voxel/` | `voxel::schema_version_stub` | implemented |
| FR-CIV-VOXEL-001 | Adaptive storage: O(1) writes in 16³ leaf; deterministic octree branching. | `crates/voxel/` | `voxel::adaptive_storage` | implemented |
| FR-CIV-VOXEL-002 | Deterministic dirty queue ordered by `(chunk_id, write_seq)`. | `crates/voxel/` | `voxel::dirty_queue_deterministic` | implemented |
| FR-CIV-VOXEL-003 | Fixed-point world coords; no `f32`/`f64` in public API. | `crates/voxel/` | `voxel::fixed_point_api` | implemented |
| FR-CIV-VOXEL-004 | `VoxelScaleMultiplier` LOD composition invariant. | `crates/voxel/` | `voxel::scale_multiplier_lod` | implemented |
| FR-CIV-VOXEL-010 | `Mesher` trait: watertight meshes for fixed test scene. | `crates/voxel/` | `voxel::mesher_watertight` | implemented |

---

## Build (FR-CIV-BUILD-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-BUILD-000 | Stub. | `crates/build/` | `build::schema_version_stub` | implemented |
| FR-CIV-BUILD-001 | `BuildingGraph` RON round-trip without loss. | `crates/build/` | `build::graph_ron_roundtrip` | implemented |
| FR-CIV-BUILD-010 | Deterministic parcel scoring from demand signals. | `crates/build/` | `build::demand_allocation_deterministic` | implemented |
| FR-CIV-BUILD-020 | Freehand tools emit same `BuildingGraph` mods as grammar equivalents. | `crates/build/` | `build::freehand_matches_grammar` | implemented |
| FR-CIV-BUILD-030 | Era-grammar transitions produce expected facade histogram. | `crates/build/` | `build::era_grammar_histogram` | implemented |

---

## Genetics (FR-CIV-GENETICS-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-GENETICS-000 | Stub. | `crates/genetics/` | `genetics::schema_version_stub` | implemented |
| FR-CIV-GENETICS-001 | Mutation deterministic under fixed seed. | `crates/genetics/` | `genetics::mutation_deterministic` | implemented |
| FR-CIV-GENETICS-002 | Recombination draws offspring loci deterministically from parents. | `crates/genetics/` | `genetics::recombination_deterministic` | implemented |
| FR-CIV-GENETICS-010 | Speciation trigger deterministic; emits new species record. | `crates/genetics/` | `genetics::speciation_trigger` | implemented |

---

## Species (FR-CIV-SPECIES-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-SPECIES-000 | Stub. | `crates/species/` | `species::schema_version_stub` | implemented |
| FR-CIV-SPECIES-001 | DNA → phenotype mapping deterministic. | `crates/species/` | `species::phenotype_deterministic` | implemented |

---

## Agents (FR-CIV-AGENTS-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-AGENTS-000 | Stub. | `crates/agents/` | `agents::schema_version_stub` | implemented |
| FR-CIV-AGENTS-001 | Wardrobe + tools state ticks deterministically. | `crates/agents/` | `agents::wardrobe_tools_deterministic` | implemented |
| FR-CIV-AGENTS-010 | LOD tick: lower frequency without state divergence (gestalt). | `crates/agents/` | `agents::lod_gestalt_no_divergence` | implemented |

---

## Diffusion (FR-CIV-DIFFUSION-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-DIFFUSION-000 | Stub. | `crates/diffusion/` | `diffusion::schema_version_stub` | implemented |
| FR-CIV-DIFFUSION-001 | Bass/Rogers S-curve adoption matches closed-form within tolerance. | `crates/diffusion/` | `diffusion::s_curve_adoption` | implemented |

---

## Laws (FR-CIV-LAWS-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-LAWS-000 | Stub. | `crates/laws/` | `laws::schema_version_stub` | implemented |
| FR-CIV-LAWS-001 | Versioned RON schema loads and round-trips. | `crates/laws/` | `laws::ron_roundtrip` | implemented |
| FR-CIV-LAWS-002 | Validator rejects extensions missing required fields. | `crates/laws/` | `laws::validator_rejects_incomplete` | implemented |
| FR-CIV-LAWS-006 | `LawDb::get` returns correct law by id and `None` for missing ids. | `crates/laws/` | `laws::get_*` | implemented |

---

## Research (FR-CIV-RESEARCH-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-RESEARCH-000 | Stub. | `crates/research/` | `research::schema_version_stub` | implemented |
| FR-CIV-RESEARCH-001 | LLM cache hit is byte-identical to cached value. | `crates/research/` | `research::llm_cache_hit` | implemented |
| FR-CIV-RESEARCH-002 | Canonical replay refuses first `LlmEvent` in log. | `crates/research/` | `research::canonical_replay_refuses_llm` | implemented |
| FR-CIV-RESEARCH-003 | Hybrid replay on cache miss refuses to advance. | `crates/research/` | `research::hybrid_cache_miss_refuses` | implemented |

---

## Tactics (FR-CIV-TACTICS-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-TACTICS-000 | Stub. | `crates/tactics/` | `tactics::schema_version_stub` | implemented |
| FR-CIV-TACTICS-001 | Voxel-destructible damage application is deterministic. | `crates/tactics/`, `engine.rs` | `apply_damage_is_deterministic`, `pending_damage_drains` | implemented |
| FR-CIV-TACTICS-001-int | Damage pulses on watch/server snapshot + web impact markers. | `watch`, `server/jsonrpc`, `web/dashboard` | `snapshot_fields_from_sim_includes_damage_after_tick` | implemented |
| FR-CIV-TACTICS-010 | Doctrine GA converges reproducibly under fixed seed. | `crates/tactics/`, `engine.rs` | `tactics::doctrine_ga_converges`, `phase_tactics_evolve_doctrine_on_cadence` | implemented |
| FR-CIV-TACTICS-020 | Voxel line-of-sight is deterministic. | `crates/tactics/los.rs` | `line_of_sight_blocks_solid_voxels` | implemented |
| FR-CIV-TACTICS-021 | Formation offsets are stable per kind/slot count. | `crates/tactics/formation.rs` | `formation_offsets_line_and_wedge` | implemented |
| FR-CIV-TACTICS-022 | War bridge queues voxel damage on cadence with LOS. | `crates/tactics/war_bridge.rs`, `engine.rs` | `war_bridge_queues_damage_on_cadence_with_los` | implemented |
| FR-CIV-TACTICS-023 | Doctrine fitness scores faction engagement stats. | `crates/tactics/doctrine_fitness.rs` | `doctrine_fitness_rewards_engagement_stats` | implemented |
| FR-CIV-TACTICS-024 | Per-soldier combat engagements on snapshot. | `crates/tactics/war_bridge.rs`, `engine.rs`, `server/jsonrpc` | `war_bridge_records_combat_replay_events` | implemented |
| FR-CIV-TACTICS-025 | Combat engagements recorded in replay log. | `crates/engine/src/replay.rs` | `war_bridge_records_combat_replay_events` | implemented |
| FR-CIV-TACTICS-030 | Operational layer hook for engagements. | `crates/tactics/operational.rs` | (compile + `NoopOperationalLayer` in engine) | implemented |
| FR-CIV-TACTICS-031 | Operational movement toward nearest enemy. | `crates/tactics/movement.rs` | `operational_movement_steps_toward_enemy` | implemented |
| FR-CIV-TACTICS-032 | Per-soldier HP on military ECS component. | `engine.rs`, `spawn.rs` | `war_bridge_records_combat_replay_events` | implemented |
| FR-CIV-TACTICS-033 | Operational pathfinding (BFS next step). | `crates/tactics/pathfinding.rs` | `pathfinding_bfs_steps_toward_enemy` | implemented |
| FR-CIV-TACTICS-034 | Military-phase mod hook (`read_military`). | `crates/mod-host/`, `engine.rs` | `mod_registry_military_phase_emits_for_read_military` | implemented |
| FR-CIV-TACTICS-035 | Higher per-tick military work (cadence + pulses). | `military_phase.rs`, `movement.rs`, `war_bridge.rs` | `operational_movement_steps_toward_enemy` | implemented |
| FR-CIV-TACTICS-025-int | Replay restores combat `pending_damage`. | `replay.rs`, `engine.rs` | `replay_combat_events_restore_pending_damage` | implemented |
| FR-CIV-TACTICS-025-int2 | Replay combat drains match live voxel state. | `engine.rs` | `replay_combat_drains_to_same_voxel_state_as_live` | implemented |
| FR-CIV-TACTICS-025-int3 | Combat replay log deterministic per seed. | `engine.rs` | `replay_combat_log_deterministic_for_seed_rerun` | implemented |
| FR-CIV-TACTICS-036 | Voxel obstacle grid for operational movement. | `grid_obstacles.rs`, `movement.rs` | `operational_movement_avoids_voxel_obstacle` | implemented |
| FR-CIV-TACTICS-037 | A* obstacle-aware pathfinding. | `pathfinding.rs` | `astar_path_with_blocked` | implemented |
| FR-CIV-TACTICS-038 | WASM `civlab_military_tick` host invoke. | `wasm_guest.rs`, `civlab-sdk` | `wasm_military_tick_invokes_civlab_export` | implemented |
| FR-CIV-TACTICS-039 | Occupied-cell blocking in operational pathfinding. | `grid_obstacles.rs`, `movement.rs` | `operational_movement_avoids_occupied_cell` | implemented |
| FR-CIV-TACTICS-040 | Military WASM tick receives `sim_tick` (i64). | `wasm_guest.rs`, `mod-host`, `civlab-sdk` | `wasm_military_tick_invokes_civlab_export` | implemented |
| FR-CIV-TACTICS-041 | Combat events extend replay hash chain. | `hash_chain.rs`, `replay.rs` | `combat_events_extend_replay_hash_chain` | implemented |
| FR-CIV-TACTICS-042 | Fog-of-war gates war-bridge engagements. | `fog_of_war.rs`, `war_bridge.rs`, `engine.rs` | `fog_blocks_engagement_beyond_vision` | implemented |
| FR-CIV-TACTICS-043 | Ed25519 mod WASM signature verification. | `mod-host/signature.rs` | `verify_accepts_valid_signature` | implemented |
| FR-CIV-TACTICS-044 | Policy/military WASM tick capability API. | `wasm_guest.rs`, `civlab-sdk` | `capability_version_is_non_empty` | implemented |
| FR-CIV-TACTICS-045 | Scenario fog fields wire military phase. | `scenario.rs`, `scenarios/baseline.yaml` | `scenario_fog_wires_military_phase` | implemented |
| FR-CIV-TACTICS-046 | Economic WASM tick host invoke. | `wasm_guest.rs`, `mod-host`, `engine.rs` | `wasm_economy_tick_invokes_civlab_export` | implemented |
| FR-CIV-TACTICS-047 | WASM host `civlab::capability_api_version` import. | `wasm_guest.rs` | `wasm_guest_reads_capability_host_import` | implemented |
| FR-CIV-TACTICS-048 | Example economic mod directory + WASM tick. | `mods/example-economic/`, `mod-host` | `example_economic_dir_wasm_ticks_when_built` | implemented |
| FR-CIV-TACTICS-049 | Host guest memory read/write imports. | `wasm_guest.rs` | `host_memory_import_read_write` | implemented |
| FR-CIV-TACTICS-050 | Scenario military cadence/range tuning. | `scenario.rs`, `baseline.yaml` | `scenario_military_wires_military_phase` | implemented |
| FR-CIV-TACTICS-051 | Baseline scenario loads example-economic mod. | `scenarios/baseline.yaml` | `baseline_yaml_parses` | implemented |
| FR-CIV-TACTICS-052 | Per-mod guest memory snapshots on ModHost. | `mod-host` | `guest_memory_persists_across_invocations` | implemented |
| FR-CIV-TACTICS-053 | Full capability host imports (`sim_tick`, import list). | `wasm_guest.rs`, `civlab-sdk` | `sim_tick_host_import_visible_to_guest` | implemented |
| FR-CIV-TACTICS-054 | Mod browser on watch/server snapshot + dashboard. | `watch`, `server`, `web/dashboard` | `mods_panel` | implemented |
| FR-CIV-TACTICS-055 | Mod guest state JSON save/load stub. | `guest_state.rs`, `engine.rs` | `mod_guest_state_save_round_trips_json` | implemented |
| FR-CIV-TACTICS-056 | WASM determinism scan at mod load. | `determinism.rs` | `rejects_f32_sqrt` | implemented |
| FR-CIV-TACTICS-057 | Float opcode count in determinism report. | `determinism.rs` | `report_counts_float_ops_without_hard_reject` | implemented |
| FR-CIV-TACTICS-061 | `action_emit` float data-flow trace at mod load. | `float_data_flow.rs`, `determinism.rs` | `rejects_reinterpret_f64_before_action_emit` | implemented |
| FR-CIV-TACTICS-058 | `.civsave/` debug save folder. | `save_bundle.rs`, `civ-watch` | `civsave_folder_round_trips_mod_guest_state` | implemented |
| FR-CIV-TACTICS-059 | Package example mods as `.civmod`. | `justfile`, `package-example-mod.ps1` | `civis-3d-mod-package-all` | implemented |
| FR-CIV-TACTICS-060 | `.civsave.zst` compressed save archive. | `save_bundle.rs`, `civ-watch` | `civsave_archive_round_trips_mod_guest_state` | implemented |
| FR-CIV-TACTICS-062 | Mod catalog + runtime install. | `civ-watch`, `web/dashboard` | `post_mods_install` | implemented |
| FR-CIV-TACTICS-063 | Mod unload HTTP + `mod.unloaded.v1` bus JSON. | `civ-watch`, `mod-host` | `post_mods_unload` | implemented |
| FR-CIV-TACTICS-064 | Signed `.civmod` upload API. | `civ-watch` | `post_mods_upload` | implemented |
| FR-CIV-TACTICS-065 | Production slots + autosave ring. | `civ-watch` | `post_save_slot`, `autosave_ring` | implemented |
| FR-CIV-TACTICS-066 | `save.slot` / `save.load` / `save.list` JSON-RPC. | `civ-server` | `ws_jsonrpc_save_slot_roundtrip` | implemented |
| FR-CIV-TACTICS-067 | Mod publish store + HTTP API. | `civ-watch` | `post_mods_publish` | implemented |
| FR-CIV-TACTICS-068 | Mod hot reload API. | `civ-watch`, `mod-host` | `post_mods_reload` | implemented |
| FR-CIV-TACTICS-069 | Session-scoped SQLite save metadata. | `civ-save-db`, `civ-watch` | `save_db::*`, `post_save_slot` | implemented |
| FR-CIV-TACTICS-070 | Remote mod fetch cache. | `civ-watch`, `mods/remote/` | `post_mods_fetch`, `list_remote_mods` | implemented |
| FR-CIV-TACTICS-071 | CIV-0700 capability enforcement (`world_read`, `action_emit`). | `mod-host/capability.rs` | `capability::*` | implemented |
| FR-CIV-TACTICS-072 | `session.saved.v1` on replay bus + snapshot. | `engine/replay.rs`, `civ-watch` | `session_saved_*` | implemented |
| FR-CIV-TACTICS-073 | Web remote mod fetch UI. | `web/dashboard/mods_panel.tsx` | manual / tsc | implemented |
| FR-CIV-TACTICS-074 | PolicyMod trait surface (CIV-0700 §5). | `civlab-sdk/policy.rs` | `policy::*` | implemented |
| FR-CIV-TACTICS-075 | `mod.permission_violation.v1` on replay bus + snapshot. | `engine/replay.rs`, `engine.rs`, `civ-watch` | `mod_permission_violation_*` | implemented |
| FR-CIV-TACTICS-076 | civ-server session-scoped SQLite on `save.slot`. | `civ-save-db`, `server/ws_bridge.rs` | `save.slot` smoke | implemented |
| FR-CIV-TACTICS-077 | Signed remote mod registry for fetch. | `mods/remote-registry.json`, `civ-watch` | `remote_registry_*` | implemented |

---

## Bevy reference client (FR-CIV-BEVY-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-BEVY-023 | P-W1 kickoff **item 48**: `event_feed` egui toasts + scrollable log on `civ-standalone`; `live_attach` pushes `EventKind::System` on WebSocket `connected` / `reconnecting` / `disconnected`. | `clients/bevy-ref` (`event_feed`, `live_attach`, `standalone`) | `cargo test -p civ-bevy-ref --features bevy,egui --lib event_feed::`, `cargo check -p civ-bevy-ref --features bevy,egui --bin civ-standalone` | implemented |
| FR-CIV-BEVY-024 | P-W1 kickoff **item 49**: `MenusPlugin` on `civ-standalone` — Escape pause overlay, settings stub, era banner; in-process sim tick gated while paused (`GameUiMode` / HUD speed `0`). | `clients/bevy-ref` (`menus`, `lib.rs`, `bin/standalone.rs`, `sim_bridge`) | `cargo check -p civ-bevy-ref --features bevy,egui --bin civ-standalone`, `cargo test -p civ-bevy-ref --features bevy,egui --lib menus::` | implemented |
| FR-CIV-BEVY-026 | P-W1 kickoff **item 51**: document and test native GPU backend selection (`CIV_BEVY_BACKEND`, DX12/Vulkan/Metal; no GLES). | `clients/bevy-ref` (`native_backend`), `clients/bevy-ref/README.md`, `docs/research/wgpu-native-escape-hatches.md` | `cargo test -p civ-bevy-ref --features bevy --lib native_backend`, `just civis-3d-live-smoke` | implemented |

---

## Planet (FR-CIV-PLANET-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-PLANET-000 | Stub. | `crates/planet/` | `planet::schema_version_stub` | implemented |
| FR-CIV-PLANET-001 | Day/night cycle deterministic and tied to tick. | `crates/planet/` | `planet::day_night_deterministic` | implemented |
| FR-CIV-PLANET-002 | Moon tides modulate coastal water level deterministically. | `crates/planet/` | `planet::tides_deterministic` | implemented |

---

## Protocol 3D (FR-CIV-PROTO3D-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-PROTO3D-000 | Stub. | `crates/protocol-3d/` | `protocol3d::schema_version_stub` | implemented |
| FR-CIV-PROTO3D-001 | Voxel delta frames binary serialize; lossless round-trip. | `crates/protocol-3d/` | `protocol3d::voxel_delta_roundtrip` | implemented |
| FR-CIV-PROTO3D-002 | Building diff frames carry procedural vs freehand provenance. | `crates/protocol-3d/` | `protocol3d::building_diff_provenance` | implemented |

---

## UX (FR-CIV-UX-*)

| FR ID | Requirement Summary | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|
| FR-CIV-UX-000 | Spawn API: N UI spawns emit N entity-create events. | `clients/godot-ref/` | `ux::spawn_emits_entity_events` | implemented |
| FR-CIV-UX-001 | Era timelapse: configurable rate without divergence vs real-time. | `clients/godot-ref/` | `ux::timelapse_no_divergence` | implemented |
| FR-CIV-UX-004 | Drag-place + convoy along path. | `godot-ref`, `web/dashboard` | `ux::convoy_positions`, `spawnConvoy.ts` | implemented |
| FR-CIV-UX-006 | Spawn palette incl. hangar. | `engine/spawn.rs`, server, watch | `spawn_kind_palette_is_wired` | implemented |
| FR-CIV-GODOT-ATTACH-000..004 | Godot civ-server WS + watch terrain. | `civis_ws_client.gd` | `ws_smoke`, attach docs | implemented |

---

*Last updated: 2026-05-25. Source of truth for FR text: `docs/development-guide/fr-3d-additions.md`.*
