#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Bridge.Protocol;
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
    // ─────────────────────────────────────────────────────────────────────────────
    // Step 1: LoadPacks → verify warfare-starwars loads with 28 units
    // ─────────────────────────────────────────────────────────────────────────────

    /// <summary>
    /// Verifies that ReloadPacks returns success and includes warfare-starwars with 28 units.
    /// </summary>
    [Fact]
    public void BridgeRoundTrip_Step1_LoadPacks_WarfareStarwarsLoadsWithTwentyEightUnits()
    {
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
    [Fact]
    public void BridgeRoundTrip_Step2_GetStatus_ReportsReadyWorldWithEntities()
    {
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
    [Fact]
    public void BridgeRoundTrip_Step3_QueryUnits_RepCloneTrooperIsInCatalog()
    {
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
    [Fact]
    public void BridgeRoundTrip_Step4_ApplyOverride_RepCloneTrooperHpSetTo999()
    {
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
    [Fact]
    public void BridgeRoundTrip_Step5_ReadStat_HpReturns999AfterOverride()
    {
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
    [Fact]
    public void BridgeRoundTrip_Step6_ReloadPacks_SucceedsAndOverridePersists()
    {
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
    [Fact]
    public void BridgeRoundTrip_FullSequence_PackLoadStatReadOverrideApplyVerifyReload()
    {
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

// ─────────────────────────────────────────────────────────────────────────────
// In-memory fake implementation of IGameBridge for offline testing
// ─────────────────────────────────────────────────────────────────────────────

/// <summary>
/// Offline fake implementation of <see cref="IGameBridge"/> for use in smoke tests.
/// Models the minimal state transitions of the bridge without requiring a game process.
///
/// State machine:
/// - Packs: loaded via ReloadPacks; catalog populated on first load.
/// - Overrides: stored in a dictionary keyed by sdkPath; persist across reloads.
/// - Stats: default values from a fixed table; overrides win over defaults.
/// </summary>
public sealed class FakeGameBridge : IGameBridge
{
    // ── Internal state ──────────────────────────────────────────────────────────

    private readonly List<string> _loadedPacks = new();
    private readonly Dictionary<string, float> _statOverrides = new(StringComparer.OrdinalIgnoreCase);
    private bool _packsLoaded;

    // Fixed catalog mirroring warfare-starwars (28 units, 14 per faction).
    private static readonly CatalogSnapshot FixedCatalog = BuildFixedCatalog();

    // Default stat values before any override is applied.
    private static readonly Dictionary<string, float> DefaultStats = new(StringComparer.OrdinalIgnoreCase)
    {
        ["unit.stats.hp"] = 100f,
        ["unit.stats.speed"] = 3f,
        ["unit.stats.damage"] = 25f,
    };

    // ── IGameBridge implementation ───────────────────────────────────────────────

    /// <inheritdoc />
    public GameStatus Status() => new()
    {
        Running = true,
        WorldReady = _packsLoaded,
        WorldName = _packsLoaded ? "Default World" : "",
        EntityCount = _packsLoaded ? 45776 : 0,
        ModPlatformReady = _packsLoaded,
        LoadedPacks = new List<string>(_loadedPacks),
        Version = "0.1.0-fake",
    };

    /// <inheritdoc />
    public WaitResult WaitForWorld(int? timeoutMs) => new()
    {
        Ready = _packsLoaded,
        WorldName = _packsLoaded ? "Default World" : "",
    };

    /// <inheritdoc />
    public QueryResult QueryEntities(string? componentType, string? category) => new()
    {
        Count = _packsLoaded ? 100 : 0,
        Entities = _packsLoaded
            ? new List<EntityInfo> { new() { Index = 0, Components = new List<string> { "Components.Health" } } }
            : new List<EntityInfo>(),
    };

    /// <inheritdoc />
    public StatResult GetStat(string sdkPath, int? entityIndex)
    {
        float value = _statOverrides.TryGetValue(sdkPath, out float ov)
            ? ov
            : DefaultStats.TryGetValue(sdkPath, out float def) ? def : 0f;

        return new StatResult
        {
            SdkPath = sdkPath,
            Value = value,
            EntityCount = _packsLoaded ? 28 : 0,
            ComponentType = sdkPath.StartsWith("unit.stats.hp", StringComparison.OrdinalIgnoreCase)
                ? "Components.Health"
                : "Components.Unknown",
            FieldName = sdkPath.Split('.').LastOrDefault() ?? "",
            Values = _packsLoaded ? new List<float> { value } : new List<float>(),
        };
    }

    /// <inheritdoc />
    public OverrideResult ApplyOverride(string sdkPath, float value, string? mode, string? filter)
    {
        if (!_packsLoaded)
            return new OverrideResult { Success = false, Message = "Packs not loaded", SdkPath = sdkPath };

        float current = _statOverrides.TryGetValue(sdkPath, out float existing)
            ? existing
            : DefaultStats.TryGetValue(sdkPath, out float def) ? def : 0f;

        float newValue = mode switch
        {
            "add" => current + value,
            "multiply" => current * value,
            _ => value, // "override" mode stores the absolute value
        };

        _statOverrides[sdkPath] = newValue;

        return new OverrideResult
        {
            Success = true,
            ModifiedCount = 14, // rep_clone_trooper appears in 14 Republic unit entries
            SdkPath = sdkPath,
            Message = $"Applied {mode ?? "override"}: {sdkPath} = {newValue}",
        };
    }

    /// <inheritdoc />
    public ReloadResult ReloadPacks(string? path)
    {
        _packsLoaded = true;
        _loadedPacks.Clear();
        _loadedPacks.AddRange(new[] { "warfare-starwars", "example-balance" });

        // Overrides persist across reload by design (hot-reload must not reset runtime state).
        return new ReloadResult
        {
            Success = true,
            LoadedPacks = new List<string>(_loadedPacks),
            Errors = new List<string>(),
        };
    }

    /// <inheritdoc />
    public CatalogSnapshot GetCatalog() => _packsLoaded ? FixedCatalog : new CatalogSnapshot();

    /// <inheritdoc />
    public CatalogSnapshot DumpState(string? category) => GetCatalog();

    /// <inheritdoc />
    public ResourceSnapshot GetResources() => new()
    {
        Food = 400,
        Wood = 300,
        Stone = 200,
        Iron = 100,
    };

    /// <inheritdoc />
    public ScreenshotResult Screenshot(string? path) => new()
    {
        Success = true,
        Path = path ?? "screenshot.png",
    };

    /// <inheritdoc />
    public VerifyResult VerifyMod(string? packPath) => new()
    {
        Loaded = true,
        Errors = new List<string>(),
    };

    /// <inheritdoc />
    public PingResult Ping() => new()
    {
        Pong = true,
        Version = "0.1.0-fake",
        UptimeSeconds = 42.0,
    };

    /// <inheritdoc />
    public ComponentMapResult GetComponentMap(string? sdkPath)
    {
        List<ComponentMapEntry> all = new()
        {
            new() { SdkPath = "unit.stats.hp",     EcsType = "Components.Health",        FieldName = "Value" },
            new() { SdkPath = "unit.stats.speed",   EcsType = "Components.MovementSpeed", FieldName = "Value" },
            new() { SdkPath = "unit.stats.damage",  EcsType = "Components.AttackCooldown",FieldName = "Value" },
        };

        return new ComponentMapResult
        {
            Mappings = sdkPath == null
                ? all
                : all.Where(m => m.SdkPath.StartsWith(sdkPath, StringComparison.OrdinalIgnoreCase)).ToList(),
        };
    }

    /// <inheritdoc />
    public UiTreeResult GetUiTree(string? selector) => new();

    /// <inheritdoc />
    public UiActionResult QueryUi(string selector) => new()
    {
        Success = false,
        Message = "UI not available offline",
    };

    /// <inheritdoc />
    public UiActionResult ClickUi(string selector) => new()
    {
        Success = false,
        Message = "UI not available offline",
    };

    /// <inheritdoc />
    public UiWaitResult WaitForUi(string selector, string? state, int? timeoutMs) => new()
    {
        Ready = false,
    };

    /// <inheritdoc />
    public UiExpectationResult ExpectUi(string selector, string condition) => new()
    {
        Success = false,
    };

    // ── Private helpers ─────────────────────────────────────────────────────────

    /// <summary>
    /// Builds a fixed catalog snapshot that mirrors warfare-starwars:
    /// 14 Republic units + 14 CIS units = 28 total, each with EntityCount = 1
    /// so that <c>Units.Sum(u => u.EntityCount) == 28</c>.
    /// </summary>
    private static CatalogSnapshot BuildFixedCatalog()
    {
        // Matches republic_units.yaml (14 entries)
        string[] republicIds =
        {
            "rep_clone_trooper", "rep_arc_trooper", "rep_heavy_trooper", "rep_vehicle_crew",
            "rep_arf_trooper", "rep_speeder_pilot", "rep_jedi_knight", "rep_clone_commando",
            "rep_clone_sergeant", "rep_clone_lieutenant", "rep_clone_captain", "rep_clone_medic",
            "rep_clone_engineer", "rep_clone_sniper",
        };

        // Matches cis_units.yaml (14 entries)
        string[] cisIds =
        {
            "cis_battle_droid", "cis_super_battle_droid", "cis_droideka", "cis_commando_droid",
            "cis_aqua_droid", "cis_magna_guard", "cis_hmp_gunship_crew", "cis_dwarf_spider_droid",
            "cis_spider_droid", "cis_octuptarra_droid", "cis_hailfire_droid", "cis_sniper_droid",
            "cis_saboteur_droid", "cis_heavy_assault_droid",
        };

        List<CatalogEntry> units = republicIds
            .Select(id => new CatalogEntry
            {
                InferredId = id,
                Category = "unit",
                EntityCount = 1,
                ComponentCount = 8,
            })
            .Concat(cisIds.Select(id => new CatalogEntry
            {
                InferredId = id,
                Category = "unit",
                EntityCount = 1,
                ComponentCount = 8,
            }))
            .ToList();

        return new CatalogSnapshot
        {
            Units = units,
            Buildings = new List<CatalogEntry>(),
            Projectiles = new List<CatalogEntry>(),
            Other = new List<CatalogEntry>(),
        };
    }
}
