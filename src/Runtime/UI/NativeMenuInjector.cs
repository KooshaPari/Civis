#nullable enable
using System;
using BepInEx.Logging;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.SceneManagement;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Monitors Unity scene changes and injects a "Mods" button into DINO's native
    /// UGUI menus (main menu / pause menu) so players can open the DINOForge mod menu
    /// without knowing the F10 hotkey.
    ///
    /// Strategy:
    ///   1. Subscribe to <see cref="SceneManager.activeSceneChanged"/> on Awake.
    ///   2. Every <see cref="RescanInterval"/> seconds (and on each scene change) call
    ///      <see cref="TryInjectMenuButton"/>.
    ///   3. Scan all active canvases for a "Settings" or "Options" button.
    ///   4. Clone that button, label it "Mods", and wire its onClick to
    ///      <see cref="ModMenuOverlay.Toggle"/>.
    ///   5. Stop scanning once injection succeeds; resume if the injected button is destroyed.
    ///
    /// Graceful failure: any exception during injection is caught and logged as a warning.
    /// The component never throws to its caller.
    /// </summary>
    public class NativeMenuInjector : MonoBehaviour
    {
        // ------------------------------------------------------------------ //
        // Background-thread scan trigger
        // ------------------------------------------------------------------ //

        /// <summary>
        /// Called by the background watcher thread every ~5 seconds to re-scan for menu buttons.
        /// Set by RuntimeDriver.Initialize() after the injector is added.
        /// </summary>
        public static System.Action? OnScanNeeded;

        /// <summary>
        /// The <see cref="UnityEngine.GameObject.name"/> of the repurposed "Mods" button.
        /// Set when <see cref="InjectButton"/> repurposes an existing Options button.
        /// Read by <see cref="UI.ModsButtonTextPatch"/> to intercept UiGrid text overwrite.
        /// </summary>
        public static string? RepurposedModsButtonGoName { get; private set; }

        // ------------------------------------------------------------------ //
        // Well-known canvas names to check (case-insensitive prefix/substring)
        // ------------------------------------------------------------------ //
        private static readonly string[] CanvasCandidateNames =
        {
            "MainMenu",
            "PauseMenu",
            "SettingsMenu",
            "UI",
            "HUD",
            "Menu",
            "Canvas",
        };

        /// <summary>Interval in seconds between injection re-scan attempts.</summary>
        private const float RescanInterval = 2f;
        private const float ClickDebounceSeconds = 0.2f;

        private ManualLogSource? _log;
        private IModMenuHost? _menuHost;

        private Button? _injectedButton;
        private bool _injected;
        private float _rescanTimer;
        private float _lastClickTimeUnscaled = -10f;
        private System.Collections.Generic.List<Button>? _allOptionsButtons;

        // Text re-enforcement: after injection, re-assert "Mods" text every N frames
        // in case UiGrid or any internal update reverts it via a path Harmony doesn't cover.
        private int _textEnforceFrame;
        private const int TextEnforceInterval = 10; // every 10 frames (~6x/sec at 60fps)

        // ===== DIAGNOSTIC FIELDS =====
        private readonly string _sessionId = System.Guid.NewGuid().ToString().Substring(0, 8);
        private int _injectionAttemptCount;
        private long _buttonClickCount;

        // ── ISSUE-044: InitialGameLoader auto-advance (skips splash screen) ──────
        private bool _anyKeyPatchApplied;
        // Static re-entrancy guard: prevents double LoadScene(1) calls when the scene transition
        // destroys RuntimeDriver and a new one is created before the old TryInjectMenuButton returns.
        private static volatile bool _s_sceneTransitionGuard;

        // ------------------------------------------------------------------ //
        // Public wiring surface
        // ------------------------------------------------------------------ //

        /// <summary>
        /// Provides the mod menu overlay that the injected button will toggle.
        /// Called by <see cref="RuntimeDriver"/> immediately after AddComponent.
        /// </summary>
        /// <param name="overlay">The persistent <see cref="ModMenuOverlay"/> instance.</param>
        public void SetModMenuHost(IModMenuHost menuHost)
        {
            _menuHost = menuHost;
        }

        /// <summary>
        /// Sets the BepInEx logger used for injection status messages.
        /// </summary>
        /// <param name="log">Logger instance from the RuntimeDriver.</param>
        public void SetLogger(ManualLogSource log)
        {
            _log = log;
        }

        // ------------------------------------------------------------------ //
        // MonoBehaviour lifecycle
        // ------------------------------------------------------------------ //

        private void Awake()
        {
            SceneManager.activeSceneChanged += OnActiveSceneChanged;
            LogInfo($"[NativeMenuInjector::{_sessionId}] ===== PLUGIN SESSION START ===== Awake at {System.DateTime.UtcNow:HH:mm:ss.fff} UTC");
            LogInfo("[NativeMenuInjector] Subscribed to scene changes.");
        }

        private void Start()
        {
            // Reset the scene transition guard for the new RuntimeDriver's NativeMenuInjector.
            // The old RuntimeDriver set _s_sceneTransitionGuard=true before calling LoadScene(1).
            _s_sceneTransitionGuard = false;
            LogInfo($"[NativeMenuInjector::{_sessionId}] Start() called at {System.DateTime.UtcNow:HH:mm:ss.fff} UTC");
            TryInjectMenuButton();
        }

        private void Update()
        {
            // Tick delayed auto-checkpoint screenshot
            TickAutoCheckpoint();

            // Screenshot-on-demand: check trigger file every ~20 frames (~3x/sec at 60fps)
            _screenshotCheckFrames++;
            if (_screenshotCheckFrames >= 20)
            {
                _screenshotCheckFrames = 0;
                CheckScreenshotRequest();
            }

            // If we have already injected and the button is still alive, re-enforce button text.
            if (_injected && _injectedButton != null)
            {
                _textEnforceFrame++;
                if (_textEnforceFrame >= TextEnforceInterval)
                {
                    _textEnforceFrame = 0;
                    EnforceModsButtonText();
                }
                return;
            }

            // Button was destroyed (e.g. scene unloaded) — reset and re-scan.
            if (_injected && _injectedButton == null)
            {
                LogWarning($"[NativeMenuInjector::{_sessionId}] ⚠ INJECTED BUTTON WAS DESTROYED! Resetting injection flag at {System.DateTime.UtcNow:HH:mm:ss.fff} UTC");
                _injected = false;
                RepurposedModsButtonGoName = null;
            }

            _rescanTimer += Time.deltaTime;
            if (_rescanTimer < RescanInterval) return;

            _rescanTimer = 0f;
            TryInjectMenuButton();
        }

        private int _screenshotCheckFrames;
        private string? _pendingScreenshotPath;
        private System.DateTime _screenshotRequestedAtUtc = System.DateTime.MinValue;

        private void CheckScreenshotRequest()
        {
            try
            {
                string bepRoot = BepInEx.Paths.BepInExRootPath;
                string reqFile = System.IO.Path.Combine(bepRoot, "dinoforge_screenshot_request.txt");
                string doneFile = System.IO.Path.Combine(bepRoot, "dinoforge_screenshot_done.txt");

                if (_pendingScreenshotPath != null)
                {
                    // Wait for Unity to actually write the file before signaling done
                    var fi = new System.IO.FileInfo(_pendingScreenshotPath);
                    fi.Refresh();
                    if (fi.Exists && fi.Length > 1000 && fi.LastWriteTimeUtc > _screenshotRequestedAtUtc)
                    {
                        System.IO.File.WriteAllText(doneFile, _pendingScreenshotPath);
                        WriteDebug($"[Screenshot] done (verified {fi.Length} bytes): {_pendingScreenshotPath}");
                        _pendingScreenshotPath = null;
                    }
                    else
                    {
                        WriteDebug($"[Screenshot] pending, file not ready: {_pendingScreenshotPath} (exists={fi.Exists}, size={fi.Length}, written={fi.LastWriteTimeUtc:HH:mm:ss.fff})");
                    }
                }
                else if (System.IO.File.Exists(reqFile))
                {
                    string path = System.IO.File.ReadAllText(reqFile).Trim();
                    System.IO.File.Delete(reqFile);
                    if (string.IsNullOrEmpty(path))
                        path = System.IO.Path.Combine(bepRoot, "screenshot.png");
                    // Delete stale file so we can detect when Unity writes the new one
                    try { if (System.IO.File.Exists(path)) System.IO.File.Delete(path); } catch { }
                    _screenshotRequestedAtUtc = System.DateTime.UtcNow;
                    WriteDebug($"[Screenshot] requested (Update/main thread) at {_screenshotRequestedAtUtc:HH:mm:ss.fff}: {path}");
                    ScreenCapture.CaptureScreenshot(path);
                    _pendingScreenshotPath = path;
                }
            }
            catch { }
        }

        private void OnDestroy()
        {
            SceneManager.activeSceneChanged -= OnActiveSceneChanged;
            LogInfo($"[NativeMenuInjector::{_sessionId}] OnDestroy called. Injector cleanup complete.");
        }

        // ------------------------------------------------------------------ //
        // Text re-enforcement
        // ------------------------------------------------------------------ //

        /// <summary>
        /// Called every <see cref="TextEnforceInterval"/> frames after injection to re-assert
        /// "Mods" on all Text/TMP_Text children of the repurposed button.  This handles the case
        /// where UiGrid or any other system reverts the label via a code path Harmony doesn't cover.
        /// </summary>
        private void EnforceModsButtonText()
        {
            if (_injectedButton == null) return;
            bool changed = false;
            foreach (UnityEngine.UI.Text t in _injectedButton.GetComponentsInChildren<UnityEngine.UI.Text>(true))
            {
                if (string.Compare(t.text, "Options", System.StringComparison.OrdinalIgnoreCase) == 0
                    || string.Compare(t.text, "OPTIONS", System.StringComparison.OrdinalIgnoreCase) == 0)
                {
                    t.text = "Mods";
                    changed = true;
                }
            }
            foreach (TMPro.TMP_Text t in _injectedButton.GetComponentsInChildren<TMPro.TMP_Text>(true))
            {
                if (string.Compare(t.text, "Options", System.StringComparison.OrdinalIgnoreCase) == 0
                    || string.Compare(t.text, "OPTIONS", System.StringComparison.OrdinalIgnoreCase) == 0)
                {
                    t.text = "Mods";
                    changed = true;
                }
            }
            if (changed)
            {
                LogInfo($"[NativeMenuInjector] EnforceModsButtonText: re-set reverted label to 'Mods'");
            }
        }

        // ------------------------------------------------------------------ //
        // Scene change handler
        // ------------------------------------------------------------------ //

        private void OnActiveSceneChanged(Scene previous, Scene next)
        {
            LogInfo($"[NativeMenuInjector] Scene changed: {previous.name} → {next.name}. Re-scanning for menu.");
            _injected = false;
            _injectedButton = null;
            RepurposedModsButtonGoName = null;
            _rescanTimer = 0f;
            // Reset the guard so TryInjectMenuButton can run for the new scene.
            // The guard was set true during the LoadScene(1) call that triggered this scene change.
            _s_sceneTransitionGuard = false;
            TryInjectMenuButton();
        }

        // ------------------------------------------------------------------ //
        // Injection logic
        // ------------------------------------------------------------------ //

        /// <summary>
        /// Attempts to locate a Settings/Options button in any active canvas and injects
        /// a sibling "Mods" button next to it.  Safe to call multiple times; idempotent
        /// once <c>_injected</c> is true.
        /// </summary>
        internal void TryInjectMenuButton()
        {
            // GUARD: Prevent re-entrant LoadScene calls. When SceneManager.LoadScene(1) is
            // called below, it synchronously destroys the RuntimeDriver (even though it has
            // DontDestroyOnLoad) and triggers the creation of a new RuntimeDriver via
            // resurrection. The new RuntimeDriver's TryInjectMenuButton runs before the old
            // call stack unwinds, leading to a second LoadScene(1) call on the same frame.
            // That second call destabilizes the scene transition and crashes the game.
            if (_s_sceneTransitionGuard) return;
            _s_sceneTransitionGuard = true;

            _injectionAttemptCount++;
            long attemptId = _injectionAttemptCount;

            try
            {
                LogInfo($"[NativeMenuInjector::{_sessionId}] ═══ INJECTION ATTEMPT #{attemptId} at {System.DateTime.UtcNow:HH:mm:ss.fff} UTC ═══");

                // If already injected and button is alive, skip scan
                if (_injected && _injectedButton != null)
                {
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}: Already injected + button alive, skipping scan");
                    return;
                }

                Canvas[] allCanvases = Resources.FindObjectsOfTypeAll<Canvas>();
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}: Scan started — found {allCanvases.Length} canvases total");
                WriteDebug($"[{_sessionId}] Attempt#{attemptId}: Scan started — {allCanvases.Length} canvases");
                WriteDebug($"[{_sessionId}] Attempt#{attemptId}: All canvases dump:");
                foreach (Canvas c in allCanvases)
                    WriteDebug($"[{_sessionId}]   Canvas '{c.name}' active={c.gameObject.activeInHierarchy}");

                int activeCount = 0;

                foreach (Canvas canvas in allCanvases)
                {
                    // Check if canvas is active in hierarchy
                    if (!IsCanvasActive(canvas))
                    {
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   Canvas '{canvas.name}': INACTIVE (skipped)");
                        continue;
                    }
                    activeCount++;

                    // Search all active canvases regardless of name — the DINO menu
                    // canvas name may vary; we rely on finding the Settings/Options button.
                    WriteDebug($"[{_sessionId}] Attempt#{attemptId} Canvas '{canvas.name}': searching for buttons...");

                    Button? settingsButton = FindSettingsButton(canvas);
                    if (settingsButton == null)
                    {
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   Canvas '{canvas.name}': NO Settings/Options button found");
                        continue;
                    }

                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId} ✓✓✓ SUCCESS FOUND Settings button '{settingsButton.name}' in canvas '{canvas.name}'. INJECTING Mods button...");

                    InjectButton(settingsButton, attemptId);

                    if (_injected)
                    {
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId} ✓✓✓✓✓ INJECTION SUCCESSFUL! Mods button is now ACTIVE.");
                        // Auto-checkpoint screenshot: schedule 180 frames later (Update-based, may not fire)
                        TakeAutoCheckpointScreenshot("cp1_mods_injected", 180);
                        return;
                    }
                }

                // ── ISSUE-044 InitialGameLoader auto-advance ─────────────────────
                // If the game is stuck on InitialGameLoader waiting for Input.anyKey,
                // skip to scene 1 (main menu). We call LoadScene and immediately return —
                // OnActiveSceneChanged will fire when the scene is ready and trigger re-scan.
                if (attemptId >= 2 && !_anyKeyPatchApplied)
                {
                    bool hasInitialLoader = false;
                    foreach (Canvas c in allCanvases)
                    {
                        if (c.name != null && c.name.IndexOf("InitialGameLoader", StringComparison.OrdinalIgnoreCase) >= 0 && c.gameObject.activeInHierarchy)
                        {
                            hasInitialLoader = true;
                            break;
                        }
                    }
                    if (hasInitialLoader)
                    {
                        _anyKeyPatchApplied = true;  // prevent re-triggering
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId} — InitialGameLoader stuck. Loading scene 1 to skip splash screen.");
                        WriteDebug($"[{_sessionId}] InitialGameLoader auto-advance: SceneManager.LoadScene(1)");
                        SceneManager.LoadScene(1);
                        return;  // IMPORTANT: return immediately; OnActiveSceneChanged will re-trigger scan
                    }
                }

                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId} SCAN COMPLETE: {allCanvases.Length} total, {activeCount} active searched, 0 Settings buttons found. Will retry in {RescanInterval}s.");
            }
            catch (Exception ex)
            {
                LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId} TryInjectMenuButton EXCEPTION: {ex.Message}\n{ex.StackTrace}");
            }
        }

        /// <summary>
        /// Searches a canvas for buttons whose label contains "Settings" or "Options".
        /// If multiple "Options" buttons are found, stores them so we can repurpose the last one.
        /// </summary>
        private Button? FindSettingsButton(Canvas canvas)
        {
            try
            {
                // Try to find Settings button
                Button? settings = NativeUiHelper.FindButtonByText(canvas.transform, "Settings");
                if (settings != null)
                {
                    LogInfo($"[NativeMenuInjector]     Found 'Settings' button: '{settings.name}'");
                    return settings;
                }

                // Find ALL "Options" buttons (case-insensitive)
                Button[] allButtons = canvas.GetComponentsInChildren<Button>(includeInactive: false);
                System.Collections.Generic.List<Button> optionsButtons = new System.Collections.Generic.List<Button>();
                foreach (Button b in allButtons)
                {
                    string label = NativeUiHelper.GetButtonText(b);
                    if (label.IndexOf("Options", StringComparison.OrdinalIgnoreCase) >= 0)
                    {
                        optionsButtons.Add(b);
                        LogInfo($"[NativeMenuInjector]     Found 'Options' button: '{b.name}' (count: {optionsButtons.Count})");
                    }
                }

                // Store all found Options buttons; InjectButton will repurpose whichever is best.
                if (optionsButtons.Count >= 1)
                {
                    _allOptionsButtons = optionsButtons;
                    LogInfo($"[NativeMenuInjector]     Found {optionsButtons.Count} 'Options' button(s); will repurpose for Mods");
                    return optionsButtons[0];
                }

                // Log all buttons found for diagnostics
                LogInfo($"[NativeMenuInjector]     No 'Settings' or 'Options' button found in canvas. Dumping all {allButtons.Length} active buttons:");
                foreach (Button b in allButtons)
                {
                    string label = b.GetComponentInChildren<UnityEngine.UI.Text>()?.text ?? "(no text)";
                    LogInfo($"[NativeMenuInjector]       Button '{b.name}' label='{label}'");
                }
                return null;
            }
            catch (Exception ex)
            {
                LogWarning($"[NativeMenuInjector] FindSettingsButton exception: {ex.Message}\n{ex.StackTrace}");
                return null;
            }
        }

        /// <summary>
        /// Enforces consistent state on an existing Mods button without re-cloning.
        /// Sets all Text/TMPro children to "Mods", ensures onClick has OnModsButtonClicked listener,
        /// and ensures targetGraphic is properly set for visual state transitions.
        /// </summary>
        private void EnforceModsButtonState(Button modsButton, long attemptId)
        {
            // Set all text components to "Mods"
            foreach (UnityEngine.UI.Text legacyText in modsButton.GetComponentsInChildren<UnityEngine.UI.Text>(true))
            {
                legacyText.text = "Mods";
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - Enforced Text '{legacyText.name}' to 'Mods'");
            }
            System.Type? tmpType = System.Type.GetType("TMPro.TMP_Text, Unity.TextMeshPro");
            if (tmpType != null)
            {
                foreach (Component c in modsButton.GetComponentsInChildren(tmpType, true))
                {
                    tmpType.GetProperty("text")?.SetValue(c, "Mods");
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - Enforced TMP_Text '{c.name}' to 'Mods'");
                }
            }

            // Ensure onClick only has OnModsButtonClicked
            modsButton.onClick.RemoveAllListeners();
            modsButton.onClick.AddListener(OnModsButtonClicked);
            LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - Wired onClick listener to OnModsButtonClicked");

            // Ensure targetGraphic is set for visual state transitions
            if (modsButton.targetGraphic == null)
            {
                Image? fallbackImage = modsButton.GetComponentInChildren<UnityEngine.UI.Image>(true);
                if (fallbackImage != null)
                {
                    modsButton.targetGraphic = fallbackImage;
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - Set targetGraphic fallback to '{fallbackImage.name}'");
                }
            }
        }

        /// <summary>
        /// Repurposes the last "Options" button as "Mods" if 2+ exist, otherwise clones the reference button.
        /// Labels it "Mods" and wires its onClick event.
        /// </summary>
        private void InjectButton(Button settingsButton, long attemptId)
        {
            try
            {
                if (settingsButton == null)
                {
                    LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId} InjectButton called with NULL settingsButton! ABORT.");
                    return;
                }

                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId} InjectButton starting with settingsButton='{settingsButton.name}'");

                // Always clone a new button — NEVER repurposes/replaces the original Options button.
                // When 2+ Options buttons exist, clone from the last one so the new Mods button
                // appears AFTER all Options buttons. When 1 or 0 exist, clone from settingsButton
                // and position BEFORE it (original behavior).
                Button? modsButton = null;
                Button? cloneSource = settingsButton;
                Button? positionAfterSibling = null; // null = position before settingsButton

                if (_allOptionsButtons != null && _allOptionsButtons.Count >= 1)
                {
                    // Use the last Options button as clone source; position new Mods button AFTER it.
                    cloneSource = _allOptionsButtons[_allOptionsButtons.Count - 1];
                    positionAfterSibling = cloneSource;
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 1: Cloning from last 'Options' button '{cloneSource.name}' — Mods will appear AFTER Options");
                }
                else
                {
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 1: Cloning from Settings button '{cloneSource.name}' to create new 'Mods' button");
                }

                // Guard: don't inject twice into the same parent
                Transform parent = cloneSource.transform.parent;
                if (parent != null)
                {
                    for (int i = 0; i < parent.childCount; i++)
                    {
                        if (parent.GetChild(i).name.StartsWith("DINOForge_ModsButton", StringComparison.OrdinalIgnoreCase))
                        {
                            Button existing = parent.GetChild(i).GetComponent<Button>();
                            if (existing != null && existing.gameObject.activeInHierarchy)
                            {
                                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 1R: Mods button already present; re-enforcing state...");
                                EnforceModsButtonState(existing, attemptId);
                                _injectedButton = existing;
                                _injected = true;
                                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId} ✓ Mods button already present; SKIPPING re-inject, using existing.");
                                return;
                            }
                        }
                    }
                }

                modsButton = NativeUiHelper.CloneButton(cloneSource, "Mods");

                if (modsButton == null)
                {
                    LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   ⚠ STEP 1 FAILED: CloneButton returned null! ABORT.");
                    return;
                }

                // Register cloned button name with Harmony text-intercept patch
                RepurposedModsButtonGoName = modsButton.gameObject.name;
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 1 OK: Cloned button '{modsButton.name}', registered for text intercept.");
                SyncButtonVisualStyle(modsButton, cloneSource, attemptId);

                // STEP 1.5: Enforce text — cloned button inherits source text ("Options"), must override
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 1.5: Enforcing 'Mods' text on all text components...");
                foreach (UnityEngine.UI.Text legacyText in modsButton.GetComponentsInChildren<UnityEngine.UI.Text>(true))
                {
                    legacyText.text = "Mods";
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - Set Text '{legacyText.name}' to 'Mods'");
                }
                // TMPro via reflection to avoid hard compile dependency
                System.Type? tmpType2 = System.Type.GetType("TMPro.TMP_Text, Unity.TextMeshPro");
                if (tmpType2 != null)
                {
                    foreach (Component c in modsButton.GetComponentsInChildren(tmpType2, true))
                    {
                        tmpType2.GetProperty("text")?.SetValue(c, "Mods");
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - Set TMP_Text '{c.name}' to 'Mods'");
                    }
                }

                // Position Mods button: AFTER last Options button when multiple exist,
                // BEFORE Settings button when only one exists.
                RectTransform modsRect = modsButton.GetComponent<RectTransform>();
                RectTransform settingsRect = settingsButton.GetComponent<RectTransform>();
                RectTransform? siblingRef = null;
                if (modsRect != null)
                {
                    if (positionAfterSibling != null)
                    {
                        // 2+ Options buttons: place Mods AFTER the last Options button.
                        RectTransform? lastOptionsRect = positionAfterSibling.GetComponent<RectTransform>();
                        if (lastOptionsRect != null)
                        {
                            NativeUiHelper.PositionAfterSibling(modsRect, lastOptionsRect);
                            siblingRef = lastOptionsRect;
                            LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 2 OK: Positioned AFTER last Options (sibling index: {modsButton.transform.GetSiblingIndex()})");
                        }
                        else
                        {
                            LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 2: lastOptionsRect is null, falling back to before Settings");
                            NativeUiHelper.PositionBeforeSibling(modsRect, settingsRect);
                            siblingRef = settingsRect;
                        }
                    }
                    else
                    {
                        // 1 or 0 Options buttons: place Mods BEFORE Settings button.
                        NativeUiHelper.PositionBeforeSibling(modsRect, settingsRect);
                        siblingRef = settingsRect;
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 2 OK: Positioned BEFORE Settings (sibling index: {modsButton.transform.GetSiblingIndex()})");
                    }
                }

                // Force layout rebuild so VerticalLayoutGroup/ContentSizeFitter includes the new button.
                if (siblingRef != null)
                {
                    Transform layoutParent = siblingRef.parent;
                    if (layoutParent != null)
                    {
                        var layoutRt = layoutParent.GetComponent<RectTransform>();
                        if (layoutRt != null)
                        {
                            UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(layoutRt);
                            LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 2 LAYOUT: Forced rebuild on '{layoutParent.name}'");
                        }
                        if (layoutParent.parent != null)
                        {
                            var gpRt = layoutParent.parent.GetComponent<RectTransform>();
                            if (gpRt != null)
                            {
                                UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(gpRt);
                                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 2 LAYOUT: Forced rebuild on grandparent '{layoutParent.parent.name}'");
                            }
                        }
                        UnityEngine.Canvas.ForceUpdateCanvases();
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 2 LAYOUT: Canvas.ForceUpdateCanvases() called");
                    }
                }
                else
                {
                    LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   ⚠ STEP 2 WARN: Could not get RectTransform for modsButton");
                }

                // Ensure button is fully interactive
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 3: Ensuring button is fully interactive...");
                modsButton.gameObject.SetActive(true);
                modsButton.interactable = true;
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 3 OK: Button activated: active={modsButton.gameObject.activeSelf}, interactable={modsButton.interactable}");

                // Ensure CanvasGroup doesn't block interaction
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 4: Checking CanvasGroup...");
                CanvasGroup? cg = modsButton.GetComponent<CanvasGroup>();
                if (cg != null)
                {
                    cg.interactable = true;
                    cg.blocksRaycasts = true;
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 4 OK: CanvasGroup configured (interactable={cg.interactable}, blocksRaycasts={cg.blocksRaycasts})");
                }
                else
                {
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 4 INFO: No CanvasGroup on button (OK, not required)");
                }

                // ===== STEP 5: RAYCAST DIAGNOSTICS =====
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 5: Raycast diagnostics...");

                // Check button's own raycast target
                Image? btnImage = modsButton.targetGraphic as Image;
                if (btnImage != null)
                {
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - targetGraphic raycastTarget: {btnImage.raycastTarget}");
                    if (!btnImage.raycastTarget)
                    {
                        LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     ⚠ raycastTarget is FALSE - ENABLING");
                        btnImage.raycastTarget = true;
                    }
                }
                else
                {
                    LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     ⚠ targetGraphic is not an Image or is null");
                }

                // Check all parent CanvasGroups
                CanvasGroup[] parentCGs = modsButton.GetComponentsInParent<CanvasGroup>();
                if (parentCGs.Length > 0)
                {
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     Found {parentCGs.Length} parent CanvasGroup(s):");
                    foreach (var parentCg in parentCGs)
                    {
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}       - CanvasGroup '{parentCg.gameObject.name}': blocksRaycasts={parentCg.blocksRaycasts}, interactable={parentCg.interactable}");
                        if (!parentCg.blocksRaycasts)
                        {
                            LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}       ⚠ CanvasGroup '{parentCg.gameObject.name}' has blocksRaycasts=FALSE - may block raycasts");
                        }
                    }
                }
                else
                {
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     No parent CanvasGroups found");
                }

                // Check sorting order
                Canvas? canvas = modsButton.GetComponentInParent<Canvas>();
                if (canvas != null)
                {
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     Canvas '{canvas.name}': sortingOrder={canvas.sortingOrder}, renderMode={canvas.renderMode}");
                }
                else
                {
                    LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     ⚠ No parent Canvas found");
                }

                // Check for GraphicRaycaster on parent canvas
                if (canvas != null)
                {
                    GraphicRaycaster? raycaster = canvas.GetComponent<GraphicRaycaster>();
                    if (raycaster != null)
                    {
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     GraphicRaycaster on canvas: enabled={raycaster.enabled}");
                        if (!raycaster.enabled)
                        {
                            LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     ⚠ GraphicRaycaster is disabled - ENABLING");
                            raycaster.enabled = true;
                        }
                    }
                    else
                    {
                        LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     ⚠ No GraphicRaycaster on canvas - raycasts may not work");
                    }
                }
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 5 OK: Raycast diagnostics complete");
                // ===== END RAYCAST DIAGNOSTICS =====

                // STEP 6: Wire onClick
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 6: Wiring onClick listener...");
                RewireModsButtonClick(modsButton, attemptId);
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 6 OK: onClick listener attached");

                // STEP 7: Fix EventSystem navigation conflict
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 7: Configuring EventSystem selection...");
                try
                {
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     [7.1] Getting EventSystem.current...");
                    EventSystem es = EventSystem.current;
                    if (es != null)
                    {
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     [7.2] EventSystem found, getting current selection...");
                        GameObject? currentSelected = es.currentSelectedGameObject;
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     [7.3] Current selection = {currentSelected?.name ?? "NULL"}");

                        // Do not force-select the injected button. Taking focus here can couple it
                        // into native submit/navigation flows and trigger non-DINO handlers.
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     [7.4] Leaving EventSystem selection unchanged for native-menu safety.");

                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     [7.7] Setting navigation mode to None...");
                        Navigation modsNav = modsButton.navigation;
                        modsNav.mode = Navigation.Mode.None;
                        modsButton.navigation = modsNav;
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     [7.8] Mods button navigation mode: {modsNav.mode} (ISOLATED)");

                        Navigation settingsNav = settingsButton.navigation;
                        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     [7.9] Settings button navigation mode: {settingsNav.mode}");
                    }
                    else
                    {
                        LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     ⚠ EventSystem.current is NULL!");
                    }
                }
                catch (Exception exEs)
                {
                    LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     ⚠ EventSystem fix exception TYPE: {exEs.GetType().Name}");
                    LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     ⚠ Message: '{exEs.Message}'");
                    LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     ⚠ StackTrace: {exEs.StackTrace}");
                }
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 7 OK: EventSystem configuration complete");

                // STEP 8: Final button state verification
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 8: FINAL BUTTON STATE VERIFICATION");
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - gameObject.activeSelf: {modsButton.gameObject.activeSelf}");
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - gameObject.activeInHierarchy: {modsButton.gameObject.activeInHierarchy}");
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - interactable: {modsButton.interactable}");
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - navigation.mode: {modsButton.navigation.mode}");
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - targetGraphic.raycastTarget: {modsButton.targetGraphic?.raycastTarget ?? false}");
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - sibling_index: {modsButton.transform.GetSiblingIndex()}");
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     - menu_host_ref: {(_menuHost != null ? "READY" : "NULL")}");
                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}   STEP 8 OK: All checks passed");

                _injectedButton = modsButton;
                _injected = true;

                LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId} ✓✓✓✓✓✓ MODS BUTTON INJECTION FULLY SUCCESSFUL ✓✓✓✓✓✓");
            }
            catch (Exception ex)
            {
                LogWarning($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId} ⚠⚠⚠ InjectButton EXCEPTION: {ex.Message}\n{ex.StackTrace}");
            }
        }

        // ------------------------------------------------------------------ //
        // Button click handler
        // ------------------------------------------------------------------ //

        private void OnModsButtonClicked()
        {
            _buttonClickCount++;
            long clickId = _buttonClickCount;

            try
            {
                LogInfo($"[NativeMenuInjector::{_sessionId}] ═══ MODS BUTTON CLICKED #{clickId} at {System.DateTime.UtcNow:HH:mm:ss.fff} UTC ═══");

                if (_menuHost == null)
                {
                    LogWarning($"[NativeMenuInjector::{_sessionId}] Click#{clickId} ⚠ menu host reference is NULL! Cannot toggle menu.");
                    return;
                }

                float now = Time.unscaledTime;
                if (now - _lastClickTimeUnscaled < ClickDebounceSeconds)
                {
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Click#{clickId} ignored by debounce window ({ClickDebounceSeconds:0.00}s).");
                    return;
                }
                _lastClickTimeUnscaled = now;

                LogInfo($"[NativeMenuInjector::{_sessionId}] Click#{clickId}   menuHost.IsVisible BEFORE toggle: {_menuHost.IsVisible}");
                _menuHost.Toggle();
                LogInfo($"[NativeMenuInjector::{_sessionId}] Click#{clickId}   menuHost.IsVisible AFTER toggle: {_menuHost.IsVisible}");
                LogInfo($"[NativeMenuInjector::{_sessionId}] Click#{clickId} ✓ Mods menu TOGGLED successfully");
            }
            catch (Exception ex)
            {
                LogWarning($"[NativeMenuInjector::{_sessionId}] Click#{clickId} ⚠ OnModsButtonClicked exception: {ex.Message}\n{ex.StackTrace}");
            }
        }

        /// <summary>
        /// Replaces all click handlers on a Mods button with only the DINOForge toggle.
        /// This avoids inherited persistent callbacks from cloned Settings/Options buttons.
        /// </summary>
        private void RewireModsButtonClick(Button modsButton, long attemptId)
        {
            modsButton.onClick.RemoveAllListeners();
            modsButton.onClick.AddListener(OnModsButtonClicked);
            LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     Click handler replaced with DINOForge toggle only");
        }

        /// <summary>
        /// Mirrors source button selectable and label style onto the injected Mods button.
        /// Keeps hover/pressed visuals aligned with the native menu skin.
        /// </summary>
        private void SyncButtonVisualStyle(Button target, Button source, long attemptId)
        {
            target.transition = source.transition;
            target.colors = source.colors;
            target.spriteState = source.spriteState;
            target.animationTriggers = source.animationTriggers;

            if (source.targetGraphic != null)
            {
                string path = GetRelativePath(source.targetGraphic.transform, source.transform);
                Transform? matching = string.IsNullOrEmpty(path) ? target.transform : target.transform.Find(path);
                target.targetGraphic = matching?.GetComponent(source.targetGraphic.GetType()) as Graphic;
            }

            // Fallback: if targetGraphic is still null, use the first Image child
            if (target.targetGraphic == null)
            {
                target.targetGraphic = target.GetComponentInChildren<UnityEngine.UI.Image>(true);
                if (target.targetGraphic != null)
                {
                    LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     Applied targetGraphic fallback to '{target.targetGraphic.name}'");
                }
            }

            Text? sourceText = source.GetComponentInChildren<Text>(includeInactive: true);
            Text? targetText = target.GetComponentInChildren<Text>(includeInactive: true);
            if (sourceText != null && targetText != null)
            {
                targetText.font = sourceText.font;
                targetText.fontStyle = sourceText.fontStyle;
                targetText.fontSize = sourceText.fontSize;
                targetText.color = sourceText.color;
                targetText.alignment = sourceText.alignment;
                targetText.material = sourceText.material;
            }

            LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId}     Synced native button visual style");
        }

        private static string GetRelativePath(Transform node, Transform root)
        {
            if (node == root) return string.Empty;

            System.Collections.Generic.Stack<string> parts = new System.Collections.Generic.Stack<string>();
            Transform? current = node;
            while (current != null && current != root)
            {
                parts.Push(current.name);
                current = current.parent;
            }

            return string.Join("/", parts.ToArray());
        }

        // ------------------------------------------------------------------ //
        // Helpers
        // ------------------------------------------------------------------ //

        private static bool IsCanvasActive(Canvas canvas)
        {
            return canvas != null
                && canvas.gameObject != null
                && canvas.gameObject.activeInHierarchy;
        }

        private static bool IsCanvasNameMatch(string canvasName)
        {
            if (string.IsNullOrEmpty(canvasName)) return false;

            foreach (string candidate in CanvasCandidateNames)
            {
                if (canvasName.IndexOf(candidate, StringComparison.OrdinalIgnoreCase) >= 0)
                    return true;
            }

            return false;
        }

        private void LogInfo(string message)
        {
            if (_log != null)
                _log.LogInfo(message);
        }

        private void LogWarning(string message)
        {
            if (_log != null)
                _log.LogWarning(message);
        }

        private int _pendingAutoCheckpointFrames = -1;
        private string? _pendingAutoCheckpointPath = null;

        /// <summary>
        /// Schedule an auto-checkpoint screenshot N frames from now (main-thread-safe).
        /// Saves to BepInEx root as a PNG with the given name suffix.
        /// Delayed to allow the render loop to settle after UI changes.
        /// </summary>
        private void TakeAutoCheckpointScreenshot(string name, int delayFrames = 120)
        {
            try
            {
                string bepRoot = BepInEx.Paths.BepInExRootPath;
                string path = System.IO.Path.Combine(bepRoot, name + ".png");
                WriteDebug($"[Screenshot] Auto-checkpoint: scheduled in {delayFrames} frames: {path}");
                _pendingAutoCheckpointPath = path;
                _pendingAutoCheckpointFrames = delayFrames;
            }
            catch (Exception ex)
            {
                WriteDebug($"[Screenshot] Auto-checkpoint schedule FAILED: {ex.Message}");
            }
        }

        private void TickAutoCheckpoint()
        {
            if (_pendingAutoCheckpointFrames < 0 || _pendingAutoCheckpointPath == null) return;
            _pendingAutoCheckpointFrames--;
            if (_pendingAutoCheckpointFrames == 0)
            {
                try
                {
                    WriteDebug($"[Screenshot] Auto-checkpoint: capturing now: {_pendingAutoCheckpointPath}");
                    ScreenCapture.CaptureScreenshot(_pendingAutoCheckpointPath);
                    WriteDebug($"[Screenshot] Auto-checkpoint: CaptureScreenshot called: {_pendingAutoCheckpointPath}");
                }
                catch (Exception ex)
                {
                    WriteDebug($"[Screenshot] Auto-checkpoint capture FAILED: {ex.Message}");
                }
                _pendingAutoCheckpointPath = null;
                _pendingAutoCheckpointFrames = -1;
            }
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = System.IO.Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                System.IO.File.AppendAllText(debugLog, $"[{System.DateTime.Now}] [NativeMenuInjector] {msg}\n");
            }
            catch { }
        }
    }
}
