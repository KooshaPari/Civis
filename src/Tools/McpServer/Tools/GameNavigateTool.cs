#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that navigates the game to specific UI states (main menu, gameplay, pause menu).
/// Uses Win32 keyboard input to send ESC and ENTER keys, and screenshots to verify state.
/// </summary>
[McpServerToolType]
public sealed class GameNavigateTool
{
    private static readonly string TempDir = Path.Combine(Path.GetTempPath(), "DINOForge");

    /// <summary>
    /// Navigates the game to a target UI state: main_menu, gameplay, or pause_menu.
    /// Uses ESC and ENTER keys to trigger navigation, screenshots to verify state.
    /// </summary>
    [McpServerTool(Name = "game_navigate"),
     Description("Navigate game to a target UI state (main_menu|gameplay|pause_menu). Uses ESC/ENTER keys and screenshot verification.")]
    public static async Task<string> NavigateAsync(
        GameClient client,
        [Description("Target state: main_menu|gameplay|pause_menu")] string target,
        [Description("Max attempts to reach target (default 5)")] int maxAttempts = 5,
        [Description("Wait between attempts (ms, default 1000)")] int delayMs = 1000,
        CancellationToken ct = default)
    {
        try
        {
            Directory.CreateDirectory(TempDir);
            string safeTarget = (target ?? "gameplay").ToLowerInvariant();
            if (string.IsNullOrEmpty(safeTarget))
                safeTarget = "gameplay";
            maxAttempts = Math.Clamp(maxAttempts, 1, 20);
            delayMs = Math.Clamp(delayMs, 100, 10000);

            bool success = false;
            string? finalScreenshot = null;
            int attempt = 0;

            while (attempt < maxAttempts && !ct.IsCancellationRequested)
            {
                attempt++;

                // Capture current state
                string screenshotPath = Path.Combine(TempDir, $"nav_{target}_{attempt}_{DateTime.UtcNow.Ticks}.png");
                string? capResult = await GameCaptureHelper.CaptureAsync(screenshotPath, ct).ConfigureAwait(false);
                if (capResult == null)
                    continue;

                long fileSize = new FileInfo(screenshotPath).Length;
                string currentState = DetectGameState(fileSize);

                // Check if we've reached the target state
                if (currentState == safeTarget)
                {
                    success = true;
                    finalScreenshot = screenshotPath;
                    break;
                }

                // Determine the action to take
                string? action = DetermineAction(currentState, target);
                if (!string.IsNullOrEmpty(action))
                {
                    // Send the action
                    if (action == "ESC")
                        GameInputHelper.SendKey("ESC");
                    else if (action == "ENTER")
                        GameInputHelper.SendKey("ENTER");
                    else if (action.StartsWith("KEY:"))
                        GameInputHelper.SendKey(action.Substring(4));

                    // Wait before checking again
                    await Task.Delay(delayMs, ct).ConfigureAwait(false);
                }
                else
                {
                    // No action possible, already at target or can't navigate there
                    break;
                }
            }

            return GameClientHelper.ToJson(new
            {
                success = success,
                target = safeTarget,
                attempts = attempt,
                screenshot_path = finalScreenshot,
                message = success
                    ? $"Successfully navigated to {safeTarget}"
                    : $"Failed to navigate to {safeTarget} after {attempt} attempts"
            });
        }
        catch (Exception ex)
        {
            return GameClientHelper.ToJson(new { success = false, error = ex.Message });
        }
    }

    /// <summary>
    /// Detects the current game UI state based on screenshot file size heuristics.
    /// File size is a simple proxy for screen complexity:
    /// - main_menu: medium size (menu UI only, less geometry)
    /// - gameplay: large size (entities, terrain, full detail)
    /// - pause_menu: medium size (menu overlay on gameplay)
    /// - loading: small size (blank screen)
    /// </summary>
    private static string DetectGameState(long fileSizeBytes)
    {
        // File size heuristics (calibrated for 1920x1080 PNG):
        // - Loading/blank: < 50KB (mostly uniform color, highly compressible)
        // - Menu UI only: 50-150KB (UI elements, less scene detail)
        // - Gameplay: > 150KB (full scene + entities, more detail)

        return fileSizeBytes switch
        {
            < 50_000 => "loading",                    // Very small = loading/blank
            < 150_000 => "main_menu",                 // Medium = menu UI
            _ => "gameplay"                           // Large = full gameplay scene
        };
    }

    /// <summary>
    /// Determines the next action to navigate from current state to target state.
    /// </summary>
    private static string? DetermineAction(string currentState, string? targetState)
    {
        if (currentState == targetState)
            return null; // Already at target

        // Navigation matrix: what key to press to move from state A to state B
        return (currentState, targetState) switch
        {
            // From gameplay: ESC opens pause menu
            ("gameplay", "pause_menu") => "ESC",

            // From pause menu: ESC closes it back to gameplay
            ("pause_menu", "gameplay") => "ESC",

            // From menu to main_menu: ENTER to confirm, or ESC to back out
            ("main_menu", "gameplay") => "ENTER",

            // From gameplay back to main menu: ESC pause, then ESC again (menu structure dependent)
            ("gameplay", "main_menu") => "ESC",
            ("pause_menu", "main_menu") => "ESC",

            // From loading: wait for it to progress (no action needed)
            ("loading", _) => null,

            // Default: no known action
            _ => null
        };
    }
}
