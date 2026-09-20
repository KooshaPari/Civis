# Intent: FR-CIV-PBR-011 -- Greedy 2D atlas packer

> Date: 2026-09-19
> FR: FR-CIV-PBR-011
> Epic: FR-CIV-PBR

## User Intent

The Phase-3 PBR substrate builds a single RGBA `texture_2d_array` from
every material's albedo / normal / ORM maps. `crates/voxel/src/atlas/
gpu_atlas.rs` is the CPU-side packing engine: it lays out rectangles into
a power-of-two atlas and emits the placement map (`HashMap<String,
Rect>`) the Bevy adapter reads to bind each chunk's per-material UV.

### What This FR Achieves

- `GreedyAtlasPacker::new` — construct a packer at the desired atlas
  size (default `DEFAULT_ATLAS_SIZE = 4096`).
- `GreedyAtlasPacker::pack(rects)` — sorts rectangles by `(height desc,
  width desc)`, walks the sorted list, places each onto the first shelf
  that fits (Next-Fit Decreasing Height shelf packing), or allocates a
  new shelf at `y = sum(shelf_heights_so_far)`. Fails when the atlas
  height is exceeded.
- `GreedyAtlasPacker::pack_to_png` debug helper writes a `.ppm` next to
  the requested path.
- `InputTexture` row-major RGBA byte buffer.
- `PackedAtlas` placement map (consumed by the Bevy adapter to bind
  chunk quads).

`#![forbid(unsafe_code)]`. Determinism: identical `(rects, atlas size)`
produce byte-identical output — replay-safe.

## Acceptance Signal

- `cargo test -p voxel atlas::gpu_atlas` passes (covers shelf packing,
  power-of-two sizing, ASCII PPM emission).
- No `unsafe` blocks in the module.
- All operations are pure; no I/O outside `pack_to_png`.

## Traceability

| Artifact | Path |
|----------|------|
| Implementing crate | `crates/voxel/` |
| Packer | `crates/voxel/src/atlas/gpu_atlas.rs:1` |
| Surface | `crates/voxel/src/atlas/gpu_atlas.rs:40` |
