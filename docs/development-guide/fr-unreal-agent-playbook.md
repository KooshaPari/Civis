# FR-CIV-UE-AGENT — Unreal showcase: agent playbook

**Client:** `clients/unreal-show`
**Engine:** UE **5.7** on this machine (`C:\Program Files\Epic Games\UE_5.7`); `CivShow.uproject` `EngineAssociation` updated to `5.7`. `build.ps1` also accepts 5.4/5.6 if installed.
**Role:** L5 visual showcase only — sim authority stays on `civ-server` / `civ-watch`

---

## While UE is downloading (agent-safe work)

These do **not** need the Editor or UBT:

| Task | Command / path |
|------|----------------|
| Offline preflight | `.\clients\unreal-show\scripts\verify-unreal-ready.ps1` |
| Rust FFI shim | `.\clients\unreal-show\scripts\build.ps1 -SkipUe` |
| UBT scaffolding | `CivShow.cpp`, `CivShow.Target.cs`, `CivShowEditor.Target.cs` (no `-projectfiles` wait) |
| C++ / protocol edits | `Source/CivShow/*.cpp`, `CivWsClient`, `CivProtocolClient` |
| L5 parity (landed) | Job colors (`CivisJobColors.h`), terrain foot height (`VoxelTerrain::SampleWorldHeightAtNorm`), cylinder civilians |
| Parity with Godot | Compare `civis_ws_client.gd` ↔ `CivWsClient.cpp` method list |
| Backend smoke | `cargo test -p civ-server --test ws_smoke` |
| Docs / asset plan | This file + `fr-l5-visual-pass.md`; `Content/.gitignore` for Megascans |

**Avoid** until UE is on disk: generating `.sln`, cooking content, Quixel import, Play-In-Editor.

---

## What agents can vs cannot do

| Can | Cannot (needs you) |
|-----|---------------------|
| Run `setup.exe modify --quiet` for MSVC workload | Click through **UAC** / first-time Epic login on your behalf |
| Open VS Installer + docs in browser (`open-vs-installer.ps1`) | Complete a **GUI installer** already open (singleton lock) |
| `winget install` Build Tools (may prompt admin) | Accept Epic/Unreal EULAs in a modal |
| Build rust-shim, edit all C++ before UBT | Fully unattended VS modify if another installer instance is running |

**This machine (2026-05-25):** UE **5.7** at `C:\Program Files\Epic Games\UE_5.7`. VS **Community 2022** has **no** `VC\Tools\MSVC` yet. VS **Preview** has only **banned** 14.42/14.43 toolchains — UBT rejects them. Fix: add **Desktop development with C++** + **MSVC v14.44-17.14** to **Community** (not Preview).

## When download finishes (automated path)

1. Confirm install folder exists, e.g. `C:\Program Files\Epic Games\UE_5.4`.
2. Optional one-time user env (PowerShell profile or CI secret):

   ```powershell
   $env:UE_ROOT = "C:\Program Files\Epic Games\UE_5.4"
   ```

3. Poll until ready, then full build:

   ```powershell
   .\clients\unreal-show\scripts\wait-for-ue.ps1
   # or manually:
   .\clients\unreal-show\scripts\build.ps1
   ```

4. **Exit codes:** `0` = UBT succeeded; `2` = UE still missing; `1` = compile error (fix C++/Build.cs).

5. Run automated PIE prep: `.\scripts\pie-validation.ps1` (backends + WS/terrain smoke + checklist below).
6. Run the **Play in Editor** checklist below (requires `civ-server` + `civ-watch`).

---

## Play in Editor (after `build.ps1` exit `0`)

Prerequisites: `cargo run -p civ-server` and `cargo run -p civ-watch` in separate terminals (same ports as [`client-attach-matrix.md`](../guides/client-attach-matrix.md)).

| Step | Action | Pass criterion |
|------|--------|----------------|
| 1 | Double-click or UE Editor → **Open** `clients/unreal-show/CivShow.uproject` | Project loads without compile errors (first open may compile shaders) |
| 2 | **Edit → Project Settings → Maps & Modes** — default **Game Mode** = `CivShowGameMode` (or set on World Settings) | `ACivShowGameMode` drives attach on Play |
| 3 | Press **Play (PIE)** | No crash; terrain mesh appears from civ-watch HTTP |
| 4 | Confirm **WS attach** — Output Log or debugger: `UCivWsClient` connected to `ws://127.0.0.1:3000/ws?tick_format=binary` | `sim.snapshot` responses arrive; civilians sync from `civ_pins` |
| 5 | **Pins** — spawn on server (`cargo test -p civ-server --test ws_smoke ws_jsonrpc_spawn_civilian_pin_appears_in_snapshot` or Godot/web spawn) | Cylinder civilians appear/update at normalized positions |
| 6 | **Day/night** — toggle sim day phase (server tick / policy) or wait for `is_day` flip | Directional light + ambient follow `ApplyDayNight` (`is_day` from snapshot JSON) |
| 7 | **Job colors** — spawn civilians with `Citizen.job` set (when engine exposes job on pins) | Mesh tint uses `CivisJobColors.h` mapping; today `civ_pins.job` may be `null` — verify tint path does not crash |

