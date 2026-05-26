using System;
using System.Collections.Generic;
using DINOForge.Domains.Scenario;
using DINOForge.Domains.Scenario.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class ScenarioPluginUnitTests
    {
        private static RegistryManager CreateMockRegistries()
        {
            return new RegistryManager();
        }

        [Fact]
        public void Constructor_ThrowsArgumentNullException_WhenRegistriesIsNull()
        {
            Action act = () => new ScenarioPlugin(null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("registries");
        }

        [Fact]
        public void Constructor_InitializesAllSubsystems()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);

            plugin.Scenarios.Should().NotBeNull();
            plugin.ContentLoader.Should().NotBeNull();
            plugin.Runner.Should().NotBeNull();
            plugin.Validator.Should().NotBeNull();
            plugin.DifficultyScaler.Should().NotBeNull();
        }

        [Fact]
        public void Scenarios_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);

            plugin.Scenarios.Should().NotBeNull();
        }

        [Fact]
        public void ContentLoader_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);

            plugin.ContentLoader.Should().NotBeNull();
        }

        [Fact]
        public void Runner_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);

            plugin.Runner.Should().NotBeNull();
        }

        [Fact]
        public void Validator_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);

            plugin.Validator.Should().NotBeNull();
        }

        [Fact]
        public void DifficultyScaler_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);

            plugin.DifficultyScaler.Should().NotBeNull();
        }

        [Fact]
        public void ValidatePack_ThrowsArgumentException_WhenPackIdIsEmpty()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);
            var scenarios = new List<ScenarioDefinition>();

            Action act = () => plugin.ValidatePack("", scenarios);
            act.Should().Throw<ArgumentException>().WithParameterName("packId");
        }

        [Fact]
        public void ValidatePack_ThrowsArgumentException_WhenPackIdIsNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);
            var scenarios = new List<ScenarioDefinition>();

            Action act = () => plugin.ValidatePack(null!, scenarios);
            act.Should().Throw<ArgumentException>().WithParameterName("packId");
        }

        [Fact]
        public void ValidatePack_ThrowsArgumentNullException_WhenScenariosIsNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);

            Action act = () => plugin.ValidatePack("test-pack", null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("scenarios");
        }

        [Fact]
        public void ValidatePack_ReturnsValidationResult_WithEmptyScenarios()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);
            var scenarios = new List<ScenarioDefinition>();

            var result = plugin.ValidatePack("test-pack", scenarios);

            result.Should().NotBeNull();
            result.PackId.Should().Be("test-pack");
        }

        [Fact]
        public void ValidatePack_CountsScenarios_Correctly()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);
            var scenarios = new List<ScenarioDefinition>
            {
                new ScenarioDefinition { Id = "scenario-1" },
                new ScenarioDefinition { Id = "scenario-2" },
            };

            var result = plugin.ValidatePack("test-pack", scenarios);

            result.ScenarioCount.Should().Be(2);
        }

        [Fact]
        public void ValidatePack_GeneratesWarning_WhenScenarioHasNoVictoryConditions()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);
            var scenario = new ScenarioDefinition
            {
                Id = "no-victory",
                VictoryConditions = new List<VictoryCondition>()
            };
            var scenarios = new List<ScenarioDefinition> { scenario };

            var result = plugin.ValidatePack("test-pack", scenarios);

            result.Warnings.Should().Contain(w => w.Contains("no victory conditions"));
        }

        [Fact]
        public void ValidatePack_GeneratesWarning_WhenScenarioHasNoDefeatConditions()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);
            var scenario = new ScenarioDefinition
            {
                Id = "no-defeat",
                DefeatConditions = new List<DefeatCondition>()
            };
            var scenarios = new List<ScenarioDefinition> { scenario };

            var result = plugin.ValidatePack("test-pack", scenarios);

            result.Warnings.Should().Contain(w => w.Contains("no defeat conditions"));
        }

        [Fact]
        public void ValidatePack_IsValid_WhenNoErrors()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);
            var scenario = new ScenarioDefinition
            {
                Id = "valid-scenario",
                DisplayName = "Valid Scenario",
                VictoryConditions = new List<VictoryCondition>
                {
                    new VictoryCondition { ConditionType = VictoryConditionType.TimeSurvival, TargetValue = 100 }
                },
                DefeatConditions = new List<DefeatCondition>(),
                WaveCount = 5
            };
            var scenarios = new List<ScenarioDefinition> { scenario };

            var result = plugin.ValidatePack("test-pack", scenarios);

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void ValidatePack_GeneratesWarning_WhenMaxDurationSetWithoutTimeCondition()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);
            var scenario = new ScenarioDefinition
            {
                Id = "time-duration-no-condition",
                MaxDuration = 600,
                VictoryConditions = new List<VictoryCondition>(),
                DefeatConditions = new List<DefeatCondition>()
            };
            var scenarios = new List<ScenarioDefinition> { scenario };

            var result = plugin.ValidatePack("test-pack", scenarios);

            result.Warnings.Should().Contain(w => w.Contains("max_duration") && w.Contains("time-based"));
        }

        [Fact]
        public void ValidatePack_NoWarning_WhenMaxDurationAndTimeSurvivalCondition()
        {
            var registries = CreateMockRegistries();
            var plugin = new ScenarioPlugin(registries);
            var scenario = new ScenarioDefinition
            {
                Id = "time-duration-with-condition",
                MaxDuration = 600,
                VictoryConditions = new List<VictoryCondition>
                {
                    new VictoryCondition { ConditionType = VictoryConditionType.TimeSurvival }
                },
                DefeatConditions = new List<DefeatCondition>()
            };
            var scenarios = new List<ScenarioDefinition> { scenario };

            var result = plugin.ValidatePack("test-pack", scenarios);

            result.Warnings.Should().NotContain(w => w.Contains("max_duration"));
        }
    }
}
