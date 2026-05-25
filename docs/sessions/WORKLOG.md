# DINOForge — Unified Work Log

**Maintained by**: kooshapari + Agent Org
**Updated**: 2026-03-14
**Branch**: main

All active, pending, and blocked work items in one place. Completed items are archived below the active section.

---

## ACTIVE ITEMS

### WI-001 — M8 Runtime Integration: End-to-End Validation
**Status**: IN PROGRESS
**Priority**: P0 (milestone blocker)
**Owner**: Runtime agent
**Branch**: main

End-to-end in-game validation of the full ModPlatform lifecycle with a live DINO instance. ADR-009 is implemented; final sign-off requires game launch validation.

**Subtasks**:
- [x] `ModPlatform` lifecycle (Initialize → LoadPacks → StartHotReload → Shutdown)
- [x] ECS bridge: ComponentMap, EntityQueries, StatModifierSystem
- [x] AssetSwapSystem with vanilla_mapping targeted entity filtering
- [x] HUD pack counter wiring (OnHudCountsChanged callback)
- [x] F9/F10 layout rebuild fix (ForceRebuildLayoutImmediate)
- [x] Pack deployment (warfare-aerial redeployed)
- [ ] Live DINO game validation — all 9 packs load, HUD shows correct counts, F9/F10 panels render correctly

**Blocking**: None
**ADR**: ADR-009

---

### WI-002 — WinUI 3 Desktop Companion
**Status**: PLANNED
**Priority**: P1
**Owner**: Desktop agent (worktree)
**Branch**: `feature/desktop-companion` (worktree, not yet created)

WinUI 3 / WindowsAppSDK 1.6 desktop GUI that mirrors F9/F10 in-game panels for out-of-game evaluation. Mica material, Fluent design, no game launch required.

**Subtasks**:
- [ ] Phase 0: Worktree + project scaffold (`src/Tools/DesktopCompanion/`)
- [ ] Phase 1: Data layer — `PackViewModel`, `LoadResultViewModel`, `DebugSectionViewModel` DTOs (avoid Unity-dependent `PackDisplayInfo`)
- [ ] Phase 2: DI setup (`Microsoft.Extensions.Hosting`, `IPackDataService`)
- [ ] Phase 3: ViewModels — `DashboardViewModel`, `PackListViewModel`, `DebugPanelViewModel`, `SettingsViewModel`
- [ ] Phase 4: Theming — `App.xaml` Mica/Fluent resource dictionary, `DinoForgeTheme.xaml`
- [ ] Phase 5: Views — `MainWindow.xaml` (NavigationView), `DashboardPage`, `PackListPage`, `DebugPanelPage`, `SettingsPage`
- [ ] Phase 6: Entry point — `Program.cs`, `App.xaml.cs`, unpackaged hosting
- [ ] Phase 7: Settings persistence — `AppConfig.json`, `disabled_packs.json` read/write parity with game
- [ ] Phase 8: Tests — `CompanionTests.csproj`, ViewModel unit tests

**Blocking**: WI-001 sign-off preferred (data contracts stable)
**ADR**: ADR-011
**Spec**: `docs/WBS.md` §3, `docs/adr/ADR-011-desktop-companion.md`

---

### WI-003 — Fuzzing Infrastructure Completion
**Status**: PLANNED
**Priority**: P1
**Owner**: Test agent
**Branch**: main

Expand property-based test coverage from 4 domains to 10+, add SharpFuzz for crash-detection on serialization paths, build persistent corpus.

**Subtasks**:
- [x] FsCheck.Xunit 3.3.2 integrated in test project
- [x] PropertyTests.cs — 4 domains (CompatibilityChecker, Registry, DependencyResolver, BalanceCalculator)
- [ ] FsCheck expansion: ContentLoader YAML→object round-trip (7 additional domains)
- [ ] FsCheck expansion: PackLoader manifest parsing edge cases
- [ ] FsCheck expansion: Schema validation boundary conditions
- [ ] FsCheck expansion: Stat modifier combinatorial correctness
- [ ] FsCheck expansion: AssetSwapRegistry thread-safety (concurrent Registers)
- [ ] FsCheck expansion: Entity query generation with varied vanilla_mapping strings
- [ ] SharpFuzz: Integrate `SharpFuzz` NuGet for coverage-guided fuzzing
- [ ] Corpus: Build `src/Tests/FuzzCorpus/` with known edge-case YAML fragments
- [ ] Differential: Add `DifferentialFuzzTests.cs` comparing behaviours across config permutations
- [ ] CI: Add fuzz gate to `DINOForge.CI.sln` (`dotnet test --filter Category=Fuzz`)

