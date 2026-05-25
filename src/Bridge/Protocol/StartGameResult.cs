#nullable enable
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>Result of a start-game (BeginGameWorldLoadingSingleton) request.</summary>
    public sealed class StartGameResult
    {
        /// <summary>Whether the game start request succeeded.</summary>
        [JsonProperty("success")] public bool Success { get; set; }

        /// <summary>Status or error message associated with the request result.</summary>
        [JsonProperty("message")] public string Message { get; set; } = "";
    }
}
