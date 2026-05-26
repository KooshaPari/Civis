#nullable enable
using System;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Minimal Win32 SendInput helper for bridge-driven key simulation (e.g. ESC for pause menu).
    /// Mirrors the MCP GameInputHelper focus-and-inject pattern without pulling in McpServer.
    /// </summary>
    internal static class Win32KeyInput
    {
        private const uint INPUT_KEYBOARD = 1;
        private const uint KEYEVENTF_KEYUP = 0x0002;

        [StructLayout(LayoutKind.Sequential)]
        private struct INPUT
        {
            public uint type;
            public INPUTUNION U;
        }

        [StructLayout(LayoutKind.Explicit)]
        private struct INPUTUNION
        {
            [FieldOffset(0)] public KEYBDINPUT ki;
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
        private static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

        [DllImport("user32.dll")]
        private static extern bool IsWindow(IntPtr hWnd);

        private const int SW_RESTORE = 9;

        /// <summary>Sends a key down+up to the game window via SendInput.</summary>
        public static bool TrySendKey(string keyName, out string message)
        {
            message = "";
            if (string.IsNullOrWhiteSpace(keyName))
            {
                message = "key name is required";
                return false;
            }

            ushort vk = ResolveVirtualKey(keyName);
            if (vk == 0)
            {
                message = $"unsupported key '{keyName}'";
                return false;
            }

            try
            {
                INPUT[] inputs =
                {
                    KeyEvent(vk, keyUp: false),
                    KeyEvent(vk, keyUp: true),
                };

                uint sent = FocusGameAndInject(() =>
                    SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(INPUT))));
                if (sent != inputs.Length)
                {
                    message = $"SendInput returned {sent} (expected {inputs.Length})";
                    return false;
                }

                message = $"Sent key '{keyName}' (VK=0x{vk:X2})";
                return true;
            }
            catch (Exception ex)
            {
                message = $"{ex.GetType().Name}: {ex.Message}";
                return false;
            }
        }

        private static INPUT KeyEvent(ushort vk, bool keyUp) =>
            new INPUT
            {
                type = INPUT_KEYBOARD,
                U = new INPUTUNION
                {
                    ki = new KEYBDINPUT
                    {
                        wVk = vk,
                        wScan = 0,
                        dwFlags = keyUp ? KEYEVENTF_KEYUP : 0,
                        time = 0,
                        dwExtraInfo = IntPtr.Zero,
                    },
                },
            };

        private static uint FocusGameAndInject(Func<uint> injectAction)
        {
            IntPtr previousFocus = GetForegroundWindow();
            IntPtr gameHwnd = FindGameWindow();

            try
            {
                if (gameHwnd != IntPtr.Zero && gameHwnd != previousFocus)
                {
                    ShowWindow(gameHwnd, SW_RESTORE);
                    SetForegroundWindow(gameHwnd);
                    Thread.Sleep(50);
                }

                return injectAction();
            }
            finally
            {
                if (previousFocus != IntPtr.Zero && IsWindow(previousFocus) && previousFocus != gameHwnd)
                {
                    Thread.Sleep(50);
                    SetForegroundWindow(previousFocus);
                }
            }
        }

        private static IntPtr FindGameWindow()
        {
            try
            {
                Process? gameProcess = Process.GetProcesses()
                    .FirstOrDefault(p =>
                        !string.IsNullOrEmpty(p.MainWindowTitle)
                        && p.MainWindowTitle.Contains("Diplomacy is Not an Option", StringComparison.OrdinalIgnoreCase));

                return gameProcess?.MainWindowHandle ?? IntPtr.Zero;
            }
            // safe-swallow: process enumeration is best-effort when locating the game window
            catch (Exception)
            {
                return IntPtr.Zero;
            }
        }

        private static ushort ResolveVirtualKey(string keyName) =>
            keyName.Trim().ToUpperInvariant() switch
            {
                "ESC" or "ESCAPE" => 0x1B,
                "ENTER" or "RETURN" => 0x0D,
                "SPACE" => 0x20,
                "TAB" => 0x09,
                "F9" => 0x78,
                "F10" => 0x79,
                "P" => 0x50,
                _ when keyName.Length == 1 && char.IsLetter(keyName[0])
                    => (ushort)char.ToUpperInvariant(keyName[0]),
                _ => 0,
            };
    }
}
