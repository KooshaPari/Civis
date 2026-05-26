# ADR-011 — WinUI 3 Desktop Companion App

**Date**: 2026-03-14
**Status**: Accepted
**Deciders**: kooshapari
**Related**: ADR-009 (Runtime Orchestration), ADR-008 (Wrap Don't Handroll)

---

## Context

The DINOForge in-game UI (F9 debug panel, F10 mod menu) requires a full game launch to evaluate any UI or pack configuration changes. This creates a high iteration cost:

- Game launch takes ~90 seconds on typical hardware
- BepInEx plugin load adds ~10 seconds
- Pack reload on change requires re-launch or F9 hot reload
- Debugging UI layout issues (e.g. blank panel body) is difficult without immediate feedback

Developers and mod authors need a way to:
1. Preview pack lists and pack status without launching DINO
2. Enable/disable packs and see the effect on load order immediately
3. Inspect debug panel data (entity counts, system state, errors) from a logged dump
4. Evaluate F9/F10 UI component layout changes without game relaunch

---

## Options Considered

### Option A — WinUI 3 / WindowsAppSDK (Chosen)
**Framework**: WindowsAppSDK 1.6, WinUI 3, Mica background, NavigationView
**VM layer**: CommunityToolkit.Mvvm (ObservableObject, RelayCommand)
**DI**: Microsoft.Extensions.Hosting + DI container
**Target**: `net8.0-windows`, unpackaged deployment

Pros:
- First-class Windows 11 Fluent design (Mica, acrylic, rounded corners)
- Native Win32 integration (no Electron overhead)
- CommunityToolkit.WinUI provides controls matching DINOForge's dark panel aesthetic
- SDK (`netstandard2.0`) can be directly referenced from `net8.0-windows`
- TabView/NavigationView maps naturally to F9/F10 dual-panel concept

Cons:
- Windows-only (acceptable — DINO is Windows-only)
- WinUI 3 XAML tooling less mature than WPF

### Option B — WPF
Pros: Mature, widely documented
Cons: No Mica, older visual design language, no Fluent controls

### Option C — MAUI
Pros: Cross-platform
Cons: DINO is Windows-only; MAUI adds unnecessary complexity; desktop desktop performance overhead

### Option D — Avalonia
Pros: Cross-platform, close to WPF
Cons: Different theming system; DINO target audience is Windows users only

---

## Decision

**Use WinUI 3 / WindowsAppSDK 1.6** (Option A).

The companion is Windows-only by nature (mirrors an in-game Windows UI for a Windows game). WinUI 3 with Mica provides the highest fidelity match to the in-game `DFCanvas` dark theme and requires the least adaptation.

---

## Architecture

```
src/Tools/DesktopCompanion/
├── App.xaml + App.xaml.cs               # WinUI 3 app entry, WindowsAppSDK init
├── Program.cs                            # IHost bootstrap, unpackaged launch
├── MainWindow.xaml                       # NavigationView shell
├── Data/
│   ├── PackViewModel.cs                  # Local DTO (NOT Unity-dependent PackDisplayInfo)
│   ├── LoadResultViewModel.cs
│   ├── DebugSectionViewModel.cs
│   ├── IPackDataService.cs
│   ├── FileSystemPackDataService.cs      # Reads packs/ from filesystem
│   ├── DisabledPacksService.cs           # disabled_packs.json parity with game
│   └── AppConfigService.cs              # AppConfig.json persistence
├── ViewModels/
│   ├── MainViewModel.cs
│   ├── DashboardViewModel.cs
│   ├── PackListViewModel.cs
│   └── DebugPanelViewModel.cs
├── Views/
│   ├── DashboardPage.xaml
│   ├── PackListPage.xaml
│   ├── DebugPanelPage.xaml
│   └── SettingsPage.xaml
├── Themes/
│   ├── DinoForgeTheme.xaml               # Colour tokens matching DinoForgeStyle.cs
│   └── Converters/
└── HostBuilderExtensions.cs              # DI registrations
```

### Key Design Constraints

1. **No Unity dependency** — `PackDisplayInfo` lives in DINOForge.Runtime (Unity-dependent). Companion defines its own `PackViewModel` DTO. SDK (`DINOForge.SDK.dll`) is safely referenceable.
2. **disabled_packs.json parity** — Both game and companion must read/write the same JSON format to avoid desync.
3. **Wrap, don't handroll** — All YAML parsing via YamlDotNet (already SDK dep), JSON via System.Text.Json.
4. **Unpackaged deployment** — No MSIX; run directly from build output. WindowsAppSDK 1.6 supports this.
5. **IHostedService for file watching** — Wraps `PackFileWatcher` (SDK) as background service; no custom polling.

---

## Consequences

**Positive**:
- Pack author iteration loop drops from ~2min (game launch) to <5s (companion refresh)
- F9/F10 component layout changes can be evaluated in companion before committing to game launch
- Companion can serve as standalone pack manager for end users

**Negative**:
- Windows-only distribution (acceptable given DINO platform)
- Requires `WindowsAppSDK` runtime on developer machine
- UI parity with in-game panels must be maintained when game panels change

**Risks**:

| Risk | Likelihood | Mitigation |
|------|------------|-----------|
| PackDisplayInfo data contract diverges from CompanionDTO | Medium | Define clear mapping in DisabledPacksService; integration test round-trip |
| SDK native deps (AssetsTools.NET) fail in unpackaged publish | Low | Verify sidecar DLLs in Phase 0 scaffold |
| CommunityToolkit.WinUI version lag | Low | Pin to stable 8.x release |
| disabled_packs.json format drift between game + companion | Medium | Shared JSON schema, shared test fixture |
