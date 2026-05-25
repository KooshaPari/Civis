#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.CliTools;

/// <summary>
/// Tests for edge cases and boundary conditions in protocol types.
/// </summary>
public class EdgeCaseTests
{
    [Fact]
    public void QueryResult_SingleEntity()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateQueryResultWithEntities(1);

        // Assert
        result.Count.Should().Be(1);
        result.Entities.Should().HaveCount(1);
    }

    [Fact]
    public void GameStatus_EmptyPackList()
    {
        // Arrange & Act
        var status = CliTestFixtures.CreateGameStatus(loadedPacks: []);

        // Assert
        status.LoadedPacks.Should().BeEmpty();
    }

    [Fact]
    public void OverrideResult_MaxIntCount()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateOverrideResult(
            modifiedCount: int.MaxValue);

        // Assert
        result.ModifiedCount.Should().Be(int.MaxValue);
    }

    [Fact]
    public void ResourceSnapshot_MaxIntValues()
    {
        // Arrange & Act
        var resources = CliTestFixtures.CreateResourceSnapshot(
            food: int.MaxValue,
            wood: int.MaxValue);

        // Assert
        resources.Food.Should().Be(int.MaxValue);
        resources.Wood.Should().Be(int.MaxValue);
    }

    [Fact]
    public void EntityInfo_EmptyComponentList()
    {
        // Arrange & Act
        var entity = new EntityInfo
        {
            Index = 0,
            Components = []
        };

        // Assert
        entity.Components.Should().BeEmpty();
    }

    [Fact]
    public void EntityInfo_LargeIndex()
    {
        // Arrange & Act
        var entity = new EntityInfo
        {
            Index = int.MaxValue,
            Components = ["Component"]
        };

        // Assert
        entity.Index.Should().Be(int.MaxValue);
    }

    [Fact]
    public void GameStatus_AllFalse()
    {
        // Arrange & Act
        var status = CliTestFixtures.CreateGameStatus(
            running: false,
            worldReady: false,
            modPlatformReady: false);

        // Assert
        status.Running.Should().BeFalse();
        status.WorldReady.Should().BeFalse();
        status.ModPlatformReady.Should().BeFalse();
    }

    [Fact]
    public void OverrideResult_LongMessage()
    {
        // Arrange
        var longMessage = new string('a', 1000);

        // Act
        var result = CliTestFixtures.CreateOverrideResult(message: longMessage);

        // Assert
        result.Message.Should().HaveLength(1000);
    }

    [Fact]
    public void ReloadResult_ManyErrors()
    {
        // Arrange
        var errors = Enumerable.Range(0, 100).Select(i => $"Error {i}").ToArray();

        // Act
        var result = CliTestFixtures.CreateFailedReloadResult(errors);

        // Assert
        result.Errors.Should().HaveCount(100);
    }

    [Fact]
    public void QueryResult_ZeroCount()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateQueryResult(count: 0, entities: []);

        // Assert
        result.Count.Should().Be(0);
        result.Entities.Should().HaveCount(0);
    }

    [Fact]
    public void GameStatus_VeryLongWorldName()
    {
        // Arrange
        var longName = new string('W', 500);

        // Act
        var status = CliTestFixtures.CreateGameStatus(worldName: longName);

        // Assert
        status.WorldName.Should().HaveLength(500);
    }

    [Fact]
    public void OverrideResult_NegativeModifiedCount()
    {
        // Arrange & Act
        var result = new OverrideResult
        {
            Success = false,
            SdkPath = "test",
            ModifiedCount = -1,
            Message = "Error"
        };

        // Assert
        result.ModifiedCount.Should().Be(-1);
    }

    [Fact]
    public void GameStatus_VersionString()
    {
        // Arrange & Act
        var versions = new[] { "0.1.0", "1.0.0", "0.15.0", "2.0.0-beta" };
        var results = versions.Select(v =>
            CliTestFixtures.CreateGameStatus(version: v)).ToList();

        // Assert
        results.Should().HaveCount(4);
        results[0].Version.Should().Be("0.1.0");
    }
}
