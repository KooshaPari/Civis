#nullable enable
using System.Collections.Generic;
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// A point-in-time snapshot of the vanilla entity catalog, organized by category.
    /// </summary>
    public sealed class CatalogSnapshot
    {
        /// <summary>Unit archetype entries discovered in the ECS world.</summary>
        [JsonProperty("units")]
        public List<CatalogEntry> Units { get; set; } = new List<CatalogEntry>(); // public-mutable-ok: JSON deserializer requires mutable List

        /// <summary>Building archetype entries.</summary>
        [JsonProperty("buildings")]
        public List<CatalogEntry> Buildings { get; set; } = new List<CatalogEntry>(); // public-mutable-ok: JSON deserializer requires mutable List

        /// <summary>Projectile archetype entries.</summary>
        [JsonProperty("projectiles")]
        public List<CatalogEntry> Projectiles { get; set; } = new List<CatalogEntry>(); // public-mutable-ok: JSON deserializer requires mutable List

        /// <summary>Other entity archetypes that do not fit known categories.</summary>
        [JsonProperty("other")]
        public List<CatalogEntry> Other { get; set; } = new List<CatalogEntry>(); // public-mutable-ok: JSON deserializer requires mutable List
    }

    /// <summary>
    /// A single entry in the entity catalog snapshot.
    /// </summary>
    public sealed class CatalogEntry
    {
        /// <summary>Inferred DINOForge registry ID (e.g. "vanilla:melee_unit").</summary>
        [JsonProperty("inferredId")]
        public string InferredId { get; set; } = "";

        /// <summary>Number of ECS component types on this archetype.</summary>
        [JsonProperty("componentCount")]
        public int ComponentCount { get; set; }

        /// <summary>Number of entities with this archetype.</summary>
        [JsonProperty("entityCount")]
        public int EntityCount { get; set; }

        /// <summary>Primary category: unit, building, projectile, or other.</summary>
        [JsonProperty("category")]
        public string Category { get; set; } = "";
    }
}
