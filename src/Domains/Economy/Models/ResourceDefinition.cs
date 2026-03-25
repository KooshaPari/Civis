using System;

namespace DINOForge.Domains.Economy.Models
{
    /// <summary>
    /// Defines a resource type in the economy system: production rates, storage capacity, and decay behavior.
    /// </summary>
    public class ResourceDefinition
    {
        /// <summary>
        /// Unique identifier for this resource (e.g. "food", "wood", "stone", "iron", "gold").
        /// </summary>
        public string Id { get; set; }

        /// <summary>
        /// Display name for the resource.
        /// </summary>
        public string Name { get; set; }

        /// <summary>
        /// Description of what this resource represents and its role in the economy.
        /// </summary>
        public string Description { get; set; }

        /// <summary>
        /// Base production rate per second (ticks are typically 0.016s in-game).
        /// </summary>
        public float ProductionRate { get; set; }

        /// <summary>
        /// Maximum storage capacity for this resource per faction.
        /// </summary>
        public float StorageCapacity { get; set; }

        /// <summary>
        /// Per-second decay rate for resources that spoil or degrade over time.
        /// Zero or negative means no decay.
        /// </summary>
        public float DecayRate { get; set; }

        /// <summary>
        /// Whether this resource is tradeable via merchant dirigibles or other trade routes.
        /// </summary>
        public bool IsTradeableDefault { get; set; }

        /// <summary>
        /// Initializes a new resource definition with default values.
        /// </summary>
        public ResourceDefinition()
        {
            Id = string.Empty;
            Name = string.Empty;
            Description = string.Empty;
            ProductionRate = 1.0f;
            StorageCapacity = 1000.0f;
            DecayRate = 0.0f;
            IsTradeableDefault = true;
        }

        /// <summary>
        /// Initializes a new resource definition with all properties.
        /// </summary>
        public ResourceDefinition(
            string id,
            string name,
            string description,
            float productionRate,
            float storageCapacity,
            float decayRate,
            bool isTradeableDefault)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            Name = name ?? throw new ArgumentNullException(nameof(name));
            Description = description ?? string.Empty;
            ProductionRate = productionRate;
            StorageCapacity = storageCapacity;
            DecayRate = decayRate;
            IsTradeableDefault = isTradeableDefault;
        }
    }
}
