#nullable enable
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Snapshot of current in-game resource stockpile values.
    /// </summary>
    public sealed class ResourceSnapshot
    {
        /// <summary>Current food stockpile.</summary>
        [JsonProperty("food")]
        public int Food { get; set; }

        /// <summary>Current wood stockpile.</summary>
        [JsonProperty("wood")]
        public int Wood { get; set; }

        /// <summary>Current stone stockpile.</summary>
        [JsonProperty("stone")]
        public int Stone { get; set; }

        /// <summary>Current iron stockpile.</summary>
        [JsonProperty("iron")]
        public int Iron { get; set; }

        /// <summary>Current money (gold) stockpile.</summary>
        [JsonProperty("money")]
        public int Money { get; set; }

        /// <summary>Current soul crystal stockpile.</summary>
        [JsonProperty("souls")]
        public int Souls { get; set; }

        /// <summary>Current bones stockpile.</summary>
        [JsonProperty("bones")]
        public int Bones { get; set; }

        /// <summary>Current spirit stockpile.</summary>
        [JsonProperty("spirit")]
        public int Spirit { get; set; }
    }
}
