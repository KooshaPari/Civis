# DINOForge Test Traceability Matrix

**Version**: 1.0.0
**Last Updated**: 2026-04-01
**Status**: Active

---

## Overview

This document maps user stories, epics, and acceptance criteria to test methods using xUnit `[Trait]` attributes.

### Trait Convention

- **Category**: High-level grouping (e.g., `UserStory`, `Epic`, `Journey`)
- **Trait**: Specific identifier (e.g., `US-F1.1`, `Epic-PackSystem`)

### Example Usage

```csharp
[Trait("Category", "UserStory")]
[Trait("UserStory", "US-F1.1")]
public class PackLoadingTests
{
    [Fact]
    public void Pack_WithValidYaml_LoadsSuccessfully() { }
}
```

---

## User Stories

### US-F1.1: Pack Manifest Loading
**Story**: "As a mod author, I can create a pack with a `pack.yaml` manifest so the game recognizes it as a mod"

**Acceptance Criteria**:
- [x] Pack with valid `pack.yaml` loads on startup
- [x] Pack with missing `pack.yaml` fails with clear error
- [x] Pack with invalid YAML fails with line number
- [x] Framework version check works
- [x] Dependency resolution respects declared `depends_on`

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `PackLoaderTests.cs` | `Load_ValidPackYaml_ReturnsPackManifest` | `US-F1.1` |
| `PackLoaderTests.cs` | `Load_MissingPackYaml_ThrowsFileNotFoundException` | `US-F1.1` |
| `PackLoaderTests.cs` | `Load_InvalidYaml_ThrowsYamlException` | `US-F1.1` |
| `DependencyResolverTests.cs` | `Resolve_CircularDependency_Fails` | `US-F1.1` |
| `DependencyResolverTests.cs` | `Resolve_MissingDependency_Fails` | `US-F1.1` |
| `PackRegistryTests.cs` | `Register_FrameworkVersionMismatch_Fails` | `US-F1.1` |

---

### US-F2.1: Debug Overlay (F9)
**Story**: "As a mod author, I can press F9 to see loaded packs and entity statistics without restarting"

**Acceptance Criteria**:
- [x] F9 toggles overlay visibility
- [x] Shows list of loaded packs with version
- [x] Shows entity count per pack/faction
- [x] Shows system performance stats
- [x] Shows error messages in red
- [x] Overlay doesn't impact game performance

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `ModMenuTests.cs` | `F9_TogglesOverlay_WhenPressed` | `US-F2.1` |
| `GameLaunchOverlayTests.cs` | `Overlay_F9_AssertDebugPanelVisible_AtMainMenu` | `US-F2.1`, SPEC-007 |
| `GameLaunchOverlayTests.cs` | `Overlay_F9_SecondToggle_ClosesDebugPanel_AtMainMenu` | `US-F2.1`, SPEC-007 |
| `GameLaunchOverlayTests.cs` | `Overlay_F9_F10_ToggleDuringGameplay` (debug leg) | `US-F2.1`, SPEC-007 |
| `GameLaunchOverlayTests.cs` | `Overlay_Panels_HiddenByDefault_AtMainMenu` (DebugPanel) | `US-F2.1`, SPEC-007 |
| `ModMenuTests.cs` | `DebugPanel_Build_StartsHiddenWithZeroAlpha` | `US-F2.1`, SPEC-007 |
| `RuntimeExtractionTests.cs` | `GetLoadedPacks_ReturnsAllRegistered` | `US-F2.1` |
| `EntityInspectorTests.cs` | `GetEntityCount_ByFaction_ReturnsCorrect` | `US-F2.1` |
| `PerformanceBenchmarkTests.cs` | `OverlayRender_Under1ms` | `US-F2.1` |

---

### US-F3.1: Mod Menu Toggle (F10)
**Story**: "As a mod author, I can press F10 to toggle individual packs on/off without restarting"

