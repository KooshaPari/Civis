# civ-godot-ref

Civis Godot 4 reference client. Per `docs/adr/ADR-007-three-renderers.md`:

> **UX iteration + WorldBox-style spawn editor.** Fastest editor + node tree for
> prototyping the spawn-anything UX. Voxel via `Zylann/godot_voxel` (Voxel Tools).

## Layout (planned)

```
clients/godot-ref/
├── README.md                  ← you are here
├── project.godot              ← Godot 4 project (added in the godot-ref PR)
├── rust/                      ← Rust GDExtension sub-crate
│   ├── Cargo.toml             ← cdylib; depends on civ-voxel + civ-agents
│   └── src/lib.rs             ← GDExt bindings using gdext crate
├── scenes/
│   └── main.tscn              ← root scene
└── scripts/                   ← GDScript-side UX (spawn editor, era timelapse)
```

## Status

Scaffold-only README. The Godot project skeleton and `gdext` Rust crate land in
follow-up phase PRs (P-U1 for the WorldBox-style spawn UX surface).

## Why a separate dir, not a workspace member

The Rust sub-crate (`clients/godot-ref/rust/`) WILL be a workspace member; the
Godot project files (`project.godot`, scenes, scripts) live alongside it but
are not part of the Cargo workspace.