**Blocking**: None
**ADR**: ADR-012
**Spec**: `docs/WBS.md` §4

---

### WI-004 — Incomplete Code Items
**Status**: PLANNED
**Priority**: P2
**Owner**: Runtime/SDK agent

Four in-code incomplete items identified in audit.

**Subtasks**:
- [ ] **WI-004a** `ContextualModMenuHost.cs` — Implement `NativeMainMenuModMenu` so this can re-join the build
- [ ] **WI-004b** `PackUnitSpawner.cs` — Implement `OnUpdate()` spawn queue and entity instantiation (M9 milestone item per TODO)
- [ ] **WI-004c** `HotReloadBridge.cs:127` — Implement affected-entity lookup and component data update after hot reload event
- [ ] **WI-004d** `Aviation/` namespace — Either complete `AerialUnitComponent` definition and re-include 8 files, or document aviation as deferred to M10+ and close the TODO

**Blocking**: WI-001
**Spec**: `docs/WBS.md` §5

---

### WI-005 — Test Coverage Expansion
**Status**: PLANNED
**Priority**: P2
**Owner**: Test agent

Fill known gaps in service-level and registry tests discovered in audit.

**Subtasks**:
- [ ] `AssetSwapRegistryTests.cs` — Register, Dequeue, thread-safety
- [ ] `FileSystemPackRootResolverTests.cs` — Path resolution, missing directory handling
- [ ] `PackFileWatcherIntegrationTests.cs` — FileSystemWatcher event → debounce → callback
- [ ] `AddressablesCatalogTests.cs` — Extend beyond "nonexistent path" stub
- [ ] `ContentRegistrationServiceTests.cs` — Schema registration, conflict detection
- [ ] `AssetReplacementEngineTests.cs` — Swap queuing, apply ordering
- [ ] Increase FsCheck from 14 properties to 30+ properties (see WI-003)

**Blocking**: None
**Target**: 130+ passing tests (currently ~80 unit + integration)

---

### WI-006 — PRD / ADR / Docs Refresh
**Status**: IN PROGRESS
**Priority**: P2
**Owner**: Docs agent

Update docs to reflect M8 completion work and new milestones M9-M11.

**Subtasks**:
- [x] `WORKLOG.md` — this file
- [x] `docs/WBS.md` — full WBS created
- [x] `docs/adr/ADR-011-desktop-companion.md` — decision record
- [x] `docs/adr/ADR-012-fuzzing-strategy.md` — decision record
- [ ] `docs/product-requirements-document.md` — add M9-M11 requirements, Desktop Companion user stories
- [ ] `docs/roadmap/index.md` — add M9 (Desktop Companion), M10 (Fuzzing+Coverage), M11 (Incomplete Code)
- [ ] `CHANGELOG.md` — entry for v0.11.0 work

---

## BLOCKED ITEMS

*(none currently)*

---

## COMPLETED (This Sprint)

| Item | Description | Completed |
|------|-------------|-----------|
| AssetSwap targeted swap | vanilla_mapping → ECS archetype → filtered EntityQuery | 2026-03-14 |
| HUD pack counter | OnHudCountsChanged callback wired to HudStrip + HudIndicator | 2026-03-14 |
| F9/F10 blank panels | ForceRebuildLayoutImmediate in Show() + rebuild methods | 2026-03-14 |
| warfare-aerial deploy | Pack redeployed to dinoforge_packs/ | 2026-03-14 |
| Error detail visibility | First error text surfaced inline in F10 status bar | 2026-03-14 |
| Desktop Companion audit | Full panel data contract, presenter pattern, ModPlatform API audit | 2026-03-14 |
| Desktop Companion plan | 8-phase implementation plan, 29-file list, risk register | 2026-03-14 |

---

## REFERENCE

- **Milestones**: `docs/roadmap/index.md`
- **ADRs**: `docs/adr/`
- **WBS**: `docs/WBS.md`
- **PRD**: `docs/product-requirements-document.md`
- **Architecture**: `CLAUDE.md`
