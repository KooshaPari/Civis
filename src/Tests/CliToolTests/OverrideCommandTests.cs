#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.CliTools;

/// <summary>
/// Tests for OverrideResult protocol type.
/// </summary>
public class OverrideCommandTests
{
    [Fact]
    public void OverrideCommand_SuccessfulOverride()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateOverrideResult(
            success: true,
            sdkPath: "unit.stats.hp",
            modifiedCount: 42,
            message: "Applied successfully");

        // Assert
        result.Success.Should().BeTrue();
        result.SdkPath.Should().Be("unit.stats.hp");
    }

    [Fact]
    public void OverrideCommand_ReturnsModifiedCount()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateOverrideResult(modifiedCount: 42);

        // Assert
        result.ModifiedCount.Should().Be(42);
    }

    [Fact]
    public void OverrideCommand_FailedOverride()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateOverrideResult(
            success: false,
            sdkPath: "invalid.path",
            message: "Invalid SDK path");

        // Assert
        result.Success.Should().BeFalse();
        result.Message.Should().Contain("Invalid");
    }

    [Fact]
    public void OverrideCommand_ZeroModifiedCount()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateOverrideResult(
            success: false,
            modifiedCount: 0,
            message: "No matching entities");

        // Assert
        result.ModifiedCount.Should().Be(0);
    }

    [Fact]
    public void OverrideCommand_AllPropertiesSet()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateOverrideResult();

        // Assert
        result.Success.Should().BeTrue();
        result.SdkPath.Should().NotBeNullOrEmpty();
        result.ModifiedCount.Should().BeGreaterThan(0);
        result.Message.Should().NotBeNullOrEmpty();
    }

    [Fact]
    public void OverrideCommand_DifferentPaths()
    {
        // Arrange
        var result1 = CliTestFixtures.CreateOverrideResult(sdkPath: "unit.stats.hp");
        var result2 = CliTestFixtures.CreateOverrideResult(sdkPath: "unit.stats.armor");

        // Act & Assert
        result1.SdkPath.Should().NotBe(result2.SdkPath);
    }

    [Fact]
    public void OverrideCommand_LargeModifiedCount()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateOverrideResult(modifiedCount: 10000);

        // Assert
        result.ModifiedCount.Should().Be(10000);
    }

    [Fact]
    public void OverrideCommand_CustomMessage()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateOverrideResult(
            message: "Custom error message");

        // Assert
        result.Message.Should().Be("Custom error message");
    }

    [Fact]
    public void OverrideCommand_ModifyingProperties()
    {
        // Arrange
        var result = CliTestFixtures.CreateOverrideResult();

        // Act
        result.ModifiedCount = 99;
        result.Success = false;

        // Assert
        result.ModifiedCount.Should().Be(99);
        result.Success.Should().BeFalse();
    }

    [Fact]
    public void OverrideCommand_NegativeValues()
    {
        // Arrange & Act
        var result = new OverrideResult
        {
            Success = true,
            SdkPath = "unit.stats.hp",
            ModifiedCount = -5,
            Message = "Negative count"
        };

        // Assert
        result.ModifiedCount.Should().Be(-5);
    }
}
