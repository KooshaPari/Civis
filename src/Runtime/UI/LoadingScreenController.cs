#nullable enable
using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using BepInEx.Logging;
using DINOForge.Runtime.Diagnostics;
using DINOForge.Runtime.Bridge;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// EPIC-027: Full-screen themed loading-screen takeover shown during DINOForge's
    /// mod-initialization phase and DINO scene transitions.
    ///
    /// Replaces the plain dark <see cref="ModLoadingOverlay"/> with a branded screen:
    /// pack-authored background sprite + logo + rotating tip text + progress bar.
    ///
    /// Theme resolution (declarative, no hardcoded content IDs):
    /// - On creation, scans the deployed <c>dinoforge_packs/</c> directory for the first
    ///   ACTIVE pack of <c>type: total_conversion</c> that declares a
    ///   <c>ui_theme.loading_screen.background</c> (or <c>ui_theme.loading_background</c>).
    /// - Falls back to the built-in DINOForge dark theme if no pack theme is found or the
    ///   background image is missing.
    ///
    /// Lifecycle:
    /// - Created by <c>RuntimeDriver.Initialize()</c> on the DINOForge_Root GameObject
    ///   (DontDestroyOnLoad) so it survives DINO's additive scene loads.
    /// - <see cref="EnsureVisible"/> re-shows it on the <c>InitialGameLoader</c> scene change.
    /// - <see cref="SetPackProgress"/> drives the progress bar during pack enumeration.
    /// - <see cref="BeginFadeOut"/> fades + destroys the canvas once packs are loaded /
    ///   the MainMenu scene + engine UI are ready.
    ///
    /// netstandard2.0 / Mono 2021.3 constraints:
    /// - No TMPro (uses <see cref="Text"/> with builtin Arial).
    /// - No reliance on <c>Update()</c> (DINO replaces Unity's PlayerLoop). All timing is
    ///   driven by a <c>WaitForEndOfFrame</c> coroutine using <see cref="Time.unscaledDeltaTime"/>.
    /// - Background images are read from disk via <c>File.ReadAllBytes</c> + <c>Texture2D.LoadImage</c>
    ///   (no Addressables at load time).
    /// </summary>
    internal sealed class LoadingScreenController : MonoBehaviour
    {
        // ── Static accessor (set on creation; cleared on destroy) ──────────────────
        internal static LoadingScreenController? Instance { get; private set; }

        // ── Defaults ───────────────────────────────────────────────────────────────
        private static readonly Color DefaultBackdrop = new Color(0.051f, 0.067f, 0.090f, 1f); // #0D1117
        private static readonly Color DefaultAccent = new Color(0.2f, 0.8f, 0.95f, 1f);        // DINOForge cyan
        private const int CanvasSortingOrder = 9998; // below DFCanvas (32767), above DINO loader

        private static readonly string[] BuiltinTips =
        {
            "Packs load in dependency order — conflicts are detected automatically.",
            "Press F10 to open the mod menu at any time during gameplay.",
            "Hot reload watches your pack files — save a YAML to see changes live.",
            "Total-conversion packs can declare their own loading screen in pack.yaml.",
            "Pack IDs must be unique and lowercase — hyphens and underscores allowed.",
            "Use the DINOForge CLI to validate a pack before deploying it.",
        };

        // ── UI references ────────────────────────────────────────────────────────────
        private Canvas? _canvas;
        private CanvasGroup? _canvasGroup;
        private Image? _background;
        private Image? _backdrop;
        private RawImage? _spinner;
        private Text? _titleText;
        private Text? _subtitleText;
        private Text? _tipText;
        private Image? _progressFill;
        private Text? _progressLabel;
        private Text? _versionText;

        // ── Theme + state ────────────────────────────────────────────────────────────
        private string[] _tips = BuiltinTips;
        private float _tipRotationSeconds = 6f;
        private int _tipIndex;
        private float _tipTimer;
        private float _spinnerAngle;
        private float _targetProgress;
        private float _shownProgress;
        private bool _fadingOut;
        private bool _fadeRequested;
        private bool _animating;
        private float _elapsed;
        private ManualLogSource? _log;
        private Timer? _safetyTimer;
        private readonly object _timerLock = new object();

        // EPIC-027 BUG B: when no real pack/scene progress has been reported yet we run an
        // INDETERMINATE animation — the bar creeps forward continuously (never frozen, so it
        // never reads as a "fake static skeleton") but asymptotically approaches, and never
        // reaches, a ceiling below 100%. Real progress (SetPackProgress) overrides it; the
        // bar only fills to 100% when BeginFadeOut() confirms the load is actually done.
        private bool _hasRealProgress;
        private float _indeterminate;            // 0..1 eased ramp driving the indeterminate cap
        private float _shimmerPhase;             // moving highlight so the bar always looks alive
        private Image? _progressShimmer;
        private const float IndeterminateCeiling = 0.92f; // never auto-fill past this without a real signal

        // Minimum on-screen time before a fade-out actually starts, so the branded screen never
        // just "flashes" when packs load quickly (UX) and is reliably observable. Kept short: the
        // load is covered for its WHOLE duration because BeginFadeOut() is only called once the
        // MainMenu scene + engine UI are actually ready (see Plugin.OnSceneLoaded / RunMainMenuInit).
        // The old 12s value made the branded screen linger ~20s as a static image after a fast
        // pack-load — that was the reported "fake static skeleton" symptom (BUG B).
        private const float MinVisibleSeconds = 1.5f;

        // Safety-net timeout: if no BeginFadeOut() call arrives within this many seconds, force
        // a fade anyway. This prevents orphaned loading screens from blocking UI clicks if any
        // caller path is missed (e.g. OnActiveSceneChanged race, _modPlatform null at RunMainMenuInit).
        private const float MaxVisibleSeconds = 8f;

        /// <summary>Creates the themed loading screen on the given parent (DINOForge_Root).</summary>
        public static LoadingScreenController? Create(GameObject parent, string packsDir, ManualLogSource? log)
        {
            try
            {
                var controller = parent.AddComponent<LoadingScreenController>();
                controller._log = log;
                controller.Build(packsDir);
                Instance = controller;
                return controller;
            }
            catch (Exception ex)
            {
                Debug.LogError($"[LoadingScreenController] Failed to create: {ex}");
                return null;
            }
        }

        private void Build(string packsDir)
        {
            LoadingTheme theme = ResolveTheme(packsDir);

            // ── Canvas ───────────────────────────────────────────────────────────────
            var canvasGo = new GameObject("DINOForge_LoadingScreen_Canvas");
            canvasGo.transform.SetParent(transform, false);
            _canvas = canvasGo.AddComponent<Canvas>();
            _canvas.renderMode = RenderMode.ScreenSpaceOverlay;
            _canvas.overrideSorting = true;
            _canvas.sortingOrder = CanvasSortingOrder;
            canvasGo.AddComponent<CanvasScaler>();
            _canvasGroup = canvasGo.AddComponent<CanvasGroup>();
            _canvasGroup.alpha = 1f;
            _canvasGroup.blocksRaycasts = false; // do not steal clicks once menu shows through
            RectTransform canvasRt = canvasGo.GetComponent<RectTransform>();
            canvasRt.anchorMin = Vector2.zero;
            canvasRt.anchorMax = Vector2.one;
            canvasRt.offsetMin = Vector2.zero;
            canvasRt.offsetMax = Vector2.zero;

            Font arial = Resources.GetBuiltinResource<Font>("Arial.ttf");

            // ── Solid backdrop (always opaque so the game canvas never bleeds through) ─
            _backdrop = CreateStretchImage(canvasGo.transform, "Backdrop", theme.Backdrop);

            // ── Full-bleed themed background sprite (if available) ─────────────────────
            if (theme.Background != null)
            {
                _background = CreateStretchImage(canvasGo.transform, "Background",
                    new Color(1f, 1f, 1f, theme.OverlayOpacity));
                _background.sprite = theme.Background;
                _background.type = Image.Type.Simple;
                _background.preserveAspect = false;
            }

            // ── Center card ────────────────────────────────────────────────────────────
            var card = new GameObject("Card");
            card.transform.SetParent(canvasGo.transform, false);
            RectTransform cardRt = card.AddComponent<RectTransform>();
            cardRt.anchorMin = new Vector2(0.5f, 0.5f);
            cardRt.anchorMax = new Vector2(0.5f, 0.5f);
            cardRt.pivot = new Vector2(0.5f, 0.5f);
            cardRt.sizeDelta = new Vector2(820f, 460f);
            var layout = card.AddComponent<VerticalLayoutGroup>();
            layout.childAlignment = TextAnchor.UpperCenter;
            layout.childForceExpandHeight = false;
            layout.childForceExpandWidth = true;
            layout.spacing = 16f;

            // Logo (image) OR title text fallback
            if (theme.Logo != null)
            {
                var logoGo = new GameObject("Logo");
                logoGo.transform.SetParent(card.transform, false);
                RectTransform logoRt = logoGo.AddComponent<RectTransform>();
                logoRt.sizeDelta = new Vector2(520f, 200f);
                var logoImg = logoGo.AddComponent<Image>();
                logoImg.sprite = theme.Logo;
                logoImg.preserveAspect = true;
                var le = logoGo.AddComponent<LayoutElement>();
                le.preferredHeight = 200f;
                le.preferredWidth = 520f;
            }
            else
            {
                _titleText = CreateText(card.transform, "Title", theme.Title, arial, 54, FontStyle.Bold, theme.Accent, 80f);
            }

            _subtitleText = CreateText(card.transform, "Subtitle", theme.Subtitle, arial, 22, FontStyle.Normal,
                new Color(1f, 1f, 1f, 0.78f), 40f);

            // Separator
            var sep = CreateStretchImage(card.transform, "Separator", new Color(theme.Accent.r, theme.Accent.g, theme.Accent.b, 0.55f));
            var sepLe = sep.gameObject.AddComponent<LayoutElement>();
            sepLe.preferredHeight = 2f;

            // Tip text
            _tipText = CreateText(card.transform, "Tip", _tips.Length > 0 ? _tips[0] : "", arial, 18, FontStyle.Italic,
                new Color(1f, 1f, 1f, 0.88f), 64f);

            // Progress bar (track + fill)
            var track = new GameObject("ProgressTrack");
            track.transform.SetParent(card.transform, false);
            RectTransform trackRt = track.AddComponent<RectTransform>();
            trackRt.sizeDelta = new Vector2(640f, 14f);
            var trackLe = track.AddComponent<LayoutElement>();
            trackLe.preferredHeight = 14f;
            trackLe.preferredWidth = 640f;
            var trackImg = track.AddComponent<Image>();
            trackImg.color = new Color(1f, 1f, 1f, 0.15f);

            var fillGo = new GameObject("ProgressFill");
            fillGo.transform.SetParent(track.transform, false);
            RectTransform fillRt = fillGo.AddComponent<RectTransform>();
            fillRt.anchorMin = new Vector2(0f, 0f);
            fillRt.anchorMax = new Vector2(0f, 1f); // grows horizontally via anchorMax.x
            fillRt.pivot = new Vector2(0f, 0.5f);
            fillRt.offsetMin = Vector2.zero;
            fillRt.offsetMax = Vector2.zero;
            _progressFill = fillGo.AddComponent<Image>();
            _progressFill.color = theme.Accent;
            SetFillAmount(0f);

            // Moving highlight that travels along the track so the bar always reads as LIVE
            // (alive even while indeterminate). Sits on top of the track, clamped to the track width.
            var shimmerGo = new GameObject("ProgressShimmer");
            shimmerGo.transform.SetParent(track.transform, false);
            RectTransform shimRt = shimmerGo.AddComponent<RectTransform>();
            shimRt.anchorMin = new Vector2(0f, 0f);
            shimRt.anchorMax = new Vector2(0f, 1f);
            shimRt.pivot = new Vector2(0.5f, 0.5f);
            shimRt.sizeDelta = new Vector2(90f, 0f);
            _progressShimmer = shimmerGo.AddComponent<Image>();
            _progressShimmer.color = new Color(1f, 1f, 1f, 0.22f);
            _progressShimmer.raycastTarget = false;

            _progressLabel = CreateText(card.transform, "ProgressLabel", "Initializing…", arial, 15, FontStyle.Normal,
                new Color(0.78f, 0.78f, 0.78f, 1f), 28f);

            // Spinner (rotated by coroutine)
            var spinnerGo = new GameObject("Spinner");
            spinnerGo.transform.SetParent(canvasGo.transform, false);
            RectTransform spinRt = spinnerGo.AddComponent<RectTransform>();
            spinRt.anchorMin = new Vector2(1f, 0f);
            spinRt.anchorMax = new Vector2(1f, 0f);
            spinRt.pivot = new Vector2(1f, 0f);
            spinRt.anchoredPosition = new Vector2(-48f, 48f);
            spinRt.sizeDelta = new Vector2(48f, 48f);
            _spinner = spinnerGo.AddComponent<RawImage>();
            _spinner.texture = BuildSpinnerTexture(theme.Accent);

            // Version label (bottom-right)
            var verGo = new GameObject("Version");
            verGo.transform.SetParent(canvasGo.transform, false);
            RectTransform verRt = verGo.AddComponent<RectTransform>();
            verRt.anchorMin = new Vector2(1f, 0f);
            verRt.anchorMax = new Vector2(1f, 0f);
            verRt.pivot = new Vector2(1f, 0f);
            verRt.anchoredPosition = new Vector2(-16f, 110f);
            verRt.sizeDelta = new Vector2(260f, 24f);
            _versionText = verGo.AddComponent<Text>();
            _versionText.font = arial;
            _versionText.fontSize = 13;
            _versionText.alignment = TextAnchor.LowerRight;
            _versionText.color = new Color(1f, 1f, 1f, 0.45f);
            _versionText.text = $"DINOForge {theme.SourceLabel}";

            DebugLog.Write("LoadingScreen",
                $"[LoadingScreenController] Built. theme={(theme.IsPackTheme ? "pack:" + theme.SourceLabel : "default")}, " +
                $"bg={(theme.Background != null)}, logo={(theme.Logo != null)}, tips={_tips.Length}");
            _log?.LogInfo($"[LoadingScreenController] Built ({(theme.IsPackTheme ? "pack theme " + theme.SourceLabel : "default theme")}).");

            StartSafetyTimer();
            StartCoroutine(AnimationLoop());
        }

        // ── Public API (called by RuntimeDriver / Plugin) ──────────────────────────────

        /// <summary>Re-shows the loading screen (e.g. on the InitialGameLoader scene change).</summary>
        public void EnsureVisible()
        {
            if (this == null || _canvasGroup == null) return;
            // A new load window has begun (e.g. InitialGameLoader re-entered): clear any pending
            // fade and resume the indeterminate animation so we cover this load fully with a LIVE
            // overlay — preventing the native DINO loader from flashing through (BUG B).
            _fadingOut = false;
            _fadeRequested = false;
            _hasRealProgress = false;
            _elapsed = 0f;
            _canvasGroup.alpha = 1f;
            if (_canvas != null) _canvas.enabled = true;
            StartSafetyTimer();
            // If the animation loop had exited (a fade had started but not yet completed),
            // restart it so the bar/shimmer resume animating for the new load window.
            if (!_animating && isActiveAndEnabled) StartCoroutine(AnimationLoop());
        }

        /// <summary>Updates the progress bar + label as packs are enumerated.</summary>
        public void SetPackProgress(int current, int total, string packName)
        {
            if (total > 0)
            {
                _hasRealProgress = true;
                _targetProgress = Mathf.Clamp01((float)current / total);
            }
            if (_progressLabel != null)
                _progressLabel.text = total > 0 ? $"Loading pack {current} / {total}: {packName}" : packName;
        }

        /// <summary>Sets a free-form status message under the progress bar.</summary>
        public void SetMessage(string message)
        {
            if (_progressLabel != null) _progressLabel.text = message;
        }

        /// <summary>
        /// Requests a fade-out. Honoured immediately once the screen has been visible for at
        /// least <see cref="MinVisibleSeconds"/>; otherwise deferred (the AnimationLoop starts
        /// the fade when the minimum is reached) so the branded screen never just flashes.
        /// </summary>
        public void BeginFadeOut()
        {
            if (_fadingOut || this == null) return;
            _fadeRequested = true;
            _targetProgress = 1f;
            CancelSafetyTimer();
            if (_elapsed >= MinVisibleSeconds) StartFadeNow();
        }

        private void StartFadeNow()
        {
            if (_fadingOut || this == null) return;
            _fadingOut = true;
            CancelSafetyTimer();
            if (isActiveAndEnabled) StartCoroutine(FadeOutRoutine());
        }

        /// <summary>Alias kept for parity with the old <see cref="ModLoadingOverlay.Hide"/> call sites.</summary>
        public void Hide() => BeginFadeOut();

        // ── Coroutines (no Update() — DINO replaces the PlayerLoop) ─────────────────────

        private IEnumerator AnimationLoop()
        {
            _animating = true;
            var wait = new WaitForEndOfFrame();
            while (!_fadingOut)
            {
                float dt = Time.unscaledDeltaTime;
                _elapsed += dt;

                // Honour a deferred fade-out request once the minimum visible time elapses.
                if (_fadeRequested && _elapsed >= MinVisibleSeconds)
                {
                    StartFadeNow();
                    break;
                }

                // Safety-net: force fade after MaxVisibleSeconds even if BeginFadeOut was never called
                // Tip rotation
                if (_tips.Length > 1)
                {
                    _tipTimer += dt;
                    if (_tipTimer >= _tipRotationSeconds)
                    {
                        _tipIndex = (_tipIndex + 1) % _tips.Length;
                        if (_tipText != null) _tipText.text = _tips[_tipIndex];
                        _tipTimer = 0f;
                    }
                }

                // Spinner rotation (~1.4 rot/s)
                _spinnerAngle = (_spinnerAngle - dt * 504f) % 360f;
                if (_spinner != null)
                    _spinner.rectTransform.localRotation = Quaternion.Euler(0f, 0f, _spinnerAngle);

                // Progress target: real pack/scene progress when reported, otherwise an
                // INDETERMINATE ramp that eases toward (but never reaches) IndeterminateCeiling.
                // This keeps the bar perpetually moving during the real load instead of sitting
                // frozen as a static skeleton (BUG B). A pending fade (load done) snaps to 100%.
                float target;
                if (_fadeRequested)
                {
                    target = 1f;
                }
                else if (_hasRealProgress)
                {
                    target = _targetProgress;
                }
                else
                {
                    // Exponential ease-out toward the ceiling; slows as it approaches.
                    _indeterminate = Mathf.Lerp(_indeterminate, 1f, 1f - Mathf.Exp(-dt * 0.55f));
                    target = _indeterminate * IndeterminateCeiling;
                }

                // Smooth shown progress toward the target (faster when snapping to 100% on done).
                float rate = _fadeRequested ? 2.0f : 0.6f;
                _shownProgress = Mathf.MoveTowards(_shownProgress, target, dt * rate);
                SetFillAmount(_shownProgress);

                // Travelling highlight along the filled portion — always-alive cue.
                _shimmerPhase = (_shimmerPhase + dt * 0.9f) % 1f;
                UpdateShimmer(_shownProgress, _shimmerPhase);

                yield return wait;
            }
            _animating = false;
        }

        private void StartSafetyTimer()
        {
            lock (_timerLock)
            {
                _safetyTimer?.Dispose();
                _safetyTimer = new Timer(OnSafetyTimerElapsed, null, (int)(MaxVisibleSeconds * 1000f), Timeout.Infinite);
            }
        }

        private void CancelSafetyTimer()
        {
            lock (_timerLock)
            {
                _safetyTimer?.Dispose();
                _safetyTimer = null;
            }
        }

        private void OnSafetyTimerElapsed(object? state)
        {
            if (_fadingOut || this == null) return;

            DebugLog.Write("LoadingScreen", $"[LoadingScreenController] Safety-net timeout ({MaxVisibleSeconds}s) — forcing fade.");
            _log?.LogWarning($"[LoadingScreenController] Safety-net timeout ({MaxVisibleSeconds}s) — forcing fade (wall-clock timer).");

            try
            {
                if (MainThreadDispatcher.IsPumpAlive)
                {
                    MainThreadDispatcher.RunOnMainThread(() =>
                    {
                        if (this != null) StartFadeNow();
                        return true;
                    });
                    return;
                }
            }
            catch (Exception ex)
            {
                Debug.LogError($"[LoadingScreenController] Failed to marshal fade timeout to main thread: {ex}");
            }

            Debug.LogWarning("[LoadingScreenController] Main-thread dispatcher unavailable; safety-net fade could not be marshaled.");
        }

        private IEnumerator FadeOutRoutine()
        {
            var wait = new WaitForEndOfFrame();
            float elapsed = 0f;
            const float duration = 0.5f;
            while (elapsed < duration)
            {
                elapsed += Time.unscaledDeltaTime;
                if (_canvasGroup != null) _canvasGroup.alpha = Mathf.Lerp(1f, 0f, elapsed / duration);
                yield return wait;
            }
            DebugLog.Write("LoadingScreen", "[LoadingScreenController] Fade-out complete; destroying.");
            if (Instance == this) Instance = null;
            Destroy(gameObject);
        }

        private void OnDestroy()
        {
            CancelSafetyTimer();
            if (Instance == this) Instance = null;
        }

        // ── Theme resolution (lightweight pack.yaml pre-scan) ──────────────────────────

        private LoadingTheme ResolveTheme(string packsDir)
        {
            var theme = new LoadingTheme
            {
                Title = "DINOForge",
                Subtitle = "Mod Platform",
                Accent = DefaultAccent,
                Backdrop = DefaultBackdrop,
                OverlayOpacity = 1f,
                SourceLabel = GetVersion(),
                IsPackTheme = false,
            };

            try
            {
                if (string.IsNullOrEmpty(packsDir) || !Directory.Exists(packsDir)) return theme;

                ISet<string> disabled = ReadDisabledPacks(packsDir);

                foreach (string packDir in Directory.GetDirectories(packsDir))
                {
                    string yamlPath = Path.Combine(packDir, "pack.yaml");
                    if (!File.Exists(yamlPath)) continue;

                    string yaml = File.ReadAllText(yamlPath, System.Text.Encoding.UTF8);
                    if (!IsTotalConversion(yaml)) continue;

                    string packId = ExtractTopLevel(yaml, "id") ?? Path.GetFileName(packDir);
                    if (disabled.Contains(packId)) continue;

                    // Resolve background path: ui_theme.loading_screen.background, then ui_theme.loading_background.
                    string? bgRel = ExtractScoped(yaml, "loading_screen:", "background")
                                    ?? ExtractScoped(yaml, "ui_theme:", "loading_background");
                    if (string.IsNullOrEmpty(bgRel)) continue;

                    string bgPath = Path.Combine(packDir, bgRel!);
                    Sprite? bg = LoadSpriteFromDisk(bgPath);
                    if (bg == null)
                    {
                        _log?.LogWarning($"[LoadingScreenController] Pack '{packId}' loading background not found: {bgPath}");
                        continue; // fall back to default theme
                    }

                    theme.Background = bg;
                    theme.IsPackTheme = true;
                    theme.SourceLabel = packId;

                    string? logoRel = ExtractScoped(yaml, "loading_screen:", "logo");
                    if (!string.IsNullOrEmpty(logoRel))
                        theme.Logo = LoadSpriteFromDisk(Path.Combine(packDir, logoRel!));

                    theme.Title = ExtractScoped(yaml, "ui_theme:", "title") ?? packId;
                    theme.Subtitle = ExtractScoped(yaml, "ui_theme:", "subtitle") ?? "Total Conversion";

                    string? accentHex = ExtractScoped(yaml, "loading_screen:", "accent_color")
                                        ?? ExtractScoped(yaml, "ui_theme:", "accent_color")
                                        ?? ExtractScoped(yaml, "ui_theme:", "primary_color");
                    if (accentHex != null && ColorUtility.TryParseHtmlString(accentHex, out Color ac)) theme.Accent = ac;

                    string? overlayStr = ExtractScoped(yaml, "loading_screen:", "overlay_opacity");
                    if (overlayStr != null && float.TryParse(overlayStr, out float op)) theme.OverlayOpacity = Mathf.Clamp01(op);

                    var packTips = ExtractTips(yaml);
                    if (packTips.Count > 0)
                    {
                        _tips = packTips.ToArray();
                        _tipIndex = 0;
                    }

                    string? rot = ExtractScoped(yaml, "loading_screen:", "tip_rotation_seconds");
                    if (rot != null && float.TryParse(rot, out float r) && r >= 2f) _tipRotationSeconds = r;

                    break; // first active total_conversion pack wins
                }
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[LoadingScreenController] Theme scan failed (using default): {ex.Message}"); // pattern-96-ok: diagnostic
            }

            return theme;
        }

        private static bool IsTotalConversion(string yaml)
        {
            string? t = ExtractTopLevel(yaml, "type");
            return string.Equals(t, "total_conversion", StringComparison.OrdinalIgnoreCase);
        }

        private static ISet<string> ReadDisabledPacks(string packsDir)
        {
            var set = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            try
            {
                string path = Path.Combine(packsDir, "disabled_packs.json");
                if (!File.Exists(path)) return set;
                string json = File.ReadAllText(path, System.Text.Encoding.UTF8);
                // Minimal parse: ["a","b"] — avoid pulling a JSON dep into the loader pre-scan.
                foreach (string part in json.Trim().Trim('[', ']').Split(','))
                {
                    string id = part.Trim().Trim('"', '\'', ' ');
                    if (id.Length > 0) set.Add(id);
                }
            }
            catch { /* safe-swallow: missing/invalid disabled list => nothing disabled */ }
            return set;
        }

        /// <summary>Reads a top-level (column-0) scalar key from pack.yaml.</summary>
        private static string? ExtractTopLevel(string yaml, string key)
        {
            foreach (string line in yaml.Split('\n'))
            {
                string l = line.TrimEnd('\r');
                if (l.Length == 0 || l[0] == ' ' || l[0] == '\t' || l[0] == '#') continue;
                int colon = l.IndexOf(':');
                if (colon <= 0) continue;
                if (string.Equals(l.Substring(0, colon).Trim(), key, StringComparison.Ordinal))
                    return Unquote(l.Substring(colon + 1).Trim());
            }
            return null;
        }

        /// <summary>Reads <c>key:</c> nested under the indented block introduced by <paramref name="blockKey"/>.</summary>
        private static string? ExtractScoped(string yaml, string blockKey, string key)
        {
            // Find the block-introducing line (e.g. "loading_screen:" / "ui_theme:"),
            // matching regardless of leading indentation. Record its indent so we can
            // bound the nested region (a less-or-equal-indented line ends the block).
            string blockName = blockKey.TrimEnd(':');
            string[] lines = yaml.Split('\n');
            int blockLine = -1, blockIndent = 0;
            for (int i = 0; i < lines.Length; i++)
            {
                string l = lines[i];
                string t = l.TrimStart();
                if (t.TrimEnd('\r').TrimEnd() == blockName + ":")
                {
                    blockLine = i;
                    blockIndent = l.Length - t.Length;
                    break;
                }
            }
            if (blockLine < 0) return null;

            string searchKey = key + ":";
            for (int i = blockLine + 1; i < lines.Length; i++)
            {
                string l = lines[i];
                if (l.Trim().Length == 0 || l.TrimStart().StartsWith("#")) continue;
                int indent = l.Length - l.TrimStart().Length;
                if (indent <= blockIndent) break; // dedented => block ended

                string t = l.TrimStart();
                if (t.StartsWith(searchKey, StringComparison.Ordinal))
                {
                    string raw = t.Substring(searchKey.Length).TrimEnd('\r').Trim();
                    return string.IsNullOrEmpty(raw) ? null : Unquote(raw);
                }
            }
            return null;
        }

        /// <summary>Reads a YAML list of strings under <c>loading_screen: ... tips:</c>.</summary>
        private static List<string> ExtractTips(string yaml)
        {
            var tips = new List<string>();
            int tipsIdx = yaml.IndexOf("tips:", StringComparison.Ordinal);
            if (tipsIdx < 0) return tips;
            int lineEnd = yaml.IndexOf('\n', tipsIdx);
            if (lineEnd < 0) return tips;
            string[] lines = yaml.Substring(lineEnd + 1).Split('\n');
            foreach (string raw in lines)
            {
                string l = raw.TrimEnd('\r');
                string trimmed = l.TrimStart();
                if (trimmed.StartsWith("- "))
                    tips.Add(Unquote(trimmed.Substring(2).Trim()));
                else if (trimmed.Length > 0)
                    break; // list ended
            }
            return tips;
        }

        private static string Unquote(string s)
        {
            if (s.Length >= 2 && (s[0] == '"' || s[0] == '\'') && s[s.Length - 1] == s[0])
                return s.Substring(1, s.Length - 2);
            return s;
        }

        // ── Disk-loaded sprite (no Addressables) ───────────────────────────────────────

        private Sprite? LoadSpriteFromDisk(string fullPath)
        {
            try
            {
                if (!File.Exists(fullPath)) return null;
                byte[] data = File.ReadAllBytes(fullPath);
                var tex = new Texture2D(2, 2, TextureFormat.RGBA32, mipChain: false)
                {
                    filterMode = FilterMode.Bilinear,
                    wrapMode = TextureWrapMode.Clamp,
                };
                if (!tex.LoadImage(data)) return null;
                return Sprite.Create(tex, new Rect(0f, 0f, tex.width, tex.height), new Vector2(0.5f, 0.5f), 100f);
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[LoadingScreenController] LoadSpriteFromDisk('{fullPath}') failed: {ex.Message}"); // pattern-96-ok
                return null;
            }
        }

        // ── UI helpers ─────────────────────────────────────────────────────────────────

        private static Image CreateStretchImage(Transform parent, string name, Color color)
        {
            var go = new GameObject(name);
            go.transform.SetParent(parent, false);
            RectTransform rt = go.AddComponent<RectTransform>();
            rt.anchorMin = Vector2.zero;
            rt.anchorMax = Vector2.one;
            rt.offsetMin = Vector2.zero;
            rt.offsetMax = Vector2.zero;
            var img = go.AddComponent<Image>();
            img.color = color;
            img.raycastTarget = false;
            return img;
        }

        private static Text CreateText(Transform parent, string name, string content, Font font,
            int size, FontStyle style, Color color, float height)
        {
            var go = new GameObject(name);
            go.transform.SetParent(parent, false);
            RectTransform rt = go.AddComponent<RectTransform>();
            rt.sizeDelta = new Vector2(700f, height);
            var t = go.AddComponent<Text>();
            t.font = font;
            t.fontSize = size;
            t.fontStyle = style;
            t.alignment = TextAnchor.MiddleCenter;
            t.color = color;
            t.text = content;
            t.horizontalOverflow = HorizontalWrapMode.Wrap;
            t.verticalOverflow = VerticalWrapMode.Overflow;
            t.raycastTarget = false;
            var le = go.AddComponent<LayoutElement>();
            le.preferredHeight = height;
            return t;
        }

        private void SetFillAmount(float amount)
        {
            if (_progressFill == null) return;
            RectTransform rt = _progressFill.rectTransform;
            rt.anchorMax = new Vector2(Mathf.Clamp01(amount), 1f);
            rt.offsetMin = Vector2.zero;
            rt.offsetMax = Vector2.zero;
        }

        /// <summary>Slides the highlight across the filled portion of the bar so it always looks live.</summary>
        private void UpdateShimmer(float fillAmount, float phase)
        {
            if (_progressShimmer == null) return;
            RectTransform parent = _progressShimmer.rectTransform.parent as RectTransform;
            float trackWidth = parent != null ? parent.rect.width : 640f;
            float filledWidth = Mathf.Max(0f, trackWidth * Mathf.Clamp01(fillAmount));
            // Hide the highlight when there is essentially no fill yet.
            _progressShimmer.enabled = filledWidth > 12f;
            RectTransform rt = _progressShimmer.rectTransform;
            rt.anchoredPosition = new Vector2(Mathf.Lerp(0f, filledWidth, phase), 0f);
        }

        /// <summary>Builds a simple radial spinner texture (arc of dots) tinted with the accent color.</summary>
        private static Texture2D BuildSpinnerTexture(Color accent)
        {
            const int n = 48;
            var tex = new Texture2D(n, n, TextureFormat.RGBA32, false) { filterMode = FilterMode.Bilinear };
            var px = new Color[n * n];
            float cx = n / 2f, cy = n / 2f, outer = n / 2f - 1f, inner = outer - 6f;
            for (int y = 0; y < n; y++)
            {
                for (int x = 0; x < n; x++)
                {
                    float dx = x - cx, dy = y - cy;
                    float dist = Mathf.Sqrt(dx * dx + dy * dy);
                    if (dist <= outer && dist >= inner)
                    {
                        float ang = Mathf.Atan2(dy, dx) + Mathf.PI; // 0..2π
                        float frac = ang / (2f * Mathf.PI);          // tail fade
                        px[y * n + x] = new Color(accent.r, accent.g, accent.b, frac);
                    }
                    else
                    {
                        px[y * n + x] = new Color(0f, 0f, 0f, 0f);
                    }
                }
            }
            tex.SetPixels(px);
            tex.Apply();
            return tex;
        }

        private static string GetVersion()
        {
            try { return "v" + typeof(LoadingScreenController).Assembly.GetName().Version?.ToString(3); }
            catch { return string.Empty; }
        }

        private sealed class LoadingTheme
        {
            public Sprite? Background;
            public Sprite? Logo;
            public string Title = "DINOForge";
            public string Subtitle = "Mod Platform";
            public Color Accent;
            public Color Backdrop;
            public float OverlayOpacity = 1f;
            public string SourceLabel = string.Empty;
            public bool IsPackTheme;
        }
    }
}
