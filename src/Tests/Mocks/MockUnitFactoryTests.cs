#nullable enable
using System.Collections.Generic;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Mocks.Tests;

public class MockUnitFactoryTests
{
    [Fact]
    public void CanSpawn_WithRegisteredUnit_ReturnsTrue()
    {
        var factory = new MockUnitFactory();
        factory.SetSpawnable("unit-marine", true);

        var result = factory.CanSpawn("unit-marine");

        result.Should().BeTrue();
        factory.CanSpawnCount.Should().Be(1);
    }

    [Fact]
    public void CanSpawn_WithUnregisteredUnit_ReturnsFalse()
    {
        var factory = new MockUnitFactory();

        var result = factory.CanSpawn("unit-unknown");

        result.Should().BeFalse();
        factory.CanSpawnCount.Should().Be(1);
    }

    [Fact]
    public void RequestSpawn_TracksSpawnRequest()
    {
        var factory = new MockUnitFactory();
        factory.SetSpawnable("unit-marine", true);

        factory.RequestSpawn("unit-marine", 10.5f, 20.3f);

        factory.RequestSpawnCount.Should().Be(1);
        factory.SpawnRequests.Should().HaveCount(1);
        factory.SpawnRequests[0].UnitDefinitionId.Should().Be("unit-marine");
        factory.SpawnRequests[0].X.Should().Be(10.5f);
        factory.SpawnRequests[0].Z.Should().Be(20.3f);
    }

    [Fact]
    public void RequestSpawn_MultipleRequests_TracksAll()
    {
        var factory = new MockUnitFactory();
        factory.SetSpawnable("unit-marine", true);
        factory.SetSpawnable("unit-tank", true);

        factory.RequestSpawn("unit-marine", 0f, 0f);
        factory.RequestSpawn("unit-tank", 100f, 50f);

        factory.SpawnRequests.Should().HaveCount(2);
        factory.RequestSpawnCount.Should().Be(2);
    }

    [Fact]
    public void ClearRequests_RemovesAllSpawnRequests()
    {
        var factory = new MockUnitFactory();
        factory.SetSpawnable("unit-marine", true);
        factory.RequestSpawn("unit-marine", 0f, 0f);

        factory.ClearRequests();

        factory.SpawnRequests.Should().BeEmpty();
        factory.RequestSpawnCount.Should().Be(1);
    }

    [Fact]
    public void SetSpawnable_CanOverride()
    {
        var factory = new MockUnitFactory();
        factory.SetSpawnable("unit-marine", true);
        factory.CanSpawn("unit-marine").Should().BeTrue();

        factory.SetSpawnable("unit-marine", false);
        factory.CanSpawn("unit-marine").Should().BeFalse();
    }

    [Fact]
    public void RequestSpawn_DoesNotCheckSpawnability()
    {
        var factory = new MockUnitFactory();

        factory.RequestSpawn("unit-nonexistent", 5f, 15f);

        factory.SpawnRequests.Should().HaveCount(1);
        factory.RequestSpawnCount.Should().Be(1);
    }

    [Fact]
    public void CanSpawn_MultipleChecks_TracksCallCount()
    {
        var factory = new MockUnitFactory();
        factory.SetSpawnable("unit-marine", true);

        factory.CanSpawn("unit-marine");
        factory.CanSpawn("unit-marine");
        factory.CanSpawn("unit-tank");

        factory.CanSpawnCount.Should().Be(3);
    }

    [Fact]
    public void SpawnRequests_ReturnsReadOnlyList()
    {
        var factory = new MockUnitFactory();
        factory.RequestSpawn("unit-marine", 0f, 0f);

        var requests = factory.SpawnRequests;

        requests.Should().HaveCount(1);
        requests.Should().BeAssignableTo<IReadOnlyList<MockUnitFactory.SpawnRequest>>();
    }
}
