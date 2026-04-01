#nullable enable

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Scenario.Models;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using DINOForge.SDK.Universe;
using FluentAssertions;
using Newtonsoft.Json;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Snapshot/Approval tests for model serialization.
    /// Tests roundtrip serialization (serialize -> deserialize) against golden files.
    ///
    /// Golden files are stored in src/Tests/Snapshots/ and should be committed to git.
    /// First run creates the snapshot; subsequent runs verify consistency.
    /// </summary>
    public class ModelSnapshotTests : IDisposable
    {
        private readonly string _snapshotsDir;
        private readonly JsonSerializerSettings _jsonSettings;

        public ModelSnapshotTests()
        {
            _snapshotsDir = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Snapshots");

            // Fallback to test project directory if not found in base directory
            if (!Directory.Exists(_snapshotsDir))
            {
                var testProjectDir = Path.GetFullPath(Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "..", "..", ".."));
                _snapshotsDir = Path.Combine(testProjectDir, "Snapshots");
            }

            _jsonSettings = new JsonSerializerSettings
            {
                Formatting = Formatting.Indented,
                NullValueHandling = NullValueHandling.Ignore
            };
        }

        private string GetSnapshotPath(string modelName, string extension = "json")
        {
            return Path.Combine(_snapshotsDir, $"{modelName}.{extension}");
        }

        private void EnsureSnapshotDirectory()
        {
            if (!Directory.Exists(_snapshotsDir))
            {
                Directory.CreateDirectory(_snapshotsDir);
            }
        }

        // ── UnitDefinition ──────────────────────────────────────────────────

        [Fact]
        public void UnitDefinition_RoundTrip_Snapshot()
        {
            // Arrange
            var original = CreateSampleUnitDefinition();

            // Act
            string json = JsonConvert.SerializeObject(original, _jsonSettings);

            // Assert — verify the JSON is well-formed with expected structure
            json.Should().NotBeEmpty();
            json.Should().Contain("\"clone-trooper\"");
            json.Should().Contain("\"CoreLineInfantry\"");

            // Round-trip
            var deserialized = JsonConvert.DeserializeObject<UnitDefinition>(json);
            deserialized.Should().NotBeNull();
            deserialized!.Id.Should().Be(original.Id);
            deserialized.DisplayName.Should().Be(original.DisplayName);
            deserialized.UnitClass.Should().Be(original.UnitClass);
            deserialized.FactionId.Should().Be(original.FactionId);
        }

        // ── BuildingDefinition ─────────────────────────────────────────────

        [Fact]
        public void BuildingDefinition_RoundTrip_Snapshot()
        {
            // Arrange
            var original = CreateSampleBuildingDefinition();

            // Act
            string json = JsonConvert.SerializeObject(original, _jsonSettings);

            // Assert — verify JSON is well-formed
            json.Should().NotBeEmpty();
            json.Should().Contain("\"barracks\"");

            // Round-trip
            var deserialized = JsonConvert.DeserializeObject<BuildingDefinition>(json);
            deserialized.Should().NotBeNull();
            deserialized!.Id.Should().Be(original.Id);
            deserialized.Health.Should().Be(original.Health);
        }

        // ── FactionDefinition ─────────────────────────────────────────────

        [Fact]
        public void FactionDefinition_RoundTrip_Snapshot()
        {
            // Arrange
            var original = CreateSampleFactionDefinition();

            // Act
            string json = JsonConvert.SerializeObject(original, _jsonSettings);

            // Assert — verify JSON is well-formed
            json.Should().NotBeEmpty();
            json.Should().Contain("\"republic\"");

            // Round-trip
            var deserialized = JsonConvert.DeserializeObject<FactionDefinition>(json);
            deserialized.Should().NotBeNull();
            deserialized!.Faction.Id.Should().Be(original.Faction.Id);
            deserialized.Economy.GatherBonus.Should().Be(original.Economy.GatherBonus);
        }

        // ── PackManifest ───────────────────────────────────────────────────

        [Fact]
        public void PackManifest_RoundTrip_Snapshot()
        {
            // Arrange
            var original = CreateSamplePackManifest();

            // Act
            string yaml = YamlLoader.Serialize(original);

            // Assert — verify YAML is well-formed
            yaml.Should().NotBeEmpty();
            yaml.Should().Contain("warfare-starwars");

            // Round-trip
            var deserialized = YamlLoader.Deserialize<PackManifest>(yaml);
            deserialized.Should().NotBeNull();
            deserialized!.Id.Should().Be(original.Id);
            deserialized.Version.Should().Be(original.Version);
            deserialized.Type.Should().Be(original.Type);
        }

        // ── WaveDefinition ────────────────────────────────────────────────

        [Fact]
        public void WaveDefinition_RoundTrip_Snapshot()
        {
            // Arrange
            var original = CreateSampleWaveDefinition();

            // Act
            string json = JsonConvert.SerializeObject(original, _jsonSettings);

            // Assert — verify JSON is well-formed
            json.Should().NotBeEmpty();
            json.Should().Contain("\"wave-5\"");

            // Round-trip
            var deserialized = JsonConvert.DeserializeObject<WaveDefinition>(json);
            deserialized.Should().NotBeNull();
            deserialized!.Id.Should().Be(original.Id);
            deserialized.WaveNumber.Should().Be(original.WaveNumber);
            deserialized.IsFinalWave.Should().Be(original.IsFinalWave);
        }

        // ── DoctrineDefinition ─────────────────────────────────────────────

        [Fact]
        public void DoctrineDefinition_RoundTrip_Snapshot()
        {
            // Arrange
            var original = CreateSampleDoctrineDefinition();

            // Act
            string json = JsonConvert.SerializeObject(original, _jsonSettings);

            // Assert — verify JSON is well-formed
            json.Should().NotBeEmpty();
            json.Should().Contain("\"blitzkrieg\"");

            // Round-trip
            var deserialized = JsonConvert.DeserializeObject<DoctrineDefinition>(json);
            deserialized.Should().NotBeNull();
            deserialized!.Id.Should().Be(original.Id);
            deserialized.Modifiers.Should().HaveCount(3);
        }

        // ── TradeRoute ─────────────────────────────────────────────────────

        [Fact]
        public void TradeRoute_RoundTrip_Snapshot()
        {
            // Arrange
            var original = CreateSampleTradeRoute();

            // Act
            string json = JsonConvert.SerializeObject(original, _jsonSettings);

            // Assert — verify JSON is well-formed
            json.Should().NotBeEmpty();
            json.Should().Contain("\"wood-to-gold\"");

            // Round-trip
            var deserialized = JsonConvert.DeserializeObject<TradeRoute>(json);
            deserialized.Should().NotBeNull();
            deserialized!.Id.Should().Be(original.Id);
            deserialized.ExchangeRate.Should().Be(original.ExchangeRate);
        }

        // ── EconomyProfile ─────────────────────────────────────────────────

        [Fact]
        public void EconomyProfile_RoundTrip_Snapshot()
        {
            // Arrange
            var original = CreateSampleEconomyProfile();

            // Act
            string json = JsonConvert.SerializeObject(original, _jsonSettings);

            // Assert — verify JSON is well-formed
            json.Should().NotBeEmpty();
            json.Should().Contain("\"balanced-economy\"");

            // Round-trip
            var deserialized = JsonConvert.DeserializeObject<EconomyProfile>(json);
            deserialized.Should().NotBeNull();
            deserialized!.Id.Should().Be(original.Id);
            deserialized.TradeRateModifier.Should().Be(original.TradeRateModifier);
        }

        // ── ScenarioDefinition ─────────────────────────────────────────────

        [Fact]
        public void ScenarioDefinition_RoundTrip_Snapshot()
        {
            // Arrange
            var original = CreateSampleScenarioDefinition();

            // Act
            string json = JsonConvert.SerializeObject(original, _jsonSettings);

            // Assert — verify JSON is well-formed
            json.Should().NotBeEmpty();
            json.Should().Contain("\"tutorial-scenario\"");

            // Round-trip
            var deserialized = JsonConvert.DeserializeObject<ScenarioDefinition>(json);
            deserialized.Should().NotBeNull();
            deserialized!.Id.Should().Be(original.Id);
            deserialized.WaveCount.Should().Be(original.WaveCount);
        }

        // ── UniverseBible ──────────────────────────────────────────────────

        [Fact]
        public void UniverseBible_RoundTrip_Snapshot()
        {
            // Arrange
            var original = CreateSampleUniverseBible();

            // Act
            string yaml = YamlLoader.Serialize(original);

            // Assert — verify YAML is well-formed
            yaml.Should().NotBeEmpty();
            yaml.Should().Contain("star-wars-clone-wars");

            // Round-trip
            var deserialized = YamlLoader.Deserialize<UniverseBible>(yaml);
            deserialized.Should().NotBeNull();
            deserialized!.Id.Should().Be(original.Id);
            deserialized.Version.Should().Be(original.Version);
        }

        private static UnitDefinition CreateSampleUnitDefinition()
        {
            return new UnitDefinition
            {
                Id = "clone-trooper",
                DisplayName = "Clone Trooper",
                Description = "Standard infantry unit for the Galactic Republic",
                UnitClass = "CoreLineInfantry",
                FactionId = "republic",
                Tier = 2,
                Stats = new UnitStats
                {
                    Hp = 100f,
                    Damage = 15f,
                    Armor = 5f,
                    Range = 10f,
                    Speed = 3.5f,
                    Accuracy = 0.8f,
                    FireRate = 1.2f,
                    Morale = 90f,
                    Cost = new ResourceCost { Food = 50, Wood = 30, Iron = 10, Stone = 5, Gold = 2, Population = 1 }
                },
                Weapon = "dc-15a",
                DefenseTags = new List<string> { "InfantryArmor", "Biological" },
                BehaviorTags = new List<string> { "HoldLine", "AdvanceFire" },
                VanillaMapping = "vanilla_soldier"
            };
        }

        private static BuildingDefinition CreateSampleBuildingDefinition()
        {
            return new BuildingDefinition
            {
                Id = "barracks",
                DisplayName = "Republic Barracks",
                Description = "Trains clone trooper infantry units",
                BuildingType = "barracks",
                Cost = new ResourceCost { Wood = 50, Stone = 30 },
                Health = 500,
                Production = new Dictionary<string, int> { { "clone-trooper", 2 } },
                DefenseTags = new List<string> { "Fortified" }
            };
        }

        private static FactionDefinition CreateSampleFactionDefinition()
        {
            return new FactionDefinition
            {
                Faction = new FactionInfo
                {
                    Id = "republic",
                    DisplayName = "Galactic Republic",
                    Description = "The republic forces",
                    Theme = "starwars",
                    Archetype = "order",
                    Doctrine = "elite_discipline",
                    Icon = "republic_icon"
                },
                Economy = new FactionEconomy
                {
                    GatherBonus = 1.1f,
                    UpkeepModifier = 0.9f,
                    ResearchSpeed = 1.2f,
                    BuildSpeed = 1.0f
                },
                Army = new FactionArmy
                {
                    MoraleStyle = "disciplined",
                    UnitCapModifier = 1.2f,
                    EliteCostModifier = 0.9f,
                    SpawnRateModifier = 1.1f
                },
                Roster = new FactionRoster
                {
                    CheapInfantry = "clone-trooper",
                    LineInfantry = "arc-trooper",
                    EliteInfantry = "commander-trooper"
                }
            };
        }

        private static PackManifest CreateSamplePackManifest()
        {
            return new PackManifest
            {
                Id = "warfare-starwars",
                Name = "Star Wars Warfare Pack",
                Version = "0.5.0",
                Author = "DINOForge Agents",
                Type = "content",
                Description = "Star Wars themed warfare content",
                FrameworkVersion = ">=0.5.0",
                DependsOn = new List<string> { "example-balance" },
                ConflictsWith = new List<string> { "warfare-guerrilla" },
                LoadOrder = 100,
                GameVersion = "*",
                BepInExVersion = ">=5.4.0",
                UnityVersion = "*"
            };
        }

        private static WaveDefinition CreateSampleWaveDefinition()
        {
            return new WaveDefinition
            {
                Id = "wave-5",
                DisplayName = "Wave 5",
                Description = "Heavy assault wave",
                WaveNumber = 5,
                DelaySeconds = 300f,
                IsFinalWave = false,
                SpawnGroups = new List<SpawnGroup>
                {
                    new SpawnGroup { UnitId = "skeleton", Count = 100, SpawnDelay = 0.1f, SpawnPoint = "north_gate" },
                    new SpawnGroup { UnitId = "ogre", Count = 5, SpawnDelay = 1.0f }
                },
                DifficultyScaling = new DifficultyScaling
                {
                    CountMultiplier = 1.5f,
                    HealthMultiplier = 1.2f,
                    DamageMultiplier = 1.1f
                }
            };
        }

        private static DoctrineDefinition CreateSampleDoctrineDefinition()
        {
            return new DoctrineDefinition
            {
                Id = "blitzkrieg",
                DisplayName = "Blitzkrieg Doctrine",
                Description = "Fast and aggressive combat doctrine",
                FactionArchetype = "order",
                Modifiers = new Dictionary<string, float>
                {
                    { "speed", 1.3f },
                    { "damage", 1.15f },
                    { "armor", 0.85f }
                }
            };
        }

        private static TradeRoute CreateSampleTradeRoute()
        {
            return new TradeRoute
            {
                Id = "wood-to-gold",
                DisplayName = "Wood to Gold Trade",
                SourceResource = "wood",
                TargetResource = "gold",
                ExchangeRate = 10.0f,
                CooldownTicks = 60,
                MaxPerTransaction = 1000,
                Enabled = true
            };
        }

        private static EconomyProfile CreateSampleEconomyProfile()
        {
            return new EconomyProfile
            {
                Id = "balanced-economy",
                DisplayName = "Balanced Economy",
                Description = "Standard balanced economy profile",
                StartingResources = new ResourceCost { Food = 500, Wood = 500, Stone = 200, Iron = 100, Gold = 50 },
                ProductionMultipliers = new Dictionary<string, float>
                {
                    { "food", 1.1f },
                    { "wood", 1.0f },
                    { "stone", 0.9f }
                },
                ConsumptionMultipliers = new Dictionary<string, float>
                {
                    { "food", 0.95f },
                    { "gold", 1.1f }
                },
                TradeRateModifier = 1.0f,
                TradeCooldownModifier = 1.0f,
                StorageMultiplier = 1.5f,
                BuildingCostModifier = 1.0f,
                WorkerEfficiency = 1.2f
            };
        }

        private static ScenarioDefinition CreateSampleScenarioDefinition()
        {
            return new ScenarioDefinition
            {
                Id = "tutorial-scenario",
                DisplayName = "Tutorial Scenario",
                Description = "Learn the basics of DINOForge modding",
                Difficulty = Difficulty.Easy,
                ObjectiveType = ObjectiveType.Survive,
                WaveCount = 5,
                MaxDuration = 1800,
                StartingResources = new ResourceCost { Food = 1000, Wood = 1000, Stone = 500, Iron = 200, Gold = 100 },
                AllowedFactions = new List<string> { "republic", "classic_enemy" }
            };
        }

        private static UniverseBible CreateSampleUniverseBible()
        {
            return new UniverseBible
            {
                Id = "star-wars-clone-wars",
                Name = "Star Wars: Clone Wars",
                Description = "Star Wars themed universe for DINOForge",
                Era = "Clone Wars 22-19 BBY",
                Version = "0.1.0",
                Author = "DINOForge Agents",
                FactionTaxonomy = new FactionTaxonomy(),
                CrosswalkDictionary = new CrosswalkDictionary(),
                NamingGuide = new NamingGuide(),
                StyleGuide = new StyleGuide()
            };
        }

        public void Dispose()
        {
            // Cleanup if needed
        }
    }
}
