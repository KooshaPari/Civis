using System;
using System.Collections.Generic;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Economy.Registries;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for ResourceRegistry covering registration, retrieval, and default initialization.
    /// </summary>
    public class ResourceRegistryUnitTests
    {
        [Fact]
        public void Constructor_LoadsDefaultResources()
        {
            var registry = new ResourceRegistry();

            registry.Count.Should().Be(5);
        }

        [Fact]
        public void All_ReturnsAllResources()
        {
            var registry = new ResourceRegistry();

            var all = registry.All;

            all.Should().HaveCount(5);
        }

        [Fact]
        public void Constructor_IncludesFoodResource()
        {
            var registry = new ResourceRegistry();

            registry.TryGetResource("food", out var resource).Should().BeTrue();
            resource.Should().NotBeNull();
            resource!.Name.Should().Be("Food");
        }

        [Fact]
        public void Constructor_IncludesWoodResource()
        {
            var registry = new ResourceRegistry();

            registry.TryGetResource("wood", out var resource).Should().BeTrue();
            resource!.Name.Should().Be("Wood");
        }

        [Fact]
        public void Constructor_IncludesStoneResource()
        {
            var registry = new ResourceRegistry();

            registry.TryGetResource("stone", out var resource).Should().BeTrue();
            resource!.Name.Should().Be("Stone");
        }

        [Fact]
        public void Constructor_IncludesIronResource()
        {
            var registry = new ResourceRegistry();

            registry.TryGetResource("iron", out var resource).Should().BeTrue();
            resource!.Name.Should().Be("Iron");
        }

        [Fact]
        public void Constructor_IncludesGoldResource()
        {
            var registry = new ResourceRegistry();

            registry.TryGetResource("gold", out var resource).Should().BeTrue();
            resource!.Name.Should().Be("Gold");
        }

        [Fact]
        public void GetResource_ExistingId_ReturnsResource()
        {
            var registry = new ResourceRegistry();

            var resource = registry.GetResource("food");

            resource.Should().NotBeNull();
            resource.Id.Should().Be("food");
        }

        [Fact]
        public void GetResource_NonExistentId_ThrowsKeyNotFoundException()
        {
            var registry = new ResourceRegistry();

            Action act = () => registry.GetResource("nonexistent");

            act.Should().Throw<KeyNotFoundException>();
        }

        [Fact]
        public void GetResource_CaseInsensitive_ReturnsResource()
        {
            var registry = new ResourceRegistry();

            var resource = registry.GetResource("FOOD");

            resource.Should().NotBeNull();
            resource.Id.Should().Be("food");
        }

        [Fact]
        public void TryGetResource_ExistingId_ReturnsTrueWithResource()
        {
            var registry = new ResourceRegistry();

            var found = registry.TryGetResource("wood", out var resource);

            found.Should().BeTrue();
            resource.Should().NotBeNull();
            resource!.Id.Should().Be("wood");
        }

        [Fact]
        public void TryGetResource_NonExistentId_ReturnsFalseWithNull()
        {
            var registry = new ResourceRegistry();

            var found = registry.TryGetResource("nonexistent", out var resource);

            found.Should().BeFalse();
            resource.Should().BeNull();
        }

        [Fact]
        public void Contains_ExistingId_ReturnsTrue()
        {
            var registry = new ResourceRegistry();

            registry.Contains("stone").Should().BeTrue();
        }

        [Fact]
        public void Contains_NonExistentId_ReturnsFalse()
        {
            var registry = new ResourceRegistry();

            registry.Contains("nonexistent").Should().BeFalse();
        }

        [Fact]
        public void Contains_CaseInsensitive_ReturnsTrue()
        {
            var registry = new ResourceRegistry();

            registry.Contains("IRON").Should().BeTrue();
        }

        [Fact]
        public void Register_NewResource_AddsToRegistry()
        {
            var registry = new ResourceRegistry();
            var newResource = new ResourceDefinition(
                "copper",
                "Copper",
                "A new metal resource",
                0.5f,
                900.0f,
                0.0f,
                true);

            registry.Register(newResource);

            registry.TryGetResource("copper", out var result).Should().BeTrue();
            result.Should().Be(newResource);
        }

        [Fact]
        public void Register_OverrideExisting_ReplacesResource()
        {
            var registry = new ResourceRegistry();
            var newResource = new ResourceDefinition(
                "food",
                "Premium Food",
                "Modified description",
                2.0f,
                2000.0f,
                0.02f,
                true);

            registry.Register(newResource);

            var result = registry.GetResource("food");
            result.Name.Should().Be("Premium Food");
        }

        [Fact]
        public void Register_NullResource_ThrowsArgumentNullException()
        {
            var registry = new ResourceRegistry();

            Action act = () => registry.Register(null!);

            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void Register_EmptyId_ThrowsArgumentException()
        {
            var registry = new ResourceRegistry();
            var resource = new ResourceDefinition(
                "",
                "No ID",
                "Description",
                1.0f,
                1000.0f,
                0.0f,
                true);

            Action act = () => registry.Register(resource);

            act.Should().Throw<ArgumentException>();
        }

        [Fact]
        public void Register_WhitespaceId_ThrowsArgumentException()
        {
            var registry = new ResourceRegistry();
            var resource = new ResourceDefinition(
                "   ",
                "Whitespace ID",
                "Description",
                1.0f,
                1000.0f,
                0.0f,
                true);

            Action act = () => registry.Register(resource);

            act.Should().Throw<ArgumentException>();
        }

        [Fact]
        public void Unregister_ExistingId_RemovesResourceAndReturnsTrue()
        {
            var registry = new ResourceRegistry();

            var removed = registry.Unregister("gold");

            removed.Should().BeTrue();
            registry.Contains("gold").Should().BeFalse();
            registry.Count.Should().Be(4);
        }

        [Fact]
        public void Unregister_NonExistentId_ReturnsFalse()
        {
            var registry = new ResourceRegistry();

            var removed = registry.Unregister("nonexistent");

            removed.Should().BeFalse();
            registry.Count.Should().Be(5);
        }

        [Fact]
        public void Unregister_MultipleResources_CountDecreases()
        {
            var registry = new ResourceRegistry();

            registry.Unregister("food");
            registry.Unregister("wood");

            registry.Count.Should().Be(3);
        }

        [Fact]
        public void All_ReturnsReadOnlyList()
        {
            var registry = new ResourceRegistry();

            var all = registry.All;

            all.Should().BeAssignableTo<IReadOnlyList<ResourceDefinition>>();
        }

        [Fact]
        public void All_MultipleRegistrations_IncludesAllResources()
        {
            var registry = new ResourceRegistry();
            var custom = new ResourceDefinition(
                "custom",
                "Custom Resource",
                "A custom resource",
                1.0f,
                1000.0f,
                0.0f,
                true);

            registry.Register(custom);

            registry.All.Should().HaveCount(6);
        }

        [Fact]
        public void Count_MatchesAllSize()
        {
            var registry = new ResourceRegistry();

            registry.Count.Should().Be(registry.All.Count);
        }

        [Fact]
        public void DefaultResources_HaveDistinctProperties()
        {
            var registry = new ResourceRegistry();

            var food = registry.GetResource("food");
            var gold = registry.GetResource("gold");

            food.Id.Should().NotBe(gold.Id);
            food.Name.Should().NotBe(gold.Name);
        }

        [Fact]
        public void GetResource_AfterUnregister_ThrowsKeyNotFoundException()
        {
            var registry = new ResourceRegistry();

            registry.Unregister("stone");

            Action act = () => registry.GetResource("stone");

            act.Should().Throw<KeyNotFoundException>();
        }

        [Fact]
        public void Register_SamIdMultipleTimes_LastRegistrationWins()
        {
            var registry = new ResourceRegistry();
            var resource1 = new ResourceDefinition(
                "custom",
                "First",
                "First description",
                1.0f,
                1000.0f,
                0.0f,
                true);
            var resource2 = new ResourceDefinition(
                "custom",
                "Second",
                "Second description",
                2.0f,
                2000.0f,
                0.0f,
                true);

            registry.Register(resource1);
            registry.Register(resource2);

            var result = registry.GetResource("custom");
            result.Name.Should().Be("Second");
        }
    }
}
