# DINOForge — Work Breakdown Structure

**Version**: 1.0
**Date**: 2026-03-14
**Status**: Active
**Scope**: M8 completion, M9 Desktop Companion, M10 Fuzzing + Coverage, M11 Incomplete Code

---

## 1. WBS Overview

```
DINOForge WBS
├── 1. M8 Runtime Integration (Completion)
├── 2. M9 WinUI 3 Desktop Companion
├── 3. M10 Fuzzing Infrastructure
├── 4. M11 Test Coverage Expansion
├── 5. M11.5 Incomplete Code Remediation
└── 6. Docs + Planning (Cross-Cutting)
```

---

## 2. M8 — Runtime Integration Completion

**Goal**: End-to-end sign-off with live DINO instance.
**Status**: 90% complete — code changes done, live validation pending.
**ADR**: ADR-009

| ID | Task | Owner | Status | Notes |
|----|------|-------|--------|-------|
| 1.1 | AssetSwap entity targeting (vanilla_mapping → archetype) | Runtime | Done | AssetSwapSystem.cs updated |
| 1.2 | HUD pack counter wiring | Runtime | Done | OnHudCountsChanged callback |
| 1.3 | F9 DebugPanel layout fix | Runtime | Done | ForceRebuildLayoutImmediate |
| 1.4 | F10 ModMenuPanel layout fix | Runtime | Done | ForceRebuildLayoutImmediate |
| 1.5 | warfare-aerial pack deployment | Ops | Done | Redeployed to BepInEx |
| 1.6 | Error detail inline in F10 | Runtime | Done | First error in status string |
| 1.7 | Live DINO launch: all 9 packs load | QA | Pending | Gate for M8 complete |
| 1.8 | Live DINO launch: HUD shows correct counts | QA | Pending | |
| 1.9 | Live DINO launch: F9/F10 panels render correctly | QA | Pending | |
| 1.10 | Live DINO launch: asset swap applies to correct entity type | QA | Pending | |

**Deliverable**: M8 marked complete in `docs/roadmap/index.md`, CHANGELOG v0.11.0.

---

## 3. M9 — WinUI 3 Desktop Companion

**Goal**: Out-of-game WinUI 3 companion app mirroring F9/F10 functionality.
**ADR**: ADR-011
**Worktree**: `src/Tools/DesktopCompanion/`
**Target framework**: `net8.0-windows`

### 3.1 Phase 0 — Project Scaffold

| ID | Task | Status |
|----|------|--------|
| 2.1.1 | Create git worktree `feature/desktop-companion` | Pending |
| 2.1.2 | Create `src/Tools/DesktopCompanion/DesktopCompanion.csproj` | Pending |
| 2.1.3 | Add to solution / standalone build | Pending |
| 2.1.4 | NuGet: Microsoft.WindowsAppSDK 1.6, CommunityToolkit.Mvvm, CommunityToolkit.WinUI | Pending |
| 2.1.5 | NuGet: Microsoft.Extensions.Hosting, YamlDotNet, Newtonsoft.Json | Pending |
| 2.1.6 | Verify AssetsTools.NET native DLL sidecar for publish | Pending |

### 3.2 Phase 1 — Data Layer

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 2.2.1 | `PackViewModel` DTO (id, name, version, enabled, errors) | Pending | No Unity dep |
| 2.2.2 | `LoadResultViewModel` DTO (loaded count, error count, errors list) | Pending | |
| 2.2.3 | `DebugSectionViewModel` DTO (section name, content lines) | Pending | |
| 2.2.4 | `IPackDataService` interface (LoadPacks, GetDebugSections) | Pending | |
| 2.2.5 | `FileSystemPackDataService` impl — read packs/ directory | Pending | |
| 2.2.6 | `DisabledPacksService` — read/write disabled_packs.json (parity with game) | Pending | |
| 2.2.7 | `PackManifestReader` — YamlDotNet wrapper around SDK PackManifest | Pending | SDK ref OK |

### 3.3 Phase 2 — DI Setup

| ID | Task | Status |
|----|------|--------|
| 2.3.1 | `HostBuilderExtensions.cs` — register IPackDataService, DisabledPacksService | Pending |
| 2.3.2 | `IHostedService` wrapper for `PackFileWatcher` (500ms debounce) | Pending |
| 2.3.3 | Verify SDK (`netstandard2.0`) references from `net8.0-windows` compile | Pending |

### 3.4 Phase 3 — ViewModels

| ID | Task | Status |
|----|------|--------|
| 2.4.1 | `DashboardViewModel` : ObservableObject — summary stats | Pending |
| 2.4.2 | `PackListViewModel` : ObservableObject — ObservableCollection&lt;PackViewModel&gt;, toggle | Pending |
| 2.4.3 | `DebugPanelViewModel` : ObservableObject — 5 sections, collapsible | Pending |
| 2.4.4 | `SettingsViewModel` : ObservableObject — packs directory, game path | Pending |
| 2.4.5 | `MainViewModel` — navigation state, tab selection | Pending |

