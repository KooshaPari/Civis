using System;
using System.Collections.Generic;
using System.Linq;

namespace DINOForge.Domains.Warfare.Archetypes
{
    /// <summary>
    /// Registry of faction archetypes. Pre-loaded with the three canonical archetypes
    /// (Order, Industrial Swarm, Asymmetric) and supports custom archetype registration.
    /// </summary>
    public sealed class ArchetypeRegistry
    {
        private readonly Dictionary<string, FactionArchetype> _archetypes =
            new Dictionary<string, FactionArchetype>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// All registered archetypes.
        /// </summary>
        public IReadOnlyList<FactionArchetype> All => _archetypes.Values.ToList().AsReadOnly();

        /// <summary>
        /// Creates a new archetype registry pre-loaded with the three canonical archetypes.
        /// </summary>
        public ArchetypeRegistry()
        {
            RegisterDefaults();
        }

        /// <summary>
        /// Retrieve an archetype by its identifier.
        /// </summary>
        /// <param name="id">Archetype identifier (e.g. "order").</param>
        /// <returns>The matching archetype.</returns>
        /// <exception cref="KeyNotFoundException">Thrown when no archetype with the given id exists.</exception>
        public FactionArchetype GetArchetype(string id)
        {
            if (_archetypes.TryGetValue(id, out FactionArchetype? archetype))
                return archetype;

            throw new KeyNotFoundException($"No archetype registered with id '{id}'.");
        }

        /// <summary>
        /// Try to retrieve an archetype by its identifier.
        /// </summary>
        /// <param name="id">Archetype identifier.</param>
        /// <param name="archetype">The matching archetype, or null if not found.</param>
        /// <returns>True if found.</returns>
        public bool TryGetArchetype(string id, out FactionArchetype? archetype)
        {
            return _archetypes.TryGetValue(id, out archetype);
        }

        /// <summary>
        /// Register a custom archetype.
        /// </summary>
        /// <param name="archetype">The archetype to register.</param>
        public void Register(FactionArchetype archetype)
        {
            if (archetype == null) throw new ArgumentNullException(nameof(archetype));
            _archetypes[archetype.Id] = archetype;
        }

        private void RegisterDefaults()
        {
            Register(new FactionArchetype(
                "order",
                "Order",
                "Balanced stats, strong defense, disciplined formations.",
                new Dictionary<string, float>(StringComparer.Ordinal)
                {
                    { "armor", 1.10f },
                    { "morale", 1.10f },
                    { "speed", 0.95f }
                }));

            Register(new FactionArchetype(
                "industrial_swarm",
                "Industrial Swarm",
                "Cheap units, fast production, weak individuals, overwhelming numbers.",
                new Dictionary<string, float>(StringComparer.Ordinal)
                {
                    { "cost", 0.80f },
                    { "speed", 1.30f },
                    { "hp", 0.85f },
                    { "damage", 0.90f }
                }));

            Register(new FactionArchetype(
                "asymmetric",
                "Asymmetric",
                "Strong individuals, expensive, ambush tactics, hit-and-run.",
                new Dictionary<string, float>(StringComparer.Ordinal)
                {
                    { "damage", 1.25f },
                    { "speed", 1.15f },
                    { "cost", 1.30f },
                    { "squad_size", 0.80f }
                }));
        }
    }
}
