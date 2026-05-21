# Pattern #226 Audit: Public Mutable Fields

**Audit Date**: 2026-05-18

## Detection Script

- **Path**: `scripts/ci/audit_public_fields.py`
- **LOC**: 137
- **Exclusions**: const, readonly static, [FieldOffset], [StructLayout], // public-field-ok: marker

## Summary

**Total Violations**: 70

### Severity Breakdown
- **HIGH** (NuGet-published): 0
- **MED** (Internal but public): 33
- **LOW** (Tools/CLI): 37

### Directory Heat Map
- `src/Runtime\VFX\VFXPrefabDescriptor.cs/`: 21 violations
- `src/Tools\McpServer\Tools\GameInputTool.cs/`: 17 violations
- `src/Tools\McpServer\Tools\GameInputHelper.cs/`: 13 violations
- `src/Runtime\Aviation\AerialUnitComponent.cs/`: 4 violations
- `src/Tools\Installer\GUI\ViewModels\MaintenancePageViewModel.cs/`: 4 violations
- `src/Runtime\Aviation\AntiAirComponent.cs/`: 2 violations
- `src/Runtime\UI\DFCanvas.cs/`: 2 violations
- `src/Tools\Installer\GUI\ViewModels\WelcomePageViewModel.cs/`: 2 violations
- `src/Runtime\ModPlatform.cs/`: 1 violations
- `src/Runtime\Aviation\AerialSpawnSystem.cs/`: 1 violations
- `src/Runtime\HotReload\HotReloadBridge.cs/`: 1 violations
- `src/Runtime\UI\HudStrip.cs/`: 1 violations
- `src/Tools\PackCompiler\Services\AssetOptimizationService.cs/`: 1 violations

## Top 15 Violations

| File | Line | Severity | Field | Context |
|------|------|----------|-------|----------|
| `Runtime\Aviation\AerialSpawnSystem.cs` | 45 | MED | `SpawnAtAltitude` | public static bool SpawnAtAltitude = true; |
| `Runtime\Aviation\AerialUnitComponent.cs` | 17 | MED | `CruiseAltitude` | public float CruiseAltitude; |
| `Runtime\Aviation\AerialUnitComponent.cs` | 22 | MED | `AscendSpeed` | public float AscendSpeed; |
| `Runtime\Aviation\AerialUnitComponent.cs` | 27 | MED | `DescendSpeed` | public float DescendSpeed; |
| `Runtime\Aviation\AerialUnitComponent.cs` | 33 | MED | `IsAttacking` | public bool IsAttacking; |
| `Runtime\Aviation\AntiAirComponent.cs` | 17 | MED | `AntiAirRange` | public float AntiAirRange; |
| `Runtime\Aviation\AntiAirComponent.cs` | 22 | MED | `AntiAirDamageBonus` | public float AntiAirDamageBonus; |
| `Runtime\HotReload\HotReloadBridge.cs` | 25 | MED | `OnRuntimeUpdated` | `public event EventHandler<HotReloadResult>? OnRuntimeUpdated;` |
| `Runtime\ModPlatform.cs` | 69 | MED | `OnHudCountsChanged` | `public Action<int, int>? OnHudCountsChanged;` |
| `Runtime\UI\DFCanvas.cs` | 51 | MED | `OnInitSuccess` | public Action? OnInitSuccess; |
| `Runtime\UI\DFCanvas.cs` | 58 | MED | `OnInitFailed` | public Action? OnInitFailed; |
| `Runtime\UI\HudStrip.cs` | 29 | MED | `OnClicked` | public Action? OnClicked; |
| `Runtime\VFX\VFXPrefabDescriptor.cs` | 90 | MED | `Duration` | public float Duration = 0.5f; |
| `Runtime\VFX\VFXPrefabDescriptor.cs` | 91 | MED | `Loop` | public bool Loop = false; |
| `Runtime\VFX\VFXPrefabDescriptor.cs` | 92 | MED | `StartLifetime` | public float StartLifetime = 0.3f; |

## Tier Classification

**MODERATE** — Sweep before next NuGet release (v0.25.0). Promote to CI lint with allowlist.

## Recommendation

No NuGet-published violations. MED/LOW violations can be fixed opportunistically.
