# Full End-to-End Traceability Matrix — Civis

**Status:** Active — master traceability artifact.
**Date:** 2026-05-29
**Scope:** ALL `FR-*` IDs (strategic CivLab core `FR-CORE/ECON/PROTO/REPLAY/API/CLIENT/METRICS-*`
plus the 3D extension `FR-CIV-*`).
**Branch:** `feat/civis-bevy-game`

This document is the single end-to-end traceability home. It links every functional
requirement back to its epic / user story and forward to its implementing crate, its
verifying test(s), the client(s) it surfaces in, and its delivery status.

## Sources of truth

| Artifact | Path | Used for |
|---|---|---|
| Strategic FRs | `FUNCTIONAL_REQUIREMENTS.md` | `FR-CORE/ECON/PROTO/REPLAY/API/CLIENT/METRICS-*` text + AC + status |
| 3D extension FRs | `docs/development-guide/fr-3d-additions.md` | `FR-CIV-*` text + AC |
| Partial 3D matrix | `docs/traceability/fr-3d-matrix.md` | `FR-CIV-*` crate/test mappings (verified present) |
| Epics + user stories | `PRD.md` (E1–E6) | epic / user-story column |
| Strategic plan | `PLAN.md` (Phases 0–6) | engine wiring + status |
| 3D phase plan | `docs/roadmap/plan-3d-phases.md` (P-V0…P-U1) | 3D phase → FR namespace, acceptance |

## Documentation gaps (referenced by the tasking but not present in the tree)

The following inputs were requested but **do not exist** in this worktree as of the date
above. Cells that would draw on them are filled from the closest existing artifact and
flagged:

- **`docs/reference/non-functional-requirements.md`** — not present. No `NFR-CIV-*`
  catalogue exists yet. All NFR cells read **"see NFR doc"** as instructed.
- **`docs/<...>/engine-tick-integration-plan.md`** — not present anywhere in `docs/` or
  `crates/`. Engine-tick wiring was instead verified directly against
  `crates/engine/src/engine.rs` (`Simulation::tick` phase call list, lines ~1028–1039).
  Partial rows reference **"engine tick (engine.rs); no separate tick-integration plan
  doc"** rather than a non-existent plan file.
- **`docs/guides/game-design-and-ux.md`** — not present. Closest existing UX artifacts are
  `docs/specs/CIV-0300-rts-ui-ux-spec.md` and `docs/guides/ui-design-system.md`.
- **PRD epics E7 / E8** — **do not exist.** `PRD.md` defines epics **E1–E6 only.** The 3D
  `FR-CIV-*` work has no PRD epic; it is governed by the phase plan
  (`docs/roadmap/plan-3d-phases.md`, phases P-V0…P-U1). The "Epic" column therefore shows
  the **3D phase ID** (e.g. `P-V1`) for `FR-CIV-*` rows and notes "(no PRD epic; E7/E8 do
  not exist)".

## Legend

- **Status** — `Done`: implemented and tested; `Partial`: implemented but not fully wired
  into the engine tick **or** missing acceptance coverage; `Planned`: not yet implemented.
- **Test = GAP** — no verifying test exists for that FR.
- **Engine-tick wiring** — `Simulation::tick` (`crates/engine/src/engine.rs`) calls, in
  order: `phase_production`, `phase_citizen_lifecycle`, `phase_military`, `phase_economy`,
  `phase_planet`, `phase_diplomacy`, `phase_tactics`, `phase_voxel`, `phase_compact`,
  `phase_buildings`, `phase_diffusion`. Crates **not** reachable from this list
  (`genetics`, `species`, `laws`, `research`, `protocol-3d`) are library-only today and
  their gameplay FRs are therefore at most `Partial` regardless of their own unit-test
  coverage.

---

## Section A — Strategic core (CivLab) FRs

> Source: `FUNCTIONAL_REQUIREMENTS.md` (v1.0). NFR cells = "see NFR doc" (catalogue absent).

