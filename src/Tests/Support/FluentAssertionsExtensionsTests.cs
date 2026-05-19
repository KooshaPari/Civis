using System;
using FluentAssertions;
using FluentAssertions.Execution;
using Xunit;
using Xunit.Sdk;

namespace DINOForge.Tests.Support;

/// <summary>
/// Tests for the HaveExactCount extension method.
/// Validates that the extension correctly asserts exact collection cardinality.
/// </summary>
public class FluentAssertionsExtensionsTests
{
    [Fact]
    public void HaveExactCount_WithExactMatch_Passes()
    {
        // Arrange
        var items = new[] { 1, 2, 3 };

        // Act & Assert
        items.Should().HaveExactCount(3);
    }

    [Fact]
    public void HaveExactCount_WithTooFewItems_Throws()
    {
        // Arrange
        var items = new[] { 1, 2 };

        // Act & Assert — verify the method throws XunitException when count doesn't match
        Assert.Throws<Xunit.Sdk.XunitException>(() => items.Should().HaveExactCount(3, "fixture has exactly 3 items"));
    }

    [Fact]
    public void HaveExactCount_WithTooManyItems_ShowsSample()
    {
        // Arrange
        var items = new[] { 1, 2, 3, 4, 5 };

        // Act & Assert — verify exception is thrown for mismatched count
        Assert.Throws<Xunit.Sdk.XunitException>(() => items.Should().HaveExactCount(2));
    }

    [Fact]
    public void HaveExactCount_WithEmptyCollection_IncludesCount()
    {
        // Arrange
        var items = Array.Empty<int>();

        // Act & Assert — verify exception is thrown for empty collections with positive expected count
        Assert.Throws<Xunit.Sdk.XunitException>(() => items.Should().HaveExactCount(1));
    }
}
