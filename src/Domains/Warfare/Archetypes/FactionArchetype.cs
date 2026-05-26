using System;
using System.Collections.Generic;

namespace DINOForge.Domains.Warfare.Archetypes
{
    /// <summary>
    /// Defines a faction archetype with base stat modifiers that shape how a faction plays.
    /// Archetypes are immutable templates: Order, Industrial Swarm, or Asymmetric.
    /// </summary>
    public sealed class FactionArchetype
    {
        /// <summary>
        /// Unique archetype identifier (e.g. "order", "industrial_swarm", "asymmetric").
        /// </summary>
        public string Id { get; }

        /// <summary>
        /// Human-readable archetype name.
        /// </summary>
        public string DisplayName { get; }

        /// <summary>
        /// Description of the archetype's play style.
        /// </summary>
        public string Description { get; }

        /// <summary>
        /// Base stat modifiers as multipliers. Key is the stat name (e.g. "armor", "speed", "cost"),
        /// value is the multiplier (e.g. 1.10 = +10%, 0.85 = -15%).
        /// </summary>
        public IReadOnlyDictionary<string, float> BaseModifiers { get; }

        /// <summary>
        /// Creates a new faction archetype.
        /// </summary>
        /// <param name="id">Unique archetype identifier.</param>
        /// <param name="displayName">Human-readable name.</param>
        /// <param name="description">Description of the archetype's play style.</param>
        /// <param name="baseModifiers">Stat name to multiplier mappings.</param>
        public FactionArchetype(string id, string displayName, string description, IDictionary<string, float> baseModifiers)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            DisplayName = displayName ?? throw new ArgumentNullException(nameof(displayName));
            Description = description ?? throw new ArgumentNullException(nameof(description));
            if (baseModifiers == null) throw new ArgumentNullException(nameof(baseModifiers));
            var modifiers = new Dictionary<string, float>(StringComparer.OrdinalIgnoreCase);
            foreach (var kvp in baseModifiers) modifiers[kvp.Key] = kvp.Value;
            BaseModifiers = modifiers;
        }
    }
}
