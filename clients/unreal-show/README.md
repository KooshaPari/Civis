# Civis Unreal 5 visual showcase

## EULA decision (resolved 2026-05-23)

Ships under the **standard Epic EULA** with full game logic in UE5.
Royalty surface accepted per owner decision.

## Target tier

**L1 observer → L5 visuals** per [product-quality-ladder.md](../../docs/roadmap/product-quality-ladder.md).
Gameplay authority stays on `civ-server`; this client prioritizes presentation.

## Run instructions

1. Install Unreal Engine 5.4
2. `cd clients/unreal-show/Source/Civis/rust-shim && cargo build --release`
3. Right-click `CivShow.uproject` → Generate Visual Studio project files
4. Open `CivShow.sln` in Visual Studio, build `Development Editor | Win64`
5. Open `CivShow.uproject` in Unreal Editor
6. Start backends (recommended dual attach, same as Godot):
   ```bash
   cargo run -p civ-server    # :3000 JSON-RPC + F3D0 (future primary)
   cargo run -p civ-watch     # :9090 terrain + snapshot HTTP (today)
   ```
7. Press Play — `CivProtocolClient` uses **civ-watch** HTTP (`/terrain`, `/snapshot`)

## Protocol parity matrix

| Capability | civ-watch HTTP | civ-server WS | Unreal today |
|------------|----------------|---------------|--------------|
| Terrain heightmap | `GET /terrain` | — | Yes |
| Snapshot pins | `GET /snapshot` | `sim.snapshot` | Yes (HTTP) |
| Speed / tick | `POST /control/speed` | `sim.set_speed` | HTTP only |
| F3D0 voxel stream | — | WS binary | Planned |
| Spawn / build | `POST /control/spawn_entity`, `place_voxel`, `damage` | `sim.spawn_entity` / `sim.place_voxel` / `sim.damage` | HTTP (`SpawnEntity`, `ApplyDamage` on `UCivProtocolClient`) |

## Dual attach (implemented)

1. **civ-watch HTTP** — terrain (`GET /terrain`), legacy controls.
2. **civ-server WS** — `UCivWsClient` JSON-RPC + F3D0 throttle (mirrors Godot `CivisWsClient`).
3. **`ACivShowGameMode`** — fetches terrain, connects WS, syncs `civ_pins` → `ACivilianActor`.

## Next integration steps

1. F3D0 voxel mesh stream (keep heightmap terrain until then).
2. Materials / Nanite polish (L5 art pass).
3. Share job colors from snapshot pins when jobs are wired on agents.

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

- Unreal is the visual showcase client and stays on a lighter iteration cadence than Godot.
- The Rust shim is detached from the workspace so the Unreal build can consume a standalone MSVC static library.
- **Unity client:** not in tree; see PRD multi-client list and quality ladder for when to add a fifth attach surface.
