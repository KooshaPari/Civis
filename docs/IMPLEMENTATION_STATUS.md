# Implementation Status

**As of:** 2026-05-24  
**Authoritative code map:** root `Cargo.toml` workspace members (not legacy crate names in `TRACEABILITY_MATRIX.md`).

## Workspace crates (implemented in repo)

| Crate | Package | Role |
|-------|---------|------|
| `crates/engine` | `civ-engine` | Tick loop, ECS (`hecs`), replay I/O + BLAKE3 `hash_chain`, planet/climate/voxel/build/diffusion, economy + market |
| `crates/voxel` | `civ-voxel` | Adaptive 3D substrate, dirty events |
| `crates/build` | `civ-build` | Building graph, parcel allocation |
| `crates/agents` | `civ-agents` | Civilian cohorts, wardrobe/tools |
| `crates/diffusion` | `civ-diffusion` | Cohort diffusion params |
| `crates/planet` | `civ-planet` | Planet/moon climate |
| `crates/tactics` | `civ-tactics` | Voxel damage events |
| `crates/genetics` | `civ-genetics` | DNA stubs / schema version |
| `crates/economy` | `civ-economy` | `EconomyState`, `InstitutionLedger` stub, `CapitalistAllocator`, `MarketState::step` |
| `crates/species` | `civ-species` | Phenotype mapping stubs |
| `crates/laws` | `civ-laws` | RON law schema stubs |
| `crates/research` | `civ-research` | ADR-006 stubs: `TechCard` validator, `ReplayMode`, `LlmEvent`, hash-keyed cache; no live LLM client |
| `crates/protocol-3d` | `civ-protocol-3d` | 3D renderer protocol (`Frame3d`); binary `F3D0` encode/decode |
| `crates/server` | `civ-server` | WebSocket JSON-RPC + HTTP `healthz`, `GET /replay/export`, `POST /replay/import` |
| `crates/infra` | `civ-infra` | Infra helpers (PG replay stubs) |
| `crates/watch` | `civ-watch` | Live-dev harness: background sim, HTTP `/snapshot` `/terrain` `/events`, `/control/*` |
| `clients/bevy-ref` | `civ-bevy-ref` | Headless mesh smoke; `bevy` feature → renderer + `ws_client` |
| `clients/godot-ref/rust` | `civis-godot-rust` | Godot 4 GDExtension; HTTP client to civ-watch (`127.0.0.1:9090`); workspace member |

## Engine & server (landed, partial)

| Area | Location | Status |
|------|----------|--------|
| FR-API-001 scenario YAML | `engine/src/scenario.rs`, `scenarios/baseline.yaml` | `load_scenario`, validation, `into_simulation`; sets `economy_policy` |
| CIV-0104 tick invariants | `engine/src/invariants.rs` | Tick/replay alignment, energy budget ≥ 0; `debug_assert` each `Simulation::tick()` |
| Economy sync (CIV-0100) | `economy/`, `engine.rs::phase_economy` | `CapitalistAllocator::allocate` → `drain_energy_budget` + `step`; joules ↔ `economy_state`; policy via `effective_consumption` |
| Economy market | `economy/src/market.rs`, `engine.rs::phase_economy` | `MarketState::step` each tick; deterministic per-good price updates |
| Hash chain (CIV-0001) | `engine/src/hash_chain.rs`, `replay.rs` | BLAKE3 tick hashes + append-only chain; `verify_hash_chain` on `.civreplay` load |
| Binary F3D0 frames | `protocol-3d/src/lib.rs`, `server/src/ws_bridge.rs` | `encode_frame3d_binary` / `decode_frame3d_binary`; WS tick push via `TickBroadcastFormat` (`Text` / `Binary` / `Both`, default `Both`) |
| Fixed metrics | `engine/src/metrics.rs` | `compute_fixed` + unit test parity vs float `compute` |
| JSON-RPC (CIV-0200) | `server/src/jsonrpc.rs`, `ws_bridge.rs` | `health`, `sim.status`, `sim.snapshot`, `sim.command`, speed/replay/policy/reset; optional operator role gate + unit tests |
| HTTP replay I/O | `ws_bridge.rs` | `GET /replay/export` → `.civreplay` bytes; `POST /replay/import` loads octet-stream into bridge (`{ "ok": true, "tick": … }`) |
| Spectator pins | `engine/src/spectator.rs` | `civ_pins` from agent `Position3d` (spawn coords on `sim.snapshot`) |
| Server integration | `server/tests/ws_smoke.rs` | 28+ tokio tests: healthz, WS RPC, spawn pin snapshot, replay I/O, role gate |

## GFX / UI (reference clients)

ADR-009 / CIV-0300 visuals in reference clients — not `crates/render`. Cross-client minimap UV/chunk rules: [`docs/guides/minimap-conventions.md`](guides/minimap-conventions.md).

