#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using BepInEx.Logging;
using DINOForge.Runtime.Diagnostics;
using UnityEngine;
using UnityEngine.UI;

namespace DINOForge.Runtime.UI
{
    internal sealed class MainMenuThemer
    {
        private readonly ManualLogSource _log;
        private readonly string _packsDirectory;
        private bool _applied;

        internal MainMenuThemer(ManualLogSource log, string packsDirectory)
        {
            _log = log;
            _packsDirectory = packsDirectory;
        }

        internal bool IsApplied => _applied;

        internal void TryApplyTheme(IReadOnlyList<PackDisplayInfo> packs)
        {
            if (_applied) return;

            // Find the best total_conversion pack: prefer one with ui_theme on disk
            PackDisplayInfo? tcPack = null;
            PackDisplayInfo? tcFallback = null;
            foreach (var p in packs)
            {
                if (!string.Equals(p.Type, "total_conversion", StringComparison.OrdinalIgnoreCase))
                    continue;

                string manifestPath = Path.Combine(_packsDirectory, p.Id, "pack.yaml");
                if (File.Exists(manifestPath))
                {
                    string content = File.ReadAllText(manifestPath, System.Text.Encoding.UTF8);
                    if (content.IndexOf("ui_theme:", StringComparison.Ordinal) >= 0)
                    {
                        tcPack = p;
                        break;
                    }
                }

                if (tcFallback == null) tcFallback = p;
            }

            if (tcPack == null) tcPack = tcFallback;

            if (tcPack == null)
            {
                DebugLog.Write("MainMenuThemer", "No total_conversion pack found among loaded packs.");
                return;
            }

            _log.LogInfo($"[MainMenuThemer] Found total_conversion pack: '{tcPack.Name}' ({tcPack.Id})");

            var theme = ReadThemeFromDisk(tcPack.Id);
            if (theme == null)
            {
                _log.LogInfo("[MainMenuThemer] No ui_theme section in pack.yaml — using pack name as fallback.");
                theme = new ThemeData
                {
                    Title = tcPack.Name,
                    Subtitle = tcPack.Description ?? "",
                    PrimaryColor = "#4ECDC4",
                    SecondaryColor = "#2C3E50",
                    TextColor = "#FFFFFF"
                };
            }

            ApplyToMainMenu(theme, tcPack);
        }

        private ThemeData? ReadThemeFromDisk(string packId)
        {
            try
            {
                string manifestPath = Path.Combine(_packsDirectory, packId, "pack.yaml");
                if (!File.Exists(manifestPath))
                {
                    _log.LogWarning($"[MainMenuThemer] pack.yaml not found at {manifestPath}");
                    return null;
                }

                string yaml = File.ReadAllText(manifestPath, System.Text.Encoding.UTF8);

                // Lightweight YAML parsing — extract ui_theme block without pulling in YamlDotNet
                // (Runtime targets netstandard2.0 and YamlDotNet may not be available at this layer).
                int themeIdx = yaml.IndexOf("ui_theme:", StringComparison.Ordinal);
                if (themeIdx < 0) return null;

                var theme = new ThemeData();
                theme.Title = ExtractYamlValue(yaml, themeIdx, "title");
                theme.Subtitle = ExtractYamlValue(yaml, themeIdx, "subtitle");
                theme.PrimaryColor = ExtractYamlValue(yaml, themeIdx, "primary_color") ?? "#4ECDC4";
                theme.SecondaryColor = ExtractYamlValue(yaml, themeIdx, "secondary_color") ?? "#2C3E50";
                theme.AccentColor = ExtractYamlValue(yaml, themeIdx, "accent_color") ?? "#E74C3C";
                theme.TextColor = ExtractYamlValue(yaml, themeIdx, "text_color") ?? "#FFFFFF";
                theme.BackgroundTint = ExtractYamlValue(yaml, themeIdx, "background_tint");

                _log.LogInfo($"[MainMenuThemer] Parsed ui_theme: title='{theme.Title}' primary={theme.PrimaryColor}");
                return theme;
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[MainMenuThemer] Failed to read theme from disk: {ex.Message}");
                return null;
            }
        }

