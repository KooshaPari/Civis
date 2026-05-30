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

        public MainMenuThemer(ManualLogSource log, string packsDirectory)
        {
            _log = log;
            _packsDirectory = packsDirectory ?? string.Empty;
        }

        public void OnSceneChanged() => _applied = false;

        public bool TryApplyTheme(IReadOnlyList<PackDisplayInfo> packs)
        {
            if (_applied) return true;
            if (packs == null || packs.Count == 0) return false;

            // Only ENABLED total_conversion packs are eligible — when multiple TC packs are
            // installed, the user's enable/disable choice in the F10 mod menu decides which
            // one takes over the menu (no hardcoded pack id). Disabled packs are skipped.
            PackDisplayInfo? best = null;
            PackDisplayInfo? fallback = null;
            foreach (var p in packs)
            {
                if (!p.IsEnabled) continue;
                if (!string.Equals(p.Type, "total_conversion", StringComparison.OrdinalIgnoreCase)) continue;
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
                if (fallback == null) fallback = p;
            }
            best = best ?? fallback;
            if (best == null) return false;

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
                    FontFamily = ExtractYamlValue(yaml, idx, "font_family")
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

                // FONT: runtime-create a TMP_FontAsset from the pack's shipped TTF and apply it
                // to every TMP_Text on the menu canvas (reflection — no compile-time TMPro ref).
                bool fontLoaded = false;
                int fontHits = 0;
                UnityEngine.Object? fontAsset = TryLoadFontAsset(theme, pack.Id);
                if (fontAsset != null)
                {
                    fontLoaded = true;
                    fontHits = ApplyFont(canvas, fontAsset);
                }

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
                string full = Path.Combine(Path.Combine(_packsDirectory, packId), theme.Font!.Replace('/', Path.DirectorySeparatorChar));
                if (!File.Exists(full))
                {
                    _log?.LogWarning($"[MainMenuThemer] font MISSING — tried abs path: '{full}' (packsDir='{_packsDirectory}', packId='{packId}', rel='{theme.Font}')"); // pattern-96-ok: diagnostic
                    return null;
                }

                // Cache hit for the same file.
                if (_cachedFontAsset != null && _cachedFontKey == full) return _cachedFontAsset;

                string family = string.IsNullOrEmpty(theme.FontFamily) ? "Kenney Future Narrow" : theme.FontFamily!;

                // Register the TTF with the (process-private) OS font table so
                // CreateDynamicFontFromOSFont can find it by family name.
                int added = 0;
                try { added = AddFontResourceEx(full, 0x10 /*FR_PRIVATE*/, System.IntPtr.Zero); }
                catch (Exception ex) { _log?.LogWarning($"[MainMenuThemer] AddFontResourceEx failed: {ex.Message}"); } // pattern-96-ok: diagnostic
                _log?.LogInfo($"[MainMenuThemer] font registered: '{full}' family='{family}' addFontResult={added}"); // pattern-96-ok: diagnostic

                Font? srcFont = Font.CreateDynamicFontFromOSFont(family, 48);
                if (srcFont == null)
                {
                    _log?.LogWarning($"[MainMenuThemer] CreateDynamicFontFromOSFont returned null for family '{family}'"); // pattern-96-ok: diagnostic
                    return null;
                }

                // TMP_FontAsset.CreateFontAsset(Font) via reflection — Unity.TextMeshPro.dll.
                Type? tmpFontAssetType = FindType("TMPro.TMP_FontAsset");
                if (tmpFontAssetType == null)
                {
                    _log?.LogWarning("[MainMenuThemer] TMPro.TMP_FontAsset type not found — cannot build TMP font"); // pattern-96-ok: diagnostic
                    return null;
                }
                MethodInfo? create = tmpFontAssetType
                    .GetMethods(BindingFlags.Public | BindingFlags.Static)
                    .FirstOrDefault(m => m.Name == "CreateFontAsset"
                                         && m.GetParameters().Length >= 1
                                         && m.GetParameters()[0].ParameterType == typeof(Font));
                if (create == null)
                {
                    _log?.LogWarning("[MainMenuThemer] TMP_FontAsset.CreateFontAsset(Font) overload not found"); // pattern-96-ok: diagnostic
                    return null;
                }

                // Supply defaults for any extra optional params (samplingPointSize, atlas dims, render mode, etc.).
                ParameterInfo[] ps = create.GetParameters();
                object?[] args = new object?[ps.Length];
                args[0] = srcFont;
                for (int i = 1; i < ps.Length; i++)
                    args[i] = ps[i].HasDefaultValue ? ps[i].DefaultValue : (ps[i].ParameterType.IsValueType ? Activator.CreateInstance(ps[i].ParameterType) : null);

                var fontAsset = create.Invoke(null, args) as UnityEngine.Object;
                if (fontAsset == null)
                {
                    _log?.LogWarning("[MainMenuThemer] CreateFontAsset returned null"); // pattern-96-ok: diagnostic
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
        }
    }
}