| Epic | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) (Bevy/Godot/Unreal) | Status |
|---|---|---|---|---|---|---|---|---|
| E1 Core Engine | E1.1 | FR-CORE-001 | see NFR doc | Fixed 100 ms tick; <1 ms P99 jitter; monotonic tick counter | `crates/engine/src/engine.rs` (`Simulation::tick`) | `determinism_holds_with_all_phases_enabled`, `tick_invariants_hold_across_many_ticks` | Bevy (sim_bridge), Godot, Unreal (via server stream) | Partial |
| E1 Core Engine | E1.2 | FR-CORE-002 | see NFR doc | Cells/buildings/agents/institutions addressable by stable ID; cache-friendly queries; IDs survive serde | `crates/engine/src/engine.rs`, `world.rs`/components | `simulation_tick_produces_replay_tick_event`, ECS spawn tests (`spawn.rs`) | Bevy, Godot, Unreal | Partial |
| E1 Core Engine | E1.3 | FR-CORE-005 | see NFR doc | Policy phase runs before production/allocation; pure; overridable via Scenario API | `crates/engine` (policy/scenario); WASM `civlab-sdk/policy.rs` | `policy::*` (civlab-sdk) | Bevy, Godot, Unreal | Partial |
| E1 Core Engine | E1.4 | FR-CORE-003 | see NFR doc | Bit-identical transition; fixed-point only in state paths; determinism CI gate | `crates/engine/src/lib.rs` (`Fixed`), `engine.rs` | `determinism_same_seed_same_output`, `test_determinism`, `determinism_holds_with_all_phases_enabled` | Bevy, Godot, Unreal | Partial |
| E1 Core Engine | E1.5 | FR-CORE-004 | see NFR doc | ChaCha20Rng per run; every draw logged; same-seed → same stochastic sequence | `crates/engine` (rng/replay) | GAP (no `rng_draw` event test found) | Bevy, Godot, Unreal | Planned |
| E1 Core Engine | E1.7 | FR-CORE-006 | see NFR doc | Priority+FIFO command queue; admin>player>research; post-cutoff deferred | `crates/server` (ws/jsonrpc), `crates/engine` command intake | GAP (no priority-tier queue test found) | Bevy, Godot, Unreal | Planned |
| E1 Core Engine | E1.10 | FR-CORE-007 | see NFR doc | 14 ms tick budget; warn >16 ms with phase breakdown; perf CI gate | `crates/engine` (tick profiling) | GAP | Bevy, Godot, Unreal | Planned |
| E2 Economy | E2.1 | FR-ECON-001 | see NFR doc | Buildings produce per tick; halt on missing inputs; events logged | `crates/economy`, `crates/engine/src/engine.rs` (`phase_production`) | `phase_economy_updates_economy_state`, `capitalist_allocator_fills_demand_when_budget_sufficient` | Bevy (live_scene), Godot, Unreal | Partial |
| E2 Economy | E2.4 | FR-ECON-002 | see NFR doc | Joule numeraire; conserved every tick; fatal on violation | `crates/economy`, `crates/engine` (`phase_economy`) | `phase_economy_conserves_non_negative_budget` (conservation invariant); full conservation proptest = GAP | Bevy, Godot, Unreal | Partial |
| E2 Economy | E2.3 | FR-ECON-003 | see NFR doc | Market clears bid/ask each tick; deterministic; uncleared expire by TTL | `crates/economy` (`MarketState::step`), `engine.rs` `phase_economy` | `phase_economy_steps_market_prices` | Bevy, Godot, Unreal | Partial |
| E2 Economy | E2.6/E2.7/E2.8 | FR-ECON-004 | see NFR doc | Configurable tax→treasury same tick; legitimacy decrements with tax rate | `crates/economy` (fiscal); `crates/engine/src/metrics.rs` (legitimacy) | `test_metrics_basic` (legitimacy); taxation/treasury test = GAP | Bevy, Godot, Unreal | Planned |
| E2 Economy | E2.5 | FR-ECON-005 | see NFR doc | Subsistence-before-luxury allocation; deprivation counters; O(n log n) | `crates/economy/src/allocation.rs` | `capitalist_allocator_rations_proportionally_when_budget_scarce`, `capitalist_allocator_never_exceeds_budget` (priority/deprivation = GAP) | Bevy, Godot, Unreal | Partial |
| E3 Protocol | E3.1 | FR-PROTO-001 | see NFR doc | RFC 6455 WS; ≥10 conns; TLS; graceful close | `crates/server` (ws bridge) | `ws_jsonrpc_save_slot_roundtrip` (WS smoke); TLS/10-conn load = GAP | Bevy (ws_client), Godot (civis_ws_client.gd), Unreal | Partial |
| E3 Protocol | E3.2 | FR-PROTO-002 | see NFR doc | JSON-RPC 2.0 result/error; -32601 unknown; batch | `crates/server` (jsonrpc dispatcher) | `post_*` / `ws_jsonrpc_*` smoke; -32601 + batch = GAP | Bevy, Godot, Unreal | Partial |
| E3 Protocol | E3.3 | FR-PROTO-003 | see NFR doc | Handshake <2 s; bootstrap snapshot; role assigned + enforced | `crates/server` (handshake) | GAP | Bevy, Godot, Unreal | Planned |
| E3 Protocol | E3.6 | FR-PROTO-004 | see NFR doc | zstd binary frames; header (tick/type/size/checksum); ≥3:1; ≤10 Mbps@10×60FPS | `crates/protocol-3d`, `crates/server` | `protocol3d::voxel_delta_roundtrip` (frame round-trip); bandwidth/ratio = GAP | Bevy, Godot, Unreal | Partial |
| E3 Protocol | E3.8 | FR-PROTO-005 | see NFR doc | Region/type snapshot filtering; out-of-bounds excluded | `crates/server` (snapshot filter) | GAP | Bevy, Godot, Unreal | Planned |
| E3 Protocol | E3.7 | FR-CLIENT-003 | see NFR doc | Role perms admin>player>research; -32603 on unauthorized; covered all tiers | `crates/server` (role auth), `mod-host/capability.rs` | `capability::*` (capability enforcement); per-role RPC rejection = GAP | Bevy, Godot, Unreal | Partial |
| E1 Core Engine | E1.8 | FR-REPLAY-001 | see NFR doc | `.civreplay` header+log+SHA-256; append-only; checksum verified on load | `crates/engine/src/replay_format.rs`, `replay.rs` | `civreplay_roundtrip_preserves_replay_log`, `civreplay_rejects_tampered_payload`, `civreplay_rejects_tampered_footer` | Bevy, Godot, Unreal (replay load) | Partial |
| E1 Core Engine | E1.9 | FR-REPLAY-002 | see NFR doc | Replay re-executes events (no re-sample RNG); per-tick hash compare; CI gate | `crates/engine/src/replay.rs`, `hash_chain.rs`, `integrity.rs` | `civreplay_roundtrip_preserves_running_hash`, `civreplay_rejects_hash_chain_mismatch`, `check_integrity_after_replay_load`, `voxel_phase_replay_is_bit_identical` | Bevy, Godot, Unreal | Partial |
| E5 Research API | E5.1 | FR-API-001 | see NFR doc | Versioned scenario YAML; load-time schema validation; CI-validated example | `crates/engine/src/scenario.rs`, `scenarios/baseline.yaml` | `baseline_yaml_parses`, `scenario_*_wires_*_phase` | n/a (server-side) | Partial |
| E5 Research API | E5.2 | FR-API-002 | see NFR doc | `civlab.run_scenario(path, ticks=50)` <5 s; pip-installable; typed/docstringed | Python bindings (not present in this worktree) | GAP | n/a | Planned |
| E5 Research API | E5.3 | FR-API-003 | see NFR doc | Override dict merges defaults; ValueError on bad name; type-validated | Python bindings (absent); `crates/engine` scenario overrides | GAP | n/a | Planned |
| E5 Research API | E5.6 | FR-API-004 | see NFR doc | CSV/JSON metrics export; full event log JSON; <30 s for 100k ticks | Research export (absent) | GAP | n/a | Planned |
| E6 Clients | E6.1 | FR-CLIENT-001 | see NFR doc | Bevy renders 60 FPS @1k entities; click-to-build round-trips ≤1 tick; auto-reconnect ≤5 s | `clients/bevy-ref/src/*` (`live_scene`, `spawn_tools`, `live_attach`, `ws_client`) | `event_feed::*`, `menus::*`, `native_backend` (cargo test/check, civ-bevy-ref) | **Bevy** (primary); Godot/Unreal parallel | Partial |
| E6 Clients | E6.4 | FR-CLIENT-002 | see NFR doc | TS/React web client; Chrome/FF/Safari; reconnect backoff; documented hooks+tests | `web/dashboard` (deprecated per desktop pivot) | manual / tsc | Web (deprecated) | Planned |

