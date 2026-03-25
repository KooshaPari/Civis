using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.UI.Models;

namespace DINOForge.Domains.UI.Registries
{
    /// <summary>
    /// Registry of HUD element definitions. Manages all HUD elements (health bars, resource counters, minimaps, etc.)
    /// registered across all packs.
    /// </summary>
    public class HudElementRegistry
    {
        private readonly Dictionary<string, HudElementDefinition> _elements =
            new Dictionary<string, HudElementDefinition>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// All registered HUD elements.
        /// </summary>
        public IReadOnlyList<HudElementDefinition> All => _elements.Values.ToList().AsReadOnly();

        /// <summary>
        /// Number of registered HUD elements.
        /// </summary>
        public int Count => _elements.Count;

        /// <summary>
        /// Retrieve a HUD element definition by its identifier.
        /// </summary>
        /// <param name="id">HUD element identifier (e.g. "player-health-bar").</param>
        /// <returns>The matching HUD element definition.</returns>
        /// <exception cref="KeyNotFoundException">Thrown when no HUD element with the given id exists.</exception>
        public HudElementDefinition GetElement(string id)
        {
            if (_elements.TryGetValue(id, out HudElementDefinition? element))
                return element;

            throw new KeyNotFoundException($"No HUD element registered with id '{id}'.");
        }

        /// <summary>
        /// Try to retrieve a HUD element definition by its identifier.
        /// </summary>
        /// <param name="id">HUD element identifier.</param>
        /// <param name="element">The matching HUD element definition, or null if not found.</param>
        /// <returns>True if found.</returns>
        public bool TryGetElement(string id, out HudElementDefinition? element)
        {
            return _elements.TryGetValue(id, out element);
        }

        /// <summary>
        /// Check if a HUD element with the given identifier is registered.
        /// </summary>
        /// <param name="id">HUD element identifier.</param>
        /// <returns>True if registered.</returns>
        public bool Contains(string id)
        {
            return _elements.ContainsKey(id);
        }

        /// <summary>
        /// Register a HUD element definition.
        /// </summary>
        /// <param name="element">The HUD element definition to register.</param>
        public void Register(HudElementDefinition element)
        {
            if (element == null) throw new ArgumentNullException(nameof(element));
            if (string.IsNullOrWhiteSpace(element.Id)) throw new ArgumentException("HUD element ID cannot be empty.", nameof(element));
            _elements[element.Id] = element;
        }

        /// <summary>
        /// Unregister a HUD element by identifier.
        /// </summary>
        /// <param name="id">HUD element identifier.</param>
        /// <returns>True if a HUD element was removed; false if not found.</returns>
        public bool Unregister(string id)
        {
            return _elements.Remove(id);
        }

        /// <summary>
        /// Get all HUD elements of a specific type.
        /// </summary>
        /// <param name="type">The HUD element type (e.g. "health_bar").</param>
        /// <returns>A list of HUD elements matching the type.</returns>
        public IReadOnlyList<HudElementDefinition> GetElementsByType(string type)
        {
            if (string.IsNullOrWhiteSpace(type))
                return new List<HudElementDefinition>().AsReadOnly();

            return _elements.Values
                .Where(e => string.Equals(e.Type, type, StringComparison.OrdinalIgnoreCase))
                .ToList()
                .AsReadOnly();
        }

        /// <summary>
        /// Get all HUD elements visible in a specific game state.
        /// </summary>
        /// <param name="state">The game state (e.g. "gameplay", "pause", "main_menu").</param>
        /// <returns>A list of HUD elements visible in that state.</returns>
        public IReadOnlyList<HudElementDefinition> GetElementsByVisibility(string state)
        {
            if (string.IsNullOrWhiteSpace(state))
                return new List<HudElementDefinition>().AsReadOnly();

            return _elements.Values
                .Where(e => e.VisibleIn != null && e.VisibleIn.Any(s => string.Equals(s, state, StringComparison.OrdinalIgnoreCase)))
                .ToList()
                .AsReadOnly();
        }
    }
}
