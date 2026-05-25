#nullable enable
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of applying a stat override to entities.
    /// </summary>
    public sealed class OverrideResult
    {
        /// <summary>Whether the override was applied successfully.</summary>
        [JsonProperty("success")]
        public bool Success { get; set; }

        /// <summary>Number of entities that were modified.</summary>
        [JsonProperty("modifiedCount")]
        public int ModifiedCount { get; set; }

        /// <summary>The SDK model path that was overridden.</summary>
        [JsonProperty("sdkPath")]
        public string SdkPath { get; set; } = "";

        /// <summary>Descriptive message about the operation.</summary>
        [JsonProperty("message")]
        public string Message { get; set; } = "";
    }
}