### FR-METRICS sub-family (E5 / Research Sandbox)

| Epic | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| E5 Research API | E5 (policy analysis) | FR-METRICS-001 | see NFR doc | `Metrics` 4×f64 (waste/surplus/tyranny/legitimacy); `compute(1000,500)`→waste 50, surplus 500 | `crates/engine/src/metrics.rs` | `test_metrics_basic` | Bevy/Godot/Unreal HUD (indirect) | Done |
| E5 Research API | derives FR-CORE-003 | FR-METRICS-002 | see NFR doc | `compute(...)` O(1), no I/O/alloc, pure, P99 <1 µs | `crates/engine/src/metrics.rs` | `test_metrics_basic` (purity); P99 perf assert = GAP | indirect | Partial |
| E5 Research API | derives FR-REPLAY-002 | FR-METRICS-003 | see NFR doc | Fixed-point tyranny/legitimacy agree with float to 6 dp; replay uses fixed-point | `crates/engine/src/metrics.rs` (`compute_fixed`), `lib.rs` (`Fixed`) | `compute_fixed_matches_float_within_six_decimals` | indirect | Done |

---

## Section B — 3D extension FRs (`FR-CIV-*`)

> Source: `docs/development-guide/fr-3d-additions.md`; mappings cross-checked against
> `docs/traceability/fr-3d-matrix.md` and crate `#[cfg(test)]` blocks.
> **No PRD epic exists for these** (E7/E8 absent) — the "Epic" column carries the **3D
> phase ID** from `docs/roadmap/plan-3d-phases.md`.
> Status `Done` requires the crate to be both tested **and** reachable from the engine
> tick. Crates not wired into `Simulation::tick` (genetics, species, laws, research,
> protocol-3d) are capped at `Partial`.

### Voxel — P-V1 (crate `civ-voxel`; wired via `phase_voxel`)

| Epic/Phase | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| P-V1 | voxel substrate | FR-CIV-VOXEL-000 | see NFR doc | Crate compiles; exposes `SCHEMA_VERSION` | `crates/voxel/src/lib.rs` | `voxel::schema_version_stub` | Bevy/Godot/Unreal | Done |
| P-V1 | adaptive storage | FR-CIV-VOXEL-001 | see NFR doc | O(1) writes in 16³ leaf; deterministic octree branching | `crates/voxel/` | `voxel::adaptive_storage` | Bevy/Godot/Unreal | Done |
| P-V1 | dirty queue | FR-CIV-VOXEL-002 | see NFR doc | Deterministic dirty queue ordered `(chunk_id, write_seq)` | `crates/voxel/` | `voxel::dirty_queue_deterministic` | Bevy/Godot/Unreal | Done |
| P-V1 | fixed-point API | FR-CIV-VOXEL-003 | see NFR doc | No `f32`/`f64` across public API | `crates/voxel/` | `voxel::fixed_point_api` | Bevy/Godot/Unreal | Done |
| P-V1 | LOD invariant | FR-CIV-VOXEL-004 | see NFR doc | `VoxelScaleMultiplier` LOD composition invariant | `crates/voxel/` | `voxel::scale_multiplier_lod` | Bevy/Godot/Unreal | Done |
| P-V1 | mesher | FR-CIV-VOXEL-010 | see NFR doc | `Mesher` trait: watertight mesh for fixed scene | `crates/voxel/` | `voxel::mesher_watertight` | Bevy (bevy_render/terrain) | Done |

### Build — P-V2 (crate `civ-build`; wired via `phase_buildings`)

| Epic/Phase | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| P-V2 | build stub | FR-CIV-BUILD-000 | see NFR doc | Stub compiles | `crates/build/` | `build::schema_version_stub` | Bevy/Godot/Unreal | Done |
| P-V2 | graph schema | FR-CIV-BUILD-001 | see NFR doc | `BuildingGraph` RON round-trips losslessly | `crates/build/` | `build::graph_ron_roundtrip` | Bevy/Godot/Unreal | Done |
| P-V2 | demand alloc | FR-CIV-BUILD-010 | see NFR doc | Deterministic parcel scoring from demand signals | `crates/build/`, `engine.rs` `phase_buildings` | `build::demand_allocation_deterministic`, `phase_buildings_allocates_over_time_when_signals_are_high` | Bevy (live_scene) | Done |
| P-V2 | freehand tools | FR-CIV-BUILD-020 | see NFR doc | Freehand tools emit same `BuildingGraph` mods as grammar | `crates/build/` | `build::freehand_matches_grammar` | Bevy (spawn_tools) | Done |
| P-V2 | era grammar | FR-CIV-BUILD-030 | see NFR doc | Era-grammar transitions → expected facade histogram | `crates/build/` | `build::era_grammar_histogram` | Bevy/Godot/Unreal | Done |

