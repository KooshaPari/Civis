# Civis Godot Ref

Civis 3D reference client for Godot 4. Per `docs/adr/ADR-007-three-renderers.md`,
this is the UX iteration surface for the WorldBox-style spawn editor.

## Run

1. Install Godot 4.3+
2. `cd clients/godot-ref/rust && cargo build`
3. Open `clients/godot-ref/project.godot` in Godot 4
4. Press F5 - connects to `civ-watch` on `http://127.0.0.1:9090`

## Layout

```
clients/godot-ref/
├── README.md
├── .gitignore
├── civis.gdextension
├── justfile
├── project.godot
├── rust/
│   ├── Cargo.toml
│   └── src/lib.rs
├── scenes/
│   └── main.tscn
└── scripts/
    ├── main.gd
    └── ui.tscn
```
