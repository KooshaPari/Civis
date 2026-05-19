# Pattern #232 Audit: Logger String Interpolation

## Summary

- **Total Violations**: 61
- **HIGH**: 61
- **MED**: 0
- **LOW**: 0
- **Tier**: MODERATE

## Top 10 Violations

- `C:\Users\koosh\Dino\src\Runtime\EntityDumper.cs:65` [HIGH]
  ```csharp
  _log.LogError($"Entity dump failed: {ex}");
  ```

- `C:\Users\koosh\Dino\src\Runtime\EntityDumper.cs:106` [HIGH]
  ```csharp
  _log.LogWarning($"  Failed to dump world '{world.Name}': {ex.Message}");
  ```

- `C:\Users\koosh\Dino\src\Runtime\EntityDumper.cs:145` [HIGH]
  ```csharp
  _log.LogWarning($"  Failed to get components for entity {entity.Index}: {ex.Message}");
  ```

- `C:\Users\koosh\Dino\src\Runtime\EntityDumper.cs:216` [HIGH]
  ```csharp
  _log.LogWarning($"  Failed to dump sample entities for archetype {archetypeIndex}: {ex.Message}");
  ```

- `C:\Users\koosh\Dino\src\Runtime\EntityDumper.cs:354` [HIGH]
  ```csharp
  _log.LogWarning($"  Failed to scan assembly {assembly.GetName().Name}: {ex.Message}");
  ```

- `C:\Users\koosh\Dino\src\Runtime\EntityDumper.cs:398` [HIGH]
  ```csharp
  _log.LogWarning($"  Failed to scan {assemblyName}: {ex.Message}");
  ```

- `C:\Users\koosh\Dino\src\Runtime\HotReload\HotReloadBridge.cs:80` [HIGH]
  ```csharp
  _log.LogError($"[HotReloadBridge] Reload error: {error}");
  ```

- `C:\Users\koosh\Dino\src\Runtime\HotReload\HotReloadBridge.cs:102` [HIGH]
  ```csharp
  _log.LogWarning($"[HotReloadBridge] Pack reload had errors:");
  ```

- `C:\Users\koosh\Dino\src\Runtime\HotReload\HotReloadBridge.cs:105` [HIGH]
  ```csharp
  _log.LogError($"  {error}");
  ```

- `C:\Users\koosh\Dino\src\Runtime\HotReload\HotReloadBridge.cs:141` [HIGH]
  ```csharp
  _log.LogWarning($"[HotReloadBridge] StatModifierSystem.Reapply() failed: {ex.Message}");
  ```

## All 61 Violations (CSV)

