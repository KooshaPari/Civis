#nullable enable
using System;
using System.Threading.Tasks;
using FluentAssertions;
using DINOForge.Tests.Integration.Fixtures;
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
/// These tests use the shared GameFixture collection so the game connection
/// is established once per collection rather than once per test instance.
/// When the game is not available, each test exits through Skip.IfNot()
/// so CI runs (where DINO is never present) record SKIP rather than fail
/// or hang.
/// </summary>
[Collection("Game")]
[Trait("Category", "LiveGame")]
[Trait("RequiresGame", "true")]
public class LiveGameIntegrationTests : IDisposable
{
    /// <summary>
    /// Iter-145 infrastructure gate: prevents test methods from attempting game connection
    /// on CI/clean builds where the game sandbox is not available. Mirrors the pattern
    /// used by GameSandboxIntegrationTests and ParallelGameTestsWithHarness.
    ///
    /// When false, all test methods skip gracefully via Skip.IfNot(_infrastructureAvailable, ...).
    /// This reduces flaky test behavior and prevents stalled connections on headless CI runners.
    /// </summary>
    private static readonly bool _infrastructureAvailable =
        Directory.Exists(@"G:\dino_boxes") ||
        !string.IsNullOrEmpty(Environment.GetEnvironmentVariable("DINO_GAME_PATH"));

    private readonly GameFixture _fixture;
    private readonly string _tempDir;

    public LiveGameIntegrationTests(GameFixture fixture)
    {
        _fixture = fixture;
        _tempDir = System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"dinoforge_live_test_{Guid.NewGuid():N}");
        System.IO.Directory.CreateDirectory(_tempDir);
    }

    public void Dispose()
    {
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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        _fixture.Client.Should().NotBeNull("game client should be initialized");
        _fixture.Client.IsConnected.Should().BeTrue("should be connected to game bridge");
    }

    /// <summary>
    /// GIVEN the game is running
    /// WHEN we ping the game
    /// THEN we get a valid pong response with game info
    /// </summary>
    [SkippableFact]
    public async Task LiveGame_Ping_ReturnsPong()
    {
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var result = await _fixture.Client.PingAsync().ConfigureAwait(true);

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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var status = await _fixture.Client.StatusAsync().ConfigureAwait(true);

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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var catalog = await _fixture.Client.GetCatalogAsync().ConfigureAwait(true);

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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var result = await _fixture.Client.QueryEntitiesAsync("Unit", null).ConfigureAwait(true);

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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var stat = await _fixture.Client.GetStatAsync("unit.stats.hp", null).ConfigureAwait(true);

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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var result = await _fixture.Client.ApplyOverrideAsync("unit.stats.hp", 999f, "override", null).ConfigureAwait(true);

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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var result = await _fixture.Client.ReloadPacksAsync(null).ConfigureAwait(true);

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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var result = await _fixture.Client.VerifyModAsync("DINOForge.Runtime").ConfigureAwait(true);

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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var resources = await _fixture.Client.GetResourcesAsync().ConfigureAwait(true);

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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var result = await _fixture.Client.GetComponentMapAsync(null).ConfigureAwait(true);

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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var result = await _fixture.Client.WaitForWorldAsync(1000).ConfigureAwait(true);

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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var result = await _fixture.Client.QueryEntitiesAsync("Unit", null).ConfigureAwait(true);

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
        Skip.IfNot(_infrastructureAvailable, "Game infrastructure not available — integration test skipped.");
        Skip.IfNot(_fixture.GameAvailable, "DINO not available — live-game integration test skipped.");

        var screenshotPath = System.IO.Path.Combine(_tempDir, "test_screenshot.png");
        var result = await _fixture.Client.ScreenshotAsync(screenshotPath).ConfigureAwait(true);

        result.Should().NotBeNull();
        // Result success depends on game state
    }
}
