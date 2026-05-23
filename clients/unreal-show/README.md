# civ-unreal-show

Civis Unreal 5 visual showcase + shipping client. Per `docs/adr/ADR-007-three-renderers.md`:

> **Visual showcase + shipping client.** Lumen + Nanite + Chaos. Lighter
> iteration cadence than Bevy/Godot. Attaches via the same WebSocket protocol;
> no sim logic duplication.

## Layout (planned)

```
clients/unreal-show/
├── README.md                  ← you are here
├── CivShow.uproject           ← Unreal 5 project (added in the unreal-show PR)
├── Source/CivShow/            ← C++ game module
│   ├── CivShow.Build.cs
│   ├── CivProtocolClient.h/cpp ← WebSocket attach to Civis core
│   └── VoxelMesher.h/cpp      ← consumes phenotype-voxel DirtyChunkEvents via FFI
├── Content/                   ← .uasset (kept in Git LFS; large blobs)
└── Plugins/                   ← any third-party voxel-destruction plugins
```

## Status

Scaffold-only README. Project files land later behind a decision on EULA / scope
(open question in `docs/roadmap/civis-3d-extension.md`).

## Why a separate dir, not a workspace member

Unreal is C++ + UnrealBuildTool, not Cargo. The FFI bindings into
`phenotype-voxel` will be generated via `cbindgen` (Civis-side Rust shim) and
consumed as a static library by the Unreal module. The Rust shim ships as a
separate crate (`clients/unreal-show/rust-shim/`) once the project lands.

## Open questions

- Standard Epic EULA (with royalty surface) vs shell-only renderer (no game
  logic in Unreal → no royalty surface)?
- Acceptable iteration cadence — full re-build after each Civis sim API change,
  or pin the FFI surface and bump only on intentional version bumps?
