#nullable enable
using System;
using BepInEx.Logging;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Root Canvas manager MonoBehaviour for the DINOForge UGUI overlay system.
    /// Owns the full Canvas hierarchy and all child panels.
    /// Attach to the DINOForge_Root persistent GameObject.
    ///
    /// Canvas hierarchy:
    /// <code>
    /// DINOForge_Root
    ///   └── DFCanvas_Root (Canvas, CanvasScaler, GraphicRaycaster)
    ///           ├── ModMenuPanel
    ///           ├── DebugPanel
    ///           └── HudStrip
    /// </code>
    /// </summary>
    public class DFCanvas : MonoBehaviour
    {
        // ── Child panels (public for RuntimeDriver access) ────────────────────────

        /// <summary>The UGUI mod menu panel. Exposes the same API as ModMenuOverlay.</summary>
        public ModMenuPanel? ModMenuPanel { get; private set; }

        /// <summary>The UGUI debug panel. Replaces DebugOverlayBehaviour.</summary>
        public DebugPanel? DebugPanel { get; private set; }

        /// <summary>The always-visible HUD strip (top-right corner).</summary>
        public HudStrip? HudStrip { get; private set; }

        // ── Private state ─────────────────────────────────────────────────────────
        private ManualLogSource? _log;
        private Canvas? _canvas;
        private bool _ready;

        // Mouse tracking for HUD hover
        private RectTransform? _hudStripRt;

        // ── Bootstrap ─────────────────────────────────────────────────────────────

        /// <summary>
        /// Callback invoked when DFCanvas successfully builds its canvas hierarchy in Initialize().
        /// RuntimeDriver sets this to mark UGUI as ready without polling IsReady from a background thread
        /// (which causes UnityException on background thread access to MonoBehaviour fields).
        /// </summary>
        public Action? OnInitSuccess;

        /// <summary>
        /// Callback invoked when DFCanvas fails to build its canvas hierarchy in Start().
        /// RuntimeDriver sets this before the first frame so it can activate the IMGUI
        /// fallback if UGUI setup fails after the component is already added.
        /// </summary>
        public Action? OnInitFailed;

        /// <summary>
        /// Whether the canvas hierarchy has been successfully built and is ready for use.
        /// False until Start() completes without error.
        /// </summary>
        public bool IsReady => _ready;

        /// <summary>
        /// Initializes DFCanvas and stores the logger.
        /// Canvas hierarchy is built in Start() (next frame) to allow Unity to finish
        /// initialising the component.
        /// Must be called from the main thread immediately after AddComponent.
        /// </summary>
        /// <param name="log">BepInEx logger for diagnostics.</param>
        public void Initialize(ManualLogSource log)
        {
            _log = log;
            // Build canvas immediately in Initialize() since Start() never fires in DINO
            // (DINO replaces the MonoBehaviour player loop — Update/Start/OnGUI never run).
            // Awake/OnEnable DO fire (called synchronously from AddComponent).
            try
            {
                BuildCanvas();
                _ready = true;
                _log?.LogInfo("[DFCanvas] UGUI canvas hierarchy built successfully in Initialize().");
                OnInitSuccess?.Invoke();
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[DFCanvas] Canvas build failed in Initialize(): {ex.Message}");
                OnInitFailed?.Invoke();
            }
        }

        private void Start()
        {
            // Start() never fires in DINO — canvas is built in Initialize() instead.
            // This is kept as a fallback in case Unity environment changes.
            if (_ready) return;
            try
            {
                BuildCanvas();
                _ready = true;
                _log?.LogInfo("[DFCanvas] UGUI canvas hierarchy built in Start() (fallback).");
                OnInitSuccess?.Invoke();
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[DFCanvas] Canvas setup failed — IMGUI fallback will activate: {ex.Message}");
                _log?.LogWarning($"[DFCanvas] Full exception: {ex}");
                _ready = false;
                OnInitFailed?.Invoke();
            }
        }

        private void BuildCanvas()
        {
            // Canvas root child object
            GameObject canvasGo = new GameObject("DFCanvas_Root");
            canvasGo.transform.SetParent(gameObject.transform, false);

            // Canvas component
            _canvas = canvasGo.AddComponent<Canvas>();
            _canvas.renderMode = RenderMode.ScreenSpaceOverlay;
            _canvas.sortingOrder = 100;

            // CanvasScaler — scale with screen size, reference 1920x1080
            CanvasScaler scaler = canvasGo.AddComponent<CanvasScaler>();
            scaler.uiScaleMode = CanvasScaler.ScaleMode.ScaleWithScreenSize;
            scaler.referenceResolution = new Vector2(1920f, 1080f);
            scaler.matchWidthOrHeight = 0.5f;

            // GraphicRaycaster for pointer events
            canvasGo.AddComponent<GraphicRaycaster>();

            Transform canvasRoot = canvasGo.transform;

            // Build HUD strip first (always visible, no panel ownership)
            GameObject hudGo = new GameObject("HudStripHost", typeof(RectTransform));
            hudGo.transform.SetParent(canvasRoot, false);
            HudStrip = hudGo.AddComponent<HudStrip>();
            HudStrip.Build(canvasRoot);
            HudStrip.OnClicked = ToggleModMenu;
            _hudStripRt = hudGo.GetComponent<RectTransform>();

            // Build mod menu panel
            GameObject menuGo = new GameObject("ModMenuPanelHost", typeof(RectTransform));
            menuGo.transform.SetParent(canvasRoot, false);
            ModMenuPanel = menuGo.AddComponent<ModMenuPanel>();
            if (_log != null)
                ModMenuPanel.Initialize(_log);
            ModMenuPanel.Build(canvasRoot);

            // Build debug panel
            GameObject debugGo = new GameObject("DebugPanelHost", typeof(RectTransform));
            debugGo.transform.SetParent(canvasRoot, false);
            DebugPanel = debugGo.AddComponent<DebugPanel>();
            DebugPanel.Build(canvasRoot);
        }

        // ── Input handling ────────────────────────────────────────────────────────
        // NOTE: F9/F10 key handling has been intentionally moved to RuntimeDriver.Update()
        // so that key bindings always work regardless of whether UGUI initialised
        // successfully.  DFCanvas only handles Escape (close panels) and HUD hover.

        private void Update()
        {
            if (!_ready) return;

            if (Input.GetKeyDown(KeyCode.Escape))
                HideAll();

            // HUD strip hover detection
            UpdateHudHover();
        }

        private void UpdateHudHover()
        {
            if (HudStrip == null || _canvas == null) return;

            // Find the HudStrip RectTransform (it adds the panel directly to canvasRoot)
            // Use RectTransformUtility for accurate screen-space hit testing
            Transform canvasRoot = _canvas.transform;
            RectTransform? stripRt = canvasRoot.Find("HudStrip")?.GetComponent<RectTransform>();
            if (stripRt == null) return;

            Vector2 localPoint;
            bool over = RectTransformUtility.ScreenPointToLocalPointInRectangle(
                stripRt,
                Input.mousePosition,
                null, // overlay canvas — no camera needed
                out localPoint)
                && stripRt.rect.Contains(localPoint);

            HudStrip.SetHovered(over);
        }

        // ── Show/Hide API ─────────────────────────────────────────────────────────

        /// <summary>Shows the mod menu panel.</summary>
        public void ShowModMenu()
        {
            if (_ready) ModMenuPanel?.Show();
        }

        /// <summary>Hides the mod menu panel.</summary>
        public void HideModMenu()
        {
            if (_ready) ModMenuPanel?.Hide();
        }

        /// <summary>Toggles the mod menu panel.</summary>
        public void ToggleModMenu()
        {
            if (!_ready) return;
            if (ModMenuPanel == null) return;

            if (ModMenuPanel.IsVisible) ModMenuPanel.Hide();
            else ModMenuPanel.Show();
        }

        /// <summary>Shows the debug panel.</summary>
        public void ShowDebug()
        {
            if (_ready) DebugPanel?.Show();
        }

        /// <summary>Hides the debug panel.</summary>
        public void HideDebug()
        {
            if (_ready) DebugPanel?.Hide();
        }

        /// <summary>Toggles the debug panel.</summary>
        public void ToggleDebug()
        {
            if (!_ready) return;
            if (DebugPanel == null) return;

            if (DebugPanel.IsVisible) DebugPanel.Hide();
            else DebugPanel.Show();
        }

        /// <summary>Hides all panels.</summary>
        public void HideAll()
        {
            HideModMenu();
            HideDebug();
        }

        /// <summary>Displays a toast notification on the HUD strip.</summary>
        public void ShowToast(string message, ToastType type)
        {
            HudStrip?.ShowToast(message, type);
        }
    }
}
