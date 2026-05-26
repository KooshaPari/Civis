#nullable enable

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// Heuristics for inferring game UI state from captured screenshot PNG file size.
/// Shared by navigation and verification tools that poll captures.
/// </summary>
internal static class GameScreenshotHeuristics
{
    /// <summary>
    /// Infers UI state from PNG byte size (calibrated for 1920x1080 captures).
    /// </summary>
    internal static string InferUiStateFromPngSizeBytes(long fileSizeBytes) =>
        fileSizeBytes switch
        {
            < 50_000 => "loading",
            < 150_000 => "main_menu",
            _ => "gameplay"
        };
}
