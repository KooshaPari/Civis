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
    // ── Doctrine Engine Comprehensive ─────────────────────────────────────

    public class DoctrineComprehensiveTests
    {
        private readonly DoctrineEngine _engine = new DoctrineEngine();
        private readonly ArchetypeRegistry _archetypes = new ArchetypeRegistry();

        private static readonly DoctrineDefinition[] PredefinedDoctrines = new[]
        {
            new DoctrineDefinition
            {
                Id = "elite_discipline", DisplayName = "Elite Discipline",
                Modifiers = new Dictionary<string, float> { { "armor", 1.10f }, { "morale", 1.15f }, { "speed", 0.95f } }
            },
            new DoctrineDefinition
            {
                Id = "mechanized_attrition", DisplayName = "Mechanized Attrition",
                Modifiers = new Dictionary<string, float> { { "hp", 1.20f }, { "fire_rate", 1.10f }, { "cost", 1.15f } }
            },
            new DoctrineDefinition
            {
                Id = "blitzkrieg", DisplayName = "Blitzkrieg",
                Modifiers = new Dictionary<string, float> { { "speed", 1.30f }, { "damage", 1.15f }, { "armor", 0.85f } }
            },
            new DoctrineDefinition
            {
                Id = "guerrilla_tactics", DisplayName = "Guerrilla Tactics",
                Modifiers = new Dictionary<string, float> { { "speed", 1.20f }, { "accuracy", 1.10f }, { "hp", 0.80f } }
            },
            new DoctrineDefinition
            {
                Id = "fortification", DisplayName = "Fortification",
                Modifiers = new Dictionary<string, float> { { "armor", 1.30f }, { "hp", 1.15f }, { "speed", 0.70f } }
            }
        };

        private static UnitStats CreateBaseStats()
        {
            return new UnitStats
            {
                Hp = 100f,
                Damage = 20f,
                Armor = 10f,
                Speed = 5f,
                Accuracy = 0.7f,
                FireRate = 1.0f,
                Morale = 100f,
                Cost = new ResourceCost { Food = 50, Wood = 30, Iron = 10 }
            };
        }

        [Theory]
        [InlineData(0)]
        [InlineData(1)]
        [InlineData(2)]
        [InlineData(3)]
        [InlineData(4)]
        public void ApplyDoctrine_AllPredefined_NoNegativeStats(int doctrineIndex)
        {
            UnitStats baseStats = CreateBaseStats();
            DoctrineDefinition doctrine = PredefinedDoctrines[doctrineIndex];

            UnitStats result = _engine.ApplyDoctrine(baseStats, doctrine);

            result.Hp.Should().BeGreaterThan(0f, "HP should never be negative after doctrine");
            result.Damage.Should().BeGreaterOrEqualTo(0f, "damage should never be negative after doctrine");
            result.Armor.Should().BeGreaterOrEqualTo(0f, "armor should never be negative after doctrine");
            result.Speed.Should().BeGreaterOrEqualTo(0f, "speed should never be negative after doctrine");
            result.Accuracy.Should().BeGreaterOrEqualTo(0f, "accuracy should never be negative");
            result.FireRate.Should().BeGreaterThan(0f, "fire rate should never be zero or negative");
            result.Morale.Should().BeGreaterOrEqualTo(0f, "morale should never be negative");
        }

        [Fact]
        public void ApplyDoctrine_MultiplyByZero_ClampsToMinimum()
        {
            UnitStats baseStats = CreateBaseStats();
            var zeroDoctrine = new DoctrineDefinition
            {
                Id = "zero",
                DisplayName = "Zero",
                Modifiers = new Dictionary<string, float>
                {
                    { "hp", 0f }, { "damage", 0f }, { "armor", 0f },
                    { "speed", 0f }, { "accuracy", 0f }, { "morale", 0f }
                }
            };

            UnitStats result = _engine.ApplyDoctrine(baseStats, zeroDoctrine);

            result.Hp.Should().BeGreaterOrEqualTo(1f, "HP is clamped to minimum 1");
            result.Damage.Should().BeGreaterOrEqualTo(0f);
            result.Armor.Should().BeGreaterOrEqualTo(0f);
            result.Speed.Should().BeGreaterOrEqualTo(0f);
            result.Accuracy.Should().BeGreaterOrEqualTo(0f);
            result.FireRate.Should().BeGreaterThan(0f, "fire rate clamped to minimum 0.01");
        }

        [Fact]
        public void ApplyDoctrine_MultiplyByTen_ProducesLargeButValidStats()
        {
            UnitStats baseStats = CreateBaseStats();
            var tenxDoctrine = new DoctrineDefinition
            {
                Id = "tenx",
                DisplayName = "10x",
                Modifiers = new Dictionary<string, float>
                {
                    { "hp", 10f }, { "damage", 10f }, { "speed", 10f }
                }
            };

            UnitStats result = _engine.ApplyDoctrine(baseStats, tenxDoctrine);

            result.Hp.Should().BeApproximately(1000f, 1f);
            result.Damage.Should().BeApproximately(200f, 1f);
            result.Speed.Should().BeApproximately(50f, 1f);
        }

        [Theory]
        [InlineData("order", 0)]
        [InlineData("order", 1)]
        [InlineData("order", 2)]
        [InlineData("order", 3)]
        [InlineData("order", 4)]
        [InlineData("industrial_swarm", 0)]
        [InlineData("industrial_swarm", 1)]
        [InlineData("industrial_swarm", 2)]
        [InlineData("industrial_swarm", 3)]
        [InlineData("industrial_swarm", 4)]
        [InlineData("asymmetric", 0)]
        [InlineData("asymmetric", 1)]
        [InlineData("asymmetric", 2)]
        [InlineData("asymmetric", 3)]
        [InlineData("asymmetric", 4)]
        public void ApplyAll_ArchetypePlusDoctrine_AllCombinationsProduceValidStats(string archetypeId, int doctrineIndex)
        {
            UnitStats baseStats = CreateBaseStats();
            FactionArchetype archetype = _archetypes.GetArchetype(archetypeId);
            DoctrineDefinition doctrine = PredefinedDoctrines[doctrineIndex];

            UnitStats result = _engine.ApplyAll(baseStats, archetype, doctrine);

            result.Hp.Should().BeGreaterThan(0f);
            result.Damage.Should().BeGreaterOrEqualTo(0f);
            result.Armor.Should().BeGreaterOrEqualTo(0f);
            result.Speed.Should().BeGreaterOrEqualTo(0f);
            result.Accuracy.Should().BeInRange(0f, 1f);
            result.FireRate.Should().BeGreaterThan(0f);
        }

        [Fact]
        public void ApplyDoctrine_UnknownModifierKey_SilentlyIgnored()
        {
            UnitStats baseStats = CreateBaseStats();
            var doctrine = new DoctrineDefinition
            {
                Id = "unknown",
                DisplayName = "Unknown",
                Modifiers = new Dictionary<string, float> { { "unknown_stat", 2.0f } }
            };

            UnitStats result = _engine.ApplyDoctrine(baseStats, doctrine);

            // All stats should remain unchanged
            result.Hp.Should().Be(baseStats.Hp);
            result.Damage.Should().Be(baseStats.Damage);
        }

        [Fact]
        public void ApplyDoctrine_EmptyModifiers_ReturnsUnchangedStats()
        {
            UnitStats baseStats = CreateBaseStats();
            var doctrine = new DoctrineDefinition
            {
                Id = "empty",
                DisplayName = "Empty",
                Modifiers = new Dictionary<string, float>()
            };

            UnitStats result = _engine.ApplyDoctrine(baseStats, doctrine);

            result.Hp.Should().Be(baseStats.Hp);
            result.Damage.Should().Be(baseStats.Damage);
            result.Armor.Should().Be(baseStats.Armor);
            result.Speed.Should().Be(baseStats.Speed);
        }

        [Fact]
        public void ApplyDoctrine_CostModifier_AffectsAllResources()
        {
            var baseStats = new UnitStats
            {
                Hp = 100f,
                Cost = new ResourceCost { Food = 100, Wood = 50, Stone = 30, Iron = 20, Gold = 10 }
            };
            var doctrine = new DoctrineDefinition
            {
                Id = "cheap",
                DisplayName = "Cheap",
                Modifiers = new Dictionary<string, float> { { "cost", 0.5f } }
            };

            UnitStats result = _engine.ApplyDoctrine(baseStats, doctrine);

            result.Cost.Food.Should().Be(50);
            result.Cost.Wood.Should().Be(25);
            result.Cost.Stone.Should().Be(15);
            result.Cost.Iron.Should().Be(10);
            result.Cost.Gold.Should().Be(5);
        }
    }

    // ── Balance Calculator Comprehensive ──────────────────────────────────

    public class BalanceComprehensiveTests
    {
        private readonly BalanceCalculator _calculator;
        private readonly ArchetypeRegistry _archetypes = new ArchetypeRegistry();

        public BalanceComprehensiveTests()
        {
            _calculator = new BalanceCalculator(new DoctrineEngine());
        }

        [Fact]
        public void CompareFactions_SymmetricFactions_RatioNearOne()
        {
            var report1 = new FactionPowerReport("alpha", "order", null, 100f, 50f, 2, new Dictionary<string, float>());
            var report2 = new FactionPowerReport("beta", "order", null, 100f, 50f, 2, new Dictionary<string, float>());

            BalanceComparisonReport result = _calculator.CompareFactions(report1, report2);

            result.PowerRatio.Should().BeApproximately(1.0f, 0.001f);
            result.Assessment.Should().Be("balanced");
            result.StrongerFaction.Should().BeNull();
        }

        [Fact]
        public void CompareFactions_ExtremeStatDifference_SignificantAdvantage()
        {
            var report1 = new FactionPowerReport("alpha", "order", null, 1000f, 500f, 2, new Dictionary<string, float>());
            var report2 = new FactionPowerReport("beta", "order", null, 100f, 50f, 2, new Dictionary<string, float>());

            BalanceComparisonReport result = _calculator.CompareFactions(report1, report2);

            result.Assessment.Should().Be("significant_advantage");
            result.StrongerFaction.Should().Be("alpha");
            result.PowerDelta.Should().BeApproximately(900f, 1f);
        }

        [Fact]
        public void CompareFactions_SlightAdvantage_Detected()
        {
            // Ratio 1.15 should be slight advantage (within 0.10-0.25 range)
            var report1 = new FactionPowerReport("alpha", "order", null, 115f, 57.5f, 2, new Dictionary<string, float>());
            var report2 = new FactionPowerReport("beta", "order", null, 100f, 50f, 2, new Dictionary<string, float>());

            BalanceComparisonReport result = _calculator.CompareFactions(report1, report2);

            result.Assessment.Should().Be("slight_advantage");
            result.StrongerFaction.Should().Be("alpha");
        }

        [Fact]
        public void CompareFactions_Faction2Stronger_CorrectlyIdentified()
        {
            var report1 = new FactionPowerReport("alpha", "order", null, 50f, 25f, 2, new Dictionary<string, float>());
            var report2 = new FactionPowerReport("beta", "order", null, 200f, 100f, 2, new Dictionary<string, float>());

            BalanceComparisonReport result = _calculator.CompareFactions(report1, report2);

            result.StrongerFaction.Should().Be("beta");
        }

        [Fact]
        public void CalculatePowerRating_ZeroCost_NoException()
        {
            var unit = new UnitDefinition
            {
                Id = "free-unit",
                Stats = new UnitStats
                {
                    Hp = 100f,
                    Damage = 10f,
                    Speed = 5f,
                    Cost = new ResourceCost() // all zero
                }
            };
            FactionArchetype order = _archetypes.GetArchetype("order");

            float power = _calculator.CalculatePowerRating(unit, order, null);

            power.Should().BeGreaterThan(0f, "zero-cost unit should still have positive power");
        }

        [Fact]
        public void CalculatePowerRating_WithDoctrine_DifferentFromWithout()
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
            var doctrine = new DoctrineDefinition
            {
                Id = "blitz",
                DisplayName = "Blitz",
                Modifiers = new Dictionary<string, float> { { "damage", 1.5f } }
            };

            float withoutDoctrine = _calculator.CalculatePowerRating(unit, order, null);
            float withDoctrine = _calculator.CalculatePowerRating(unit, order, doctrine);

            withDoctrine.Should().BeGreaterThan(withoutDoctrine, "doctrine adds damage so power should increase");
        }
    }

    // ── Wave Composer Comprehensive ───────────────────────────────────────

    public class WaveComposerComprehensiveTests
    {
        private readonly WaveComposer _composer = new WaveComposer();

        [Fact]
        public void ComposeWaves_50Waves_MonotonicDifficultyIncrease()
        {
            FactionDefinition faction = CreateFactionWithTieredUnits(out IRegistry<UnitDefinition> units);

            IReadOnlyList<WaveDefinition> waves = _composer.ComposeWaves(faction, units, 50);

            waves.Should().HaveCount(50);

            // Check that total spawn count is non-decreasing
            for (int i = 1; i < waves.Count; i++)
            {
                int prevTotal = waves[i - 1].SpawnGroups.Sum(g => g.Count);
                int currTotal = waves[i].SpawnGroups.Sum(g => g.Count);
                currTotal.Should().BeGreaterOrEqualTo(prevTotal,
                    $"wave {i + 1} should have at least as many units as wave {i}");
            }
        }

        [Fact]
        public void ComposeWaves_NoFactionUnits_ReturnsEmptyWaves()
        {
            var faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "empty-faction" }
            };
            var registry = new Registry<UnitDefinition>();

            IReadOnlyList<WaveDefinition> waves = _composer.ComposeWaves(faction, registry, 5);

            waves.Should().HaveCount(5);
            foreach (WaveDefinition wave in waves)
            {
                wave.SpawnGroups.Should().BeEmpty();
            }
        }

        [Fact]
        public void ComposeWaves_SingleUnit_AllWavesUseIt()
        {
            var registry = new Registry<UnitDefinition>();
            registry.Register("soldier", new UnitDefinition { Id = "soldier", FactionId = "test", Tier = 1 },
                RegistrySource.Pack, "test");
            var faction = new FactionDefinition { Faction = new FactionInfo { Id = "test" } };

            IReadOnlyList<WaveDefinition> waves = _composer.ComposeWaves(faction, registry, 5);

            foreach (WaveDefinition wave in waves)
            {
                wave.SpawnGroups.Should().NotBeEmpty();
                wave.SpawnGroups[0].UnitId.Should().Be("soldier");
            }
        }

        [Fact]
        public void ComposeWaves_HighDifficulty_MoreUnitsThanNormal()
        {
            FactionDefinition faction = CreateFactionWithTieredUnits(out IRegistry<UnitDefinition> units);

            IReadOnlyList<WaveDefinition> normalWaves = _composer.ComposeWaves(faction, units, 10, 1.0f);
            IReadOnlyList<WaveDefinition> hardWaves = _composer.ComposeWaves(faction, units, 10, 3.0f);

            int normalLast = normalWaves.Last().SpawnGroups.Sum(g => g.Count);
            int hardLast = hardWaves.Last().SpawnGroups.Sum(g => g.Count);

            hardLast.Should().BeGreaterThan(normalLast);
        }

        [Fact]
        public void ScaleWave_ZeroMultiplier_MinCountIsOne()
        {
            var wave = new WaveDefinition
            {
                Id = "test",
                WaveNumber = 1,
                SpawnGroups = new List<SpawnGroup>
                {
                    new SpawnGroup { UnitId = "a", Count = 100 },
                    new SpawnGroup { UnitId = "b", Count = 50 }
                }
            };

            WaveDefinition scaled = _composer.ScaleWave(wave, 0.001f);

            foreach (SpawnGroup group in scaled.SpawnGroups)
            {
                group.Count.Should().BeGreaterOrEqualTo(1);
            }
        }

        [Fact]
        public void ScaleWave_PreservesMetadata()
        {
            var wave = new WaveDefinition
            {
                Id = "wave-7",
                DisplayName = "Wave Seven",
                Description = "The seventh wave",
                WaveNumber = 7,
                DelaySeconds = 420f,
                IsFinalWave = true,
                SpawnGroups = new List<SpawnGroup>
                {
                    new SpawnGroup { UnitId = "a", Count = 10, SpawnPoint = "north" }
                }
            };

            WaveDefinition scaled = _composer.ScaleWave(wave, 2.0f);

            scaled.Id.Should().Be("wave-7");
            scaled.DisplayName.Should().Be("Wave Seven");
            scaled.Description.Should().Be("The seventh wave");
            scaled.WaveNumber.Should().Be(7);
            scaled.DelaySeconds.Should().Be(420f);
            scaled.IsFinalWave.Should().BeTrue();
            scaled.SpawnGroups[0].SpawnPoint.Should().Be("north");
        }

        private static FactionDefinition CreateFactionWithTieredUnits(out IRegistry<UnitDefinition> units)
        {
            var registry = new Registry<UnitDefinition>();
            registry.Register("militia", new UnitDefinition { Id = "militia", FactionId = "test", Tier = 1 },
                RegistrySource.Pack, "test");
            registry.Register("soldier", new UnitDefinition { Id = "soldier", FactionId = "test", Tier = 1 },
                RegistrySource.Pack, "test");
            registry.Register("knight", new UnitDefinition { Id = "knight", FactionId = "test", Tier = 2 },
                RegistrySource.Pack, "test");
            registry.Register("champion", new UnitDefinition { Id = "champion", FactionId = "test", Tier = 3 },
                RegistrySource.Pack, "test");
            units = registry;
            return new FactionDefinition { Faction = new FactionInfo { Id = "test" } };
        }
    }

    // ── Unit Role Validator Comprehensive ─────────────────────────────────

    public class RoleValidatorComprehensiveTests
    {
        private readonly UnitRoleValidator _validator = new UnitRoleValidator();

        [Fact]
        public void ValidateRoster_OverfilledRoster_AllFilledRolesDetected()
        {
            // Create a roster where all 11 roles are filled plus extra units in registry
            var registry = new Registry<UnitDefinition>();
            string[] unitIds = { "m", "l", "e", "at", "sw", "sc", "lv", "hv", "art", "hero", "spike", "extra1", "extra2" };
            foreach (string id in unitIds)
            {
                registry.Register(id, new UnitDefinition { Id = id, FactionId = "test" }, RegistrySource.Pack, "test");
            }

            var faction = CreateFullFaction("m", "l", "e", "at", "sw", "sc", "lv", "hv", "art", "hero", "spike");

            RosterValidationResult result = _validator.ValidateRoster(faction, registry);

            result.IsComplete.Should().BeTrue();
            result.FilledRoles.Should().HaveCount(11);
            result.MissingRoles.Should().BeEmpty();
        }

        [Fact]
        public void ValidateRoster_PartialRoster_CorrectCountsReported()
        {
            var registry = new Registry<UnitDefinition>();
            registry.Register("m", new UnitDefinition { Id = "m" }, RegistrySource.Pack, "test");
            registry.Register("l", new UnitDefinition { Id = "l" }, RegistrySource.Pack, "test");
            registry.Register("e", new UnitDefinition { Id = "e" }, RegistrySource.Pack, "test");

            var faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "test" },
                Roster = new FactionRoster
                {
                    CheapInfantry = "m",
                    LineInfantry = "l",
                    EliteInfantry = "e"
                }
            };

            RosterValidationResult result = _validator.ValidateRoster(faction, registry);

            result.IsComplete.Should().BeFalse();
            result.FilledRoles.Should().HaveCount(3);
            result.MissingRoles.Should().HaveCount(8);
        }

        [Fact]
        public void ValidateRoster_RoleToUnitMap_ContainsCorrectMappings()
        {
            var registry = new Registry<UnitDefinition>();
            registry.Register("militia-unit", new UnitDefinition { Id = "militia-unit" }, RegistrySource.Pack, "test");
            registry.Register("line-unit", new UnitDefinition { Id = "line-unit" }, RegistrySource.Pack, "test");

            var faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "test" },
                Roster = new FactionRoster
                {
                    CheapInfantry = "militia-unit",
                    LineInfantry = "line-unit"
                }
            };

            RosterValidationResult result = _validator.ValidateRoster(faction, registry);

            result.RoleToUnitMap.Should().ContainKey("cheap_infantry");
            result.RoleToUnitMap["cheap_infantry"].Should().Be("militia-unit");
            result.RoleToUnitMap.Should().ContainKey("line_infantry");
            result.RoleToUnitMap["line_infantry"].Should().Be("line-unit");
        }

        private static FactionDefinition CreateFullFaction(
            string ci, string li, string ei, string aa, string sw,
            string sc, string lv, string hv, string art, string hero, string spike)
        {
            return new FactionDefinition
            {
                Faction = new FactionInfo { Id = "test" },
                Roster = new FactionRoster
                {
                    CheapInfantry = ci,
                    LineInfantry = li,
                    EliteInfantry = ei,
                    AntiArmor = aa,
                    SupportWeapon = sw,
                    Recon = sc,
                    LightVehicle = lv,
                    HeavyVehicle = hv,
                    Artillery = art,
                    HeroCommander = hero,
                    SpikeUnit = spike
                }
            };
        }
    }
}
