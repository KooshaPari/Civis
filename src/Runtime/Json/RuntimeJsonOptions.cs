using System.Text.Json;

namespace DINOForge.Runtime.Json
{
    /// <summary>
    /// Canonical JSON options for Runtime persistence (pack settings, etc.).
    /// </summary>
    internal static class RuntimeJsonOptions
    {
        /// <summary>
        /// Pack settings file: camelCase keys, indented for human editing.
        /// </summary>
        public static readonly JsonSerializerOptions PackSettings = new JsonSerializerOptions
        {
            WriteIndented = true,
            PropertyNameCaseInsensitive = false,
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        };
    }
}
