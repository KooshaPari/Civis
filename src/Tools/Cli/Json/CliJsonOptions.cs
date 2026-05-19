#nullable enable
using System.Text.Json;

namespace DINOForge.Tools.Cli.Json;

/// <summary>
/// Project-static <see cref="JsonSerializerOptions"/> holders for the DINOForge CLI tool.
/// Centralizes the AssetManifest read/write casing so producers and consumers cannot drift
/// (Pattern #109: inline-JsonSerializerOptions consolidation).
/// </summary>
public static class CliJsonOptions
{
    /// <summary>
    /// Canonical options for AssetManifest read/write paths under <c>assetctl</c>.
    /// Writers and readers MUST use this single instance so the on-disk JSON shape stays
    /// stable across the pipeline (intake → normalize → validate → stylize → register).
    /// </summary>
    /// <remarks>
    /// <see cref="JsonNamingPolicy.CamelCase"/> is harmless for AssetManifest types because every
    /// property is decorated with an explicit <c>JsonPropertyName</c> (snake_case), and attributes
    /// override naming policies. <see cref="JsonSerializerOptions.PropertyNameCaseInsensitive"/>
    /// keeps reads tolerant when manifests are hand-edited.
    /// </remarks>
    public static JsonSerializerOptions AssetManifest { get; } = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true,
        WriteIndented = true,
    };
}
