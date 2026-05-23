# Civis Unreal 5 visual showcase

## Open question

Ship under the standard Epic EULA, which carries royalty surface, or constrain this to a shell-only renderer with no game logic in UE5 so the royalty surface stays minimized. That decision should be made before the MVP hardens.

## Run instructions

1. Install Unreal Engine 5.4
2. `cd clients/unreal-show/Source/Civis/rust-shim && cargo build --release`
3. Right-click `CivShow.uproject` -> Generate Visual Studio project files
4. Open `CivShow.sln` in Visual Studio, build `Development Editor | Win64`
5. Open `CivShow.uproject` in Unreal Editor
6. Press Play. The client connects to `civ-watch` on `http://127.0.0.1:9090` and renders the live terrain

## Layout

```
clients/unreal-show/
├── CivShow.uproject
├── Content/
├── README.md
└── Source/
    ├── CivShow/
    │   ├── CivShow.Build.cs
    │   ├── CivShowGameMode.cpp
    │   ├── CivShowGameMode.h
    │   ├── CivProtocolClient.cpp
    │   ├── CivProtocolClient.h
    │   ├── CivilianActor.cpp
    │   ├── CivilianActor.h
    │   ├── VoxelTerrain.cpp
    │   └── VoxelTerrain.h
    └── Civis/
        ├── include/
        │   └── civis_ffi.h
        └── rust-shim/
            ├── Cargo.toml
            ├── build.rs
            ├── cbindgen.toml
            └── src/
                └── lib.rs
```

## Notes

- Unreal is the visual showcase client and stays on a lighter iteration cadence than the Bevy and Godot references.
- The runtime HTTP surface matches `civ-watch`: `/terrain`, `/snapshot`, and `/control/*`.
- The Rust shim is detached from the workspace so the Unreal build can consume a standalone MSVC static library.
