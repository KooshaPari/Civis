// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 2 Dispatch 8 of #193 SDK split — fourth extended-side runtime adapter (theme provider).

#nullable enable
using System;
using System.Globalization;
using DINOForge.SDK.UI.Extended;
using DINOForge.SDK.UI.Models;

namespace DINOForge.Runtime.UI.Adapters
{
    /// <summary>
    /// Runtime adapter implementing the SDK <see cref="IThemeProvider"/> contract.
    /// Resolves semantic style tokens (e.g. <c>"primary"</c>, <c>"accent"</c>, <c>"heading"</c>)
    /// to concrete <see cref="ColorRgba"/> / <see cref="FontSize"/> values, falling back to
    /// the runtime <see cref="DinoForgeStyle"/> palette defaults when no theme is active or a
    /// token is unrecognised.
    /// </summary>
    /// <remarks>
    /// Lifetime: process-wide singleton. Packs may call <see cref="ApplyTheme"/> from any
    /// thread; <see cref="ResolveColor"/> / <see cref="ResolveFontSize"/> are read-mostly and
    /// guarded by the same lock so a hot-reload swap is observed atomically.
    ///
    /// The adapter intentionally does NOT mutate <see cref="DinoForgeStyle"/> constants —
    /// those are <c>readonly</c> Unity <c>Color</c>/<c>GUIStyle</c> values cached for IMGUI
    /// rendering and live on the public surface for native UI code. ApplyTheme records the
    /// active <see cref="ThemeDefinition"/>; resolution prefers the theme, then falls back
    /// to the palette defaults baked into this file (mirroring DinoForgeStyle).
    ///
    /// JIT-defer: hex parsing and palette fallbacks are isolated in <c>Core</c> methods so
    /// the public surface stays argument-validation-only — keeps tests free of UnityEngine.
    ///
    /// Token vocabulary (case-insensitive):
    /// <list type="bullet">
    /// <item><description>Colors: <c>primary</c>, <c>secondary</c>, <c>accent</c>, <c>background</c>,
    /// <c>text</c> / <c>textPrimary</c>, <c>textMuted</c>, <c>error</c>, <c>warning</c>, <c>success</c>.</description></item>
    /// <item><description>Font sizes: <c>small</c>, <c>medium</c> / <c>body</c>, <c>large</c>, <c>heading</c>, <c>title</c>.</description></item>
    /// </list>
    /// </remarks>
    public sealed class ThemeProviderAdapter : IThemeProvider
    {
        private static ThemeProviderAdapter? _instance;

        /// <summary>Singleton accessor — the active theme is process-wide.</summary>
        public static ThemeProviderAdapter Instance => _instance ??= new ThemeProviderAdapter();

        private readonly object _lock = new object();
        private ThemeDefinition? _activeTheme;

        private ThemeProviderAdapter() { }

        /// <inheritdoc />
        public void ApplyTheme(ThemeDefinition theme)
        {
            if (theme is null) throw new ArgumentNullException(nameof(theme));
            ApplyThemeCore(theme);
        }

        /// <inheritdoc />
        public ColorRgba ResolveColor(string semanticName)
        {
            if (semanticName is null) throw new ArgumentNullException(nameof(semanticName));
            if (semanticName.Length == 0)
                throw new ArgumentException("semanticName must not be empty", nameof(semanticName));

            return ResolveColorCore(semanticName);
        }

        /// <inheritdoc />
        public FontSize ResolveFontSize(string semanticName)
        {
            if (semanticName is null) throw new ArgumentNullException(nameof(semanticName));
            if (semanticName.Length == 0)
                throw new ArgumentException("semanticName must not be empty", nameof(semanticName));

            return ResolveFontSizeCore(semanticName);
        }

        /// <summary>Returns the currently active theme, or <c>null</c> if none has been applied.</summary>
        public ThemeDefinition? GetActiveTheme()
        {
            lock (_lock) { return _activeTheme; }
        }

