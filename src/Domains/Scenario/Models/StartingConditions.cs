using System;
using System.Collections.Generic;

namespace DINOForge.Domains.Scenario.Models
{
    /// <summary>
    /// Defines the initial state of a scenario: starting resources, enabled factions, and difficulty.
    /// </summary>
    public class StartingConditions
    {
        /// <summary>
        /// Initial resource amounts keyed by resource ID (e.g. "food", "wood").
        /// </summary>
        public Dictionary<string, float> StartingResources { get; set; }

        /// <summary>
        /// List of faction IDs that are enabled and playable in this scenario.
        /// </summary>
        public List<string> EnabledFactions { get; set; }

        /// <summary>
        /// Difficulty multiplier applied to enemy wave strength, resource costs, and timing.
        /// 1.0 = normal, 0.5 = easy, 2.0 = hard.
        /// </summary>
        public float DifficultyMultiplier { get; set; }

        /// <summary>
        /// Random seed for map generation and procedural elements. 0 = use system random.
        /// </summary>
        public int MapSeed { get; set; }

        /// <summary>
        /// Initializes a new starting conditions with default values.
        /// </summary>
        public StartingConditions()
        {
            StartingResources = new Dictionary<string, float>(StringComparer.OrdinalIgnoreCase);
            EnabledFactions = new List<string>();
            DifficultyMultiplier = 1.0f;
            MapSeed = 0;
        }

        /// <summary>
        /// Initializes a new starting conditions with specified properties.
        /// </summary>
        public StartingConditions(
            Dictionary<string, float> startingResources,
            List<string> enabledFactions,
            float difficultyMultiplier,
            int mapSeed)
        {
            StartingResources = startingResources ?? new Dictionary<string, float>(StringComparer.OrdinalIgnoreCase);
            EnabledFactions = enabledFactions ?? new List<string>();
            DifficultyMultiplier = difficultyMultiplier;
            MapSeed = mapSeed;
        }
    }
}
