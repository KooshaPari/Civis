# Minimap conventions (cross-client)

Shared rules for reference-client minimaps. Authoritative Bevy helpers: `clients/bevy-ref/src/lib.rs`.

## Coordinate system

- **UV origin:** top-left `(0, 0)`; `(1, 1)` is bottom-right.
- **Chunk mapping:** each chunk dot or cell is placed at the **centre** of its grid cell, not the corner.
- **Bounds:** inclusive chunk-grid rectangle `(min_x, min_z, max_x, max_z)` on the XZ plane.

## Bevy (`civ-bevy-ref`)

- `chunk_to_minimap_uv(ChunkId, MinimapBounds) -> [u, v]` — normalised centre UV for a loaded chunk.
- `minimap_uv_to_chunk_grid([u, v], bounds) -> (cx, cz)` — inverse; clamps to bounds.
- UI: 160×160 top-right panel; 4×4 px dots at inset-mapped UV positions (`bevy_window.rs`).

## Godot (`godot-ref`)

- 128×128 terrain texture (`minimap.gd`); one pixel per world cell, same palette as 3D mesh.
- Camera orbit target shown as a white dot; **left-click** maps local `(x, y)` → grid `(x, z)` and calls `set_orbit_target`.
- Godot `y` down matches top-left UV; no separate normalisation layer yet.

## Dashboard (`web/dashboard`)

- **Today:** 160×160 canvas terrain preview from `/terrain`; faction capitals overlaid; no click-to-focus.
- **Future:** adopt `chunk_to_minimap_uv` semantics (or a TS port) for chunk-dot overlays and click-to-pan when F3D0 chunk keys drive the view.

## Out of scope (for now)

- Minimap click on Bevy ref and dashboard (Godot only).
- CIV-0300 production minimap (200×150, alerts, batched re-render) — see `docs/specs/CIV-0300-rts-ui-ux-spec.md`.
