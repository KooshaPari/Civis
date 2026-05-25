# Traceability Matrix

**Status:** Living document — updated as FRs are assigned, implemented, and tested.
**Format:** FR ID | Requirement Summary (SHALL) | Spec Doc | Crate / Source Path | Test Name Pattern | Status

Status values: `planned` | `in_progress` | `implemented`

> **Workspace note (2026-05-23):** The **Crate / Source Path** column below describes the
> *target* layout from CIV-01xx strategic specs. The repo workspace is the 3D extension
> (`civ-engine`, `civ-voxel`, `civ-planet`, …). See
> [`docs/IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md) for the live crate list and
> gap summary. For FR-CIV-VOXEL/BUILD/AGENTS IDs, see
> `docs/development-guide/fr-3d-additions.md` and
> [`docs/traceability/fr-3d-matrix.md`](fr-3d-matrix.md).

**Governance traceability pillars:** `CIV-CORE-1` (simulation core),
`CIV-POLICY-1` (policy / quality gates), `CIV-METRICS-1` (metrics export),
`CIV-EVENT-1` (event taxonomy — see [`EVENT_TAXONOMY.md`](EVENT_TAXONOMY.md)).

---

## Core Engine (FR-CORE-*)

Source spec: `docs/specs/CIV-0001-core-simulation-loop.md`  
**Implemented in:** package `civ-engine` (`crates/engine/`). ECS is `hecs`, not Bevy.

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-CORE-001 | The engine SHALL advance simulation state by exactly one tick per `Engine::step()` invocation. | CIV-0001 | `crates/engine/src/lib.rs`, `crates/engine/src/engine.rs` | `step_advances_tick`, `test_tick_advances` | in_progress |
| FR-CORE-002 | The engine SHALL produce identical output for identical seed and input sequence (determinism). | CIV-0001 | `crates/engine/src/engine.rs` | `determinism_same_seed_same_output`, `test_determinism`, `determinism_holds_with_all_phases_enabled` | in_progress |
| FR-CORE-003 | The engine SHALL use ChaCha20Rng seeded per-run; no global mutable RNG state. | CIV-0001 | `crates/engine/src/lib.rs` (`ChaCha8Rng` today) | *(no dedicated test)* | in_progress |
| FR-CORE-004 | Each tick SHALL complete within 100 ms wall-clock on the reference hardware profile. | CIV-0001 | `crates/engine/src/engine.rs` | `perf::tick_under_100ms` | planned |
| FR-CORE-005 | The engine SHALL emit a BLAKE3 hash of full world state at the end of every tick. | CIV-0001 | *(not present — target `hash_chain.rs`)* | `hash_chain::tick_hash_emitted` | planned |
| FR-CORE-006 | Consecutive tick hashes SHALL form an append-only chain (each hash includes prior hash). | CIV-0001 | *(not present — target `hash_chain.rs`)* | `hash_chain::chain_includes_prior` | planned |
| FR-CORE-007 | The engine SHALL surface a `run.hash.mismatch.v1` event when replayed state diverges. | CIV-0001 | *(not present — target `integrity.rs`)* | `integrity::mismatch_event_emitted` | planned |
| FR-CORE-008 | World state SHALL be modelled as bevy_ecs 0.18.x `World`; no global singletons. | CIV-0001 | `crates/engine/src/engine.rs` (`hecs::World`) | `world::no_global_resources` | in_progress |
| FR-CORE-009 | Hex grid SHALL use `hexx` 0.21.x axial coordinates throughout engine and render crates. | CIV-0001 | `crates/engine/src/engine.rs` (`Position {x,y}` only) | `grid::axial_roundtrip` | planned |
| FR-CORE-010 | All integer quantities SHALL use fixed-point types (`FixedI32\<U16\>`, `i64` KiloJoules, `i64` MilliCredits). | CIV-0001 | `crates/engine/src/lib.rs` (`Fixed` i64 scale) | `numerics::no_float_in_state` | in_progress |

---

## Economy (FR-ECON-*)

> **Crate `crates/economy` is not in the workspace.** Joule-like energy is a single
> `WorldState::energy_budget_joules` field updated in `civ-engine::phase_economy()`.

Source specs: `docs/specs/CIV-0100-economy.md`, `docs/specs/CIV-0107-joule-economy.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-ECON-001 | Each district SHALL produce Joules each tick according to its resource type and capacity. | CIV-0100 | `crates/economy/src/production.rs` *(target)* | `production::district_produces_joules` | planned |
| FR-ECON-002 | Joule consumption SHALL be deducted from district reserves before regional distribution. | CIV-0107 | `crates/economy/src/consumption.rs` *(target)* | `consumption::deducted_before_distribution` | planned |
| FR-ECON-003 | Joule consumption per tick SHALL never be negative (consumption_non_negative invariant). | CIV-0107 | `crates/economy/src/consumption.rs` *(target)* | `consumption::consumption_non_negative` | planned |
| FR-ECON-004 | Surplus Joules SHALL flow to adjacent districts via the distribution graph each tick. | CIV-0100 | `crates/economy/src/distribution.rs` | `distribution::surplus_flows_adjacent` | in_progress |
| FR-ECON-005 | Waste heat SHALL be computed as a percentage of total Joules consumed per tick. | CIV-0107 | `crates/economy/src/waste.rs` | `waste::heat_computed_from_consumption` | planned |
| FR-ECON-006 | GDP SHALL be derived from sum of regional Joule throughput converted at a fixed exchange rate. | CIV-0100 | `crates/economy/src/gdp.rs` | `gdp::sum_of_regional_joules` | planned |
| FR-ECON-007 | Trade agreements SHALL transfer Joules and MilliCredits between civilizations each tick. | CIV-0100 | `crates/economy/src/trade.rs` | `trade::bilateral_transfer_balanced` | planned |
| FR-ECON-008 | A district in Joule deficit for 3 consecutive ticks SHALL emit `economy.district.collapsed.v1`. | CIV-0100 | `crates/economy/src/district.rs` | `district::collapse_after_deficit_ticks` | planned |
| FR-ECON-009 | Subsistence mode SHALL activate when a civilization's total Joule balance drops below threshold. | CIV-0107 | `crates/economy/src/subsistence.rs` | `subsistence::activates_below_threshold` | planned |
| FR-ECON-010 | Treasury balance SHALL be tracked in MilliCredits (`i64`) with no floating-point accumulation. | CIV-0100 | `crates/economy/src/treasury.rs` | `treasury::milliCredits_no_float` | planned |

---

## Level of Detail (FR-LOD-*)

Source spec: `docs/specs/CIV-0101-lod.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-LOD-001 | The engine SHALL support two zoom levels: strategic (region) and operational (district/hex). | CIV-0101 | `crates/engine/src/lod.rs` | `lod::two_levels_defined` | planned |
| FR-LOD-002 | Strategic view SHALL aggregate district data into region summaries each tick. | CIV-0101 | `crates/engine/src/lod.rs` | `lod::strategic_aggregation` | planned |
| FR-LOD-003 | LOD transitions SHALL not alter simulation state, only view projection. | CIV-0101 | `crates/engine/src/lod.rs` | `lod::transition_no_state_mutation` | planned |
| FR-LOD-004 | Operational view SHALL expose individual hex-cell resource and population data. | CIV-0101 | `crates/engine/src/lod.rs` | `lod::operational_hex_data_visible` | planned |

---

## Climate (FR-CLIM-*)

Source spec: `docs/specs/CIV-0102-climate.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-CLIM-001 | Atmospheric CO2 SHALL accumulate each tick based on industrial Joule consumption. | CIV-0102 | `crates/climate/src/co2.rs` | `co2::accumulates_with_consumption` | planned |
| FR-CLIM-002 | Global mean temperature SHALL be derived from CO2 concentration via parameterised formula. | CIV-0102 | `crates/climate/src/temperature.rs` | `temperature::derived_from_co2` | planned |
| FR-CLIM-003 | The engine SHALL emit `climate.threshold.crossed.v1` when temperature crosses a defined level. | CIV-0102 | `crates/climate/src/events.rs` | `climate_events::threshold_event_emitted` | planned |
| FR-CLIM-004 | Climate damage SHALL reduce district Joule production capacity when temperature exceeds threshold. | CIV-0102 | `crates/climate/src/damage.rs` | `damage::reduces_production_above_threshold` | planned |
| FR-CLIM-005 | The engine SHALL model at least one tipping-point cascade (e.g. ice-albedo) above critical temperature. | CIV-0102 | `crates/climate/src/tipping.rs` | `tipping::cascade_triggered` | planned |
| FR-CLIM-006 | Civilizations SHALL be able to invest MilliCredits into adaptation to reduce climate damage. | CIV-0102 | `crates/climate/src/adaptation.rs` | `adaptation::investment_reduces_damage` | planned |

---

## Institutions (FR-INST-*)

Source spec: `docs/specs/CIV-0103-institutions.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-INST-001 | Each civilization SHALL have an institutional type (democracy, autocracy, technocracy, etc.). | CIV-0103 | `crates/institutions/src/governance.rs` | `governance::type_assigned_at_init` | planned |
| FR-INST-002 | Institutional capture score SHALL accumulate each tick based on resource concentration. | CIV-0103 | `crates/institutions/src/capture.rs` | `capture::accumulates_with_concentration` | planned |
| FR-INST-003 | The engine SHALL emit `institution.capture.threshold.v1` when capture crosses 0.75. | CIV-0103 | `crates/institutions/src/events.rs` | `inst_events::capture_threshold_event` | planned |
| FR-INST-004 | Institutional collapse SHALL trigger a governance type transition. | CIV-0103 | `crates/institutions/src/collapse.rs` | `collapse::triggers_type_transition` | planned |
| FR-INST-005 | Institution time-series data SHALL be stored in the metrics DB for post-run analysis. | CIV-0103 | `crates/db/src/institution_series.rs` | `db::institution_series_stored` | planned |
| FR-INST-006 | Citizen lifecycle (birth, migration, death) SHALL be driven by institutional and economic state. | CIV-0103 | `crates/citizens/src/lifecycle.rs` | `lifecycle::driven_by_inst_economy` | planned |

---

## Theorem / Invariants (FR-THRY-*)

Source spec: `docs/specs/CIV-0104-theorem.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-THRY-001 | Total Joule energy in a closed system SHALL be conserved each tick (production - consumption - waste = 0). | CIV-0104 | `crates/engine/src/invariants.rs` | `invariants::joule_conservation` | planned |
| FR-THRY-002 | Total MilliCredit supply SHALL remain constant absent explicit treasury mint/burn operations. | CIV-0104 | `crates/engine/src/invariants.rs` | `invariants::credit_supply_conserved` | planned |
| FR-THRY-003 | Population delta per tick SHALL equal births minus deaths minus emigration plus immigration. | CIV-0104 | `crates/engine/src/invariants.rs` | `invariants::population_delta_balanced` | planned |
| FR-THRY-004 | The invariant checker SHALL run every tick and panic in debug builds on violation. | CIV-0104 | `crates/engine/src/invariants.rs` | `invariants::checker_panics_on_violation` | planned |

---

## Diplomacy (FR-DIPL-*)

Source spec: `docs/specs/CIV-0105-war-diplomacy.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-DIPL-001 | Civilizations SHALL be able to declare war, producing `diplomacy.war.declared.v1`. | CIV-0105 | `crates/diplomacy/src/war.rs` | `war::declare_emits_event` | planned |
| FR-DIPL-002 | Peace SHALL be negotiated via signed treaty, producing `diplomacy.peace.signed.v1`. | CIV-0105 | `crates/diplomacy/src/peace.rs` | `peace::signed_emits_event` | planned |
| FR-DIPL-003 | Treaties SHALL encode terms (trade ratios, non-aggression, alliance) as structured data. | CIV-0105 | `crates/diplomacy/src/treaty.rs` | `treaty::terms_structured` | planned |
| FR-DIPL-004 | Treaty breach SHALL emit `diplomacy.treaty.broken.v1` and apply reputation penalty. | CIV-0105 | `crates/diplomacy/src/treaty.rs` | `treaty::breach_emits_event_and_penalty` | planned |
| FR-DIPL-005 | Espionage operations SHALL have a configurable detection probability per tick. | CIV-0105 | `crates/diplomacy/src/espionage.rs` | `espionage::detection_probability_applied` | planned |
| FR-DIPL-006 | Detected espionage SHALL emit `diplomacy.espionage.detected.v1`. | CIV-0105 | `crates/diplomacy/src/espionage.rs` | `espionage::detected_emits_event` | planned |
| FR-DIPL-007 | Shadow networks SHALL model covert influence as a hidden resource accumulating per tick. | CIV-0105 | `crates/diplomacy/src/shadow.rs` | `shadow::influence_accumulates` | planned |

---

## Social (FR-SOCI-*)

Source spec: `docs/specs/CIV-0106-social.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-SOCI-001 | Ideological alignment SHALL be tracked per-citizen cohort as a continuous score. | CIV-0106 | `crates/social/src/ideology.rs` | `ideology::per_cohort_continuous` | planned |
| FR-SOCI-002 | Citizen stress SHALL accumulate when Joule access falls below subsistence level. | CIV-0106 | `crates/social/src/stress.rs` | `stress::accumulates_below_subsistence` | planned |
| FR-SOCI-003 | Insurgency SHALL start when aggregate stress exceeds the configured threshold. | CIV-0106 | `crates/social/src/insurgency.rs` | `insurgency::starts_above_threshold` | planned |
| FR-SOCI-004 | The engine SHALL emit `social.insurgency.started.v1` and `social.insurgency.ended.v1`. | CIV-0106 | `crates/social/src/events.rs` | `social_events::insurgency_lifecycle_events` | planned |
| FR-SOCI-005 | Health index SHALL be computed from food Joules, clean water, and medical infrastructure. | CIV-0106 | `crates/social/src/health.rs` | `health::computed_from_inputs` | planned |
| FR-SOCI-006 | A health crisis SHALL emit `social.health.crisis.v1` and reduce labor productivity. | CIV-0106 | `crates/social/src/health.rs` | `health::crisis_emits_event_reduces_labor` | planned |

---

## AI (FR-AI-*)

Source spec: `docs/specs/CIV-0400-ai.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-AI-001 | AI civilizations SHALL select actions using a utility scoring function over available moves. | CIV-0400 | `crates/ai/src/utility.rs` | `utility::scores_all_moves` | planned |
| FR-AI-002 | MCTS SHALL be used for multi-step lookahead planning beyond depth 1. | CIV-0400 | `crates/ai/src/mcts.rs` | `mcts::lookahead_depth_gt_1` | planned |
| FR-AI-003 | Each AI leader SHALL have a personality profile affecting utility weights. | CIV-0400 | `crates/ai/src/personality.rs` | `personality::weights_differ_per_profile` | planned |
| FR-AI-004 | Personality drift SHALL accumulate stochastically each N ticks. | CIV-0400 | `crates/ai/src/personality.rs` | `personality::drift_accumulates_stochastically` | planned |
| FR-AI-005 | AI SHALL never exceed a configurable MilliCredit/Joule expenditure per tick (fair-play cap). | CIV-0400 | `crates/ai/src/fair_play.rs` | `fair_play::cap_enforced_per_tick` | planned |
| FR-AI-006 | AI decision events SHALL be emitted for post-run analysis and replay. | CIV-0400 | `crates/ai/src/events.rs` | `ai_events::decision_emitted` | planned |
| FR-AI-007 | MCTS computation time SHALL be capped at a fraction of the 100 ms tick budget. | CIV-0400 | `crates/ai/src/mcts.rs` | `mcts::time_capped_within_budget` | planned |

---

## Protocol (FR-PROT-*)

Source spec: `docs/specs/CIV-0200-protocol.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-PROT-001 | The engine SHALL expose a JSON-RPC 2.0 API over WebSocket. | CIV-0200 | `crates/protocol/src/server.rs` | `protocol::jsonrpc_handshake` | planned |
| FR-PROT-002 | All events SHALL be emitted as JSON-RPC notifications with a common envelope. | CIV-0200 | `crates/protocol/src/events.rs` | `protocol::event_envelope_valid` | planned |
| FR-PROT-003 | Event envelope SHALL contain `event_id` (UUIDv7), `event_type`, `session_id`, `tick`, `created_at`, `payload`. | CIV-0200 | `crates/protocol/src/events.rs` | `protocol::envelope_fields_present` | planned |
| FR-PROT-004 | The server SHALL persist all emitted events to the DB audit log within the same tick. | CIV-0200 | `crates/db/src/audit_log.rs` | `db::events_persisted_same_tick` | planned |
| FR-PROT-005 | Client connections SHALL authenticate before receiving any session events. | CIV-0200 | `crates/protocol/src/auth.rs` | `protocol::unauthenticated_rejected` | planned |
| FR-PROT-006 | The protocol SHALL support at least 10 concurrent client connections per session. | CIV-0200 | `crates/protocol/src/server.rs` | `protocol::concurrent_clients_10` | planned |

---

## UI/UX (FR-UX-*)

Source spec: `docs/specs/CIV-0300-ui-ux.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-UX-001 | The UI SHALL render the hex map using the `crates/render` crate at 60 fps target. | CIV-0300 | `crates/render/src/hex_map.rs` | `render::hex_map_60fps` | planned |
| FR-UX-002 | The UI SHALL support RTS-style camera pan, zoom, and unit selection. | CIV-0300 | `crates/render/src/camera.rs` | `render::rts_camera_controls` | planned |
| FR-UX-003 | A timeline scrubber SHALL display tick history and allow rewind to any stored tick. | CIV-0300 | `crates/render/src/timeline.rs` | `render::timeline_scrubber_rewind` | planned |
| FR-UX-004 | LOD transitions SHALL be visually seamless within one rendered frame. | CIV-0300 | `crates/render/src/lod.rs` | `render::lod_seamless_transition` | planned |
| FR-UX-005 | All UI state changes SHALL derive from events; no direct engine state polling. | CIV-0300 | `crates/render/src/state.rs` | `render::state_from_events_only` | planned |

---

## Assets (FR-ASSET-*)

Source specs: `docs/specs/CIV-0600-2d-assets.md`, `docs/specs/CIV-0601-3d-assets.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-ASSET-001 | All 2D tile sprites SHALL be derived from SVG sources and rasterised at build time. | CIV-0600 | `crates/render/src/atlas.rs` | `asset::svg_rasterised_at_build` | planned |
| FR-ASSET-002 | The asset pipeline SHALL pack all tile sprites into a single texture atlas per LOD level. | CIV-0600 | `crates/render/src/atlas.rs` | `asset::atlas_packed_per_lod` | planned |
| FR-ASSET-003 | Atlas build SHALL emit `asset.atlas.built.v1` on success or `asset.generation.failed.v1` on error. | CIV-0600 | `crates/render/src/atlas.rs` | `asset::atlas_build_events` | planned |
| FR-ASSET-004 | 3D assets SHALL be stored as glTF 2.0 and loaded lazily on demand. | CIV-0601 | `crates/render/src/gltf.rs` | `asset::gltf_lazy_loaded` | planned |

---

## Modding (FR-MOD-*)

Source spec: `docs/specs/CIV-0700-modding.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-MOD-001 | Mods SHALL be loaded from WASM binaries compiled against the published SDK. | CIV-0700 | `crates/engine/src/mod_loader.rs` | `modding::wasm_mod_loaded` | planned |
| FR-MOD-002 | Mod execution SHALL be sandboxed; mods SHALL NOT access host file system or network. | CIV-0700 | `crates/engine/src/mod_sandbox.rs` | `modding::sandbox_no_host_access` | planned |
| FR-MOD-003 | Mod state SHALL be persisted and restored as part of save/load (CIV-1000). | CIV-0700 | `crates/engine/src/mod_state.rs` | `modding::state_persisted_restored` | planned |
| FR-MOD-004 | The engine SHALL emit `mod.loaded.v1`, `mod.unloaded.v1`, and `mod.error.v1` events. | CIV-0700 | `crates/engine/src/mod_events.rs` | `modding::lifecycle_events_emitted` | planned |
| FR-MOD-005 | Mods SHALL be able to register new resource types, policy levers, and event handlers. | CIV-0700 | `crates/engine/src/mod_registry.rs` | `modding::can_register_resources` | planned |

---

## Audio (FR-AUD-*)

Source spec: `docs/specs/CIV-0800-audio.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-AUD-001 | Background music SHALL be driven by Kira and adapt to game state each tick. | CIV-0800 | `crates/render/src/audio.rs` | `audio::kira_initialized` | planned |
| FR-AUD-002 | Music layers SHALL fade in/out based on tension, prosperity, and war state. | CIV-0800 | `crates/render/src/audio.rs` | `audio::layers_respond_to_state` | planned |
| FR-AUD-003 | SFX SHALL be triggered by specific events (war declared, district collapsed, etc.). | CIV-0800 | `crates/render/src/audio.rs` | `audio::sfx_triggered_by_events` | planned |

---

## Session (FR-SESS-*)

Source spec: `docs/specs/CIV-0900-pve-session.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-SESS-001 | The engine SHALL support PvE (human vs AI) sessions. | CIV-0900 | `crates/engine/src/session.rs` | `session::pve_mode_supported` | planned |
| FR-SESS-002 | Hot-seat multiplayer SHALL allow multiple human players per session. | CIV-0900 | `crates/engine/src/session.rs` | `session::hotseat_multi_human` | planned |
| FR-SESS-003 | Observer mode SHALL allow read-only session access without influencing simulation. | CIV-0900 | `crates/engine/src/session.rs` | `session::observer_read_only` | planned |
| FR-SESS-004 | Challenge mode SHALL allow async submission of a civilization seed for scoring. | CIV-0900 | `crates/engine/src/challenge.rs` | `challenge::async_submission_accepted` | planned |
| FR-SESS-005 | Session speed SHALL be configurable (1x, 2x, 4x, paused) and emit `session.speed_changed.v1`. | CIV-0900 | `crates/engine/src/session.rs` | `session::speed_change_emits_event` | planned |
| FR-SESS-006 | Turn boundaries in hot-seat mode SHALL emit `session.turn.start.v1` and `session.turn.end.v1`. | CIV-0900 | `crates/engine/src/session.rs` | `session::turn_events_emitted` | planned |

---

## Save / Load (FR-SAVE-*)

Source spec: `docs/specs/CIV-1000-save-load.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-SAVE-001 | Quicksave SHALL serialize full world state to a named slot within 500 ms. | CIV-1000 | `crates/db/src/save.rs` | `save::quicksave_under_500ms` | planned |
| FR-SAVE-002 | Save SHALL emit `session.saved.v1` on success or `session.save_failed.v1` on error. | CIV-1000 | `crates/db/src/save.rs` | `save::save_events_emitted` | planned |
| FR-SAVE-003 | Load SHALL restore world state to byte-identical engine state (determinism guarantee). | CIV-1000 | `crates/db/src/load.rs` | `save::load_restores_identical_state` | planned |
| FR-SAVE-004 | Autosave SHALL trigger every N ticks (configurable, default 100). | CIV-1000 | `crates/db/src/autosave.rs` | `save::autosave_every_n_ticks` | planned |
| FR-SAVE-005 | Save format SHALL include a schema version; older saves SHALL be rejected with an error (no silent migration). | CIV-1000 | `crates/db/src/schema.rs` | `save::old_schema_rejected_explicitly` | planned |

---

## Performance (FR-PERF-*)

Source spec: `docs/specs/CIV-0500-performance.md`

| FR ID | Requirement Summary | Spec Doc | Crate / Source Path | Test Name Pattern | Status |
|---|---|---|---|---|---|
| FR-PERF-001 | The engine SHALL sustain 100 ms/tick (10 ticks/s) with 8 civilizations and 1,000 hex cells. | CIV-0500 | `crates/engine/src/tick.rs` | `perf::sustained_10_ticks_per_sec` | planned |
| FR-PERF-002 | Engine heap allocation per tick SHALL not exceed 1 MiB outside of initial world setup. | CIV-0500 | `crates/engine/src/tick.rs` | `perf::heap_under_1mib_per_tick` | planned |
| FR-PERF-003 | The render crate SHALL maintain 60 fps at 1080p on the reference GPU profile. | CIV-0500 | `crates/render/src/frame.rs` | `perf::render_60fps_1080p` | planned |
| FR-PERF-004 | DB write throughput SHALL not become a bottleneck for tick latency (async writes). | CIV-0500 | `crates/db/src/writer.rs` | `perf::db_writes_async_nonblocking` | planned |
| FR-PERF-005 | JSON-RPC serialization SHALL complete within 5 ms per event batch. | CIV-0500 | `crates/protocol/src/serializer.rs` | `perf::serialization_under_5ms` | planned |

---

*Last updated: 2026-05-23. Maintainer: add new FRs as specs are finalized; update Status as tests are written and pass CI. Cross-check workspace members in root `Cargo.toml` and `docs/IMPLEMENTATION_STATUS.md` before marking `implemented`.*
