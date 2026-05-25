#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using ModelContextProtocol.Server;
using DINOForge.Tools.McpServer.Cua;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that sends keyboard and mouse input to the game window without requiring foreground.
/// Uses Win32 SendInput API to inject input directly into the game.
/// </summary>
[McpServerToolType]
public sealed class GameInputTool
{
    /// <summary>
    /// Sends keyboard or mouse input to the game window without requiring foreground.
    /// </summary>
    /// <param name="type">Input type: "key", "click", or "move".</param>
    /// <param name="key">For keyboard input, the key name (e.g., "F9", "Enter", "A").</param>
    /// <param name="x">For mouse input, the X coordinate (screen-relative).</param>
    /// <param name="y">For mouse input, the Y coordinate (screen-relative).</param>
    /// <param name="button">For click input, the button: "left", "right", or "middle".</param>
    /// <param name="processManager">Process manager to find game window (injected).</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>JSON result with success status.</returns>
    [McpServerTool(Name = "game_input"), Description("Send keyboard or mouse input to the game without foreground. Types: key (key=F9), click (x=100,y=200,button=left), move (x=100,y=200).")]
    public static async Task<string> SendInputAsync(
        [Description("Input type: key|click|move")] string type,
        [Description("Key name for keyboard (F1-F12, ESC, ENTER, arrow keys, letters)")] string? key = null,
        [Description("X coordinate for mouse input")] int? x = null,
        [Description("Y coordinate for mouse input")] int? y = null,
        [Description("Mouse button: left|right|middle")] string? button = null,
        GameProcessManager? processManager = null,
        CancellationToken ct = default)
    {
        try
        {
            if (processManager == null || !processManager.IsRunning)
            {
                return GameClientHelper.ToJson(new
                {
                    success = false,
                    error = "Game is not running."
                });
            }

            // Normalize input type
            type = type?.ToLowerInvariant() ?? "";

            return type switch
            {
                "key" => await SendKeyInput(key).ConfigureAwait(false),
                "click" => SendMouseClick(x, y, button),
                "move" => SendMouseMove(x, y),
                _ => GameClientHelper.ToJson(new { success = false, error = $"Unknown input type: {type}" })
            };
        }
        catch (Exception ex)
        {
            return GameClientHelper.ToJson(new { success = false, error = ex.Message });
        }
    }

    private static async Task<string> SendKeyInput(string? keyName)
    {
        if (string.IsNullOrEmpty(keyName))
        {
            return GameClientHelper.ToJson(new { success = false, error = "key parameter required for key input" });
        }

        try
        {
            // Pattern #116 fix (#800): await instead of .Result to avoid sync-over-async deadlock risk.
            if (await CuaNativeSession.TryPressKeyAsync(
                    CuaNativeClientOptions.Bare, CuaNativePaths.TryFindBare(), keyName, CancellationToken.None)
                .ConfigureAwait(false))
            {
                return GameClientHelper.ToJson(new
                {
                    success = true,
                    key = keyName,
                    message = $"Key '{keyName}' sent successfully via bare-cua"
                });
            }
        }
        catch
        {
            // bare-cua not available; fall through to Win32 SendInput
        }

        try
        {
            if (GameInputHelper.GetVirtualKeyCode(keyName) == 0)
            {
                return GameClientHelper.ToJson(new { success = false, error = $"Unknown key: {keyName}" });
            }

            if (GameInputHelper.SendKey(keyName))
            {
                return GameClientHelper.ToJson(new
                {
                    success = true,
                    key = keyName,
                    message = $"Key '{keyName}' sent successfully via Win32 SendInput"
                });
            }

            return GameClientHelper.ToJson(new
            {
                success = false,
                error = "Failed to send key input"
            });
        }
        catch (Exception ex)
        {
            return GameClientHelper.ToJson(new { success = false, error = $"Key input error: {ex.Message}" });
        }
    }

    private static string SendMouseClick(int? x, int? y, string? button)
    {
        if (!x.HasValue || !y.HasValue)
        {
            return GameClientHelper.ToJson(new { success = false, error = "x and y parameters required for mouse click" });
        }

        if (string.IsNullOrEmpty(button))
        {
            return GameClientHelper.ToJson(new { success = false, error = "button parameter required (left|right|middle)" });
        }

        try
        {
            button = button.ToLowerInvariant();

            if (button is not ("left" or "right" or "middle"))
            {
                return GameClientHelper.ToJson(new { success = false, error = $"Unknown button: {button}" });
            }

            if (GameInputHelper.SendMouseClick(x.Value, y.Value, button))
            {
                return GameClientHelper.ToJson(new
                {
                    success = true,
                    x = x.Value,
                    y = y.Value,
                    button = button,
                    message = $"Mouse click at ({x}, {y}) with button '{button}' sent successfully"
                });
            }

            return GameClientHelper.ToJson(new
            {
                success = false,
                error = "Failed to send mouse click"
            });
        }
        catch (Exception ex)
        {
            return GameClientHelper.ToJson(new { success = false, error = $"Mouse click error: {ex.Message}" });
        }
    }

    private static string SendMouseMove(int? x, int? y)
    {
        if (!x.HasValue || !y.HasValue)
        {
            return GameClientHelper.ToJson(new { success = false, error = "x and y parameters required for mouse move" });
        }

        try
        {
            if (GameInputHelper.SendMouseMove(x.Value, y.Value))
            {
                return GameClientHelper.ToJson(new
                {
                    success = true,
                    x = x.Value,
                    y = y.Value,
                    message = $"Mouse moved to ({x}, {y}) successfully"
                });
            }

            return GameClientHelper.ToJson(new
            {
                success = false,
                error = "Failed to send mouse move"
            });
        }
        catch (Exception ex)
        {
            return GameClientHelper.ToJson(new { success = false, error = $"Mouse move error: {ex.Message}" });
        }
    }
}
