#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.CliTools;

/// <summary>
/// Integration tests for Bridge Protocol types used in CLI commands.
/// </summary>
public class ProtocolIntegrationTests
{
    [Fact]
    public void GameStatus_CompleteScenario()
    {
        // Arrange & Act
        var status = CliTestFixtures.CreateGameStatus(
            running: true,
            worldReady: true,
            worldName: "Campaign",
            entityCount: 5000,
            modPlatformReady: true,
            version: "0.15.0",
            loadedPacks: ["warfare-modern", "economy-balanced"]);

        // Assert
        status.Running.Should().BeTrue();
        status.EntityCount.Should().Be(5000);
        status.LoadedPacks.Should().HaveCount(2);
    }

    [Fact]
    public void QueryResult_WithMultipleEntities()
    {
        // Arrange
        var entities = new List<EntityInfo>
        {
            new() { Index = 0, Components = ["Health", "Faction", "UnitType"] },
            new() { Index = 1, Components = ["Health", "Armor", "Weapon"] },
            new() { Index = 2, Components = ["Building", "Production", "Health"] }
        };

        // Act
        var result = CliTestFixtures.CreateQueryResult(count: 3, entities: entities);

        // Assert
        result.Count.Should().Be(3);
        result.Entities.Should().HaveCount(3);
        result.Entities[0].Index.Should().Be(0);
        result.Entities[1].Index.Should().Be(1);
        result.Entities[2].Index.Should().Be(2);
    }

    [Fact]
    public void OverrideResult_SuccessfulApplication()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateOverrideResult(
            success: true,
            sdkPath: "unit.stats.hp",
            modifiedCount: 100,
            message: "Applied to 100 units");

        // Assert
        result.Success.Should().BeTrue();
        result.ModifiedCount.Should().Be(100);
        result.Message.Should().Contain("100 units");
    }

    [Fact]
    public void ResourceSnapshot_GameState()
    {
        // Arrange & Act
        var resources = CliTestFixtures.CreateResourceSnapshot(
            food: 500,
            wood: 300,
            stone: 200,
            iron: 100,
            money: 50);

        // Assert
        var total = resources.Food + resources.Wood + resources.Stone +
                   resources.Iron + resources.Money;
        total.Should().Be(1150);
    }

    [Fact]
    public void ReloadResult_SuccessfulReload()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateSuccessfulReloadResult(
            ["pack1", "pack2", "pack3"]);

        // Assert
        result.Success.Should().BeTrue();
        result.LoadedPacks.Should().HaveCount(3);
        result.Errors.Should().BeEmpty();
    }

    [Fact]
    public void ReloadResult_FailedReload()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateFailedReloadResult(
            ["Syntax error", "Missing dependency", "Version conflict"]);

        // Assert
        result.Success.Should().BeFalse();
        result.LoadedPacks.Should().BeEmpty();
        result.Errors.Should().HaveCount(3);
    }

    [Fact]
    public void MultipleResults_SequentialOperations()
    {
        // Arrange
        var statusBefore = CliTestFixtures.CreateGameStatus(entityCount: 100);
        var queryResult = CliTestFixtures.CreateQueryResultWithEntities(10);
        var overrideResult = CliTestFixtures.CreateOverrideResult(modifiedCount: 10);
        var statusAfter = CliTestFixtures.CreateGameStatus(entityCount: 100);

        // Assert
        statusBefore.EntityCount.Should().Be(100);
        queryResult.Count.Should().Be(10);
        overrideResult.ModifiedCount.Should().Be(10);
        statusAfter.EntityCount.Should().Be(100);
    }

    [Fact]
    public void EntityInfo_CompleteData()
    {
        // Arrange & Act
        var entity = new EntityInfo
        {
            Index = 42,
            Components = ["Health", "Armor", "Weapon", "Movement", "AI"]
        };

        // Assert
        entity.Index.Should().Be(42);
        entity.Components.Should().HaveCount(5);
        entity.Components.Should().Contain("Health");
    }

    [Fact]
    public void GameStatus_LargeEntityCount()
    {
        // Arrange & Act
        var status = CliTestFixtures.CreateGameStatus(entityCount: 50000);

        // Assert
        status.EntityCount.Should().Be(50000);
    }

    [Fact]
    public void QueryResult_LargeResultSet()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateQueryResultWithEntities(500);

        // Assert
        result.Count.Should().Be(500);
        result.Entities.Should().HaveCount(500);
        result.Entities.Last().Index.Should().Be(499);
    }

    [Fact]
    public void ResourceSnapshot_AllZero()
    {
        // Arrange & Act
        var resources = CliTestFixtures.CreateResourceSnapshot(
            food: 0, wood: 0, stone: 0, iron: 0, money: 0,
            souls: 0, bones: 0, spirit: 0);

        // Assert
        resources.Food.Should().Be(0);
        resources.Wood.Should().Be(0);
        resources.Money.Should().Be(0);
    }
}
