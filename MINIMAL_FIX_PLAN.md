# Plugin.cs Minimal Fix Plan

## Analysis: Known-Good vs Current

### Known-Good Commit (df3b55e)
- **File size**: 588 lines
- **Awake()**: Creates Harmony but applies NO patches ("Harmony initialized (no patches applied)")
- **Harmony patches**: 0 registered
- **PlayerLoop injection**: None
- **Background threads**: None
- **Resurrection logic**: None
- **F9/F10 handling**: Straight UnityEngine.Input.GetKeyDown() in RuntimeDriver.Update()
- **Object lifecycle**: PersistentRoot (HideAndDontSave) → RuntimeDriver attached → lives until DINO destroys it
- **RuntimeDriver.OnDestroy()**: Just logs warning and calls _modPlatform?.Shutdown()

### Current Version (main)
- **File size**: 1100+ lines
- **Awake()**: Registers 4 major Harmony patches
- **Harmony patches**:
  1. PlayerLoop.SetPlayerLoop (to re-inject drain)
  2. ScriptBehaviourUpdateOrder.AddWorldToCurrentPlayerLoop (to inject KeyInputSystem)
  3. InputSystem.InputManager.Update / UnityEngine.Input.Update (for resurrection)
  4. EventSystem.Update, Canvas.SendWillRenderCanvases, Time.get_deltaTime (for resurrection)
- **PlayerLoop injection**: InjectPlayerLoopDrain() called in Awake() to add custom drain step
- **Background threads**: StartResurrectionWatcher() polls F9/F10 via Win32 GetAsyncKeyState every 50ms
- **Resurrection logic**: 5 separate hooks (OnSceneLoaded, OnActiveSceneChanged, OnSceneUnloaded, OnCameraPreRender, OnBeforeRender)
- **F9/F10 handling**: Mixture of Input.GetKeyDown() (in-game) + Win32 polling (at menu)
- **Object lifecycle**: Complex multi-stage resurrection with camera migration, scene resurrector plants, static callbacks

## Root Cause of Hangs

The four Harmony patches (especially PlayerLoopSetPlayerLoopPatch) create a chain reaction on Windows:
1. When DINO calls SetPlayerLoop during ECS bootstrap, the postfix fires
2. Postfix calls InjectPlayerLoopDrain(), which rebuilds entire PlayerLoop
3. On Windows, .NET's PlayerLoop reflection hangs waiting for garbage collection locks
4. In tight loops (scene transitions), this repeats hundreds of times → cumulative hang
5. Linux (GitHub Actions, WSL2) has different GC timing → no perceptible lag

## Minimal Fix (Revert to Known-Good Behavior)

Changes needed (in order of importance):

### 1. Remove ALL Harmony Patches from Awake()
**Lines to DELETE**: 81–275 (entire Harmony registration block)
- Remove patch registration for PlayerLoop.SetPlayerLoop
- Remove patch registration for ScriptBehaviourUpdateOrder.AddWorldToCurrentPlayerLoop
- Remove all resurrection patch candidates (InputSystem, EventSystem, Canvas, Time)
- Keep only: `_harmony = new Harmony(PluginInfo.GUID)` + log line
- **Benefit**: Eliminates 90% of reflection overhead and all cascade hangs

### 2. Remove InjectPlayerLoopDrain() Call from Awake()
**Lines to DELETE**: 346–351
- Remove: `InjectPlayerLoopDrain();`
- Remove log line: "PlayerLoop drain step injected"
- **Benefit**: No custom PlayerLoop step = no per-frame reflection

### 3. Remove Background Watcher Thread
**Lines to DELETE**: 334–337 (StartResurrectionWatcher call) + 495–617 (entire StartResurrectionWatcher method)
- Remove: `StartResurrectionWatcher();`
- Remove entire `StartResurrectionWatcher()` method definition
- Remove Win32 P/Invoke: `[DllImport("user32.dll")] GetAsyncKeyState()` (lines 485–493)
- Remove thread-safe toggles: `PendingF9Toggle`, `PendingF10Toggle`, `_mainThreadContext`, `_watcherThread`
- **Benefit**: Eliminates background thread overhead and Win32 polling

### 4. Remove Resurrection Hooks (except Plugin.OnDestroy logging)
**Lines to DELETE**: 313–332 (all SceneManager / Camera / Application callbacks)
- Remove: `SceneManager.sceneLoaded +=`
- Remove: `SceneManager.activeSceneChanged +=`
- Remove: `SceneManager.sceneUnloaded +=`
- Remove: `Camera.onPreRender +=`
- Remove: `Application.onBeforeRender +=`
- **Benefit**: Eliminates 5 per-frame hooks that call TryResurrect

