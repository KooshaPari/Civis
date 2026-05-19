using System;
using System.Collections.Generic;
using DINOForge.Domains.Scenario.Models;
using DINOForge.Domains.Scenario.Registries;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for ScenarioRegistry covering registration, retrieval, and default initialization.
    /// </summary>
    public class ScenarioRegistryUnitTests
    {
        [Fact]
        public void Constructor_InitializesEmptyRegistry()
        {
            var registry = new ScenarioRegistry();

            registry.Count.Should().Be(0);
            registry.All.Should().BeEmpty();
        }

        [Fact]
        public void All_ReturnsEmptyReadOnlyListOnStartup()
        {
            var registry = new ScenarioRegistry();

            var all = registry.All;

            all.Should().BeEmpty();
            all.Should().BeAssignableTo<IReadOnlyList<ScenarioDefinition>>();
        }

        [Fact]
        public void Register_NewScenario_AddsToRegistry()
        {
            var registry = new ScenarioRegistry();
            var scenario = new ScenarioDefinition
            {
                Id = "tutorial",
                DisplayName = "Tutorial Scenario",
                Description = "Learn the basics",
                WaveCount = 1
            };

            registry.Register(scenario);

            registry.Count.Should().Be(1);
        }

        [Fact]
        public void GetScenario_ExistingId_ReturnsScenario()
        {
            var registry = new ScenarioRegistry();
            var scenario = new ScenarioDefinition
            {
                Id = "tutorial",
                DisplayName = "Tutorial",
                Description = "Description",
                WaveCount = 1
            };
            registry.Register(scenario);

            var retrieved = registry.GetScenario("tutorial");

            retrieved.Should().NotBeNull();
            retrieved.Id.Should().Be("tutorial");
        }

        [Fact]
        public void GetScenario_NonExistentId_ThrowsKeyNotFoundException()
        {
            var registry = new ScenarioRegistry();

            Action act = () => registry.GetScenario("nonexistent");

            act.Should().Throw<KeyNotFoundException>();
        }

        [Fact]
        public void GetScenario_CaseInsensitive_ReturnsScenario()
        {
            var registry = new ScenarioRegistry();
            var scenario = new ScenarioDefinition
            {
                Id = "tutorial",
                DisplayName = "Tutorial",
                Description = "Description",
                WaveCount = 1
            };
            registry.Register(scenario);

            var retrieved = registry.GetScenario("TUTORIAL");

            retrieved.Should().NotBeNull();
            retrieved.Id.Should().Be("tutorial");
        }

        [Fact]
        public void TryGetScenario_ExistingId_ReturnsTrueWithScenario()
        {
            var registry = new ScenarioRegistry();
            var scenario = new ScenarioDefinition
            {
                Id = "tutorial",
                DisplayName = "Tutorial",
                Description = "Description",
                WaveCount = 1
            };
            registry.Register(scenario);

            var found = registry.TryGetScenario("tutorial", out var result);

            found.Should().BeTrue();
            result.Should().NotBeNull();
            result!.Id.Should().Be("tutorial");
        }

        [Fact]
        public void TryGetScenario_NonExistentId_ReturnsFalseWithNull()
        {
            var registry = new ScenarioRegistry();

            var found = registry.TryGetScenario("nonexistent", out var result);

            found.Should().BeFalse();
            result.Should().BeNull();
        }

        [Fact]
        public void TryGetScenario_CaseInsensitive_FindsScenario()
        {
            var registry = new ScenarioRegistry();
            var scenario = new ScenarioDefinition
            {
                Id = "campaign",
                DisplayName = "Campaign",
                Description = "Description",
                WaveCount = 3
            };
            registry.Register(scenario);

            var found = registry.TryGetScenario("CAMPAIGN", out var result);

            found.Should().BeTrue();
            result!.Id.Should().Be("campaign");
        }

        [Fact]
        public void Contains_ExistingId_ReturnsTrue()
        {
            var registry = new ScenarioRegistry();
            var scenario = new ScenarioDefinition
            {
                Id = "tutorial",
                DisplayName = "Tutorial",
                Description = "Description",
                WaveCount = 1
            };
            registry.Register(scenario);

            registry.Contains("tutorial").Should().BeTrue();
        }

        [Fact]
        public void Contains_NonExistentId_ReturnsFalse()
        {
            var registry = new ScenarioRegistry();

            registry.Contains("nonexistent").Should().BeFalse();
        }

        [Fact]
        public void Contains_CaseInsensitive_ReturnsTrue()
        {
            var registry = new ScenarioRegistry();
            var scenario = new ScenarioDefinition
            {
                Id = "tutorial",
                DisplayName = "Tutorial",
                Description = "Description",
                WaveCount = 1
            };
            registry.Register(scenario);

            registry.Contains("TUTORIAL").Should().BeTrue();
        }

        [Fact]
        public void Register_NullScenario_ThrowsArgumentNullException()
        {
            var registry = new ScenarioRegistry();

            Action act = () => registry.Register(null!);

            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void Register_OverrideExisting_ReplacesScenario()
        {
            var registry = new ScenarioRegistry();
            var scenario1 = new ScenarioDefinition
            {
                Id = "tutorial",
                DisplayName = "Tutorial",
                Description = "Description 1",
                WaveCount = 1
            };
            var scenario2 = new ScenarioDefinition
            {
                Id = "tutorial",
                DisplayName = "Modified Tutorial",
                Description = "Description 2",
                WaveCount = 2
            };

            registry.Register(scenario1);
            registry.Register(scenario2);

            var retrieved = registry.GetScenario("tutorial");
            retrieved.DisplayName.Should().Be("Modified Tutorial");
        }

        [Fact]
        public void Count_MatchesAllSize()
        {
            var registry = new ScenarioRegistry();
            var scenario1 = new ScenarioDefinition
            {
                Id = "tutorial",
                DisplayName = "Tutorial",
                Description = "Description",
                WaveCount = 1
            };
            var scenario2 = new ScenarioDefinition
            {
                Id = "campaign",
                DisplayName = "Campaign",
                Description = "Description",
                WaveCount = 3
            };

            registry.Register(scenario1);
            registry.Register(scenario2);

            registry.Count.Should().Be(registry.All.Count);
        }

        [Fact]
        public void All_MultipleRegistrations_IncludesAllScenarios()
        {
            var registry = new ScenarioRegistry();
            var scenario1 = new ScenarioDefinition
            {
                Id = "tutorial",
                DisplayName = "Tutorial",
                Description = "Description",
                WaveCount = 1
            };
            var scenario2 = new ScenarioDefinition
            {
                Id = "campaign",
                DisplayName = "Campaign",
                Description = "Description",
                WaveCount = 3
            };
            var scenario3 = new ScenarioDefinition
            {
                Id = "skirmish",
                DisplayName = "Skirmish",
                Description = "Description",
                WaveCount = 2
            };

            registry.Register(scenario1);
            registry.Register(scenario2);
            registry.Register(scenario3);

            registry.All.Should().HaveCount(3);
        }

        [Fact]
        public void All_ReturnsReadOnlyList()
        {
            var registry = new ScenarioRegistry();
            var scenario = new ScenarioDefinition
            {
                Id = "tutorial",
                DisplayName = "Tutorial",
                Description = "Description",
                WaveCount = 1
            };
            registry.Register(scenario);

            var all = registry.All;

            all.Should().BeAssignableTo<IReadOnlyList<ScenarioDefinition>>();
        }

        [Fact]
        public void Register_MultipleScenarios_IncreasesCount()
        {
            var registry = new ScenarioRegistry();

            var scenario1 = new ScenarioDefinition
            {
                Id = "s1",
                DisplayName = "Scenario 1",
                Description = "Description",
                WaveCount = 1
            };
            var scenario2 = new ScenarioDefinition
            {
                Id = "s2",
                DisplayName = "Scenario 2",
                Description = "Description",
                WaveCount = 2
            };
            var scenario3 = new ScenarioDefinition
            {
                Id = "s3",
                DisplayName = "Scenario 3",
                Description = "Description",
                WaveCount = 3
            };

            registry.Register(scenario1);
            registry.Count.Should().Be(1);

            registry.Register(scenario2);
            registry.Count.Should().Be(2);

            registry.Register(scenario3);
            registry.Count.Should().Be(3);
        }

        [Fact]
        public void GetScenario_AfterMultipleRegistrations_ReturnsLatest()
        {
            var registry = new ScenarioRegistry();
            var scenario1 = new ScenarioDefinition
            {
                Id = "test",
                DisplayName = "First",
                Description = "Description 1",
                WaveCount = 1
            };
            var scenario2 = new ScenarioDefinition
            {
                Id = "test",
                DisplayName = "Second",
                Description = "Description 2",
                WaveCount = 2
            };

            registry.Register(scenario1);
            registry.Register(scenario2);

            var retrieved = registry.GetScenario("test");
            retrieved.DisplayName.Should().Be("Second");
        }

        [Fact]
        public void All_ReturnsUniqueScenarios()
        {
            var registry = new ScenarioRegistry();
            var scenario1 = new ScenarioDefinition
            {
                Id = "test",
                DisplayName = "Test",
                Description = "Description",
                WaveCount = 1
            };
            var scenario2 = new ScenarioDefinition
            {
                Id = "other",
                DisplayName = "Other",
                Description = "Description",
                WaveCount = 2
            };

            registry.Register(scenario1);
            registry.Register(scenario1);
            registry.Register(scenario2);

            registry.All.Should().HaveCount(2);
        }

        [Fact]
        public void Register_DifferentIdsSameData_CountsAsTwo()
        {
            var registry = new ScenarioRegistry();
            var scenario1 = new ScenarioDefinition
            {
                Id = "scenario1",
                DisplayName = "Scenario",
                Description = "Description",
                WaveCount = 1
            };
            var scenario2 = new ScenarioDefinition
            {
                Id = "scenario2",
                DisplayName = "Scenario",
                Description = "Description",
                WaveCount = 1
            };

            registry.Register(scenario1);
            registry.Register(scenario2);

            registry.Count.Should().Be(2);
        }

        [Fact]
        public void Contains_AfterMultipleRegistrations_WorksCorrectly()
        {
            var registry = new ScenarioRegistry();

            var scenario1 = new ScenarioDefinition
            {
                Id = "s1",
                DisplayName = "Scenario 1",
                Description = "Description",
                WaveCount = 1
            };
            var scenario2 = new ScenarioDefinition
            {
                Id = "s2",
                DisplayName = "Scenario 2",
                Description = "Description",
                WaveCount = 2
            };

            registry.Register(scenario1);
            registry.Register(scenario2);

            registry.Contains("s1").Should().BeTrue();
            registry.Contains("s2").Should().BeTrue();
            registry.Contains("s3").Should().BeFalse();
        }
    }
}