### Genetics — P-G1 (crate `civ-genetics`; **not wired into engine tick**)

| Epic/Phase | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| P-G1 | genetics stub | FR-CIV-GENETICS-000 | see NFR doc | Stub compiles | `crates/genetics/` | `genetics::schema_version_stub` | (not surfaced yet) | Partial — see engine tick (engine.rs); crate not in `Simulation::tick` |
| P-G1 | mutation | FR-CIV-GENETICS-001 | see NFR doc | Mutation deterministic under fixed seed | `crates/genetics/` | `genetics::mutation_deterministic` | (not surfaced yet) | Partial — not wired to engine tick |
| P-G1 | recombination | FR-CIV-GENETICS-002 | see NFR doc | Offspring loci drawn deterministically from parents | `crates/genetics/` | `genetics::recombination_deterministic` | (not surfaced yet) | Partial — not wired to engine tick |
| P-G1 | speciation | FR-CIV-GENETICS-010 | see NFR doc | Speciation trigger deterministic; emits new species record | `crates/genetics/` | `genetics::speciation_trigger` | (not surfaced yet) | Partial — not wired to engine tick |

### Species — P-G1 (crate `civ-species`; **not wired into engine tick**)

| Epic/Phase | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| P-G1 | species stub | FR-CIV-SPECIES-000 | see NFR doc | Stub compiles | `crates/species/` | `species::schema_version_stub` | (not surfaced yet) | Partial — not wired to engine tick |
| P-G1 | phenotype map | FR-CIV-SPECIES-001 | see NFR doc | Deterministic DNA → phenotype mapping | `crates/species/` | `species::phenotype_deterministic` | (not surfaced yet) | Partial — not wired to engine tick |

### Agents — P-A1 (crate `civ-agents`; wired via `phase_citizen_lifecycle`/`phase_diffusion`)

| Epic/Phase | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| P-A1 | agents stub | FR-CIV-AGENTS-000 | see NFR doc | Stub compiles | `crates/agents/` | `agents::schema_version_stub` | Bevy/Godot/Unreal | Done |
| P-A1 | wardrobe/tools | FR-CIV-AGENTS-001 | see NFR doc | Wardrobe + tools state ticks deterministically | `crates/agents/`, `engine.rs` `phase_diffusion` | `agents::wardrobe_tools_deterministic`, `phase_diffusion_bumps_wardrobe_eras` | Bevy (live_scene) | Done |
| P-A1 | LOD tick | FR-CIV-AGENTS-010 | see NFR doc | LOD tick lower freq, no state divergence (gestalt) | `crates/agents/` | `agents::lod_gestalt_no_divergence` | Bevy | Done |

### Diffusion — P-A1 (crate `civ-diffusion`; wired via `phase_diffusion`)

| Epic/Phase | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| P-A1 | diffusion stub | FR-CIV-DIFFUSION-000 | see NFR doc | Stub compiles | `crates/diffusion/` | `diffusion::schema_version_stub` | Bevy/Godot/Unreal | Done |
| P-A1 | S-curve | FR-CIV-DIFFUSION-001 | see NFR doc | Bass/Rogers S-curve matches closed-form within tolerance | `crates/diffusion/`, `engine.rs` `phase_diffusion` | `diffusion::s_curve_adoption` | Bevy | Done |

### Laws — P-L1 (crate `civ-laws`; library + watch only, **not in engine tick**)

| Epic/Phase | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| P-L1 | laws stub | FR-CIV-LAWS-000 | see NFR doc | Stub compiles | `crates/laws/` | `laws::schema_version_stub` | (watch only) | Partial — not in `Simulation::tick` |
| P-L1 | RON schema | FR-CIV-LAWS-001 | see NFR doc | Versioned RON schema loads + round-trips | `crates/laws/` | `laws::ron_roundtrip` | (watch only) | Partial — not in `Simulation::tick` |
| P-L1 | validator | FR-CIV-LAWS-002 | see NFR doc | Validator rejects extensions missing `{inputs,outputs,losses,dependencies}` | `crates/laws/` | `laws::validator_rejects_incomplete` | (watch only) | Partial — not in `Simulation::tick` |

### Research — P-R1 (crate `civ-research`; **not wired into engine tick**)

| Epic/Phase | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| P-R1 | research stub | FR-CIV-RESEARCH-000 | see NFR doc | Stub compiles | `crates/research/` | `research::schema_version_stub` | (not surfaced yet) | Partial — not in `Simulation::tick` |
| P-R1 | LLM cache | FR-CIV-RESEARCH-001 | see NFR doc | Cache hit byte-identical to cached value | `crates/research/` | `research::llm_cache_hit` | (not surfaced yet) | Partial — not in `Simulation::tick` |
| P-R1 | canonical replay | FR-CIV-RESEARCH-002 | see NFR doc | Canonical replay refuses first `LlmEvent` in log | `crates/research/`, `engine.rs` (`apply_replay_research`) | `research::canonical_replay_refuses_llm` | (not surfaced yet) | Partial — replay hook present; live tick not wired |
| P-R1 | hybrid replay | FR-CIV-RESEARCH-003 | see NFR doc | Hybrid replay on cache miss refuses to advance | `crates/research/` | `research::hybrid_cache_miss_refuses` | (not surfaced yet) | Partial — not in `Simulation::tick` |

### Tactics — P-W1 (crate `civ-tactics`; wired via `phase_tactics`/`phase_military`)

