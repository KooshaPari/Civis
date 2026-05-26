using System.Text.Json;
using System.Text.Json.Serialization;

namespace DINOForge.SDK.Json
{
    /// <summary>
    /// Canonical System.Text.Json options for DINOForge: case-insensitive property names, allow trailing commas,
    /// allow comments, and string enum converter. Use for all JSON I/O across the codebase to prevent options drift.
    /// </summary>
    public static class JsonOptions
    {
        /// <summary>
        /// Default JSON serialization/deserialization options for DINOForge.
        /// Configured with:
        /// - PropertyNameCaseInsensitive = true (matches JSON/YAML conventions)
        /// - ReadCommentHandling = Skip (allows // and /* */ comments in JSON)
        /// - AllowTrailingCommas = true (relaxed parsing)
        /// - JsonStringEnumConverter for enum values as strings
        /// </summary>
        public static readonly JsonSerializerOptions Default = new JsonSerializerOptions
        {
            PropertyNameCaseInsensitive = true,
            ReadCommentHandling = JsonCommentHandling.Skip,
            AllowTrailingCommas = true,
            Converters = { new JsonStringEnumConverter() },
        };

        /// <summary>
        /// Same as <see cref="Default"/> but with <c>WriteIndented = true</c>. Use for
        /// human-readable manifest and dump output.
        /// </summary>
        public static readonly JsonSerializerOptions Indented = new JsonSerializerOptions
        {
            PropertyNameCaseInsensitive = true,
            ReadCommentHandling = JsonCommentHandling.Skip,
            AllowTrailingCommas = true,
            WriteIndented = true,
            Converters = { new JsonStringEnumConverter() },
        };

        /// <summary>
        /// Same as <see cref="Default"/> but with <c>WriteIndented = false</c> (compact). Use for
        /// IPC payloads and on-the-wire serialization where size matters.
        /// </summary>
        public static readonly JsonSerializerOptions Compact = new JsonSerializerOptions
        {
            PropertyNameCaseInsensitive = true,
            ReadCommentHandling = JsonCommentHandling.Skip,
            AllowTrailingCommas = true,
            WriteIndented = false,
            Converters = { new JsonStringEnumConverter() },
        };
    }
}
