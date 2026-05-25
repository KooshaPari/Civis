# FR-CIV-L5 — Incremental visual presentation pass

**Status:** IN PROGRESS (scoped slices; not full Manor Lords bar)  
**Ladder:** [product-quality-ladder.md](../roadmap/product-quality-ladder.md) tier **L5**

Manor Lords–grade means art direction, animation density, and UX polish on L3–L4 depth. This doc tracks **incremental** presentation work that raises L1→L2 feel without a full art pipeline.

## Landed

| Client | Change |
|--------|--------|
| **Engine** | `spectator_view().civ_pins` from agent `Position3d` |
| **Godot** | `is_day` sun/ambient; capsule civilians; `SpawnBurst` on spawn/damage |
| **Web** | Day factor on terrain; spawn burst sprites; L2 authoring |
| **Bevy** | Minimap focus; `sim.snapshot` JSON-RPC side-channel → `is_day` lighting |
| **Unreal** | WS client + civilians; `ApplyDayNight` from snapshot `is_day` |

## Next L5 slices (ordered)

1. Godot job-specific capsule materials + foot placement on terrain height.
2. Bevy ambient + sky tint from `is_day` (not only directional illuminance).
3. Unreal materials / Nanite terrain (artist-owned).
4. Audio + UI density (out of scope for protocol milestone).

## Acceptance

- Spawn at norm `(0.4, 0.6)` appears in `sim.snapshot.civ_pins` within one RPC round-trip (`ws_jsonrpc_spawn_civilian_pin_appears_in_snapshot`).
- Minimap click moves Bevy orbit centre to chunk (`minimap_uv_to_chunk_grid` tests).
- `parse_jsonrpc_snapshot_meta` extracts `is_day` from `sim.snapshot` JSON-RPC for Bevy lighting.
