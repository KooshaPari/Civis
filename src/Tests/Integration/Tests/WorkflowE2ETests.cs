#nullable enable
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// E2E tests for complete game workflows using FakeGameBridge.
/// Tests simulate full user journeys without requiring live game.
/// 
/// Maps to user journeys:
/// - Journey-InstallPlay: Pack loading and verification
/// - Journey-CreateBalance: Stat overrides and hot reload
/// - Journey-AutomateGame: MCP tool automation
/// </summary>
[Trait("Category", "E2E")]
[Trait("Category", "Journey")]
[Trait("Journey", "Journey-AutomateGame")]
[Trait("Category", "UserStory")]
[Trait("UserStory", "US-F1.1")]
[Trait("UserStory", "US-F4.1")]
public class WorkflowE2ETests
{
    private readonly FakeGameBridge _bridge = new();

    /// <summary>
    /// E2E: User queries stats, applies override, verifies change persists.
    /// </summary>
    [Fact]
    public void E2E_OverrideStat_ChangePersists()
    {
        // Step 1: Load packs
        var loadResult = _bridge.ReloadPacks(null);
        loadResult.Success.Should().BeTrue();

        // Step 2: Read default stat
        var stat1 = _bridge.GetStat("unit.stats.hp", null);
        stat1.Value.Should().Be(100f);

        // Step 3: Apply override
        var overrideResult = _bridge.ApplyOverride("unit.stats.hp", 200f, "override", null);
        overrideResult.Success.Should().BeTrue();

        // Step 4: Read stat after override
        var stat2 = _bridge.GetStat("unit.stats.hp", null);
        stat2.Value.Should().Be(200f);
    }

    /// <summary>
    /// E2E: User reloads packs, catalog is available.
    /// </summary>
    [Fact]
    public void E2E_ReloadPacks_CatalogAvailable()
    {
        // Step 1: Reload packs
        var reloadResult = _bridge.ReloadPacks(null);
        reloadResult.Success.Should().BeTrue();

        // Step 2: Get catalog
        var catalog = _bridge.GetCatalog();
        catalog.Units.Should().HaveCount(28, "warfare-starwars has 28 units (14 Republic + 14 CIS)");
    }

    /// <summary>
    /// E2E: User queries entities by component type.
    /// </summary>
    [Fact]
    public void E2E_QueryEntities_ReturnsEntities()
    {
        // Setup packs
        _bridge.ReloadPacks(null);

        // Query units
        var allUnits = _bridge.QueryEntities("Unit", null);
        allUnits.Count.Should().Be(100);
    }

    /// <summary>
    /// E2E: User verifies mod is loaded.
    /// </summary>
    [Fact]
    public void E2E_VerifyMod_ModLoaded()
    {
        // Reload packs to load mods
        _bridge.ReloadPacks(null);

        // Verify mod
        var verifyResult = _bridge.VerifyMod("DINOForge.Runtime");
        verifyResult.Should().NotBeNull();
        verifyResult.Loaded.Should().BeTrue();
    }

    /// <summary>
    /// E2E: User takes screenshot of gameplay.
    /// </summary>
    [Fact]
    public void E2E_Screenshot_Succeeds()
    {
        // Take screenshot
        var screenshotResult = _bridge.Screenshot(null);
        screenshotResult.Should().NotBeNull();
        screenshotResult.Success.Should().BeTrue();
    }

    /// <summary>
    /// E2E: User waits for world to be ready.
    /// </summary>
    [Fact]
    public void E2E_WaitForWorld_BecomesReady()
    {
        // Not ready initially
        var wait1 = _bridge.WaitForWorld(1000);
        wait1.Ready.Should().BeFalse();

        // Load packs
        _bridge.ReloadPacks(null);

        // Now ready
        var wait2 = _bridge.WaitForWorld(1000);
        wait2.Ready.Should().BeTrue();
    }

    /// <summary>
    /// E2E: User queries resource snapshot.
    /// </summary>
    [Fact]
    public void E2E_GetResources_ReturnsSnapshot()
    {
        var resources = _bridge.GetResources();
        resources.Should().NotBeNull();
        resources.Food.Should().Be(400);
        resources.Wood.Should().Be(300);
    }

    /// <summary>
    /// E2E: User dumps state for debugging.
    /// </summary>
    [Fact]
    public void E2E_DumpState_ReturnsCatalog()
    {
        _bridge.ReloadPacks(null);
        var dump = _bridge.DumpState(null);
        dump.Should().NotBeNull();
        dump.Units.Should().NotBeNull();
    }

    /// <summary>
    /// E2E: User pings bridge for health check.
    /// </summary>
    [Fact]
    public void E2E_Ping_RespondsPong()
    {
        var pingResult = _bridge.Ping();
        pingResult.Pong.Should().BeTrue();
        pingResult.UptimeSeconds.Should().BeGreaterThan(0);
    }

    /// <summary>
    /// E2E: User gets component map.
    /// </summary>
    [Fact]
    public void E2E_GetComponentMap_ReturnsMappings()
    {
        var result = _bridge.GetComponentMap(null);
        result.Should().NotBeNull();
        result.Mappings.Should().HaveCount(3);
    }

    /// <summary>
    /// E2E: Override persists across reload (hot reload scenario).
    /// </summary>
    [Fact]
    public void E2E_HotReload_OverridePersists()
    {
        // Load packs
        _bridge.ReloadPacks(null);

        // Apply override
        _bridge.ApplyOverride("unit.stats.hp", 500f, "override", null);

        // First read
        var stat1 = _bridge.GetStat("unit.stats.hp", null);
        stat1.Value.Should().Be(500f);

        // Hot reload
        _bridge.ReloadPacks(null);

        // Override should persist
        var stat2 = _bridge.GetStat("unit.stats.hp", null);
        stat2.Value.Should().Be(500f);
    }
}
