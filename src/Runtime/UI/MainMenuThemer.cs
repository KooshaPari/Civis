#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using BepInEx.Logging;
using DINOForge.Runtime.Diagnostics;
using DINOForge.Runtime.UI;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime
{
    /// <summary>
    /// Applies a total_conversion pack's ui_theme to DINO's native main menu UI.
    /// Performs in-place replacement (title, background tint, button colors, label rewrites).
    /// TMP_Text accessed via reflection — no compile-time TMPro reference.
    /// </summary>
    internal sealed class MainMenuThemer
    {
        private readonly ManualLogSource _log;
        private readonly string _packsDirectory;
        private bool _applied;

        // Runtime-created TMP_FontAsset (from the pack's shipped TTF) cached for the session.
        // Created once via reflection (Runtime has no compile-time TMPro reference) and reused.
        private static UnityEngine.Object? _cachedFontAsset;
        private static string? _cachedFontKey;

        public bool IsApplied => _applied;

        // #3 page-skinning: cache the resolved theme + parsed colors so the auxiliary-canvas
        // pass (Options/Settings/subpages + create/select screens) can re-skin those canvases
        // each time one of them becomes active, without re-reading YAML.
        private ThemeData? _cachedTheme;
        private Color _cPrimary, _cSecondary, _cText, _cAccent, _cBgTint;
        private bool _cHasBgTint;
        // Instance IDs of auxiliary canvases already themed this session (avoid re-walking).
        private readonly HashSet<int> _themedAuxCanvases = new HashSet<int>();

        // Auxiliary (non-MainMenu) menu canvases that DINO shows for settings + create/select.
        // These are separate canvases from "MainMenu" and were previously left unskinned-native.
        private static readonly string[] AuxMenuCanvasNames =
        {
            "Options", "Settings", "Video", "Sound", "Audio", "Controls", "Twitch", "Game",
            "ProfilesList", "SpecialMissions", "SandBox", "Sandbox", "EndlessMissions",
            "Endless", "Saves", "Conditions", "Map", "ArmyPanel", "CustomMaps", "Campaign",
            "Tutorial", "Enter", "ConfirmWindow", "AnalyticsConsent", "Credits"
        };

        public MainMenuThemer(ManualLogSource log, string packsDirectory)
        {
            _log = log;
            _packsDirectory = packsDirectory ?? string.Empty;
        }

        // Aux/subpage takeover (#974) caches the resolved theme + pack id so the per-frame
        // re-application pump (subpages open AFTER the main menu and re-open repeatedly,
        // e.g. settings tabs) is cheap. Reset on scene change.
        private ThemeData? _auxTheme;
        private PackDisplayInfo? _auxPack;
        private int _auxLastCanvasCount = -1;

        public void OnSceneChanged()
        {
            _applied = false;
            _themedAuxCanvases.Clear();
            _auxTheme = null;
            _auxPack = null;
            _auxLastCanvasCount = -1;
        }

        /// <summary>
        /// #3 — 100% page skinning. Re-skins every currently-active auxiliary menu canvas
        /// (Options/Settings + GAME/VIDEO/SOUND/CONTROLS/TWITCH subpages, and the game
        /// create/select screens) with the active conversion's color layer. These are separate
        /// canvases from "MainMenu" that open later (e.g. when the player clicks Options), so this
        /// runs every pump frame; each canvas is themed once (tracked by instance ID) and skipped
        /// thereafter. No-op until <see cref="TryApplyTheme"/> has resolved a theme.
        /// Returns the number of canvases newly skinned this call.
        /// </summary>
        public int ApplyToAuxiliaryMenus()
        {
            if (_cachedTheme == null) return 0;
            int skinned = 0;
            try
            {
                var canvases = UnityEngine.Object.FindObjectsOfType<Canvas>();
                foreach (var c in canvases)
                {
                    if (c == null || !c.gameObject.activeInHierarchy) continue;
                    string name = c.name ?? string.Empty;
                    // Skip the main menu (handled by TryApplyTheme), PrimeCanvas, and DINOForge's
                    // own canvases (already self-styled).
                    if (name.IndexOf("PrimeCanvas", StringComparison.OrdinalIgnoreCase) >= 0) continue;
                    if (name.IndexOf("DINOForge", StringComparison.OrdinalIgnoreCase) >= 0) continue;
                    if (name.IndexOf("DFCanvas", StringComparison.OrdinalIgnoreCase) >= 0) continue;
                    if (name.IndexOf("MainMenu", StringComparison.OrdinalIgnoreCase) >= 0) continue;

                    bool isMenuCanvas = false;
                    foreach (string n in AuxMenuCanvasNames)
                    {
                        if (name.IndexOf(n, StringComparison.OrdinalIgnoreCase) >= 0) { isMenuCanvas = true; break; }
                    }
                    if (!isMenuCanvas) continue;

                    int id = c.GetInstanceID();
                    if (_themedAuxCanvases.Contains(id)) continue;
                    _themedAuxCanvases.Add(id);

                    int btn = RestyleSelectables(c, _cPrimary, _cSecondary, _cText, _cAccent);
                    int lbl = RewriteLabels(c, _cText);
                    int bg = _cHasBgTint ? TintBackground(c, _cBgTint) : 0;
                    int txt = RecolorText(c, _cText);
                    int font = 0;
                    if (_cachedFontAsset != null) font = ApplyFont(c, _cachedFontAsset);
                    skinned++;
                    _log?.LogInfo($"[MainMenuThemer] AUX-SKIN canvas='{name}' btn={btn} label={lbl} bgTint={bg} text={txt} font={font}");
                    DebugLog.Write("MainMenuThemer", $"AUX-SKIN '{name}' btn={btn} label={lbl} bg={bg} text={txt}");
                }
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[MainMenuThemer] ApplyToAuxiliaryMenus failed: {ex.Message}"); // pattern-96-ok: diagnostic
            }
            return skinned;
        }

        /// <summary>
        /// Recolors all legacy Text + TMP_Text on a canvas to the theme text color (skinning the
        /// settings sliders/selectors/tab labels that RewriteLabels leaves untouched).
        /// </summary>
        private int RecolorText(Canvas canvas, Color textCol)
        {
            int hits = 0;
            foreach (var t in canvas.GetComponentsInChildren<Text>(true))
            {
                if (t == null) continue;
                if (t.gameObject.name.IndexOf("DINOForge", StringComparison.OrdinalIgnoreCase) >= 0) continue;
                try { t.color = textCol; hits++; } catch { /* best-effort */ }
            }
            foreach (var c in canvas.GetComponentsInChildren<Component>(true))
            {
                if (c == null) continue;
                if (!(c.GetType().FullName ?? "").StartsWith("TMPro.", StringComparison.Ordinal)) continue;
                try { c.GetType().GetProperty("color")?.SetValue(c, textCol); hits++; }
                catch { /* best-effort: TMPro reflection */ }
            }
            return hits;
        }

        public bool TryApplyTheme(IReadOnlyList<PackDisplayInfo> packs)
        {
            if (_applied) return true;
            if (packs == null || packs.Count == 0) return false;

            // Only ENABLED total_conversion packs are eligible — when multiple TC packs are
            // installed, the user's enable/disable choice in the F10 mod menu decides which
            // one takes over the menu (no hardcoded pack id). Disabled packs are skipped.
            //
            // Deterministic active-conversion selection (fix: "Modern shows instead of Star Wars").
            //
            // NOTE: this list (ModPlatform.BuildPackDisplayInfos) enumerates every pack.yaml on disk,
            // INCLUDING dirs renamed *.disabled, and marks them IsEnabled=false. We therefore filter
            // to IsEnabled candidates so a DISABLED total_conversion can never be themed.
            // Previously this picked the FIRST enabled-or-not total_conversion with a ui_theme: block
            // in iteration order, which is the alphabetical Directory.GetDirectories order. With
            // multiple total_conversions present (e.g. warfare-modern + warfare-starwars),
            // "warfare-modern" sorted first and won — even when Star Wars was intended. The rule is
            // now order-independent:
            //   1. Restrict to ENABLED total_conversions (IsEnabled == true).
            //   2. Among those WITH a ui_theme: block, pick the lexicographically-smallest pack Id
            //      (stable, never iteration-order dependent).
            //   3. If none declare ui_theme:, fall back to the lexicographically-smallest enabled
            //      total_conversion so a title is still applied.
            // When exactly one total_conversion is enabled (the post-cut steady state) this trivially
            // selects it.
            PackDisplayInfo? best = null;
            PackDisplayInfo? fallback = null;
            foreach (var p in packs.OrderBy(p => p.Id, StringComparer.Ordinal))
            {
                if (!p.IsEnabled) continue;
                if (!string.Equals(p.Type, "total_conversion", StringComparison.OrdinalIgnoreCase)) continue;
                if (fallback == null) fallback = p;
                string yamlPath = Path.Combine(_packsDirectory, p.Id, "pack.yaml");
                if (File.Exists(yamlPath))
                {
                    string content = File.ReadAllText(yamlPath, System.Text.Encoding.UTF8);
                    if (content.IndexOf("ui_theme:", StringComparison.Ordinal) >= 0)
                    {
                        best = p;
                        break;
                    }
                }
            }
            best = best ?? fallback;
            if (best == null) return false;

            _log?.LogInfo($"[MainMenuThemer] Active total_conversion selected: '{best.Id}' (deterministic).");

            var theme = ReadThemeFromDisk(best.Id) ?? new ThemeData { Title = best.Name };
            return ApplyToMainMenu(theme, best);
        }

        private ThemeData? ReadThemeFromDisk(string packId)
        {
            try
            {
                string yamlPath = Path.Combine(_packsDirectory, packId, "pack.yaml");
                if (!File.Exists(yamlPath)) return null;
                string yaml = File.ReadAllText(yamlPath, System.Text.Encoding.UTF8);
                int idx = yaml.IndexOf("ui_theme:", StringComparison.Ordinal);
                if (idx < 0) return null;

                return new ThemeData
                {
                    Title = ExtractYamlValue(yaml, idx, "title"),
                    Subtitle = ExtractYamlValue(yaml, idx, "subtitle"),
                    PrimaryColor = ExtractYamlValue(yaml, idx, "primary_color") ?? "#FFE81F",
                    SecondaryColor = ExtractYamlValue(yaml, idx, "secondary_color") ?? "#000000",
                    AccentColor = ExtractYamlValue(yaml, idx, "accent_color") ?? "#C0392B",
                    TextColor = ExtractYamlValue(yaml, idx, "text_color") ?? "#FFE81F",
                    BackgroundTint = ExtractYamlValue(yaml, idx, "background_tint"),
                    // EPIC-027 visual takeover — pack-relative PNG art references.
                    Logo = ExtractYamlValue(yaml, idx, "logo"),
                    BackgroundImage = ExtractYamlValue(yaml, idx, "background_image"),
                    ButtonFrame = ExtractYamlValue(yaml, idx, "button_frame"),
                    ButtonFrameHover = ExtractYamlValue(yaml, idx, "button_frame_hover"),
                    Font = ExtractYamlValue(yaml, idx, "font"),
                    FontFamily = ExtractYamlValue(yaml, idx, "font_family"),
                    // #965 baked-TMP-font path: prebuilt TMP_FontAsset name inside the bundle.
                    FontAssetName = ExtractYamlValue(yaml, idx, "font_asset_name")
                };
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[MainMenuThemer] ReadThemeFromDisk failed: {ex.Message}"); // pattern-96-ok: diagnostic
                return null;
            }
        }

        private static string? ExtractYamlValue(string yaml, int blockStart, string key)
        {
            string searchKey = key + ":";
            int keyIdx = yaml.IndexOf(searchKey, blockStart, StringComparison.Ordinal);
            if (keyIdx < 0) return null;
            int valueStart = keyIdx + searchKey.Length;
            int lineEnd = yaml.IndexOf('\n', valueStart);
            if (lineEnd < 0) lineEnd = yaml.Length;
            string raw = yaml.Substring(valueStart, lineEnd - valueStart).Trim();
            if (raw.Length >= 2 && (raw[0] == '"' || raw[0] == '\'')) raw = raw.Substring(1, raw.Length - 2);
            return string.IsNullOrEmpty(raw) ? null : raw;
        }

        private bool ApplyToMainMenu(ThemeData theme, PackDisplayInfo pack)
        {
            try
            {
                Canvas? canvas = FindMainMenuCanvas();
                if (canvas == null) return false;

                ColorUtility.TryParseHtmlString(theme.PrimaryColor, out Color primary);
                ColorUtility.TryParseHtmlString(theme.SecondaryColor, out Color secondary);
                ColorUtility.TryParseHtmlString(theme.TextColor, out Color textCol);
                ColorUtility.TryParseHtmlString(theme.AccentColor ?? "#C0392B", out Color accent);
                Color bgTint = Color.black;
                bool hasBgTint = theme.BackgroundTint != null && ColorUtility.TryParseHtmlString(theme.BackgroundTint, out bgTint);

                // #3: cache resolved theme + colors for the auxiliary-canvas skinning pass.
                _cachedTheme = theme;
                _cPrimary = primary; _cSecondary = secondary; _cText = textCol; _cAccent = accent;
                _cBgTint = bgTint; _cHasBgTint = hasBgTint;

                // ── EPIC-027 VISUAL TAKEOVER ────────────────────────────────────
                // When the pack ships PNG art, perform a real reskin (full sprite
                // swaps + injected logo) rather than the cosmetic tint-only path.
                // Each step degrades gracefully to the tint/label path when art is
                // absent (LoadSpriteFromPack returns null, never throws).
                bool bgSwapped = TrySwapBackground(canvas, theme, pack.Id);
                int bgTintHits = (!bgSwapped && hasBgTint) ? TintBackground(canvas, bgTint) : 0;
                bool logoInjected = TryInjectLogo(canvas, theme, pack.Id);
                int frameHits = ApplyButtonFrames(canvas, theme, pack.Id);

                // Title: if a logo sprite was injected, hide the native title text;
                // otherwise rewrite it to the themed title (text-as-logo fallback).
                int titleHits = logoInjected
                    ? HideTitle(canvas)
                    : ReplaceTitle(canvas, theme.Title, primary);
                int btnHits = RestyleSelectables(canvas, primary, secondary, textCol, accent);
                int labelHits = RewriteLabels(canvas, textCol);

                // FONT (#965): load the prebuilt baked TMP_FontAsset from the pack bundle and apply
                // it to every TMP_Text on the menu canvas (reflection — no compile-time TMPro ref).
                // Replaces the old runtime CreateFontAsset path (returns null in DINO's shipped player).
                int fontHits = ApplyFont(canvas, theme, pack);
                bool fontLoaded = fontHits > 0;

                bool takeover = bgSwapped || logoInjected || frameHits > 0;
                _applied = true;
                _log?.LogInfo($"[MainMenuThemer] TAKEOVER applied: '{theme.Title}' from '{pack.Id}': takeover={takeover} bgSwap={bgSwapped} logo={logoInjected} frames={frameHits} font={fontLoaded} (fontHits={fontHits} tint-bg={bgTintHits} title={titleHits} btn={btnHits} label={labelHits})");
                DebugLog.Write("MainMenuThemer", $"{(takeover ? "TAKEOVER" : "Tint")} applied: '{theme.Title}' canvas='{canvas.name}' bgSwap={bgSwapped} logo={logoInjected} frames={frameHits} font={fontLoaded}");
                return true;
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[MainMenuThemer] ApplyToMainMenu failed: {ex.Message}"); // pattern-96-ok: diagnostic
                return false;
            }
        }

        private static Canvas? FindMainMenuCanvas()
        {
            var canvases = UnityEngine.Object.FindObjectsOfType<Canvas>();
            foreach (var c in canvases)
            {
                if (c == null || !c.gameObject.activeInHierarchy) continue;
                if (c.name.IndexOf("MainMenu", StringComparison.OrdinalIgnoreCase) >= 0
                    && c.name.IndexOf("PrimeCanvas", StringComparison.OrdinalIgnoreCase) < 0)
                    return c;
            }
            return null;
        }

        private int ReplaceTitle(Canvas canvas, string? newTitle, Color color)
        {
            if (string.IsNullOrEmpty(newTitle)) return 0;
            int hits = 0;

            foreach (var c in canvas.GetComponentsInChildren<Component>(true))
            {
                if (c == null) continue;
                string n = c.GetType().FullName ?? "";
                if (!n.StartsWith("TMPro.")) continue;
                var textProp = c.GetType().GetProperty("text");
                if (textProp == null) continue;
                string? cur = textProp.GetValue(c) as string;
                if (cur == null) continue;
                string lower = cur.ToLowerInvariant();
                if (lower.Contains("diplomacy") || lower.Contains("not an option"))
                {
                    textProp.SetValue(c, newTitle);
                    c.GetType().GetProperty("color")?.SetValue(c, color);
                    hits++;
                }
            }

            foreach (var t in canvas.GetComponentsInChildren<Text>(true))
            {
                if (t == null || t.text == null) continue;
                string lower = t.text.ToLowerInvariant();
                if (lower.Contains("diplomacy") || lower.Contains("not an option"))
                {
                    t.text = newTitle;
                    t.color = color;
                    hits++;
                }
            }
            return hits;
        }

        private int TintBackground(Canvas canvas, Color tint)
        {
            Image? largest = null;
            float largestArea = 0;
            foreach (var img in canvas.GetComponentsInChildren<Image>(true))
            {
                if (img == null) continue;
                if (img.gameObject.name.Contains("DINOForge")) continue;
                var rt = img.GetComponent<RectTransform>();
                if (rt == null) continue;
                float area = rt.rect.width * rt.rect.height;
                if (area > largestArea) { largestArea = area; largest = img; }
            }
            if (largest != null)
            {
                largest.color = new Color(tint.r, tint.g, tint.b, 0.85f);
                return 1;
            }
            return 0;
        }

        // ── EPIC-027 visual takeover helpers ─────────────────────────────────────

        /// <summary>
        /// Replaces the menu background with the pack's themed PNG. DINO renders its menu
        /// backdrop as a 3D camera scene (not a UGUI Image), so a sprite swap on an existing
        /// Image is unreliable. Instead we INJECT a DINOForge-owned full-canvas Image as the
        /// first sibling (drawn behind every interactive element but above the 3D scene),
        /// which deterministically covers the vanilla backdrop. We also still swap any
        /// existing large background Image's sprite as a belt-and-braces measure.
        /// Returns false when no art is supplied so the caller can fall back to tinting.
        /// </summary>
        private bool TrySwapBackground(Canvas canvas, ThemeData theme, string packId)
        {
            if (string.IsNullOrEmpty(theme.BackgroundImage)) return false;
            Sprite? bgSprite = LoadSpriteFromPack(packId, theme.BackgroundImage!);
            if (bgSprite == null) return false;

            // 1) Inject a full-canvas overlay (idempotent per scene).
            var existing = canvas.transform.Find("DINOForge_ModBackground");
            if (existing == null)
            {
                var bgGo = new GameObject("DINOForge_ModBackground", typeof(RectTransform));
                bgGo.transform.SetParent(canvas.transform, false);
                var bgImg = bgGo.AddComponent<Image>();
                bgImg.sprite = bgSprite;
                bgImg.type = Image.Type.Simple;
                bgImg.preserveAspect = false;       // fill the whole canvas
                bgImg.raycastTarget = false;        // never block button clicks (Pattern #235)
                var rt = bgGo.GetComponent<RectTransform>();
                rt.anchorMin = Vector2.zero;
                rt.anchorMax = Vector2.one;
                rt.offsetMin = Vector2.zero;
                rt.offsetMax = Vector2.zero;
                rt.SetAsFirstSibling();             // behind buttons/logo, above 3D scene
            }

            // 2) Belt-and-braces: also swap an existing large background Image if present.
            Image? largest = FindLargestImage(canvas);
            if (largest != null)
            {
                largest.sprite = bgSprite;
                largest.color = Color.white;
                largest.type = Image.Type.Simple;
                largest.preserveAspect = false;
            }
            return true;
        }

        /// <summary>
        /// Injects a DINOForge-owned logo <see cref="Image"/> at upper-center of the menu
        /// canvas using the pack's logo PNG. The image never blocks raycasts so menu
        /// buttons remain clickable (Pattern #235). Idempotent per scene.
        /// </summary>
        private bool TryInjectLogo(Canvas canvas, ThemeData theme, string packId)
        {
            if (string.IsNullOrEmpty(theme.Logo)) return false;

            // Already injected this scene? bail out (idempotent).
            var existing = canvas.transform.Find("DINOForge_ModLogo");
            if (existing != null) return true;

            Sprite? logoSprite = LoadSpriteFromPack(packId, theme.Logo!);
            if (logoSprite == null) return false;

            var logoGo = new GameObject("DINOForge_ModLogo", typeof(RectTransform));
            logoGo.transform.SetParent(canvas.transform, false);
            var logoImg = logoGo.AddComponent<Image>();
            logoImg.sprite = logoSprite;
            logoImg.preserveAspect = true;
            logoImg.raycastTarget = false;          // MUST NOT block menu button clicks

            var rt = logoGo.GetComponent<RectTransform>();
            rt.anchorMin = new Vector2(0.5f, 0.78f);
            rt.anchorMax = new Vector2(0.5f, 0.98f);
            rt.pivot = new Vector2(0.5f, 0.5f);
            rt.offsetMin = new Vector2(-450f, 0f);  // ~900px wide logo plate
            rt.offsetMax = new Vector2(450f, 0f);
            rt.SetAsLastSibling();                   // render above background
            return true;
        }

        /// <summary>
        /// Swaps the frame <see cref="Image"/> sprite on each native menu button to the
        /// pack's themed 9-slice button art (normal + highlighted states). Returns the
        /// number of buttons restyled. No-op when no frame art is supplied.
        /// </summary>
        private int ApplyButtonFrames(Canvas canvas, ThemeData theme, string packId)
        {
            if (string.IsNullOrEmpty(theme.ButtonFrame)) return 0;
            Sprite? normal = LoadSpriteFromPack(packId, theme.ButtonFrame!);
            if (normal == null) return 0;
            Sprite? hover = string.IsNullOrEmpty(theme.ButtonFrameHover)
                ? normal
                : (LoadSpriteFromPack(packId, theme.ButtonFrameHover!) ?? normal);

            int hits = 0;
            foreach (var sel in canvas.GetComponentsInChildren<Selectable>(false))
            {
                if (sel == null) continue;
                string n = sel.gameObject.name;
                if (n.Contains("DINOForge") || n.Contains("Mods_Button")) continue;
                if (sel is Slider || sel is Scrollbar || sel is Toggle || sel is Dropdown || sel is InputField) continue;

                // Prefer the Selectable's own targetGraphic; else its background Image.
                Image? frame = sel.targetGraphic as Image ?? sel.GetComponent<Image>();
                if (frame == null) continue;
                try
                {
                    frame.sprite = normal;
                    frame.type = Image.Type.Sliced;
                    frame.color = Color.white;          // let the frame art carry the color
                    // Drive hover/pressed via SpriteState so Unity swaps frames natively.
                    var ss = sel.spriteState;
                    ss.highlightedSprite = hover;
                    ss.pressedSprite = hover;
                    sel.spriteState = ss;
                    sel.transition = Selectable.Transition.SpriteSwap;
                    hits++;
                }
                catch { /* safe-swallow: best-effort frame swap */ }
            }
            return hits;
        }

        /// <summary>
        /// Zeroes the alpha on the native title text once a logo sprite covers it.
        /// Scans the WHOLE scene (DINO's title may live on a different canvas than the
        /// menu-button canvas), matching by the vanilla title text content.
        /// </summary>
        private int HideTitle(Canvas canvas)
        {
            int hits = 0;
            // TMP text across the whole scene (reflection — no compile-time TMPro ref).
            foreach (var c in UnityEngine.Object.FindObjectsOfType<Component>())
            {
                if (c == null) continue;
                if (!(c.GetType().FullName ?? "").StartsWith("TMPro.")) continue;
                var textProp = c.GetType().GetProperty("text");
                string? cur = textProp?.GetValue(c) as string;
                if (cur == null) continue;
                string lower = cur.ToLowerInvariant();
                if (lower.Contains("diplomacy") || lower.Contains("not an option"))
                {
                    c.GetType().GetProperty("color")?.SetValue(c, Color.clear);
                    var go = (c as Component)?.gameObject;
                    if (go != null) go.SetActive(false);     // also disable so it cannot re-show
                    hits++;
                }
            }
            // Legacy UGUI Text across the whole scene.
            foreach (var t in UnityEngine.Object.FindObjectsOfType<Text>())
            {
                if (t == null || t.text == null) continue;
                string lower = t.text.ToLowerInvariant();
                if (lower.Contains("diplomacy") || lower.Contains("not an option"))
                {
                    t.color = Color.clear;
                    t.gameObject.SetActive(false);
                    hits++;
                }
            }
            // DINO may render the title as an Image (logo sprite) rather than text —
            // hide any non-DINOForge Image whose GameObject name hints at a logo/title.
            foreach (var img in canvas.GetComponentsInChildren<Image>(true))
            {
                if (img == null) continue;
                string gn = img.gameObject.name;
                if (gn.IndexOf("DINOForge", StringComparison.Ordinal) >= 0) continue;
                if (gn.IndexOf("logo", StringComparison.OrdinalIgnoreCase) >= 0
                    || gn.IndexOf("title", StringComparison.OrdinalIgnoreCase) >= 0)
                {
                    img.enabled = false;
                    hits++;
                }
            }
            return hits;
        }

        private static Image? FindLargestImage(Canvas canvas)
        {
            Image? largest = null;
            float largestArea = 0;
            foreach (var img in canvas.GetComponentsInChildren<Image>(true))
            {
                if (img == null) continue;
                if (img.gameObject.name.Contains("DINOForge")) continue;
                var rt = img.GetComponent<RectTransform>();
                if (rt == null) continue;
                float area = rt.rect.width * rt.rect.height;
                if (area > largestArea) { largestArea = area; largest = img; }
            }
            return largest;
        }

        /// <summary>
        /// Loads a pack-relative PNG into a <see cref="Sprite"/> using Unity's
        /// <c>Texture2D.LoadImage</c> (no AssetBundle, no Addressables needed for raw 2D art).
        /// Resolves against the pack's deployed directory; returns null (never throws) when
        /// the file is absent so all takeover steps degrade gracefully.
        /// </summary>
        private Sprite? LoadSpriteFromPack(string packId, string relativePath)
        {
            try
            {
                string full = Path.Combine(Path.Combine(_packsDirectory, packId), relativePath.Replace('/', Path.DirectorySeparatorChar));
                if (!File.Exists(full))
                {
                    // Always surface the ABSOLUTE path tried so a runtime path mismatch is
                    // immediately visible in the live log (root-cause of the tint-only fallback).
                    _log?.LogWarning($"[MainMenuThemer] takeover art MISSING — tried abs path: '{full}' (packsDir='{_packsDirectory}', packId='{packId}', rel='{relativePath}')"); // pattern-96-ok: diagnostic
                    return null;
                }
                _log?.LogInfo($"[MainMenuThemer] takeover art LOADED: '{full}'"); // pattern-96-ok: diagnostic
                byte[] data = File.ReadAllBytes(full);
                var tex = new Texture2D(2, 2, TextureFormat.RGBA32, mipChain: false)
                {
                    filterMode = FilterMode.Bilinear,
                    wrapMode = TextureWrapMode.Clamp
                };
                if (!tex.LoadImage(data)) return null;
                bool sliced = relativePath.IndexOf("btn", StringComparison.OrdinalIgnoreCase) >= 0
                              || relativePath.IndexOf("button", StringComparison.OrdinalIgnoreCase) >= 0
                              || relativePath.IndexOf("frame", StringComparison.OrdinalIgnoreCase) >= 0;
                // 9-slice insets for 256x96 button art are 18px (see assets/svg button design notes);
                // scale proportionally for other sizes.
                Vector4 border = Vector4.zero;
                if (sliced)
                {
                    float bx = tex.width * (18f / 256f);
                    float by = tex.height * (18f / 96f);
                    border = new Vector4(bx, by, bx, by);
                }
                return Sprite.Create(
                    tex,
                    new Rect(0f, 0f, tex.width, tex.height),
                    new Vector2(0.5f, 0.5f),
                    pixelsPerUnit: 100f,
                    extrude: 0,
                    meshType: SpriteMeshType.FullRect,
                    border: border);
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[MainMenuThemer] LoadSpriteFromPack failed '{relativePath}': {ex.Message}"); // pattern-96-ok: diagnostic
                return null;
            }
        }

        // ── EPIC-027 font takeover ───────────────────────────────────────────────

        [System.Runtime.InteropServices.DllImport("gdi32.dll", CharSet = System.Runtime.InteropServices.CharSet.Auto)]
        private static extern int AddFontResourceEx(string lpFileName, uint fl, System.IntPtr pdv);

        /// <summary>
        /// Loads the pack's shipped TTF and produces a <c>TMP_FontAsset</c> via reflection
        /// (Runtime has no compile-time TMPro reference). The TTF is registered with the OS
        /// font subsystem (<c>AddFontResourceEx</c>, process-private) so Unity's
        /// <c>Font.CreateDynamicFontFromOSFont</c> can rasterise it; the resulting dynamic
        /// <see cref="Font"/> is wrapped with <c>TMP_FontAsset.CreateFontAsset(Font)</c>.
        /// Result is cached per font path. Returns null (never throws) on any failure so the
        /// reskin degrades to the native font.
        /// </summary>
        private UnityEngine.Object? TryLoadFontAsset(ThemeData theme, string packId)
        {
            if (string.IsNullOrEmpty(theme.Font)) return null;
            try
            {
                string absBase = Path.Combine(Path.Combine(_packsDirectory, packId), theme.Font!.Replace('/', Path.DirectorySeparatorChar));
                // Try the path as-is first, then common font extensions in case the YAML
                // omits the extension (e.g. font: sw_menu_font with no .ttf).
                string[] fontExts = { "", ".ttf", ".otf", ".asset" };
                string? full = fontExts
                    .Select(ext => absBase + ext)
                    .FirstOrDefault(p => File.Exists(p));
                if (full == null)
                {
                    _log?.LogWarning($"[MainMenuThemer] font MISSING — tried abs path: '{absBase}' (+ .ttf/.otf/.asset) (packsDir='{_packsDirectory}', packId='{packId}', rel='{theme.Font}')"); // pattern-96-ok: diagnostic
                    return null;
                }

                // Cache hit for the same file.
                if (_cachedFontAsset != null && _cachedFontKey == full) return _cachedFontAsset;

                string family = string.IsNullOrEmpty(theme.FontFamily) ? "Kenney Future Narrow" : theme.FontFamily!;

                // Strategy 1: try TMP_FontAsset.CreateFontAsset(byte[]) via reflection — TMP 3.x+
                // has a bytes overload that bypasses the OS font subsystem entirely.
                Type? tmpFontAssetTypeEarly = FindType("TMPro.TMP_FontAsset");
                if (tmpFontAssetTypeEarly != null)
                {
                    MethodInfo? byteOverload = tmpFontAssetTypeEarly
                        .GetMethods(BindingFlags.Public | BindingFlags.Static)
                        .FirstOrDefault(m => m.Name == "CreateFontAsset"
                                             && m.GetParameters().Length >= 1
                                             && m.GetParameters()[0].ParameterType == typeof(byte[]));
                    if (byteOverload != null)
                    {
                        try
                        {
                            byte[] fontBytes = File.ReadAllBytes(full);
                            object?[] byteArgs = new object?[byteOverload.GetParameters().Length];
                            byteArgs[0] = fontBytes;
                            // Fill optional params with defaults.
                            ParameterInfo[] bps = byteOverload.GetParameters();
                            for (int i = 1; i < bps.Length; i++)
                                byteArgs[i] = bps[i].HasDefaultValue ? bps[i].DefaultValue : null;
                            UnityEngine.Object? byteResult = byteOverload.Invoke(null, byteArgs) as UnityEngine.Object;
                            if (byteResult != null)
                            {
                                _log?.LogInfo($"[MainMenuThemer] TMP bytes overload succeeded for '{full}'"); // pattern-96-ok: diagnostic
                                UnityEngine.Object.DontDestroyOnLoad(byteResult);
                                _cachedFontAsset = byteResult;
                                _cachedFontKey = full;
                                return byteResult;
                            }
                            _log?.LogWarning("[MainMenuThemer] TMP bytes overload returned null — falling back to Font(path)"); // pattern-96-ok: diagnostic
                        }
                        catch (Exception ex) { _log?.LogWarning($"[MainMenuThemer] TMP bytes overload threw: {ex.InnerException?.Message ?? ex.Message}"); } // pattern-96-ok: diagnostic
                    }
                }

                // Strategy 2: Font(path) constructor (Unity 2021.3 supports direct path loading),
                // then CreateFontAsset(Font). Does not require OS font registration.
                Font? srcFont = null;
                try
                {
                    srcFont = new Font(full);
                    _log?.LogInfo($"[MainMenuThemer] Font(path) constructor used for '{full}'"); // pattern-96-ok: diagnostic
                }
                catch (Exception ex) { _log?.LogWarning($"[MainMenuThemer] Font(path) constructor failed: {ex.Message}"); } // pattern-96-ok: diagnostic

                // Strategy 2b: write TTF bytes to a temp file and try Font(tempPath), which
                // avoids the "no embedded sourceFontFile" problem that causes CreateFontAsset
                // to return null for fonts loaded via CreateDynamicFontFromOSFont.
                if (srcFont == null)
                {
                    srcFont = TryLoadTTFAsFont(full);
                }

                // Try CreateFontAsset(Font) for the font loaded via Strategy 2 or 2b before
                // falling back to the OS font subsystem, since OS-font-based Font objects lack
                // an embedded sourceFontFile and always cause CreateFontAsset to return null.
                if (srcFont != null)
                {
                    Type? tmpTypeEarly2 = FindType("TMPro.TMP_FontAsset");
                    if (tmpTypeEarly2 != null)
                    {
                        MethodInfo[] earlyOverloads = tmpTypeEarly2
                            .GetMethods(BindingFlags.Public | BindingFlags.Static)
                            .Where(m => m.Name == "CreateFontAsset"
                                        && m.GetParameters().Length >= 1
                                        && m.GetParameters()[0].ParameterType == typeof(Font))
                            .OrderByDescending(m => m.GetParameters().Length)
                            .ToArray();
                        Font capturedFont = srcFont;
                        foreach (MethodInfo create2 in earlyOverloads)
                        {
                            ParameterInfo[] ps2 = create2.GetParameters();
                            object?[] args2 = new object?[ps2.Length];
                            args2[0] = capturedFont;
                            for (int i = 1; i < ps2.Length; i++)
                            {
                                ParameterInfo pi2 = ps2[i];
                                string pn2 = pi2.Name ?? "";
                                Type pt2 = pi2.ParameterType;
                                object? val2;
                                if (pt2 == typeof(int))
                                {
                                    if (pn2.IndexOf("samplingPointSize", StringComparison.OrdinalIgnoreCase) >= 0 || pn2.IndexOf("pointSize", StringComparison.OrdinalIgnoreCase) >= 0) val2 = 90;
                                    else if (pn2.IndexOf("padding", StringComparison.OrdinalIgnoreCase) >= 0) val2 = 9;
                                    else if (pn2.IndexOf("Width", StringComparison.OrdinalIgnoreCase) >= 0 || pn2.IndexOf("Height", StringComparison.OrdinalIgnoreCase) >= 0) val2 = 1024;
                                    else val2 = pi2.HasDefaultValue ? pi2.DefaultValue : 0;
                                }
                                else if (pt2 == typeof(bool)) val2 = pi2.HasDefaultValue ? pi2.DefaultValue : true;
                                else if (pt2.IsEnum) val2 = pi2.HasDefaultValue ? pi2.DefaultValue : EnumDefaultByName(pt2, pn2);
                                else val2 = pi2.HasDefaultValue ? pi2.DefaultValue : (pt2.IsValueType ? Activator.CreateInstance(pt2) : null);
                                args2[i] = val2;
                            }
                            UnityEngine.Object? earlyResult2 = null;
                            try { earlyResult2 = create2.Invoke(null, args2) as UnityEngine.Object; }
                            catch (Exception ex2) { _log?.LogWarning($"[MainMenuThemer] CreateFontAsset(Font path/temp, {ps2.Length} args) threw: {ex2.InnerException?.Message ?? ex2.Message}"); } // pattern-96-ok: diagnostic
                            if (earlyResult2 != null)
                            {
                                _log?.LogInfo($"[MainMenuThemer] CreateFontAsset succeeded via Font(path/temp) {ps2.Length}-arg overload"); // pattern-96-ok: diagnostic
                                UnityEngine.Object.DontDestroyOnLoad(earlyResult2);
                                _cachedFontAsset = earlyResult2;
                                _cachedFontKey = full;
                                return earlyResult2;
                            }
                            _log?.LogWarning($"[MainMenuThemer] CreateFontAsset(Font path/temp, {ps2.Length} args) returned null"); // pattern-96-ok: diagnostic
                        }
                    }
                    // Reset srcFont so Strategy 3 can still try OS registration.
                    srcFont = null;
                }

                // Strategy 3 (last resort): AddFontResourceEx + CreateDynamicFontFromOSFont.
                if (srcFont == null)
                {
                    string osFontFamily = string.IsNullOrEmpty(theme.FontFamily) ? "Kenney Future Narrow" : theme.FontFamily!;
                    int added = 0;
                    try { added = AddFontResourceEx(full, 0x10 /*FR_PRIVATE*/, System.IntPtr.Zero); }
                    catch (Exception ex) { _log?.LogWarning($"[MainMenuThemer] AddFontResourceEx failed: {ex.Message}"); } // pattern-96-ok: diagnostic
                    _log?.LogInfo($"[MainMenuThemer] font registered: '{full}' family='{osFontFamily}' addFontResult={added}"); // pattern-96-ok: diagnostic
                    srcFont = Font.CreateDynamicFontFromOSFont(osFontFamily, 48);
                    if (srcFont == null)
                    {
                        _log?.LogWarning($"[MainMenuThemer] All font loading strategies failed for '{full}'"); // pattern-96-ok: diagnostic
                        return null;
                    }
                }

                // TMP_FontAsset.CreateFontAsset(Font) via reflection — Unity.TextMeshPro.dll.
                Type? tmpFontAssetType = FindType("TMPro.TMP_FontAsset");
                if (tmpFontAssetType == null)
                {
                    _log?.LogWarning("[MainMenuThemer] TMPro.TMP_FontAsset type not found — cannot build TMP font"); // pattern-96-ok: diagnostic
                    return null;
                }
                // Prefer the richest CreateFontAsset(Font, ...) overload so we can pass real
                // atlas params — the bare CreateFontAsset(Font) path returns null for a font
                // produced by CreateDynamicFontFromOSFont (no embedded sourceFontFile).
                MethodInfo[] overloads = tmpFontAssetType
                    .GetMethods(BindingFlags.Public | BindingFlags.Static)
                    .Where(m => m.Name == "CreateFontAsset"
                                && m.GetParameters().Length >= 1
                                && m.GetParameters()[0].ParameterType == typeof(Font))
                    .OrderByDescending(m => m.GetParameters().Length)
                    .ToArray();
                if (overloads.Length == 0)
                {
                    _log?.LogWarning("[MainMenuThemer] TMP_FontAsset.CreateFontAsset(Font) overload not found"); // pattern-96-ok: diagnostic
                    return null;
                }

                UnityEngine.Object? fontAsset = null;
                foreach (MethodInfo create in overloads)
                {
                    ParameterInfo[] ps = create.GetParameters();
                    object?[] args = new object?[ps.Length];
                    args[0] = srcFont;
                    for (int i = 1; i < ps.Length; i++)
                    {
                        ParameterInfo pi = ps[i];
                        string pn = pi.Name ?? "";
                        Type pt = pi.ParameterType;
                        object? val;
                        if (pt == typeof(int))
                        {
                            if (pn.IndexOf("samplingPointSize", StringComparison.OrdinalIgnoreCase) >= 0 || pn.IndexOf("pointSize", StringComparison.OrdinalIgnoreCase) >= 0) val = 90;
                            else if (pn.IndexOf("padding", StringComparison.OrdinalIgnoreCase) >= 0) val = 9;
                            else if (pn.IndexOf("Width", StringComparison.OrdinalIgnoreCase) >= 0 || pn.IndexOf("Height", StringComparison.OrdinalIgnoreCase) >= 0) val = 1024;
                            else val = pi.HasDefaultValue ? pi.DefaultValue : 0;
                        }
                        else if (pt == typeof(bool))
                        {
                            val = pi.HasDefaultValue ? pi.DefaultValue : true; // enableMultiAtlasSupport
                        }
                        else if (pt.IsEnum)
                        {
                            // GlyphRenderMode → SDFAA (common value), AtlasPopulationMode → Dynamic.
                            val = pi.HasDefaultValue ? pi.DefaultValue : EnumDefaultByName(pt, pn);
                        }
                        else
                        {
                            val = pi.HasDefaultValue ? pi.DefaultValue : (pt.IsValueType ? Activator.CreateInstance(pt) : null);
                        }
                        args[i] = val;
                    }
                    try
                    {
                        fontAsset = create.Invoke(null, args) as UnityEngine.Object;
                    }
                    catch (Exception ex)
                    {
                        _log?.LogWarning($"[MainMenuThemer] CreateFontAsset({ps.Length} args) threw: {ex.InnerException?.Message ?? ex.Message}"); // pattern-96-ok: diagnostic
                        fontAsset = null;
                    }
                    if (fontAsset != null)
                    {
                        _log?.LogInfo($"[MainMenuThemer] CreateFontAsset succeeded via {ps.Length}-arg overload"); // pattern-96-ok: diagnostic
                        break;
                    }
                    _log?.LogWarning($"[MainMenuThemer] CreateFontAsset({ps.Length} args) returned null — trying next overload"); // pattern-96-ok: diagnostic
                }
                if (fontAsset == null)
                {
                    _log?.LogWarning("[MainMenuThemer] CreateFontAsset returned null for all overloads"); // pattern-96-ok: diagnostic
                    return null;
                }
                UnityEngine.Object.DontDestroyOnLoad(fontAsset);
                _cachedFontAsset = fontAsset;
                _cachedFontKey = full;
                _log?.LogInfo($"[MainMenuThemer] TMP_FontAsset created from '{full}' family='{family}'"); // pattern-96-ok: diagnostic
                return fontAsset;
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[MainMenuThemer] TryLoadFontAsset failed: {ex.Message}"); // pattern-96-ok: diagnostic
                return null;
            }
        }

        /// <summary>
        /// Assigns the runtime <c>TMP_FontAsset</c> to every <c>TMP_Text</c> on the menu canvas
        /// via the reflected <c>font</c> property (no compile-time TMPro reference). Skips
        /// DINOForge-owned objects. Returns the number of labels re-fonted.
        /// </summary>
        private int ApplyFont(Canvas canvas, UnityEngine.Object fontAsset)
        {
            int hits = 0;
            try
            {
                foreach (var c in canvas.GetComponentsInChildren<Component>(true))
                {
                    if (c == null) continue;
                    if (!(c.GetType().FullName ?? "").StartsWith("TMPro.")) continue;
                    var fontProp = c.GetType().GetProperty("font");
                    if (fontProp == null || !fontProp.CanWrite) continue;
                    if (!fontProp.PropertyType.IsInstanceOfType(fontAsset)) continue;
                    try
                    {
                        fontProp.SetValue(c, fontAsset);
                        hits++;
                    }
                    catch { /* safe-swallow: best-effort per-label font swap */ }
                }
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[MainMenuThemer] ApplyFont failed: {ex.Message}"); // pattern-96-ok: diagnostic
            }
            return hits;
        }

        /// <summary>
        /// Writes the TTF file to a system temp path and constructs a <see cref="Font"/> from
        /// that path. Unity's <c>Font(string path)</c> constructor requires the file to exist
        /// at a stable, extension-bearing path — some Unity builds reject the original pack
        /// path because it lives inside a subdirectory structure that the Mono file resolver
        /// treats as a resource bundle. Copying to %TEMP% ensures a plain on-disk path.
        /// Returns null (never throws) on any failure.
        /// </summary>
        private Font? TryLoadTTFAsFont(string ttfPath)
        {
            try
            {
                byte[] bytes = File.ReadAllBytes(ttfPath);
                string tempPath = Path.Combine(
                    Path.GetTempPath(),
                    "DINOForge_" + Path.GetFileName(ttfPath));
                File.WriteAllBytes(tempPath, bytes);
                Font? font = null;
                try { font = new Font(tempPath); }
                catch (Exception ex) { _log?.LogWarning($"[MainMenuThemer] TryLoadTTFAsFont Font(tempPath) failed: {ex.Message}"); } // pattern-96-ok: diagnostic
                if (font != null)
                    _log?.LogInfo($"[MainMenuThemer] TryLoadTTFAsFont loaded '{ttfPath}' via temp '{tempPath}'"); // pattern-96-ok: diagnostic
                return font;
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[MainMenuThemer] TryLoadTTFAsFont failed: {ex.Message}"); // pattern-96-ok: diagnostic
                return null;
            }
        }

        /// <summary>Best-effort enum default for TMP CreateFontAsset params: SDFAA for the
        /// glyph render mode, Dynamic for the atlas population mode, else the first value.</summary>
        private static object EnumDefaultByName(Type enumType, string paramName)
        {
            string[] names = Enum.GetNames(enumType);
            string[] preferred = paramName.IndexOf("population", StringComparison.OrdinalIgnoreCase) >= 0
                ? new[] { "Dynamic" }
                : new[] { "SDFAA", "SDF", "SMOOTH_HINTED", "SMOOTH" };
            foreach (string want in preferred)
                foreach (string n in names)
                    if (string.Equals(n, want, StringComparison.OrdinalIgnoreCase))
                        return Enum.Parse(enumType, n);
            return Enum.GetValues(enumType).GetValue(0)!;
        }

        private static Type? FindType(string fullName)
        {
            Type? t = Type.GetType(fullName);
            if (t != null) return t;
            foreach (var asm in AppDomain.CurrentDomain.GetAssemblies())
            {
                try { t = asm.GetType(fullName); }
                catch { t = null; }
                if (t != null) return t;
            }
            return null;
        }

        private int RestyleSelectables(Canvas canvas, Color primary, Color secondary, Color text, Color accent)
        {
            int hits = 0;
            foreach (var sel in canvas.GetComponentsInChildren<Selectable>(false))
            {
                if (sel == null) continue;
                string n = sel.gameObject.name;
                if (n.Contains("DINOForge") || n.Contains("Mods_Button")) continue;
                if (sel is Slider || sel is Scrollbar || sel is Toggle || sel is Dropdown || sel is InputField) continue;
                try
                {
                    var colors = sel.colors;
                    colors.normalColor = new Color(secondary.r, secondary.g, secondary.b, 0.9f);
                    colors.highlightedColor = new Color(primary.r, primary.g, primary.b, 0.85f);
                    colors.pressedColor = new Color(accent.r, accent.g, accent.b, 1f);
                    colors.selectedColor = new Color(primary.r, primary.g, primary.b, 0.7f);
                    sel.colors = colors;
                    hits++;
                }
                catch { /* safe-swallow: best-effort styling */ }
            }
            return hits;
        }

        private int RewriteLabels(Canvas canvas, Color textCol)
        {
            var labels = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
            {
                { "New Game", "New Campaign" },
                { "Continue", "Resume Campaign" },
                { "Load Game", "Load Campaign" },
                { "Special Missions", "Clone Wars Missions" }
            };
            int hits = 0;

            foreach (var c in canvas.GetComponentsInChildren<Component>(true))
            {
                if (c == null) continue;
                if (!(c.GetType().FullName ?? "").StartsWith("TMPro.")) continue;
                var textProp = c.GetType().GetProperty("text");
                if (textProp == null) continue;
                string? cur = textProp.GetValue(c) as string;
                if (cur == null) continue;
                foreach (var kv in labels)
                {
                    if (string.Equals(cur.Trim(), kv.Key, StringComparison.OrdinalIgnoreCase))
                    {
                        textProp.SetValue(c, kv.Value);
                        c.GetType().GetProperty("color")?.SetValue(c, textCol);
                        hits++;
                        break;
                    }
                }
            }

            foreach (var t in canvas.GetComponentsInChildren<Text>(true))
            {
                if (t == null || t.text == null) continue;
                foreach (var kv in labels)
                {
                    if (string.Equals(t.text.Trim(), kv.Key, StringComparison.OrdinalIgnoreCase))
                    {
                        t.text = kv.Value;
                        t.color = textCol;
                        hits++;
                        break;
                    }
                }
            }
            return hits;
        }

        // ──────────────────────────────────────────────────────────────────────────
        // EPIC-027 / subpage FULL TAKEOVER
        //
        // The main-menu takeover above themes only the MainMenu canvas. Subpages
        // (Options + GAME/VIDEO/SOUND/CONTROLS/TWITCH tabs, game create/select, and the
        // in-game HUD/build/pause panels) are SEPARATE canvases that the user reaches by
        // clicking buttons — they open AFTER the main menu and re-open repeatedly. The
        // previous aux path only recolored TMP text gold, leaving panels, frames and
        // native controls (sliders, ◄►selectors, tab rails) vanilla.
        //
        // ApplyToAuxiliaryMenus performs the SAME full takeover the main menu got, but
        // generalized per-surface and driven each pump frame (idempotent via
        // DINOForge-owned marker objects + sentinel components):
        //   1. PANEL/BACKGROUND — inject a themed full-rect background Image behind each
        //      subpage panel (overlay; DINO panels may have no UGUI bg of their own).
        //   2. 9-SLICE FRAMES   — apply the pack's btn_normal/btn_hover art to every
        //      button + a menu_bg-derived frame to large panel containers.
        //   3. NATIVE CONTROLS  — restyle sliders (fill+handle gold), ◄► selector arrow
        //      buttons (frame + gold tint), and tab rails (active/inactive coloring).
        //   4. FONT             — apply the themed TMP font to every subpage label.
        // All steps degrade gracefully (missing art ⇒ tint/color fallback, never throws).
        public bool ApplyToAuxiliaryMenus(IReadOnlyList<PackDisplayInfo> packs)
        {
            try
            {
                if (_auxTheme == null || _auxPack == null)
                {
                    if (!ResolveAuxTheme(packs)) return false;
                }
                ThemeData theme = _auxTheme!;
                PackDisplayInfo pack = _auxPack!;

                Canvas[] canvases = UnityEngine.Object.FindObjectsOfType<Canvas>();
                // Cheap guard: only do the (slightly heavier) scan when the live canvas
                // set changes — opening/closing a subpage adds/removes a canvas.
                if (canvases.Length == _auxLastCanvasCount) return true;
                _auxLastCanvasCount = canvases.Length;

                ColorUtility.TryParseHtmlString(theme.PrimaryColor, out Color primary);
                ColorUtility.TryParseHtmlString(theme.SecondaryColor, out Color secondary);
                ColorUtility.TryParseHtmlString(theme.TextColor, out Color textCol);
                ColorUtility.TryParseHtmlString(theme.AccentColor ?? "#C0392B", out Color accent);
                Color bgTint = new Color(0.04f, 0.04f, 0.10f, 1f);
                if (theme.BackgroundTint != null) ColorUtility.TryParseHtmlString(theme.BackgroundTint, out bgTint);

                Sprite? bgSprite = string.IsNullOrEmpty(theme.BackgroundImage) ? null : LoadSpriteFromPack(pack.Id, theme.BackgroundImage!);
                Sprite? frameNormal = string.IsNullOrEmpty(theme.ButtonFrame) ? null : LoadSpriteFromPack(pack.Id, theme.ButtonFrame!);
                Sprite? frameHover = string.IsNullOrEmpty(theme.ButtonFrameHover)
                    ? frameNormal
                    : (LoadSpriteFromPack(pack.Id, theme.ButtonFrameHover!) ?? frameNormal);
                Sprite? buildHelmetIcon = LoadSpriteFromPack(pack.Id, "assets/ui/icon_build_helmet.png");
                Sprite? buildDroidIcon = LoadSpriteFromPack(pack.Id, "assets/ui/icon_droid_head.png");
                Sprite? buildGunshipIcon = LoadSpriteFromPack(pack.Id, "assets/ui/icon_gunship.png");
                UnityEngine.Object? fontAsset = TryLoadFontAsset(theme, pack.Id);

                int surfaces = 0;
                foreach (Canvas c in canvases)
                {
                    if (c == null || !c.gameObject.activeInHierarchy) continue;
                    string cn = c.name;
                    // Skip the main menu canvas (themed separately) and our own overlays.
                    if (cn.IndexOf("PrimeCanvas", StringComparison.OrdinalIgnoreCase) >= 0) continue;
                    if (cn.IndexOf("DINOForge", StringComparison.Ordinal) >= 0) continue;
                    if (cn.IndexOf("MainMenu", StringComparison.OrdinalIgnoreCase) >= 0) continue;
                    if (!IsAuxSurface(c)) continue;

                    ApplyTakeoverToSurface(c, theme,
                        bgSprite, frameNormal, frameHover,
                        buildHelmetIcon, buildDroidIcon, buildGunshipIcon,
                        fontAsset, primary, secondary, textCol, accent, bgTint);
                    surfaces++;
                }

                if (surfaces > 0)
                {
                    _log?.LogInfo($"[MainMenuThemer] AUX TAKEOVER applied to {surfaces} subpage canvas(es) from '{pack.Id}' (bg={bgSprite != null} frame={frameNormal != null} icons={buildHelmetIcon != null || buildDroidIcon != null || buildGunshipIcon != null} font={fontAsset != null})"); // pattern-96-ok: diagnostic
                    DebugLog.Write("MainMenuThemer", $"AUX TAKEOVER: {surfaces} subpage canvas(es) bg={bgSprite != null} frame={frameNormal != null} icons={buildHelmetIcon != null || buildDroidIcon != null || buildGunshipIcon != null} font={fontAsset != null}");
                }
                return surfaces > 0;
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[MainMenuThemer] ApplyToAuxiliaryMenus failed: {ex.Message}"); // pattern-96-ok: diagnostic
                return false;
            }
        }

        private bool ResolveAuxTheme(IReadOnlyList<PackDisplayInfo> packs)
        {
            if (packs == null || packs.Count == 0) return false;
            PackDisplayInfo? best = null;
            PackDisplayInfo? fallback = null;
            foreach (var p in packs.OrderBy(p => p.Id, StringComparer.Ordinal))
            {
                if (!p.IsEnabled) continue;
                if (!string.Equals(p.Type, "total_conversion", StringComparison.OrdinalIgnoreCase)) continue;
                if (fallback == null) fallback = p;
                string yamlPath = Path.Combine(_packsDirectory, p.Id, "pack.yaml");
                if (File.Exists(yamlPath) && File.ReadAllText(yamlPath, System.Text.Encoding.UTF8).IndexOf("ui_theme:", StringComparison.Ordinal) >= 0)
                {
                    best = p;
                    break;
                }
            }
            best = best ?? fallback;
            if (best == null) return false;
            _auxPack = best;
            _auxTheme = ReadThemeFromDisk(best.Id) ?? new ThemeData { Title = best.Name };
            return true;
        }

        /// <summary>
        /// Heuristic: is this canvas a themeable subpage (Options/settings tab, game
        /// create/select, in-game HUD/build/pause) rather than a transient/system canvas?
        /// We theme any active non-main-menu canvas that carries menu-like content:
        /// a Selectable (button/slider/etc.) or recognizable settings/HUD text. This keeps
        /// the rule ID-free (no hardcoded canvas names) per agent governance.
        /// </summary>
        private static bool IsAuxSurface(Canvas canvas)
        {
            // Must have at least one interactive control or a sizable panel image to be worth theming.
            if (canvas.GetComponentInChildren<Selectable>(false) != null) return true;
            foreach (var img in canvas.GetComponentsInChildren<Image>(false))
            {
                if (img == null) continue;
                var rt = img.GetComponent<RectTransform>();
                if (rt != null && rt.rect.width * rt.rect.height > 40000f) return true; // ~200x200+
            }
            return false;
        }

        /// <summary>Full per-surface takeover: bg overlay + panel/button frames + native
        /// control restyle + font. Idempotent via DINOForge marker objects.</summary>
        private void ApplyTakeoverToSurface(
            Canvas canvas, ThemeData theme,
            Sprite? bgSprite, Sprite? frameNormal, Sprite? frameHover,
            Sprite? buildHelmetIcon, Sprite? buildDroidIcon, Sprite? buildGunshipIcon,
            UnityEngine.Object? fontAsset,
            Color primary, Color secondary, Color textCol, Color accent, Color bgTint)
        {
            // 1) PANEL/BACKGROUND — inject a full-canvas themed backdrop (idempotent).
            InjectSurfaceBackground(canvas, bgSprite, bgTint);

            // 2) 9-SLICE FRAMES on buttons + panel containers.
            ApplyFramesToSurface(canvas, frameNormal, frameHover, primary, secondary, accent, textCol);

            // 3) NATIVE CONTROLS — sliders, ◄► selectors, tab rails.
            RestyleNativeControls(canvas, frameNormal, frameHover, primary, secondary, accent, textCol);

            // 3b) BUILD/PANEL ICONS — swap icon-sized child Images on build buttons.
            ReplaceBuildPanelIcons(canvas, buildHelmetIcon, buildDroidIcon, buildGunshipIcon);

            // 4) FONT.
            if (fontAsset != null) ApplyFont(canvas, fontAsset);

            // Recolor remaining labels to themed text color (does not override frame text).
            RecolorLabels(canvas, textCol);
        }

        private void InjectSurfaceBackground(Canvas canvas, Sprite? bgSprite, Color bgTint)
        {
            // Anchor the backdrop to the largest panel root so it covers the subpage but
            // not the whole screen if the panel is windowed; fall back to the canvas.
            Transform parent = canvas.transform;
            Image? largestPanel = FindLargestImage(canvas);
            if (largestPanel != null && largestPanel.transform.parent != null)
                parent = largestPanel.transform;

            var existing = parent.Find("DINOForge_AuxBackground");
            Image bgImg;
            if (existing == null)
            {
                var bgGo = new GameObject("DINOForge_AuxBackground", typeof(RectTransform));
                bgGo.transform.SetParent(parent, false);
                bgImg = bgGo.AddComponent<Image>();
                bgImg.raycastTarget = false;            // never block control input (Pattern #235)
                var rt = bgGo.GetComponent<RectTransform>();
                rt.anchorMin = Vector2.zero;
                rt.anchorMax = Vector2.one;
                rt.offsetMin = Vector2.zero;
                rt.offsetMax = Vector2.zero;
                rt.SetAsFirstSibling();                 // behind controls
            }
            else
            {
                bgImg = existing.GetComponent<Image>();
                if (bgImg == null) return;
            }
            if (bgSprite != null)
            {
                bgImg.sprite = bgSprite;
                bgImg.type = Image.Type.Simple;
                bgImg.preserveAspect = false;
                bgImg.color = new Color(1f, 1f, 1f, 0.97f);
            }
            else
            {
                // No art: solid dark SW panel tint.
                bgImg.color = new Color(bgTint.r, bgTint.g, bgTint.b, 0.92f);
            }
        }

        private void ApplyFramesToSurface(Canvas canvas, Sprite? normal, Sprite? hover,
            Color primary, Color secondary, Color accent, Color textCol)
        {
            foreach (var sel in canvas.GetComponentsInChildren<Selectable>(false))
            {
                if (sel == null) continue;
                string n = sel.gameObject.name;
                if (n.Contains("DINOForge")) continue;
                // Sliders/scrollbars/toggles/dropdowns are handled by RestyleNativeControls.
                if (sel is Slider || sel is Scrollbar || sel is Toggle || sel is Dropdown || sel is InputField) continue;

                Image? frame = sel.targetGraphic as Image ?? sel.GetComponent<Image>();
                if (frame != null)
                {
                    try
                    {
                        if (normal != null)
                        {
                            frame.sprite = normal;
                            frame.type = Image.Type.Sliced;
                            frame.color = Color.white;
                            var ss = sel.spriteState;
                            ss.highlightedSprite = hover;
                            ss.pressedSprite = hover;
                            ss.selectedSprite = hover;
                            sel.spriteState = ss;
                            sel.transition = Selectable.Transition.SpriteSwap;
                        }
                        else
                        {
                            // No art: drive color transition gold-on-dark.
                            var colors = sel.colors;
                            colors.normalColor = new Color(secondary.r, secondary.g, secondary.b, 0.9f);
                            colors.highlightedColor = new Color(primary.r, primary.g, primary.b, 0.85f);
                            colors.pressedColor = new Color(accent.r, accent.g, accent.b, 1f);
                            colors.selectedColor = new Color(primary.r, primary.g, primary.b, 0.7f);
                            sel.colors = colors;
                        }
                    }
                    catch { /* safe-swallow: best-effort frame swap */ }
                }
            }
        }

        /// <summary>
        /// Restyles DINO's native controls — sliders (fill+handle gold), ◄►selector arrow
        /// buttons (gold-tinted frames), tab rails (active gold / inactive dim). Driven by
        /// component type + GameObject-name hints, never hardcoded IDs.
        /// </summary>
        private void RestyleNativeControls(Canvas canvas, Sprite? frame, Sprite? frameHover,
            Color primary, Color secondary, Color accent, Color textCol)
        {
            // ── Sliders: fill → gold, handle → gold frame, background → dim. ──
            foreach (var slider in canvas.GetComponentsInChildren<Slider>(false))
            {
                if (slider == null || slider.gameObject.name.Contains("DINOForge")) continue;
                try
                {
                    if (slider.fillRect != null)
                    {
                        var fill = slider.fillRect.GetComponent<Image>();
                        if (fill != null) fill.color = primary;
                    }
                    if (slider.handleRect != null)
                    {
                        var handle = slider.handleRect.GetComponent<Image>();
                        if (handle != null)
                        {
                            if (frame != null) { handle.sprite = frame; handle.type = Image.Type.Sliced; handle.color = Color.white; }
                            else handle.color = primary;
                        }
                    }
                    // Slider background track → dim secondary.
                    foreach (var img in slider.GetComponentsInChildren<Image>(true))
                    {
                        if (img == null) continue;
                        bool isFill = slider.fillRect != null && img.transform.IsChildOf(slider.fillRect);
                        bool isHandle = slider.handleRect != null && img.transform.IsChildOf(slider.handleRect);
                        if (!isFill && !isHandle)
                            img.color = new Color(secondary.r, secondary.g, secondary.b, 0.85f);
                    }
                }
                catch { /* safe-swallow */ }
            }

            // ── ◄► selector arrow buttons + tab-rail buttons. ──
            // Heuristic: small square buttons with arrow glyphs ("<",">","◄","►") are
            // selectors; buttons grouped on a horizontal rail are tabs. We tint both with
            // the themed frame and set gold text; active/selected state via SpriteSwap.
            foreach (var btn in canvas.GetComponentsInChildren<Button>(false))
            {
                if (btn == null) continue;
                string gn = btn.gameObject.name;
                if (gn.Contains("DINOForge")) continue;
                string label = GetButtonLabel(btn);
                bool isArrow = label == "<" || label == ">" || label == "◄" || label == "►"
                               || gn.IndexOf("arrow", StringComparison.OrdinalIgnoreCase) >= 0
                               || gn.IndexOf("prev", StringComparison.OrdinalIgnoreCase) >= 0
                               || gn.IndexOf("next", StringComparison.OrdinalIgnoreCase) >= 0;
                bool isTab = gn.IndexOf("tab", StringComparison.OrdinalIgnoreCase) >= 0
                             || gn.IndexOf("category", StringComparison.OrdinalIgnoreCase) >= 0;
                if (!isArrow && !isTab) continue;
                try
                {
                    Image? img = btn.targetGraphic as Image ?? btn.GetComponent<Image>();
                    if (img != null)
                    {
                        if (frame != null) { img.sprite = frame; img.type = Image.Type.Sliced; img.color = Color.white; }
                        else img.color = new Color(secondary.r, secondary.g, secondary.b, 0.9f);
                        var ss = btn.spriteState;
                        ss.highlightedSprite = frameHover;
                        ss.selectedSprite = frameHover;
                        btn.spriteState = ss;
                        if (frame != null) btn.transition = Selectable.Transition.SpriteSwap;
                    }
                    SetSelectableTextColor(btn, primary);
                }
                catch { /* safe-swallow */ }
            }

            // ── Toggles (e.g. fullscreen checkbox) → gold checkmark. ──
            foreach (var toggle in canvas.GetComponentsInChildren<Toggle>(false))
            {
                if (toggle == null || toggle.gameObject.name.Contains("DINOForge")) continue;
                try
                {
                    if (toggle.graphic is Image check) check.color = primary;
                    if (toggle.targetGraphic is Image box) box.color = new Color(secondary.r, secondary.g, secondary.b, 0.9f);
                }
                catch { /* safe-swallow */ }
            }
        }

        /// <summary>
        /// Restyles build-panel/HUD button icon images by mapping recognized button labels
        /// and container names to SW-themed icon sprites. We only write when a mapped sprite
        /// was loaded; if a match has no available art, the native icon is left untouched to
        /// avoid blank-box regressions.
        /// </summary>
        private void ReplaceBuildPanelIcons(Canvas canvas, Sprite? helmetIcon, Sprite? droidIcon, Sprite? gunshipIcon)
        {
            if (helmetIcon == null && droidIcon == null && gunshipIcon == null) return;

            foreach (var btn in canvas.GetComponentsInChildren<Button>(false))
            {
                if (btn == null) continue;
                string bname = btn.gameObject.name;
                if (bname.IndexOf("DINOForge", StringComparison.OrdinalIgnoreCase) >= 0) continue;

                // Consider non-frame child images that look like unit/building icons.
                foreach (var img in btn.GetComponentsInChildren<Image>(true))
                {
                    if (img == null) continue;
                    if (img.sprite == null) continue;
                    if (ReferenceEquals(img, btn.targetGraphic as Image)) continue;
                    if (img.gameObject.name.IndexOf("DINOForge", StringComparison.OrdinalIgnoreCase) >= 0) continue;
                    if (!IsLikelyIconImage(img, btn)) continue;

                    Sprite? chosen = PickBuildPanelIconSprite(btn, bname, helmetIcon, droidIcon, gunshipIcon);
                    if (chosen == null) continue;
                    img.sprite = chosen;
                    img.color = Color.white;
                    img.preserveAspect = true;
                    img.raycastTarget = false;
                }
            }
        }

        private static bool IsLikelyIconImage(Image img, Button button)
        {
            if (img.rectTransform == null || button == null) return false;
            float iconArea = img.rectTransform.rect.width * img.rectTransform.rect.height;
            if (iconArea <= 16f || iconArea >= 16384f) return false;

            var buttonRt = button.GetComponent<RectTransform>();
            if (buttonRt == null) return true;
            float btnArea = buttonRt.rect.width * buttonRt.rect.height;
            if (btnArea <= 0f) return true;
            return iconArea < btnArea * 0.70f;
        }

        private static Sprite? PickBuildPanelIconSprite(Button button, string buttonName, Sprite? helmetIcon, Sprite? droidIcon, Sprite? gunshipIcon)
        {
            string nameHint = buttonName + " " + GetButtonLabel(button) + " " + button.gameObject.name;
            if (string.IsNullOrWhiteSpace(nameHint)) return null;
            string key = nameHint.ToLowerInvariant();

            if ((key.IndexOf("build", StringComparison.Ordinal) >= 0 || key.IndexOf("factory", StringComparison.Ordinal) >= 0) && gunshipIcon != null)
                return gunshipIcon;
            if ((key.IndexOf("unit", StringComparison.Ordinal) >= 0 || key.IndexOf("trooper", StringComparison.Ordinal) >= 0 || key.IndexOf("clone", StringComparison.Ordinal) >= 0 || key.IndexOf("soldier", StringComparison.Ordinal) >= 0) && helmetIcon != null)
                return helmetIcon;
            if ((key.IndexOf("droid", StringComparison.Ordinal) >= 0 || key.IndexOf("drone", StringComparison.Ordinal) >= 0 || key.IndexOf("probe", StringComparison.Ordinal) >= 0) && droidIcon != null)
                return droidIcon;
            if ((key.IndexOf("air", StringComparison.Ordinal) >= 0 || key.IndexOf("aircraft", StringComparison.Ordinal) >= 0 || key.IndexOf("ship", StringComparison.Ordinal) >= 0 || key.IndexOf("fighter", StringComparison.Ordinal) >= 0) && gunshipIcon != null)
                return gunshipIcon;
            return null;
        }

        private static string GetButtonLabel(Button btn)
        {
            foreach (var t in btn.GetComponentsInChildren<Text>(true))
                if (t != null && !string.IsNullOrEmpty(t.text)) return t.text.Trim();
            foreach (var c in btn.GetComponentsInChildren<Component>(true))
            {
                if (c == null || !(c.GetType().FullName ?? "").StartsWith("TMPro.")) continue;
                string? s = c.GetType().GetProperty("text")?.GetValue(c) as string;
                if (!string.IsNullOrEmpty(s)) return s!.Trim();
            }
            return string.Empty;
        }

        private static void SetSelectableTextColor(Selectable sel, Color color)
        {
            foreach (var t in sel.GetComponentsInChildren<Text>(true))
                if (t != null) t.color = color;
            foreach (var c in sel.GetComponentsInChildren<Component>(true))
            {
                if (c == null || !(c.GetType().FullName ?? "").StartsWith("TMPro.")) continue;
                c.GetType().GetProperty("color")?.SetValue(c, color);
            }
        }

        /// <summary>Recolors every TMP/UGUI label on the surface to the themed text color,
        /// skipping DINOForge-owned objects.</summary>
        private void RecolorLabels(Canvas canvas, Color textCol)
        {
            foreach (var c in canvas.GetComponentsInChildren<Component>(true))
            {
                if (c == null) continue;
                if (c.gameObject.name.Contains("DINOForge")) continue;
                if (!(c.GetType().FullName ?? "").StartsWith("TMPro.")) continue;
                try { c.GetType().GetProperty("color")?.SetValue(c, textCol); }
                catch { /* safe-swallow: best-effort per-label recolor */ }
            }
            foreach (var t in canvas.GetComponentsInChildren<Text>(true))
            {
                if (t == null || t.gameObject.name.Contains("DINOForge")) continue;
                t.color = textCol;
            }
        }

        // AssetBundle handles can only be loaded once per process; cache by path.
        private static readonly Dictionary<string, UnityEngine.Object?> _fontCache =
            new Dictionary<string, UnityEngine.Object?>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// Loads the prebuilt TMP SDF font asset (baked offline; runtime
        /// TMP_FontAsset.CreateFontAsset returns null in DINO) from the pack's
        /// AssetBundle and applies it to every menu TMP_Text via reflection.
        /// </summary>
        private int ApplyFont(Canvas canvas, ThemeData theme, PackDisplayInfo pack)
        {
            if (string.IsNullOrEmpty(theme.Font)) return 0;

            // Skip AssetBundle path for raw TTF/OTF — LoadFromFile returns null on non-bundle files.
            string _fontExt = Path.GetExtension(theme.Font!).ToLowerInvariant();
            bool _isTTF = _fontExt == ".ttf" || _fontExt == ".otf";
            UnityEngine.Object? fontAsset = _isTTF ? null : LoadPrebuiltFontAsset(pack.Id, theme.Font!, theme.FontAssetName);
            if (fontAsset == null)
            {
                if (_isTTF)
                    fontAsset = TryLoadFontAsset(theme, pack.Id);
                if (fontAsset == null)
                {
                    _log?.LogWarning($"[MainMenuThemer] Prebuilt font '{theme.Font}' not loaded; skipping font apply."); // pattern-96-ok: diagnostic
                    return 0;
                }
            }

            // Resolve the shared material from the font asset (TMP_FontAsset.material).
            UnityEngine.Object? sharedMat = fontAsset.GetType().GetProperty("material")?.GetValue(fontAsset) as UnityEngine.Object;

            int hits = 0;
            foreach (var c in canvas.GetComponentsInChildren<Component>(true))
            {
                if (c == null) continue;
                Type t = c.GetType();
                if (!(t.FullName ?? "").StartsWith("TMPro.")) continue;

                PropertyInfo? fontProp = t.GetProperty("font");
                if (fontProp == null || !fontProp.CanWrite) continue;
                // Only assign if the target property type is assignable from the asset.
                if (!fontProp.PropertyType.IsInstanceOfType(fontAsset)) continue;

                try
                {
                    fontProp.SetValue(c, fontAsset);
                    if (sharedMat != null)
                    {
                        PropertyInfo? matProp = t.GetProperty("fontSharedMaterial");
                        if (matProp != null && matProp.CanWrite && matProp.PropertyType.IsInstanceOfType(sharedMat))
                            matProp.SetValue(c, sharedMat);
                    }
                    // Force a glyph/layout rebuild so the new atlas takes effect.
                    t.GetMethod("SetAllDirty", Type.EmptyTypes)?.Invoke(c, null);
                    hits++;
                }
                catch (Exception ex)
                {
                    _log?.LogWarning($"[MainMenuThemer] Font apply on '{c.name}' failed: {ex.Message}"); // pattern-96-ok: diagnostic
                }
            }

            DebugLog.Write("MainMenuThemer", $"Prebuilt font applied to {hits} TMP_Text from bundle '{theme.Font}'");
            return hits;
        }

        private UnityEngine.Object? LoadPrebuiltFontAsset(string packId, string relativeFontPath, string? fontAssetName)
        {
            try
            {
                string bundlePath = Path.Combine(_packsDirectory, packId, relativeFontPath.Replace('/', Path.DirectorySeparatorChar));
                if (_fontCache.TryGetValue(bundlePath, out var cached))
                    return cached;

                if (!File.Exists(bundlePath))
                {
                    _log?.LogWarning($"[MainMenuThemer] Font bundle missing: {bundlePath}"); // pattern-96-ok: diagnostic
                    _fontCache[bundlePath] = null;
                    return null;
                }

                AssetBundle bundle = AssetBundle.LoadFromFile(bundlePath);
                if (bundle == null)
                {
                    _log?.LogWarning($"[MainMenuThemer] AssetBundle.LoadFromFile returned null for {bundlePath}"); // pattern-96-ok: diagnostic
                    _fontCache[bundlePath] = null;
                    return null;
                }

                // Find the TMP_FontAsset inside the bundle (by name if provided, else first match).
                UnityEngine.Object? result = null;
                foreach (var obj in bundle.LoadAllAssets())
                {
                    if (obj == null) continue;
                    if (!(obj.GetType().FullName ?? "").StartsWith("TMPro.")) continue;
                    if (!string.IsNullOrEmpty(fontAssetName) &&
                        !string.Equals(obj.name, fontAssetName, StringComparison.Ordinal))
                        continue;
                    result = obj;
                    break;
                }

                if (result == null)
                    _log?.LogWarning($"[MainMenuThemer] No TMP font asset '{fontAssetName}' found in bundle {bundlePath}"); // pattern-96-ok: diagnostic

                _fontCache[bundlePath] = result;
                return result;
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[MainMenuThemer] LoadPrebuiltFontAsset failed: {ex.Message}"); // pattern-96-ok: diagnostic
                return null;
            }
        }

        private sealed class ThemeData
        {
            public string? Title;
            public string? Subtitle;
            public string PrimaryColor = "#FFE81F";
            public string SecondaryColor = "#000000";
            public string? AccentColor = "#C0392B";
            public string TextColor = "#FFE81F";
            public string? BackgroundTint;
            // EPIC-027 visual takeover — pack-relative PNG art paths.
            public string? Logo;
            public string? BackgroundImage;
            public string? ButtonFrame;
            public string? ButtonFrameHover;
            public string? Font;
            public string? FontFamily;
            // #965 baked-TMP-font path: prebuilt TMP_FontAsset name inside the bundle.
            public string? FontAssetName;
        }
    }
}