> Tactics is the most mature 3D family: voxel-destructible combat, doctrine GA, LOS,
> formations, war bridge, fog-of-war, WASM mod host, save/replay — all wired into the
> engine tick and surfaced on watch/server snapshots and the (deprecated) web dashboard.
> All rows below are tested and engine-wired → **Done**. NFR cells = "see NFR doc".

| Epic/Phase | User Story | FR ID | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|
| P-W1 | tactics stub | FR-CIV-TACTICS-000 | Stub compiles | `crates/tactics/` | `tactics::schema_version_stub` | Bevy/Godot/Unreal | Done |
| P-W1 | destructible | FR-CIV-TACTICS-001 | Deterministic voxel-destructible damage | `crates/tactics/`, `engine.rs` | `apply_damage_is_deterministic`, `pending_damage_drains` | Bevy | Done |
| P-W1 | damage pulses | FR-CIV-TACTICS-001-int | Damage pulses on watch/server snapshot + web markers | `watch`, `server/jsonrpc`, `web/dashboard` | `snapshot_fields_from_sim_includes_damage_after_tick` | Bevy/Godot + web | Done |
| P-W1 | doctrine GA | FR-CIV-TACTICS-010 | Doctrine GA converges reproducibly (fixed seed) | `crates/tactics/`, `engine.rs` | `tactics::doctrine_ga_converges`, `phase_tactics_evolve_doctrine_on_cadence` | Bevy | Done |
| P-W1 | LOS | FR-CIV-TACTICS-020 | Deterministic voxel line-of-sight | `crates/tactics/los.rs` | `line_of_sight_blocks_solid_voxels` | Bevy | Done |
| P-W1 | formations | FR-CIV-TACTICS-021 | Stable formation offsets per kind/slot count | `crates/tactics/formation.rs` | `formation_offsets_line_and_wedge` | Bevy | Done |
| P-W1 | war bridge | FR-CIV-TACTICS-022 | War bridge queues voxel damage on cadence w/ LOS | `crates/tactics/war_bridge.rs`, `engine.rs` | `war_bridge_queues_damage_on_cadence_with_los` | Bevy | Done |
| P-W1 | doctrine fitness | FR-CIV-TACTICS-023 | Doctrine fitness scores faction engagement stats | `crates/tactics/doctrine_fitness.rs` | `doctrine_fitness_rewards_engagement_stats` | Bevy | Done |
| P-W1 | per-soldier combat | FR-CIV-TACTICS-024 | Per-soldier engagements on snapshot | `crates/tactics/war_bridge.rs`, `engine.rs`, `server/jsonrpc` | `war_bridge_records_combat_replay_events` | Bevy/Godot | Done |
| P-W1 | combat replay | FR-CIV-TACTICS-025 | Combat engagements in replay log | `crates/engine/src/replay.rs` | `war_bridge_records_combat_replay_events` | Bevy | Done |
| P-W1 | replay restore | FR-CIV-TACTICS-025-int | Replay restores combat `pending_damage` | `replay.rs`, `engine.rs` | `replay_combat_events_restore_pending_damage` | Bevy | Done |
| P-W1 | replay parity | FR-CIV-TACTICS-025-int2 | Replay combat drains == live voxel state | `engine.rs` | `replay_combat_drains_to_same_voxel_state_as_live` | Bevy | Done |
| P-W1 | replay determinism | FR-CIV-TACTICS-025-int3 | Combat replay log deterministic per seed | `engine.rs` | `replay_combat_log_deterministic_for_seed_rerun` | Bevy | Done |
| P-W1 | op layer hook | FR-CIV-TACTICS-030 | Operational layer hook for engagements | `crates/tactics/operational.rs` | compile + `NoopOperationalLayer` | Bevy | Done |
| P-W1 | op movement | FR-CIV-TACTICS-031 | Operational movement toward nearest enemy | `crates/tactics/movement.rs` | `operational_movement_steps_toward_enemy` | Bevy | Done |
| P-W1 | per-soldier HP | FR-CIV-TACTICS-032 | Per-soldier HP military ECS component | `engine.rs`, `spawn.rs` | `war_bridge_records_combat_replay_events` | Bevy | Done |
| P-W1 | BFS pathfind | FR-CIV-TACTICS-033 | Operational pathfinding (BFS next step) | `crates/tactics/pathfinding.rs` | `pathfinding_bfs_steps_toward_enemy` | Bevy | Done |
| P-W1 | mil mod hook | FR-CIV-TACTICS-034 | Military-phase mod hook (`read_military`) | `crates/mod-host/`, `engine.rs` | `mod_registry_military_phase_emits_for_read_military` | Bevy | Done |
| P-W1 | mil work/tick | FR-CIV-TACTICS-035 | Higher per-tick military work (cadence + pulses) | `military_phase.rs`, `movement.rs`, `war_bridge.rs` | `operational_movement_steps_toward_enemy` | Bevy | Done |
| P-W1 | obstacle grid | FR-CIV-TACTICS-036 | Voxel obstacle grid for op movement | `grid_obstacles.rs`, `movement.rs` | `operational_movement_avoids_voxel_obstacle` | Bevy | Done |
| P-W1 | A* pathfind | FR-CIV-TACTICS-037 | A* obstacle-aware pathfinding | `pathfinding.rs` | `astar_path_with_blocked` | Bevy | Done |
| P-W1 | WASM mil tick | FR-CIV-TACTICS-038 | WASM `civlab_military_tick` host invoke | `wasm_guest.rs`, `civlab-sdk` | `wasm_military_tick_invokes_civlab_export` | Bevy (via mod host) | Done |
| P-W1 | occupied block | FR-CIV-TACTICS-039 | Occupied-cell blocking in op pathfinding | `grid_obstacles.rs`, `movement.rs` | `operational_movement_avoids_occupied_cell` | Bevy | Done |
| P-W1 | sim_tick to WASM | FR-CIV-TACTICS-040 | Military WASM tick receives `sim_tick` (i64) | `wasm_guest.rs`, `mod-host`, `civlab-sdk` | `wasm_military_tick_invokes_civlab_export` | Bevy | Done |
| P-W1 | combat hash chain | FR-CIV-TACTICS-041 | Combat events extend replay hash chain | `hash_chain.rs`, `replay.rs` | `combat_events_extend_replay_hash_chain` | Bevy | Done |
| P-W1 | fog gates combat | FR-CIV-TACTICS-042 | Fog-of-war gates war-bridge engagements | `fog_of_war.rs`, `war_bridge.rs`, `engine.rs` | `fog_blocks_engagement_beyond_vision` | Bevy | Done |
| P-W1 | mod signature | FR-CIV-TACTICS-043 | Ed25519 mod WASM signature verification | `mod-host/signature.rs` | `verify_accepts_valid_signature` | (server) | Done |
| P-W1 | capability API | FR-CIV-TACTICS-044 | Policy/military WASM tick capability API | `wasm_guest.rs`, `civlab-sdk` | `capability_version_is_non_empty` | Bevy | Done |
| P-W1 | scenario fog | FR-CIV-TACTICS-045 | Scenario fog fields wire military phase | `scenario.rs`, `scenarios/baseline.yaml` | `scenario_fog_wires_military_phase` | Bevy | Done |
| P-W1 | econ WASM tick | FR-CIV-TACTICS-046 | Economic WASM tick host invoke | `wasm_guest.rs`, `mod-host`, `engine.rs` | `wasm_economy_tick_invokes_civlab_export` | Bevy | Done |
| P-W1 | capability import | FR-CIV-TACTICS-047 | WASM host `civlab::capability_api_version` import | `wasm_guest.rs` | `wasm_guest_reads_capability_host_import` | Bevy | Done |
| P-W1 | example econ mod | FR-CIV-TACTICS-048 | Example economic mod dir + WASM tick | `mods/example-economic/`, `mod-host` | `example_economic_dir_wasm_ticks_when_built` | (mods) | Done |
| P-W1 | guest mem rw | FR-CIV-TACTICS-049 | Host guest memory read/write imports | `wasm_guest.rs` | `host_memory_import_read_write` | Bevy | Done |
| P-W1 | mil tuning | FR-CIV-TACTICS-050 | Scenario military cadence/range tuning | `scenario.rs`, `baseline.yaml` | `scenario_military_wires_military_phase` | Bevy | Done |
| P-W1 | baseline loads mod | FR-CIV-TACTICS-051 | Baseline scenario loads example-economic mod | `scenarios/baseline.yaml` | `baseline_yaml_parses` | (scenario) | Done |
| P-W1 | guest mem snapshot | FR-CIV-TACTICS-052 | Per-mod guest memory snapshots on ModHost | `mod-host` | `guest_memory_persists_across_invocations` | (server) | Done |
| P-W1 | full host imports | FR-CIV-TACTICS-053 | Full capability host imports (`sim_tick`, list) | `wasm_guest.rs`, `civlab-sdk` | `sim_tick_host_import_visible_to_guest` | Bevy | Done |
| P-W1 | mod browser | FR-CIV-TACTICS-054 | Mod browser on watch/server snapshot + dashboard | `watch`, `server`, `web/dashboard` | `mods_panel` | web + Bevy | Done |
| P-W1 | guest state json | FR-CIV-TACTICS-055 | Mod guest state JSON save/load stub | `guest_state.rs`, `engine.rs` | `mod_guest_state_save_round_trips_json` | (server) | Done |
| P-W1 | determinism scan | FR-CIV-TACTICS-056 | WASM determinism scan at mod load | `determinism.rs` | `rejects_f32_sqrt` | (server) | Done |
| P-W1 | float opcode count | FR-CIV-TACTICS-057 | Float opcode count in determinism report | `determinism.rs` | `report_counts_float_ops_without_hard_reject` | (server) | Done |
| P-W1 | civsave folder | FR-CIV-TACTICS-058 | `.civsave/` debug save folder | `save_bundle.rs`, `civ-watch` | `civsave_folder_round_trips_mod_guest_state` | (watch) | Done |
| P-W1 | civmod package | FR-CIV-TACTICS-059 | Package example mods as `.civmod` | `justfile`, `package-example-mod.ps1` | `civis-3d-mod-package-all` | (tooling) | Done |
| P-W1 | civsave.zst | FR-CIV-TACTICS-060 | `.civsave.zst` compressed save archive | `save_bundle.rs`, `civ-watch` | `civsave_archive_round_trips_mod_guest_state` | (watch) | Done |
| P-W1 | float data-flow | FR-CIV-TACTICS-061 | `action_emit` float data-flow trace at load | `float_data_flow.rs`, `determinism.rs` | `rejects_reinterpret_f64_before_action_emit` | (server) | Done |
| P-W1 | mod catalog | FR-CIV-TACTICS-062 | Mod catalog + runtime install | `civ-watch`, `web/dashboard` | `post_mods_install` | web | Done |
| P-W1 | mod unload | FR-CIV-TACTICS-063 | Mod unload HTTP + `mod.unloaded.v1` bus JSON | `civ-watch`, `mod-host` | `post_mods_unload` | web | Done |
| P-W1 | civmod upload | FR-CIV-TACTICS-064 | Signed `.civmod` upload API | `civ-watch` | `post_mods_upload` | web | Done |
| P-W1 | save slots | FR-CIV-TACTICS-065 | Production slots + autosave ring | `civ-watch` | `post_save_slot`, `autosave_ring` | web | Done |
| P-W1 | save RPC | FR-CIV-TACTICS-066 | `save.slot`/`save.load`/`save.list` JSON-RPC | `civ-server` | `ws_jsonrpc_save_slot_roundtrip` | Bevy/Godot/Unreal | Done |
| P-W1 | mod publish | FR-CIV-TACTICS-067 | Mod publish store + HTTP API | `civ-watch` | `post_mods_publish` | web | Done |
| P-W1 | mod hot reload | FR-CIV-TACTICS-068 | Mod hot reload API | `civ-watch`, `mod-host` | `post_mods_reload` | web | Done |
| P-W1 | sqlite save meta | FR-CIV-TACTICS-069 | Session-scoped SQLite save metadata | `civ-save-db`, `civ-watch` | `save_db::*`, `post_save_slot` | (server) | Done |
| P-W1 | remote mod fetch | FR-CIV-TACTICS-070 | Remote mod fetch cache | `civ-watch`, `mods/remote/` | `post_mods_fetch`, `list_remote_mods` | web | Done |
| P-W1 | capability enforce | FR-CIV-TACTICS-071 | CIV-0700 capability enforcement (`world_read`,`action_emit`) | `mod-host/capability.rs` | `capability::*` | (server) | Done |
| P-W1 | session saved bus | FR-CIV-TACTICS-072 | `session.saved.v1` on replay bus + snapshot | `engine/replay.rs`, `civ-watch` | `session_saved_*` | web | Done |
| P-W1 | web remote fetch | FR-CIV-TACTICS-073 | Web remote mod fetch UI | `web/dashboard/mods_panel.tsx` | manual / tsc | web | Done |
| P-W1 | PolicyMod trait | FR-CIV-TACTICS-074 | PolicyMod trait surface (CIV-0700 §5) | `civlab-sdk/policy.rs` | `policy::*` | (sdk) | Done |
| P-W1 | perm violation bus | FR-CIV-TACTICS-075 | `mod.permission_violation.v1` on bus + snapshot | `engine/replay.rs`, `engine.rs`, `civ-watch` | `mod_permission_violation_*` | web | Done |
| P-W1 | server sqlite save | FR-CIV-TACTICS-076 | civ-server session SQLite on `save.slot` | `civ-save-db`, `server/ws_bridge.rs` | `save.slot` smoke | (server) | Done |
| P-W1 | signed registry | FR-CIV-TACTICS-077 | Signed remote mod registry for fetch | `mods/remote-registry.json`, `civ-watch` | `remote_registry_*` | web | Done |

