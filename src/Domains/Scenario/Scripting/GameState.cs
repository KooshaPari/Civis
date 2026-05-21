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
        /// Set of entity IDs (units, buildings, etc.) currently alive in the ECS world.
        /// Populated by Runtime-side callers that have access to the ECS bridge; the
        /// Scenario domain itself never queries ECS directly (architectural boundary).
        /// When empty, DestroyTarget falls back to <see cref="BuildingsBuilt"/> for
        /// backward compatibility with callers that only populate construction state.
        /// </summary>
        public HashSet<string> LiveEntities { get; set; } = new HashSet<string>(StringComparer.Ordinal);

        /// <summary>
        /// Set of entity IDs (units, buildings, etc.) that have been destroyed during the
        /// scenario. Populated by Runtime-side callers that observe ECS destruction events;
        /// the Scenario domain itself never queries ECS directly (architectural boundary).
        /// </summary>
        public HashSet<string> DestroyedEntities { get; set; } = new HashSet<string>(StringComparer.Ordinal);

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
