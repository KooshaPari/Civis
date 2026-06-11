#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using DINOForge.Domains.Warfare;
using DINOForge.Domains.Warfare.Archetypes;
using DINOForge.Domains.Warfare.Balance;
using DINOForge.Domains.Warfare.Doctrines;
using DINOForge.Domains.Warfare.Roles;
using DINOForge.Domains.Warfare.Waves;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Targeted coverage tests for Warfare domain public constructors and methods
/// that were not exercised by the broader Warfare suites.
/// </summary>
public sealed class WarfareCoverageTests
{
    private readonly DoctrineEngine _doctrineEngine = new();
    private readonly UnitRoleValidator _roleValidator = new();
    private readonly WaveComposer _waveComposer = new();

    [Fact]
    public void ArchetypeRegistry_Register_AddsCustomArchetypeAndSupportsCaseInsensitiveLookup()
    {
        ArchetypeRegistry registry = new ArchetypeRegistry();
        FactionArchetype archetype = new FactionArchetype(
            "custom_support",
            "Custom Support",
            "Player-authored archetype.",
            new Dictionary<string, float>
            {
                { "armor", 1.05f },
                { "speed", 1.10f }
            });

        registry.Register(archetype);

        registry.TryGetArchetype("CUSTOM_SUPPORT", out FactionArchetype? resolved).Should().BeTrue();
        resolved.Should().BeSameAs(archetype);
        registry.All.Should().HaveCount(4);
    }

    [Fact]
    public void BalanceCalculator_Constructor_NullDoctrineEngine_ThrowsArgumentNullException()
    {
        Action action = () => new BalanceCalculator(null!);

        action.Should().Throw<ArgumentNullException>()
            .WithParameterName("doctrineEngine");
    }

    [Fact]
    public void BalanceComparisonReport_Constructor_PreservesInputs()
    {
        FactionPowerReport first = new FactionPowerReport(
            "alpha",
            "order",
            "elite_discipline",
            125.5f,
            62.75f,
            2,
            new Dictionary<string, float> { { "unit_a", 80.25f } });

        FactionPowerReport second = new FactionPowerReport(
            "beta",
            "industrial_swarm",
            null,
            100f,
            50f,
            2,
            new Dictionary<string, float> { { "unit_b", 50f } });

        BalanceComparisonReport report = new BalanceComparisonReport(
            first,
            second,
            25.5f,
            1.255f,
            "slight_advantage",
            "alpha");

        report.Faction1.Should().BeSameAs(first);
        report.Faction2.Should().BeSameAs(second);
        report.PowerDelta.Should().Be(25.5f);
        report.PowerRatio.Should().Be(1.255f);
        report.Assessment.Should().Be("slight_advantage");
        report.StrongerFaction.Should().Be("alpha");
    }

    [Fact]
    public void FactionPowerReport_Constructor_PreservesInputs()
    {
        IReadOnlyDictionary<string, float> unitPowerRatings = new Dictionary<string, float>
        {
            { "unit_a", 42.5f },
            { "unit_b", 17.25f }
        };

        FactionPowerReport report = new FactionPowerReport(
            "alpha",
            "order",
            null,
            59.75f,
            29.875f,
            2,
            unitPowerRatings);

        report.FactionId.Should().Be("alpha");
        report.ArchetypeId.Should().Be("order");
        report.DoctrineId.Should().BeNull();
        report.TotalPower.Should().Be(59.75f);
        report.AveragePower.Should().Be(29.875f);
        report.UnitCount.Should().Be(2);
        report.UnitPowerRatings.Should().BeSameAs(unitPowerRatings);
    }

    [Fact]
    public void RosterValidationResult_Constructor_PreservesInputs()
    {
        IReadOnlyList<string> missingRoles = new List<string> { "artillery", "hero_commander" };
        IReadOnlyList<string> filledRoles = new List<string> { "cheap_infantry", "line_infantry" };
        IReadOnlyDictionary<string, string> roleMap = new Dictionary<string, string>
        {
            { "cheap_infantry", "militia" },
            { "line_infantry", "line" }
        };

        RosterValidationResult result = new RosterValidationResult(
            isComplete: false,
            missingRoles: missingRoles,
            filledRoles: filledRoles,
            roleToUnitMap: roleMap);

        result.IsComplete.Should().BeFalse();
        result.MissingRoles.Should().BeSameAs(missingRoles);
        result.FilledRoles.Should().BeSameAs(filledRoles);
        result.RoleToUnitMap.Should().BeSameAs(roleMap);
    }

