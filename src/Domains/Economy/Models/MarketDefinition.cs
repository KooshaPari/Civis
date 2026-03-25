using System;
using System.Collections.Generic;

namespace DINOForge.Domains.Economy.Models
{
    /// <summary>
    /// Defines a market building configuration: which resources are traded and custom pricing modifiers.
    /// </summary>
    public class MarketDefinition
    {
        /// <summary>
        /// Unique identifier for this market definition.
        /// </summary>
        public string Id { get; set; }

        /// <summary>
        /// Display name for the market building.
        /// </summary>
        public string DisplayName { get; set; }

        /// <summary>
        /// Description of this market's role and capabilities.
        /// </summary>
        public string Description { get; set; }

        /// <summary>
        /// List of resource IDs that can be traded through this market.
        /// </summary>
        public List<string> ResourcesTradeable { get; set; }

        /// <summary>
        /// Per-resource price modifiers (e.g., 0.8 for 20% discount, 1.2 for 20% markup).
        /// </summary>
        public Dictionary<string, float> PriceModifiers { get; set; }

        /// <summary>
        /// Multiplier for transaction throughput (cargo capacity per transaction).
        /// </summary>
        public float ThroughputModifier { get; set; }

        /// <summary>
        /// Initializes a new market definition with default values.
        /// </summary>
        public MarketDefinition()
        {
            Id = string.Empty;
            DisplayName = string.Empty;
            Description = string.Empty;
            ResourcesTradeable = new List<string>();
            PriceModifiers = new Dictionary<string, float>(StringComparer.OrdinalIgnoreCase);
            ThroughputModifier = 1.0f;
        }

        /// <summary>
        /// Initializes a new market definition with all properties.
        /// </summary>
        public MarketDefinition(
            string id,
            string displayName,
            string description,
            List<string> resourcesTradeable,
            Dictionary<string, float> priceModifiers,
            float throughputModifier)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            DisplayName = displayName ?? throw new ArgumentNullException(nameof(displayName));
            Description = description ?? string.Empty;
            ResourcesTradeable = resourcesTradeable ?? new List<string>();
            PriceModifiers = priceModifiers ?? new Dictionary<string, float>(StringComparer.OrdinalIgnoreCase);
            ThroughputModifier = throughputModifier;
        }

        /// <summary>
        /// Get the price modifier for a resource, with fallback to 1.0 if not defined.
        /// </summary>
        public float GetPriceModifier(string resourceId)
        {
            return PriceModifiers.TryGetValue(resourceId, out float modifier) ? modifier : 1.0f;
        }

        /// <summary>
        /// Check if a resource can be traded through this market.
        /// </summary>
        public bool CanTrade(string resourceId)
        {
            return ResourcesTradeable.Contains(resourceId);
        }
    }
}
