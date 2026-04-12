#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Bridge.Protocol;

namespace DINOForge.Tests.Mocks;

/// <summary>
/// Offline fake implementation of <see cref="IGameBridge"/> for use in offline testing.
/// Models the minimal state transitions of the bridge without requiring a game process.
/// </summary>
/// <remarks>
/// State machine:
/// - Packs: loaded via ReloadPacks; catalog populated on first load.
/// - Overrides: stored in a dictionary keyed by sdkPath; persist across reloads.
/// - Stats: default values from a fixed table; overrides win over defaults.
/// </remarks>
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