| Client | Location | Landed |
|--------|----------|--------|
| **Bevy** orbit / fade / LOD / agents | `clients/bevy-ref/` (`bevy_window.rs`, `lib.rs`) | `OrbitCamera` drag/scroll/`R`/`WASD`; chunk fade-in; `mesh_lod_level` → `CubicMesher`; `agent_color_from_id` + optional `#id` labels + payload scale; `LiveHudSnapshot` overlay |
| **Bevy** minimap / binary WS | `bevy-ref/ws_client.rs`, `bevy_window.rs`, `lib.rs` | F3D0-first ticks; `sim.snapshot` JSON-RPC poll for `is_day`; minimap click-to-focus; day/night lighting |
| **Godot** camera / terrain / UI | `clients/godot-ref/scripts/` | Orbit `Camera3D` (`camera.gd`); 128×128 minimap grid + click-to-focus (`minimap.gd`); `terrain_height_exaggeration`; `biome_color` / `height_color`; control `tooltip_text` hints |
| **Godot** civ-server attach | `clients/godot-ref/scripts/civis_ws_client.gd`, `main.gd` | Default `attach_mode=server`: WS JSON-RPC + F3D0-throttled `sim.snapshot`; terrain via civ-watch HTTP; `spectator_mode` default |
| **Godot** P-U1 | `main.gd`, `era_timelapse.gd`, `camera.gd` | Buildings + military pins; era HUD; server attach + L2 authoring; camera presets wide/close/orbit |
| **Unreal** HTTP + WS | `CivProtocolClient.cpp`, `CivWsClient.cpp`, `CivShowGameMode` | HTTP terrain/controls; WS JSON-RPC + `civ_pins` civilian sync |
| **Web** FR-CIV-WEB-007 | `web/dashboard/src/babylon_scene.tsx`, `scene_view.tsx` | Optional Babylon renderer; Three fallback |
| **Web** FR-CIV-WEB-008 | `web/dashboard/src/lib/authoring.ts`, `bottom_bar.tsx` | L2 authoring default on; `?spectator=1` for read-only |
| **Server** institutions on snapshot | `crates/server/src/jsonrpc.rs` | `sim.snapshot.institutions[]` from economy ledger |
| **Server** authoring RPC | `sim.spawn_civilian`, `sim.spawn_entity`, `sim.place_voxel` | Vehicle → `Knight` military pin; airport → `CityCenter` building; `military_units` on `sim.snapshot` |
| **Roadmap** quality ladder | `docs/roadmap/product-quality-ladder.md` | L0–L5 definitions; Manor Lords = L5 on L3–L4 |
| **Dashboard** theme / perf / stats / agents | `web/dashboard/` (`theme.ts`, `perf_panel.tsx`, `stats_panel.tsx`, `agents_panel.tsx`, `useCivisAttach.ts`) | Persisted `data-theme`; sparkline; **`StatsPanel`** tick + voxel chunk count + FPS; **`AgentsPanel`** seen-agent count + recent ids from `AgentAppearance` frames |
| **Dashboard** attach / minimap | `attachConfig.ts`, `bottom_bar.tsx`, `side_panel.tsx` | Binary-first WS (`?binary=`); `?attach=watch` SSE; 160×160 terrain minimap with click-to-inspect chunk; `mergeSnapshot` parses nested `economy.institutions` and population/diplomacy pulses |

## Spec domains vs code (gap summary)

| Spec area | Matrix column | In workspace? | Notes |
|-----------|---------------|---------------|-------|
| Core loop (CIV-0001) | `crates/engine` | **Partial** | Tick + hash chain + integrity checks |
| Economy (CIV-0100, 0107) | `crates/economy` | **Partial** | Allocator + market; no full fiscal layer |
| Research (FR-CIV-RESEARCH-*) | `crates/research` | **Partial** | ADR-006 types/cache; no live LLM |
| LOD (CIV-0101) | `crates/engine/src/lod.rs` | **Partial** | `LodPolicy`, `should_tick_entity`, zoom stubs + FR-LOD tests; engine `phase_diffusion` skips Warm/Cold civilians off cadence |
| Climate (CIV-0102) | `crates/climate` | **No** | `civ-planet` orbital climate; not CIV-0102 CO₂ model |
| Institutions, citizens, social, diplomacy | dedicated crates | **No** | Citizen components in `civ-engine` ECS only |
| AI (CIV-0400) | `crates/ai` | **No** | |
| Protocol (CIV-0200) | `crates/protocol-3d`, `crates/server` | **Partial** | JSON-RPC + HTTP replay I/O + `F3D0` WS tick broadcast (`TickBroadcastFormat`) |
| UI / assets (CIV-0300, 060x) | reference clients | **Partial** | **GFX / UI** above; no production `crates/render` |
| Save/load, DB (CIV-1000) | `crates/db` | **No** | `ReplayLog` / `.civreplay` in engine + WS/HTTP; no persistence DB |
| Modding, audio, session (CIV-07–09) | various | **No** | Spec-closed; not wired |
## What is tested today

- **`cargo test -p civ-engine`** (+ `determinism_proptest`, `invariants_proptest`) — tick/replay/economy/metrics/invariants
- **`cargo test -p civ-economy`** — allocator, ledger, market
- **`cargo test -p civ-research`** — ADR-006 validator/cache
- **`cargo test -p civ-protocol-3d`** — `F3D0` roundtrip
- **`cargo test -p civ-server`** — jsonrpc + ws_bridge + ws_smoke (28+)
- **`cargo test -p civ-watch`** — 16 tests: `/terrain`, `/snapshot`, `/events`, `/control/spawn_entity`, `damage`, save/load
- **`cargo test -p civ-bevy-ref`** / **`civis-godot-rust`** — reference client surface (GFX / UI)
- **`cargo test -p civ-infra`** (+ `--features pg`; integration `#[ignore]` without Docker Postgres)
- **CI / docs / web** — `fr-coverage.yml`; `docs:check`; `web/dashboard` npm test (GFX / UI)

## API & traceability docs

- **Root [`README.md`](../README.md)** — `civ-server` HTTP/WS (`healthz`, replay export/import, JSON-RPC incl. `sim.get_speed`)
- **`TRACEABILITY_MATRIX.md`** — FR catalog; crate column is target layout, not current tree
- **`docs/development-guide/fr-3d-additions.md`** — FR-CIV-VOXEL/BUILD/AGENTS/… aligned with 3D workspace crates

When assigning work: map CIV-01xx rows to new crates only when those crates exist; otherwise extend `civ-engine` or the 3D crate named in `fr-3d-additions.md`.
