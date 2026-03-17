# DINOForge — QA Matrix

> **Last updated**: 2026-03-15
> **Status legend**: ✅ exists · 🆕 new (this doc) · ⚠️ partial

---

## Tier Pyramid

```
           ┌──────────────────────────────┐
           │   P2 · Game Launch / E2E     │  live game process, bridge
           │   (game-launch.yml — manual) │
           ├──────────────────────────────┤
           │   P1 · UI Automation         │  FlaUI (Companion) · UiSelectorEngine (overlay)
           │   (ui-automation.yml)        │
           ├──────────────────────────────┤
           │   P1 · Integration           │  FakeBridge, mock ECS, real FS
           │   (ci.yml — every PR)        │
           ├──────────────────────────────┤
           │   P0 · Unit / Arch / Schema  │  pure .NET, no game dep
           │   (ci.yml — every PR)        │
           └──────────────────────────────┘
```

### CI Workflows

| Workflow | Trigger | Tier(s) Covered |
|----------|---------|-----------------|
| `ci.yml` | push/PR → main | P0 unit, P0 arch, P1 integration |
| `validate-packs.yml` | push packs/ or schemas/ | Pack schema + CLI validation |
| `fuzz.yml` | nightly 02:00 UTC | P2 property / fuzz |
| `mutation-test.yml` | weekly Mon 06:00 UTC | SDK mutation (Stryker.NET) |
| `benchmarks.yml` | on demand | P1 performance |
| `ui-automation.yml` | manual + weekly | P1 companion UI automation (FlaUI) |
| `game-launch.yml` | manual + weekly (self-hosted) | P2 game launch + overlay automation |

---

## Master Matrix

### P0 — SDK / Pack System

| ID | Scenario | Type | File | Pass Criteria | Status |
|----|----------|------|------|---------------|--------|
| SDK-001 | Valid manifest loads all registries | unit/bdd | `BddSpecs.cs` | `LoadedPacks.Count == loaded` | ✅ |
| SDK-002 | Missing manifest returns error, no throw | unit | `ContentLoaderTests.cs` | `LoadPack_MissingManifest_Fails` | ✅ |
| SDK-003 | Circular dependency detected | unit | `DependencyResolverTests.cs` | `DependencyException` with full chain | ✅ |
| SDK-004 | Missing dependency detected | unit | `DependencyResolverTests.cs` | Error names missing pack | ✅ |
| SDK-005 | Load order respects dependency graph | unit | `ContentLoaderTests.cs` | `WithLoadOrder_LoadsInCorrectOrder` | ✅ |
| SDK-006 | Registry: register + retrieve by ID | unit | `RegistryTests.cs` | Exact object returned | ✅ |
| SDK-007 | Registry: duplicate raises conflict | unit | `RegistryTests.cs` | Conflict event fired | ✅ |
| SDK-008 | All pack YAML validates against schema | schema | `SchemaValidationTests.cs` + `validate-packs.yml` | Zero violations | ✅ |
| SDK-009 | Conflicting packs both refused | unit | `CompatibilityCheckerTests.cs` | Conflict reason in result | ✅ |
| SDK-010 | Corrupt YAML fails gracefully | unit | `ContentLoaderEdgeCaseTests.cs` | Error returned, no unhandled exception | ✅ |

### P0 — Architecture Enforcement

| ID | Scenario | Type | File | Pass Criteria | Status |
|----|----------|------|------|---------------|--------|
| ARCH-001 | SDK has no dependency on Runtime | arch | `ArchitectureTests.cs` | NetArchTest passes | ✅ |
| ARCH-002 | SDK has no dependency on Domains | arch | `ArchitectureTests.cs` | NetArchTest passes | ✅ |
| ARCH-003 | Public interfaces reside in SDK namespace | arch | `ArchitectureTests.cs` | NetArchTest passes | ✅ |

### P0 — Domain: Warfare

