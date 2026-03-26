# Plugin.cs Code Comparison: df3b55e vs main

## Plugin.Awake() Comparison

### Known-Good (df3b55e) - Lines 32–122

```csharp
private void Awake()
{
    Log = Logger;
    Log.LogInfo($"DINOForge Runtime v{PluginInfo.VERSION} loading...");

    // Config for debug features
    ConfigEntry<bool> dumpOnStartup = Config.Bind("Debug", "DumpOnStartup", true, ...);
    ConfigEntry<string> dumpOutputPath = Config.Bind("Debug", "DumpOutputPath", ...);

    // Detect game and log version compatibility info
    try
    {
        var bepinexVersion = typeof(BaseUnityPlugin).Assembly.GetName().Version?.ToString() ?? "unknown";
        Log.LogInfo($"DINOForge v{PluginInfo.VERSION} | BepInEx {bepinexVersion} | Unity {Application.unityVersion}");
        Log.LogInfo($"Platform: {Application.platform}");
    }
    catch (Exception ex)
    {
        Log.LogWarning($"Version detection failed: {ex.Message}");
    }

    // Harmony (available but unused per ADR-005)
    try
    {
        _harmony = new Harmony(PluginInfo.GUID);
        Log.LogInfo("Harmony initialized (no patches applied).");  // <-- KEY: NO PATCHES
    }
    catch (Exception ex)
    {
        Log.LogError($"Harmony init failed: {ex.Message}");
    }

    // Create a dedicated persistent GameObject that won't be destroyed.
    try
    {
        PersistentRoot = new GameObject("DINOForge_Root");
        PersistentRoot.hideFlags = HideFlags.HideAndDontSave;
        UnityEngine.Object.DontDestroyOnLoad(PersistentRoot);
        Log.LogInfo("[Plugin] Persistent root GameObject created.");
    }
    catch (Exception ex)
    {
        Log.LogError($"[Plugin] Failed to create persistent root: {ex.Message}");
        return;
    }

    // Add the runtime driver to the persistent root.
    try
    {
        RuntimeDriver driver = PersistentRoot.AddComponent<RuntimeDriver>();
        driver.Initialize(Logger, Config, dumpOnStartup.Value, dumpOutputPath.Value);
        Log.LogInfo("[Plugin] RuntimeDriver initialized on persistent root.");
    }
    catch (Exception ex)
    {
        Log.LogError($"[Plugin] RuntimeDriver setup failed: {ex.Message}");
    }

    WriteDebug("Awake completed");
    Log.LogInfo("DINOForge Runtime loaded successfully.");
}
```

**Total: ~80 lines, simple linear flow**

---

### Current (main) - Lines 56–355

