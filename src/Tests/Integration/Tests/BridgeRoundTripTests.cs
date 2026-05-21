#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// End-to-end offline smoke test for the full bridge round-trip.
///
/// Uses an in-memory fake implementation of <see cref="IGameBridge"/> to validate
/// the full offline sequence without requiring the game process to be running:
/// 1. LoadPacks  — verify warfare-starwars loads with 28 units
/// 2. GetStatus  — verify entity count > 0 from mock
/// 3. QueryUnits — verify rep_clone_trooper is in catalog
/// 4. ApplyOverride — apply hp=999 to rep_clone_trooper
/// 5. ReadStat   — verify hp changed to 999
/// 6. ReloadPacks — verify reload succeeds and override persists
/// </summary>
/// <remarks>
/// Maps to:
/// - US-F1.1 Pack Manifest Loading
/// - US-F4.1 Hot Module Reload
/// - US-F5.1 Asset Swap System
/// - Journey-CreateTotalConversion
/// </remarks>
[Trait("Category", "BridgeRoundTrip")]
[Trait("Category", "Epic")]
[Trait("Epic", "Epic-Integration")]
[Trait("Category", "Journey")]
[Trait("Journey", "Journey-CreateTotalConversion")]
[Trait("Category", "UserStory")]
[Trait("UserStory", "US-F1.1")]
[Trait("UserStory", "US-F4.1")]
[Trait("UserStory", "US-F5.1")]
public class BridgeRoundTripTests
{
    // #820: Pre-push gate. These tests use FakeGameBridge (in-process mock), so they do
    // NOT require a running DINO. The gate exists so pre-push test-integration can be
    // narrowed to "no game-touching tests" via DINO_DISABLE_TEST_LAUNCH=1 without
    // sweeping in this offline class. Default behavior (env unset) still runs the tests.
    private readonly bool _gameAvailable;

    public BridgeRoundTripTests()
    {
        var disableLaunch = Environment.GetEnvironmentVariable("DINO_DISABLE_TEST_LAUNCH");
        // Offline mock-only — "gameAvailable" here means "test infrastructure available".
        // Skip only when explicitly disabled (covers paranoid pre-push contexts).
        _gameAvailable = string.IsNullOrEmpty(disableLaunch) || disableLaunch == "0";
    }

