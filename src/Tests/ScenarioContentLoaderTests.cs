using System;
using System.IO;
using System.Text;
using DINOForge.Domains.Scenario;
using DINOForge.Domains.Scenario.Models;
using DINOForge.Domains.Scenario.Registries;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for the ScenarioContentLoader class.
    /// </summary>
    public class ScenarioContentLoaderTests
    {
        // ── Initialization ───────────────────────────────────────

        [Fact]
        public void ScenarioContentLoader_Constructor_NullRegistryThrows()
        {
            Action act = () => new ScenarioContentLoader(null!);

            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void ScenarioContentLoader_Constructor_CreatesSuccessfully()
        {
            ScenarioRegistry registry = new ScenarioRegistry();

            ScenarioContentLoader loader = new ScenarioContentLoader(registry);

            loader.Should().NotBeNull();
        }

        // ── Pack Loading with Missing Directory ──────────────────

        [Fact]
        public void ScenarioContentLoader_LoadPack_ThrowsForMissingDirectory()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioContentLoader loader = new ScenarioContentLoader(registry);

            Action act = () => loader.LoadPack("/nonexistent/pack", "test-pack");

            act.Should().Throw<DirectoryNotFoundException>();
        }

        // ── Pack Loading with Empty Directory ────────────────────

        [Fact]
        public void ScenarioContentLoader_LoadPack_HandlesEmptyDirectory()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioContentLoader loader = new ScenarioContentLoader(registry);

            string tempDir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
            Directory.CreateDirectory(tempDir);

            try
            {
                // Should not throw for empty directory
                loader.LoadPack(tempDir, "empty-pack");

                registry.Count.Should().Be(0);
            }
            finally
            {
                Directory.Delete(tempDir, true);
            }
        }

        // ── Pack Loading with Scenarios ──────────────────────────

        [Fact]
        public void ScenarioContentLoader_LoadPack_LoadsValidScenario()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioContentLoader loader = new ScenarioContentLoader(registry);

            string tempDir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
            string scenariosDir = Path.Combine(tempDir, "scenarios");
            Directory.CreateDirectory(scenariosDir);

            try
            {
                // Create a simple scenario YAML file
                string scenarioYaml = @"
id: test-scenario
display_name: Test Scenario
description: A test scenario
difficulty: Normal
objective_type: Survive
wave_count: 5
max_duration: 600
starting_resources:
  food: 100
  wood: 50
allowed_factions:
  - faction1
  - faction2
victory_conditions:
  - condition_type: SurviveWaves
    target_value: 5
defeat_conditions:
  - condition_type: CommandCenterDestroyed
scripted_events: []
";
                string scenarioFile = Path.Combine(scenariosDir, "test.yaml");
                File.WriteAllText(scenarioFile, scenarioYaml, Encoding.UTF8);

                loader.LoadPack(tempDir, "test-pack");

                registry.Count.Should().Be(1);
                registry.Contains("test-scenario").Should().BeTrue();
                registry.GetScenario("test-scenario").DisplayName.Should().Be("Test Scenario");
            }
            finally
            {
                Directory.Delete(tempDir, true);
            }
        }

        // ── Pack Loading with Malformed YAML ─────────────────────

        [Fact]
        public void ScenarioContentLoader_LoadPack_ThrowsForMalformedYaml()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioContentLoader loader = new ScenarioContentLoader(registry);

            string tempDir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
            string scenariosDir = Path.Combine(tempDir, "scenarios");
            Directory.CreateDirectory(scenariosDir);

            try
            {
                // Create a malformed YAML file
                string malformedYaml = @"
id: test
display_name: Test
[invalid yaml content
";
                string scenarioFile = Path.Combine(scenariosDir, "bad.yaml");
                File.WriteAllText(scenarioFile, malformedYaml, Encoding.UTF8);

                Action act = () => loader.LoadPack(tempDir, "test-pack");

                act.Should().Throw<InvalidOperationException>()
                    .WithInnerException<Exception>();
            }
            finally
            {
                Directory.Delete(tempDir, true);
            }
        }

        // ── Pack Loading with Multiple Scenarios ─────────────────

        [Fact]
        public void ScenarioContentLoader_LoadPack_LoadsMultipleScenarios()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioContentLoader loader = new ScenarioContentLoader(registry);

            string tempDir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
            string scenariosDir = Path.Combine(tempDir, "scenarios");
            Directory.CreateDirectory(scenariosDir);

            try
            {
                // Create multiple scenario files
                for (int i = 1; i <= 3; i++)
                {
                    string scenarioYaml = $@"
id: scenario-{i}
display_name: Scenario {i}
description: Test scenario {i}
difficulty: Normal
objective_type: Survive
wave_count: 5
max_duration: 600
starting_resources:
  food: 100
allowed_factions:
  - faction1
victory_conditions:
  - condition_type: SurviveWaves
    target_value: 5
defeat_conditions:
  - condition_type: CommandCenterDestroyed
scripted_events: []
";
                    string scenarioFile = Path.Combine(scenariosDir, $"scenario-{i}.yaml");
                    File.WriteAllText(scenarioFile, scenarioYaml, Encoding.UTF8);
                }

                loader.LoadPack(tempDir, "multi-pack");

                registry.Count.Should().Be(3);
                registry.Contains("scenario-1").Should().BeTrue();
                registry.Contains("scenario-2").Should().BeTrue();
                registry.Contains("scenario-3").Should().BeTrue();
            }
            finally
            {
                Directory.Delete(tempDir, true);
            }
        }

        // ── Pack Loading with Nested Directories ─────────────────

        [Fact]
        public void ScenarioContentLoader_LoadPack_LoadsFromNestedDirectories()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioContentLoader loader = new ScenarioContentLoader(registry);

            string tempDir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
            string scenariosDir = Path.Combine(tempDir, "scenarios");
            string nestedDir = Path.Combine(scenariosDir, "tutorial");
            Directory.CreateDirectory(nestedDir);

            try
            {
                // Create a scenario in nested directory
                string scenarioYaml = @"
id: tutorial-1
display_name: Tutorial 1
description: First tutorial
difficulty: Easy
objective_type: Survive
wave_count: 3
max_duration: 300
starting_resources:
  food: 200
allowed_factions:
  - player
victory_conditions:
  - condition_type: SurviveWaves
    target_value: 3
defeat_conditions:
  - condition_type: CommandCenterDestroyed
scripted_events: []
";
                string scenarioFile = Path.Combine(nestedDir, "tutorial-1.yaml");
                File.WriteAllText(scenarioFile, scenarioYaml, Encoding.UTF8);

                loader.LoadPack(tempDir, "tutorial-pack");

                registry.Contains("tutorial-1").Should().BeTrue();
                registry.GetScenario("tutorial-1").Difficulty.Should().Be(Difficulty.Easy);
            }
            finally
            {
                Directory.Delete(tempDir, true);
            }
        }

        // ── Scenario Properties ──────────────────────────────────

        [Fact]
        public void ScenarioContentLoader_LoadPack_PreservesAllScenarioProperties()
        {
            ScenarioRegistry registry = new ScenarioRegistry();
            ScenarioContentLoader loader = new ScenarioContentLoader(registry);

            string tempDir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
            string scenariosDir = Path.Combine(tempDir, "scenarios");
            Directory.CreateDirectory(scenariosDir);

            try
            {
                string scenarioYaml = @"
id: comprehensive-test
display_name: Comprehensive Test Scenario
description: A comprehensive test with all properties
difficulty: Hard
objective_type: Defend
wave_count: 10
max_duration: 1800
starting_resources:
  food: 500
  wood: 300
  stone: 200
  iron: 100
  gold: 50
allowed_factions:
  - faction-a
  - faction-b
  - faction-c
victory_conditions:
  - condition_type: SurviveWaves
    target_value: 10
defeat_conditions:
  - condition_type: CommandCenterDestroyed
scripted_events: []
";
                string scenarioFile = Path.Combine(scenariosDir, "comprehensive.yaml");
                File.WriteAllText(scenarioFile, scenarioYaml, Encoding.UTF8);

                loader.LoadPack(tempDir, "comprehensive-pack");

                ScenarioDefinition scenario = registry.GetScenario("comprehensive-test");
                scenario.DisplayName.Should().Be("Comprehensive Test Scenario");
                scenario.Difficulty.Should().Be(Difficulty.Hard);
                scenario.ObjectiveType.Should().Be(ObjectiveType.Defend);
                scenario.WaveCount.Should().Be(10);
                scenario.MaxDuration.Should().Be(1800);
                scenario.AllowedFactions.Should().HaveCount(3);
            }
            finally
            {
                Directory.Delete(tempDir, true);
            }
        }
    }
}
