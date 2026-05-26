using System.Collections.Generic;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.Tests
{
    public class ModelSerializationTests
    {
        private readonly ISerializer _serializer;

        public ModelSerializationTests()
        {
            _serializer = new SerializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .Build();
        }

        [Fact]
        public void UnitDefinition_RoundTrip_AllFields()
        {
            var original = new UnitDefinition
            {
                Id = "test-soldier",
                DisplayName = "Test Soldier",
                Description = "A test soldier unit",
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
                Weapon = "standard-rifle",
                DefenseTags = new List<string> { "InfantryArmor", "Biological" },
                BehaviorTags = new List<string> { "HoldLine", "AdvanceFire" },
                VanillaMapping = "vanilla_soldier",
                TechRequirement = "barracks_lvl2"
            };

            string yaml = _serializer.Serialize(original);
            UnitDefinition deserialized = YamlLoader.Deserializer.Deserialize<UnitDefinition>(yaml);

            deserialized.Id.Should().Be(original.Id);
            deserialized.DisplayName.Should().Be(original.DisplayName);
            deserialized.Description.Should().Be(original.Description);
            deserialized.UnitClass.Should().Be(original.UnitClass);
            deserialized.FactionId.Should().Be(original.FactionId);
            deserialized.Tier.Should().Be(original.Tier);
            deserialized.Stats.Hp.Should().Be(original.Stats.Hp);
            deserialized.Stats.Cost.Food.Should().Be(original.Stats.Cost.Food);
            deserialized.Stats.Cost.Gold.Should().Be(original.Stats.Cost.Gold);
            deserialized.Weapon.Should().Be(original.Weapon);
            deserialized.DefenseTags.Should().BeEquivalentTo(original.DefenseTags);
            deserialized.BehaviorTags.Should().BeEquivalentTo(original.BehaviorTags);
            deserialized.VanillaMapping.Should().Be(original.VanillaMapping);
            deserialized.TechRequirement.Should().Be(original.TechRequirement);
        }

        [Fact]
        public void UnitDefinition_RoundTrip_MinimalFields()
        {
            var original = new UnitDefinition
            {
                Id = "minimal-unit",
                DisplayName = "Minimal"
            };

            string yaml = _serializer.Serialize(original);
            UnitDefinition deserialized = YamlLoader.Deserializer.Deserialize<UnitDefinition>(yaml);

            deserialized.Id.Should().Be("minimal-unit");
            deserialized.DisplayName.Should().Be("Minimal");
        }

        [Fact]
        public void FactionDefinition_RoundTrip_AllFields()
        {
            var original = new FactionDefinition
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
                    LineInfantry = "clone-rifleman",
                    EliteInfantry = "arc-trooper"
                }
            };

            string yaml = _serializer.Serialize(original);
            FactionDefinition deserialized = YamlLoader.Deserializer.Deserialize<FactionDefinition>(yaml);

            deserialized.Faction.Id.Should().Be("republic");
            deserialized.Faction.Theme.Should().Be("starwars");
            deserialized.Faction.Archetype.Should().Be("order");
            deserialized.Economy.GatherBonus.Should().Be(1.1f);
            deserialized.Army.MoraleStyle.Should().Be("disciplined");
            deserialized.Roster.CheapInfantry.Should().Be("clone-trooper");
        }

        [Fact]
        public void WeaponDefinition_RoundTrip()
        {
            var original = new WeaponDefinition
            {
                Id = "dc-15a",
                DisplayName = "DC-15A Blaster Rifle",
                WeaponClass = "rifle",
                DamageType = "energy",
                BaseDamage = 25f,
                Range = 30f,
                RateOfFire = 2.0f,
                ProjectileId = "blaster-bolt-blue",
                AoeRadius = 0f
            };

            string yaml = _serializer.Serialize(original);
            WeaponDefinition deserialized = YamlLoader.Deserializer.Deserialize<WeaponDefinition>(yaml);

            deserialized.Id.Should().Be("dc-15a");
            deserialized.WeaponClass.Should().Be("rifle");
            deserialized.BaseDamage.Should().Be(25f);
            deserialized.ProjectileId.Should().Be("blaster-bolt-blue");
        }

        [Fact]
        public void ProjectileDefinition_RoundTrip()
        {
            var original = new ProjectileDefinition
            {
                Id = "blaster-bolt",
                DisplayName = "Blaster Bolt",
                Speed = 50f,
                Damage = 20f,
                AoeRadius = 0f,
                VisualPrefab = "vfx_blaster_blue",
                ImpactEffect = "vfx_blaster_impact"
            };

            string yaml = _serializer.Serialize(original);
            ProjectileDefinition deserialized = YamlLoader.Deserializer.Deserialize<ProjectileDefinition>(yaml);

            deserialized.Id.Should().Be("blaster-bolt");
            deserialized.Speed.Should().Be(50f);
            deserialized.VisualPrefab.Should().Be("vfx_blaster_blue");
        }

        [Fact]
        public void DoctrineDefinition_RoundTrip()
        {
            var original = new DoctrineDefinition
            {
                Id = "blitzkrieg",
                DisplayName = "Blitzkrieg",
                Description = "Fast and aggressive",
                FactionArchetype = "order",
                Modifiers = new Dictionary<string, float>
                {
                    { "speed", 1.3f },
                    { "damage", 1.15f },
                    { "armor", 0.85f }
                }
            };

            string yaml = _serializer.Serialize(original);
            DoctrineDefinition deserialized = YamlLoader.Deserializer.Deserialize<DoctrineDefinition>(yaml);

            deserialized.Id.Should().Be("blitzkrieg");
            deserialized.FactionArchetype.Should().Be("order");
            deserialized.Modifiers.Should().HaveCount(3);
            deserialized.Modifiers["speed"].Should().Be(1.3f);
        }

        [Fact]
        public void BuildingDefinition_RoundTrip()
        {
            var original = new BuildingDefinition
            {
                Id = "barracks",
                DisplayName = "Barracks",
                Description = "Trains infantry",
                BuildingType = "barracks",
                Cost = new ResourceCost { Wood = 50, Stone = 30 },
                Health = 500,
                Production = new Dictionary<string, int> { { "infantry", 2 } }
            };

            string yaml = _serializer.Serialize(original);
            BuildingDefinition deserialized = YamlLoader.Deserializer.Deserialize<BuildingDefinition>(yaml);

            deserialized.Id.Should().Be("barracks");
            deserialized.Health.Should().Be(500);
            deserialized.Cost.Wood.Should().Be(50);
            deserialized.Production.Should().ContainKey("infantry");
        }

        [Fact]
        public void SkillDefinition_RoundTrip()
        {
            var original = new SkillDefinition
            {
                Id = "heal",
                DisplayName = "Heal",
                Description = "Heals a single ally",
                SkillClass = "heal",
                TargetType = "single_ally",
                Cooldown = 10f,
                Duration = 0f,
                Range = 5f,
                Radius = 0f,
                Effects = new List<SkillEffect>
                {
                    new SkillEffect { Stat = "health", ModifierType = SkillModifierType.Flat, Value = 50f }
                },
                VanillaMapping = "Components.Skills.HealSkillData"
            };

            string yaml = _serializer.Serialize(original);
            SkillDefinition deserialized = YamlLoader.Deserializer.Deserialize<SkillDefinition>(yaml);

            deserialized.Id.Should().Be("heal");
            deserialized.SkillClass.Should().Be("heal");
            deserialized.Effects.Should().HaveCount(1);
            deserialized.Effects[0].Value.Should().Be(50f);
            deserialized.VanillaMapping.Should().Be("Components.Skills.HealSkillData");
        }

        [Fact]
        public void WaveDefinition_RoundTrip()
        {
            var original = new WaveDefinition
            {
                Id = "wave-5",
                DisplayName = "Wave 5",
                WaveNumber = 5,
                DelaySeconds = 300f,
                IsFinalWave = true,
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

            string yaml = _serializer.Serialize(original);
            WaveDefinition deserialized = YamlLoader.Deserializer.Deserialize<WaveDefinition>(yaml);

            deserialized.Id.Should().Be("wave-5");
            deserialized.WaveNumber.Should().Be(5);
            deserialized.IsFinalWave.Should().BeTrue();
            deserialized.SpawnGroups.Should().HaveCount(2);
            deserialized.SpawnGroups[0].UnitId.Should().Be("skeleton");
            deserialized.SpawnGroups[0].Count.Should().Be(100);
            deserialized.SpawnGroups[0].SpawnPoint.Should().Be("north_gate");
            deserialized.DifficultyScaling.Should().NotBeNull();
            deserialized.DifficultyScaling!.CountMultiplier.Should().Be(1.5f);
        }

        [Fact]
        public void SquadDefinition_RoundTrip()
        {
            var original = new SquadDefinition
            {
                Id = "archer-squad",
                DisplayName = "Archer Squad",
                Description = "A squad of archers",
                UnitId = "archer",
                MinSize = 5,
                MaxSize = 30,
                DefaultFormation = "line",
                FormationSpacing = 1.5f,
                ColorPrimary = "#FF0000",
                ColorSecondary = "#0000FF",
                BehaviorTags = new List<string> { "defensive", "hold_position" }
            };

            string yaml = _serializer.Serialize(original);
            SquadDefinition deserialized = YamlLoader.Deserializer.Deserialize<SquadDefinition>(yaml);

            deserialized.Id.Should().Be("archer-squad");
            deserialized.UnitId.Should().Be("archer");
            deserialized.MinSize.Should().Be(5);
            deserialized.MaxSize.Should().Be(30);
            deserialized.DefaultFormation.Should().Be("line");
            deserialized.ColorPrimary.Should().Be("#FF0000");
            deserialized.BehaviorTags.Should().BeEquivalentTo(new[] { "defensive", "hold_position" });
        }
    }
}
