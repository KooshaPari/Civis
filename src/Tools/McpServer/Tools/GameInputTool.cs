#nullable enable
using System.ComponentModel;
using System.Runtime.InteropServices;
using DINOForge.Bridge.Client;
using ModelContextProtocol.Server;
using BareCua;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that sends keyboard and mouse input to the game window without requiring foreground.
/// Uses Win32 SendInput API to inject input directly into the game.
/// </summary>
[McpServerToolType]
public sealed class GameInputTool
{
    // Win32 constants and structures for SendInput
    private const uint INPUT_KEYBOARD = 1;
    private const uint INPUT_MOUSE = 0;
    private const uint KEYEVENTF_KEYUP = 2;
    private const uint MOUSEEVENTF_LEFTDOWN = 0x0002;
    private const uint MOUSEEVENTF_LEFTUP = 0x0004;
    private const uint MOUSEEVENTF_RIGHTDOWN = 0x0008;
    private const uint MOUSEEVENTF_RIGHTUP = 0x0010;
    private const uint MOUSEEVENTF_ABSOLUTE = 0x8000;
    private const uint MOUSEEVENTF_MOVE = 0x0001;

    [StructLayout(LayoutKind.Sequential)]
    private struct INPUT
    {
        public uint type;
        public INPUTUNION U;
    }

    [StructLayout(LayoutKind.Explicit)]
    private struct INPUTUNION
    {
        [FieldOffset(0)] public MOUSEINPUT mi;
        [FieldOffset(0)] public KEYBDINPUT ki;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct MOUSEINPUT
    {
        public int dx;
        public int dy;
        public uint mouseData;
        public uint dwFlags;
        public uint time;
        public IntPtr dwExtraInfo;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct KEYBDINPUT
    {
        public ushort wVk;
        public ushort wScan;
        public uint dwFlags;
        public uint time;
        public IntPtr dwExtraInfo;
    }

    [DllImport("user32.dll", SetLastError = true)]
    private static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);

    [DllImport("user32.dll")]
    private static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll")]
    private static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    private static extern bool IsWindow(IntPtr hWnd);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);

    [DllImport("user32.dll")]
    private static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    private static extern bool GetClientRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    private static extern int GetSystemMetrics(int nIndex);

    [StructLayout(LayoutKind.Sequential)]
    private struct RECT
    {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

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
                "key" => SendKeyInput(key),
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

    private static string SendKeyInput(string? keyName)
    {
        if (string.IsNullOrEmpty(keyName))
        {
            return GameClientHelper.ToJson(new { success = false, error = "key parameter required for key input" });
        }

        try
        {
            // Try bare-cua first if available
            if (TryBareCuaPressKey(keyName).Result)
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
            ushort vkCode = GetVirtualKeyCode(keyName);
            if (vkCode == 0)
            {
                return GameClientHelper.ToJson(new { success = false, error = $"Unknown key: {keyName}" });
            }

            // Create key down input
            INPUT[] inputs = new INPUT[2];

            // Key down
            inputs[0] = new INPUT
            {
                type = INPUT_KEYBOARD,
                U = new INPUTUNION
                {
                    ki = new KEYBDINPUT
                    {
                        wVk = vkCode,
                        wScan = 0,
                        dwFlags = 0,
                        time = 0,
                        dwExtraInfo = IntPtr.Zero
                    }
                }
            };

            // Key up
            inputs[1] = new INPUT
            {
                type = INPUT_KEYBOARD,
                U = new INPUTUNION
                {
                    ki = new KEYBDINPUT
                    {
                        wVk = vkCode,
                        wScan = 0,
                        dwFlags = KEYEVENTF_KEYUP,
                        time = 0,
                        dwExtraInfo = IntPtr.Zero
                    }
                }
            };

            uint result = FocusGameAndInject(() => SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(INPUT))));

