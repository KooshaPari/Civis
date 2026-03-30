# DINOForge QA Validation Matrix

> **Last validated**: 2026-03-29
> **Source**: Auto-generated from live game session evidence, CI results, and runtime logs.
> This matrix documents every user-facing feature and agent/dev-facing tool in DINOForge with honest validation status.

---

## Status Legend

| Icon | Status | Meaning |
|------|--------|---------|
| ✅ | **Test-backed** | Automated tests prove it, reproducible in CI |
| ✅ | **Proven** | Verified via live game + log evidence this session |
| 🤖 | **Agent-assumed** | Agent ran it and it worked, no durable proof |
| 👤 | **Human-assumed** | Built correctly, needs human eyes |
| 🎮 | **Needs gameplay** | Requires active gameplay state (not just main menu) |
| 🪟 | **Windows-GUI only** | Needs WinUI3/FlaUI session |
| 🔧 | **Infra-dependent** | Needs Unity Editor, Blender, or specific hardware |
| ❌ | **Broken** | Known failure with identified root cause |
| ❓ | **Untested** | No evidence either way |

---

## User-Facing Features

| ID | Feature | Status | Evidence / Notes |
|----|---------|--------|------------------|
| U1 | Runtime loads in game | ✅ Proven | Log: `Loaded runtime assembly`, `Awake completed` |
| U2 | 7 packs load at startup | ✅ Proven | Bridge: `Loaded packs: 7`, log: `Successfully loaded 7 pack(s)` |
| U3 | Main menu Mods button | ❌ Broken | NativeMenuInjector injects but RuntimeDriver is destroyed before the button becomes visible; requires M13-D3 (persistent UI host) |
| U4 | F9 Debug Overlay | ❌ Broken | KeyInputSystem created but destroyed with ECS world; needs D1 ECS-based pump to survive PlayerLoop replacement |
| U5 | F10 Mod Menu | ❌ Broken | Same root cause as U4 -- KeyInputSystem lifetime tied to destroyed ECS world |
| U6 | Stat overrides (YAML) | ✅ Proven | Log: 21+ YAML overrides enqueued at startup |
| U7 | Stat override (live API) | ❌ Broken | Bridge status command times out; MainThreadDispatcher.Update() never fires, so dispatched work is never executed; needs D1 ECS pump |
| U8 | Hot reload | ❓ Untested | FileSystemWatcher and HMR signal systems exist but were not triggered during this session |
| U9 | Economy pack active | ✅ Proven | Bridge confirms `economy-balanced` loaded in pack list |
| U10 | Scenario pack active | ✅ Proven (loaded) / ❓ Untested (runtime activation) | Pack loads successfully; ScenarioRunner activation requires gameplay state |
| U11 | Asset swap (visual) | ❌ Broken | 36/36 swap failures -- catalog address mismatch (custom keys vs Unity paths) + bundles require Unity 2021.3.45f2 to build |
| U12 | Aerial/Aviation systems | ✅ Proven | 3 aviation systems (`AerialMovementSystem`, `AerialCombatSystem`, `FormationFlyingSystem`) logged OnCreate |
| U13 | PowerShell Installer | ✅ Test-backed | `eval-installer.ps1` passes all checks |
| U14 | Bash Installer | 👤 Human-assumed | Syntax validated; no Linux/macOS live run recorded |
| U15 | Installer GUI (Avalonia) | 👤 Human-assumed | Builds successfully; no GUI interaction test recorded |
| U16 | Desktop Companion (WinUI 3) | 🪟 Windows-GUI only | Builds; FlaUI tests excluded from CI due to desktop session requirement |
| U17 | VitePress docs site | ✅ Test-backed | CI deploy workflow green; site live at kooshapari.github.io/Dino |

---

## Agent/Dev-Facing Tooling

