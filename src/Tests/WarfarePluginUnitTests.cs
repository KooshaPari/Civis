using System;
using System.Collections.Generic;
using DINOForge.Domains.Warfare;
using DINOForge.Domains.Warfare.Archetypes;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class WarfarePluginUnitTests
    {
        private static RegistryManager CreateMockRegistries()
        {
            return new RegistryManager();
        }

        [Fact]
        public void Constructor_ThrowsArgumentNullException_WhenRegistriesIsNull()
        {
            Action act = () => new WarfarePlugin(null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("registries");
        }

        [Fact]
        public void Constructor_InitializesAllSubsystems()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            plugin.Archetypes.Should().NotBeNull();
            plugin.Doctrines.Should().NotBeNull();
            plugin.RoleValidator.Should().NotBeNull();
            plugin.WaveComposer.Should().NotBeNull();
            plugin.Balance.Should().NotBeNull();
        }

        [Fact]
        public void Archetypes_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            plugin.Archetypes.Should().NotBeNull();
        }

        [Fact]
        public void Doctrines_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            plugin.Doctrines.Should().NotBeNull();
        }

        [Fact]
        public void RoleValidator_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            plugin.RoleValidator.Should().NotBeNull();
        }

        [Fact]
        public void WaveComposer_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            plugin.WaveComposer.Should().NotBeNull();
        }

        [Fact]
        public void Balance_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            plugin.Balance.Should().NotBeNull();
        }

        [Fact]
        public void ValidatePack_ThrowsArgumentException_WhenPackIdIsEmpty()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            Action act = () => plugin.ValidatePack("", registries);
            act.Should().Throw<ArgumentException>().WithParameterName("packId");
        }

        [Fact]
        public void ValidatePack_ThrowsArgumentException_WhenPackIdIsNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            Action act = () => plugin.ValidatePack(null!, registries);
            act.Should().Throw<ArgumentException>().WithParameterName("packId");
        }

        [Fact]
        public void ValidatePack_ThrowsArgumentNullException_WhenRegistriesIsNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            Action act = () => plugin.ValidatePack("test-pack", null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("registries");
        }

        [Fact]
        public void ValidatePack_ReturnsValidationResult()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            var result = plugin.ValidatePack("test-pack", registries);

            result.Should().NotBeNull();
            result.PackId.Should().Be("test-pack");
        }

        [Fact]
        public void ValidatePack_IsValid_WithEmptyContent()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            var result = plugin.ValidatePack("test-pack", registries);

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void ValidatePack_ReturnsRosterResults()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            var result = plugin.ValidatePack("test-pack", registries);

            result.RosterResults.Should().NotBeNull();
        }

        [Fact]
        public void ValidatePack_IncludesRosterResults()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            var result = plugin.ValidatePack("test-pack", registries);

            result.RosterResults.Should().NotBeNull();
        }

        [Fact]
        public void ArchetypesRegistry_IsNotNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            plugin.Archetypes.Should().NotBeNull();
        }

        [Fact]
        public void ArchetypesRegistry_ContainsDefaultArchetypes()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            plugin.Archetypes.TryGetArchetype("order", out _).Should().BeTrue();
        }

        [Fact]
        public void DoctrineEngine_IsNotNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            plugin.Doctrines.Should().NotBeNull();
        }

        [Fact]
        public void WaveComposer_IsNotNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            plugin.WaveComposer.Should().NotBeNull();
        }

        [Fact]
        public void BalanceCalculator_IsNotNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new WarfarePlugin(registries);

            plugin.Balance.Should().NotBeNull();
        }
    }
}