```csharp
private void Awake()
{
    Log = Logger;
    Log.LogInfo($"DINOForge Runtime v{PluginInfo.VERSION} loading...");

    // Config for debug features
    ConfigEntry<bool> dumpOnStartup = Config.Bind("Debug", "DumpOnStartup", true, ...);
    ConfigEntry<string> dumpOutputPath = Config.Bind("Debug", "DumpOutputPath", ...);

    // Detect game and log version compatibility info
    try { ... LogInstallDiagnostics(); ... }
    catch (Exception ex) { ... }

    // Harmony patches  <-- STARTS 200-LINE BLOCK
    try
    {
        _harmony = new Harmony(PluginInfo.GUID);
        StaticHarmony = _harmony;

        // Patch PlayerLoop.SetPlayerLoop so our drain step re-injects itself after every call
        var setPlayerLoopMethod = typeof(PlayerLoop).GetMethod(...);
        if (setPlayerLoopMethod != null)
        {
            var postfix = new HarmonyMethod(typeof(PlayerLoopSetPlayerLoopPatch).GetMethod(nameof(PlayerLoopSetPlayerLoopPatch.Postfix)));
            _harmony.Patch(setPlayerLoopMethod, postfix: postfix);
            Log.LogInfo("[Plugin] Patched PlayerLoop.SetPlayerLoop to re-inject drain after every call.");
        }
        else { Log.LogWarning("[Plugin] Could not find PlayerLoop.SetPlayerLoop to patch."); }

        // Patch ScriptBehaviourUpdateOrder.AddWorldToCurrentPlayerLoop
        try
        {
            var scriptBehaviourType = System.Type.GetType("Unity.Entities.ScriptBehaviourUpdateOrder, Unity.Entities", throwOnError: false);
            if (scriptBehaviourType != null)
            {
                System.Reflection.MethodInfo? addWorldMethod = null;
                foreach (var m in scriptBehaviourType.GetMethods(...))
                {
                    if (m.Name == "AddWorldToCurrentPlayerLoop" || m.Name == "AppendWorldToCurrentPlayerLoop")
                    {
                        addWorldMethod = m;
                        break;
                    }
                }
                if (addWorldMethod != null)
                {
                    var addWorldPostfix = typeof(AddWorldToPlayerLoopPatch).GetMethod(nameof(AddWorldToPlayerLoopPatch.Postfix));
                    _harmony.Patch(addWorldMethod, postfix: new HarmonyMethod(addWorldPostfix));
                    Log.LogInfo($"[Plugin] Patched {scriptBehaviourType.Name}.{addWorldMethod.Name} to inject KeyInputSystem.");
                }
                else { Log.LogWarning("[Plugin] Could not find AddWorldToCurrentPlayerLoop on ScriptBehaviourUpdateOrder."); }
            }
            else { Log.LogWarning("[Plugin] ScriptBehaviourUpdateOrder type not found."); }
        }
        catch (Exception exAddWorld)
        {
            Log.LogWarning($"[Plugin] AddWorldToCurrentPlayerLoop patch failed: {exAddWorld.Message}");
        }

        // Patch a managed method called every frame for resurrection
        // Try InputSystem.InputManager.Update, UnityEngine.Input.Update, EventSystem.Update, Canvas.SendWillRenderCanvases, Time.get_deltaTime
        var resurrectPostfixMethod = typeof(DeltaTimeResurrectionPatch).GetMethod(nameof(DeltaTimeResurrectionPatch.Postfix));
        bool resurrectPatched = false;

        // Candidate 0: InputManager.Update
        try { ... (30+ lines of reflection) ... }
        catch (Exception exInput) { ... }

        // Candidate 1: EventSystem.Update
        if (!resurrectPatched) { try { ... (30+ lines of reflection) ... } catch (Exception ex2) { ... } }

        // Candidate 2: Canvas.SendWillRenderCanvases
        if (!resurrectPatched) { ... (10+ lines) ... }

        // Candidate 3: Time.get_deltaTime
        if (!resurrectPatched) { ... (10+ lines) ... }
    }
    catch (Exception ex)
    {
        Log.LogError($"Harmony init failed: {ex.Message}");
    }
    // <-- ENDS 200-LINE HARMONY BLOCK

    // Create a dedicated persistent GameObject
    try
    {
        PersistentRoot = new GameObject("DINOForge_Root");
        PersistentRoot.hideFlags = HideFlags.HideAndDontSave;
        UnityEngine.Object.DontDestroyOnLoad(PersistentRoot);
        Log.LogInfo("[Plugin] Persistent root GameObject created (DontDestroyOnLoad, HideAndDontSave).");
    }
    catch (Exception ex) { ... }

    // Add the runtime driver
    try
    {
        RuntimeDriver driver = PersistentRoot.AddComponent<RuntimeDriver>();
        driver.Initialize(Logger, Config, dumpOnStartup.Value, dumpOutputPath.Value);
        Log.LogInfo("[Plugin] RuntimeDriver initialized on persistent root.");
    }
    catch (Exception ex) { ... }

    // Capture state for static resurrection callback
    _resurrectionLog = Logger;
    _resurrectionConfig = Config;
    _resurrectionDump = dumpOnStartup.Value;
    _resurrectionDumpPath = dumpOutputPath.Value;

    // Register static SceneManager callbacks
    SceneManager.sceneLoaded -= OnSceneLoaded;
    SceneManager.sceneLoaded += OnSceneLoaded;
    SceneManager.activeSceneChanged -= OnActiveSceneChanged;
    SceneManager.activeSceneChanged += OnActiveSceneChanged;
    SceneManager.sceneUnloaded -= OnSceneUnloaded;
    SceneManager.sceneUnloaded += OnSceneUnloaded;

    // Camera.onPreRender fires every frame from DINO's own cameras
    Camera.onPreRender -= OnCameraPreRender;
    Camera.onPreRender += OnCameraPreRender;

    // Application.onBeforeRender fires every frame before rendering
    Application.onBeforeRender -= OnBeforeRender;
    Application.onBeforeRender += OnBeforeRender;

    // Background watcher thread
    StartResurrectionWatcher();

    Log.LogInfo("[Plugin] SceneManager resurrection hooks registered (+ Camera.onPreRender + bg watcher).");

    // Inject a custom step into Unity's PlayerLoop
    InjectPlayerLoopDrain();
    Log.LogInfo("[Plugin] PlayerLoop drain step injected (initial).");

    WriteDebug("Awake completed");
    Log.LogInfo("DINOForge Runtime loaded successfully.");
}
```

