#nullable enable
using Newtonsoft.Json;

namespace DINOForge.Bridge.Protocol
{
    /// <summary>
    /// Result of a screenshot capture operation.
    /// </summary>
    public sealed class ScreenshotResult
    {
        /// <summary>File path where the screenshot was saved.</summary>
        [JsonProperty("path")]
        public string Path { get; set; } = "";

        /// <summary>Width of the captured image in pixels.</summary>
        [JsonProperty("width")]
        public int Width { get; set; }

        /// <summary>Height of the captured image in pixels.</summary>
        [JsonProperty("height")]
        public int Height { get; set; }

        /// <summary>Whether the screenshot was captured successfully.</summary>
        [JsonProperty("success")]
        public bool Success { get; set; }

        /// <summary>Base64-encoded PNG image data (only populated if requested).</summary>
        [JsonProperty("base64")]
        public string? Base64 { get; set; }

        /// <summary>Timestamp when the screenshot was captured (ISO 8601 format).</summary>
        [JsonProperty("timestamp")]
        public string? Timestamp { get; set; }
    }
}