### 3.5 Phase 4 — Theming

| ID | Task | Status |
|----|------|--------|
| 2.5.1 | `App.xaml` — WindowsAppSDK resources, Mica background | Pending |
| 2.5.2 | `DinoForgeTheme.xaml` — colour tokens matching in-game DinoForgeStyle | Pending |
| 2.5.3 | `Converters/` — BoolToVisibility, IntToStatus, ErrorCountToColour | Pending |

### 3.6 Phase 5 — Views

| ID | Task | Status |
|----|------|--------|
| 2.6.1 | `MainWindow.xaml` — NavigationView, tab shell | Pending |
| 2.6.2 | `DashboardPage.xaml` — pack summary, error banner, refresh button | Pending |
| 2.6.3 | `PackListPage.xaml` — ListView + ItemTemplate mirroring F10 | Pending |
| 2.6.4 | `DebugPanelPage.xaml` — 5 Expander sections mirroring F9 | Pending |
| 2.6.5 | `SettingsPage.xaml` — directory picker, reload interval | Pending |
| 2.6.6 | `PackListItemTemplate.xaml` — name, version, enabled toggle, error indicator | Pending |

### 3.7 Phase 6 — Entry Point

| ID | Task | Status |
|----|------|--------|
| 2.7.1 | `Program.cs` — unpackaged app launch, HostBuilder.Build().Run() | Pending |
| 2.7.2 | `App.xaml.cs` — WindowsAppSDK init, OnLaunched → MainWindow | Pending |

### 3.8 Phase 7 — Settings Persistence

| ID | Task | Status |
|----|------|--------|
| 2.8.1 | `AppConfig.json` — packs directory, game path, theme preference | Pending |
| 2.8.2 | `AppConfigService` — read/write, defaults | Pending |
| 2.8.3 | Verify disabled_packs.json round-trips identically to game format | Pending |

### 3.9 Phase 8 — Tests

| ID | Task | Status |
|----|------|--------|
| 2.9.1 | `src/Tests/CompanionTests/CompanionTests.csproj` | Pending |
| 2.9.2 | `PackDataServiceTests` — FileSystemPackDataService with temp directory | Pending |
| 2.9.3 | `DisabledPacksServiceTests` — round-trip JSON parity | Pending |
| 2.9.4 | `PackListViewModelTests` — toggle, refresh, error propagation | Pending |
| 2.9.5 | `DebugPanelViewModelTests` — section population, collapse state | Pending |

**Deliverable**: `src/Tools/DesktopCompanion/` compilable, launchable WinUI 3 app. M9 in roadmap.

---

## 4. M10 — Fuzzing Infrastructure

**Goal**: Comprehensive property-based and coverage-guided fuzzing across all serialization and registry paths.
**ADR**: ADR-012

### 4.1 FsCheck Expansion

| ID | Task | Status | Domain |
|----|------|--------|--------|
| 3.1.1 | ContentLoader YAML→object round-trip properties | Pending | SDK |
| 3.1.2 | PackLoader manifest edge-case properties | Pending | SDK |
| 3.1.3 | Schema validation boundary conditions | Pending | SDK |
| 3.1.4 | Stat modifier combinatorial correctness | Pending | SDK/Runtime |
| 3.1.5 | AssetSwapRegistry concurrent Register properties | Pending | Runtime |
| 3.1.6 | Entity query generation with varied vanilla_mapping | Pending | Runtime |
| 3.1.7 | DependencyResolver cycle detection properties | Pending | SDK |
| 3.1.8 | CompatibilityChecker monotonicity additional properties | Pending | SDK |
| 3.1.9 | TradeEngine balance invariants | Pending | Economy |
| 3.1.10 | ScenarioRunner condition evaluation | Pending | Scenario |

**Target**: 30+ FsCheck properties (up from 14).

### 4.2 SharpFuzz Integration

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 3.2.1 | Add `SharpFuzz` NuGet to test project | Pending | Coverage-guided |
| 3.2.2 | `FuzzTargets/YamlFuzzTarget.cs` — fuzz YAML deserialization | Pending | |
| 3.2.3 | `FuzzTargets/JsonFuzzTarget.cs` — fuzz JSON schema validation | Pending | |
| 3.2.4 | `FuzzTargets/SemverFuzzTarget.cs` — fuzz semver parsing | Pending | |
| 3.2.5 | `FuzzTargets/PackManifestFuzzTarget.cs` — fuzz pack.yaml parsing | Pending | |
| 3.2.6 | libFuzzer runner script `scripts/run-fuzz.sh` | Pending | |

### 4.3 Corpus Management