**Quick smoke:** with both backends running, PIE should show heightmap terrain within ~1 s and at least one civilian pin after a spawn RPC from another client or `ws_smoke`.

**F3D0:** `VoxelDelta` frames show coarse chunk box markers (same idea as Godot `F3d0VoxelOverlay`). Full 16³ mesh remains Bevy-only.

**Out of scope:** minimap (attach matrix UX-04).

---

## What agents can automate vs cannot

| Automatable | Hard / manual |
|-------------|----------------|
| `build.ps1` / `wait-for-ue.ps1` on a machine **with** UE installed | GitHub `windows-latest` (no UE; workflow is informational only) |
| Rust shim + copy `.lib` | First Editor open (shader compile, plugin consent) |
| UBT `CivShowEditor Win64 Development` | Pixel-perfect L5 art sign-off |
| JSON-RPC / HTTP parity in C++ | Quixel asset *selection* (creative) |

ADR-007: Unreal is **hostile to full agent loop** (no headless PIE in CI like Bevy). Use **Bevy + Godot** for agent screenshots; use **Unreal** for visual bar validation on a dev box.

---

## Epic ecosystem: what helps Civis?

### High value (L5 showcase)

| Product | Use for Civis | Agent notes |
|---------|---------------|-------------|
| **UE 5.4 + Lumen** | Runtime GI on terrain + actors | Built-in; enable in project settings after first open |
| **Nanite** | Dense static environment (rocks, cliffs) | Optional; our terrain is procedural mesh today, not Nanite-first |
| **Quixel / Fab (Megascans)** | Ground materials, rocks, foliage, debris | Import via **Bridge** or Fab → `Content/Materials/`, `Content/Megascans/`; assign to `VoxelTerrain` / `CivilianActor` master materials |
| **Niagara** | Spawn/damage bursts (parity with Godot/web) | Plugin already enabled in `CivShow.uproject` |
| **ProceduralMeshComponent** | Heightmap terrain | Already in `CivShow.Build.cs` |

### Medium value (later)

| Product | Use |
|---------|-----|
| **Starter Content / City kits** | Placeholder buildings until sim `buildings[]` meshes |
| **Chaos** | `sim.damage` crater/debris (tactics spec) |
| **Enhanced Input** | Camera orbit presets (FR-CIV-UX-005 parity) |

### Low value / skip for now

| Product | Why skip |
|---------|----------|
| **MetaHuman** | We sync lightweight `civ_pins`, not cinematic characters |
| **Motion Matching / Animator** | No locomotion auth in UE client |
| **Pixel Streaming** | Ops complexity; web already has L2 spectator |
| **Verse / UEFN** | Wrong product surface |

**Quixel rule of thumb:** one **landscape master material** (grass/rock blend) + 2–3 **Nanite rocks** + **foliage** cards = big L5 jump for ~1–2 hours of artist time; agents can wire **material slots** in C++, artists assign textures.

---

## Suggested asset folders (gitignored binaries)

```
Content/
  Materials/          # Master M_* / MI_* (job tint params for civilians)
  Megascans/          # Quixel imports (local only, .gitignore)
  VFX/                # Niagara spawn/damage
  Maps/               # Persistent level + lighting rig
```

Keep **no large binaries in git**; document paths in README only.

---

## Agent checklist after UE install

- [ ] `build.ps1` exit `0`
- [ ] **Play in Editor** checklist (terrain HTTP, WS `civ_pins`, `is_day`, job tint path)
- [ ] Optional: one Quixel ground material on `AVoxelTerrain`
- [ ] Optional: Niagara burst on `SpawnEntity` HTTP/WS ack

---

## Related

- `clients/unreal-show/README.md` — build flags
- `docs/development-guide/fr-l5-visual-pass.md` — presentation slices
- `docs/adr/ADR-007-three-renderers.md` — why Unreal cadence is lighter
