#nullable enable
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of waiting for the ECS world to become available.
    /// </summary>
    public sealed class WaitResult
    {
        /// <summary>Whether the world is ready (loaded within the timeout).</summary>
        [JsonProperty("ready")]
        public bool Ready { get; set; }

        /// <summary>Name of the world, if ready.</summary>
        [JsonProperty("worldName")]
        public string WorldName { get; set; } = "";
    }
}
