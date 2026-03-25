using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Economy.Models;

namespace DINOForge.Domains.Economy.Registries
{
    /// <summary>
    /// Registry of resource definitions. Pre-loaded with the five canonical resources
    /// (Food, Wood, Stone, Iron, Gold) and supports custom resource registration.
    /// </summary>
    public class ResourceRegistry
    {
        private readonly Dictionary<string, ResourceDefinition> _resources =
            new Dictionary<string, ResourceDefinition>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// All registered resources.
        /// </summary>
        public IReadOnlyList<ResourceDefinition> All => _resources.Values.ToList().AsReadOnly();

        /// <summary>
        /// Number of registered resources.
        /// </summary>
        public int Count => _resources.Count;

        /// <summary>
        /// Creates a new resource registry pre-loaded with the five canonical resources.
        /// </summary>
        public ResourceRegistry()
        {
            RegisterDefaults();
        }

        /// <summary>
        /// Retrieve a resource definition by its identifier.
        /// </summary>
        /// <param name="id">Resource identifier (e.g. "food", "wood").</param>
        /// <returns>The matching resource definition.</returns>
        /// <exception cref="KeyNotFoundException">Thrown when no resource with the given id exists.</exception>
        public ResourceDefinition GetResource(string id)
        {
            if (_resources.TryGetValue(id, out ResourceDefinition? resource))
                return resource;

            throw new KeyNotFoundException($"No resource registered with id '{id}'.");
        }

        /// <summary>
        /// Try to retrieve a resource definition by its identifier.
        /// </summary>
        /// <param name="id">Resource identifier.</param>
        /// <param name="resource">The matching resource definition, or null if not found.</param>
        /// <returns>True if found.</returns>
        public bool TryGetResource(string id, out ResourceDefinition? resource)
        {
            return _resources.TryGetValue(id, out resource);
        }

        /// <summary>
        /// Check if a resource with the given identifier is registered.
        /// </summary>
        /// <param name="id">Resource identifier.</param>
        /// <returns>True if registered.</returns>
        public bool Contains(string id)
        {
            return _resources.ContainsKey(id);
        }

        /// <summary>
        /// Register a custom resource.
        /// </summary>
        /// <param name="resource">The resource definition to register.</param>
        public void Register(ResourceDefinition resource)
        {
            if (resource == null) throw new ArgumentNullException(nameof(resource));
            if (string.IsNullOrWhiteSpace(resource.Id)) throw new ArgumentException("Resource ID cannot be empty.", nameof(resource));
            _resources[resource.Id] = resource;
        }

        /// <summary>
        /// Unregister a resource by identifier.
        /// </summary>
        /// <param name="id">Resource identifier.</param>
        /// <returns>True if a resource was removed; false if not found.</returns>
        public bool Unregister(string id)
        {
            return _resources.Remove(id);
        }

        private void RegisterDefaults()
        {
            Register(new ResourceDefinition(
                "food",
                "Food",
                "Primary nutrition resource. Workers consume food; spoils over time.",
                1.0f,
                1000.0f,
                0.01f,
                true));

            Register(new ResourceDefinition(
                "wood",
                "Wood",
                "Building material and fuel. Essential for construction.",
                0.8f,
                1200.0f,
                0.0f,
                true));

            Register(new ResourceDefinition(
                "stone",
                "Stone",
                "Defensive material. Used for fortifications and walls.",
                0.6f,
                1500.0f,
                0.0f,
                true));

            Register(new ResourceDefinition(
                "iron",
                "Iron",
                "Weapons material. Scarce but critical for military units.",
                0.4f,
                800.0f,
                0.0f,
                true));

            Register(new ResourceDefinition(
                "gold",
                "Gold",
                "Premium currency. Obtained only through trade; cannot be produced.",
                0.0f,
                500.0f,
                0.0f,
                true));
        }
    }
}
