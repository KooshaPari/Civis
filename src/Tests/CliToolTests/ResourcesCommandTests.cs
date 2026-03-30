#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.CliTools;

/// <summary>
/// Tests for ResourceSnapshot protocol type.
/// </summary>
public class ResourcesCommandTests
{
    [Fact]
    public void ResourcesCommand_ReturnsResourceData()
    {
        // Arrange & Act
        var resourceData = CliTestFixtures.CreateResourceSnapshot(
            food: 100,
            wood: 200,
            stone: 150);

        // Assert
        resourceData.Should().NotBeNull();
        resourceData.Food.Should().Be(100);
        resourceData.Wood.Should().Be(200);
        resourceData.Stone.Should().Be(150);
    }

    [Fact]
    public void ResourcesCommand_ReturnsZeroResources()
    {
        // Arrange & Act
        var resourceData = CliTestFixtures.CreateResourceSnapshot(
            food: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            money: 0);

        // Assert
        resourceData.Food.Should().Be(0);
        resourceData.Wood.Should().Be(0);
        resourceData.Money.Should().Be(0);
    }

    [Fact]
    public void ResourcesCommand_ReturnsLargeNumbers()
    {
        // Arrange & Act
        var resourceData = CliTestFixtures.CreateResourceSnapshot(
            food: 999999,
            wood: 999999,
            stone: 999999);

        // Assert
        resourceData.Food.Should().Be(999999);
        resourceData.Wood.Should().Be(999999);
        resourceData.Stone.Should().Be(999999);
    }

    [Fact]
    public void ResourcesCommand_IncludesAllResourceTypes()
    {
        // Arrange & Act
        var resourceData = CliTestFixtures.CreateResourceSnapshot();

        // Assert
        resourceData.Food.Should().BeGreaterThanOrEqualTo(0);
        resourceData.Wood.Should().BeGreaterThanOrEqualTo(0);
        resourceData.Stone.Should().BeGreaterThanOrEqualTo(0);
        resourceData.Iron.Should().BeGreaterThanOrEqualTo(0);
        resourceData.Money.Should().BeGreaterThanOrEqualTo(0);
        resourceData.Souls.Should().BeGreaterThanOrEqualTo(0);
        resourceData.Bones.Should().BeGreaterThanOrEqualTo(0);
        resourceData.Spirit.Should().BeGreaterThanOrEqualTo(0);
    }

    [Fact]
    public void ResourcesCommand_DefaultValues()
    {
        // Arrange & Act
        var resourceData = CliTestFixtures.CreateResourceSnapshot();

        // Assert
        resourceData.Food.Should().Be(100);
        resourceData.Wood.Should().Be(200);
        resourceData.Stone.Should().Be(150);
        resourceData.Iron.Should().Be(75);
        resourceData.Money.Should().Be(50);
    }

    [Fact]
    public void ResourcesCommand_AllPropertiesAccessible()
    {
        // Arrange
        var resourceData = CliTestFixtures.CreateResourceSnapshot();

        // Act & Assert
        _ = resourceData.Food;
        _ = resourceData.Wood;
        _ = resourceData.Stone;
        _ = resourceData.Iron;
        _ = resourceData.Money;
        _ = resourceData.Souls;
        _ = resourceData.Bones;
        _ = resourceData.Spirit;
    }

    [Fact]
    public void ResourcesCommand_ModifyingProperties()
    {
        // Arrange
        var resourceData = CliTestFixtures.CreateResourceSnapshot();

        // Act
        resourceData.Food = 500;
        resourceData.Wood = 1000;

        // Assert
        resourceData.Food.Should().Be(500);
        resourceData.Wood.Should().Be(1000);
    }

    [Fact]
    public void ResourcesCommand_CustomInitialization()
    {
        // Arrange & Act
        var resources = new ResourceSnapshot
        {
            Food = 250,
            Wood = 300,
            Stone = 200,
            Iron = 100,
            Money = 75,
            Souls = 5,
            Bones = 10,
            Spirit = 3
        };

        // Assert
        resources.Food.Should().Be(250);
        resources.Souls.Should().Be(5);
        resources.Spirit.Should().Be(3);
    }
}
