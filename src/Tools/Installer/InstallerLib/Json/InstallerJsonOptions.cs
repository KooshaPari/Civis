#nullable enable
using System.Text.Json;

namespace DINOForge.Tools.Installer.Json;

/// <summary>
/// Project-static <see cref="JsonSerializerOptions"/> holders for the DINOForge installer
/// (InstallerLib + CLI install commands). Pattern #109: inline-JsonSerializerOptions consolidation.
/// </summary>
public static class InstallerJsonOptions
{
    /// <summary>
    /// Default options for installer manifest read/write and CLI status payloads. CamelCase keeps
    /// the on-disk shape consistent with the rest of the InstallerLib JSON surface, and
    /// <see cref="JsonSerializerOptions.PropertyNameCaseInsensitive"/> keeps reads tolerant of
    /// older manifests written before the naming policy was unified.
    /// </summary>
    public static JsonSerializerOptions Default { get; } = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        PropertyNameCaseInsensitive = true,
        WriteIndented = true,
    };
}
