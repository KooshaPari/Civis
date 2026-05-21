#nullable enable
using System.Text.Json;

namespace DINOForge.Tools.GameControlCli.Json;

/// <summary>
/// Project-static <see cref="JsonSerializerOptions"/> holders for the GameControl CLI.
/// Centralizes serialization options per Pattern #109 (inline-JsonSerializerOptions consolidation).
/// </summary>
public static class GameControlCliJsonOptions
{
    /// <summary>
    /// Indented JSON for human-readable CLI output.
    /// </summary>
    public static JsonSerializerOptions Indented { get; } = new()
    {
        PropertyNameCaseInsensitive = true,
        WriteIndented = true,
    };
}
