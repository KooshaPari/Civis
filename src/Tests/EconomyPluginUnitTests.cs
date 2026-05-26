using System;
using System.Collections.Generic;
using DINOForge.Domains.Economy;
using DINOForge.Domains.Economy.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class EconomyPluginUnitTests
    {
        private static RegistryManager CreateMockRegistries()
        {
            return new RegistryManager();
        }

        [Fact]
        public void Constructor_ThrowsArgumentNullException_WhenRegistriesIsNull()
        {
            Action act = () => new EconomyPlugin(null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("registries");
        }

        [Fact]
        public void Constructor_InitializesAllSubsystems()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);

            plugin.Production.Should().NotBeNull();
            plugin.Trade.Should().NotBeNull();
            plugin.Balance.Should().NotBeNull();
            plugin.Validator.Should().NotBeNull();
        }

        [Fact]
        public void Production_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);

            plugin.Production.Should().NotBeNull();
        }

        [Fact]
        public void Trade_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);

            plugin.Trade.Should().NotBeNull();
        }

        [Fact]
        public void Balance_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);

            plugin.Balance.Should().NotBeNull();
        }

        [Fact]
        public void Validator_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);

            plugin.Validator.Should().NotBeNull();
        }

        [Fact]
        public void ValidatePack_ThrowsArgumentException_WhenPackIdIsEmpty()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);
            var profiles = new List<EconomyProfile>();
            var routes = new List<TradeRoute>();

            Action act = () => plugin.ValidatePack("", profiles, routes);
            act.Should().Throw<ArgumentException>().WithParameterName("packId");
        }

        [Fact]
        public void ValidatePack_ThrowsArgumentException_WhenPackIdIsNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);
            var profiles = new List<EconomyProfile>();
            var routes = new List<TradeRoute>();

            Action act = () => plugin.ValidatePack(null!, profiles, routes);
            act.Should().Throw<ArgumentException>().WithParameterName("packId");
        }

        [Fact]
        public void ValidatePack_ThrowsArgumentNullException_WhenProfilesIsNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);
            var routes = new List<TradeRoute>();

            Action act = () => plugin.ValidatePack("test-pack", null!, routes);
            act.Should().Throw<ArgumentNullException>().WithParameterName("profiles");
        }

        [Fact]
        public void ValidatePack_ThrowsArgumentNullException_WhenTradeRoutesIsNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);
            var profiles = new List<EconomyProfile>();

            Action act = () => plugin.ValidatePack("test-pack", profiles, null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("tradeRoutes");
        }

        [Fact]
        public void ValidatePack_ReturnsValidationResult()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);
            var profiles = new List<EconomyProfile>();
            var routes = new List<TradeRoute>();

            var result = plugin.ValidatePack("test-pack", profiles, routes);

            result.Should().NotBeNull();
        }

        [Fact]
        public void GenerateBalanceReport_ThrowsArgumentException_WhenPackIdIsEmpty()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);
            var profiles = new Dictionary<string, EconomyProfile>();
            var routes = new List<TradeRoute>();

            Action act = () => plugin.GenerateBalanceReport("", profiles, routes);
            act.Should().Throw<ArgumentException>().WithParameterName("packId");
        }

        [Fact]
        public void GenerateBalanceReport_ThrowsArgumentException_WhenPackIdIsNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);
            var profiles = new Dictionary<string, EconomyProfile>();
            var routes = new List<TradeRoute>();

            Action act = () => plugin.GenerateBalanceReport(null!, profiles, routes);
            act.Should().Throw<ArgumentException>().WithParameterName("packId");
        }

        [Fact]
        public void GenerateBalanceReport_ThrowsArgumentNullException_WhenProfilesIsNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);
            var routes = new List<TradeRoute>();

            Action act = () => plugin.GenerateBalanceReport("test-pack", null!, routes);
            act.Should().Throw<ArgumentNullException>().WithParameterName("profiles");
        }

        [Fact]
        public void GenerateBalanceReport_ThrowsArgumentNullException_WhenTradeRoutesIsNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);
            var profiles = new Dictionary<string, EconomyProfile>();

            Action act = () => plugin.GenerateBalanceReport("test-pack", profiles, null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("tradeRoutes");
        }

        [Fact]
        public void GenerateBalanceReport_ReturnsBalanceReport()
        {
            var registries = CreateMockRegistries();
            var plugin = new EconomyPlugin(registries);
            var profiles = new Dictionary<string, EconomyProfile>();
            var routes = new List<TradeRoute>();

            var result = plugin.GenerateBalanceReport("test-pack", profiles, routes);

            result.Should().NotBeNull();
        }
    }
}
