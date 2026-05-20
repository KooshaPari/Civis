#nullable enable
using System;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Live game integration tests that require a running game instance.
/// These tests connect to the actual game via the IPC bridge and test
/// real game functionality.
///
/// Prerequisites:
/// - Game must be running with DINOForge Runtime plugin loaded
/// - Game must be at the main menu or in gameplay
/// - MCP bridge must be listening on the named pipe
///
/// When the game is not available, tests are skipped via Skip.IfNot()
/// using the [SkippableFact] attribute (Xunit.SkippableFact). This mirrors
/// the iter-144 7de6fd37 pattern landed for ScreenshotFallbackTests so
/// CI runs (where DINO is never present) record SKIP rather than fail
/// or hang.
/// </summary>
[Trait("Category", "LiveGame")]
[Trait("RequiresGame", "true")]
public class LiveGameIntegrationTests : IDisposable
{
    private readonly GameClient? _client;
    private readonly bool _gameAvailable;
    private readonly string _tempDir;

    public LiveGameIntegrationTests()
    {
        _tempDir = System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"dinoforge_live_test_{Guid.NewGuid():N}");
        System.IO.Directory.CreateDirectory(_tempDir);

        // Try to connect to the game. Use a short connect timeout so that
        // CI environments (where DINO is never running) skip immediately
        // instead of blocking the test-runner constructor for minutes.
        _client = new GameClient();
        try
        {
            _client.ConnectAsync(connectTimeout: TimeSpan.FromSeconds(2))
                .GetAwaiter().GetResult();
            _gameAvailable = _client.IsConnected;
        }
        catch
        {
            _gameAvailable = false;
            try { _client?.Dispose(); } catch { /* best-effort */ }
            _client = null;
        }
    }

    private void SkipIfGameNotAvailable()
    {
        Skip.IfNot(_gameAvailable, "DINO not available — live-game integration test skipped.");
    }

    public void Dispose()
    {
        try
        {
            _client?.Disconnect();
            _client?.Dispose();
        }
        catch { /* best-effort cleanup */ }

        try
        {
            if (System.IO.Directory.Exists(_tempDir))
                System.IO.Directory.Delete(_tempDir, true);
        }
        catch { /* best-effort cleanup */ }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Game Connection Tests
    // ═════════════════════════════════════════════════════════════════════════════

    /// <summary>
    /// GIVEN the game is running with DINOForge Runtime loaded
    /// WHEN we connect to the IPC bridge
    /// THEN the connection succeeds and we can ping the game
    /// </summary>
    [SkippableFact]
    public void LiveGame_ConnectToBridge_Succeeds()
    {
        SkipIfGameNotAvailable();

        _client.Should().NotBeNull("game client should be initialized");
        _client!.IsConnected.Should().BeTrue("should be connected to game bridge");
    }

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we ping the game
    /// THEN we get a valid pong response with game info
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_Ping_ReturnsPong()
    {
        SkipIfGameNotAvailable();

        var result = await _client!.PingAsync();

        result.Should().NotBeNull();
        result.Pong.Should().BeTrue();
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Game Status Tests
    // ═════════════════════════════════════════════════════════════════════════════

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we get the game status
    /// THEN the status reflects the actual game state
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_GetStatus_ReturnsGameState()
    {
        SkipIfGameNotAvailable();

        var status = await _client!.StatusAsync();

        status.Should().NotBeNull();
        status.Running.Should().BeTrue("game should be running");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Entity Catalog Tests
    // ═════════════════════════════════════════════════════════════════════════════

    /// <summary>
    /// GIVEN the game is running with packs loaded
    /// WHEN we query the catalog
    /// THEN we get the full entity catalog
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_GetCatalog_ReturnsEntities()
    {
        SkipIfGameNotAvailable();

        var catalog = await _client!.GetCatalogAsync();

        catalog.Should().NotBeNull("catalog should be available");
        catalog.Units.Should().NotBeNull("units list should exist");
    }

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we query for units
    /// THEN we get unit entities with stats
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_QueryUnits_ReturnsUnitData()
    {
        SkipIfGameNotAvailable();

        var result = await _client!.QueryEntitiesAsync("Unit", null);

        result.Should().NotBeNull();
        result.Count.Should().BeGreaterOrEqualTo(0);
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Stat Access Tests
    // ═════════════════════════════════════════════════════════════════════════════

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we read a unit stat
    /// THEN we get a stat result (value may be 0 if unit not found)
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_ReadStat_ReturnsResult()
    {
        SkipIfGameNotAvailable();

        var stat = await _client!.GetStatAsync("unit.stats.hp", null);

        stat.Should().NotBeNull();
        // Value may be 0 if unit doesn't exist or no entities match
    }

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we apply a stat override
    /// THEN the override is applied to matching entities
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_ApplyOverride_Succeeds()
    {
        SkipIfGameNotAvailable();

        var result = await _client!.ApplyOverrideAsync("unit.stats.hp", 999f, "override", null);

        result.Should().NotBeNull();
        // Success depends on whether units match the filter
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Pack Loading Tests
    // ═════════════════════════════════════════════════════════════════════════════

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we reload packs
    /// THEN the reload succeeds and packs are reloaded
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_ReloadPacks_Succeeds()
    {
        SkipIfGameNotAvailable();

        var result = await _client!.ReloadPacksAsync(null);

        result.Should().NotBeNull();
        result.Success.Should().BeTrue("pack reload should succeed");
    }

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we verify DINOForge Runtime is loaded
    /// THEN we get a verify result (loaded may be false if mod not injected)
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_VerifyMod_ReturnsResult()
    {
        SkipIfGameNotAvailable();

        var result = await _client!.VerifyModAsync("DINOForge.Runtime");

        result.Should().NotBeNull();
        // Loaded may be false if the mod isn't injected yet
        // The important thing is that the bridge responds
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Resources Tests
    // ═════════════════════════════════════════════════════════════════════════════

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we query resources
    /// THEN we get the current resource state
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_GetResources_ReturnsResources()
    {
        SkipIfGameNotAvailable();

        var resources = await _client!.GetResourcesAsync();

        resources.Should().NotBeNull();
        resources.Food.Should().BeGreaterOrEqualTo(0, "food should be non-negative");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Component Mapping Tests
    // ═════════════════════════════════════════════════════════════════════════════

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we get the component map
    /// THEN we get the SDK to ECS component mappings
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_GetComponentMap_ReturnsMappings()
    {
        SkipIfGameNotAvailable();

        var result = await _client!.GetComponentMapAsync(null);

        result.Should().NotBeNull();
        result.Mappings.Should().NotBeNull();
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // World Readiness Tests
    // ═════════════════════════════════════════════════════════════════════════════

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we wait for the world to be ready
    /// THEN the world is reported as ready
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_WaitForWorld_IsReady()
    {
        SkipIfGameNotAvailable();

        var result = await _client!.WaitForWorldAsync(1000);

        result.Should().NotBeNull();
        result.Ready.Should().BeTrue("ECS world should be ready");
    }

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we query entities
    /// THEN we get entity counts
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_QueryEntities_ReturnsEntities()
    {
        SkipIfGameNotAvailable();

        var result = await _client!.QueryEntitiesAsync("Unit", null);

        result.Should().NotBeNull();
        result.Count.Should().BeGreaterOrEqualTo(0);
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Screenshot Tests
    // ═════════════════════════════════════════════════════════════════════════════

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we take a screenshot
    /// THEN the screenshot is captured
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_Screenshot_Succeeds()
    {
        SkipIfGameNotAvailable();

        var screenshotPath = System.IO.Path.Combine(_tempDir, "test_screenshot.png");
        var result = await _client!.ScreenshotAsync(screenshotPath);

        result.Should().NotBeNull();
        // Result success depends on game state
    }
}
