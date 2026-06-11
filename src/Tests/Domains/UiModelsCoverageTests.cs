#nullable enable
using DINOForge.Domains.UI.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for the UI-domain <c>HudElementDefinition</c> (Id + Opacity-range) and
/// <c>ThemeDefinition</c> (Id/Name + hex-color validation). First UI-domain coverage.
/// </summary>
public class UiModelsCoverageTests
{
    // --- HudElementDefinition ---

    [Fact]
    public void HudElement_ValidWithDefaultOpacity_IsValid()
    {
        HudElementDefinition hud = new() { Id = "hud:health" };

        hud.Opacity.Should().Be(1.0f); // default
        hud.Validate().IsValid.Should().BeTrue();
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void HudElement_MissingId_Fails(string id)
    {
        HudElementDefinition hud = new() { Id = id };

        hud.Validate().Errors.Should().Contain(e => e.Path == "id");
    }

    [Theory]
    [InlineData(-0.1f)]
    [InlineData(1.5f)]
    public void HudElement_OpacityOutOfRange_Fails(float opacity)
    {
        HudElementDefinition hud = new() { Id = "hud:x", Opacity = opacity };

        ValidationResult result = hud.Validate();

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Path == "opacity");
    }

    [Theory]
    [InlineData(0f)]
    [InlineData(1f)]
    [InlineData(0.5f)]
    public void HudElement_OpacityAtBounds_IsValid(float opacity)
    {
        HudElementDefinition hud = new() { Id = "hud:x", Opacity = opacity };

        hud.Validate().IsValid.Should().BeTrue();
    }

    // --- ThemeDefinition ---

    private static ThemeDefinition ValidTheme() => new()
    {
        Id = "theme:dark",
        Name = "Dark",
        PrimaryColor = "#112233",
        SecondaryColor = "#445566",
        AccentColor = "#778899"
    };

    [Fact]
    public void Theme_FullyPopulated_IsValid()
    {
        ValidTheme().Validate().IsValid.Should().BeTrue();
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void Theme_MissingId_Fails(string id)
    {
        ThemeDefinition t = ValidTheme();
        t.Id = id;

        t.Validate().Errors.Should().Contain(e => e.Path == "id");
    }

    [Fact]
    public void Theme_MissingName_Fails()
    {
        ThemeDefinition t = ValidTheme();
        t.Name = "";

        t.Validate().Errors.Should().Contain(e => e.Path == "name");
    }

    [Fact]
    public void Theme_InvalidHexPrimaryColor_Fails()
    {
        ThemeDefinition t = ValidTheme();
        t.PrimaryColor = "not-a-hex";

        ValidationResult result = t.Validate();

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Message.Contains("hex color"));
    }

    [Fact]
    public void Theme_EmptyColorField_IsSkipped()
    {
        ThemeDefinition t = ValidTheme();
        t.AccentColor = ""; // empty → color check short-circuits, no error

        t.Validate().IsValid.Should().BeTrue();
    }
}
