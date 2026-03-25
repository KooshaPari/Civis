using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.Domains.Scenario.Models;
using DINOForge.Domains.Scenario.Registries;
using DINOForge.SDK;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.Domains.Scenario
{
    /// <summary>
    /// Loads scenario definitions from pack directories into the scenario registry.
    /// Handles scenarios/ subdirectories containing YAML scenario definitions.
    /// </summary>
    public class ScenarioContentLoader
    {
        private readonly ScenarioRegistry _scenarioRegistry;
        private readonly IDeserializer _deserializer;

        /// <summary>
        /// Initializes a new scenario content loader with the provided registry.
        /// </summary>
        /// <param name="scenarioRegistry">The registry to load scenarios into.</param>
        public ScenarioContentLoader(ScenarioRegistry scenarioRegistry)
        {
            _scenarioRegistry = scenarioRegistry ?? throw new ArgumentNullException(nameof(scenarioRegistry));

            _deserializer = new DeserializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .Build();
        }

        /// <summary>
        /// Load all scenario definitions from a pack directory.
        /// </summary>
        /// <param name="packDir">The root directory of the pack.</param>
        /// <param name="packId">The pack identifier (for logging and error reporting).</param>
        public void LoadPack(string packDir, string packId)
        {
            if (!Directory.Exists(packDir))
                throw new DirectoryNotFoundException($"Pack directory not found: {packDir}");

            LoadScenarios(Path.Combine(packDir, "scenarios"), packId);
        }

        /// <summary>
        /// Load all scenario definitions from a specific directory.
        /// </summary>
        /// <param name="scenariosDir">Directory containing scenario YAML files.</param>
        /// <param name="packId">Pack identifier for error reporting.</param>
        private void LoadScenarios(string scenariosDir, string packId)
        {
            if (!Directory.Exists(scenariosDir))
                return;

            string[] files = Directory.GetFiles(scenariosDir, "*.yaml", SearchOption.AllDirectories);
            foreach (string file in files)
            {
                try
                {
                    string yaml = File.ReadAllText(file);
                    ScenarioDefinition scenario = _deserializer.Deserialize<ScenarioDefinition>(yaml);
                    if (scenario != null)
                    {
                        _scenarioRegistry.Register(scenario);
                    }
                }
                catch (Exception ex)
                {
                    throw new InvalidOperationException(
                        $"Failed to load scenario from {file} in pack '{packId}'.", ex);
                }
            }
        }
    }
}
