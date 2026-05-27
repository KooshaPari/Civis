# Build Status

Date: 2026-05-26

## Status

- `cargo build -p civis-godot-rust` completed successfully from `clients/godot-ref/rust`.
- The GDExtension DLL is present at `clients/godot-ref/rust/target/debug/civis_godot_rust.dll`.
- `godot --path clients/godot-ref --headless` loads the project and initializes the Rust extension successfully.

## Notes

- The workspace build was initially blocked by a stale call in `crates/mod-host/src/lib.rs` to `remember_reload_root`, which no longer existed on `ModHost`.
- That call was removed so the workspace could compile far enough to verify the Godot client package.

## Launch Instructions

1. Open Godot 4.6.3.
2. Choose `Import` or `Open` and select `C:\Users\koosh\Dev\Civis\clients\godot-ref\project.godot`.
3. Confirm the project is using the Rust extension from `civis.gdextension`.
4. Press `Play` or use `F5`.
5. The main scene is `res://scenes/main.tscn`.

## Extension Paths

- `clients/godot-ref/civis.gdextension`
  - `windows.debug.x86_64 = "res://rust/target/debug/civis_godot_rust.dll"`
  - `windows.release.x86_64 = "res://rust/target/release/civis_godot_rust.dll"`
- `clients/godot-ref/project.godot`
  - `run/main_scene="res://scenes/main.tscn"`