**Acceptance Criteria**:
- [x] F10 opens mod menu overlay
- [x] Menu shows checkbox list of loaded packs
- [x] Toggling checkbox writes `disabled_packs.json`
- [x] Game respects disabled packs on next launch
- [x] Menu has settings for each pack
- [x] Menu is responsive

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `ModMenuTests.cs` | `F10_OpensModMenu` | `US-F3.1` |
| `GameLaunchUiTests.cs` | `Overlay_F10_TogglesModMenu` | `US-F3.1`, SPEC-007 |
| `GameLaunchUiTests.cs` | `Overlay_SecondToggle_ClosesModMenu` | `US-F3.1`, SPEC-007 |
| `GameLaunchOverlayTests.cs` | `Overlay_F9_F10_ToggleDuringGameplay` (mod menu leg) | `US-F3.1`, SPEC-007 |
| `GameLaunchOverlayTests.cs` | `Overlay_Panels_HiddenByDefault_AtMainMenu` (ModMenuPanel) | `US-F3.1`, SPEC-007 |
| `GameLaunchNativeMenuTests.cs` | `MainMenu_HasModsButton_AfterInjection` | `US-F3.1`, SPEC-007 |
| `GameLaunchNativeMenuTests.cs` | `MainMenu_ModsButton_OpensOverlay` | `US-F3.1`, SPEC-007 |
| `GameLaunchNativeMenuTests.cs` | `MainMenu_ModsButton_StyleMatchesSettings_AfterInjection` | `US-F3.1`, SPEC-007 |
| `ModMenuTests.cs` | `ModMenuPanel_Build_StartsHiddenWithZeroAlpha` | `US-F3.1`, SPEC-007 |
| `ModMenuTests.cs` | `TogglePack_WritesDisabledPacksJson` | `US-F3.1` |
| `DisabledPacksPersistenceTests.cs` | `DisabledPacks_LoadedOnStartup` | `US-F3.1` |
| `ModMenuTests.cs` | `ModMenu_ResponsiveUnderLoad` | `US-F3.1` |

---

### US-F4.1: Hot Module Reload
**Story**: "As a mod author, I can edit `units.yaml`, press F10 → Reload, and see changes instantly"

**Acceptance Criteria**:
- [x] Reload reads all modified YAML files from disk
- [x] Registry updates with new definitions
- [x] New entities use updated stats
- [x] Existing entities reflect partial updates
- [x] No crashes or memory leaks during reload
- [ ] Players notified of reload via chat message

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `HotReloadTests.cs` | `Reload_ModifiedYaml_UpdatesRegistry` | `US-F4.1` |
| `HotReloadTests.cs` | `Reload_ExistingEntities_UpdateStats` | `US-F4.1` |
| `HotReloadTests.cs` | `Reload_NoMemoryLeaks` | `US-F4.1` |
| `PackFileWatcherTests.cs` | `FileWatcher_DetectsChanges` | `US-F4.1` |

---

### US-F5.1: Asset Swap System
**Story**: "As a pack author, I can define `visual_asset: my-unit-model` and the game uses my custom 3D model"

**Acceptance Criteria**:
- [x] Asset bundles load from `packs/my-pack/assets/bundles/`
- [x] Addressables catalog maps asset IDs to bundles
- [x] Live entity swap replaces vanilla prefab with custom one
- [x] Catalog patch updates game's internal asset references
- [x] Fallback to vanilla model if custom asset fails to load
- [x] LOD variants reduce polycount for distant units

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `AssetSwapRegistryTests.cs` | `Swap_WithValidBundle_ReplacesPrefab` | `US-F5.1` |
| `AssetSwapRegistryTests.cs` | `Swap_MissingBundle_FallsBackToVanilla` | `US-F5.1` |
| `AddressablesCatalogTests.cs` | `LoadCatalog_MapsAssetIdsToBundles` | `US-F5.1` |
| `AssetSwapTests.cs` | `EntitySwap_ReplacesVanillaWithCustom` | `US-F5.1` |
| `LODCalculationTests.cs` | `LOD_ReducesPolycountCorrectly` | `US-F5.1` |

---

### US-F6.1: Pack Validation & Compiler
**Story**: "As a mod author, I can run `pack-deploy` and get immediate feedback on schema violations"

**Acceptance Criteria**:
- [x] Validates `pack.yaml` against schema
- [x] Reports missing YAML files
- [x] Checks all asset references exist
- [x] Detects circular dependencies
- [x] Flags conflicting packs
- [x] Provides clear error messages with line numbers
- [x] Builds pack artifact

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `PackCompilerCliTests.cs` | `Validate_ValidPack_Succeeds` | `US-F6.1` |
| `PackCompilerCliTests.cs` | `Validate_MissingFiles_ReportsErrors` | `US-F6.1` |
| `PackCompilerCliTests.cs` | `Validate_CircularDeps_Detected` | `US-F6.1` |
| `CompatibilityCheckerTests.cs` | `CheckConflicts_DetectsOverlappingPacks` | `US-F6.1` |
| `SchemaValidationTests.cs` | `ValidatePackYaml_SchemaViolations` | `US-F6.1` |

