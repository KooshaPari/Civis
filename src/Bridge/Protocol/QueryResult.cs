#nullable enable
using System.Collections.Generic;
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of an entity query operation.
    /// </summary>
    public sealed class QueryResult
    {
        /// <summary>Total number of entities matching the query.</summary>
        [JsonProperty("count")]
        public int Count { get; set; }

        /// <summary>List of matching entity summaries.</summary>
        [JsonProperty("entities")]
        public List<EntityInfo> Entities { get; set; } = new List<EntityInfo>(); // public-mutable-ok: JSON deserializer requires mutable List
    }

    /// <summary>
    /// Summary information about a single entity.
    /// </summary>
    public sealed class EntityInfo
    {
        /// <summary>Entity index in the ECS world.</summary>
        [JsonProperty("index")]
        public int Index { get; set; }

        /// <summary>List of component type names on this entity.</summary>
        [JsonProperty("components")]
        public List<string> Components { get; set; } = new List<string>(); // public-mutable-ok: JSON deserializer requires mutable List
    }
}
