# civ-bevy-ref

Civis Bevy 3D reference client. Per `docs/adr/ADR-007-three-renderers.md`:

> **Daily-driver for CI, deterministic replay verification, screenshot regression,
> agent-driven workflows.** Visual quality below Unreal but improving (`bevy_pbr`,
> `bevy_solari` for RT GI in 0.15+).

## Status

Pre-renderer headless smoke. The binary builds a tiny `VoxelWorld`, drains its
dirty events, meshes one populated chunk with the engine-neutral `CubicMesher`,
and prints the face count. Real Bevy rendering lands behind the `bevy` feature
flag in a follow-up PR.

## Run

```bash
cargo run -p civ-bevy-ref
```

Expected output (current iteration):

```
dirty events: 64
mesh: 384 vertices, 576 indices
```

(4³ = 64 voxel writes; the 4×4×4 cube exposes 6 × 4² = 96 faces → 384 vertices,
576 indices — internal faces correctly culled by the cubic mesher.)