---

### US-F7.1: Entity Inspector
**Story**: "As a mod author, I can press F9 → Entity Inspector and search for units to see their component values"

**Acceptance Criteria**:
- [x] Search by unit name or ID
- [x] Display all components on entity
- [x] Show calculated stats vs. base stats
- [x] Show stat overrides from packs
- [ ] Allow live editing of values (for testing)
- [x] No performance impact when hidden

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `UnitSpawnerTests.cs` | `QueryByName_ReturnsMatchingEntities` | `US-F7.1` |
| `UnitSpawnerTests.cs` | `GetComponents_AllOnEntity` | `US-F7.1` |
| `StatTests.cs` | `GetStat_CalculatedVsBase` | `US-F7.1` |
| `OverrideApplicatorTests.cs` | `Overrides_DisplayedCorrectly` | `US-F7.1` |

---

### US-F8.1: Desktop Companion App
**Story**: "As a mod author, I can open the Desktop Companion, see all installed packs, toggle them on/off"

**Acceptance Criteria**:
- [x] Companion launches in <3 seconds
- [x] Shows list of packs with version, author, status
- [x] Can toggle packs on/off
- [x] Shows pack dependencies and conflicts
- [x] Shows F9/F10 debug panel snapshots
- [x] Real-time YAML file watcher updates UI
- [x] No game process required

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `CompanionTests/PackDataServiceTests.cs` | `GetPacks_AllRegistered` | `US-F8.1` |
| `CompanionTests/DisabledPacksServiceTests.cs` | `TogglePack_WritesDisabledPacks` | `US-F8.1` |
| `CompanionTests/ViewModelTests.cs` | `LaunchTime_Under3Seconds` | `US-F8.1` |
| `CompanionTests/PackListTests.cs` | `ShowsDependenciesAndConflicts` | `US-F8.1` |

---

## Epics

### Epic: Pack System
**ID**: `Epic-PackSystem`
**Description**: Core pack loading, validation, and dependency resolution

**User Stories**:
- US-F1.1: Pack Manifest Loading
- US-F6.1: Pack Validation & Compiler

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `PackLoaderTests.cs` | All tests | `Epic-PackSystem` |
| `PackRegistryTests.cs` | All tests | `Epic-PackSystem` |
| `DependencyResolverTests.cs` | All tests | `Epic-PackSystem` |
| `PackCompilerCliTests.cs` | All tests | `Epic-PackSystem` |

---

### Epic: Runtime UI
**ID**: `Epic-RuntimeUI`
**Description**: In-game debug overlay, mod menu, and hot reload

**User Stories**:
- US-F2.1: Debug Overlay (F9)
- US-F3.1: Mod Menu Toggle (F10)
- US-F4.1: Hot Module Reload

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `ModMenuTests.cs` | All tests | `Epic-RuntimeUI` |
| `GameLaunchOverlayTests.cs` | All tests | `Epic-RuntimeUI`, SPEC-007 |
| `GameLaunchUiTests.cs` | All tests | `Epic-RuntimeUI`, SPEC-007 |
| `GameLaunchNativeMenuTests.cs` | All tests | `Epic-RuntimeUI`, SPEC-007 |
| `HotReloadTests.cs` | All tests | `Epic-RuntimeUI` |
| `DisabledPacksPersistenceTests.cs` | All tests | `Epic-RuntimeUI` |

---

### Epic: Asset Management
**ID**: `Epic-AssetManagement`
**Description**: Asset swap, LOD generation, and addressables

**User Stories**:
- US-F5.1: Asset Swap System

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `AssetSwapRegistryTests.cs` | All tests | `Epic-AssetManagement` |
| `AssetSwapTests.cs` | All tests | `Epic-AssetManagement` |
| `AddressablesCatalogTests.cs` | All tests | `Epic-AssetManagement` |
| `LODCalculationTests.cs` | All tests | `Epic-AssetManagement` |

---

### Epic: Developer Tools
**ID**: `Epic-DeveloperTools`
**Description**: CLI tools, debugging, and diagnostics

