#nullable enable
using System.Collections.Generic;
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of reading a stat value from the ECS world.
    /// </summary>
    public sealed class StatResult
    {
        /// <summary>The SDK model path that was queried.</summary>
        [JsonProperty("sdkPath")]
        public string SdkPath { get; set; } = "";

        /// <summary>The first (or averaged) value found.</summary>
        [JsonProperty("value")]
        public float Value { get; set; }

        /// <summary>Number of entities that have this component.</summary>
        [JsonProperty("entityCount")]
        public int EntityCount { get; set; }

        /// <summary>All individual values read from matching entities.</summary>
        [JsonProperty("values")]
        // public-mutable-ok: JSON deserialization requires mutable List<T> for Newtonsoft.Json
        public List<float> Values { get; set; } = new List<float>();

        /// <summary>The resolved ECS component type name.</summary>
        [JsonProperty("componentType")]
        public string ComponentType { get; set; } = "";

        /// <summary>The field name read from the component.</summary>
        [JsonProperty("fieldName")]
        public string FieldName { get; set; } = "";
    }
}
