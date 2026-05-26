using System;
using System.Collections.Generic;
using DINOForge.Domains.Warfare.Archetypes;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for ArchetypeRegistry covering registration, retrieval, and default initialization.
    /// </summary>
    public class ArchetypeRegistryUnitTests
    {
        [Fact]
        public void Constructor_LoadsDefaultArchetypes()
        {
            var registry = new ArchetypeRegistry();

            registry.All.Should().HaveCount(3);
        }

        [Fact]
        public void Constructor_IncludesOrderArchetype()
        {
            var registry = new ArchetypeRegistry();

            registry.TryGetArchetype("order", out var archetype).Should().BeTrue();
            archetype.Should().NotBeNull();
            archetype!.DisplayName.Should().Be("Order");
        }

        [Fact]
        public void Constructor_IncludesIndustrialSwarmArchetype()
        {
            var registry = new ArchetypeRegistry();

            registry.TryGetArchetype("industrial_swarm", out var archetype).Should().BeTrue();
            archetype.Should().NotBeNull();
            archetype!.DisplayName.Should().Be("Industrial Swarm");
        }

        [Fact]
        public void Constructor_IncludesAsymmetricArchetype()
        {
            var registry = new ArchetypeRegistry();

            registry.TryGetArchetype("asymmetric", out var archetype).Should().BeTrue();
            archetype.Should().NotBeNull();
            archetype!.DisplayName.Should().Be("Asymmetric");
        }

        [Fact]
        public void GetArchetype_ExistingId_ReturnsArchetype()
        {
            var registry = new ArchetypeRegistry();

            var archetype = registry.GetArchetype("order");

            archetype.Should().NotBeNull();
            archetype.Id.Should().Be("order");
        }

        [Fact]
        public void GetArchetype_NonExistentId_ThrowsKeyNotFoundException()
        {
            var registry = new ArchetypeRegistry();

            Action act = () => registry.GetArchetype("nonexistent");

            act.Should().Throw<KeyNotFoundException>();
        }

        [Fact]
        public void GetArchetype_CaseInsensitive_ReturnsArchetype()
        {
            var registry = new ArchetypeRegistry();

            var archetype = registry.GetArchetype("ORDER");

            archetype.Should().NotBeNull();
            archetype.Id.Should().Be("order");
        }

        [Fact]
        public void TryGetArchetype_ExistingId_ReturnsTrueWithArchetype()
        {
            var registry = new ArchetypeRegistry();

            var found = registry.TryGetArchetype("order", out var archetype);

            found.Should().BeTrue();
            archetype.Should().NotBeNull();
            archetype!.Id.Should().Be("order");
        }

        [Fact]
        public void TryGetArchetype_NonExistentId_ReturnsFalseWithNull()
        {
            var registry = new ArchetypeRegistry();

            var found = registry.TryGetArchetype("nonexistent", out var archetype);

            found.Should().BeFalse();
            archetype.Should().BeNull();
        }

        [Fact]
        public void Register_NewArchetype_AddsToRegistry()
        {
            var registry = new ArchetypeRegistry();
            var newArchetype = new FactionArchetype(
                "custom",
                "Custom",
                "Custom archetype",
                new Dictionary<string, float> { { "speed", 1.0f } });

            registry.Register(newArchetype);

            registry.TryGetArchetype("custom", out var result).Should().BeTrue();
            result.Should().Be(newArchetype);
        }

        [Fact]
        public void Register_OverrideExisting_ReplacesArchetype()
        {
            var registry = new ArchetypeRegistry();
            var newArchetype = new FactionArchetype(
                "order",
                "Modified Order",
                "Modified description",
                new Dictionary<string, float> { { "speed", 0.5f } });

            registry.Register(newArchetype);

            var result = registry.GetArchetype("order");
            result.DisplayName.Should().Be("Modified Order");
        }

        [Fact]
        public void Register_NullArchetype_ThrowsArgumentNullException()
        {
            var registry = new ArchetypeRegistry();

            Action act = () => registry.Register(null!);

            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void All_ReturnsReadOnlyList()
        {
            var registry = new ArchetypeRegistry();

            var all = registry.All;

            all.Should().NotBeNull();
            all.Should().BeAssignableTo<IReadOnlyList<FactionArchetype>>();
        }

        [Fact]
        public void All_CountMatchesRegisteredArchetypes()
        {
            var registry = new ArchetypeRegistry();

            registry.All.Should().HaveCount(3);
        }

        [Fact]
        public void All_MultipleRegistrations_IncludesAllArchetypes()
        {
            var registry = new ArchetypeRegistry();
            var custom1 = new FactionArchetype(
                "custom1",
                "Custom 1",
                "Description 1",
                new Dictionary<string, float>());
            var custom2 = new FactionArchetype(
                "custom2",
                "Custom 2",
                "Description 2",
                new Dictionary<string, float>());

            registry.Register(custom1);
            registry.Register(custom2);

            registry.All.Should().HaveCount(5);
        }

        [Fact]
        public void All_EmptyAfterMultipleOverrides_StaysConsistent()
        {
            var registry = new ArchetypeRegistry();
            var replacement = new FactionArchetype(
                "order",
                "Replaced",
                "Replaced description",
                new Dictionary<string, float>());

            registry.Register(replacement);

            registry.All.Should().HaveCount(3);
        }

        [Fact]
        public void Register_ModifiersCopied_ArchetypeRetainsModifiers()
        {
            var registry = new ArchetypeRegistry();
            var customArchetype = new FactionArchetype(
                "elite",
                "Elite",
                "Elite archetype",
                new Dictionary<string, float>
                {
                    { "armor", 1.5f },
                    { "damage", 1.2f }
                });

            registry.Register(customArchetype);

            var retrieved = registry.GetArchetype("elite");
            retrieved.BaseModifiers.Should().ContainKey("armor").WhoseValue.Should().Be(1.5f);
            retrieved.BaseModifiers.Should().ContainKey("damage").WhoseValue.Should().Be(1.2f);
        }

        [Fact]
        public void DefaultArchetypes_HaveDistinctModifiers()
        {
            var registry = new ArchetypeRegistry();

            var order = registry.GetArchetype("order");
            var swarm = registry.GetArchetype("industrial_swarm");
            var asymmetric = registry.GetArchetype("asymmetric");

            order.BaseModifiers.Keys.Should().NotBeEquivalentTo(swarm.BaseModifiers.Keys);
            swarm.BaseModifiers.Keys.Should().NotBeEquivalentTo(asymmetric.BaseModifiers.Keys);
            order.BaseModifiers.Keys.Should().NotBeEquivalentTo(asymmetric.BaseModifiers.Keys);
        }

        [Fact]
        public void TryGetArchetype_CaseInsensitive_FindsArchetype()
        {
            var registry = new ArchetypeRegistry();

            var found = registry.TryGetArchetype("INDUSTRIAL_SWARM", out var archetype);

            found.Should().BeTrue();
            archetype!.Id.Should().Be("industrial_swarm");
        }
    }
}