| ID | Scenario | Type | File | Pass Criteria | Status |
|----|----------|------|------|---------------|--------|
| DOM-001 | All canonical archetypes resolve | unit | `WarfareTests.cs` | ≥ 20 archetype IDs present | ✅ |
| DOM-002 | Clone Trooper ≠ ARC Trooper archetype | unit | `WarfareTests.cs` | Different `BaseStats` records | ✅ |
| DOM-003 | IndustrialSwarm has correct modifiers | unit | `WarfareTests.cs` | Modifier values match spec | ✅ |
| DOM-004 | Wave definitions reference valid squad IDs | unit | `SkillWaveSquadTests.cs` | All squad IDs resolvable | ✅ |
| DOM-005 | Doctrine modifiers are non-zero | unit | `WarfareTests.cs` | At least one modifier ≠ 1.0 | ✅ |

### P0 — Bridge Protocol (Offline)

| ID | Scenario | Type | File | Pass Criteria | Status |
|----|----------|------|------|---------------|--------|
| BRG-001 | 6-step offline round-trip (FakeGameBridge) | integration | `BridgeRoundTripTests.cs` | All steps pass: load→query→override→read→reload | ✅ |
| BRG-002 | Ping returns healthy | integration | `PingTests.cs` | `result.Healthy == true` | ✅ |
| BRG-003 | ComponentMap resolves 30+ DINO types | integration | `ComponentMapTests.cs` | All mappings non-null | ✅ |
| BRG-004 | StatModifier applies HP override | integration | `StatTests.cs` | Entity HP == override value | ✅ |
| BRG-005 | Resource delivery system integration | integration | `ResourceTests.cs` | Resource count matches delivery | ✅ |

### P0 — Asset Pipeline

| ID | Scenario | Type | File | Pass Criteria | Status |
|----|----------|------|------|---------------|--------|
| AST-001 | AddressablesCatalog loads valid catalog | integration | `CatalogTests.cs` | Bundle paths resolve to files | ✅ |
| AST-002 | Bundle path placeholder replaced with StreamingAssets | unit | `AddressablesCatalogTests.cs` | Path starts with StreamingAssets | ✅ |
| AST-003 | Non-existent catalog path throws `FileNotFoundException` | unit | `AddressablesCatalogTests.cs` | Correct exception type | ✅ |
| AST-004 | AssetSwapRegistry de-duplicates same address | unit | `AssetSwapRegistryTests.cs` | Only one patch written | ✅ |
| AST-005 | Patched bundle differs from source | integration | `AssetSwapRegistryTests.cs` | Byte-level diff detected | ✅ |
| AST-006 | `ReadCatalog()` failure in AssetSwapSystem is caught | unit | `AssetSwapRegistryTests.cs` | Phase 2 still executes | ✅ |

---

### P1 — Integration (CI gate)

| ID | Scenario | Type | File | Pass Criteria | Status |
|----|----------|------|------|---------------|--------|
| INT-001 | Full pack loading against mock ECS | integration | `PackLoadingTests.cs` | Pack types registered in mock world | ✅ |
| INT-002 | JSON-RPC bridge round-trip (in-process) | integration | `BridgeRoundTripTests.cs` | Protocol framing correct | ✅ |
| INT-003 | Hot reload fires within 5s on file change | integration | `HotReloadTests.cs` | Event ≤ 5000 ms | ✅ |
| INT-004 | Hot reload on invalid YAML keeps old state | integration | `HotReloadTests.cs` | Old pack still loaded, error logged | ✅ |
| INT-005 | PackCompiler CLI `validate` exits 0 for valid packs | cli-integration | `validate-packs.yml` | Exit code 0 | ✅ |
| INT-006 | PackCompiler CLI `validate` exits non-zero for invalid pack | cli-integration | **new** `PackCompilerCliTests.cs` | Exit code ≠ 0, message contains path | 🆕 |
| INT-007 | Bridge latency < 50ms (FakeBridge, 1000 req) | perf | **new** `BridgeLatencyTests.cs` | P99 < 50 ms | 🆕 |

### P1 — Performance (benchmarks.yml)

