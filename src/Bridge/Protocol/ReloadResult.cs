#nullable enable
using System.Collections.Generic;
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of reloading content packs from disk.
    /// </summary>
    public sealed class ReloadResult
    {
        /// <summary>Whether the reload completed without fatal errors.</summary>
        [JsonProperty("success")]
        public bool Success { get; set; }

        /// <summary>List of pack IDs that were loaded.</summary>
        [JsonProperty("loadedPacks")]
        public List<string> LoadedPacks { get; set; } = new List<string>(); // public-mutable-ok: JSON deserializer requires mutable List

        /// <summary>Any errors encountered during the reload.</summary>
        [JsonProperty("errors")]
        public List<string> Errors { get; set; } = new List<string>(); // public-mutable-ok: JSON deserializer requires mutable List
    }
}
