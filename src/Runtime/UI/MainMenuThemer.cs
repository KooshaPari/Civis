#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using BepInEx.Logging;
using DINOForge.Runtime.Diagnostics;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Reads the active total_conversion pack's <c>ui_theme</c> section from its <c>pack.yaml</c>
    /// on disk and applies in-place UI element replacement to DINO's main menu canvas.
    ///
    /// Instantiated by RuntimeDriver after pack-load (not a MonoBehaviour).
    /// Uses lightweight string parsing for YAML — Runtime targets netstandard2.0 and does not
    /// reference YamlDotNet.
    /// </summary>
    internal sealed class MainMenuThemer
    {
#pragma warning disable DF1006 // _log is externally owned — we do not dispose it
        private readonly ManualLogSource _log;
#pragma warning restore DF1006
        private readonly string _packsDirectory;
        private bool _applied;

        /// <summary>Whether the theme has been successfully applied to the main menu.</summary>
        public bool IsApplied => _applied;

        /// <summary>
        /// Creates a new MainMenuThemer.
        /// </summary>
        /// <param name="log">BepInEx logger for diagnostics.</param>
        /// <param name="packsDirectory">Absolute path to the packs directory on disk.</param>
        public MainMenuThemer(ManualLogSource log, string packsDirectory)
        {
            _log = log ?? throw new ArgumentNullException(nameof(log));
            _packsDirectory = packsDirectory ?? throw new ArgumentNullException(nameof(packsDirectory));
        }

        /// <summary>
        /// Called on scene change to reset the applied state so the theme can be re-applied
        /// to the new scene's main menu canvas.
        /// </summary>
        public void OnSceneChanged()
        {
            _applied = false;
        }

        /// <summary>
        /// Finds the total_conversion pack with a <c>ui_theme:</c> section in its pack.yaml
        /// and applies the theme to the MainMenu canvas.
        /// </summary>
        /// <param name="packs">Currently loaded pack display infos.</param>
        public void TryApplyTheme(IReadOnlyList<PackDisplayInfo> packs)
        {
            if (_applied) return;
            if (packs == null || packs.Count == 0) return;

            try
            {
                ThemeData? theme = FindThemeFromPacks(packs);
                if (theme == null)
                {
                    _log.LogInfo("[MainMenuThemer] No total_conversion pack with ui_theme found.");
                    return;
                }

                _log.LogInfo($"[MainMenuThemer] Found theme from pack '{theme.PackId}': title='{theme.Title}', subtitle='{theme.Subtitle}'");
                DebugLog.Write("MainMenuThemer", $"Theme resolved: pack={theme.PackId} title={theme.Title}");

                Canvas? mainMenuCanvas = FindMainMenuCanvas();
                if (mainMenuCanvas == null)
                {
                    _log.LogInfo("[MainMenuThemer] MainMenu canvas not found — will retry.");
                    return;
                }

                _log.LogInfo($"[MainMenuThemer] Found MainMenu canvas: '{mainMenuCanvas.name}'");
                DumpHierarchy(mainMenuCanvas.transform, 0, 5);

                ApplyThemeToCanvas(mainMenuCanvas, theme);
                _applied = true;
                _log.LogInfo("[MainMenuThemer] Theme applied successfully.");
                DebugLog.Write("MainMenuThemer", "Theme applied successfully.");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[MainMenuThemer] TryApplyTheme failed: {ex}");
                DebugLog.Write("MainMenuThemer", $"TryApplyTheme exception: {ex.Message}");
            }
        }

        // ── Theme data extraction ────────────────────────────────────────────────

        private ThemeData? FindThemeFromPacks(IReadOnlyList<PackDisplayInfo> packs)
        {
            // First pass: prefer total_conversion packs with ui_theme
            for (int i = 0; i < packs.Count; i++)
            {
                PackDisplayInfo pack = packs[i];
                if (!pack.IsEnabled) continue;
                if (string.Compare(pack.Type, "total_conversion", StringComparison.OrdinalIgnoreCase) != 0) continue;

                ThemeData? theme = TryReadThemeFromPackYaml(pack.Id);
                if (theme != null) return theme;
            }

            // Second pass: any enabled pack with ui_theme
            for (int i = 0; i < packs.Count; i++)
            {
                PackDisplayInfo pack = packs[i];
                if (!pack.IsEnabled) continue;

                ThemeData? theme = TryReadThemeFromPackYaml(pack.Id);
                if (theme != null) return theme;
            }

            return null;
        }

        private ThemeData? TryReadThemeFromPackYaml(string packId)
        {
            try
            {
                string packYamlPath = Path.Combine(_packsDirectory, packId, "pack.yaml");
                if (!File.Exists(packYamlPath))
                {
                    packYamlPath = Path.Combine(_packsDirectory, packId, "pack.yml");
                    if (!File.Exists(packYamlPath)) return null;
                }

                // Pattern #106: explicit UTF-8 encoding
                string content = File.ReadAllText(packYamlPath, Encoding.UTF8);

                // Lightweight string parsing — find ui_theme: block
                int themeIndex = content.IndexOf("ui_theme:", StringComparison.OrdinalIgnoreCase);
                if (themeIndex < 0) return null;

                string themeBlock = content.Substring(themeIndex);

                ThemeData theme = new ThemeData { PackId = packId };
                theme.Title = ExtractYamlValue(themeBlock, "title");
                theme.Subtitle = ExtractYamlValue(themeBlock, "subtitle");
                theme.PrimaryColor = ParseColorOrDefault(ExtractYamlValue(themeBlock, "primary_color"), new Color(0.3f, 0.6f, 1f));
                theme.SecondaryColor = ParseColorOrDefault(ExtractYamlValue(themeBlock, "secondary_color"), new Color(0.2f, 0.4f, 0.8f));
                theme.AccentColor = ParseColorOrDefault(ExtractYamlValue(themeBlock, "accent_color"), new Color(1f, 0.84f, 0f));
                theme.TextColor = ParseColorOrDefault(ExtractYamlValue(themeBlock, "text_color"), Color.white);
                theme.BackgroundTint = ParseColorOrDefault(ExtractYamlValue(themeBlock, "background_tint"), new Color(0.05f, 0.05f, 0.15f));

                return theme;
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[MainMenuThemer] Failed to read pack.yaml for '{packId}': {ex}");
                return null;
            }
        }

        /// <summary>
        /// Extracts a simple YAML value for a given key from a block of text.
        /// Handles: <c>key: value</c> and <c>key: "value"</c>.
        /// Returns empty string if not found.
        /// </summary>
        private static string ExtractYamlValue(string block, string key)
        {
            string searchKey = key + ":";
            int keyIndex = -1;
            int searchFrom = 0;
            while (searchFrom < block.Length)
            {
                int idx = block.IndexOf(searchKey, searchFrom, StringComparison.OrdinalIgnoreCase);
                if (idx < 0) break;

                // Ensure we're at line start or after whitespace (not a substring of another key)
                if (idx > 0)
                {
                    char before = block[idx - 1];
                    if (before != '\n' && before != '\r' && before != ' ' && before != '\t')
                    {
                        searchFrom = idx + searchKey.Length;
                        continue;
                    }
                }

                keyIndex = idx;
                break;
            }

            if (keyIndex < 0) return string.Empty;

            int valueStart = keyIndex + searchKey.Length;
            int lineEnd = block.IndexOf('\n', valueStart);
            if (lineEnd < 0) lineEnd = block.Length;

            string raw = block.Substring(valueStart, lineEnd - valueStart).Trim();

            // Strip surrounding quotes
            if (raw.Length >= 2 && ((raw[0] == '"' && raw[raw.Length - 1] == '"') || (raw[0] == '\'' && raw[raw.Length - 1] == '\'')))
            {
                raw = raw.Substring(1, raw.Length - 2);
            }

            return raw;
        }

        private static Color ParseColorOrDefault(string hex, Color defaultColor)
        {
            if (string.IsNullOrEmpty(hex)) return defaultColor;
            // Ensure hex starts with #
            if (hex[0] != '#') hex = "#" + hex;
            if (ColorUtility.TryParseHtmlString(hex, out Color c))
                return c;
            return defaultColor;
        }

        // ── Canvas discovery ─────────────────────────────────────────────────────

        private Canvas? FindMainMenuCanvas()
        {
            Canvas[] allCanvases = Resources.FindObjectsOfTypeAll<Canvas>();
            foreach (Canvas canvas in allCanvases)
            {
                if (canvas == null) continue;
                if (canvas.name != null && canvas.name.IndexOf("MainMenu", StringComparison.OrdinalIgnoreCase) >= 0
                    && canvas.gameObject.activeInHierarchy)
                {
                    return canvas;
                }
            }
            return null;
        }

        // ── Hierarchy dump ───────────────────────────────────────────────────────

        private void DumpHierarchy(Transform root, int depth, int maxDepth)
        {
            if (depth > maxDepth || root == null) return;

            string indent = new string(' ', depth * 2);
            StringBuilder info = new StringBuilder(256);
            info.Append(indent).Append(root.name);

            // Note component types
            Component[] components = root.GetComponents<Component>();
            if (components.Length > 0)
            {
                info.Append(" [");
                for (int i = 0; i < components.Length; i++)
                {
                    if (i > 0) info.Append(", ");
                    info.Append(components[i] != null ? components[i].GetType().Name : "null");
                }
                info.Append("]");
            }

            _log.LogInfo($"[MainMenuThemer:Hierarchy] {info}");

            for (int i = 0; i < root.childCount; i++)
            {
                DumpHierarchy(root.GetChild(i), depth + 1, maxDepth);
            }
        }

        // ── Theme application ────────────────────────────────────────────────────

        private void ApplyThemeToCanvas(Canvas canvas, ThemeData theme)
        {
            Transform root = canvas.transform;

            // 1. Replace game title text (TMP_Text via reflection, priority 1; legacy Text fallback)
            if (!string.IsNullOrEmpty(theme.Title))
            {
                ReplaceTitle(root, theme);
            }

            // 2. Tint the largest background Image with background_tint at 0.85 alpha
            TintLargestBackground(root, theme.BackgroundTint);

            // 3. Restyle ALL Selectable components with theme colors
            RestyleSelectables(root, theme);

            // 4. Rewrite button labels
            RewriteButtonLabels(root);
        }

        private void ReplaceTitle(Transform root, ThemeData theme)
        {
            // Try TMP_Text first via reflection
            Type? tmpType = Type.GetType("TMPro.TextMeshProUGUI, Unity.TextMeshPro")
                         ?? Type.GetType("TMPro.TextMeshProUGUI, Assembly-CSharp");

            if (tmpType != null)
            {
                Component[] tmpTexts = root.GetComponentsInChildren(tmpType, true);
                if (tmpTexts.Length > 0)
                {
                    // Find the largest text component (likely the title)
                    Component? bestCandidate = null;
                    float bestSize = 0f;

                    System.Reflection.PropertyInfo? fontSizeProp = tmpType.GetProperty("fontSize");
                    System.Reflection.PropertyInfo? textProp = tmpType.GetProperty("text");
                    System.Reflection.PropertyInfo? colorProp = tmpType.GetProperty("color");

                    foreach (Component tmp in tmpTexts)
                    {
                        if (tmp == null) continue;
                        object? sizeObj = fontSizeProp?.GetValue(tmp);
                        float size = sizeObj is float f ? f : 0f;
                        if (size > bestSize)
                        {
                            bestSize = size;
                            bestCandidate = tmp;
                        }
                    }

                    if (bestCandidate != null && textProp != null)
                    {
                        textProp.SetValue(bestCandidate, theme.Title);
                        colorProp?.SetValue(bestCandidate, theme.TextColor);
                        _log.LogInfo($"[MainMenuThemer] Set TMP title to '{theme.Title}' (fontSize={bestSize})");

                        // Apply subtitle to the second-largest if available
                        if (!string.IsNullOrEmpty(theme.Subtitle) && tmpTexts.Length > 1)
                        {
                            Component? secondBest = null;
                            float secondBestSize = 0f;
                            foreach (Component tmp in tmpTexts)
                            {
                                if (tmp == null || tmp == bestCandidate) continue;
                                object? sizeObj = fontSizeProp?.GetValue(tmp);
                                float size = sizeObj is float f2 ? f2 : 0f;
                                if (size > secondBestSize)
                                {
                                    secondBestSize = size;
                                    secondBest = tmp;
                                }
                            }
                            if (secondBest != null)
                            {
                                textProp.SetValue(secondBest, theme.Subtitle);
                                colorProp?.SetValue(secondBest, theme.TextColor);
                                _log.LogInfo($"[MainMenuThemer] Set TMP subtitle to '{theme.Subtitle}'");
                            }
                        }
                        return;
                    }
                }
            }

            // Fallback: legacy UnityEngine.UI.Text
            Text[] texts = root.GetComponentsInChildren<Text>(true);
            if (texts.Length == 0) return;

            Text? largestText = null;
            int largestFontSize = 0;
            foreach (Text t in texts)
            {
                if (t == null) continue;
                if (t.fontSize > largestFontSize)
                {
                    largestFontSize = t.fontSize;
                    largestText = t;
                }
            }

            if (largestText != null)
            {
                largestText.text = theme.Title;
                largestText.color = theme.TextColor;
                _log.LogInfo($"[MainMenuThemer] Set legacy Text title to '{theme.Title}' (fontSize={largestFontSize})");

                if (!string.IsNullOrEmpty(theme.Subtitle) && texts.Length > 1)
                {
                    Text? secondLargest = null;
                    int secondLargestSize = 0;
                    foreach (Text t in texts)
                    {
                        if (t == null || t == largestText) continue;
                        if (t.fontSize > secondLargestSize)
                        {
                            secondLargestSize = t.fontSize;
                            secondLargest = t;
                        }
                    }
                    if (secondLargest != null)
                    {
                        secondLargest.text = theme.Subtitle;
                        secondLargest.color = theme.TextColor;
                        _log.LogInfo($"[MainMenuThemer] Set legacy Text subtitle to '{theme.Subtitle}'");
                    }
                }
            }
        }

        private void TintLargestBackground(Transform root, Color tint)
        {
            Image[] images = root.GetComponentsInChildren<Image>(true);
            if (images.Length == 0) return;

            Image? largest = null;
            float largestArea = 0f;

            foreach (Image img in images)
            {
                if (img == null) continue;
                RectTransform? rt = img.GetComponent<RectTransform>();
                if (rt == null) continue;
                float area = rt.rect.width * rt.rect.height;
                if (area > largestArea)
                {
                    largestArea = area;
                    largest = img;
                }
            }

            if (largest != null)
            {
                Color applied = tint;
                applied.a = 0.85f;
                largest.color = applied;
                _log.LogInfo($"[MainMenuThemer] Tinted largest background '{largest.name}' (area={largestArea:F0}) with {ColorUtility.ToHtmlStringRGBA(applied)}");
            }
        }

        private void RestyleSelectables(Transform root, ThemeData theme)
        {
            // Restyle ALL Selectable components — DINO uses MainMenuButton:Selectable, not just Button
            Selectable[] selectables = root.GetComponentsInChildren<Selectable>(true);
            int count = 0;

            foreach (Selectable sel in selectables)
            {
                if (sel == null) continue;

                ColorBlock cb = sel.colors;
                cb.normalColor = theme.PrimaryColor;
                cb.highlightedColor = theme.AccentColor;
                cb.pressedColor = theme.SecondaryColor;
                cb.selectedColor = theme.AccentColor;
                cb.disabledColor = new Color(theme.PrimaryColor.r, theme.PrimaryColor.g, theme.PrimaryColor.b, 0.4f);
                cb.colorMultiplier = 1f;
                cb.fadeDuration = 0.1f;
                sel.colors = cb;
                count++;
            }

            _log.LogInfo($"[MainMenuThemer] Restyled {count} Selectable components with theme colors.");
        }

#pragma warning disable DF1006 // static readonly Dictionary — not a lifecycle-managed disposable
        private static readonly Dictionary<string, string> ButtonLabelMap = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
        {
            { "New Game", "New Campaign" },
            { "Continue", "Resume Campaign" },
            { "Load Game", "Load Campaign" },
            { "Special Missions", "Clone Wars Missions" },
        };
#pragma warning restore DF1006

        private void RewriteButtonLabels(Transform root)
        {
            int rewritten = 0;

            // Legacy Text labels
            Text[] texts = root.GetComponentsInChildren<Text>(true);
            foreach (Text t in texts)
            {
                if (t == null || string.IsNullOrEmpty(t.text)) continue;
                if (ButtonLabelMap.TryGetValue(t.text.Trim(), out string? replacement))
                {
                    _log.LogInfo($"[MainMenuThemer] Rewriting label '{t.text}' -> '{replacement}'");
                    t.text = replacement;
                    rewritten++;
                }
            }

            // TMP_Text labels via reflection
            Type? tmpType = Type.GetType("TMPro.TMP_Text, Unity.TextMeshPro")
                         ?? Type.GetType("TMPro.TMP_Text, Assembly-CSharp");
            if (tmpType != null)
            {
                System.Reflection.PropertyInfo? textProp = tmpType.GetProperty("text");
                if (textProp != null)
                {
                    Component[] tmpTexts = root.GetComponentsInChildren(tmpType, true);
                    foreach (Component tmp in tmpTexts)
                    {
                        if (tmp == null) continue;
                        string? currentText = textProp.GetValue(tmp) as string;
                        if (string.IsNullOrEmpty(currentText)) continue;
                        if (ButtonLabelMap.TryGetValue(currentText!.Trim(), out string? replacement2) && replacement2 != null)
                        {
                            _log.LogInfo($"[MainMenuThemer] Rewriting TMP label '{currentText}' -> '{replacement2}'");
                            textProp.SetValue(tmp, replacement2!);
                            rewritten++;
                        }
                    }
                }
            }

            _log.LogInfo($"[MainMenuThemer] Rewrote {rewritten} button labels.");
        }

        // ── Inner types ──────────────────────────────────────────────────────────

        private sealed class ThemeData
        {
            public string PackId = string.Empty;
            public string Title = string.Empty;
            public string Subtitle = string.Empty;
            public Color PrimaryColor;
            public Color SecondaryColor;
            public Color AccentColor;
            public Color TextColor;
            public Color BackgroundTint;
        }
    }
}