| ID | Scenario | Type | File | Pass Criteria | Status |
|----|----------|------|------|---------------|--------|
| PERF-001 | Asset import < 5s/model | perf | `PerformanceBenchmarkTests.cs` | P99 < 5 000 ms | ✅ |
| PERF-002 | Full 9-model pipeline < 5 min | perf | `PerformanceBenchmarkTests.cs` | Total < 300 s | ✅ |
| PERF-003 | Bridge round-trip P99 < 50 ms | perf | **new** `BridgeLatencyTests.cs` | P99 < 50 ms | 🆕 |
| PERF-004 | AssetSwapSystem phase 1 completes before frame 5 | perf | **new** `AssetSwapLatencyTests.cs` | Patch exists by frame 5 mock | 🆕 |

### P1 — UI: Desktop Companion (ViewModel unit)

| ID | Scenario | Type | File | Pass Criteria | Status |
|----|----------|------|------|---------------|--------|
| COMP-001 | Pack list loads from data service | unit | `CompanionTests/ViewModelTests.cs` | Observable collection has items | ✅ |
| COMP-002 | Toggle updates `IsEnabled` state | unit | `CompanionTests/ViewModelTests.cs` | State flips on toggle | ✅ |
| COMP-003 | Error pack shows error badge state | unit | `CompanionTests/ViewModelTests.cs` | `HasErrors=true` | ✅ |
| COMP-004 | Status text correct when no errors | unit | `CompanionTests/ViewModelTests.cs` | `"All N pack(s) loaded OK"` | ✅ |
| COMP-005 | Disabled pack service persists across restart | unit | `CompanionTests/DisabledPacksServiceTests.cs` | Pack IDs survive round-trip | ✅ |

### P1 — UI Automation: Desktop Companion (FlaUI — ui-automation.yml, Windows)

> **Tooling**: `FlaUI.Core` + `FlaUI.UIA3` (Windows Automation API)
> **Runner**: `windows-latest` GitHub Actions
> **Category trait**: `[Trait("Category", "UiAutomation")]`
> **Project**: `src/Tests/UiAutomation/DINOForge.Tests.UiAutomation.csproj`

| ID | Scenario | Type | File | Pass Criteria | Status |
|----|----------|------|------|---------------|--------|
| COMP-UI-001 | Main window launches and is visible | ui-auto-companion | **new** `CompanionLaunchTests.cs` | FlaUI finds window by `AutomationId="MainWindow"` | 🆕 |
| COMP-UI-002 | Pack list ListView shows ≥ 1 item | ui-auto-companion | **new** `CompanionPackListTests.cs` | `ListView.Items.Count ≥ 1` | 🆕 |
| COMP-UI-003 | Toggle pack changes service state | ui-auto-companion | **new** `CompanionPackToggleTests.cs` | FlaUI `ToggleButton.Toggle()` → service `IsEnabled` flips | 🆕 |
| COMP-UI-004 | Settings page saves game path | ui-auto-companion | **new** `CompanionSettingsTests.cs` | FlaUI edit `TextBox(AutoId="GamePathInput")` → settings file updated | 🆕 |
| COMP-UI-005 | Status bar shows "Not connected" when bridge absent | ui-auto-companion | **new** `CompanionStatusBarTests.cs` | `StatusBar.Text` matches pattern | 🆕 |
| COMP-UI-006 | Keyboard shortcut Ctrl+R triggers reload | ui-auto-companion | **new** `CompanionShortcutTests.cs` | Reload event fired after key send | 🆕 |

### P1 — UI Automation: In-Game Overlay (UiSelectorEngine — game-launch.yml)

> **Tooling**: BepInEx in-game test plugin + `UiSelectorEngine` + `UiActionTrace`
> **Execution**: plugin runs inside game process, results exported via `UiActionTrace.SaveToFile()`
> **Category trait**: `[Trait("Category", "GameLaunch")]` on the bridge-side assertions
> **Project**: `src/Tests/GameLaunch/` (bridge-side) + `src/Runtime/Tests/` (in-game plugin side)

