using System;
using System.Collections.Generic;

namespace DINOForge.Domains.Scenario.Scripting
{
    /// <summary>
    /// Snapshot of the current game state, used for evaluating scenario conditions
    /// and scripted event triggers. This is a lightweight data container, not a live
    /// reference to game internals.
    /// </summary>
    public sealed class GameState
    {
        /// <summary>
        /// The current wave number (1-based). Zero means no waves have started.
        /// </summary>
        public int CurrentWave { get; set; } = 0;

        /// <summary>
        /// Total elapsed time in seconds since the scenario started.
        /// </summary>
        public double ElapsedSeconds { get; set; } = 0.0;

        /// <summary>
        /// Current player population count.
        /// </summary>
        public int Population { get; set; } = 0;

        /// <summary>
        /// Current resource amounts keyed by resource name (food, wood, stone, iron, gold).
        /// </summary>
        public Dictionary<string, int> Resources { get; set; } = new Dictionary<string, int>(StringComparer.Ordinal);

        /// <summary>
        /// Set of building IDs that have been constructed at least once during the scenario.
        /// </summary>
        public HashSet<string> BuildingsBuilt { get; set; } = new HashSet<string>();

        /// <summary>
        /// Total number of enemy units killed during the scenario.
        /// </summary>
        public int UnitsKilled { get; set; } = 0;

        /// <summary>
        /// Whether the player's command center is still alive.
        /// </summary>
        public bool CommandCenterAlive { get; set; } = true;
    }
}
