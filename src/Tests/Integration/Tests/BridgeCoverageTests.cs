#nullable enable
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Integration tests for bridge methods that were previously uncovered or minimally tested.
/// Covers: Screenshot, VerifyMod, GetComponentMap, DumpState, UI methods, Ping with timing.
/// </summary>
[Trait("Category", "BridgeCoverage")]
public class BridgeCoverageTests
{
    private readonly FakeGameBridge _bridge = new();

    // ────────────────────── Screenshot ──────────────────────

    [Fact]
    public void Screenshot_WithNullPath_ReturnsDefaultPath()
    {
        ScreenshotResult result = _bridge.Screenshot(null);

        result.Success.Should().BeTrue();
        result.Path.Should().NotBeNullOrEmpty();
    }

    [Fact]
    public void Screenshot_WithCustomPath_ReturnsCustomPath()
    {
        ScreenshotResult result = _bridge.Screenshot("test-capture.png");

        result.Success.Should().BeTrue();
        result.Path.Should().Be("test-capture.png");
    }

    // ────────────────────── VerifyMod ──────────────────────

    [Fact]
    public void VerifyMod_WithValidPath_ReturnsLoaded()
    {
        VerifyResult result = _bridge.VerifyMod("DINOForge.Runtime");

        result.Loaded.Should().BeTrue();
        result.Errors.Should().BeEmpty();
    }

    [Fact]
    public void VerifyMod_WithNullPath_StillReturnsLoaded()
    {
        VerifyResult result = _bridge.VerifyMod(null);

        result.Loaded.Should().BeTrue();
    }

    // ────────────────────── GetComponentMap ──────────────────────

    [Fact]
    public void GetComponentMap_WithNullSdkPath_ReturnsAllMappings()
    {
        ComponentMapResult result = _bridge.GetComponentMap(null);

        result.Mappings.Should().NotBeEmpty();
        result.Mappings.Should().Contain(m => m.SdkPath == "unit.stats.hp");
        result.Mappings.Should().Contain(m => m.SdkPath == "unit.stats.speed");
        result.Mappings.Should().Contain(m => m.SdkPath == "unit.stats.damage");
    }

    [Fact]
    public void GetComponentMap_WithSpecificPath_FiltersCorrectly()
    {
        ComponentMapResult result = _bridge.GetComponentMap("unit.stats.hp");

        result.Mappings.Should().ContainSingle(m => m.SdkPath == "unit.stats.hp");
        result.Mappings.All(m => m.SdkPath.StartsWith("unit.stats.hp", StringComparison.OrdinalIgnoreCase)).Should().BeTrue();
    }

    [Fact]
    public void GetComponentMap_ReturnsEcsTypeAndFieldName()
    {
        ComponentMapResult result = _bridge.GetComponentMap(null);
        ComponentMapEntry hpEntry = result.Mappings.First(m => m.SdkPath == "unit.stats.hp");

        hpEntry.EcsType.Should().Be("Components.Health");
        hpEntry.FieldName.Should().Be("Value");
    }

    // ────────────────────── DumpState ──────────────────────

    [Fact]
    public void DumpState_WithNullCategory_ReturnsCatalog()
    {
        CatalogSnapshot result = _bridge.DumpState(null);

        result.Should().NotBeNull();
        result.Units.Should().NotBeNull();
        result.Buildings.Should().NotBeNull();
    }

    [Fact]
    public void DumpState_BeforePackLoad_ReturnsEmpty()
    {
        // Create fresh bridge without loading packs
        var freshBridge = new FakeGameBridge();
        CatalogSnapshot result = freshBridge.DumpState(null);

        result.Units.Should().BeEmpty();
        result.Buildings.Should().BeEmpty();
    }

    // ────────────────────── UI Methods ──────────────────────

    [Fact]
    public void GetUiTree_WithNullSelector_ReturnsTree()
    {
        UiTreeResult result = _bridge.GetUiTree(null);

        result.Should().NotBeNull();
    }

    [Fact]
    public void QueryUi_WithSelector_ReturnsNotAvailable()
    {
        UiActionResult result = _bridge.QueryUi("role=button");

        result.Success.Should().BeFalse();
        result.Message.Should().Contain("UI not available");
    }

    [Fact]
    public void ClickUi_WithSelector_ReturnsNotAvailable()
    {
        UiActionResult result = _bridge.ClickUi("name=StartButton");

        result.Success.Should().BeFalse();
    }

    [Fact]
    public void WaitForUi_ReturnsNotReady()
    {
        UiWaitResult result = _bridge.WaitForUi("role=button", "visible", 5000);

        result.Ready.Should().BeFalse();
    }

    [Fact]
    public void ExpectUi_WithSelector_ReturnsNotMet()
    {
        UiExpectationResult result = _bridge.ExpectUi("role=button", "visible");

        result.Success.Should().BeFalse();
    }

    // ────────────────────── Ping ──────────────────────

    [Fact]
    public void Ping_ReturnsPongAndVersion()
    {
        PingResult result = _bridge.Ping();

        result.Pong.Should().BeTrue();
        result.Version.Should().NotBeNullOrEmpty();
        result.UptimeSeconds.Should().BeGreaterThan(0);
    }

    // ────────────────────── Resources ──────────────────────

    [Fact]
    public void GetResources_ReturnsAllResourceTypes()
    {
        ResourceSnapshot result = _bridge.GetResources();

        result.Food.Should().BeGreaterThan(0);
        result.Wood.Should().BeGreaterThan(0);
        result.Stone.Should().BeGreaterThan(0);
        result.Iron.Should().BeGreaterThan(0);
    }

    [Fact]
    public void GetResources_AllNonNegative()
    {
        ResourceSnapshot result = _bridge.GetResources();

        result.Food.Should().BeGreaterOrEqualTo(0);
        result.Wood.Should().BeGreaterOrEqualTo(0);
        result.Stone.Should().BeGreaterOrEqualTo(0);
        result.Iron.Should().BeGreaterOrEqualTo(0);
    }

    // ────────────────────── WaitForWorld ──────────────────────

    [Fact]
    public void WaitForWorld_BeforePackLoad_ReportsNotReady()
    {
        var freshBridge = new FakeGameBridge();
        WaitResult result = freshBridge.WaitForWorld(1000);

        result.Ready.Should().BeFalse();
    }

    [Fact]
    public void WaitForWorld_AfterPackLoad_ReportsReady()
    {
        _bridge.ReloadPacks(null);
        WaitResult result = _bridge.WaitForWorld(1000);

        result.Ready.Should().BeTrue();
        result.WorldName.Should().NotBeNullOrEmpty();
    }

    [Fact]
    public void WaitForWorld_WithNullTimeout_StillWorks()
    {
        _bridge.ReloadPacks(null);
        WaitResult result = _bridge.WaitForWorld(null);

        result.Ready.Should().BeTrue();
    }
}
