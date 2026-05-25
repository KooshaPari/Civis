#nullable enable
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of a scene load request.
    /// </summary>
    public sealed class LoadSceneResult
    {
        /// <summary>Whether the scene load was dispatched successfully.</summary>
        [JsonProperty("success")]
        public bool Success { get; set; }

        /// <summary>The scene name that was requested.</summary>
        [JsonProperty("scene")]
        public string Scene { get; set; } = "";

        /// <summary>Total number of scenes in build settings (-1 if unknown).</summary>
        [JsonProperty("sceneCount")]
        public int SceneCount { get; set; } = -1;

        /// <summary>The build index used (-1 if loaded by name).</summary>
        [JsonProperty("buildIndex")]
        public int BuildIndex { get; set; } = -1;
    }
}
