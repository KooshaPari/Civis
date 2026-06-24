# Godot Ref Build Status

## Status

- `cargo build` completed successfully in `clients/godot-ref/rust`.
- Output DLL exists at `clients/godot-ref/rust/target/debug/civis_godot_rust.dll`.
- `civis.gdextension` points to the correct debug/release paths under `res://rust/target/...`.
- `project.godot` is set up for Godot `4.6` and the extension declares `compatibility_minimum = "4.6"`.

## Build Notes

- The first build attempt failed because `Godot_v4.6.3-stable_win64` was running and held `civis_godot_rust.dll` open.
- After closing the editor process, the rebuild succeeded.
- The build produced warnings about unused code in `src/ux.rs`, but no errors.

## How To Rebuild

From this directory:

```powershell
cd clients/godot-ref/rust
cargo build
```

If Windows reports `Access is denied` while linking the DLL, close the running Godot editor first and rerun the build.

## Open In Godot

1. Open `clients/godot-ref/project.godot` in Godot 4.6.3.
2. Ensure the editor is not already holding `rust/target/debug/civis_godot_rust.dll` open while rebuilding.
3. Run the project normally after the extension DLL has been rebuilt.