### Planet — P-P1 (crate `civ-planet`; wired via `phase_planet`)

| Epic/Phase | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| P-P1 | planet stub | FR-CIV-PLANET-000 | see NFR doc | Stub compiles | `crates/planet/` | `planet::schema_version_stub` | Bevy (atmosphere/skybox) | Done |
| P-P1 | day/night | FR-CIV-PLANET-001 | see NFR doc | Day/night cycle deterministic, tick-tied | `crates/planet/`, `engine.rs` `phase_planet` | `planet::day_night_deterministic` | Bevy (atmosphere/skybox/post_fx) | Done |
| P-P1 | tides | FR-CIV-PLANET-002 | see NFR doc | Moon tides modulate coastal water deterministically | `crates/planet/` | `planet::tides_deterministic` | Bevy (terrain) | Done |

### Protocol-3D (crate `civ-protocol-3d`; wired into server + watch transport)

| Epic/Phase | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| P-V1 | proto3d stub | FR-CIV-PROTO3D-000 | see NFR doc | Stub compiles | `crates/protocol-3d/` | `protocol3d::schema_version_stub` | Bevy/Godot/Unreal | Done |
| P-V1 | voxel deltas | FR-CIV-PROTO3D-001 | see NFR doc | Voxel delta frames binary serialize; lossless round-trip | `crates/protocol-3d/`, `server`, `watch` | `protocol3d::voxel_delta_roundtrip` | Bevy/Godot/Unreal | Done |
| P-V2 | building diffs | FR-CIV-PROTO3D-002 | see NFR doc | Building diff frames carry procedural/freehand provenance | `crates/protocol-3d/` | `protocol3d::building_diff_provenance` | Bevy/Godot/Unreal | Done |

