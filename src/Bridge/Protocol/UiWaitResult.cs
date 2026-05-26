#nullable enable
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of waiting for a UI selector to reach a requested state.
    /// </summary>
    public sealed class UiWaitResult
    {
        /// <summary>Whether the requested state was observed before timeout.</summary>
        [JsonProperty("ready")]
        public bool Ready { get; set; }

        /// <summary>The selector string used for the wait.</summary>
        [JsonProperty("selector")]
        public string Selector { get; set; } = "";

        /// <summary>The target state that was requested.</summary>
        [JsonProperty("state")]
        public string State { get; set; } = "";

        /// <summary>Human-readable wait status message.</summary>
        [JsonProperty("message")]
        public string Message { get; set; } = "";

        /// <summary>The last matched node observed while waiting, when available.</summary>
        [JsonProperty("matchedNode", NullValueHandling = NullValueHandling.Ignore)]
        public UiNode? MatchedNode { get; set; }

        /// <summary>Total matches observed on the final poll.</summary>
        [JsonProperty("matchCount")]
        public int MatchCount { get; set; }
    }
}