    [Fact]
    public void WarfareValidationResult_Constructor_PreservesInputs()
    {
        RosterValidationResult rosterResult = new RosterValidationResult(
            isComplete: true,
            missingRoles: Array.Empty<string>(),
            filledRoles: new[] { "cheap_infantry" },
            roleToUnitMap: new Dictionary<string, string> { { "cheap_infantry", "militia" } });

        IReadOnlyList<string> errors = new List<string> { "faction missing archetype" };
        IReadOnlyList<string> warnings = new List<string> { "fallback roster used" };
        IReadOnlyDictionary<string, RosterValidationResult> rosterResults = new Dictionary<string, RosterValidationResult>
        {
            { "alpha", rosterResult }
        };

        WarfareValidationResult result = new WarfareValidationResult(
            packId: "test-pack",
            isValid: false,
            errors: errors,
            warnings: warnings,
            rosterResults: rosterResults);

        result.PackId.Should().Be("test-pack");
        result.IsValid.Should().BeFalse();
        result.Errors.Should().BeSameAs(errors);
        result.Warnings.Should().BeSameAs(warnings);
        result.RosterResults.Should().BeSameAs(rosterResults);
    }

    [Fact]
    public void WaveComposer_ComposeWaves_WithNonPositiveWaveCount_ThrowsArgumentOutOfRangeException()
    {
        FactionDefinition faction = new FactionDefinition
        {
            Faction = new FactionInfo { Id = "alpha" }
        };
        IRegistry<UnitDefinition> units = new Registry<UnitDefinition>();

        Action action = () => _waveComposer.ComposeWaves(faction, units, 0);

        action.Should().Throw<ArgumentOutOfRangeException>()
            .WithParameterName("waveCount");
    }

    [Fact]
    public void WaveComposer_ComposeWaves_WithNoFactionUnits_ReturnsPlaceholderWaves()
    {
        FactionDefinition faction = new FactionDefinition
        {
            Faction = new FactionInfo { Id = "empty-faction" }
        };
        IRegistry<UnitDefinition> units = new Registry<UnitDefinition>();

        IReadOnlyList<WaveDefinition> waves = _waveComposer.ComposeWaves(faction, units, 3);

        waves.Should().HaveCount(3);
        waves[0].Id.Should().Be("empty-faction_wave_1");
        waves[0].DisplayName.Should().Be("Wave 1");
        waves[0].WaveNumber.Should().Be(1);
        waves[2].IsFinalWave.Should().BeTrue();
        waves.Should().OnlyContain(wave => wave.SpawnGroups.Count == 0);
    }

    [Fact]
    public void WarfareContentLoader_Constructor_NullFactions_ThrowsArgumentNullException()
    {
        Action action = () => new WarfareContentLoader(
            null!,
            new Registry<UnitDefinition>(),
            new Registry<BuildingDefinition>(),
            new Registry<WeaponDefinition>(),
            new Registry<ProjectileDefinition>(),
            new Registry<DoctrineDefinition>(),
            new Registry<WaveDefinition>(),
            new Registry<SquadDefinition>(),
            new ArchetypeRegistry());

        action.Should().Throw<ArgumentNullException>()
            .WithParameterName("factions");
    }

    [Fact]
    public void WarfareContentLoader_LoadPack_WithMissingDirectory_ThrowsDirectoryNotFoundException()
    {
        WarfareContentLoader loader = CreateContentLoader();
        string missingDir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString("N"));

        Action action = () => loader.LoadPack(missingDir, "test-pack");

        action.Should().Throw<DirectoryNotFoundException>();
    }

    [Fact]
    public async Task WarfareContentLoader_LoadPackAsync_WithMissingDirectory_ThrowsDirectoryNotFoundException()
    {
        WarfareContentLoader loader = CreateContentLoader();
        string missingDir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString("N"));

        Func<Task> action = () => loader.LoadPackAsync(missingDir, "test-pack");

        await action.Should().ThrowAsync<DirectoryNotFoundException>();
    }

    [Fact]
    public void DoctrineEngine_ApplyAll_WithNullArchetype_ThrowsArgumentNullException()
    {
        UnitStats baseStats = new UnitStats
        {
            Hp = 100f,
            Damage = 25f,
            Armor = 10f
        };

        Action action = () => _doctrineEngine.ApplyAll(baseStats, null!, null);

        action.Should().Throw<ArgumentNullException>()
            .WithParameterName("archetype");
    }

    [Fact]
    public void UnitRoleValidator_ValidateRoster_WithNullUnits_ThrowsArgumentNullException()
    {
        FactionDefinition faction = new FactionDefinition
        {
            Faction = new FactionInfo { Id = "alpha" }
        };

        Action action = () => _roleValidator.ValidateRoster(faction, null!);

        action.Should().Throw<ArgumentNullException>()
            .WithParameterName("units");
    }

    private static WarfareContentLoader CreateContentLoader()
    {
        return new WarfareContentLoader(
            new Registry<FactionDefinition>(),
            new Registry<UnitDefinition>(),
            new Registry<BuildingDefinition>(),
            new Registry<WeaponDefinition>(),
            new Registry<ProjectileDefinition>(),
            new Registry<DoctrineDefinition>(),
            new Registry<WaveDefinition>(),
            new Registry<SquadDefinition>(),
            new ArchetypeRegistry());
    }
}
