#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.CliTools;

/// <summary>
/// Tests for GameStatus protocol type and properties.
/// </summary>
public class StatusCommandTests
{
    [Fact]
    public void StatusCommand_ReturnsGameStatus_WithDefaultValues()
    {
        // Arrange & Act
        var status = CliTestFixtures.CreateGameStatus(running: true, entityCount: 1000);

        // Assert
        status.Running.Should().BeTrue();
        status.EntityCount.Should().Be(1000);
    }

    [Fact]
    public void StatusCommand_IncludesWorldInfo()
    {
        // Arrange & Act
        var status = CliTestFixtures.CreateGameStatus(worldName: "Default", worldReady: true);

        // Assert
        status.WorldReady.Should().BeTrue();
        status.WorldName.Should().Be("Default");
    }

    [Fact]
    public void StatusCommand_ReturnsVersion()
    {
        // Arrange & Act
        var status = CliTestFixtures.CreateGameStatus(version: "0.15.0");

        // Assert
        status.Version.Should().Be("0.15.0");
    }

    [Fact]
    public void StatusCommand_IncludesLoadedPacks()
    {
        // Arrange
        var packs = new[] { "example-balance", "warfare-modern" };

        // Act
        var status = CliTestFixtures.CreateGameStatus(loadedPacks: packs);

        // Assert
        status.LoadedPacks.Should().ContainInOrder("example-balance", "warfare-modern");
    }

    [Fact]
    public void StatusCommand_ModPlatformStatus()
    {
        // Arrange & Act
        var status = CliTestFixtures.CreateGameStatus(modPlatformReady: true);

        // Assert
        status.ModPlatformReady.Should().BeTrue();
    }

    [Fact]
    public void StatusCommand_DefaultStatusIsPopulated()
    {
        // Arrange & Act
        var status = CliTestFixtures.CreateGameStatus();

        // Assert
        status.Should().NotBeNull();
        status.Running.Should().BeTrue();
        status.Version.Should().NotBeNullOrEmpty();
        status.LoadedPacks.Should().NotBeEmpty();
    }

    [Fact]
    public void StatusCommand_CustomPropertiesOverrideDefaults()
    {
        // Arrange & Act
        var status = CliTestFixtures.CreateGameStatus(
            running: false,
            entityCount: 5000,
            version: "0.16.0");

        // Assert
        status.Running.Should().BeFalse();
        status.EntityCount.Should().Be(5000);
        status.Version.Should().Be("0.16.0");
    }

    [Fact]
    public void StatusCommand_AllPropertiesAccessible()
    {
        // Arrange
        var status = CliTestFixtures.CreateGameStatus();

        // Act & Assert
        _ = status.Running;
        _ = status.WorldReady;
        _ = status.WorldName;
        _ = status.EntityCount;
        _ = status.ModPlatformReady;
        _ = status.Version;
        _ = status.LoadedPacks;
    }

    [Fact]
    public void StatusCommand_ModifyingProperties()
    {
        // Arrange
        var status = CliTestFixtures.CreateGameStatus();

        // Act
        status.EntityCount = 2000;
        status.Running = false;

        // Assert
        status.EntityCount.Should().Be(2000);
        status.Running.Should().BeFalse();
    }
}
