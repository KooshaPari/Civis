using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class SkillWaveSquadTests
    {
        // ── SkillDefinition Registry ─────────────────────────────────────────

        [Fact]
        public void Skill_Register_And_Get_ReturnsEntry()
        {
            var registry = new Registry<SkillDefinition>();
            var skill = new SkillDefinition
            {
                Id = "heal",
                DisplayName = "Heal",
                SkillClass = "heal",
                TargetType = "single_ally",
                Cooldown = 10f,
                Range = 5f
            };

            registry.Register("heal", skill, RegistrySource.BaseGame, "base-game");

            SkillDefinition? result = registry.Get("heal");
            result.Should().NotBeNull();
            result!.DisplayName.Should().Be("Heal");
            result.SkillClass.Should().Be("heal");
            result.Cooldown.Should().Be(10f);
        }

        [Fact]
        public void Skill_Override_HigherPriority_ReplacesExisting()
        {
            var registry = new Registry<SkillDefinition>();
            var baseSkill = new SkillDefinition { Id = "rage", DisplayName = "Rage", SkillClass = "buff" };
            var modSkill = new SkillDefinition { Id = "rage", DisplayName = "Super Rage", SkillClass = "buff" };

            registry.Register("rage", baseSkill, RegistrySource.BaseGame, "base-game");
            registry.Override("rage", modSkill, RegistrySource.Pack, "warfare-pack");

            registry.Get("rage")!.DisplayName.Should().Be("Super Rage");
        }

        [Fact]
        public void Skill_SamePriority_CreatesConflict()
        {
            var registry = new Registry<SkillDefinition>();
            var skillA = new SkillDefinition { Id = "meteor", DisplayName = "Meteor A", SkillClass = "aoe_damage" };
            var skillB = new SkillDefinition { Id = "meteor", DisplayName = "Meteor B", SkillClass = "aoe_damage" };

            registry.Register("meteor", skillA, RegistrySource.Pack, "pack-a", 100);
            registry.Override("meteor", skillB, RegistrySource.Pack, "pack-b", 100);

            var conflicts = registry.DetectConflicts();
            conflicts.Should().HaveCount(1);
            conflicts[0].EntryId.Should().Be("meteor");
        }

        // ── WaveDefinition Registry ──────────────────────────────────────────

        [Fact]
        public void Wave_Register_And_Get_ReturnsEntry()
        {
            var registry = new Registry<WaveDefinition>();
            var wave = new WaveDefinition
            {
                Id = "wave-1",
                DisplayName = "First Wave",
                WaveNumber = 1,
                DelaySeconds = 30f,
                IsFinalWave = false,
                SpawnGroups = new List<SpawnGroup>
                {
                    new SpawnGroup { UnitId = "skeleton", Count = 50, SpawnDelay = 0.1f }
                }
            };

            registry.Register("wave-1", wave, RegistrySource.BaseGame, "base-game");

            WaveDefinition? result = registry.Get("wave-1");
            result.Should().NotBeNull();
            result!.WaveNumber.Should().Be(1);
            result.SpawnGroups.Should().HaveCount(1);
            result.SpawnGroups[0].UnitId.Should().Be("skeleton");
            result.SpawnGroups[0].Count.Should().Be(50);
        }

        [Fact]
        public void Wave_Override_HigherPriority_ReplacesExisting()
        {
            var registry = new Registry<WaveDefinition>();
            var baseWave = new WaveDefinition { Id = "wave-5", DisplayName = "Wave 5", WaveNumber = 5 };
            var modWave = new WaveDefinition { Id = "wave-5", DisplayName = "Epic Wave 5", WaveNumber = 5, IsFinalWave = true };

            registry.Register("wave-5", baseWave, RegistrySource.BaseGame, "base-game");
            registry.Override("wave-5", modWave, RegistrySource.Pack, "scenario-pack");

            WaveDefinition? result = registry.Get("wave-5");
            result!.DisplayName.Should().Be("Epic Wave 5");
            result.IsFinalWave.Should().BeTrue();
        }

        [Fact]
        public void Wave_SamePriority_CreatesConflict()
        {
            var registry = new Registry<WaveDefinition>();
            var waveA = new WaveDefinition { Id = "boss-wave", DisplayName = "Boss A" };
            var waveB = new WaveDefinition { Id = "boss-wave", DisplayName = "Boss B" };

            registry.Register("boss-wave", waveA, RegistrySource.Pack, "pack-a", 100);
            registry.Override("boss-wave", waveB, RegistrySource.Pack, "pack-b", 100);

            var conflicts = registry.DetectConflicts();
            conflicts.Should().HaveCount(1);
            conflicts[0].EntryId.Should().Be("boss-wave");
        }

        // ── SquadDefinition Registry ─────────────────────────────────────────

        [Fact]
        public void Squad_Register_And_Get_ReturnsEntry()
        {
            var registry = new Registry<SquadDefinition>();
            var squad = new SquadDefinition
            {
                Id = "archer-squad",
                DisplayName = "Archer Squad",
                UnitId = "archer",
                MinSize = 5,
                MaxSize = 30,
                DefaultFormation = "line",
                FormationSpacing = 1.5f,
                ColorPrimary = "#FF0000",
                BehaviorTags = new List<string> { "defensive", "hold_position" }
            };

            registry.Register("archer-squad", squad, RegistrySource.BaseGame, "base-game");

            SquadDefinition? result = registry.Get("archer-squad");
            result.Should().NotBeNull();
            result!.UnitId.Should().Be("archer");
            result.MaxSize.Should().Be(30);
            result.DefaultFormation.Should().Be("line");
            result.ColorPrimary.Should().Be("#FF0000");
            result.BehaviorTags.Should().Contain("defensive");
        }

        [Fact]
        public void Squad_Override_HigherPriority_ReplacesExisting()
        {
            var registry = new Registry<SquadDefinition>();
            var baseSquad = new SquadDefinition { Id = "infantry", DisplayName = "Infantry", MaxSize = 20 };
            var modSquad = new SquadDefinition { Id = "infantry", DisplayName = "Elite Infantry", MaxSize = 40 };

            registry.Register("infantry", baseSquad, RegistrySource.BaseGame, "base-game");
            registry.Override("infantry", modSquad, RegistrySource.Pack, "warfare-pack");

            SquadDefinition? result = registry.Get("infantry");
            result!.DisplayName.Should().Be("Elite Infantry");
            result.MaxSize.Should().Be(40);
        }

        [Fact]
        public void Squad_SamePriority_CreatesConflict()
        {
            var registry = new Registry<SquadDefinition>();
            var squadA = new SquadDefinition { Id = "cavalry", DisplayName = "Cavalry A" };
            var squadB = new SquadDefinition { Id = "cavalry", DisplayName = "Cavalry B" };

            registry.Register("cavalry", squadA, RegistrySource.Pack, "pack-a", 100);
            registry.Override("cavalry", squadB, RegistrySource.Pack, "pack-b", 100);

            var conflicts = registry.DetectConflicts();
            conflicts.Should().HaveCount(1);
            conflicts[0].EntryId.Should().Be("cavalry");
        }

        // ── RegistryManager Integration ──────────────────────────────────────

        [Fact]
        public void RegistryManager_Exposes_SkillWaveSquad_Registries()
        {
            var manager = new RegistryManager();

            manager.Skills.Should().NotBeNull();
            manager.Waves.Should().NotBeNull();
            manager.Squads.Should().NotBeNull();
        }
    }
}
