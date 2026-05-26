#nullable enable
using DINOForge.SDK.Registry;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

public sealed class MockRegistryTests
{
    [Fact]
    public void Register_IncrementCount()
    {
        var mock = new MockRegistry<string>();

        mock.Register("id-1", "value-1", RegistrySource.BaseGame, "pack-1");

        mock.RegisterCount.Should().Be(1);
        mock.RegisterCount.Should().NotBe(0);
    }

    [Fact]
    public void Get_RetrievesRegisteredEntry()
    {
        var mock = new MockRegistry<string>();
        mock.Register("id-1", "test-value", RegistrySource.BaseGame, "pack-1");

        var result = mock.Get("id-1");

        result.Should().Be("test-value");
        mock.GetCount.Should().Be(1);
    }

    [Fact]
    public void Clear_EmptiesRegistry_ViaMultipleRegistrations()
    {
        var mock = new MockRegistry<string>();
        mock.Register("id-1", "value-1", RegistrySource.BaseGame, "pack-1");
        mock.Register("id-2", "value-2", RegistrySource.BaseGame, "pack-1");
        mock.Register("id-3", "value-3", RegistrySource.BaseGame, "pack-1");

        mock.All.Count.Should().Be(3);

        // Verify that re-registering with higher priority replaces
        mock.Register("id-1", "overridden", RegistrySource.Pack, "pack-2");
        mock.Get("id-1").Should().Be("overridden");
    }

    [Fact]
    public void Contains_ReturnsTrueForRegisteredEntry()
    {
        var mock = new MockRegistry<string>();
        mock.Register("id-1", "value-1", RegistrySource.BaseGame, "pack-1");

        mock.Contains("id-1").Should().BeTrue();
        mock.ContainsCount.Should().Be(1);
    }

    [Fact]
    public void Contains_ReturnsFalseForMissingEntry()
    {
        var mock = new MockRegistry<string>();

        mock.Contains("id-missing").Should().BeFalse();
    }

    [Fact]
    public void Override_IncrementsOverrideCount()
    {
        var mock = new MockRegistry<string>();
        mock.Register("id-1", "original", RegistrySource.BaseGame, "pack-1");

        mock.Override("id-1", "overridden", RegistrySource.Pack, "pack-override", 200);

        mock.OverrideCount.Should().Be(1);
        mock.Get("id-1").Should().Be("overridden");
    }

    [Fact]
    public void DetectConflicts_ReturnsEmptyWhenNoConflicts()
    {
        var mock = new MockRegistry<string>();
        mock.Register("id-1", "value-1", RegistrySource.BaseGame, "pack-1");

        var conflicts = mock.DetectConflicts();

        conflicts.Should().BeEmpty();
        mock.DetectConflictsCount.Should().Be(1);
    }

    [Fact]
    public void DetectConflicts_DetectsTiedPriorities()
    {
        var mock = new MockRegistry<string>();
        // Both register at Vanilla source with same loadOrder (100) → same priority
        mock.Register("id-1", "value-from-pack1", RegistrySource.BaseGame, "pack-1", 100);
        mock.Register("id-1", "value-from-pack2", RegistrySource.BaseGame, "pack-2", 100);

        var conflicts = mock.DetectConflicts();

        conflicts.Should().HaveCount(1);
        conflicts[0].EntryId.Should().Be("id-1");
        conflicts[0].ConflictingPackIds.Should().Contain(new[] { "pack-1", "pack-2" });
    }
}