| ID | Tool / System | Status | Evidence / Notes |
|----|---------------|--------|------------------|
| D1 | `dotnet build` (full solution) | ✅ Test-backed | CI green across all runners |
| D2 | Unit tests (1,327) | ✅ Test-backed | All passing in CI |
| D3 | Integration tests (20) | ✅ Test-backed | 20 total, 3 skipped (infra-dependent) |
| D4 | Schema validation (19 schemas) | ✅ Test-backed | NJsonSchema validation in CI |
| D5 | ContentLoader pipeline | ✅ Proven | Live log shows packs parsed, validated, and registered |
| D6 | Registry system | ✅ Proven | 7 packs registered across typed registries (Units, Buildings, Factions, Weapons, etc.) |
| D7 | Dependency resolver | ✅ Test-backed | Cycle detection and semver resolution covered by unit tests |
| D8 | GameControlCli `status` | ❌ Broken | Connects to named pipe but times out -- MainThreadDispatcher dead, dispatched queries never execute |
| D9 | GameControlCli `resources` | ❌ Broken | Same root cause as D8 |
| D10 | GameControlCli `screenshot` | ✅ Proven | Path printed, PNG captured successfully via ScreenCapture.CaptureScreenshot |
| D11 | GameControlCli `help` | ✅ Proven | No crash after Spectre.Console markup fix |
| D12 | MCP server health | ✅ Proven | `/health` endpoint returns ok on port 8765 |
| D13 | MCP to GameControlCli bridge | ❌ Broken | Bridge thread aborted after ~23s; timeout cascades from MainThreadDispatcher being dead |
| D14 | PackCompiler `validate` | ✅ Test-backed | Mocked IO tests pass in CI |
| D15 | PackCompiler `assets import` | ✅ Test-backed | Mocked IO tests pass (AssimpNet integration) |
| D16 | Lefthook pre-commit | ✅ Test-backed | Fires on every commit; format + lint gates enforced |
| D17 | Lefthook pre-push | ✅ Proven | Last push passed all gates |
| D18 | Hot reload watcher (unit tests) | ✅ Test-backed | FileSystemWatcher behavior covered |
| D19 | Property/fuzz tests (33) | ✅ Test-backed | Category=Property/Fuzz, 20 corpus seeds, nightly fuzz.yml |
| D20 | ECS system creation | ✅ Proven | 12 systems logged OnCreate in live game session |
| D21 | Aviation subsystem | ✅ Proven | 3 aviation systems created and logged |
| D22 | DestroyGuard Harmony patch | ✅ Proven | Patches applied; however, native Unity destruction bypasses C# Harmony hooks |
| D23 | Bridge server singleton | ✅ Proven | SharedBridgeServer created and survives RuntimeDriver destruction |
| D24 | AssetSwapRegistry MaxRetries | ✅ Test-backed | 3 new tests confirm retry cap stops infinite loop |
| D25 | Hidden desktop isolation | 🤖 Agent-assumed | `hidden_desktop_test.ps1` exists and was run; no durable proof artifact |
| D26 | Dual-instance (TEST copy) | 🔧 Infra-dependent | Requires `Diplomacy is Not an Option_TEST` directory on disk |
| D27 | VDD virtual display driver | ❌ Not implemented | Planned future work; currently uses Win32 CreateDesktop fallback |
| D28 | CI (GitHub Actions, 20 workflows) | ✅ Test-backed | All workflows green |
| D29 | VitePress build | ✅ Test-backed | CI deploys to gh-pages |
| D30 | Lefthook `check-yaml` | ✅ Test-backed | 148 YAML files validated |

---

## Summary

| Status | Count |
|--------|-------|
| ✅ Test-backed / Proven | 22 |
| 🤖 Agent-assumed | 1 |
| 👤 Human-assumed | 2 |
| ❌ Broken (known root cause) | 8 |
| ❓ Untested | 2 |
| 🪟 Windows-GUI only | 1 |
| 🔧 Infra-dependent | 1 |

---

## Critical Path

All **8 Broken items** trace back to a single root cause:

> **`MainThreadDispatcher.Update()` never fires because DINO replaces Unity's PlayerLoop.**

The standard Unity `MonoBehaviour.Update()` loop is destroyed at frame 0. Any system that depends on per-frame callbacks via MonoBehaviour (MainThreadDispatcher, KeyInputSystem, NativeMenuInjector visibility) silently fails.

### Dependency Chain

```
M13-D1: ECS-based pump (SystemBase.OnUpdate)
  └── unblocks MainThreadDispatcher replacement
        ├── D8, D9: GameControlCli status/resources
        ├── D13: MCP bridge
        └── U7: Live stat override API
  └── unblocks KeyInputSystem survival
        ├── U4: F9 Debug Overlay
        └── U5: F10 Mod Menu
  └── unblocks NativeMenuInjector persistence
        └── U3: Main menu Mods button

U11 (Asset swap) has an independent blocker:
  └── Catalog address mismatch (custom keys vs Unity asset paths)
  └── Bundles must be built with Unity 2021.3.45f2 (infra-dependent)
```

### Resolution Priority

1. **M13-D1**: Implement ECS-based pump (`SystemBase.OnUpdate` in a Simulation system group) to replace `MainThreadDispatcher`. This single fix unblocks 7 of 8 broken items.
2. **U11**: Fix Addressables catalog key mapping and build bundles with correct Unity version. Independent of D1.

---

## Notes

- This matrix reflects the state of the `main` branch at commit `3933327`.
- Evidence was collected from BepInEx logs (`LogOutput.log`, `dinoforge_debug.log`), CI workflow results, and MCP bridge responses.
- Items marked **Proven** have log lines or screenshots from the 2026-03-29 session.
- Items marked **Test-backed** are reproducible via `dotnet test src/DINOForge.sln` on any machine with .NET 11 preview.
- The 3 skipped integration tests (D3) require game process or Unity Editor and are excluded from CI.