        // ------------------------------------------------------------------ //
        // Core methods isolate parsing/fallback so the public surface stays
        // argument-validation-only (mirrors HudElementRendererAdapter pattern).
        // ------------------------------------------------------------------ //

        private void ApplyThemeCore(ThemeDefinition theme)
        {
            lock (_lock)
            {
                _activeTheme = theme;
            }
        }

        private ColorRgba ResolveColorCore(string semanticName)
        {
            ThemeDefinition? theme;
            lock (_lock) { theme = _activeTheme; }

            // Theme overrides come first; an unparseable hex falls through to defaults.
            if (theme != null)
            {
                string? hex = SelectThemeColor(theme, semanticName);
                if (!string.IsNullOrEmpty(hex) && TryParseHex(hex!, out ColorRgba parsed))
                {
                    return parsed;
                }
            }

            return DefaultColorFor(semanticName);
        }

        private FontSize ResolveFontSizeCore(string semanticName)
        {
            // ThemeDefinition has no semantic font-size table today (Phase 1 stub); fall
            // through to the canonical mapping. When ThemeDefinition gains a font-size
            // table, this becomes a theme-first lookup like ResolveColorCore.
            return semanticName.ToLowerInvariant() switch
            {
                "small" => FontSize.Small,
                "medium" or "body" => FontSize.Medium,
                "large" => FontSize.Large,
                "heading" => FontSize.Heading,
                "title" => FontSize.Title,
                _ => FontSize.Medium,
            };
        }

        private static string? SelectThemeColor(ThemeDefinition theme, string semanticName)
        {
            return semanticName.ToLowerInvariant() switch
            {
                "primary" => theme.PrimaryColor,
                "secondary" => theme.SecondaryColor,
                "accent" => theme.AccentColor,
                "background" => theme.BackgroundColor,
                "text" or "textprimary" => theme.TextColor,
                _ => null,
            };
        }

        // Defaults mirror DinoForgeStyle palette so resolution stays consistent with the
        // native IMGUI surface. Kept out of DinoForgeStyle because that file's Color values
        // depend on UnityEngine — referencing them here would block CI compilation.
        private static ColorRgba DefaultColorFor(string semanticName)
        {
            return semanticName.ToLowerInvariant() switch
            {
                "primary" or "text" or "textprimary" => new ColorRgba(0.90f, 0.90f, 0.90f, 1f),
                "secondary" or "textmuted" => new ColorRgba(0.55f, 0.55f, 0.65f, 1f),
                "accent" => new ColorRgba(0.941f, 0.647f, 0f, 1f),
                "background" => new ColorRgba(0.102f, 0.102f, 0.180f, 0.97f),
                "error" => new ColorRgba(0.902f, 0.224f, 0.275f, 1f),
                "warning" => new ColorRgba(0.957f, 0.635f, 0.380f, 1f),
                "success" => new ColorRgba(0.165f, 0.616f, 0.561f, 1f),
                _ => ColorRgba.White,
            };
        }

        // Parses #RRGGBB / #RRGGBBAA / RRGGBB — no exceptions on malformed input; returns false.
        private static bool TryParseHex(string hex, out ColorRgba color)
        {
            color = default;
            if (string.IsNullOrEmpty(hex)) return false;

            string s = hex[0] == '#' ? hex.Substring(1) : hex;
            if (s.Length != 6 && s.Length != 8) return false;

            if (!byte.TryParse(s.Substring(0, 2), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out byte r)) return false;
            if (!byte.TryParse(s.Substring(2, 2), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out byte g)) return false;
            if (!byte.TryParse(s.Substring(4, 2), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out byte b)) return false;
            byte a = 0xFF;
            if (s.Length == 8 && !byte.TryParse(s.Substring(6, 2), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out a)) return false;

            color = new ColorRgba(r / 255f, g / 255f, b / 255f, a / 255f);
            return true;
        }
    }
}
