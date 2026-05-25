# Civis Unreal 5 visual showcase

## EULA decision (resolved 2026-05-23)

Ships under the **standard Epic EULA** with full game logic in UE5.
Royalty surface accepted per owner decision.

## Target tier

**L1 observer → L5 visuals** per [product-quality-ladder.md](../../docs/roadmap/product-quality-ladder.md).
Gameplay authority stays on `civ-server`; this client prioritizes presentation.

## Automated build

From the repo root (or `clients/unreal-show`):

```powershell
.\clients\unreal-show\scripts\build.ps1
```

On Git Bash / WSL calling into Windows PowerShell:

```bash
./clients/unreal-show/scripts/build.sh
```

The script:

1. Builds `Source/Civis/rust-shim` with `cargo build --release` and copies `civis_unreal_ffi.lib` into `Source/Civis/lib/`.
2. Locates Unreal Engine **5.4** (see detection below).
3. Runs `UnrealBuildTool` / `Build.bat` for **`CivShowEditor Win64 Development`** (generates `Target.cs` / `.sln` on first run via `-projectfiles` if missing).

| Exit code | Meaning |
|-----------|---------|
| `0` | Rust shim and UE editor target built |
| `1` | `cargo`, copy, or UBT failure |
| `2` | UE 5.4 not installed (rust shim may still have built) |

**UE detection order:** `UE_ROOT` env → `C:\Program Files\Epic Games\UE_{5.7,5.4,...}` → Epic Launcher manifest.

**Toolchain:** UE 5.7 requires Visual Studio 2022 **17.8+** with MSVC **14.44.35207** (UBT rejects 14.40–14.43). Install via VS Installer → *MSVC v143 … (v14.44-17.14)*.

Flags: `-SkipRust`, `-SkipUe`, `-Configuration DebugGame|Shipping`.

**CI:** `.github/workflows/unreal-build.yml` runs the script on `windows-latest` with `continue-on-error: true`. Hosted runners do not include UE; exit `2` is expected until a self-hosted runner with UE is wired.

**While UE downloads:** run `build.ps1 -SkipUe` (rust shim only). Agent playbook: [`docs/development-guide/fr-unreal-agent-playbook.md`](../../docs/development-guide/fr-unreal-agent-playbook.md).

**While downloading:** `.\scripts\verify-unreal-ready.ps1` — checks Target.cs, module stub, rust `.lib`, runs `build.ps1 -SkipUe`.

**After install:** `.\scripts\wait-for-ue.ps1` polls `detect-ue.ps1` every 2 minutes, then runs a full build. Probe only: `.\scripts\detect-ue.ps1` (exit `0` = found).

**Materials / Nanite (L5):** artist-owned; not required for compile. **Quixel / Fab (Megascans)** — high value for ground materials and rocks on `VoxelTerrain`; import to `Content/Megascans/` (gitignore binaries). See agent playbook for Niagara / MetaHuman / etc.

## Run instructions (manual / editor)

1. Install Unreal Engine **5.7** (or 5.4; `build.ps1` auto-detects `UE_*` under Epic Games)
2. Run `.\scripts\build.ps1` (or build rust-shim manually: `cd Source/Civis/rust-shim && cargo build --release`)
3. Open `CivShow.uproject` in Unreal Editor (or use generated `CivShow.sln` after the automated build)
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
├── scripts/
│   ├── build.ps1
│   └── build.sh
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
