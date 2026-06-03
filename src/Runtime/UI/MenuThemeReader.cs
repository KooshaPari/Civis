#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using UnityEngine;

namespace DINOForge.Runtime.UI
{
    /// <summary>
    /// Reads the active total_conversion pack's <c>ui_theme</c> block from disk and
    /// exposes it as resolved Unity <see cref="Color"/> values for runtime UI (the
    /// native quick-mod panel). Defaults to Star Wars gold (<c>#FFE81F</c>) so themed
    /// chrome looks intentional even before a total_conversion pack is loaded.
    ///
    /// This mirrors the YAML-extraction logic in <c>MainMenuThemer</c> but is a small,
    /// dependency-free static helper so multiple UI surfaces can resolve the same theme
    /// without a full YAML parser.
    /// </summary>
    public static class MenuThemeReader
    {
        /// <summary>Resolved theme colors for runtime menu UI.</summary>
        public readonly struct MenuTheme
        {
            public readonly Color Primary;
            public readonly Color Secondary;
            public readonly Color Accent;
            public readonly Color Text;

            public MenuTheme(Color primary, Color secondary, Color accent, Color text)
            {
                Primary = primary;
                Secondary = secondary;
                Accent = accent;
                Text = text;
            }

            /// <summary>Star Wars-leaning gold-on-black default.</summary>
            public static MenuTheme Default => new MenuTheme(
                Parse("#FFE81F", new Color(1f, 0.91f, 0.12f, 1f)),
                Parse("#0A0A12", new Color(0.04f, 0.04f, 0.07f, 1f)),
                Parse("#C0392B", new Color(0.75f, 0.22f, 0.17f, 1f)),
                Parse("#FFE81F", new Color(1f, 0.91f, 0.12f, 1f)));
        }

        /// <summary>
        /// Resolves the active theme by scanning loaded packs for the first
        /// total_conversion pack whose <c>pack.yaml</c> declares a <c>ui_theme</c> block.
        /// Falls back to <see cref="MenuTheme.Default"/> when none is found.
        /// </summary>
        /// <param name="packs">Loaded pack snapshot.</param>
        /// <param name="packsDirectory">Directory containing &lt;packId&gt;/pack.yaml.</param>
        public static MenuTheme Resolve(IReadOnlyList<PackDisplayInfo>? packs, string? packsDirectory)
        {
            if (packs == null || string.IsNullOrEmpty(packsDirectory))
                return MenuTheme.Default;

            foreach (PackDisplayInfo p in packs)
            {
                if (!string.Equals(p.Type, "total_conversion", StringComparison.OrdinalIgnoreCase))
                    continue;
                if (!p.IsEnabled) continue;

                try
                {
                    string yamlPath = Path.Combine(packsDirectory, p.Id, "pack.yaml");
                    if (!File.Exists(yamlPath)) continue;
                    string yaml = File.ReadAllText(yamlPath, System.Text.Encoding.UTF8);
                    int idx = yaml.IndexOf("ui_theme:", StringComparison.Ordinal);
                    if (idx < 0) continue;

                    Color primary = Parse(ExtractYamlValue(yaml, idx, "primary_color"), MenuTheme.Default.Primary);
                    Color secondary = Parse(ExtractYamlValue(yaml, idx, "secondary_color"), MenuTheme.Default.Secondary);
                    Color accent = Parse(ExtractYamlValue(yaml, idx, "accent_color"), MenuTheme.Default.Accent);
                    Color text = Parse(ExtractYamlValue(yaml, idx, "text_color"), MenuTheme.Default.Text);
                    return new MenuTheme(primary, secondary, accent, text);
                }
                catch
                {
                    // best-effort: theme read must never break the quick panel
                }
            }

            return MenuTheme.Default;
        }

        private static Color Parse(string? hex, Color fallback)
        {
            if (!string.IsNullOrEmpty(hex) && ColorUtility.TryParseHtmlString(hex, out Color c))
                return c;
            return fallback;
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
    }
}