        private static string? ExtractYamlValue(string yaml, int blockStart, string key)
        {
            string searchKey = key + ":";
            int keyIdx = yaml.IndexOf(searchKey, blockStart, StringComparison.Ordinal);
            if (keyIdx < 0) return null;

            // Don't read past the next top-level key (a line starting without indentation)
            int nextTopLevel = -1;
            int searchFrom = blockStart + "ui_theme:".Length;
            for (int i = searchFrom; i < yaml.Length; i++)
            {
                if (i > 0 && yaml[i - 1] == '\n' && i < yaml.Length && yaml[i] != ' ' && yaml[i] != '\r' && yaml[i] != '\n')
                {
                    nextTopLevel = i;
                    break;
                }
            }

            if (nextTopLevel >= 0 && keyIdx >= nextTopLevel) return null;

            int valueStart = keyIdx + searchKey.Length;
            int lineEnd = yaml.IndexOf('\n', valueStart);
            if (lineEnd < 0) lineEnd = yaml.Length;

            string raw = yaml.Substring(valueStart, lineEnd - valueStart).Trim();
            if (raw.Length >= 2 && raw[0] == '"' && raw[raw.Length - 1] == '"')
                raw = raw.Substring(1, raw.Length - 2);
            if (raw.Length >= 2 && raw[0] == '\'' && raw[raw.Length - 1] == '\'')
                raw = raw.Substring(1, raw.Length - 2);

            return string.IsNullOrEmpty(raw) ? null : raw;
        }

        private void ApplyToMainMenu(ThemeData theme, PackDisplayInfo pack)
        {
            try
            {
                ColorUtility.TryParseHtmlString(theme.PrimaryColor, out Color primaryColor);
                ColorUtility.TryParseHtmlString(theme.SecondaryColor, out Color secondaryColor);
                ColorUtility.TryParseHtmlString(theme.TextColor, out Color textColor);
                ColorUtility.TryParseHtmlString(theme.AccentColor ?? "#E74C3C", out Color accentColor);

                Canvas? menuCanvas = FindMainMenuCanvas();
                if (menuCanvas == null)
                {
                    _log.LogWarning("[MainMenuThemer] Could not find MainMenu canvas — will retry on next scene change.");
                    return;
                }

                _log.LogInfo($"[MainMenuThemer] Applying theme to canvas '{menuCanvas.name}'");

                // Dump hierarchy for diagnostics
                DumpCanvasHierarchy(menuCanvas);

                // 1. Replace the game title/logo text with mod title
                ReplaceGameTitle(menuCanvas, theme, primaryColor);

                // 2. Tint the background image
                TintBackgroundImage(menuCanvas, theme, secondaryColor);

                // 3. Restyle all menu buttons in-place
                RestyleMenuButtons(menuCanvas, primaryColor, secondaryColor, textColor, accentColor);

                // 4. Rewrite button labels for total conversion flavor
                RewriteButtonLabels(menuCanvas, theme, textColor);

                _applied = true;
                _log.LogInfo($"[MainMenuThemer] Theme applied: '{theme.Title}' on canvas '{menuCanvas.name}'");
                DebugLog.Write("MainMenuThemer", $"Theme applied: title='{theme.Title}' primary={theme.PrimaryColor} canvas='{menuCanvas.name}'");
            }
            catch (Exception ex)
            {
                _log.LogError($"[MainMenuThemer] ApplyToMainMenu failed: {ex}");
            }
        }

        private void DumpCanvasHierarchy(Canvas canvas)
        {
            var sb = new System.Text.StringBuilder(4096);
            sb.AppendLine("[MainMenuThemer] Canvas hierarchy dump:");
            DumpTransform(canvas.transform, sb, 0, 3);
            _log.LogInfo(sb.ToString());
        }

        private void DumpTransform(Transform t, System.Text.StringBuilder sb, int depth, int maxDepth)
        {
            if (depth > maxDepth) return;
            string indent = new string(' ', depth * 2);
            string components = "";

            var img = t.GetComponent<Image>();
            if (img != null) components += $" [Image sprite={(img.sprite != null ? img.sprite.name : "null")} color={img.color}]";

            var txt = t.GetComponent<Text>();
            if (txt != null) components += $" [Text='{(txt.text?.Length > 30 ? txt.text.Substring(0, 30) + "..." : txt.text)}']";

            // Check for TMP_Text via reflection (may not be available)
            try
            {
                var tmpType = t.GetComponent("TMPro.TMP_Text");
                if (tmpType != null)
                {
                    var textProp = tmpType.GetType().GetProperty("text");
                    string? tmpText = textProp?.GetValue(tmpType) as string;
                    if (tmpText != null) components += $" [TMP='{(tmpText.Length > 30 ? tmpText.Substring(0, 30) + "..." : tmpText)}']";
                }
            }
            catch { /* safe-swallow: TMP reflection is diagnostic only */ }

            sb.AppendLine($"{indent}{t.name}{components}");

            for (int i = 0; i < t.childCount; i++)
                DumpTransform(t.GetChild(i), sb, depth + 1, maxDepth);
        }