            if (result == inputs.Length)
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
                error = $"Failed to send key input (SendInput returned {result})"
            });
        }
        catch (Exception ex)
        {
            return GameClientHelper.ToJson(new { success = false, error = $"Key input error: {ex.Message}" });
        }
    }

    /// <summary>
    /// Attempts to press a key using bare-cua-native.exe (optional).
    /// Returns true on success, false if bare-cua is not available.
    /// </summary>
    private static async Task<bool> TryBareCuaPressKey(string keyName)
    {
        try
        {
            string? nativePath = FindBareCuaNative();
            if (string.IsNullOrEmpty(nativePath) || !File.Exists(nativePath))
                return false;

            await using var computer = await NativeComputer.StartAsync(nativePath, "warn", CancellationToken.None).ConfigureAwait(false);
            await computer.PressKeyAsync(keyName).ConfigureAwait(false);
            return true;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Finds bare-cua-native.exe by checking environment variable and hardcoded path.
    /// </summary>
    private static string? FindBareCuaNative()
    {
        string[] candidatePaths =
        [
            Environment.GetEnvironmentVariable("BARE_CUA_NATIVE") ?? string.Empty,
            Path.Combine(AppContext.BaseDirectory, "bare-cua-native.exe"),
            "C:\\Users\\koosh\\bare-cua\\target\\release\\bare-cua-native.exe"
        ];

        return candidatePaths.FirstOrDefault(p => !string.IsNullOrEmpty(p) && File.Exists(p));
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
            // Normalize button name
            button = button.ToLowerInvariant();

            // Get screen dimensions to convert to absolute coordinates
            int screenWidth = GetSystemMetrics(0);   // SM_CXSCREEN
            int screenHeight = GetSystemMetrics(1);  // SM_CYSCREEN

            // Calculate absolute coordinates (0-65535 range)
            int absX = (int)((x.Value / (double)screenWidth) * 65535);
            int absY = (int)((y.Value / (double)screenHeight) * 65535);

            INPUT[] inputs = new INPUT[2];

            // Determine button down/up flags
            uint downFlag, upFlag;
            (downFlag, upFlag) = button switch
            {
                "left" => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
                "right" => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
                "middle" => (0x0020u, 0x0040u), // Middle button down/up
                _ => (0u, 0u)
            };

            if (downFlag == 0)
            {
                return GameClientHelper.ToJson(new { success = false, error = $"Unknown button: {button}" });
            }

            // Mouse move to position
            inputs[0] = new INPUT
            {
                type = INPUT_MOUSE,
                U = new INPUTUNION
                {
                    mi = new MOUSEINPUT
                    {
                        dx = absX,
                        dy = absY,
                        mouseData = 0,
                        dwFlags = MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
                        time = 0,
                        dwExtraInfo = IntPtr.Zero
                    }
                }
            };

            // Click (down + up combined into one)
            inputs[1] = new INPUT
            {
                type = INPUT_MOUSE,
                U = new INPUTUNION
                {
                    mi = new MOUSEINPUT
                    {
                        dx = 0,
                        dy = 0,
                        mouseData = 0,
                        dwFlags = downFlag | upFlag,
                        time = 0,
                        dwExtraInfo = IntPtr.Zero
                    }
                }
            };

            uint result = FocusGameAndInject(() => SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(INPUT))));

            if (result == inputs.Length)
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
                error = $"Failed to send mouse click (SendInput returned {result})"
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
            // Get screen dimensions to convert to absolute coordinates
            int screenWidth = GetSystemMetrics(0);   // SM_CXSCREEN
            int screenHeight = GetSystemMetrics(1);  // SM_CYSCREEN

            // Calculate absolute coordinates (0-65535 range)
            int absX = (int)((x.Value / (double)screenWidth) * 65535);
            int absY = (int)((y.Value / (double)screenHeight) * 65535);

            INPUT[] inputs = new INPUT[1];

            inputs[0] = new INPUT
            {
                type = INPUT_MOUSE,
                U = new INPUTUNION
                {
                    mi = new MOUSEINPUT
                    {
                        dx = absX,
                        dy = absY,
                        mouseData = 0,
                        dwFlags = MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
                        time = 0,
                        dwExtraInfo = IntPtr.Zero
                    }
                }
            };

            uint result = FocusGameAndInject(() => SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(INPUT))));

            if (result == inputs.Length)
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
                error = $"Failed to send mouse move (SendInput returned {result})"
            });
        }
        catch (Exception ex)
        {
            return GameClientHelper.ToJson(new { success = false, error = $"Mouse move error: {ex.Message}" });
        }
    }

    /// <summary>
    /// Finds the game window and temporarily brings it to foreground, executes the injection action,
    /// then restores the previous foreground window.
    /// Prevents SendInput from going to the wrong window (e.g., Claude Code terminal).
    /// </summary>
    private static uint FocusGameAndInject(Func<uint> injectAction)
    {
        IntPtr previousFocus = GetForegroundWindow();
        IntPtr gameHwnd = FindGameWindow();

        try
        {
            if (gameHwnd != IntPtr.Zero && gameHwnd != previousFocus)
            {
                SetForegroundWindow(gameHwnd);
                System.Threading.Thread.Sleep(50); // Let focus settle
            }

            return injectAction();
        }
        finally
        {
            // Restore previous focus only if it's still a valid window
            if (previousFocus != IntPtr.Zero && IsWindow(previousFocus) && previousFocus != gameHwnd)
            {
                System.Threading.Thread.Sleep(50); // Let input register
                SetForegroundWindow(previousFocus);
            }
        }
    }

    /// <summary>
    /// Finds the game window by title "Diplomacy is Not an Option".
    /// </summary>
    private static IntPtr FindGameWindow()
    {
        try
        {
            var gameProcess = System.Diagnostics.Process.GetProcesses()
                .FirstOrDefault(p => p.MainWindowTitle.Contains("Diplomacy is Not an Option"));

            return gameProcess?.MainWindowHandle ?? IntPtr.Zero;
        }
        catch
        {
            return IntPtr.Zero;
        }
    }

    private static ushort GetVirtualKeyCode(string keyName)
    {
        return keyName.ToUpperInvariant() switch
        {
            // Function keys
            "F1" => 0x70,
            "F2" => 0x71,
            "F3" => 0x72,
            "F4" => 0x73,
            "F5" => 0x74,
            "F6" => 0x75,
            "F7" => 0x76,
            "F8" => 0x77,
            "F9" => 0x78,
            "F10" => 0x79,
            "F11" => 0x7A,
            "F12" => 0x7B,

            // Special keys
            "ESC" => 0x1B,
            "ESCAPE" => 0x1B,
            "ENTER" => 0x0D,
            "RETURN" => 0x0D,
            "SPACE" => 0x20,
            "TAB" => 0x09,
            "BACKSPACE" => 0x08,
            "DELETE" => 0x46,
            "INSERT" => 0x2D,
            "HOME" => 0x24,
            "END" => 0x23,
            "PAGEUP" => 0x21,
            "PAGEDOWN" => 0x22,

            // Arrow keys
            "UP" => 0x26,
            "DOWN" => 0x28,
            "LEFT" => 0x25,
            "RIGHT" => 0x27,

            // Control keys
            "SHIFT" => 0x10,
            "CTRL" => 0x11,
            "CONTROL" => 0x11,
            "ALT" => 0x12,
            "LWIN" => 0x5B,
            "RWIN" => 0x5C,

            // Letters (A-Z) - all map to 0x41-0x5A
            "A" => 0x41,
            "B" => 0x42,
            "C" => 0x43,
            "D" => 0x44,
            "E" => 0x45,
            "F" => 0x46,
            "G" => 0x47,
            "H" => 0x48,
            "I" => 0x49,
            "J" => 0x4A,
            "K" => 0x4B,
            "L" => 0x4C,
            "M" => 0x4D,
            "N" => 0x4E,
            "O" => 0x4F,
            "P" => 0x50,
            "Q" => 0x51,
            "R" => 0x52,
            "S" => 0x53,
            "T" => 0x54,
            "U" => 0x55,
            "V" => 0x56,
            "W" => 0x57,
            "X" => 0x58,
            "Y" => 0x59,
            "Z" => 0x5A,

            // Numbers (0-9) - same as shifted chars, map to 0x30-0x39
            "0" => 0x30,
            "1" => 0x31,
            "2" => 0x32,
            "3" => 0x33,
            "4" => 0x34,
            "5" => 0x35,
            "6" => 0x36,
            "7" => 0x37,
            "8" => 0x38,
            "9" => 0x39,

            _ => 0 // Unknown key
        };
    }
}