| File | Line | Severity | Text |
|------|------|----------|------|
| C:\Users\koosh\Dino\src\Runtime\EntityDumper.cs | 65 | HIGH | `_log.LogError($"Entity dump failed: {ex}");...` |
| C:\Users\koosh\Dino\src\Runtime\EntityDumper.cs | 106 | HIGH | `_log.LogWarning($"  Failed to dump world '{world.Name}': {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\EntityDumper.cs | 145 | HIGH | `_log.LogWarning($"  Failed to get components for entity {entity.Index}: {ex.Mess...` |
| C:\Users\koosh\Dino\src\Runtime\EntityDumper.cs | 216 | HIGH | `_log.LogWarning($"  Failed to dump sample entities for archetype {archetypeIndex...` |
| C:\Users\koosh\Dino\src\Runtime\EntityDumper.cs | 354 | HIGH | `_log.LogWarning($"  Failed to scan assembly {assembly.GetName().Name}: {ex.Messa...` |
| C:\Users\koosh\Dino\src\Runtime\EntityDumper.cs | 398 | HIGH | `_log.LogWarning($"  Failed to scan {assemblyName}: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\HotReload\HotReloadBridge.cs | 80 | HIGH | `_log.LogError($"[HotReloadBridge] Reload error: {error}");...` |
| C:\Users\koosh\Dino\src\Runtime\HotReload\HotReloadBridge.cs | 102 | HIGH | `_log.LogWarning($"[HotReloadBridge] Pack reload had errors:");...` |
| C:\Users\koosh\Dino\src\Runtime\HotReload\HotReloadBridge.cs | 105 | HIGH | `_log.LogError($"  {error}");...` |
| C:\Users\koosh\Dino\src\Runtime\HotReload\HotReloadBridge.cs | 141 | HIGH | `_log.LogWarning($"[HotReloadBridge] StatModifierSystem.Reapply() failed: {ex.Mes...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 130 | HIGH | `_log.LogError($"[ModPlatform] Config binding failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 152 | HIGH | `_log.LogError($"[ModPlatform] Failed to create subsystems: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 168 | HIGH | `_log.LogWarning($"[ModPlatform] Could not create packs directory: {ex.Message}")...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 204 | HIGH | `_log.LogError($"[ModPlatform] Failed to register StatModifierSystem: {ex.Message...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 215 | HIGH | `_log.LogError($"[ModPlatform] Failed to register PackUnitSpawner: {ex.Message}")...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 227 | HIGH | `_log.LogWarning($"[ModPlatform] WaveInjector failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 240 | HIGH | `_log.LogWarning($"[ModPlatform] FactionSystem failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 255 | HIGH | `_log.LogWarning($"[ModPlatform] VanillaCatalog build failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 265 | HIGH | `_log.LogWarning($"[ModPlatform] Unresolved component type: {unresolvedType}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 270 | HIGH | `_log.LogWarning($"[ModPlatform] ComponentMap validation failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 292 | HIGH | `_log.LogError($"[ModPlatform] Failed to start GameBridgeServer: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 314 | HIGH | `_log.LogWarning($"[ModPlatform] VanillaCatalog rebuild failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 332 | HIGH | `_log.LogWarning($"[ModPlatform] PackStatInjector failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 346 | HIGH | `_log.LogWarning($"[ModPlatform] Unit stat override re-apply failed: {ex.Message}...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 359 | HIGH | `_log.LogWarning($"[ModPlatform] YAML stat override re-apply failed: {ex.Message}...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 399 | HIGH | `_log.LogWarning($"[ModPlatform] Failed to disable pack {packId}: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 413 | HIGH | `_log.LogError($"[ModPlatform] Pack loading failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 436 | HIGH | `_log.LogWarning($"[ModPlatform] Failed to re-enable pack {originalPath}: {ex.Mes...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 448 | HIGH | `_log.LogWarning($"[ModPlatform] Loaded {result.LoadedPacks.Count} pack(s) with {...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 451 | HIGH | `_log.LogError($"  {error}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 463 | HIGH | `_log.LogError($"[ModPlatform] Failed to initialize PackUnitSpawner: {ex.Message}...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 475 | HIGH | `_log.LogError($"[ModPlatform] Failed to initialize AerialSpawnSystem: {ex.Messag...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 488 | HIGH | `_log.LogError($"[ModPlatform] Stat override application failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 504 | HIGH | `_log.LogError($"[ModPlatform] YAML stat override application failed: {ex.Message...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 555 | HIGH | `_log.LogError($"[ModPlatform] Failed to start hot reload: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 613 | HIGH | `_log.LogError($"[ModPlatform] Error handling hot reload completion: {ex.Message}...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 670 | HIGH | `_log.LogWarning($"[ModPlatform] Could not read manifest in {dir}: {ex.Message}")...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 695 | HIGH | `_log.LogError($"[ModPlatform] UI update failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 746 | HIGH | `_log.LogError($"[ModPlatform] Reload failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 779 | HIGH | `_log.LogError($"[ModPlatform] Failed to reload after toggle: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 800 | HIGH | `_log.LogWarning($"[ModPlatform] Failed to save disabled packs: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs | 829 | HIGH | `_log.LogWarning($"[ModPlatform] Failed to load disabled packs: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 395 | HIGH | `_log.LogWarning($"[RuntimeDriver] TryRegisterKeyInputSystem failed: {ex.Message}...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 451 | HIGH | `_log.LogWarning($"[RuntimeDriver] UiAssets initialization failed: {ex.Message}")...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 463 | HIGH | `_log.LogError($"[RuntimeDriver] ModPlatform initialization failed: {ex.Message}"...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 475 | HIGH | `_log.LogError($"[RuntimeDriver] MainThreadDispatcher setup failed: {ex.Message}"...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 490 | HIGH | `_log.LogError($"[RuntimeDriver] DebugOverlayBehaviour setup failed: {ex.Message}...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 577 | HIGH | `_log.LogWarning($"[RuntimeDriver] DFCanvas AddComponent failed, falling back to ...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 605 | HIGH | `_log.LogWarning($"[RuntimeDriver] NativeMenuInjector setup failed: {ex.Message}"...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 909 | HIGH | `_log.LogWarning($"[RuntimeDriver] Destroying stale UiEventInterceptor on '{inter...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 933 | HIGH | `_log.LogWarning($"[RuntimeDriver] UiEventInterceptor cleanup failed: {ex.Message...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 973 | HIGH | `_log.LogError($"[RuntimeDriver] IMGUI fallback ModMenuOverlay setup failed: {ex....` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 997 | HIGH | `_log.LogWarning($"[RuntimeDriver] HudIndicator setup failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 1070 | HIGH | `_log.LogWarning($"[RuntimeDriver] UGUI→ModPlatform wiring failed, activating IMG...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 1097 | HIGH | `_log.LogWarning($"[RuntimeDriver] DumpSystem registration failed: {ex.Message}")...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 1111 | HIGH | `_log.LogError($"[RuntimeDriver] ModPlatform.OnWorldReady failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 1123 | HIGH | `_log.LogError($"[RuntimeDriver] Pack loading failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 1134 | HIGH | `_log.LogError($"[RuntimeDriver] Hot reload startup failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\Plugin.cs | 1148 | HIGH | `_log.LogWarning($"[RuntimeDriver] Settings discovery failed: {ex.Message}");...` |
| C:\Users\koosh\Dino\src\Runtime\SystemEnumerator.cs | 46 | HIGH | `_log.LogError($"System enumeration failed: {ex}");...` |
| C:\Users\koosh\Dino\src\Runtime\SystemEnumerator.cs | 128 | HIGH | `_log.LogError($"  Failed to enumerate systems in '{world.Name}': {ex}");...` |
