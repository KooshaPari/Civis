#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.UI.Extended;
using DINOForge.SDK.UI.Models;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// In-memory mock implementation of <see cref="IThemeProvider"/> for unit testing.
/// Provides configurable color and font-size resolution with call tracking.
/// NOT thread-safe; intended for single-threaded test fixtures only.
/// </summary>
public class MockThemeProvider : IThemeProvider
{
    private readonly Dictionary<string, ColorRgba> _colorTokens;
    private readonly Dictionary<string, FontSize> _fontSizeTokens;
    private ThemeDefinition? _activeTheme;

    /// <summary>
    /// Number of times <see cref="ApplyTheme"/> has been called.
    /// </summary>
    public int ApplyThemeCount { get; set; }

    /// <summary>
    /// Number of times <see cref="ResolveColor"/> has been called.
    /// </summary>
    public int ResolveColorCount { get; set; }

    /// <summary>
    /// Number of times <see cref="ResolveFontSize"/> has been called.
    /// </summary>
    public int ResolveFontSizeCount { get; set; }

    /// <summary>
    /// Gets the currently applied theme, or null if none has been applied.
    /// </summary>
    public ThemeDefinition? ActiveTheme => _activeTheme;

    /// <summary>
    /// Creates a new mock theme provider with default semantic colors and font sizes.
    /// </summary>
    public MockThemeProvider()
    {
        _colorTokens = new Dictionary<string, ColorRgba>(StringComparer.Ordinal)
        {
            { "primary", new ColorRgba(0f / 255f, 120f / 255f, 215f / 255f, 1f) },
            { "secondary", new ColorRgba(100f / 255f, 100f / 255f, 100f / 255f, 1f) },
            { "warning", new ColorRgba(1f, 165f / 255f, 0f, 1f) },
            { "error", new ColorRgba(1f, 0f, 0f, 1f) },
            { "success", new ColorRgba(0f, 200f / 255f, 0f, 1f) }
        };
        _fontSizeTokens = new Dictionary<string, FontSize>(StringComparer.Ordinal)
        {
            { "heading", FontSize.Heading },
            { "body", FontSize.Medium },
            { "small", FontSize.Small }
        };
    }

    /// <summary>
    /// Registers or overrides a semantic color token.
    /// </summary>
    /// <param name="semanticName">The semantic name (e.g., "primary", "warning").</param>
    /// <param name="color">The RGBA color value.</param>
    public void SetColor(string semanticName, ColorRgba color)
    {
        _colorTokens[semanticName] = color;
    }

    /// <summary>
    /// Registers or overrides a semantic font-size token.
    /// </summary>
    /// <param name="semanticName">The semantic name (e.g., "heading", "body").</param>
    /// <param name="fontSize">The font size definition.</param>
    public void SetFontSize(string semanticName, FontSize fontSize)
    {
        _fontSizeTokens[semanticName] = fontSize;
    }

    public void ApplyTheme(ThemeDefinition theme)
    {
        ApplyThemeCount++;
        _activeTheme = theme;
    }

    public ColorRgba ResolveColor(string semanticName)
    {
        ResolveColorCount++;
        if (_colorTokens.TryGetValue(semanticName, out var color))
            return color;
        return new ColorRgba(128f / 255f, 128f / 255f, 128f / 255f, 1f);
    }

    public FontSize ResolveFontSize(string semanticName)
    {
        ResolveFontSizeCount++;
        if (_fontSizeTokens.TryGetValue(semanticName, out var fontSize))
            return fontSize;
        return FontSize.Medium;
    }
}
