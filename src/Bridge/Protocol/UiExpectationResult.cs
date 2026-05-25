#nullable enable
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of asserting a specific condition on a live Unity UI selector.
    /// </summary>
    public sealed class UiExpectationResult
    {
        /// <summary>Whether the expectation passed.</summary>
        [JsonProperty("success")]
        public bool Success { get; set; }

        /// <summary>The selector string used for the expectation.</summary>
        [JsonProperty("selector")]
        public string Selector { get; set; } = "";

        /// <summary>The expectation condition that was evaluated.</summary>
        [JsonProperty("condition")]
        public string Condition { get; set; } = "";

        /// <summary>Human-readable result message.</summary>
        [JsonProperty("message")]
        public string Message { get; set; } = "";

        /// <summary>The first matched node, when available.</summary>
        [JsonProperty("matchedNode", NullValueHandling = NullValueHandling.Ignore)]
        public UiNode? MatchedNode { get; set; }

        /// <summary>Total number of matches seen during evaluation.</summary>
        [JsonProperty("matchCount")]
        public int MatchCount { get; set; }
    }
}
