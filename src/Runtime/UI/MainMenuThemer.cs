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

            PackDisplayInfo? best = null;
            PackDisplayInfo? fallback = null;
            foreach (var p in packs)
            {
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
                    BackgroundTint = ExtractYamlValue(yaml, idx, "background_tint")
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

                int titleHits = ReplaceTitle(canvas, theme.Title, primary);
                int bgHits = hasBgTint ? TintBackground(canvas, bgTint) : 0;
                int btnHits = RestyleSelectables(canvas, primary, secondary, textCol, accent);
                int labelHits = RewriteLabels(canvas, textCol);

                _applied = true;
                _log?.LogInfo($"[MainMenuThemer] Theme '{theme.Title}' from '{pack.Id}': title={titleHits}, bg={bgHits}, btn={btnHits}, label={labelHits}");
                DebugLog.Write("MainMenuThemer", $"Theme applied: '{theme.Title}' canvas='{canvas.name}'");
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
        }
    }
}
