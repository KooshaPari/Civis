#nullable enable
using System.Collections.Generic;
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of verifying a mod pack, including load status and detected issues.
    /// </summary>
    public sealed class VerifyResult
    {
        /// <summary>The pack ID from the manifest.</summary>
        [JsonProperty("packId")]
        public string PackId { get; set; } = "";

        /// <summary>Whether the pack was loaded successfully.</summary>
        [JsonProperty("loaded")]
        public bool Loaded { get; set; }

        /// <summary>List of stat changes that would be applied by this pack.</summary>
        [JsonProperty("statChanges")]
        public List<string> StatChanges { get; set; } = new List<string>(); // public-mutable-ok: JSON deserializer requires mutable List

        /// <summary>Errors encountered during verification.</summary>
        [JsonProperty("errors")]
        public List<string> Errors { get; set; } = new List<string>(); // public-mutable-ok: JSON deserializer requires mutable List

        /// <summary>Total entity count affected by this pack.</summary>
        [JsonProperty("entityCount")]
        public int EntityCount { get; set; }
    }
}
