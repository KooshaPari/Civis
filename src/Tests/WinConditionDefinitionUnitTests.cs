using System;
using System.Collections.Generic;
using FluentAssertions;
using DINOForge.Domains.Scenario.Models;
using Xunit;

namespace DINOForge.Tests
{
    public class WinConditionDefinitionUnitTests
    {
        [Fact]
        public void DefaultConstructor_InitializesWithExpectedValues()
        {
            var winCondition = new WinConditionDefinition();

            winCondition.Id.Should().Be(string.Empty);
            winCondition.Type.Should().Be(string.Empty);
            winCondition.Description.Should().Be(string.Empty);
            winCondition.Parameters.Should().NotBeNull();
            winCondition.Parameters.Should().BeEmpty();
        }

        [Fact]
        public void DefaultConstructor_ParametersDictUsesOrdinalComparer()
        {
            var winCondition = new WinConditionDefinition();

            // Verify StringComparer.Ordinal is used (case-sensitive)
            winCondition.Parameters["Key"] = "value";
            winCondition.Parameters.ContainsKey("key").Should().BeFalse();
            winCondition.Parameters.ContainsKey("Key").Should().BeTrue();
        }

        [Fact]
        public void FullConstructor_WithValidArguments_InitializesCorrectly()
        {
            var parameters = new Dictionary<string, object> { { "duration_seconds", 300 } };
            var winCondition = new WinConditionDefinition(
                id: "win-001",
                type: "survive_duration",
                description: "Survive for 5 minutes",
                parameters: parameters);

            winCondition.Id.Should().Be("win-001");
            winCondition.Type.Should().Be("survive_duration");
            winCondition.Description.Should().Be("Survive for 5 minutes");
            winCondition.Parameters.Should().ContainKey("duration_seconds");
            winCondition.Parameters["duration_seconds"].Should().Be(300);
        }

        [Fact]
        public void FullConstructor_WithNullId_ThrowsArgumentNullException()
        {
            var action = () => new WinConditionDefinition(
                id: null!,
                type: "test",
                description: "test",
                parameters: new Dictionary<string, object>());

            action.Should().Throw<ArgumentNullException>().WithParameterName("id");
        }

        [Fact]
        public void FullConstructor_WithNullType_ThrowsArgumentNullException()
        {
            var action = () => new WinConditionDefinition(
                id: "test",
                type: null!,
                description: "test",
                parameters: new Dictionary<string, object>());

            action.Should().Throw<ArgumentNullException>().WithParameterName("type");
        }

        [Fact]
        public void FullConstructor_WithNullDescription_DefaultsToEmpty()
        {
            var winCondition = new WinConditionDefinition(
                id: "win-001",
                type: "test_type",
                description: null,
                parameters: new Dictionary<string, object>());

            winCondition.Description.Should().Be(string.Empty);
        }

        [Fact]
        public void FullConstructor_WithNullParameters_DefaultsToEmptyDict()
        {
            var winCondition = new WinConditionDefinition(
                id: "win-001",
                type: "test_type",
                description: "test",
                parameters: null);

            winCondition.Parameters.Should().NotBeNull();
            winCondition.Parameters.Should().BeEmpty();
        }

        [Fact]
        public void Parameters_CanStoreStringValues()
        {
            var winCondition = new WinConditionDefinition();
            winCondition.Parameters["faction"] = "enemy";

            winCondition.Parameters["faction"].Should().Be("enemy");
        }

        [Fact]
        public void Parameters_CanStoreIntValues()
        {
            var winCondition = new WinConditionDefinition();
            winCondition.Parameters["population"] = 500;

            winCondition.Parameters["population"].Should().Be(500);
        }

        [Fact]
        public void Parameters_CanStoreFloatValues()
        {
            var winCondition = new WinConditionDefinition();
            winCondition.Parameters["threshold"] = 75.5;

            winCondition.Parameters["threshold"].Should().Be(75.5);
        }

        [Fact]
        public void Parameters_CanStoreMultipleEntries()
        {
            var winCondition = new WinConditionDefinition();
            winCondition.Parameters["region_id"] = "castle";
            winCondition.Parameters["percentage"] = 100;
            winCondition.Parameters["zone_type"] = "fortified";

            winCondition.Parameters.Should().HaveCount(3);
            winCondition.Parameters.Should().ContainKey("region_id");
            winCondition.Parameters.Should().ContainKey("percentage");
            winCondition.Parameters.Should().ContainKey("zone_type");
        }

        [Fact]
        public void Parameters_CanBeCleared()
        {
            var winCondition = new WinConditionDefinition();
            winCondition.Parameters["key"] = "value";
            winCondition.Parameters.Clear();

            winCondition.Parameters.Should().BeEmpty();
        }

        [Fact]
        public void CanCreateElminateFactionWinCondition()
        {
            var winCondition = new WinConditionDefinition
            {
                Id = "win-eliminate-faction",
                Type = "eliminate_faction",
                Description = "Eliminate the enemy faction",
                Parameters = new Dictionary<string, object> { { "target_faction", "enemy" } }
            };

            winCondition.Type.Should().Be("eliminate_faction");
            winCondition.Parameters.Should().ContainKey("target_faction");
        }

        [Fact]
        public void CanCreateReachPopulationWinCondition()
        {
            var winCondition = new WinConditionDefinition
            {
                Id = "win-population",
                Type = "reach_population",
                Description = "Reach 500 population",
                Parameters = new Dictionary<string, object> { { "population_threshold", 500 } }
            };

            winCondition.Type.Should().Be("reach_population");
            winCondition.Parameters["population_threshold"].Should().Be(500);
        }

        [Fact]
        public void CanCreateSurviveDurationWinCondition()
        {
            var winCondition = new WinConditionDefinition
            {
                Id = "win-survive",
                Type = "survive_duration",
                Description = "Survive 10 minutes",
                Parameters = new Dictionary<string, object> { { "duration_seconds", 600 } }
            };

            winCondition.Type.Should().Be("survive_duration");
            winCondition.Parameters["duration_seconds"].Should().Be(600);
        }

        [Fact]
        public void CanCreateControlRegionWinCondition()
        {
            var winCondition = new WinConditionDefinition
            {
                Id = "win-control-region",
                Type = "control_region",
                Description = "Control 100% of castle",
                Parameters = new Dictionary<string, object>
                {
                    { "region_id", "castle" },
                    { "percentage", 100 }
                }
            };

            winCondition.Parameters.Should().HaveCount(2);
            winCondition.Parameters["region_id"].Should().Be("castle");
            winCondition.Parameters["percentage"].Should().Be(100);
        }

        [Fact]
        public void PropertiesAreIndependent()
        {
            var condition1 = new WinConditionDefinition { Id = "win-001", Type = "test" };
            var condition2 = new WinConditionDefinition { Id = "win-002", Type = "other" };

            condition1.Id.Should().NotBe(condition2.Id);
            condition1.Type.Should().NotBe(condition2.Type);
        }
    }
}
