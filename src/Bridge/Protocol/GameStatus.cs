#nullable enable
using System.Collections.Generic;
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Describes the current state of the game and mod platform.
    /// </summary>
    public sealed class GameStatus
    {
        /// <summary>Whether the game process is running.</summary>
        [JsonProperty("running")]
        public bool Running { get; set; }

        /// <summary>Whether the ECS world is loaded and ready.</summary>
        [JsonProperty("worldReady")]
        public bool WorldReady { get; set; }

        /// <summary>Name of the current ECS world, if any.</summary>
        [JsonProperty("worldName")]
        public string WorldName { get; set; } = "";

        /// <summary>Total number of entities in the world.</summary>
        [JsonProperty("entityCount")]
        public int EntityCount { get; set; }

        /// <summary>Whether the DINOForge mod platform is fully initialized.</summary>
        [JsonProperty("modPlatformReady")]
        public bool ModPlatformReady { get; set; }

        /// <summary>List of currently loaded pack IDs.</summary>
        [JsonProperty("loadedPacks")]
        public List<string> LoadedPacks { get; set; } = new List<string>(); // public-mutable-ok: JSON deserializer requires mutable List

        /// <summary>DINOForge runtime version string.</summary>
        [JsonProperty("version")]
        public string Version { get; set; } = "";
    }
}
