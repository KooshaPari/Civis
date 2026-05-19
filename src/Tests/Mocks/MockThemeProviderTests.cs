#nullable enable
using DINOForge.SDK.UI.Models;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Mocks.Tests;

public class MockThemeProviderTests
{
    [Fact]
    public void ResolveColor_WithDefaultToken_ReturnsRegisteredColor()
    {
        var provider = new MockThemeProvider();

        var color = provider.ResolveColor("primary");

        color.R.Should().Be(0f / 255f);
        color.G.Should().Be(120f / 255f);
        color.B.Should().Be(215f / 255f);
        color.A.Should().Be(1f);
        provider.ResolveColorCount.Should().Be(1);
    }

    [Fact]
    public void ResolveColor_WithUnknownToken_ReturnsDefaultGray()
    {
        var provider = new MockThemeProvider();

        var color = provider.ResolveColor("unknown-color");

        color.R.Should().Be(128f / 255f);
        color.G.Should().Be(128f / 255f);
        color.B.Should().Be(128f / 255f);
        color.A.Should().Be(1f);
        provider.ResolveColorCount.Should().Be(1);
    }

    [Fact]
    public void ResolveFontSize_WithDefaultToken_ReturnsRegisteredSize()
    {
        var provider = new MockThemeProvider();

        var size = provider.ResolveFontSize("heading");

        size.Should().Be(FontSize.Heading);
        provider.ResolveFontSizeCount.Should().Be(1);
    }

    [Fact]
    public void ResolveFontSize_WithUnknownToken_ReturnsDefaultSize()
    {
        var provider = new MockThemeProvider();

        var size = provider.ResolveFontSize("unknown-size");

        size.Should().Be(FontSize.Medium);
        provider.ResolveFontSizeCount.Should().Be(1);
    }

    [Fact]
    public void SetColor_OverridesExistingToken()
    {
        var provider = new MockThemeProvider();
        var newColor = new ColorRgba(1f, 0f, 0f, 1f);

        provider.SetColor("primary", newColor);
        var result = provider.ResolveColor("primary");

        result.R.Should().Be(1f);
        result.G.Should().Be(0f);
        result.B.Should().Be(0f);
    }

    [Fact]
    public void SetFontSize_OverridesExistingToken()
    {
        var provider = new MockThemeProvider();
        var newSize = FontSize.Large;

        provider.SetFontSize("heading", newSize);
        var result = provider.ResolveFontSize("heading");

        result.Should().Be(FontSize.Large);
    }

    [Fact]
    public void ApplyTheme_SetsActiveTheme()
    {
        var provider = new MockThemeProvider();
        var theme = new ThemeDefinition { Id = "dark-theme", Name = "Dark Theme" };

        provider.ApplyTheme(theme);

        provider.ActiveTheme.Should().NotBeNull();
        provider.ActiveTheme!.Id.Should().Be("dark-theme");
        provider.ApplyThemeCount.Should().Be(1);
    }

    [Fact]
    public void ResolveColor_TracksCallCount()
    {
        var provider = new MockThemeProvider();

        provider.ResolveColor("primary");
        provider.ResolveColor("warning");
        provider.ResolveColor("error");

        provider.ResolveColorCount.Should().Be(3);
    }

    [Fact]
    public void ResolveFontSize_TracksCallCount()
    {
        var provider = new MockThemeProvider();

        provider.ResolveFontSize("heading");
        provider.ResolveFontSize("body");
        provider.ResolveFontSize("small");

        provider.ResolveFontSizeCount.Should().Be(3);
    }

    [Fact]
    public void ApplyTheme_MultipleTimes_OverwritesPrevious()
    {
        var provider = new MockThemeProvider();
        var theme1 = new ThemeDefinition { Id = "dark-theme", Name = "Dark" };
        var theme2 = new ThemeDefinition { Id = "light-theme", Name = "Light" };

        provider.ApplyTheme(theme1);
        provider.ApplyTheme(theme2);

        provider.ActiveTheme!.Id.Should().Be("light-theme");
        provider.ApplyThemeCount.Should().Be(2);
    }
}