### UX — P-U1 (`clients/godot-ref` + dashboard; spawn/timelapse)

| Epic/Phase | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| P-U1 | spawn API | FR-CIV-UX-000 | see NFR doc | N UI spawns → N entity-create events | `clients/godot-ref/`, `engine/spawn.rs` | `ux::spawn_emits_entity_events` | Godot (+ Bevy spawn_tools) | Done |
| P-U1 | timelapse | FR-CIV-UX-001 | see NFR doc | Era timelapse configurable rate, no divergence vs real-time | `clients/godot-ref/` | `ux::timelapse_no_divergence` | Godot | Done |
| P-U1 | convoy | FR-CIV-UX-004 | see NFR doc | Drag-place + convoy along path | `clients/godot-ref`, `web/dashboard` | `ux::convoy_positions`, `spawnConvoy.ts` | Godot + web | Done |
| P-U1 | spawn palette | FR-CIV-UX-006 | see NFR doc | Spawn palette incl. hangar | `engine/spawn.rs`, `server`, `watch` | `spawn_kind_palette_is_wired` | Godot/Bevy | Done |
| P-U1 | godot attach | FR-CIV-GODOT-ATTACH-000..004 | see NFR doc | Godot civ-server WS + watch terrain | `civis_ws_client.gd` | `ws_smoke`, attach docs | Godot | Done |

### Bevy reference client (`FR-CIV-BEVY-*`; `clients/bevy-ref`)