| ID | Scenario | Type | File | Pass Criteria | Status |
|----|----------|------|------|---------------|--------|
| OVL-001 | F10 keypress opens overlay | ui-auto-overlay | **new** `OverlayToggleTests.cs` | `ModMenuOverlay.IsVisible == true` | 🆕 |
| OVL-002 | Second F10 closes overlay | ui-auto-overlay | **new** `OverlayToggleTests.cs` | `ModMenuOverlay.IsVisible == false` | 🆕 |
| OVL-003 | Pack list shows all loaded packs | ui-auto-overlay | **new** `OverlayPackListTests.cs` | `UiSelectorEngine.Query("pack-list").Nodes.Count == loadedPacks.Count` | 🆕 |
| OVL-004 | Clicking pack toggle fires `OnPackToggled` | ui-auto-overlay | **new** `OverlayPackToggleTests.cs` | Callback invoked with correct pack ID + `false` | 🆕 |
| OVL-005 | Debug panel shows numeric entity count | ui-auto-overlay | **new** `DebugPanelTests.cs` | `UiSelectorEngine.Query("debug-entity-count").Text` matches `\d+` | 🆕 |
| OVL-006 | HUD indicator shows mod-active badge | ui-auto-overlay | **new** `HudIndicatorTests.cs` | `UiSelectorEngine.Query("hud-mod-active").IsVisible == true` | 🆕 |
| OVL-007 | UiActionTrace exports valid JSON | ui-auto-overlay | **new** `UiActionTraceTests.cs` (unit, no Unity) | `GetHistory().Count ≥ 1`, exported JSON parses | 🆕 |
| OVL-008 | Settings panel saves preference and survives reload | ui-auto-overlay | **new** `OverlaySettingsTests.cs` | Preference file updated, value persists after `ReloadPacks()` | 🆕 |

---

### P2 — Game Launch / End-to-End (game-launch.yml — self-hosted, game installed)

> **Tooling**: `GameClient` (JSON-RPC bridge) launched from `GameLaunchFixture`
> **Runner**: self-hosted Windows runner with DINO installed at `DINO_GAME_PATH`
> **Category trait**: `[Trait("Category", "GameLaunch")]`
> **Project**: `src/Tests/GameLaunch/DINOForge.Tests.GameLaunch.csproj`
> **Timeout**: per-test 120s, collection fixture timeout 300s

| ID | Scenario | Type | File | Pass Criteria | Status |
|----|----------|------|------|---------------|--------|
| GL-001 | BepInEx bootstraps DINOForge plugin | game-launch | **new** `GameLaunchSmokeTests.cs` | Bridge `PingAsync()` responds `Healthy=true` within 30s | 🆕 |
| GL-002 | `warfare-starwars` loads 28 units in live catalog | game-launch | **new** `GameLaunchPackTests.cs` | `GetCatalog()` totalUnits == 28 | 🆕 |
| GL-003 | Phase 1: bundle patched to disk before entity load | game-launch | **new** `GameLaunchAssetSwapTests.cs` | Patched bundle file exists by frame 5 (queried via bridge) | 🆕 |
| GL-004 | Phase 2: RenderMesh.mesh swapped on clone trooper | game-launch | **new** `GameLaunchAssetSwapTests.cs` | `QueryUnits("rep_clone_trooper")[0].MeshId == swappedMeshId` | 🆕 |
| GL-005 | HP override persists after `ReloadPacks()` | game-launch | **new** `GameLaunchStatTests.cs` | `ReadStat(entityId, "hp") == 999` post-reload | 🆕 |
| GL-006 | F10 overlay opens in live game (MCP `overlay_status`) | game-launch | **new** `GameLaunchUiTests.cs` | Bridge tool `overlay_status` returns `{"visible":true}` | 🆕 |
| GL-007 | Hot reload: pack YAML change reloads within 5s | game-launch | **new** `GameLaunchHotReloadTests.cs` | Bridge `GetStatus()` reports new pack version within 5s | 🆕 |
| GL-008 | Economy pack changes resource rate in live game | game-launch | **new** `GameLaunchEconomyTests.cs` | `ReadStat(resourceEntity,"rate")` matches pack definition | 🆕 |

### P2 — Property / Fuzz (fuzz.yml — nightly)

