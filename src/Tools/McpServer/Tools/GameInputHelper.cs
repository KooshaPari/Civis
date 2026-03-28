#nullable enable
using System.Runtime.InteropServices;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// Shared Win32 input helpers for keyboard and mouse control.
/// Wraps SendInput API for background input injection to the game.
/// </summary>
internal static class GameInputHelper
{
    // Win32 constants for SendInput
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
    private static extern int GetSystemMetrics(int nIndex);

    [DllImport("user32.dll")]
    private static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll")]
    private static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    private static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [DllImport("user32.dll")]
    private static extern bool IsWindow(IntPtr hWnd);

    private const int SW_RESTORE = 9;

    /// <summary>
    /// Sends a single key press (down + up) to the game window.
    /// Manages focus to ensure input goes to the game, not the foreground window.
    /// </summary>
    internal static bool SendKey(string keyName)
    {
        if (string.IsNullOrEmpty(keyName))
            return false;

        try
        {
            ushort vkCode = GetVirtualKeyCode(keyName);
            if (vkCode == 0)
                return false;

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

            return FocusGameAndInject(() => SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(INPUT)))) == inputs.Length;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Sends a mouse click at the specified screen coordinates to the game window.
    /// Manages focus to ensure input goes to the game, not the foreground window.
    /// </summary>
    internal static bool SendMouseClick(int x, int y, string button)
    {
        if (string.IsNullOrEmpty(button))
            return false;

        try
        {
            button = button.ToLowerInvariant();

            // Get screen dimensions to convert to absolute coordinates
            int screenWidth = GetSystemMetrics(0);   // SM_CXSCREEN
            int screenHeight = GetSystemMetrics(1);  // SM_CYSCREEN

            // Calculate absolute coordinates (0-65535 range)
            int absX = (int)((x / (double)screenWidth) * 65535);
            int absY = (int)((y / (double)screenHeight) * 65535);

            INPUT[] inputs = new INPUT[2];

            // Determine button down/up flags
            uint downFlag, upFlag;
            (downFlag, upFlag) = button switch
            {
                "left" => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
                "right" => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
                "middle" => (0x0020u, 0x0040u),
                _ => (0u, 0u)
            };

            if (downFlag == 0)
                return false;

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

            // Click (down + up)
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

            return FocusGameAndInject(() => SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(INPUT)))) == inputs.Length;
        }
        catch
        {
            return false;
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
            "ESC" or "ESCAPE" => 0x1B,
            "ENTER" or "RETURN" => 0x0D,
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
            "CTRL" or "CONTROL" => 0x11,
            "ALT" => 0x12,
            "LWIN" => 0x5B,
            "RWIN" => 0x5C,

            // Letters (A-Z)
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

            // Numbers (0-9)
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

            _ => 0
        };
    }
}
