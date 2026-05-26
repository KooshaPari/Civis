#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.CliTools;

/// <summary>
/// Tests for QueryResult protocol type and entity queries.
/// </summary>
public class QueryCommandTests
{
    [Fact]
    public void QueryCommand_ReturnsEntities_WhenCount5()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateQueryResultWithEntities(5);

        // Assert
        result.Count.Should().Be(5);
        result.Entities.Should().HaveCount(5);
    }

    [Fact]
    public void QueryCommand_ReturnsEmptyList_WhenZeroEntities()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateQueryResult(count: 0, entities: []);

        // Assert
        result.Count.Should().Be(0);
        result.Entities.Should().BeEmpty();
    }

    [Fact]
    public void QueryCommand_IncludesEntityIndices()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateQueryResultWithEntities(5);

        // Assert
        result.Entities.Select(e => e.Index).Should().Equal(0, 1, 2, 3, 4);
    }

    [Fact]
    public void QueryCommand_IncludesComponentNames()
    {
        // Arrange
        var entities = new List<EntityInfo>
        {
            new() { Index = 0, Components = ["Health", "Armor"] },
            new() { Index = 1, Components = ["Health", "Transform"] }
        };

        // Act
        var result = CliTestFixtures.CreateQueryResult(count: 2, entities: entities);

        // Assert
        result.Entities[0].Components.Should().Contain("Health");
        result.Entities[0].Components.Should().Contain("Armor");
    }

    [Fact]
    public void QueryCommand_EntityCountMatches()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateQueryResultWithEntities(3);

        // Assert
        result.Count.Should().Be(result.Entities.Count);
    }

    [Fact]
    public void QueryCommand_HandlesLargeEntityCounts()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateQueryResultWithEntities(1000);

        // Assert
        result.Count.Should().Be(1000);
        result.Entities.Should().HaveCount(1000);
    }

    [Fact]
    public void QueryCommand_AllEntitiesHaveComponents()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateQueryResultWithEntities(3);

        // Assert
        result.Entities.Should().AllSatisfy(e =>
        {
            e.Components.Should().NotBeEmpty();
        });
    }

    [Fact]
    public void QueryCommand_EntityIndicesAreSequential()
    {
        // Arrange & Act
        var result = CliTestFixtures.CreateQueryResultWithEntities(10);

        // Assert
        for (int i = 0; i < result.Entities.Count; i++)
        {
            result.Entities[i].Index.Should().Be(i);
        }
    }

    [Fact]
    public void QueryCommand_CustomEntityList()
    {
        // Arrange
        var entities = new List<EntityInfo>
        {
            new() { Index = 10, Components = ["ComponentA"] },
            new() { Index = 20, Components = ["ComponentB"] }
        };

        // Act
        var result = CliTestFixtures.CreateQueryResult(count: 2, entities: entities);

        // Assert
        result.Entities[0].Index.Should().Be(10);
        result.Entities[1].Index.Should().Be(20);
    }

    [Fact]
    public void QueryCommand_CountCanBeZero()
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
}
