# FR-CIV-L5 — Incremental visual presentation pass

**Status:** IN PROGRESS — **Quixel / Megascans art only** (artist-owned materials; agents wire slots)
**Ladder:** [product-quality-ladder.md](../roadmap/product-quality-ladder.md) tier **L5**

Manor Lords–grade means art direction, animation density, and UX polish on L3–L4 depth. Scoped **protocol-driven** presentation slices below are **done**; remaining work is external art import, not JSON-RPC shape.

## Completed slices

| Slice | Godot | Web | Bevy | Unreal |
|-------|-------|-----|------|--------|
| **`is_day` lighting** | `_apply_day_night` — sun + ambient from snapshot | Terrain day factor from `is_day` | `presentation_day_factor_target`, `bevy_window.rs` clear/ambient | `ApplyDayNight` from snapshot JSON |
| **Job colors on pins** | `JOB_COLORS` on capsule `job` field | `jobColor()` in `scene3d.tsx` | — (tooling; pins via JSON-RPC meta) | `FCivisJobColors::FromJobName` |
| **Spawn burst** | `SpawnBurst` on spawn + damage | Burst sprites on spawn | — | — |
| **Foot placement** | `_world_y_at_norm` for civ/buildings/military/burst | Pin Y from terrain sample | — | `VoxelTerrain::SampleWorldHeightAtNorm` |
| **WS + terrain attach** | `civis_ws_client.gd` + `CivisClient` HTTP | L2 authoring + server WS | `sim.snapshot` side-channel + F3D0 voxels | `UCivWsClient` + `UCivProtocolClient` |

### Evidence (quick links)

| Client | Change |
|--------|--------|
| **Engine** | `spectator_view().civ_pins` from agent `Position3d`; `Citizen.job` on pins (`civ_pins_include_job_when_citizen_component_present`) |
| **Godot** | `main.gd`, `spawn_burst.gd`; extension terrain in `rust/src/lib.rs` |
| **Web** | `scene3d.tsx`, `mergeSnapshot.ts` |
| **Bevy** | `clients/bevy-ref/src/lib.rs`, `bevy_window.rs` |
| **Unreal** | `CivShowGameMode.cpp`, `CivisJobColors.h` — see [fr-unreal-agent-playbook.md](fr-unreal-agent-playbook.md) |

## IN PROGRESS

| Item | Owner | Notes |
|------|-------|-------|
| **Quixel / Fab (Megascans)** | Artist | Landscape master material, Nanite rocks, foliage — import via Bridge → `Content/Materials/`, `Content/Megascans/` ([playbook](fr-unreal-agent-playbook.md)) |
| Unreal materials / Nanite terrain | Artist + optional C++ slot wiring | `Content/Materials/` |
| Godot drag-preview Y | Optional polish | Align drag ghost to terrain foot helper (foot placement for spawned entities is done) |
| Audio + UI density | Out of scope | Protocol milestone |

## Acceptance (regression)

- Spawn at norm `(0.4, 0.6)` appears in `sim.snapshot.civ_pins` within one RPC round-trip — `ws_jsonrpc_spawn_civilian_pin_appears_in_snapshot`.
- Startup snapshot includes non-null `civ_pins[].job` (e.g. `farmer`) — `ws_jsonrpc_sim_snapshot_returns_snapshot_fields` (run via `agent-smoke.ps1`).
- Minimap click moves Bevy orbit centre to chunk — `minimap_uv_to_chunk_grid` tests.
- `parse_jsonrpc_snapshot_meta` extracts `is_day` from `sim.snapshot` JSON-RPC for Bevy lighting.

## Related

- [fr-p-u1-roadmap.md](fr-p-u1-roadmap.md)
- [client-attach-matrix.md](../guides/client-attach-matrix.md)
- [deferred-crates.md](../guides/deferred-crates.md) — L3–L4 domain crates not blocking L5 protocol slices