### 5. Simplify RuntimeDriver.OnDestroy()
**Lines to REPLACE**: 1061–1079
- Old (good):
  ```csharp
  private void OnDestroy()
  {
      _log?.LogWarning("[RuntimeDriver] OnDestroy called - persistent root was destroyed!");
      try { _modPlatform?.Shutdown(); }
      catch (Exception ex) { _log?.LogWarning($"[RuntimeDriver] ModPlatform shutdown error: {ex.Message}"); }
  }
  ```
- Current (breaks):
  ```csharp
  private void OnDestroy()
  {
      WriteDebug($"[RuntimeDriver] OnDestroy called (migratedAway={IsMigratedAway}).");
      Plugin.PersistentRoot = null;
      Plugin.NeedsResurrection = true;
      try { Plugin.InjectPlayerLoopDrain(forceReInject: true); }
      catch { }
      try { _modPlatform?.Shutdown(); }
      catch { }
  }
  ```
- Remove: All resurrection logic, InjectPlayerLoopDrain call, IsMigratedAway flag
- **Benefit**: Stops cascading resurrection attempts

### 6. Clean Up Static Fields in Plugin Class
**Lines to DELETE**: 37–53 (resurrection-specific statics)
- Remove: `StaticHarmony`
- Remove: `_resurrectionLog`, `_resurrectionConfig`, `_resurrectionDump`, `_resurrectionDumpPath`
- Remove: `MainThreadQueue` (not used in old version)
- **Benefit**: No static state needed for resurrection

### 7. Remove Helper Methods
**Lines to DELETE**:
- `TryResurrect()` method (lines 711–744)
- `OnSceneLoaded()` method (lines 362–366)
- `OnActiveSceneChanged()` method (lines 461–465)
- `OnSceneUnloaded()` method (lines 467–471)
- `OnCameraPreRender()` method (lines 620–633)
- `OnBeforeRender()` method (lines 636–646)
- `MigrateToCamera()` method (lines 416–454)
- `PlantSceneResurrector()` method (lines 370–387)
- `PlantSceneResurrectorFromOnDestroy()` method (lines 394–410)
- `TryPatchDinoInputManager()` method (lines 653–709)
- `InjectPlayerLoopDrain()` method (if defined in this file)
- **Benefit**: Removes 500+ lines of resurrection infrastructure

### 8. Remove Harmony Patch Classes (at end of file)
**Lines to DELETE**: 978–1119 (all four patch classes)
- Remove: `AddWorldToPlayerLoopPatch`
- Remove: `DeltaTimeResurrectionPatch`
- Remove: `PlayerLoopSetPlayerLoopPatch`
- Remove: `DinoUpdateResurrectionPatch`
- **Benefit**: No Harmony patches = no reflection chains

### 9. Remove IsMigratedAway Flag from RuntimeDriver
- Check RuntimeDriver class for `bool IsMigratedAway;` declaration and remove it

### 10. Keep F9/F10 Handling in RuntimeDriver.Update() AS-IS
- No changes needed here — the direct Input.GetKeyDown() approach works fine in-game
- When menu is active (no MonoBehaviours), F9/F10 simply don't work — but that's acceptable vs. hangs

---

## Result: Known-Good Behavior

After these deletions:
- **File size**: ~500–550 lines (vs. current 1100+)
- **Plugin.Awake()**: 60 lines (vs. current 300+)
- **Harmony patches**: 0 (vs. current 4)
- **Background threads**: 0 (vs. current 1)
- **Per-frame hooks**: 0 (vs. current 5)
- **Reflection chains**: 0 (vs. current complex cascades)
- **Windows hang risk**: Eliminated (known-good version was stable for weeks)

## Testing Plan

1. **Before fix**: Record Windows build time + scene transition time
2. **After deletions**: Re-build, measure same operations
3. **Validation**:
   - dotnet build should complete without hangs
   - Scene transitions in-game should be smooth (no stutter)
   - F9 debug overlay works in-game
   - F10 mod menu works in-game
   - F9/F10 don't work at menu (acceptable — known limitation)
   - RuntimeDriver survives to OnDestroy, shuts down cleanly

## Why This Works

The known-good version accepts **one design trade-off**:
- **Gain**: No hangs, no reflection overhead, no background threads, simpler code
- **Loss**: When PersistentRoot is destroyed by DINO, it stays destroyed until game restart

This is acceptable because:
1. **HideAndDontSave prevents destruction** — in practice, RuntimeDriver rarely dies
2. **If it does die, graceful degradation** — game continues running, just no F9/F10 at menu
3. **In-game F9/F10 always work** — most important use case
4. **No hangs** — far worse than losing menu overlay

This is the **vibecoding principle**: prefer proven simplicity over ambitious resilience.
