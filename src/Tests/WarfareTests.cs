using System.Collections.Generic;
using System.Linq;
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

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for Warfare domain (factions, archetypes, waves, balance).
    /// Maps to: Journey-CreateTotalConversion
    /// </summary>
    [Trait("Category", "Epic")]
    [Trait("Epic", "Epic-Warfare")]
    [Trait("Category", "Journey")]
    [Trait("Journey", "Journey-CreateTotalConversion")]
    public class ArchetypeTests
    {
        private readonly ArchetypeRegistry _registry = new ArchetypeRegistry();

        [Theory]
        [InlineData("order")]
        [InlineData("industrial_swarm")]
        [InlineData("asymmetric")]
        public void GetArchetype_ReturnsArchetype_ForAllCanonicalTypes(string id)
        {
            FactionArchetype archetype = _registry.GetArchetype(id);
            archetype.Should().NotBeNull();
            archetype.Id.Should().Be(id);
            archetype.BaseModifiers.Should().NotBeEmpty();
        }

        [Fact]
        public void Order_HasCorrectModifiers()
        {
            FactionArchetype order = _registry.GetArchetype("order");
            order.BaseModifiers["armor"].Should().BeApproximately(1.10f, 0.001f);
            order.BaseModifiers["morale"].Should().BeApproximately(1.10f, 0.001f);
            order.BaseModifiers["speed"].Should().BeApproximately(0.95f, 0.001f);
        }

        [Fact]
        public void IndustrialSwarm_HasCorrectModifiers()
        {
            FactionArchetype swarm = _registry.GetArchetype("industrial_swarm");
            swarm.BaseModifiers["cost"].Should().BeApproximately(0.80f, 0.001f);
            swarm.BaseModifiers["speed"].Should().BeApproximately(1.30f, 0.001f);
            swarm.BaseModifiers["hp"].Should().BeApproximately(0.85f, 0.001f);
            swarm.BaseModifiers["damage"].Should().BeApproximately(0.90f, 0.001f);
        }

        [Fact]
        public void Asymmetric_HasCorrectModifiers()
        {
            FactionArchetype asym = _registry.GetArchetype("asymmetric");
            asym.BaseModifiers["damage"].Should().BeApproximately(1.25f, 0.001f);
            asym.BaseModifiers["speed"].Should().BeApproximately(1.15f, 0.001f);
            asym.BaseModifiers["cost"].Should().BeApproximately(1.30f, 0.001f);
            asym.BaseModifiers["squad_size"].Should().BeApproximately(0.80f, 0.001f);
        }

        [Fact]
        public void All_ReturnsThreeCanonicalArchetypes()
        {
            _registry.All.Should().HaveCount(3);
        }

        [Fact]
        public void GetArchetype_ThrowsForUnknown()
        {
            Assert.Throws<KeyNotFoundException>(() => _registry.GetArchetype("nonexistent"));
        }
    }

    public class DoctrineTests
    {
        private readonly DoctrineEngine _engine = new DoctrineEngine();

        [Fact]
        public void ApplyDoctrine_ModifiesStatsCorrectly()
        {
            var baseStats = new UnitStats
            {
                Hp = 100f,
                Damage = 20f,
                Armor = 10f,
                Speed = 5f,
                Accuracy = 0.7f,
                FireRate = 1.0f,
                Morale = 100f,
                Cost = new ResourceCost { Food = 50, Wood = 30 }
            };

            var blitzkrieg = new DoctrineDefinition
            {
                Id = "blitzkrieg",
                DisplayName = "Blitzkrieg",
                Modifiers = new Dictionary<string, float>
                {
                    { "speed", 1.20f },
                    { "damage", 1.15f },
                    { "armor", 0.90f }
                }
            };

            UnitStats result = _engine.ApplyDoctrine(baseStats, blitzkrieg);

            result.Speed.Should().BeApproximately(6.0f, 0.01f);
            result.Damage.Should().BeApproximately(23.0f, 0.01f);
            result.Armor.Should().BeApproximately(9.0f, 0.01f);
            // Unchanged stats
            result.Hp.Should().BeApproximately(100f, 0.01f);
            result.Accuracy.Should().BeApproximately(0.7f, 0.01f);
        }

        [Fact]
        public void ApplyDoctrine_DoesNotMutateInput()
        {
            var baseStats = new UnitStats { Hp = 100f, Damage = 20f };
            var doctrine = new DoctrineDefinition
            {
                Id = "test",
                DisplayName = "Test",
                Modifiers = new Dictionary<string, float> { { "hp", 0.5f } }
            };

            _engine.ApplyDoctrine(baseStats, doctrine);

            baseStats.Hp.Should().Be(100f);
        }

        [Fact]
        public void ApplyAll_AppliesArchetypeThenDoctrine()
        {
            var baseStats = new UnitStats
            {
                Hp = 100f,
                Damage = 20f,
                Armor = 10f,
                Speed = 5f,
                Cost = new ResourceCost { Food = 100 }
            };

            var archetype = new FactionArchetype(
                "order", "Order", "Test",
                new Dictionary<string, float> { { "armor", 1.10f } });

            var doctrine = new DoctrineDefinition
            {
                Id = "turtle",
                DisplayName = "Turtle",
                Modifiers = new Dictionary<string, float> { { "armor", 1.25f } }
            };

            UnitStats result = _engine.ApplyAll(baseStats, archetype, doctrine);

            // 10 * 1.10 * 1.25 = 13.75
            result.Armor.Should().BeApproximately(13.75f, 0.01f);
        }

        [Fact]
        public void ApplyAll_WithNullDoctrine_OnlyAppliesArchetype()
        {
            var baseStats = new UnitStats { Armor = 10f };
            var archetype = new FactionArchetype(
                "order", "Order", "Test",
                new Dictionary<string, float> { { "armor", 1.10f } });

            UnitStats result = _engine.ApplyAll(baseStats, archetype, null);

            result.Armor.Should().BeApproximately(11.0f, 0.01f);
        }

        [Fact]
        public void ValidateDoctrine_ReportsNegativeModifiers()
        {
            var doctrine = new DoctrineDefinition
            {
                Id = "bad",
                DisplayName = "Bad",
                Modifiers = new Dictionary<string, float> { { "hp", -0.5f } }
            };

            IReadOnlyList<string> errors = _engine.ValidateDoctrine(doctrine);
            errors.Should().ContainSingle(e => e.Contains("negative"));
        }

        [Fact]
        public void ValidateDoctrine_AcceptsValidModifiers()
        {
            var doctrine = new DoctrineDefinition
            {
                Id = "good",
                DisplayName = "Good",
                Modifiers = new Dictionary<string, float>
                {
                    { "speed", 1.2f },
                    { "damage", 0.9f }
                }
            };

            IReadOnlyList<string> errors = _engine.ValidateDoctrine(doctrine);
            errors.Should().BeEmpty();
        }

        [Fact]
        public void ValidateDoctrine_ReportsExtremeValues()
        {
            var doctrine = new DoctrineDefinition
            {
                Id = "extreme",
                DisplayName = "Extreme",
                Modifiers = new Dictionary<string, float> { { "damage", 15f } }
            };

            IReadOnlyList<string> errors = _engine.ValidateDoctrine(doctrine);
            errors.Should().ContainSingle(e => e.Contains("extreme"));
        }
    }

    public class RoleValidatorTests
    {
        private readonly UnitRoleValidator _validator = new UnitRoleValidator();

        [Fact]
        public void RequiredRoles_HasCorrectCount()
        {
            UnitRoleValidator.RequiredRoles.Should().HaveCount(11);
        }

        [Fact]
        public void ValidateRoster_CompleteRoster_Passes()
        {
            IRegistry<UnitDefinition> units = CreateUnitsRegistry(
                "militia", "line", "elite", "at", "support",
                "scout", "jeep", "tank", "arty", "hero", "special");

            FactionDefinition faction = CreateFaction(
                cheapInfantry: "militia",
                lineInfantry: "line",
                eliteInfantry: "elite",
                antiArmor: "at",
                supportWeapon: "support",
                recon: "scout",
                lightVehicle: "jeep",
                heavyVehicle: "tank",
                artillery: "arty",
                heroCommander: "hero",
                spikeUnit: "special");

            RosterValidationResult result = _validator.ValidateRoster(faction, units);

            result.IsComplete.Should().BeTrue();
            result.MissingRoles.Should().BeEmpty();
            result.FilledRoles.Should().HaveCount(11);
        }

        [Fact]
        public void ValidateRoster_MissingRoles_Detected()
        {
            IRegistry<UnitDefinition> units = CreateUnitsRegistry("militia", "line");

            FactionDefinition faction = CreateFaction(
                cheapInfantry: "militia",
                lineInfantry: "line");

            RosterValidationResult result = _validator.ValidateRoster(faction, units);

            result.IsComplete.Should().BeFalse();
            result.MissingRoles.Should().Contain("elite_infantry");
            result.MissingRoles.Should().Contain("artillery");
            result.FilledRoles.Should().HaveCount(2);
        }

        [Fact]
        public void ValidateRoster_UnitNotInRegistry_TreatedAsMissing()
        {
            IRegistry<UnitDefinition> units = CreateUnitsRegistry("militia");

            FactionDefinition faction = CreateFaction(
                cheapInfantry: "militia",
                lineInfantry: "nonexistent_unit");

            RosterValidationResult result = _validator.ValidateRoster(faction, units);

            result.MissingRoles.Should().Contain("line_infantry");
        }

        [Fact]
        public void ValidateRoster_EmptyRoster_AllMissing()
        {
            IRegistry<UnitDefinition> units = new Registry<UnitDefinition>();
            FactionDefinition faction = CreateFaction();

            RosterValidationResult result = _validator.ValidateRoster(faction, units);

            result.IsComplete.Should().BeFalse();
            result.MissingRoles.Should().HaveCount(11);
        }

        private static IRegistry<UnitDefinition> CreateUnitsRegistry(params string[] unitIds)
        {
            var registry = new Registry<UnitDefinition>();
            foreach (string id in unitIds)
            {
                registry.Register(id, new UnitDefinition { Id = id }, RegistrySource.Pack, "test-pack");
            }
            return registry;
        }

        private static FactionDefinition CreateFaction(
            string? cheapInfantry = null,
            string? lineInfantry = null,
            string? eliteInfantry = null,
            string? antiArmor = null,
            string? supportWeapon = null,
            string? recon = null,
            string? lightVehicle = null,
            string? heavyVehicle = null,
            string? artillery = null,
            string? heroCommander = null,
            string? spikeUnit = null)
        {
            return new FactionDefinition
            {
                Faction = new FactionInfo { Id = "test_faction" },
                Roster = new FactionRoster
                {
                    CheapInfantry = cheapInfantry,
                    LineInfantry = lineInfantry,
                    EliteInfantry = eliteInfantry,
                    AntiArmor = antiArmor,
                    SupportWeapon = supportWeapon,
                    Recon = recon,
                    LightVehicle = lightVehicle,
                    HeavyVehicle = heavyVehicle,
                    Artillery = artillery,
                    HeroCommander = heroCommander,
                    SpikeUnit = spikeUnit
                }
            };
        }
    }

    public class WaveComposerTests
    {
        private readonly WaveComposer _composer = new WaveComposer();

        [Fact]
        public void ComposeWaves_GeneratesCorrectCount()
        {
            FactionDefinition faction = CreateFactionWithUnits(out IRegistry<UnitDefinition> units);

            IReadOnlyList<WaveDefinition> waves = _composer.ComposeWaves(faction, units, 5);

            waves.Should().HaveCount(5);
        }

        [Fact]
        public void ComposeWaves_FinalWaveMarkedCorrectly()
        {
            FactionDefinition faction = CreateFactionWithUnits(out IRegistry<UnitDefinition> units);

            IReadOnlyList<WaveDefinition> waves = _composer.ComposeWaves(faction, units, 3);

            waves[0].IsFinalWave.Should().BeFalse();
            waves[1].IsFinalWave.Should().BeFalse();
            waves[2].IsFinalWave.Should().BeTrue();
        }

        [Fact]
        public void ComposeWaves_WaveNumbersAreSequential()
        {
            FactionDefinition faction = CreateFactionWithUnits(out IRegistry<UnitDefinition> units);

            IReadOnlyList<WaveDefinition> waves = _composer.ComposeWaves(faction, units, 4);

            for (int i = 0; i < waves.Count; i++)
            {
                waves[i].WaveNumber.Should().Be(i + 1);
            }
        }

        [Fact]
        public void ComposeWaves_DifficultyMultiplierAffectsSpawnCounts()
        {
            FactionDefinition faction = CreateFactionWithUnits(out IRegistry<UnitDefinition> units);

            IReadOnlyList<WaveDefinition> normalWaves = _composer.ComposeWaves(faction, units, 3, 1.0f);
            IReadOnlyList<WaveDefinition> hardWaves = _composer.ComposeWaves(faction, units, 3, 2.0f);

            int normalTotal = normalWaves.Last().SpawnGroups.Sum(g => g.Count);
            int hardTotal = hardWaves.Last().SpawnGroups.Sum(g => g.Count);

            hardTotal.Should().BeGreaterThan(normalTotal);
        }

        [Fact]
        public void ScaleWave_AppliesMultiplier()
        {
            var wave = new WaveDefinition
            {
                Id = "test_wave",
                WaveNumber = 1,
                SpawnGroups = new List<SpawnGroup>
                {
                    new SpawnGroup { UnitId = "unit_a", Count = 10 },
                    new SpawnGroup { UnitId = "unit_b", Count = 5 }
                }
            };

            WaveDefinition scaled = _composer.ScaleWave(wave, 2.0f);

            scaled.SpawnGroups[0].Count.Should().Be(20);
            scaled.SpawnGroups[1].Count.Should().Be(10);
        }

        [Fact]
        public void ScaleWave_MinimumCountIsOne()
        {
            var wave = new WaveDefinition
            {
                Id = "test_wave",
                WaveNumber = 1,
                SpawnGroups = new List<SpawnGroup>
                {
                    new SpawnGroup { UnitId = "unit_a", Count = 1 }
                }
            };

            WaveDefinition scaled = _composer.ScaleWave(wave, 0.1f);

            scaled.SpawnGroups[0].Count.Should().BeGreaterOrEqualTo(1);
        }

        private static FactionDefinition CreateFactionWithUnits(out IRegistry<UnitDefinition> units)
        {
            var registry = new Registry<UnitDefinition>();
            registry.Register("militia", new UnitDefinition { Id = "militia", FactionId = "test", Tier = 1 },
                RegistrySource.Pack, "test-pack");
            registry.Register("infantry", new UnitDefinition { Id = "infantry", FactionId = "test", Tier = 2 },
                RegistrySource.Pack, "test-pack");
            registry.Register("elite", new UnitDefinition { Id = "elite", FactionId = "test", Tier = 3 },
                RegistrySource.Pack, "test-pack");
            units = registry;

            return new FactionDefinition
            {
                Faction = new FactionInfo { Id = "test" }
            };
        }
    }

    public class BalanceCalculatorTests
    {
        private readonly BalanceCalculator _calculator;
        private readonly ArchetypeRegistry _archetypes = new ArchetypeRegistry();

        public BalanceCalculatorTests()
        {
            _calculator = new BalanceCalculator(new DoctrineEngine());
        }

        [Fact]
        public void CalculatePowerRating_ReturnsPositiveValue()
        {
            var unit = new UnitDefinition
            {
                Id = "trooper",
                Stats = new UnitStats
                {
                    Hp = 100f,
                    Damage = 20f,
                    Armor = 10f,
                    Speed = 5f,
                    Accuracy = 0.7f,
                    FireRate = 1.0f,
                    Cost = new ResourceCost { Food = 50 }
                }
            };

            FactionArchetype order = _archetypes.GetArchetype("order");
            float power = _calculator.CalculatePowerRating(unit, order, null);

            power.Should().BeGreaterThan(0f);
        }

        [Fact]
        public void CalculatePowerRating_HigherStatsProduceHigherPower()
        {
            FactionArchetype order = _archetypes.GetArchetype("order");

            var weak = new UnitDefinition
            {
                Id = "weak",
                Stats = new UnitStats { Hp = 50f, Damage = 5f, Armor = 0f, Speed = 3f, Cost = new ResourceCost { Food = 50 } }
            };

            var strong = new UnitDefinition
            {
                Id = "strong",
                Stats = new UnitStats { Hp = 200f, Damage = 30f, Armor = 20f, Speed = 5f, Cost = new ResourceCost { Food = 50 } }
            };

            float weakPower = _calculator.CalculatePowerRating(weak, order, null);
            float strongPower = _calculator.CalculatePowerRating(strong, order, null);

            strongPower.Should().BeGreaterThan(weakPower);
        }

        [Fact]
        public void CalculateFactionPower_AggregatesAllFactionUnits()
        {
            var registry = new Registry<UnitDefinition>();
            registry.Register("u1", new UnitDefinition
            {
                Id = "u1",
                FactionId = "alpha",
                Stats = new UnitStats { Hp = 100, Damage = 10, Speed = 5, Cost = new ResourceCost { Food = 30 } }
            }, RegistrySource.Pack, "test");

            registry.Register("u2", new UnitDefinition
            {
                Id = "u2",
                FactionId = "alpha",
                Stats = new UnitStats { Hp = 150, Damage = 15, Speed = 4, Cost = new ResourceCost { Food = 40 } }
            }, RegistrySource.Pack, "test");

            // Different faction, should not be included
            registry.Register("u3", new UnitDefinition
            {
                Id = "u3",
                FactionId = "beta",
                Stats = new UnitStats { Hp = 200, Damage = 20, Speed = 3, Cost = new ResourceCost { Food = 60 } }
            }, RegistrySource.Pack, "test");

            var faction = new FactionDefinition { Faction = new FactionInfo { Id = "alpha" } };
            FactionArchetype order = _archetypes.GetArchetype("order");

            FactionPowerReport report = _calculator.CalculateFactionPower(faction, registry, order, null);

            report.UnitCount.Should().Be(2);
            report.TotalPower.Should().BeGreaterThan(0f);
            report.UnitPowerRatings.Should().ContainKey("u1");
            report.UnitPowerRatings.Should().ContainKey("u2");
            report.UnitPowerRatings.Should().NotContainKey("u3");
        }

        [Fact]
        public void CompareFactions_BalancedFactions_ReportsBalanced()
        {
            var report1 = new FactionPowerReport("a", "order", null, 100f, 50f, 2, new Dictionary<string, float>());
            var report2 = new FactionPowerReport("b", "order", null, 105f, 52.5f, 2, new Dictionary<string, float>());

            BalanceComparisonReport result = _calculator.CompareFactions(report1, report2);

            result.Assessment.Should().Be("balanced");
            result.StrongerFaction.Should().BeNull();
        }

        [Fact]
        public void CompareFactions_ImbalancedFactions_ReportsAdvantage()
        {
            var report1 = new FactionPowerReport("a", "order", null, 200f, 100f, 2, new Dictionary<string, float>());
            var report2 = new FactionPowerReport("b", "order", null, 100f, 50f, 2, new Dictionary<string, float>());

            BalanceComparisonReport result = _calculator.CompareFactions(report1, report2);

            result.Assessment.Should().NotBe("balanced");
            result.StrongerFaction.Should().Be("a");
            result.PowerDelta.Should().BeGreaterThan(0f);
        }
    }

    public class WarfarePluginIntegrationTests
    {
        [Fact]
        public void ValidatePack_WithValidContent_ReturnsValid()
        {
            var registries = new RegistryManager();

            // Register units
            string[] unitIds = { "militia", "line", "elite", "at", "support", "scout",
                                 "jeep", "tank", "arty", "hero", "special" };
            foreach (string id in unitIds)
            {
                registries.Units.Register(id, new UnitDefinition { Id = id, FactionId = "republic" },
                    RegistrySource.Pack, "test-pack");
            }

            // Register faction with complete roster
            var faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "republic", Archetype = "order" },
                Roster = new FactionRoster
                {
                    CheapInfantry = "militia",
                    LineInfantry = "line",
                    EliteInfantry = "elite",
                    AntiArmor = "at",
                    SupportWeapon = "support",
                    Recon = "scout",
                    LightVehicle = "jeep",
                    HeavyVehicle = "tank",
                    Artillery = "arty",
                    HeroCommander = "hero",
                    SpikeUnit = "special"
                }
            };
            registries.Factions.Register("republic", faction, RegistrySource.Pack, "test-pack");

            // Register a valid doctrine
            registries.Doctrines.Register("elite_discipline",
                new DoctrineDefinition
                {
                    Id = "elite_discipline",
                    DisplayName = "Elite Discipline",
                    Modifiers = new Dictionary<string, float> { { "armor", 1.1f }, { "morale", 1.15f } }
                },
                RegistrySource.Pack, "test-pack");

            // Register a wave referencing valid units
            registries.Waves.Register("wave_1",
                new WaveDefinition
                {
                    Id = "wave_1",
                    WaveNumber = 1,
                    SpawnGroups = new List<SpawnGroup>
                    {
                        new SpawnGroup { UnitId = "militia", Count = 10 }
                    }
                },
                RegistrySource.Pack, "test-pack");

            var plugin = new WarfarePlugin(registries);
            WarfareValidationResult result = plugin.ValidatePack("test-pack", registries);

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void ValidatePack_WithInvalidWaveUnit_ReportsError()
        {
            var registries = new RegistryManager();

            registries.Waves.Register("bad_wave",
                new WaveDefinition
                {
                    Id = "bad_wave",
                    WaveNumber = 1,
                    SpawnGroups = new List<SpawnGroup>
                    {
                        new SpawnGroup { UnitId = "nonexistent", Count = 5 }
                    }
                },
                RegistrySource.Pack, "my-pack");

            var plugin = new WarfarePlugin(registries);
            WarfareValidationResult result = plugin.ValidatePack("my-pack", registries);

            result.IsValid.Should().BeFalse();
            result.Errors.Should().ContainSingle(e => e.Contains("nonexistent"));
        }

        [Fact]
        public void WarfarePlugin_AllSubsystems_Initialized()
        {
            var plugin = new WarfarePlugin(new RegistryManager());

            plugin.Archetypes.Should().NotBeNull();
            plugin.Doctrines.Should().NotBeNull();
            plugin.RoleValidator.Should().NotBeNull();
            plugin.WaveComposer.Should().NotBeNull();
            plugin.Balance.Should().NotBeNull();
        }
    }
}
