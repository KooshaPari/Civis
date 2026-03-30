#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.CliTools;

/// <summary>
/// Tests for Bridge Protocol types used in CLI commands.
/// </summary>
public class ProtocolTypeTests
{
    [Fact]
    public void GameStatus_CanBeCreated()
    {
        // Arrange & Act
        var status = new GameStatus
        {
            Running = true,
            WorldReady = true,
            WorldName = "Test",
            EntityCount = 100,
            ModPlatformReady = true,
            Version = "0.15.0",
            LoadedPacks = ["test-pack"]
        };

        // Assert
        status.Should().NotBeNull();
        status.Running.Should().BeTrue();
        status.EntityCount.Should().Be(100);
    }

    [Fact]
    public void GameStatus_AllPropertiesAccessible()
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
    public void QueryResult_CanBeCreated()
    {
        // Arrange & Act
        var result = new QueryResult
        {
            Count = 3,
            Entities = new List<EntityInfo>
            {
                new() { Index = 0, Components = ["Health"] },
                new() { Index = 1, Components = ["Armor"] },
                new() { Index = 2, Components = ["Movement"] }
            }
        };

        // Assert
        result.Count.Should().Be(3);
        result.Entities.Should().HaveCount(3);
    }

    [Fact]
    public void EntityInfo_ContainsComponentList()
    {
        // Arrange & Act
        var entity = new EntityInfo
        {
            Index = 42,
            Components = ["Health", "Armor", "Weapon"]
        };

        // Assert
        entity.Index.Should().Be(42);
        entity.Components.Should().Contain("Health");
        entity.Components.Should().Contain("Armor");
    }

    [Fact]
    public void OverrideResult_IndicatesSuccess()
    {
        // Arrange & Act
        var result = new OverrideResult
        {
            Success = true,
            SdkPath = "unit.stats.hp",
            ModifiedCount = 10,
            Message = "Success"
        };

        // Assert
        result.Success.Should().BeTrue();
        result.ModifiedCount.Should().Be(10);
    }

    [Fact]
    public void OverrideResult_IndicatesFailure()
    {
        // Arrange & Act
        var result = new OverrideResult
        {
            Success = false,
            SdkPath = "invalid.path",
            ModifiedCount = 0,
            Message = "Invalid path"
        };

        // Assert
        result.Success.Should().BeFalse();
        result.ModifiedCount.Should().Be(0);
        result.Message.Should().Contain("Invalid");
    }

    [Fact]
    public void ResourceSnapshot_ContainsAllResourceTypes()
    {
        // Arrange & Act
        var resources = CliTestFixtures.CreateResourceSnapshot(
            food: 100,
            wood: 200,
            stone: 150,
            iron: 75);

        // Assert
        resources.Food.Should().Be(100);
        resources.Wood.Should().Be(200);
        resources.Stone.Should().Be(150);
        resources.Iron.Should().Be(75);
    }

    [Fact]
    public void ResourceSnapshot_SupportsZeroValues()
    {
        // Arrange & Act
        var resources = CliTestFixtures.CreateResourceSnapshot(
            food: 0,
            wood: 0,
            stone: 0);

        // Assert
        resources.Food.Should().Be(0);
        resources.Wood.Should().Be(0);
        resources.Stone.Should().Be(0);
    }

    [Fact]
    public void ResourceSnapshot_SupportsLargeValues()
    {
        // Arrange & Act
        var resources = CliTestFixtures.CreateResourceSnapshot(
            food: int.MaxValue,
            wood: int.MaxValue,
            stone: int.MaxValue);

        // Assert
        resources.Food.Should().Be(int.MaxValue);
        resources.Wood.Should().Be(int.MaxValue);
        resources.Stone.Should().Be(int.MaxValue);
    }

    [Fact]
    public void ReloadResult_IndicatesSuccess()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateSuccessfulReloadResult(
            ["pack1", "pack2"]);

        // Assert
        result.Success.Should().BeTrue();
        result.LoadedPacks.Should().HaveCount(2);
        result.Errors.Should().BeEmpty();
    }

    [Fact]
    public void ReloadResult_IndicatesFailure()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateFailedReloadResult(
            ["Pack not found", "Invalid manifest"]);

        // Assert
        result.Success.Should().BeFalse();
        result.LoadedPacks.Should().BeEmpty();
        result.Errors.Should().HaveCount(2);
    }

    [Fact]
    public void QueryResult_SupportsEmptyEntities()
    {
        // Arrange & Act
        var result = new QueryResult
        {
            Count = 0,
            Entities = new List<EntityInfo>()
        };

        // Assert
        result.Count.Should().Be(0);
        result.Entities.Should().BeEmpty();
    }

    [Fact]
    public void QueryResult_CountMatchesEntityCount()
    {
        // Arrange
        var entities = new List<EntityInfo>
        {
            new() { Index = 0, Components = ["A"] },
            new() { Index = 1, Components = ["B"] },
            new() { Index = 2, Components = ["C"] }
        };

        // Act
        var result = new QueryResult
        {
            Count = 3,
            Entities = entities
        };

        // Assert
        result.Count.Should().Be(result.Entities.Count);
    }
}
