// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Phase 1 of #193 SDK split — interface-only.

using DINOForge.SDK.UI.Models;

namespace DINOForge.SDK.UI.Extended
{
    /// <summary>
    /// Resolves semantic style tokens (e.g. "primary", "warning", "heading")
    /// to concrete colors / font sizes for Extended UI rendering.
    /// </summary>
    public interface IThemeProvider
    {
        /// <summary>Apply a theme as the active resolution source.</summary>
        void ApplyTheme(ThemeDefinition theme);

        /// <summary>Resolve a semantic color token to RGBA.</summary>
        ColorRgba ResolveColor(string semanticName);

        /// <summary>Resolve a semantic font-size token to a <see cref="FontSize"/>.</summary>
        FontSize ResolveFontSize(string semanticName);
    }
}
