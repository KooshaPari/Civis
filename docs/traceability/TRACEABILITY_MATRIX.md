# Traceability Matrix

**Status:** Living document — strategic simulation FRs only (CIV-01xx line).
**Format:** FR ID | Requirement Summary (SHALL) | Spec Doc | Crate / Source Path | Test Name Pattern | Status

Status values: `planned` | `in_progress` | `implemented`

> **Do not add 3D or web rows here.** Use the dedicated matrices below; this file stays
> the home for `FR-CORE-*`, `FR-ECON-*`, and other CIV-01xx strategic IDs.
>
> | Extension | Authoritative matrix | FR source doc |
> |-----------|---------------------|---------------|
> | **3D workspace** (`civ-voxel`, `civ-server`, Godot, …) | [`docs/traceability/fr-3d-matrix.md`](fr-3d-matrix.md) | [`docs/development-guide/fr-3d-additions.md`](../development-guide/fr-3d-additions.md) |
> | **Web spectator / L2 dashboard** | [`docs/traceability/fr-web-matrix.md`](fr-web-matrix.md) | [`docs/development-guide/fr-web-spectator.md`](../development-guide/fr-web-spectator.md) |
>
> **Workspace note (2026-05-25):** The **Crate / Source Path** column below describes the
> *target* layout from CIV-01xx strategic specs. The active repo is the 3D extension
> (`civ-engine`, `civ-voxel`, `civ-planet`, …). See
> [`docs/IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md) for the live crate list and
> gap summary before marking any row `implemented`.

**Governance traceability pillars:** `CIV-CORE-1` (simulation core),
`CIV-POLICY-1` (policy / quality gates), `CIV-METRICS-1` (metrics export),
`CIV-EVENT-1` (event taxonomy — see [`EVENT_TAXONOMY.md`](EVENT_TAXONOMY.md)).

---

## Core Engine (FR-CORE-*)

Source spec: `docs/specs/CIV-0001-core-simulation-loop.md`  
**Implemented in:** package `civ-engine` (`crates/engine/`). ECS is `hecs`, not Bevy.

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-CORE-001 | The engine SHALL advance simulation state by exactly one tick per `Engine::step()` invocation. | CIV-0001 | `crates/engine/src/lib.rs`, `crates/engine/src/engine.rs` | `step_advances_tick`, `test_tick_advances` | implemented |
| FR-CORE-002 | The engine SHALL produce identical output for identical seed and input sequence (determinism). | CIV-0001 | `crates/engine/src/engine.rs` | `determinism_same_seed_same_output`, `test_determinism`, `determinism_holds_with_all_phases_enabled` | implemented |
| FR-CORE-003 | The engine SHALL use ChaCha20Rng seeded per-run; no global mutable RNG state. | CIV-0001 | `crates/engine/src/engine.rs` (`ChaCha8Rng`; deviation tracked in `docs/adr/ADR-022-runtime-representation-deviations.md`) | `rng::no_global_rng_state` (seeded-per-run + no shared state verified) | in_progress |
| FR-CORE-004 | Each tick SHALL complete within 100 ms wall-clock on the reference hardware profile. | CIV-0001 | `crates/engine/src/engine.rs` | `perf::tick_under_100ms`, `tick_perf_100_ticks_under_10s` | implemented |
| FR-CORE-005 | The engine SHALL emit a BLAKE3 hash of full world state at the end of every tick. | CIV-0001 | `crates/engine/src/hash_chain.rs` | `hash_chain::tick_hash_emitted` | implemented |
| FR-CORE-006 | Consecutive tick hashes SHALL form an append-only chain (each hash includes prior hash). | CIV-0001 | `crates/engine/src/hash_chain.rs` | `hash_chain::chain_includes_prior` | implemented |
| FR-CORE-007 | The engine SHALL surface a `run.hash.mismatch.v1` event when replayed state diverges. | CIV-0001 | `crates/engine/src/integrity.rs` | `integrity::mismatch_event_emitted` | implemented |
| FR-CORE-008 | World state SHALL be modelled as bevy_ecs 0.18.x `World`; no global singletons. | CIV-0001 | `crates/engine/src/engine.rs` (`hecs::World`; deviation tracked in `docs/adr/ADR-022-runtime-representation-deviations.md`) | `world::no_global_resources` | in_progress |
| FR-CORE-009 | Hex grid SHALL use `hexx` 0.21.x axial coordinates throughout engine and render crates. | CIV-0001 | `crates/engine/src/grid.rs` (`PositionAxial`, `PositionCube`) | `grid::axial_roundtrip`, `grid::cube_roundtrip` | implemented |
| FR-CORE-010 | All integer quantities SHALL use fixed-point types (`FixedI32\<U16\>`, `i64` KiloJoules, `i64` MilliCredits). | CIV-0001 | `crates/engine/src/fixed_math.rs` (`Fixed` i64; deviation tracked in `docs/adr/ADR-022-runtime-representation-deviations.md`) | `numerics::integer_quantities_use_fixed_point` | in_progress |

---

## Economy (FR-ECON-*)

> **Crate `crates/economy`:** Active workspace crate with production, consumption, waste, GDP, district, trade, and metrics modules.

Source specs: `docs/specs/CIV-0100-economy.md`, `docs/specs/CIV-0107-joule-economy.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-ECON-001 | Each district SHALL produce Joules each tick according to its resource type and capacity. | CIV-0100 | `crates/economy/src/production.rs` | `production::district_produces_joules` | implemented |
| FR-ECON-002 | Joule consumption SHALL be deducted from district reserves before regional distribution. | CIV-0107 | `crates/economy/src/distribution.rs` (`deduct_consumption`, `step_distribution`) | `deducted_before_distribution` | implemented |
| FR-ECON-003 | Joule consumption per tick SHALL never be negative (consumption_non_negative invariant). | CIV-0107 | `crates/engine/src/policy.rs` (`effective_consumption`) | `consumption_non_negative` | implemented |
| FR-ECON-004 | Surplus Joules SHALL flow to adjacent districts via the distribution graph each tick. | CIV-0100 | `crates/economy/src/distribution.rs` (`distribute_surplus`, `DistrictGraph`) | `surplus_flows_adjacent` | implemented |
| FR-ECON-005 | Waste heat SHALL be computed as a percentage of total Joules consumed per tick. | CIV-0107 | `crates/economy/src/waste.rs` | `waste::heat_computed_from_consumption` | implemented |
| FR-ECON-006 | GDP SHALL be derived from sum of regional Joule throughput converted at a fixed exchange rate. | CIV-0100 | `crates/economy/src/gdp.rs` | `gdp::sum_of_regional_joules` | implemented |
| FR-ECON-007 | Trade agreements SHALL transfer Joules and MilliCredits between civilizations each tick. | CIV-0100 | `crates/economy/src/trade.rs` (`TradeAgreement`, `bilateral_transfer`) | `trade::bilateral_transfer_balanced`, `bilateral_transfer_insufficient_joules` | implemented |
| FR-ECON-008 | A district in Joule deficit for 3 consecutive ticks SHALL emit `economy.district.collapsed.v1`. | CIV-0100 | `crates/economy/src/district.rs` | `district::collapse_after_deficit_ticks` | implemented |
| FR-ECON-009 | Subsistence mode SHALL activate when a civilization's total Joule balance drops below threshold. | CIV-0107 | `crates/economy/src/subsistence.rs` (`SubsistenceMode`) | `subsistence::activates_below_threshold`, `subsistence_deactivates_above_threshold` | implemented |
| FR-ECON-010 | Treasury balance SHALL be tracked in MilliCredits (`i64`) with no floating-point accumulation. | CIV-0100 | `crates/economy/src/treasury.rs` (`Treasury`) | `treasury::no_float_accumulation`, `treasury_credit_debit_joules` | implemented |

---

## Level of Detail (FR-LOD-*)

Source spec: `docs/specs/CIV-0101-lod.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-LOD-001 | The engine SHALL support two zoom levels: strategic (region) and operational (district/hex). | CIV-0101 | `crates/engine/src/lod.rs` | `lod::two_levels_defined` | implemented |
| FR-LOD-002 | Strategic view SHALL aggregate district data into region summaries each tick. | CIV-0101 | `crates/engine/src/lod.rs` | `lod::strategic_aggregation` | implemented |
| FR-LOD-003 | LOD transitions SHALL not alter simulation state, only view projection. | CIV-0101 | `crates/engine/src/lod.rs` | `lod::transition_no_state_mutation` | implemented |
| FR-LOD-004 | Operational view SHALL expose individual hex-cell resource and population data. | CIV-0101 | `crates/engine/src/lod.rs` | `lod::operational_hex_data_visible` | implemented |

---

## Climate (FR-CLIM-*)

Source spec: `docs/specs/CIV-0102-climate.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-CLIM-001 | Atmospheric CO2 SHALL accumulate each tick based on industrial Joule consumption. | CIV-0102 | `crates/climate/src/co2.rs` | `co2::accumulates_with_consumption` | implemented |
| FR-CLIM-002 | Global mean temperature SHALL be derived from CO2 concentration via parameterised formula. | CIV-0102 | `crates/climate/src/temperature.rs` | `temperature::derived_from_co2` | implemented |
| FR-CLIM-003 | The engine SHALL emit `climate.threshold.crossed.v1` when temperature crosses a defined level. | CIV-0102 | `crates/climate/src/events.rs` | `climate_events::threshold_event_emitted` | implemented |
| FR-CLIM-004 | Climate damage SHALL reduce district Joule production capacity when temperature exceeds threshold. | CIV-0102 | `crates/climate/src/damage.rs` | `damage::reduces_production_above_threshold` | implemented |
| FR-CLIM-005 | The engine SHALL model at least one tipping-point cascade (e.g. ice-albedo) above critical temperature. | CIV-0102 | `crates/climate/src/tipping.rs` | `tipping::cascade_triggered` | implemented |
| FR-CLIM-006 | Civilizations SHALL be able to invest MilliCredits into adaptation to reduce climate damage. | CIV-0102 | `crates/climate/src/adaptation.rs` | `adaptation::investment_reduces_damage` | implemented |

---

## Institutions (FR-INST-*)

Source spec: `docs/specs/CIV-0103-institutions.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-INST-001 | Each civilization SHALL have an institutional type (democracy, autocracy, technocracy, etc.). | CIV-0103 | `crates/civ-institutions/src/governance.rs` | `governance_type_assigned_at_init` | implemented |
| FR-INST-002 | Institutional capture score SHALL accumulate each tick based on resource concentration. | CIV-0103 | `crates/civ-institutions/src/capture.rs` | `capture_accumulates_with_concentration` | implemented |
| FR-INST-003 | The engine SHALL emit `institution.capture.threshold.v1` when capture crosses 0.75. | CIV-0103 | `crates/civ-institutions/src/events.rs` | `inst_events_capture_threshold_event` | implemented |
| FR-INST-004 | Institutional collapse SHALL trigger a governance type transition. | CIV-0103 | `crates/civ-institutions/src/collapse.rs` | `collapse_triggers_type_transition` | implemented |
| FR-INST-005 | Institution time-series data SHALL be stored in the metrics DB for post-run analysis. | CIV-0103 | `crates/civ-institutions/src/lib.rs` (`InstitutionTimeSeries`) | `db_institution_series_stored` | implemented |
| FR-INST-006 | Citizen lifecycle (birth, migration, death) SHALL be driven by institutional and economic state. | CIV-0103 | `crates/civ-institutions/src/lib.rs` (`evaluate_lifecycle`) | `lifecycle_driven_by_inst_economy` | implemented |

---

## Theorem / Invariants (FR-THRY-*)

Source spec: `docs/specs/CIV-0104-theorem.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-THRY-001 | Total Joule energy in a closed system SHALL be conserved each tick (production - consumption - waste = 0). | CIV-0104 | `crates/engine/src/invariants.rs` | `invariants::joule_conservation` | implemented |
| FR-THRY-002 | Total MilliCredit supply SHALL remain constant absent explicit treasury mint/burn operations. | CIV-0104 | `crates/engine/src/invariants.rs` | `invariants::credit_supply_conserved` | implemented |
| FR-THRY-003 | Population delta per tick SHALL equal births minus deaths minus emigration plus immigration. | CIV-0104 | `crates/engine/src/invariants.rs` | `invariants::population_delta_balanced` | implemented |
| FR-THRY-004 | The invariant checker SHALL run every tick and panic in debug builds on violation. | CIV-0104 | `crates/engine/src/invariants.rs` | `invariants::checker_panics_on_violation` | implemented |

---

## Diplomacy (FR-DIPL-*)

Source spec: `docs/specs/CIV-0105-war-diplomacy.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-DIPL-001 | Civilizations SHALL be able to declare war, producing `diplomacy.war.declared.v1`. | CIV-0105 | `crates/diplomacy/src/war.rs` | `war::declare_emits_event` | implemented |
| FR-DIPL-002 | Peace SHALL be negotiated via signed treaty, producing `diplomacy.peace.signed.v1`. | CIV-0105 | `crates/diplomacy/src/peace.rs` | `peace::signed_emits_event` | implemented |
| FR-DIPL-003 | Treaties SHALL encode terms (trade ratios, non-aggression, alliance) as structured data. | CIV-0105 | `crates/diplomacy/src/treaty.rs` | `treaty::terms_structured` | implemented |
| FR-DIPL-004 | Treaty breach SHALL emit `diplomacy.treaty.broken.v1` and apply reputation penalty. | CIV-0105 | `crates/diplomacy/src/treaty.rs` | `treaty::breach_emits_event_and_penalty` | implemented |
| FR-DIPL-005 | Espionage operations SHALL have a configurable detection probability per tick. | CIV-0105 | `crates/diplomacy/src/espionage.rs` | `espionage::detection_probability_applied` | implemented |
| FR-DIPL-006 | Detected espionage SHALL emit `diplomacy.espionage.detected.v1`. | CIV-0105 | `crates/diplomacy/src/espionage.rs` | `espionage::detected_emits_event` | implemented |
| FR-DIPL-007 | Shadow networks SHALL model covert influence as a hidden resource accumulating per tick. | CIV-0105 | `crates/diplomacy/src/shadow.rs` | `shadow::influence_accumulates` | implemented |

---

## Social (FR-SOCI-*)

Source spec: `docs/specs/CIV-0106-social.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-SOCI-001 | Ideological alignment SHALL be tracked per-citizen cohort as a continuous score. | CIV-0106 | `crates/social/src/ideology.rs` | `ideology::per_cohort_continuous` | implemented |
| FR-SOCI-002 | Citizen stress SHALL accumulate when Joule access falls below subsistence level. | CIV-0106 | `crates/social/src/stress.rs` | `stress::accumulates_below_subsistence` | implemented |
| FR-SOCI-003 | Insurgency SHALL start when aggregate stress exceeds the configured threshold. | CIV-0106 | `crates/social/src/insurgency.rs` | `insurgency::starts_above_threshold` | implemented |
| FR-SOCI-004 | The engine SHALL emit `social.insurgency.started.v1` and `social.insurgency.ended.v1`. | CIV-0106 | `crates/social/src/events.rs` | `social_events::insurgency_lifecycle_events` | implemented |
| FR-SOCI-005 | Health index SHALL be computed from food Joules, clean water, and medical infrastructure. | CIV-0106 | `crates/social/src/health.rs` | `health::computed_from_inputs` | implemented |
| FR-SOCI-006 | A health crisis SHALL emit `social.health.crisis.v1` and reduce labor productivity. | CIV-0106 | `crates/social/src/health.rs` | `health::crisis_emits_event_reduces_labor` | implemented |

---

## AI (FR-AI-*)

Source spec: `docs/specs/CIV-0400-ai.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-AI-001 | AI civilizations SHALL select actions using a utility scoring function over available moves. | CIV-0400 | `crates/ai/src/utility.rs` | `utility::scores_all_moves` | implemented |
| FR-AI-002 | MCTS SHALL be used for multi-step lookahead planning beyond depth 1. | CIV-0400 | `crates/ai/src/mcts.rs` | `mcts::lookahead_depth_gt_1` | implemented |
| FR-AI-003 | Each AI leader SHALL have a personality profile affecting utility weights. | CIV-0400 | `crates/ai/src/personality.rs` | `personality::weights_differ_per_profile` | implemented |
| FR-AI-004 | Personality drift SHALL accumulate stochastically each N ticks. | CIV-0400 | `crates/ai/src/personality.rs` | `personality::drift_accumulates_stochastically` | implemented |
| FR-AI-005 | AI SHALL never exceed a configurable MilliCredit/Joule expenditure per tick (fair-play cap). | CIV-0400 | `crates/ai/src/fair_play.rs` | `fair_play::cap_enforced_per_tick` | implemented |
| FR-AI-006 | AI decision events SHALL be emitted for post-run analysis and replay. | CIV-0400 | `crates/ai/src/events.rs` | `ai_events::decision_emitted` | implemented |
| FR-AI-007 | MCTS computation time SHALL be capped at a fraction of the 100 ms tick budget. | CIV-0400 | `crates/ai/src/mcts.rs` | `mcts::time_capped_within_budget` | implemented |

---

## Protocol (FR-PROT-*)

Source spec: `docs/specs/CIV-0200-protocol.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-PROT-001 | The engine SHALL expose a JSON-RPC 2.0 API over WebSocket. | CIV-0200 | `crates/server/src/jsonrpc.rs` | `protocol::jsonrpc_handshake` | implemented |
| FR-PROT-002 | All events SHALL be emitted as JSON-RPC notifications with a common envelope. | CIV-0200 | `crates/server/src/ws_bridge.rs` | `protocol::event_envelope_valid` | implemented |
| FR-PROT-003 | Event envelope SHALL contain `event_id` (UUIDv7), `event_type`, `session_id`, `tick`, `created_at`, `payload`. | CIV-0200 | `crates/server/src/ws_bridge.rs` | `protocol::envelope_fields_present` | implemented |
| FR-PROT-004 | The server SHALL persist all emitted events to the DB audit log within the same tick. | CIV-0200 | `crates/server/src/audit_log.rs` | `db::events_persisted_same_tick`, `fr_prot_004_persist_event_to_audit_log` | implemented |
| FR-PROT-005 | Client connections SHALL authenticate before receiving any session events. | CIV-0200 | `crates/server/src/authn.rs` | `protocol::unauthenticated_rejected` | implemented |
| FR-PROT-006 | The protocol SHALL support at least 10 concurrent client connections per session. | CIV-0200 | `crates/server/src/ws_bridge.rs` | `protocol::concurrent_clients_10`, `fr_prot_006_app_state_tracks_10_concurrent_sessions`, `fr_prot_006_max_clients_enforced` | implemented |

---

## Save/Load (FR-SAVE-*)

Source spec: `docs/specs/CIV-1000-save-load.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-SAVE-002 | Save SHALL emit `session.saved.v1` or `session.save_failed.v1` events. | CIV-1000 | `crates/server/src/saves.rs` | `fr_save_002_success_emits_session_saved_v1`, `fr_save_002_failure_emits_session_save_failed_v1`, `fr_save_002_roundtrip_serialize_deserialize` | implemented |
| FR-SAVE-003 | Load SHALL restore byte-identical state (determinism guarantee). | CIV-1000 | `crates/engine/src/save_bundle.rs` | `fr_save_003_load_restores_byte_identical_state`, `fr_save_003_archive_bytes_roundtrip_deterministic` | implemented |
| FR-SAVE-005 | Save format SHALL include schema version; old saves SHALL be rejected. | CIV-1000 | `crates/save-db/src/lib.rs` | `fr_save_005_schema_version_stamped_on_fresh_db`, `fr_save_005_schema_version_rejects_old_saves`, `fr_save_005_schema_version_accepts_matching_version` | implemented |

---

## UI/UX (FR-UX-*)

Source spec: `docs/specs/CIV-0300-ui-ux.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-UX-001 | The UI SHALL render the hex map using the `crates/render` crate at 60 fps target. | CIV-0300 | `crates/render/src/hex_map.rs` | `render::hex_map_60fps` | implemented |
| FR-UX-002 | The UI SHALL support RTS-style camera pan, zoom, and unit selection. | CIV-0300 | `crates/render/src/camera.rs` | `render::rts_camera_controls` | implemented |
| FR-UX-003 | A timeline scrubber SHALL display tick history and allow rewind to any stored tick. | CIV-0300 | `crates/render/src/timeline.rs` | `render::timeline_scrubber_rewind` | implemented |
| FR-UX-004 | LOD transitions SHALL be visually seamless within one rendered frame. | CIV-0300 | `crates/render/src/lod.rs` | `render::lod_seamless_transition` | implemented |
| FR-UX-005 | All UI state changes SHALL derive from events; no direct engine state polling. | CIV-0300 | `crates/render/src/state.rs` | `render::state_from_events_only` | implemented |

---

## Assets (FR-ASSET-*)

Source specs: `docs/specs/CIV-0600-2d-assets.md`, `docs/specs/CIV-0601-3d-assets.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-ASSET-001 | All 2D tile sprites SHALL be derived from SVG sources and rasterised at build time. | CIV-0600 | `crates/render/src/atlas.rs` | `asset::svg_rasterised_at_build` | implemented |
| FR-ASSET-002 | The asset pipeline SHALL pack all tile sprites into a single texture atlas per LOD level. | CIV-0600 | `crates/render/src/atlas.rs` | `asset::atlas_packed_per_lod` | implemented |
| FR-ASSET-003 | Atlas build SHALL emit `asset.atlas.built.v1` on success or `asset.generation.failed.v1` on error. | CIV-0600 | `crates/render/src/atlas.rs` | `asset::atlas_build_events` | implemented |
| FR-ASSET-004 | 3D assets SHALL be stored as glTF 2.0 and loaded lazily on demand. | CIV-0601 | `crates/render/src/gltf.rs` | `asset::gltf_lazy_loaded` | implemented |

---

## Modding (FR-MOD-*)

Source spec: `docs/specs/CIV-0700-modding.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-MOD-001 | Mods SHALL be loaded from WASM binaries compiled against the published SDK. | CIV-0700 | `crates/mod-host/src/lib.rs` | `modding::wasm_mod_loaded` | implemented |
| FR-MOD-002 | Mod execution SHALL be sandboxed; mods SHALL NOT access host file system or network. | CIV-0700 | `crates/mod-host/src/capability.rs` | `modding::sandbox_no_host_access` | implemented |
| FR-MOD-003 | Mod state SHALL be persisted and restored as part of save/load (CIV-1000). | CIV-0700 | `crates/mod-host/src/guest_state.rs` | `modding::state_persisted_restored` | implemented |
| FR-MOD-004 | The engine SHALL emit `mod.loaded.v1`, `mod.unloaded.v1`, and `mod.error.v1` events. | CIV-0700 | `crates/mod-host/src/lib.rs` | `modding::lifecycle_events_emitted` | implemented |
| FR-MOD-005 | Mods SHALL be able to register new resource types, policy levers, and event handlers. | CIV-0700 | `crates/mod-host/src/hooks.rs` | `modding::can_register_resources` | implemented |

---

## Audio (FR-AUD-*)

Source spec: `docs/specs/CIV-0800-audio.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-AUD-001 | Background music SHALL be driven by Kira and adapt to game state each tick. | CIV-0800 | `crates/render/src/audio.rs` (`KiraMusic`) | `audio::kira_music_init`, `audio::kira_music_set_tension_clamps` | implemented |
| FR-AUD-002 | Music layers SHALL fade in/out based on tension, prosperity, and war state. | CIV-0800 | `crates/render/src/audio.rs` (`MusicLayers`) | `audio::layers_fade_in_out`, `audio::layers_respond_to_state` | implemented |
| FR-AUD-003 | SFX SHALL be triggered by specific events (war declared, district collapsed, etc.). | CIV-0800 | `crates/render/src/audio.rs` (`SfxTriggerEvent`) | `audio::sfx_triggered_by_events`, `audio::sfx_all_kinds_produce_commands` | implemented |

---

## Session (FR-SESS-*)

Source spec: `docs/specs/CIV-0900-pve-session.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-SESS-001 | The engine SHALL support PvE (human vs AI) sessions. | CIV-0900 | `crates/session/src/lib.rs` | `session::pve_mode_supported` | implemented |
| FR-SESS-002 | Hot-seat multiplayer SHALL allow multiple human players per session. | CIV-0900 | `crates/session/src/lib.rs` | `session::hotseat_multi_human` | implemented |
| FR-SESS-003 | Observer mode SHALL allow read-only session access without influencing simulation. | CIV-0900 | `crates/session/src/lib.rs` | `session::observer_read_only` | implemented |
| FR-SESS-004 | Challenge mode SHALL allow async submission of a civilization seed for scoring. | CIV-0900 | `crates/session/src/lib.rs` | `challenge::async_submission_accepted` | implemented |
| FR-SESS-005 | Session speed SHALL be configurable (1x, 2x, 4x, paused) and emit `session.speed_changed.v1`. | CIV-0900 | `crates/server/src/jsonrpc.rs` (`sim.set_speed`) | `session::speed_change_emits_event` | implemented |
| FR-SESS-006 | Turn boundaries in hot-seat mode SHALL emit `session.turn.start.v1` and `session.turn.end.v1`. | CIV-0900 | `crates/session/src/lib.rs` | `session::turn_events_emitted` | implemented |

---

## Save / Load (FR-SAVE-*)

Source spec: `docs/specs/CIV-1000-save-load.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-SAVE-001 | Quicksave SHALL serialize full world state to a named slot within 500 ms. | CIV-1000 | `crates/server/src/saves.rs` | `save::quicksave_under_500ms` | implemented |
| FR-SAVE-002 | Save SHALL emit `session.saved.v1` on success or `session.save_failed.v1` on error. | CIV-1000 | `crates/server/src/saves.rs` | `save::save_events_emitted` | implemented |
| FR-SAVE-003 | Load SHALL restore world state to byte-identical engine state (determinism guarantee). | CIV-1000 | `crates/engine/src/save_bundle.rs` | `save::load_restores_identical_state` | implemented |
| FR-SAVE-004 | Autosave SHALL trigger every N ticks (configurable, default 100). | CIV-1000 | `crates/server/src/autosave.rs` | `save::autosave_every_n_ticks` | implemented |
| FR-SAVE-005 | Save format SHALL include a schema version; older saves SHALL be rejected with an error (no silent migration). | CIV-1000 | `crates/engine/src/save_bundle.rs` | `save::old_schema_rejected_explicitly` | implemented |

---

## Performance (FR-PERF-*)

Source spec: `docs/specs/CIV-0500-performance.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-PERF-001 | The engine SHALL sustain 100 ms/tick (10 ticks/s) with 8 civilizations and 1,000 hex cells. | CIV-0500 | `crates/engine/tests/fr_fr_perf_001.rs` | `sustained_10_ticks_per_sec` | implemented |
| FR-PERF-002 | Engine heap allocation per tick SHALL not exceed 1 MiB outside of initial world setup. | CIV-0500 | `crates/engine/src/perf.rs` | `perf::heap_under_1mib_per_tick` | implemented |
| FR-PERF-003 | The render crate SHALL maintain 60 fps at 1080p on the reference GPU profile. | CIV-0500 | `crates/render/src/frame.rs` (`FrameBudget`) | `frame::frame_budget_60fps_1080p`, `frame::frame_budget_type_exists` | implemented |
| FR-PERF-004 | DB write throughput SHALL not become a bottleneck for tick latency (async writes). | CIV-0500 | `crates/save-db/src/lib.rs` (`AsyncWriter`) | `async_writer_is_send_sync`, `async_writer_write_tick_enqueues` | implemented |
| FR-PERF-005 | JSON-RPC serialization SHALL complete within 5 ms per event batch. | CIV-0500 | `crates/engine/src/perf.rs` | `perf::serialization_under_5ms` | implemented |

---

*Last updated: 2026-05-25. Maintainer: add new strategic FRs here only; put 3D IDs in [`fr-3d-matrix.md`](fr-3d-matrix.md) and web IDs in [`fr-web-matrix.md`](fr-web-matrix.md). Cross-check `Cargo.toml` and [`docs/IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md) before marking `implemented`.*
