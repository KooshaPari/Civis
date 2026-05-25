#nullable enable
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of a ping operation, confirming server liveness.
    /// </summary>
    public sealed class PingResult
    {
        /// <summary>Always true when the server responds.</summary>
        [JsonProperty("pong")]
        public bool Pong { get; set; }

        /// <summary>DINOForge runtime version string.</summary>
        [JsonProperty("version")]
        public string Version { get; set; } = "";

        /// <summary>Server uptime in seconds since the plugin loaded.</summary>
        [JsonProperty("uptimeSeconds")]
        public double UptimeSeconds { get; set; }
    }
}
