#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Coverage tests for <see cref="UiStyleSnapshot"/>.
/// </summary>
public sealed class UiStyleSnapshotCoverageTests
{
    /// <summary>
    /// Verifies the DTO constructor initializes the documented defaults.
    /// </summary>
    [Fact]
    public void Construction_InitializesDefaultValues()
    {
        UiStyleSnapshot snapshot = new();

        snapshot.Transition.Should().BeEmpty();
        snapshot.NormalColor.Should().BeEmpty();
        snapshot.HighlightedColor.Should().BeEmpty();
        snapshot.PressedColor.Should().BeEmpty();
        snapshot.DisabledColor.Should().BeEmpty();
        snapshot.FontSize.Should().BeNull();
        snapshot.TextColor.Should().BeNull();
        snapshot.ImageColor.Should().BeNull();
    }

    /// <summary>
    /// Verifies all properties remain writable and preserve assigned values.
    /// </summary>
    [Fact]
    public void Properties_CanBeAssigned()
    {
        UiStyleSnapshot snapshot = new()
        {
            Transition = "ColorTint",
            NormalColor = "#ffffffff",
            HighlightedColor = "#eeeeeeff",
            PressedColor = "#ccccccff",
            DisabledColor = "#888888ff",
            FontSize = 24,
            TextColor = "#11223344",
            ImageColor = "#55667788"
        };

        snapshot.Transition.Should().Be("ColorTint");
        snapshot.NormalColor.Should().Be("#ffffffff");
        snapshot.HighlightedColor.Should().Be("#eeeeeeff");
        snapshot.PressedColor.Should().Be("#ccccccff");
        snapshot.DisabledColor.Should().Be("#888888ff");
        snapshot.FontSize.Should().Be(24);
        snapshot.TextColor.Should().Be("#11223344");
        snapshot.ImageColor.Should().Be("#55667788");
    }
}