**Total: ~300 lines, heavily orchestrated**

---

## RuntimeDriver.OnDestroy() Comparison

### Known-Good (df3b55e)

```csharp
private void OnDestroy()
{
    // This should NOT normally fire since we use HideAndDontSave.
    // If it does, log it clearly.
    _log?.LogWarning("[RuntimeDriver] OnDestroy called - persistent root was destroyed!");

    try
    {
        _modPlatform?.Shutdown();
    }
    catch (Exception ex)
    {
        _log?.LogWarning($"[RuntimeDriver] ModPlatform shutdown error: {ex.Message}");
    }
}
```

**Total: 13 lines, just logs and shuts down**

---

### Current (main)

```csharp
private void OnDestroy()
{
    WriteDebug($"[RuntimeDriver] OnDestroy called (migratedAway={IsMigratedAway}).");

    // Signal resurrection watchers — static events will resurrect us next frame.
    Plugin.PersistentRoot = null;
    Plugin.NeedsResurrection = true;

    // Force-reinject the PlayerLoop drain
    try { Plugin.InjectPlayerLoopDrain(forceReInject: true); }
    catch { }

    try { _modPlatform?.Shutdown(); }
    catch { }
}
```

**Total: 13 lines, but with resurrection logic baked in**

---

## F9/F10 Handling Comparison

### Known-Good (df3b55e) - RuntimeDriver.Update()

```csharp
// Simple, direct Input polling (works when MonoBehaviour is alive)
if (Input.GetKeyDown(KeyCode.F9))
{
    if (_uguiReady && _dfCanvas != null)
        _dfCanvas.ToggleDebug();
    else
        _debugOverlay?.Toggle();
}

if (Input.GetKeyDown(KeyCode.F10))
{
    if (_uguiReady && _dfCanvas != null)
        _dfCanvas.ToggleModMenu();
    else
        _modMenuOverlay?.Toggle();
}
```

**Total: 15 lines, straightforward**
**Works**: In-game (MonoBehaviour is alive)
**Doesn't work**: At menu (MonoBehaviour destroyed)
**Acceptable**: Yes, in-game is primary use case

---

### Current (main) - Dual-path (Input + Win32)

RuntimeDriver.Update():
```csharp
if (Input.GetKeyDown(KeyCode.F9))
{
    try
    {
        WriteDebug("[RuntimeDriver] F9 pressed");
        if (_uguiReady && _dfCanvas != null) _dfCanvas.ToggleDebug();
        else _debugOverlay?.Toggle();
    }
    catch { }
}

if (Input.GetKeyDown(KeyCode.F10))
{
    try
    {
        WriteDebug("[RuntimeDriver] F10 pressed");
        if (_uguiReady && _dfCanvas != null) _dfCanvas.ToggleModMenu();
        else _modMenuHost?.Toggle();
    }
    catch { }
}
```

