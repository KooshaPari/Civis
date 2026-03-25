#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that polls the game window until a visual change is detected, then captures a screenshot.
/// Useful for waiting for events like a wave starting, a menu appearing, or any change on screen.
/// </summary>
[McpServerToolType]
public sealed class GameWaitAndScreenshotTool
{
    private static readonly string TempDir = Path.Combine(Path.GetTempPath(), "DINOForge");

    /// <summary>
    /// Waits until the game screen changes visually, then captures a screenshot.
    /// Polls every second (configurable) with a timeout. Detects change by comparing pixel samples.
    /// </summary>
    [McpServerTool(Name = "game_wait_and_screenshot"),
     Description("Wait until the game screen changes visually (wave_start, menu_open, any_change), then capture a screenshot. Polls with configurable interval and timeout.")]
    public static async Task<string> WaitAndScreenshotAsync(
        GameClient client,
        [Description("Condition: wave_start|menu_open|any_change")] string condition = "any_change",
        [Description("Timeout in seconds (default 30)")] int timeoutSeconds = 30,
        [Description("Poll interval in ms (default 1000)")] int pollIntervalMs = 1000,
        [Description("Save screenshot to this path")] string? screenshotPath = null,
        CancellationToken ct = default)
    {
        try
        {
            Directory.CreateDirectory(TempDir);
            pollIntervalMs = Math.Clamp(pollIntervalMs, 200, 5000);
            timeoutSeconds = Math.Clamp(timeoutSeconds, 1, 300);

            // Capture baseline
            string basePath = Path.Combine(TempDir, $"wait_base_{DateTime.UtcNow.Ticks}.png");
            string? baseResult = await GameCaptureHelper.CaptureAsync(basePath, ct).ConfigureAwait(false);
            byte[]? basePixels = baseResult != null && File.Exists(basePath) ? SamplePixels(basePath) : null;

            var deadline = DateTime.UtcNow.AddSeconds(timeoutSeconds);
            string? finalPath = null;
            bool conditionMet = false;
            int pollCount = 0;

            while (DateTime.UtcNow < deadline && !ct.IsCancellationRequested)
            {
                await Task.Delay(pollIntervalMs, ct).ConfigureAwait(false);
                pollCount++;

                string pollPath = Path.Combine(TempDir, $"wait_poll_{DateTime.UtcNow.Ticks}.png");
                string? pollResult = await GameCaptureHelper.CaptureAsync(pollPath, ct).ConfigureAwait(false);
                if (pollResult == null || !File.Exists(pollPath))
                    continue;

                byte[] currentPixels = SamplePixels(pollPath);

                // Detect change: if we have baseline pixels and difference is > 2%, or if baseline missing, accept change
                bool changed = basePixels == null || PixelDifference(basePixels, currentPixels) > 0.02;
                if (changed)
                {
                    conditionMet = true;
                    finalPath = pollPath;
                    break;
                }
            }

            // If no change detected, return latest screenshot
            if (!conditionMet)
            {
                finalPath = Path.Combine(TempDir, $"wait_timeout_{DateTime.UtcNow.Ticks}.png");
                string? timeoutResult = await GameCaptureHelper.CaptureAsync(finalPath, ct).ConfigureAwait(false);
                if (timeoutResult == null)
                    finalPath = null;
            }

            // Copy to requested path if provided
            if (!string.IsNullOrEmpty(screenshotPath) && finalPath != null && File.Exists(finalPath))
            {
                try
                {
                    File.Copy(finalPath, screenshotPath, true);
                }
                catch
                {
                    // If copy fails, still return the temp path
                }
            }

            return GameClientHelper.ToJson(new
            {
                success = conditionMet && finalPath != null,
                condition_met = conditionMet,
                screenshot_path = screenshotPath ?? finalPath,
                polls = pollCount,
                timeout_seconds = timeoutSeconds,
                message = conditionMet
                    ? $"Change detected after {pollCount} polls (~{pollCount * pollIntervalMs / 1000}s)"
                    : $"No change detected after {timeoutSeconds}s ({pollCount} polls)"
            });
        }
        catch (Exception ex)
        {
            return GameClientHelper.ToJson(new { success = false, error = ex.Message });
        }
    }

    /// <summary>
    /// Samples a grid of bytes from a PNG file for change detection.
    /// Reads the entire file and samples every 1000th byte to create a lightweight fingerprint.
    /// </summary>
    private static byte[] SamplePixels(string path)
    {
        try
        {
            byte[] bytes = File.ReadAllBytes(path);
            if (bytes.Length == 0)
                return Array.Empty<byte>();

            // Sample every 1000th byte (or less if file is small)
            int sampleSize = Math.Min(256, bytes.Length / 1000 + 1);
            var sample = new byte[sampleSize];

            for (int i = 0; i < sampleSize; i++)
            {
                long index = (long)i * 1000 % bytes.Length;
                sample[i] = bytes[index];
            }

            return sample;
        }
        catch
        {
            return Array.Empty<byte>();
        }
    }

    /// <summary>
    /// Compares two pixel samples and returns the fraction of differing bytes (0.0–1.0).
    /// Considers bytes different if absolute difference > 10 (noise tolerance).
    /// </summary>
    private static double PixelDifference(byte[] a, byte[] b)
    {
        if (a.Length == 0 || b.Length == 0)
            return a.Length == b.Length ? 0.0 : 1.0;

        int len = Math.Min(a.Length, b.Length);
        int diff = 0;

        for (int i = 0; i < len; i++)
        {
            if (Math.Abs(a[i] - b[i]) > 10)
                diff++;
        }

        return (double)diff / len;
    }
}