        private Canvas? FindMainMenuCanvas()
        {
            Canvas[] canvases = UnityEngine.Object.FindObjectsOfType<Canvas>();
            foreach (Canvas c in canvases)
            {
                if (c == null || !c.gameObject.activeInHierarchy) continue;
                if (c.name.Contains("MainMenu") || c.name.Contains("mainmenu"))
                    return c;
            }

            // Fallback: find any canvas with buttons (likely the menu)
            foreach (Canvas c in canvases)
            {
                if (c == null || !c.gameObject.activeInHierarchy) continue;
                if (c.name.Contains("DINOForge") || c.name.Contains("DFCanvas")) continue;
                Button[] buttons = c.GetComponentsInChildren<Button>(false);
                if (buttons.Length >= 3) return c;
            }

            return null;
        }

        private void ReplaceGameTitle(Canvas canvas, ThemeData theme, Color primaryColor)
        {
            if (string.IsNullOrEmpty(theme.Title)) return;

            // Walk all Text and TMP_Text in the canvas looking for the game title
            // DINO's main menu typically has the game title as a large Text or Image element
            Text[] texts = canvas.GetComponentsInChildren<Text>(true);
            int replaced = 0;
            foreach (Text t in texts)
            {
                if (t == null || t.text == null) continue;
                string lower = t.text.ToLowerInvariant();
                // Match the game title or any prominent heading text
                if (lower.Contains("diplomacy") || lower.Contains("not an option") || lower.Contains("dino"))
                {
                    _log.LogInfo($"[MainMenuThemer] Replacing title text '{t.text}' → '{theme.Title}'");
                    t.text = theme.Title;
                    t.color = primaryColor;
                    t.fontStyle = FontStyle.Bold;
                    replaced++;
                }
            }

            // Also try TMP_Text via reflection
            try
            {
                var allTmp = canvas.GetComponentsInChildren<Component>(true);
                foreach (var c in allTmp)
                {
                    if (c == null) continue;
                    var cType = c.GetType();
                    if (cType.FullName != "TMPro.TMP_Text" && !cType.FullName.StartsWith("TMPro.TextMeshPro")) continue;

                    var textProp = cType.GetProperty("text");
                    if (textProp == null) continue;
                    string? tmpText = textProp.GetValue(c) as string;
                    if (tmpText == null) continue;

                    string lower = tmpText.ToLowerInvariant();
                    if (lower.Contains("diplomacy") || lower.Contains("not an option") || lower.Contains("dino"))
                    {
                        _log.LogInfo($"[MainMenuThemer] Replacing TMP title '{tmpText}' → '{theme.Title}'");
                        textProp.SetValue(c, theme.Title);

                        var colorProp = cType.GetProperty("color");
                        colorProp?.SetValue(c, primaryColor);
                        replaced++;
                    }
                }
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[MainMenuThemer] TMP title replacement failed: {ex.Message}"); // pattern-96-ok: diagnostic only
            }

            _log.LogInfo($"[MainMenuThemer] Title replacement: {replaced} element(s) updated.");
        }

        private void TintBackgroundImage(Canvas canvas, ThemeData theme, Color secondaryColor)
        {
            // Find the largest Image in the canvas — likely the background
            Image[] images = canvas.GetComponentsInChildren<Image>(true);
            Image? largestImg = null;
            float largestArea = 0;

            foreach (Image img in images)
            {
                if (img == null) continue;
                // Skip DINOForge elements
                if (img.gameObject.name.Contains("DINOForge") || img.gameObject.name.Contains("DFCanvas")) continue;

                RectTransform rt = img.GetComponent<RectTransform>();
                if (rt == null) continue;
                float area = rt.rect.width * rt.rect.height;
                if (area > largestArea)
                {
                    largestArea = area;
                    largestImg = img;
                }
            }

            if (largestImg != null)
            {
                Color bgTint = theme.BackgroundTint != null && ColorUtility.TryParseHtmlString(theme.BackgroundTint, out Color parsed)
                    ? new Color(parsed.r, parsed.g, parsed.b, 0.6f)
                    : new Color(secondaryColor.r, secondaryColor.g, secondaryColor.b, 0.5f);

                _log.LogInfo($"[MainMenuThemer] Tinting background '{largestImg.name}' (area={largestArea:F0})");
                largestImg.color = bgTint;
            }
        }

