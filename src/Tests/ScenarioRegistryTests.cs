using System;
using System.Collections.Generic;
using DINOForge.Domains.Scenario.Models;
using DINOForge.Domains.Scenario.Registries;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for the ScenarioRegistry class.
    /// </summary>
    public class ScenarioRegistryTests
    {
        // ── Registration & Lookup ────────────────────────────────

        [Fact]
        public void ScenarioRegistry_Register_StoresScenario()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioDefinition scenario = new ScenarioDefinition { Id = "test-1", DisplayName = "Test Scenario" };

            registry.Register(scenario);

            registry.Contains("test-1").Should().BeTrue();
        }

        [Fact]
        public void ScenarioRegistry_Register_NullThrows()
        {
            ScenarioRegistry registry = new ScenarioRegistry();

            Action act = () => registry.Register(null!);

            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void ScenarioRegistry_Register_EmptyIdThrows()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioDefinition scenario = new ScenarioDefinition { Id = "", DisplayName = "Bad" };

            Action act = () => registry.Register(scenario);

            act.Should().Throw<ArgumentException>();
        }

        [Fact]
        public void ScenarioRegistry_GetScenario_ReturnsRegisteredScenario()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioDefinition scenario = new ScenarioDefinition { Id = "test-2", DisplayName = "Test Scenario 2" };
            registry.Register(scenario);

            ScenarioDefinition retrieved = registry.GetScenario("test-2");

            retrieved.Id.Should().Be("test-2");
            retrieved.DisplayName.Should().Be("Test Scenario 2");
        }

        [Fact]
        public void ScenarioRegistry_GetScenario_ThrowsForMissingId()
        {
            ScenarioRegistry registry = new ScenarioRegistry();

            Action act = () => registry.GetScenario("nonexistent");

            act.Should().Throw<KeyNotFoundException>();
        }

        [Fact]
        public void ScenarioRegistry_TryGetScenario_ReturnsTrueForExisting()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioDefinition scenario = new ScenarioDefinition { Id = "test-3" };
            registry.Register(scenario);

            bool result = registry.TryGetScenario("test-3", out ScenarioDefinition? retrieved);

            result.Should().BeTrue();
            retrieved.Should().NotBeNull();
            retrieved!.Id.Should().Be("test-3");
        }

        [Fact]
        public void ScenarioRegistry_TryGetScenario_ReturnsFalseForMissing()
        {
            ScenarioRegistry registry = new ScenarioRegistry();

            bool result = registry.TryGetScenario("missing", out ScenarioDefinition? retrieved);

            result.Should().BeFalse();
            retrieved.Should().BeNull();
        }

        [Fact]
        public void ScenarioRegistry_Contains_ReturnsTrueForRegistered()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioDefinition scenario = new ScenarioDefinition { Id = "test-4" };
            registry.Register(scenario);

            registry.Contains("test-4").Should().BeTrue();
        }

        [Fact]
        public void ScenarioRegistry_Contains_ReturnsFalseForUnregistered()
        {
            ScenarioRegistry registry = new ScenarioRegistry();

            registry.Contains("missing").Should().BeFalse();
        }

        // ── Count & All ──────────────────────────────────────────

        [Fact]
        public void ScenarioRegistry_Count_ReturnsCorrectNumber()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            registry.Register(new ScenarioDefinition { Id = "s1" });
            registry.Register(new ScenarioDefinition { Id = "s2" });
            registry.Register(new ScenarioDefinition { Id = "s3" });

            registry.Count.Should().Be(3);
        }

        [Fact]
        public void ScenarioRegistry_All_ReturnsAllScenarios()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            registry.Register(new ScenarioDefinition { Id = "s1", DisplayName = "Scenario 1" });
            registry.Register(new ScenarioDefinition { Id = "s2", DisplayName = "Scenario 2" });

            IReadOnlyList<ScenarioDefinition> all = registry.All;

            all.Should().HaveCount(2);
            all.Should().Contain(s => s.Id == "s1");
            all.Should().Contain(s => s.Id == "s2");
        }

        // ── Unregistration ───────────────────────────────────────

        [Fact]
        public void ScenarioRegistry_Unregister_RemovesScenario()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            registry.Register(new ScenarioDefinition { Id = "to-remove" });

            bool removed = registry.Unregister("to-remove");

            removed.Should().BeTrue();
            registry.Contains("to-remove").Should().BeFalse();
        }

        [Fact]
        public void ScenarioRegistry_Unregister_ReturnsFalseForMissing()
        {
            ScenarioRegistry registry = new ScenarioRegistry();

            bool removed = registry.Unregister("missing");

            removed.Should().BeFalse();
        }

        // ── Case Insensitivity ───────────────────────────────────

        [Fact]
        public void ScenarioRegistry_Lookup_IsCaseInsensitive()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            registry.Register(new ScenarioDefinition { Id = "MyScenario" });

            registry.Contains("myscenario").Should().BeTrue();
            registry.Contains("MYSCENARIO").Should().BeTrue();
            registry.GetScenario("myscenario").Id.Should().Be("MyScenario");
        }

        // ── Filtering by Difficulty ─────────────────────────────

        [Fact]
        public void ScenarioRegistry_GetByDifficulty_FiltersCorrectly()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            registry.Register(new ScenarioDefinition { Id = "easy-1", Difficulty = Difficulty.Easy });
            registry.Register(new ScenarioDefinition { Id = "normal-1", Difficulty = Difficulty.Normal });
            registry.Register(new ScenarioDefinition { Id = "hard-1", Difficulty = Difficulty.Hard });
            registry.Register(new ScenarioDefinition { Id = "easy-2", Difficulty = Difficulty.Easy });

            IReadOnlyList<ScenarioDefinition> easy = registry.GetScenariosByDifficulty(Difficulty.Easy);

            easy.Should().HaveCount(2);
            easy.Should().Contain(s => s.Id == "easy-1");
            easy.Should().Contain(s => s.Id == "easy-2");
        }

        [Fact]
        public void ScenarioRegistry_GetByDifficulty_ReturnsEmptyWhenNoneMatch()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            registry.Register(new ScenarioDefinition { Id = "normal-1", Difficulty = Difficulty.Normal });

            IReadOnlyList<ScenarioDefinition> nightmare = registry.GetScenariosByDifficulty(Difficulty.Nightmare);

            nightmare.Should().HaveCount(0);
        }

        // ── Filtering by Objective Type ──────────────────────────

        [Fact]
        public void ScenarioRegistry_GetByObjective_FiltersCorrectly()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            registry.Register(new ScenarioDefinition { Id = "survive-1", ObjectiveType = ObjectiveType.Survive });
            registry.Register(new ScenarioDefinition { Id = "defend-1", ObjectiveType = ObjectiveType.Defend });
            registry.Register(new ScenarioDefinition { Id = "attack-1", ObjectiveType = ObjectiveType.Attack });
            registry.Register(new ScenarioDefinition { Id = "defend-2", ObjectiveType = ObjectiveType.Defend });

            IReadOnlyList<ScenarioDefinition> defend = registry.GetScenariosByObjective(ObjectiveType.Defend);

            defend.Should().HaveCount(2);
            defend.Should().Contain(s => s.Id == "defend-1");
            defend.Should().Contain(s => s.Id == "defend-2");
        }

        [Fact]
        public void ScenarioRegistry_GetByObjective_ReturnsEmptyWhenNoneMatch()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            registry.Register(new ScenarioDefinition { Id = "s1", ObjectiveType = ObjectiveType.Survive });

            IReadOnlyList<ScenarioDefinition> prosper = registry.GetScenariosByObjective(ObjectiveType.Prosper);

            prosper.Should().HaveCount(0);
        }

        // ── Clear ────────────────────────────────────────────────

        [Fact]
        public void ScenarioRegistry_Clear_RemovesAllScenarios()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            registry.Register(new ScenarioDefinition { Id = "s1" });
            registry.Register(new ScenarioDefinition { Id = "s2" });
            registry.Register(new ScenarioDefinition { Id = "s3" });

            registry.Clear();

            registry.Count.Should().Be(0);
            registry.All.Should().HaveCount(0);
        }

        // ── Scenario Overwrite ───────────────────────────────────

        [Fact]
        public void ScenarioRegistry_Register_OverwritesPreviousScenario()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioDefinition first = new ScenarioDefinition { Id = "same", DisplayName = "First" };
            ScenarioDefinition second = new ScenarioDefinition { Id = "same", DisplayName = "Second" };

            registry.Register(first);
            registry.Register(second);

            registry.Count.Should().Be(1);
            registry.GetScenario("same").DisplayName.Should().Be("Second");
        }
    }
}
