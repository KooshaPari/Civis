#nullable enable
using System.Collections.Generic;
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of querying the SDK-to-ECS component type mapping table.
    /// </summary>
    public sealed class ComponentMapResult
    {
        /// <summary>List of component mappings matching the query.</summary>
        [JsonProperty("mappings")]
        public List<ComponentMapEntry> Mappings { get; set; } = new List<ComponentMapEntry>(); // public-mutable-ok: JSON deserializer requires mutable List
    }

    /// <summary>
    /// A single entry in the component mapping table, linking an SDK model path
    /// to an ECS component type and field.
    /// </summary>
    public sealed class ComponentMapEntry
    {
        /// <summary>Dot-separated SDK model path (e.g. "unit.stats.hp").</summary>
        [JsonProperty("sdkPath")]
        public string SdkPath { get; set; } = "";

        /// <summary>Full ECS component type name (e.g. "Components.Health").</summary>
        [JsonProperty("ecsType")]
        public string EcsType { get; set; } = "";

        /// <summary>Target field name on the component, if applicable.</summary>
        [JsonProperty("fieldName")]
        public string FieldName { get; set; } = "";

        /// <summary>Whether the CLR type was successfully resolved at runtime.</summary>
        [JsonProperty("resolved")]
        public bool Resolved { get; set; }

        /// <summary>Human-readable description of the mapping.</summary>
        [JsonProperty("description")]
        public string Description { get; set; } = "";
    }
}