PLUS background watcher thread (lines 495–617):
```csharp
private static void StartResurrectionWatcher()
{
    // Capture Unity main-thread SynchronizationContext
    _mainThreadContext = System.Threading.SynchronizationContext.Current;
    ...

    _watcherThread = new System.Threading.Thread(() =>
    {
        WriteDebug("[Plugin] ResurrectionWatcher thread started");
        bool dinoPatched = false;
        var seenWorlds = new System.Collections.Generic.HashSet<string>();
        bool f9WasDown = false;
        bool f10WasDown = false;
        while (true)
        {
            System.Threading.Thread.Sleep(50); // 50ms poll loop

            try
            {
                // Poll F9/F10 via Win32 GetAsyncKeyState
                bool f9Down  = (GetAsyncKeyState(VK_F9)  & 0x8000) != 0;
                bool f10Down = (GetAsyncKeyState(VK_F10) & 0x8000) != 0;

                if (f9Down && !f9WasDown)
                {
                    WriteDebug("[Plugin] Win32: F9 pressed");
                    PendingF9Toggle = true;
                }
                if (f10Down && !f10WasDown)
                {
                    WriteDebug("[Plugin] Win32: F10 pressed");
                    PendingF10Toggle = true;
                }
                f9WasDown  = f9Down;
                f10WasDown = f10Down;

                if (NeedsResurrection) { ... }

                // Lazy-patch DINO's PlayerInputManager.Update()
                if (!dinoPatched && StaticHarmony != null) { dinoPatched = TryPatchDinoInputManager(); }

                // Periodically dump PlayerLoop from background thread
                if (NeedsResurrection) { ... }

                // Poll for new ECS worlds
                try
                {
                    foreach (var w in World.All)
                    {
                        if (!w.IsCreated) continue;
                        string key = w.Name + "_" + w.GetHashCode();
                        if (seenWorlds.Contains(key)) continue;
                        seenWorlds.Add(key);
                        ...
                        w.GetOrCreateSystem<Bridge.KeyInputSystem>();
                        ...
                    }
                }
                catch { }
            }
            catch { }
        }
    })
    { IsBackground = true, Name = "DINOForge-ResurrectionWatcher" };
    _watcherThread.Start();
}
```

**Total**: ~130 lines (20 in Update + 110 in StartResurrectionWatcher)
**Works**: In-game (Input) + Menu (Win32 polling)
**Cost**: Background thread, Win32 DLL import, platform-specific code, lock contention

---

## Summary Table

| Aspect | Known-Good (df3b55e) | Current (main) | Delta |
|--------|----------------------|----------------|-------|
| **File size** | 588 lines | 1100+ lines | +512 lines |
| **Plugin.Awake()** | 80 lines | 300 lines | +220 lines |
| **Harmony patches** | 0 | 4 | +4 |
| **Background threads** | 0 | 1 | +1 |
| **Per-frame hooks** | 0 | 5 | +5 |
| **Helper methods** | 2 | 12+ | +10 |
| **F9/F10 at menu** | No (acceptable) | Yes (complex) | Tradeoff |
| **Hang risk on Windows** | None (known stable) | CRITICAL | Major issue |
| **Code complexity** | Linear, simple | Branching, orchestrated | High |
| **Reflection overhead** | None | Heavy (patches + loop) | Significant |
| **Win32 platform dependency** | None | Yes (GetAsyncKeyState) | Added |

---

## Why df3b55e is Superior for Vibecoding

1. **Simplicity**: Anyone can understand 80-line Awake vs 300-line Awake with 4 patches
2. **Reliability**: Proven stable for weeks, no reflection hangs
3. **Debuggability**: Fewer moving parts = fewer failure modes
4. **Maintainability**: Less code = less debt = easier for agents to modify
5. **Performance**: No background threads, no Harmony reflection cascades
6. **Trade-off is acceptable**: Losing F9/F10 at menu << winning no hangs

This is exactly why CLAUDE.md says:
> **ALWAYS prefer** (in order):
&gt; 1. Direct use of an existing library/tool as-is
&gt; 2. Thin wrapper / adapter around an existing library
&gt; 3. Composition of multiple existing libraries
&gt; 4. Modified fork of an existing library (last resort before handroll)
>
> **ONLY handroll when** no existing solution covers the need OR wrapping would be more complex than a simple implementation

The current version **handrolled resurrection logic** when the simple solution was to **accept graceful degradation** (F9/F10 unavailable at menu after rare destruction).