**User Stories**:
- US-F7.1: Entity Inspector
- US-F8.1: Desktop Companion App

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `UnitSpawnerTests.cs` | All tests | `Epic-DeveloperTools` |
| `CompanionTests/*.cs` | All tests | `Epic-DeveloperTools` |
| `CliToolTests/*.cs` | All tests | `Epic-DeveloperTools` |

---

## User Journeys

### Journey 1: Install & Play (E2E)
**ID**: `Journey-InstallPlay`
**Path**: End-user installs a mod pack and plays

```
1. Download pack → 2. Extract to BepInEx/dinoforge_packs/ → 3. Launch game → 4. Pack loads → 5. Play with mod
```

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `Integration/Tests/PackLoadingTests.cs` | `ReloadPacks_Succeeds` | `Journey-InstallPlay` |
| `Integration/Tests/GameWorkflowTests.cs` | `LoadSave_GivenMainMenu_CreatesLoadRequestEntity` | `Journey-InstallPlay` |

---

### Journey 2: Create Balance Mod (E2E)
**ID**: `Journey-CreateBalance`
**Path**: Mod author creates a cost/stats balance pack

```
1. Create pack.yaml → 2. Create units.yaml with overrides → 3. Run pack-deploy → 4. Launch game → 5. Verify changes → 6. Hot reload
```

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `PackLoaderTests.cs` | `Load_WithStatOverrides_AppliesCorrectly` | `Journey-CreateBalance` |
| `HotReloadTests.cs` | `Reload_StatChanges_VisibleInGame` | `Journey-CreateBalance` |
| `StatTests.cs` | `ApplyOverride_ChangesStatValue` | `Journey-CreateBalance` |

---

### Journey 3: Create Total Conversion (E2E)
**ID**: `Journey-CreateTotalConversion`
**Path**: Mod author creates Star Wars themed total conversion

```
1. Create pack + factions → 2. Define 30+ units → 3. Add visual assets → 4. Build bundles → 5. Deploy → 6. Test
```

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `WarfareTests.cs` | `LoadFactions_AllRegistered` | `Journey-CreateTotalConversion` |
| `TotalConversionTests.cs` | `AllUnits_LoadWithVisuals` | `Journey-CreateTotalConversion` |
| `AssetSwapTests.cs` | `AssetSwapSystem_GivenAll28StarWarsEntities_AllSwapsSucceedOrDiagnosed` | `Journey-CreateTotalConversion` |

---

### Journey 4: Debug & Troubleshoot (E2E)
**ID**: `Journey-Debug`
**Path**: Mod author diagnoses why a pack doesn't work

```
1. Check manifest → 2. Check assets → 3. Launch game → 4. Check F9 overlay → 5. Query entities
```

**Test Methods**:
| Test File | Test Method | Traits |
|-----------|-------------|--------|
| `PackCompilerCliTests.cs` | `Validate_ReportsClearErrors` | `Journey-Debug` |
| `Integration/Tests/GameWorkflowTests.cs` | `CliJsonOutput_GivenFormatJsonFlag_OutputsValidJson` | `Journey-Debug` |

---

## EPIC-027 NFR Traceability (v0.27.0)