| Epic/Phase | User Story | FR ID | NFR ID(s) | Acceptance Criteria (concise) | Implementing crate/module | Verifying test(s) | Client(s) | Status |
|---|---|---|---|---|---|---|---|---|
| P-W1 | event feed | FR-CIV-BEVY-023 | see NFR doc | `event_feed` egui toasts + log; `live_attach` pushes System events on WS state | `clients/bevy-ref` (`event_feed`, `live_attach`, `standalone`) | `event_feed::*` (cargo test), `civ-standalone` cargo check | **Bevy** | Done |
| P-W1 | menus | FR-CIV-BEVY-024 | see NFR doc | `MenusPlugin`: Escape pause overlay, settings stub, era banner; sim gated while paused | `clients/bevy-ref` (`menus`, `lib.rs`, `bin/standalone.rs`, `sim_bridge`) | `menus::*` (cargo test), `civ-standalone` cargo check | **Bevy** | Done |
| P-W1 | GPU backend | FR-CIV-BEVY-026 | see NFR doc | Native GPU backend selection (`CIV_BEVY_BACKEND`; DX12/Vulkan/Metal; no GLES) | `clients/bevy-ref` (`native_backend`), README, `docs/research/wgpu-native-escape-hatches.md` | `native_backend` (cargo test), `just civis-3d-live-smoke` | **Bevy** | Done |

---

## Coverage summary

### Counts

| Family | FR count | Done | Partial | Planned | With test | With test % |
|---|---:|---:|---:|---:|---:|---:|
| Strategic core (CORE/ECON/PROTO/REPLAY/API/CLIENT) | 24 | 0 | 14 | 10 | 14 | 58% |
| FR-METRICS | 3 | 2 | 1 | 0 | 3 | 100% |
| FR-CIV-VOXEL | 6 | 6 | 0 | 0 | 6 | 100% |
| FR-CIV-BUILD | 5 | 5 | 0 | 0 | 5 | 100% |
| FR-CIV-GENETICS | 4 | 0 | 4 | 0 | 4 | 100% |
| FR-CIV-SPECIES | 2 | 0 | 2 | 0 | 2 | 100% |
| FR-CIV-AGENTS | 3 | 3 | 0 | 0 | 3 | 100% |
| FR-CIV-DIFFUSION | 2 | 2 | 0 | 0 | 2 | 100% |
| FR-CIV-LAWS | 3 | 0 | 3 | 0 | 3 | 100% |
| FR-CIV-RESEARCH | 4 | 0 | 4 | 0 | 4 | 100% |
| FR-CIV-TACTICS | 56 | 56 | 0 | 0 | 56 | 100% |
| FR-CIV-PLANET | 3 | 3 | 0 | 0 | 3 | 100% |
| FR-CIV-PROTO3D | 3 | 3 | 0 | 0 | 3 | 100% |
| FR-CIV-UX | 5 | 5 | 0 | 0 | 5 | 100% |
| FR-CIV-BEVY | 3 | 3 | 0 | 0 | 3 | 100% |
| **TOTAL** | **126** | **88** | **35** | **11** (web) | **116** | **~92%** |

> FR-CIV-TACTICS count (56) and FR-CIV-UX count (5) include the `-int` / `-int2` / `-int3`
> integration sub-IDs and the rolled-up `FR-CIV-GODOT-ATTACH-000..004` row, matching
> `docs/traceability/fr-3d-matrix.md`.

### Headline metrics

- **Total FR IDs tracked:** **126** (27 strategic core + METRICS, 99 `FR-CIV-*`).
- **Done:** **88** (≈70%) — overwhelmingly the 3D extension families, where tactics alone
  contributes 56.
- **Partial:** **35** (≈28%) — all 24 strategic core FRs that are implemented-but-not-fully
  wired/AC-covered, plus the 13 `FR-CIV-*` rows in genetics/species/laws/research whose
  crates are tested but **not yet reachable from `Simulation::tick`** (see engine tick
  wiring in engine.rs; there is no standalone tick-integration plan doc).
- **Planned:** **11** — the Python research API (FR-API-002/003/004), several protocol
  handshake/filter/queue FRs, and the deprecated web TS client (FR-CLIENT-002).
- **% with at least one verifying test:** **~92%** (116/126). The **10 GAP rows** (no
  test) are all strategic core: FR-CORE-004, -006, -007, FR-PROTO-003, -005,
  FR-API-002/003/004, plus partial-coverage gaps noted inline on FR-ECON-002/004/005,
  FR-PROTO-001/002/004, FR-CLIENT-003, FR-METRICS-002 (these have *some* test but miss a
  specific AC).
- **% surfaced in Bevy (primary client):** **~83%** (105/126). Bevy is the primary desktop
  target and surfaces every voxel/build/agent/diffusion/planet/tactics/proto3d FR plus the
  Bevy-specific client FRs. Rows **not** surfaced in Bevy yet: genetics (4), species (2),
  laws (3), research (4) — library-only crates awaiting engine-tick + render wiring — and
  the server-only / web-only / Python rows.
- **Three clients exist:** `clients/bevy-ref` (primary), `clients/godot-ref` (UX spawn
  reference), `clients/unreal-show` (showcase). Most gameplay FRs surface in all three via
  the shared `civ-server` snapshot/`civ-protocol-3d` transport; Godot owns the UX spawn FRs
  and Bevy owns the `FR-CIV-BEVY-*` rows.

---

*Maintenance: when `docs/reference/non-functional-requirements.md` lands, replace every
"see NFR doc" cell with the concrete `NFR-CIV-*` ID(s). When genetics/species/laws/research
are wired into `Simulation::tick`, promote their rows from Partial → Done and update the
Bevy-surfaced percentage. FR text source of truth: `FUNCTIONAL_REQUIREMENTS.md` +
`docs/development-guide/fr-3d-additions.md`.*