    private void SkipIfGameNotAvailable()
    {
        Skip.IfNot(_gameAvailable, "Integration tests disabled via DINO_DISABLE_TEST_LAUNCH.");
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Step 1: LoadPacks → verify warfare-starwars loads with 28 units
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// Verifies that ReloadPacks returns success and includes warfare-starwars with 28 units.
    /// </summary>
    [SkippableFact]
    public void BridgeRoundTrip_Step1_LoadPacks_WarfareStarwarsLoadsWithTwentyEightUnits()
    {
        SkipIfGameNotAvailable();

        // Arrange
        FakeGameBridge bridge = new();

        // Act
        ReloadResult result = bridge.ReloadPacks(path: null);

        // Assert
        result.Success.Should().BeTrue("pack reload should succeed");
        result.Errors.Should().BeEmpty("no errors should occur during load");
        result.LoadedPacks.Should().Contain("warfare-starwars",
            because: "the Star Wars pack should be registered in the fake bridge");

        // Verify the catalog shows 28 units (14 Republic + 14 CIS)
        CatalogSnapshot catalog = bridge.GetCatalog();
        int totalUnits = catalog.Units.Sum(u => u.EntityCount);
        totalUnits.Should().Be(28,
            because: "warfare-starwars defines exactly 28 units (14 Republic + 14 CIS)");
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Step 2: GetStatus → verify entity count > 0 from mock
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// Verifies that the fake bridge status reports a ready ECS world with entities.
    /// </summary>
    [SkippableFact]
    public void BridgeRoundTrip_Step2_GetStatus_ReportsReadyWorldWithEntities()
    {
        SkipIfGameNotAvailable();

        // Arrange
        FakeGameBridge bridge = new();
        bridge.ReloadPacks(path: null); // Ensure packs are loaded first

        // Act
        GameStatus status = bridge.Status();

        // Assert
        status.Running.Should().BeTrue("the fake bridge reports the game as running");
        status.WorldReady.Should().BeTrue("the ECS world should be ready after pack load");
        status.EntityCount.Should().BeGreaterThan(0,
            because: "the mock world should report at least one entity");
        status.ModPlatformReady.Should().BeTrue("the mod platform should be fully initialized");
        status.LoadedPacks.Should().NotBeEmpty("at least one pack should be loaded");
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Step 3: QueryUnits → verify rep_clone_trooper is in catalog
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// Verifies that querying the catalog finds rep_clone_trooper among the loaded units.
    /// </summary>
    [SkippableFact]
    public void BridgeRoundTrip_Step3_QueryUnits_RepCloneTrooperIsInCatalog()
    {
        SkipIfGameNotAvailable();

        // Arrange
        FakeGameBridge bridge = new();
        bridge.ReloadPacks(path: null);

        // Act
        CatalogSnapshot catalog = bridge.GetCatalog();

        // Assert — rep_clone_trooper must be in the unit catalog
        bool hasCloneTrooper = catalog.Units.Any(u =>
            u.InferredId.Equals("rep_clone_trooper", StringComparison.OrdinalIgnoreCase));

        hasCloneTrooper.Should().BeTrue(
            because: "warfare-starwars defines rep_clone_trooper as the baseline Republic infantry unit");
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Step 4: ApplyOverride → apply hp=999 to rep_clone_trooper
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// Verifies that applying an override to rep_clone_trooper hp succeeds.
    /// </summary>
    [SkippableFact]
    public void BridgeRoundTrip_Step4_ApplyOverride_RepCloneTrooperHpSetTo999()
    {
        SkipIfGameNotAvailable();

        // Arrange
        FakeGameBridge bridge = new();
        bridge.ReloadPacks(path: null);

        // Act
        OverrideResult result = bridge.ApplyOverride(
            sdkPath: "unit.stats.hp",
            value: 999f,
            mode: "override",
            filter: "rep_clone_trooper");

        // Assert
        result.Success.Should().BeTrue("override should apply without error");
        result.ModifiedCount.Should().BeGreaterThan(0,
            because: "at least one rep_clone_trooper entity should be modified");
        result.SdkPath.Should().Be("unit.stats.hp",
            because: "the override result should echo back the SDK path");
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Step 5: ReadStat → verify hp changed to 999
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// Verifies that after applying hp=999, reading the stat returns 999.
    /// </summary>
    [SkippableFact]
    public void BridgeRoundTrip_Step5_ReadStat_HpReturns999AfterOverride()
    {
        SkipIfGameNotAvailable();

        // Arrange
        FakeGameBridge bridge = new();
        bridge.ReloadPacks(path: null);
        bridge.ApplyOverride("unit.stats.hp", 999f, "override", "rep_clone_trooper");

        // Act
        StatResult stat = bridge.GetStat("unit.stats.hp", entityIndex: null);

        // Assert
        stat.SdkPath.Should().Be("unit.stats.hp");
        stat.EntityCount.Should().BeGreaterThan(0,
            because: "there should be entities with a Health component");
        stat.Value.Should().BeApproximately(999f, 0.01f,
            because: "the override set hp=999 and no other operation changed it");
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Step 6: ReloadPacks → verify reload succeeds and override persists
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// Verifies that a second reload succeeds and that the applied override is still in effect.
    /// </summary>
    [SkippableFact]
    public void BridgeRoundTrip_Step6_ReloadPacks_SucceedsAndOverridePersists()
    {
        SkipIfGameNotAvailable();

        // Arrange
        FakeGameBridge bridge = new();
        bridge.ReloadPacks(path: null);
        bridge.ApplyOverride("unit.stats.hp", 999f, "override", "rep_clone_trooper");

        // Act — second reload (simulates hot reload triggered from CLI/MCP)
        ReloadResult reloadResult = bridge.ReloadPacks(path: null);

        // Assert — reload succeeded
        reloadResult.Success.Should().BeTrue("second reload should complete without errors");
        reloadResult.LoadedPacks.Should().Contain("warfare-starwars",
            because: "the pack should still be loaded after reload");

        // Assert — override is still in effect (hot-reload must not discard applied overrides)
        StatResult stat = bridge.GetStat("unit.stats.hp", entityIndex: null);
        stat.Value.Should().BeApproximately(999f, 0.01f,
            because: "stat overrides must persist across pack reloads");
    }

    // ─────────────────────────────────────────────────────────────────────────────
    // Full sequential round-trip in one test
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// Executes the complete bridge round-trip sequence in a single test, verifying
    /// that each step transitions the fake bridge state correctly.
    /// </summary>
    [SkippableFact]
    public void BridgeRoundTrip_FullSequence_PackLoadStatReadOverrideApplyVerifyReload()
    {
        SkipIfGameNotAvailable();

        // Arrange
        FakeGameBridge bridge = new();

        // Step 1 — load packs
        ReloadResult load = bridge.ReloadPacks(path: null);
        load.Success.Should().BeTrue("step 1: pack load");
        load.LoadedPacks.Should().Contain("warfare-starwars");

        // Step 2 — status
        GameStatus status = bridge.Status();
        status.WorldReady.Should().BeTrue("step 2: world ready");
        status.EntityCount.Should().BeGreaterThan(0, "step 2: entity count");

        // Step 3 — query units
        CatalogSnapshot catalog = bridge.GetCatalog();
        catalog.Units.Should().NotBeEmpty("step 3: units in catalog");
        catalog.Units.Any(u => u.InferredId == "rep_clone_trooper").Should().BeTrue(
            "step 3: rep_clone_trooper in catalog");

        // Step 4 — apply override
        OverrideResult ov = bridge.ApplyOverride("unit.stats.hp", 999f, "override", "rep_clone_trooper");
        ov.Success.Should().BeTrue("step 4: override applied");
        ov.ModifiedCount.Should().BeGreaterThan(0, "step 4: entities modified");

        // Step 5 — read stat
        StatResult stat = bridge.GetStat("unit.stats.hp", entityIndex: null);
        stat.Value.Should().BeApproximately(999f, 0.01f, "step 5: stat reflects override");

        // Step 6 — reload and verify persistence
        ReloadResult reload = bridge.ReloadPacks(path: null);
        reload.Success.Should().BeTrue("step 6: reload succeeded");
        StatResult statAfterReload = bridge.GetStat("unit.stats.hp", entityIndex: null);
        statAfterReload.Value.Should().BeApproximately(999f, 0.01f, "step 6: override persists after reload");
    }
}