| ID | Task | Status |
|----|------|--------|
| 3.3.1 | Create `src/Tests/FuzzCorpus/yaml/` — known edge-case YAML fragments | Pending |
| 3.3.2 | Create `src/Tests/FuzzCorpus/json/` — known edge-case JSON fragments | Pending |
| 3.3.3 | Create `src/Tests/FuzzCorpus/semver/` — boundary semver strings | Pending |
| 3.3.4 | Document corpus maintenance policy in `src/Tests/FuzzCorpus/README.md` | Pending |

### 4.4 Differential Fuzzing

| ID | Task | Status |
|----|------|--------|
| 3.4.1 | `DifferentialFuzzTests.cs` — compare ContentLoader across schema versions | Pending |
| 3.4.2 | Differential pack-loading: enabled vs disabled packs must produce identical registry state | Pending |

### 4.5 CI Integration

| ID | Task | Status |
|----|------|--------|
| 3.5.1 | Add `Category=Fuzz` attribute to all fuzz/property tests | Pending |
| 3.5.2 | `.github/workflows/fuzz.yml` — nightly fuzz run (separate from PR gate) | Pending |
| 3.5.3 | `dotnet test --filter Category=Fuzz` in CI matrix | Pending |

**Deliverable**: 30+ FsCheck properties, SharpFuzz targets for 4 paths, corpus seeded, nightly CI gate.

---

## 5. M11 — Test Coverage Expansion

**Goal**: Fill service-level and registry test gaps identified in audit.
**Target**: 130+ passing tests.

| ID | Task | File | Status |
|----|------|------|--------|
| 4.1 | AssetSwapRegistry: Register, Dequeue, TryDequeue, thread-safety | `AssetSwapRegistryTests.cs` | Pending |
| 4.2 | FileSystemPackRootResolver: path resolution, missing dir | `FileSystemPackRootResolverTests.cs` | Pending |
| 4.3 | PackFileWatcher integration: FSW event → debounce → callback | `PackFileWatcherIntegrationTests.cs` | Pending |
| 4.4 | AddressablesCatalog: extend beyond nonexistent-path stub | `AddressablesCatalogTests.cs` | Pending |
| 4.5 | ContentRegistrationService: schema registration, conflict | `ContentRegistrationServiceTests.cs` | Pending |
| 4.6 | AssetReplacementEngine: swap queuing, apply ordering | `AssetReplacementEngineTests.cs` | Pending |
| 4.7 | Excluded tests re-enabled: UnitSpawnerTests (mock ECS) | `UnitSpawnerTests.cs` | Pending |
| 4.8 | Excluded tests re-enabled: FactionSystemTests (mock ECS) | `FactionSystemTests.cs` | Pending |

---

## 6. M11.5 — Incomplete Code Remediation

**Goal**: Eliminate build-excluded and TODO-incomplete code items.

| ID | Item | File | Action | Status |
|----|------|------|--------|--------|
| 5.1 | NativeMainMenuModMenu | Runtime/UI/ | Implement stub to unblock ContextualModMenuHost.cs | Pending |
| 5.2 | ContextualModMenuHost | Runtime/UI/ContextualModMenuHost.cs | Re-add to build after NativeMainMenuModMenu lands | Pending |
| 5.3 | PackUnitSpawner.OnUpdate() | Runtime/Bridge/PackUnitSpawner.cs | Implement spawn queue + entity instantiation (M9 item) | Pending |
| 5.4 | HotReloadBridge:127 | Runtime/HotReload/HotReloadBridge.cs | Find affected entities + update component data | Pending |
| 5.5 | Aviation namespace | Runtime/Aviation/ | Decision: complete AerialUnitComponent or defer to M12 | Pending |

---

## 7. Cross-Cutting: Docs + Planning

| ID | Task | Status |
|----|------|--------|
| 6.1 | `WORKLOG.md` | Done |
| 6.2 | `docs/WBS.md` (this file) | Done |
| 6.3 | `docs/adr/ADR-011-desktop-companion.md` | Done |
| 6.4 | `docs/adr/ADR-012-fuzzing-strategy.md` | Done |
| 6.5 | Update `docs/product-requirements-document.md` — add M9-M11 reqs | Pending |
| 6.6 | Update `docs/roadmap/index.md` — add M9, M10, M11 milestones | Pending |
| 6.7 | `CHANGELOG.md` — v0.11.0 entry for M8 work | Pending |

---

## 8. Effort Summary

| Milestone | Tasks | Done | Status |
|-----------|-------|------|----|
| M8 Runtime Integration | 10 | 6 | Partial |
| M9 Desktop Companion | 29 | 0 | Not Started |
| M10 Fuzzing | 20 | 2 | Started |
| M11 Test Coverage | 8 | 0 | Not Started |
| M11.5 Incomplete Code | 5 | 0 | Not Started |
| Docs/Planning | 7 | 4 | Partial |
| **Total** | **79** | **12** | In Progress |
