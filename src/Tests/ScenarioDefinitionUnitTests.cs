using FluentAssertions;
using DINOForge.Domains.Scenario.Models;
using DINOForge.SDK.Models;
using Xunit;

namespace DINOForge.Tests
{
    public class ScenarioDefinitionUnitTests
    {
        [Fact]
        public void Constructor_Defaults_InitializesWithExpectedValues()
        {
            var scenario = new ScenarioDefinition();

            scenario.Id.Should().Be(string.Empty);
            scenario.DisplayName.Should().Be(string.Empty);
            scenario.Description.Should().Be(string.Empty);
            scenario.Difficulty.Should().Be(Difficulty.Normal);
            scenario.ObjectiveType.Should().Be(ObjectiveType.Survive);
            scenario.WaveCount.Should().Be(1);
            scenario.MaxDuration.Should().Be(0);
            scenario.StartingResources.Should().NotBeNull();
            scenario.AllowedFactions.Should().NotBeNull().And.BeEmpty();
            scenario.VictoryConditions.Should().NotBeNull().And.BeEmpty();
            scenario.DefeatConditions.Should().NotBeNull().And.BeEmpty();
            scenario.ScriptedEvents.Should().NotBeNull().And.BeEmpty();
        }

        [Fact]
        public void Validate_WithValidScenario_ReturnsSuccess()
        {
            var scenario = new ScenarioDefinition
            {
                Id = "scenario-001",
                DisplayName = "Tutorial Scenario",
                Description = "A simple tutorial",
                Difficulty = Difficulty.Easy,
                ObjectiveType = ObjectiveType.Defend,
                WaveCount = 5,
                MaxDuration = 600,
                AllowedFactions = new() { "faction-a", "faction-b" }
            };

            var result = scenario.Validate();

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_WithBlankId_ReturnsFailure()
        {
            var scenario = new ScenarioDefinition
            {
                Id = "   ",
                DisplayName = "Test Scenario",
                WaveCount = 1
            };

            var result = scenario.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "id");
        }

        [Fact]
        public void Validate_WithBlankDisplayName_ReturnsFailure()
        {
            var scenario = new ScenarioDefinition
            {
                Id = "scenario-001",
                DisplayName = "",
                WaveCount = 1
            };

            var result = scenario.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "display_name");
        }

        [Fact]
        public void Validate_WithZeroWaveCount_ReturnsFailure()
        {
            var scenario = new ScenarioDefinition
            {
                Id = "scenario-001",
                DisplayName = "Test",
                WaveCount = 0
            };

            var result = scenario.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "wave_count");
        }

        [Fact]
        public void Validate_WithNegativeWaveCount_ReturnsFailure()
        {
            var scenario = new ScenarioDefinition
            {
                Id = "scenario-001",
                DisplayName = "Test",
                WaveCount = -5
            };

            var result = scenario.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "wave_count");
        }

        [Fact]
        public void Validate_WithNegativeMaxDuration_ReturnsFailure()
        {
            var scenario = new ScenarioDefinition
            {
                Id = "scenario-001",
                DisplayName = "Test",
                WaveCount = 1,
                MaxDuration = -100
            };

            var result = scenario.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "max_duration");
        }

        [Fact]
        public void Validate_WithZeroMaxDuration_Succeeds()
        {
            var scenario = new ScenarioDefinition
            {
                Id = "scenario-001",
                DisplayName = "Test",
                WaveCount = 1,
                MaxDuration = 0
            };

            var result = scenario.Validate();

            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_WithBlankFactionInList_ReturnsFailure()
        {
            var scenario = new ScenarioDefinition
            {
                Id = "scenario-001",
                DisplayName = "Test",
                WaveCount = 1,
                AllowedFactions = new() { "faction-a", "   ", "faction-b" }
            };

            var result = scenario.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "allowed_factions[1]");
        }

        [Fact]
        public void Validate_WithMultipleFactionErrors_ReturnsAllErrors()
        {
            var scenario = new ScenarioDefinition
            {
                Id = "scenario-001",
                DisplayName = "Test",
                WaveCount = 1,
                AllowedFactions = new() { "", "valid", null! }
            };

            var result = scenario.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Count.Should().BeGreaterThanOrEqualTo(2);
        }

        [Fact]
        public void Validate_WithAllDifficulties_Succeeds()
        {
            foreach (var difficulty in new[] { Difficulty.Easy, Difficulty.Normal, Difficulty.Hard, Difficulty.Nightmare })
            {
                var scenario = new ScenarioDefinition
                {
                    Id = "scenario-001",
                    DisplayName = "Test",
                    Difficulty = difficulty,
                    WaveCount = 1
                };

                var result = scenario.Validate();
                result.IsValid.Should().BeTrue();
            }
        }

        [Fact]
        public void Validate_WithAllObjectiveTypes_Succeeds()
        {
            foreach (var objectiveType in new[]
            {
                ObjectiveType.Survive,
                ObjectiveType.Defend,
                ObjectiveType.Attack,
                ObjectiveType.Prosper,
                ObjectiveType.Custom
            })
            {
                var scenario = new ScenarioDefinition
                {
                    Id = "scenario-001",
                    DisplayName = "Test",
                    ObjectiveType = objectiveType,
                    WaveCount = 1
                };

                var result = scenario.Validate();
                result.IsValid.Should().BeTrue();
            }
        }
    }
}