| NFR ID | Category | Owning Stories | Description |
|--------|----------|----------------|-------------|
| EPIC-027-NFR-001 | Performance | SW-001, SW-013 | Mods page opens ≤ 500 ms |
| EPIC-027-NFR-002 | Regression | SW-006, SW-007, SW-013 | No regressions from v0.26.0 |
| EPIC-027-NFR-003 | Compatibility | SW-003 | Unity 2021.3.45f2 bundle version enforcement |
| EPIC-027-NFR-004 | Memory | SW-001, SW-004 | No monotonic GameObject/memory growth on open/close |
| EPIC-027-NFR-005 | Build | SW-002–SW-013 | `netstandard2.0` TFM; no compile-time TMPro/Addressables refs |
| EPIC-027-NFR-006 | Compatibility | SW-003–SW-012 | BepInEx 5.4.23.5 load without TypeLoadException |
| EPIC-027-NFR-007 | Architecture | SW-001, SW-005, SW-006 | No Harmony patches on DINO UI types |
| EPIC-027-NFR-008 | Naming | SW-001, SW-004–SW-007 | All injected GameObjects prefixed `DINOForge_` |
| EPIC-027-NFR-009 | Security | SW-001 | No unvalidated pack data passed to Process.Start / URL open |
| EPIC-027-NFR-010 | Security | SW-003, SW-008 | Tampered/wrong-version bundles skipped with warning |
| EPIC-027-NFR-011 | Security | SW-003, SW-007, SW-008 | PackCompiler rejects `../` or absolute asset paths |
| EPIC-027-NFR-013 | Stability | SW-004, SW-006–SW-012 | No TypeLoadException in LogOutput.log after clean launch |
| EPIC-027-NFR-014 | Resilience | SW-004, SW-007, SW-009 | Missing asset bundles degrade gracefully; vanilla fallback + WARNING |
| EPIC-027-NFR-015 | Input Safety | SW-001, SW-004–SW-008 | `raycastTarget=false` on injected Images; EventSystem guard (Pattern #235) |
| EPIC-027-NFR-016 | UX | SW-001, SW-006 | Injected UI matches native DINO button hover/layout |
| EPIC-027-NFR-017 | UX | SW-001 | Escape closes Mods page; keyboard navigation works |
| EPIC-027-NFR-018 | i18n | SW-001, SW-002, SW-005 | All new strings pass through locale layer |
| EPIC-027-NFR-019 | Asset Gate | SW-003, SW-008 | `detect_stub_bundles.py` exits 0; visual_asset count == non-stub bundle count |
| EPIC-027-NFR-020 | Visual | SW-005, SW-007, SW-008 | No vanilla DINO medieval 2D art visible with TC active (judge receipt) |
| EPIC-027-NFR-021 | Visual | SW-005, SW-008 | Faction emblems + unit portraits visible in-game for both mods |
| EPIC-027-NFR-022 | Legal | SW-008, SW-012 | Asset licensing manifest complete; all shipped audio/images/3D documented as original or CC0; `LICENSE-audio.md`/`LICENSE-assets.md` present in each pack |

---

## AgilePlus Story Links

| AgilePlus Story ID | User Story | Test Methods |
|-------------------|------------|--------------|
| WP01-001 | US-F1.1 Pack Manifest | `PackLoaderTests.*` |
| WP01-002 | US-F2.1 Debug Overlay | `ModMenuTests.F9_*` |
| WP01-003 | US-F3.1 Mod Menu | `ModMenuTests.F10_*` |
| WP01-004 | US-F4.1 Hot Reload | `HotReloadTests.*` |
| WP01-005 | US-F5.1 Asset Swap | `AssetSwapTests.*` |
| WP01-006 | US-F6.1 Pack Compiler | `PackCompilerCliTests.*` |
| WP01-007 | US-F7.1 Entity Inspector | `UnitSpawnerTests.*` |
| WP01-008 | US-F8.1 Desktop Companion | `CompanionTests.*` |

---

## Coverage Summary

| Category | Total Criteria | Covered | Coverage % |
|----------|---------------|--------|-----------|
| User Stories | 48 | 47 | 97.9% |
| Epics | 4 | 4 | 100% |
| Journeys | 4 | 4 | 100% |
| AgilePlus Stories | 8 | 8 | 100% |

**Overall Test Coverage**: 97.9%

---

## Implementation Specifications

| Spec | Status | Notes |
|------|--------|-------|
| [SPEC-002](./SPEC-002-native-menu-injector.md) | Accepted | Native menu Mods button injection |
| [SPEC-003](./SPEC-003-prove-features-skill.md) | Active — v2 pipeline implemented | [WORK-001](../work-items/WORK-001-prove-features-improvements.md) closed |
| [SPEC-004](./SPEC-004-key-input-system.md) | Implemented — Active | F9/F10 layered redundancy |
| [SPEC-005](./SPEC-005-duplicate-instance-bypass.md) | Cancelled / Superseded | `boot.config` `single-instance=0` |
| [SPEC-006](./SPEC-006-prove-features-video-pipeline.md) | Superseded | [v2 design](../superpowers/specs/2026-03-27-prove-features-video-pipeline-v2-design.md) |
| [SPEC-007](./SPEC-007-runtime-features-baseline.md) | Active | Runtime feature baseline; GameLaunch + `ModMenuTests` characterization |
| [M13](./M13-runtime-survival-hmr-concurrency.md) | Draft | HMR and concurrent instances |

---

**Last Updated**: 2026-05-23
**Next Review**: After each release
