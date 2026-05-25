using System;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;

class Program
{
    [DllImport("user32.dll")] static extern IntPtr FindWindow(string lpClassName, string lpWindowName);
    [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);
    [DllImport("user32.dll")] static extern IntPtr SendMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);
    [DllImport("user32.dll")] static extern bool GetAsyncKeyState(int vKey);

    const int VK_F9 = 0x78;
    const int VK_F10 = 0x79;
    const uint KEYEVENTF_KEYUP = 0x0002;

    static string GameTitle = "Diplomacy is Not an Option";
    static string OutDir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "DINOForge", "screenshots");


    static void Main(string[] args)
    {
        Directory.CreateDirectory(OutDir);

        string action = args.Length > 0 ? args[0].ToLower() : "all";

        IntPtr hwnd = FindWindow(null, GameTitle);
        if (hwnd == IntPtr.Zero)
        {
            Console.WriteLine($"[ERROR] Game window not found: {GameTitle}");
            Environment.Exit(1);
        }

        Console.WriteLine($"[OK] Game window found, HWND={hwnd}");

        if (action == "all" || action == "f9")
        {
            BringToFront(hwnd);
            SendFKey(VK_F9, "F9");
            Thread.Sleep(800);
            TakeScreenshot(Path.Combine(OutDir, $"f9_debug_$(date +%H%M%S).png"));
        }

        if (action == "all" || action == "f10")
        {
            BringToFront(hwnd);
            SendFKey(VK_F10, "F10");
            Thread.Sleep(800);
            TakeScreenshot(Path.Combine(OutDir, $"f10_modmenu_$(date +%H%M%S).png"));
        }

        if (action == "all" || action == "mods")
        {
            // Check if Mods button might be visible on main menu
            BringToFront(hwnd);
            Console.WriteLine("[INFO] Checking for Mods button — taking screenshot...");
            TakeScreenshot(Path.Combine(OutDir, $"mods_check_$(date +%H%M%S).png"));
        }

        Console.WriteLine($"[OK] Done. Screenshots in: {OutDir}");
    }

    static void BringToFront(IntPtr hwnd)
    {
        SetForegroundWindow(hwnd);
        Thread.Sleep(200);
    }

    static void SendFKey(int vk, string name)
    {
        Console.WriteLine($"[SEND] {name}");
        keybd_event((byte)vk, 0, 0, UIntPtr.Zero);
        Thread.Sleep(100);
        keybd_event((byte)vk, 0, KEYEVENTF_KEYUP, UIntPtr.Zero);
        Thread.Sleep(500);
    }

    static void TakeScreenshot(string path)
    {
        try
        {
            Rectangle bounds = Screen.PrimaryScreen?.Bounds ?? new Rectangle(0, 0, 1920, 1080);
            using var bmp = new Bitmap(bounds.Width, bounds.Height);
            using (var g = Graphics.FromImage(bmp))
            {
                g.CopyFromScreen(bounds.Location, System.Drawing.Point.Empty, bounds.Size);
            }
            string ts = DateTime.Now.ToString("HHmmss");
            string finalPath = path.Replace("$(date +%H%M%S)", ts);
            bmp.Save(finalPath);
            Console.WriteLine($"[SNAP] {finalPath} ({bounds.Width}x{bounds.Height})");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"[ERROR] Screenshot failed: {ex.Message}");
        }
    }
}
