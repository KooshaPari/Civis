using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class RegistryStressTests
    {
        [Fact]
        public void Register_1000Entries_AllRetrievable()
        {
            var registry = new Registry<TestItem>();

            for (int i = 0; i < 1000; i++)
            {
                string id = $"item-{i}";
                registry.Register(id, new TestItem($"Item {i}"), RegistrySource.BaseGame, "base");
            }

            for (int i = 0; i < 1000; i++)
            {
                string id = $"item-{i}";
                registry.Contains(id).Should().BeTrue($"item-{i} should be registered");
                registry.Get(id)!.Name.Should().Be($"Item {i}");
            }

            registry.All.Should().HaveCount(1000);
        }

        [Fact]
        public void OverrideChain_BaseGame_Framework_DomainPlugin_Pack_PriorityCascade()
        {
            var registry = new Registry<TestItem>();

            registry.Register("raptor", new TestItem("BaseRaptor"), RegistrySource.BaseGame, "base-game");
            registry.Register("raptor", new TestItem("FrameworkRaptor"), RegistrySource.Framework, "framework");
            registry.Register("raptor", new TestItem("DomainRaptor"), RegistrySource.DomainPlugin, "warfare");
            registry.Register("raptor", new TestItem("PackRaptor"), RegistrySource.Pack, "my-pack");

            // Pack has highest priority, should win
            registry.Get("raptor")!.Name.Should().Be("PackRaptor");
        }

        [Fact]
        public void OverrideChain_Framework_BeatsBaseGame()
        {
            var registry = new Registry<TestItem>();

            registry.Register("raptor", new TestItem("BaseRaptor"), RegistrySource.BaseGame, "base-game");
            registry.Register("raptor", new TestItem("FrameworkRaptor"), RegistrySource.Framework, "framework");

            registry.Get("raptor")!.Name.Should().Be("FrameworkRaptor");
        }

        [Fact]
        public void OverrideChain_DomainPlugin_BeatsFramework()
        {
            var registry = new Registry<TestItem>();

            registry.Register("raptor", new TestItem("FrameworkRaptor"), RegistrySource.Framework, "framework");
            registry.Register("raptor", new TestItem("DomainRaptor"), RegistrySource.DomainPlugin, "warfare");

            registry.Get("raptor")!.Name.Should().Be("DomainRaptor");
        }

        [Fact]
        public void MultiplePacks_OverridingSameEntry_HigherLoadOrderWins()
        {
            var registry = new Registry<TestItem>();

            registry.Register("raptor", new TestItem("PackA"), RegistrySource.Pack, "pack-a", 50);
            registry.Register("raptor", new TestItem("PackB"), RegistrySource.Pack, "pack-b", 150);
            registry.Register("raptor", new TestItem("PackC"), RegistrySource.Pack, "pack-c", 100);

            // Pack B has highest load order within same source tier
            registry.Get("raptor")!.Name.Should().Be("PackB");
        }

        [Fact]
        public void MultiplePacks_SameLoadOrder_CreatesConflicts()
        {
            var registry = new Registry<TestItem>();

            registry.Register("raptor", new TestItem("PackA"), RegistrySource.Pack, "pack-a", 100);
            registry.Register("raptor", new TestItem("PackB"), RegistrySource.Pack, "pack-b", 100);
            registry.Register("raptor", new TestItem("PackC"), RegistrySource.Pack, "pack-c", 100);

            var conflicts = registry.DetectConflicts();
            conflicts.Should().HaveCount(1);
            conflicts[0].ConflictingPackIds.Should().HaveCount(3);
        }

        [Fact]
        public void SequentialAccess_SameKeys_ReturnsConsistentResults()
        {
            var registry = new Registry<TestItem>();
            registry.Register("alpha", new TestItem("Alpha"), RegistrySource.BaseGame, "base");
            registry.Register("beta", new TestItem("Beta"), RegistrySource.BaseGame, "base");

            // Access the same keys many times sequentially
            for (int i = 0; i < 100; i++)
            {
                registry.Get("alpha")!.Name.Should().Be("Alpha");
                registry.Get("beta")!.Name.Should().Be("Beta");
                registry.Contains("alpha").Should().BeTrue();
                registry.Contains("gamma").Should().BeFalse();
            }
        }

        [Fact]
        public void RegistryManager_AllContentTypes_Populated()
        {
            var manager = new RegistryManager();

            // Register one entry in each registry
            manager.Units.Register("u1", new UnitDefinition { Id = "u1" }, RegistrySource.Pack, "test");
            manager.Buildings.Register("b1", new BuildingDefinition { Id = "b1" }, RegistrySource.Pack, "test");
            manager.Factions.Register("f1", new FactionDefinition { Faction = new FactionInfo { Id = "f1" } }, RegistrySource.Pack, "test");
            manager.Weapons.Register("w1", new WeaponDefinition { Id = "w1" }, RegistrySource.Pack, "test");
            manager.Projectiles.Register("p1", new ProjectileDefinition { Id = "p1" }, RegistrySource.Pack, "test");
            manager.Doctrines.Register("d1", new DoctrineDefinition { Id = "d1" }, RegistrySource.Pack, "test");
            manager.Skills.Register("s1", new SkillDefinition { Id = "s1" }, RegistrySource.Pack, "test");
            manager.Waves.Register("wave1", new WaveDefinition { Id = "wave1" }, RegistrySource.Pack, "test");
            manager.Squads.Register("sq1", new SquadDefinition { Id = "sq1" }, RegistrySource.Pack, "test");

            manager.Units.Contains("u1").Should().BeTrue();
            manager.Buildings.Contains("b1").Should().BeTrue();
            manager.Factions.Contains("f1").Should().BeTrue();
            manager.Weapons.Contains("w1").Should().BeTrue();
            manager.Projectiles.Contains("p1").Should().BeTrue();
            manager.Doctrines.Contains("d1").Should().BeTrue();
            manager.Skills.Contains("s1").Should().BeTrue();
            manager.Waves.Contains("wave1").Should().BeTrue();
            manager.Squads.Contains("sq1").Should().BeTrue();
        }

        [Fact]
        public void Registry_CaseSensitiveLookup()
        {
            var registry = new Registry<TestItem>();
            registry.Register("Raptor", new TestItem("Raptor"), RegistrySource.BaseGame, "base");

            // Pattern #99: Registry keys are case-sensitive (StringComparer.Ordinal).
            registry.Get("Raptor")!.Name.Should().Be("Raptor");
            registry.Get("raptor").Should().BeNull();
            registry.Get("RAPTOR").Should().BeNull();
            registry.Contains("RaPtOr").Should().BeFalse();
        }

        [Fact]
        public void Register_ManyKeys_NoConflicts()
        {
            var registry = new Registry<TestItem>();

            for (int i = 0; i < 500; i++)
            {
                registry.Register($"item-{i}", new TestItem($"Item {i}"), RegistrySource.Pack, "test-pack", 100);
            }

            var conflicts = registry.DetectConflicts();
            conflicts.Should().BeEmpty();
        }
    }
}