| ID | Scenario | Type | File | Pass Criteria | Status |
|----|----------|------|------|---------------|--------|
| FUZZ-001 | Arbitrary YAML does not crash loader | property | `FuzzTargets/PackManifest.cs` | No unhandled exception | ✅ |
| FUZZ-002 | Malformed JSON-RPC does not crash bridge | property | `FuzzTargets/Json.cs` | No unhandled exception | ✅ |
| FUZZ-003 | Version strings parse without throws | property | `FuzzTargets/Semver.cs` | No unhandled exception | ✅ |
| FUZZ-004 | Corrupt asset catalog fails gracefully | property | **new** `FuzzTargets/AssetCatalog.cs` | `FileParseException`, not unhandled | 🆕 |
| FUZZ-005 | Schema validator rejects all invalid docs | property | **new** `FuzzTargets/SchemaValidation.cs` | No valid doc incorrectly rejected | 🆕 |

---

## New Tests: Implementation Summary

### `src/Tests/GameLaunch/` — Game Launch Project

```
DINOForge.Tests.GameLaunch.csproj    ← net11.0, xunit, FluentAssertions, GameClient ref
GameLaunchFixture.cs                  ← IAsyncLifetime: launch process, await bridge ping
GameLaunchSmokeTests.cs               ← GL-001
GameLaunchPackTests.cs                ← GL-002
GameLaunchAssetSwapTests.cs           ← GL-003, GL-004
GameLaunchStatTests.cs                ← GL-005
GameLaunchUiTests.cs                  ← GL-006
GameLaunchHotReloadTests.cs           ← GL-007
GameLaunchEconomyTests.cs             ← GL-008
```

### `src/Tests/UiAutomation/` — Desktop Companion FlaUI Project

```
DINOForge.Tests.UiAutomation.csproj  ← net11.0-windows10.0.26100.0, FlaUI.Core, FlaUI.UIA3
CompanionFixture.cs                   ← IAsyncLifetime: launch companion, find MainWindow
CompanionLaunchTests.cs               ← COMP-UI-001
CompanionPackListTests.cs             ← COMP-UI-002
CompanionPackToggleTests.cs           ← COMP-UI-003
CompanionSettingsTests.cs             ← COMP-UI-004
CompanionStatusBarTests.cs            ← COMP-UI-005
CompanionShortcutTests.cs             ← COMP-UI-006
```

### In-Game Overlay (runs inside game via BepInEx test plugin)

> Overlay tests (OVL-001…008) execute inside the game process.
> The BepInEx plugin (`DINOForge.Tests.InGame`) uses `UiSelectorEngine` + `UiActionTrace` to
> drive and record overlay interactions, then writes results to
> `BepInEx/logs/dinoforge_ui_test_results.json`.
> The `game-launch.yml` workflow reads that file and asserts pass/fail.

### New CI Workflows

**`ui-automation.yml`** — Windows runner, FlaUI companion tests:
```yaml
on: [workflow_dispatch, schedule: "0 4 * * 1"]  # weekly Monday
runs-on: windows-latest
filter: dotnet test --filter "Category=UiAutomation"
```

**`game-launch.yml`** — self-hosted runner, live game:
```yaml
on: [workflow_dispatch, schedule: "0 5 * * 1"]  # weekly Monday
runs-on: [self-hosted, windows, dino-installed]
env: DINO_GAME_PATH, DINO_BEPINEX_PATH
filter: dotnet test --filter "Category=GameLaunch"
```

---

## Coverage Targets

| Layer | Current | Target | Gap |
|-------|---------|--------|-----|
| SDK (unit) | ~65% | 80% | INT-006, fuzz targets |
| Bridge Protocol | ~70% | 85% | BRG latency tests |
| Asset Swap | ~55% | 75% | GL-003/004, AST-006 |
| Overlay UI logic | 0% | 60% | OVL-001…008 (game-launch) |
| Desktop Companion UI | ~45% | 70% | COMP-UI-001…006 (FlaUI) |
| E2E / Game Launch | 0% | — (not in coverage gate) | GL-001…008 |
