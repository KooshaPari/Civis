#nullable enable
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Integration tests using FakeGameBridge for workflows without requiring live game.
/// Tests cover: status, catalog, stat access, override application, reload.
/// </summary>
[Trait("Category", "MockBridge")]
public class MockBridgeIntegrationTests
{
    private readonly FakeGameBridge _bridge = new();

    /// <summary>
    /// GIVEN a FakeGameBridge before pack load
    /// WHEN Status is called
    /// THEN the status reflects unloaded state
    /// </summary>
    [Fact]
    public void Status_BeforePackLoad_ReturnsUnloadedState()
    {
        // Act
        GameStatus status = _bridge.Status();

        // Assert
        status.Running.Should().BeTrue();
        status.WorldReady.Should().BeFalse();
        status.EntityCount.Should().Be(0);
    }

    /// <summary>
    /// GIVEN a FakeGameBridge after pack load
    /// WHEN Status is called
    /// THEN the status reflects loaded state
    /// </summary>
    [Fact]
    public void Status_AfterPackLoad_ReturnsLoadedState()
    {
        // Arrange
        _bridge.ReloadPacks(null);

        // Act
        GameStatus status = _bridge.Status();

        // Assert
        status.Running.Should().BeTrue();
        status.WorldReady.Should().BeTrue();
        status.EntityCount.Should().BeGreaterThan(0);
        status.LoadedPacks.Should().Contain("warfare-starwars");
    }

    /// <summary>
    /// GIVEN a FakeGameBridge
    /// WHEN GetStat is called with default value
    /// THEN the stat value is returned
    /// </summary>
    [Fact]
    public void GetStat_DefaultValue_ReturnsDefault()
    {
        // Act
        StatResult result = _bridge.GetStat("unit.stats.hp", null);

        // Assert
        result.Should().NotBeNull();
        result.SdkPath.Should().Be("unit.stats.hp");
        result.Value.Should().Be(100f);
    }

    /// <summary>
    /// GIVEN a FakeGameBridge with packs loaded
    /// WHEN ApplyOverride is called
    /// THEN the override is applied
    /// </summary>
    [Fact]
    public void ApplyOverride_ValidStat_AppliesSuccessfully()
    {
        // Arrange - must load packs first
        _bridge.ReloadPacks(null);

        // Act
        OverrideResult result = _bridge.ApplyOverride("unit.stats.hp", 200f, "override", "rep_clone_trooper");

        // Assert
        result.Success.Should().BeTrue();
        result.ModifiedCount.Should().BeGreaterThan(0);
    }

    /// <summary>
    /// GIVEN a FakeGameBridge with packs loaded and override applied
    /// WHEN GetStat is called
    /// THEN the overridden value is returned
    /// </summary>
    [Fact]
    public void GetStat_AfterOverride_ReturnsOverriddenValue()
    {
        // Arrange - must load packs first
        _bridge.ReloadPacks(null);
        _bridge.ApplyOverride("unit.stats.hp", 999f, "override", "rep_clone_trooper");

        // Act
        StatResult result = _bridge.GetStat("unit.stats.hp", null);

        // Assert
        result.Value.Should().BeApproximately(999f, 0.01f);
    }

    /// <summary>
    /// GIVEN a FakeGameBridge
    /// WHEN ReloadPacks is called
    /// THEN the reload succeeds and packs are loaded
    /// </summary>
    [Fact]
    public void ReloadPacks_NoArgs_LoadsPacks()
    {
        // Act
        ReloadResult result = _bridge.ReloadPacks(null);

        // Assert
        result.Success.Should().BeTrue();
        result.LoadedPacks.Should().Contain("warfare-starwars");
    }

    /// <summary>
    /// GIVEN a FakeGameBridge after pack load
    /// WHEN QueryEntities is called
    /// THEN entity count is returned
    /// </summary>
    [Fact]
    public void QueryEntities_AfterPackLoad_ReturnsEntities()
    {
        // Arrange
        _bridge.ReloadPacks(null);

        // Act
        QueryResult result = _bridge.QueryEntities("Unit", null);

        // Assert
        result.Count.Should().Be(100);
    }

    /// <summary>
    /// GIVEN a FakeGameBridge
    /// WHEN Ping is called
    /// THEN pong is returned
    /// </summary>
    [Fact]
    public void Ping_ReturnsPong()
    {
        // Act
        PingResult result = _bridge.Ping();

        // Assert
        result.Pong.Should().BeTrue();
    }

    /// <summary>
    /// GIVEN a FakeGameBridge after pack load
    /// WHEN GetCatalog is called
    /// THEN catalog is returned
    /// </summary>
    [Fact]
    public void GetCatalog_AfterPackLoad_ReturnsCatalog()
    {
        // Arrange
        _bridge.ReloadPacks(null);

        // Act
        CatalogSnapshot catalog = _bridge.GetCatalog();

        // Assert
        catalog.Should().NotBeNull();
        catalog.Units.Should().HaveCount(28);
    }

    /// <summary>
    /// GIVEN a FakeGameBridge after pack load
    /// WHEN OverrideAppliedAndReloaded
    /// THEN the override persists after reload
    /// </summary>
    [Fact]
    public void Override_PersistsAfterReload()
    {
        // Arrange
        _bridge.ReloadPacks(null);
        _bridge.ApplyOverride("unit.stats.hp", 500f, "override", "rep_clone_trooper");

        // Act - Reload packs (simulating hot reload)
        _bridge.ReloadPacks(null);
        StatResult result = _bridge.GetStat("unit.stats.hp", null);

        // Assert - Override should persist
        result.Value.Should().BeApproximately(500f, 0.01f);
    }
}