        private void RestyleMenuButtons(Canvas canvas, Color primaryColor, Color secondaryColor, Color textColor, Color accentColor)
        {
            Button[] buttons = canvas.GetComponentsInChildren<Button>(false);
            int styled = 0;

            foreach (Button btn in buttons)
            {
                if (btn == null || btn.gameObject == null) continue;
                if (btn.gameObject.name.Contains("DINOForge") || btn.gameObject.name.Contains("Mods_Button")) continue;

                try
                {
                    // Restyle the button's color block
                    var colors = btn.colors;
                    colors.normalColor = new Color(secondaryColor.r, secondaryColor.g, secondaryColor.b, 0.9f);
                    colors.highlightedColor = new Color(primaryColor.r, primaryColor.g, primaryColor.b, 0.85f);
                    colors.pressedColor = new Color(accentColor.r, accentColor.g, accentColor.b, 1f);
                    colors.selectedColor = new Color(primaryColor.r, primaryColor.g, primaryColor.b, 0.7f);
                    btn.colors = colors;

                    // Tint child Image (button background)
                    Image? btnImg = btn.GetComponent<Image>();
                    if (btnImg != null)
                    {
                        btnImg.color = new Color(secondaryColor.r, secondaryColor.g, secondaryColor.b, btnImg.color.a);
                    }

                    // Restyle button text
                    Text? btnText = btn.GetComponentInChildren<Text>();
                    if (btnText != null)
                    {
                        btnText.color = textColor;
                    }

                    // Also try TMP_Text
                    try
                    {
                        var tmpComponents = btn.GetComponentsInChildren<Component>(false);
                        foreach (var c in tmpComponents)
                        {
                            if (c == null) continue;
                            if (!c.GetType().FullName.StartsWith("TMPro.")) continue;
                            var colorProp = c.GetType().GetProperty("color");
                            colorProp?.SetValue(c, textColor);
                        }
                    }
                    catch { /* safe-swallow: TMP text styling is best-effort */ }

                    styled++;
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"[MainMenuThemer] Failed to restyle button '{btn.name}': {ex.Message}"); // pattern-96-ok: diagnostic only
                }
            }

            _log.LogInfo($"[MainMenuThemer] Restyled {styled} menu buttons.");
        }

        private void RewriteButtonLabels(Canvas canvas, ThemeData theme, Color textColor)
        {
            // Map vanilla labels to themed equivalents
            var labelMap = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
            {
                { "New Game", "New Campaign" },
                { "Continue", "Resume Campaign" },
                { "Load Game", "Load Campaign" },
                { "Special Missions", "Clone Wars Missions" },
            };

            Text[] texts = canvas.GetComponentsInChildren<Text>(true);
            int rewritten = 0;
            foreach (Text t in texts)
            {
                if (t == null || t.text == null) continue;
                foreach (var kv in labelMap)
                {
                    if (string.Equals(t.text.Trim(), kv.Key, StringComparison.OrdinalIgnoreCase))
                    {
                        _log.LogInfo($"[MainMenuThemer] Rewriting label '{t.text}' → '{kv.Value}'");
                        t.text = kv.Value;
                        t.color = textColor;
                        rewritten++;
                        break;
                    }
                }
            }

            // TMP_Text variant
            try
            {
                var allComponents = canvas.GetComponentsInChildren<Component>(true);
                foreach (var c in allComponents)
                {
                    if (c == null) continue;
                    var cType = c.GetType();
                    if (!cType.FullName.StartsWith("TMPro.")) continue;
                    var textProp = cType.GetProperty("text");
                    if (textProp == null) continue;
                    string? tmpText = textProp.GetValue(c) as string;
                    if (tmpText == null) continue;

                    foreach (var kv in labelMap)
                    {
                        if (string.Equals(tmpText.Trim(), kv.Key, StringComparison.OrdinalIgnoreCase))
                        {
                            _log.LogInfo($"[MainMenuThemer] Rewriting TMP label '{tmpText}' → '{kv.Value}'");
                            textProp.SetValue(c, kv.Value);
                            var colorProp = cType.GetProperty("color");
                            colorProp?.SetValue(c, textColor);
                            rewritten++;
                            break;
                        }
                    }
                }
            }
            catch { /* safe-swallow: TMP label rewrite is best-effort */ }

            _log.LogInfo($"[MainMenuThemer] Rewrote {rewritten} button labels.");
        }

        internal void OnSceneChanged()
        {
            _applied = false;
        }

        internal sealed class ThemeData
        {
            internal string? Title;
            internal string? Subtitle;
            internal string PrimaryColor = "#4ECDC4";
            internal string SecondaryColor = "#2C3E50";
            internal string? AccentColor = "#E74C3C";
            internal string TextColor = "#FFFFFF";
            internal string? BackgroundTint;
        }
    }
}
