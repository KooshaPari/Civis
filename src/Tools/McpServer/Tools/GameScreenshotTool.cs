#nullable enable
using System.ComponentModel;
using System.Diagnostics;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that captures a screenshot of the game window.
/// Uses ScreenRecorderLib (Windows.Graphics.Capture) for robust background DirectX capture.
/// Falls back to ffmpeg gdigrab if ScreenRecorderLib unavailable.
/// </summary>
[McpServerToolType]
public sealed class GameScreenshotTool
{
    private static readonly string BepInExDirectory =
        Path.Combine(Environment.GetEnvironmentVariable("ProgramFiles(x86)") ?? "C:\\Program Files (x86)",
            "Steam\\steamapps\\common\\Diplomacy is Not an Option\\BepInEx");

    /// <summary>
    /// Captures a screenshot of the game window and saves it to disk.
    /// Uses ScreenRecorderLib for robust DirectX capture, with ffmpeg gdigrab fallback.
    /// </summary>
    /// <param name="client">The game client (injected).</param>
    /// <param name="path">Optional file path to save the screenshot. If omitted, uses a default in the BepInEx directory.</param>
    /// <param name="returnBase64">If true, also return the image data as base64-encoded PNG string.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>JSON with the screenshot file path, dimensions, and optionally base64 data.</returns>
    [McpServerTool(Name = "game_screenshot"), Description("Capture a screenshot of the game window. Returns the file path, image dimensions, and optionally base64-encoded image data.")]
    public static async Task<string> ScreenshotAsync(
        GameClient client,
        [Description("Optional file path to save the screenshot")] string? path = null,
        [Description("If true, return base64-encoded PNG image data")] bool returnBase64 = false,
        CancellationToken ct = default)
    {
        try
        {
            // Determine output path
            if (string.IsNullOrEmpty(path))
            {
                path = Path.Combine(BepInExDirectory, $"game_screenshot_{DateTime.Now:yyyyMMdd_HHmmss}.png");
            }

            // Ensure output directory exists
            string? dir = Path.GetDirectoryName(path);
            if (dir != null && !Directory.Exists(dir))
            {
                Directory.CreateDirectory(dir);
            }

            // Use GameCaptureHelper to capture (tries ScreenRecorderLib first, then ffmpeg fallback)
            string? capturedPath = await GameCaptureHelper.CaptureAsync(path, ct).ConfigureAwait(false);
            if (capturedPath != null)
            {
                return await BuildResponseAsync(capturedPath, returnBase64, ct).ConfigureAwait(false);
            }

            // Capture failed
            return GameClientHelper.ToJson(new
            {
                success = false,
                error = "Failed to capture screenshot: ScreenRecorderLib and ffmpeg gdigrab both failed. Ensure game window is accessible and ffmpeg is installed."
            });
        }
        catch (Exception ex)
        {
            return GameClientHelper.ToJson(new { success = false, error = ex.Message });
        }
    }

    /// <summary>
    /// Builds the response JSON after a successful capture.
    /// </summary>
    private static async Task<string> BuildResponseAsync(string path, bool returnBase64, CancellationToken ct)
    {
        try
        {
            // Try to read image dimensions (basic PNG header parsing)
            var (width, height) = GetImageDimensions(path);

            // If base64 is requested, read the file
            string? base64Data = null;
            if (returnBase64 && File.Exists(path))
            {
                try
                {
                    byte[] imageBytes = await File.ReadAllBytesAsync(path, ct).ConfigureAwait(false);
                    base64Data = Convert.ToBase64String(imageBytes);
                }
                catch
                {
                    // If base64 encoding fails, we'll return without it
                }
            }

            return GameClientHelper.ToJson(new
            {
                success = true,
                path = path,
                width = width,
                height = height,
                base64 = base64Data,
                timestamp = DateTime.UtcNow.ToString("o")
            });
        }
        catch (Exception ex)
        {
            return GameClientHelper.ToJson(new { success = false, error = ex.Message });
        }
    }

    /// <summary>
    /// Extracts image dimensions from PNG file header.
    /// PNG header: [137, 80, 78, 71] + 4 bytes length + width (4 bytes big-endian) + height (4 bytes big-endian)
    /// </summary>
    private static (int width, int height) GetImageDimensions(string path)
    {
        try
        {
            using var fs = File.Open(path, FileMode.Open, FileAccess.Read, FileShare.Read);
            byte[] buffer = new byte[24]; // PNG signature (8) + IHDR chunk (16)
            int read = fs.Read(buffer, 0, buffer.Length);

            if (read < 24 || buffer[0] != 137 || buffer[1] != 80 || buffer[2] != 78 || buffer[3] != 71)
            {
                return (0, 0);
            }

            // Width: bytes 16-19 (big-endian)
            int width = (buffer[16] << 24) | (buffer[17] << 16) | (buffer[18] << 8) | buffer[19];
            // Height: bytes 20-23 (big-endian)
            int height = (buffer[20] << 24) | (buffer[21] << 16) | (buffer[22] << 8) | buffer[23];

            return (width, height);
        }
        catch
        {
            return (0, 0);
        }
    }
}
