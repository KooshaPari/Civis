using Avalonia;
using System;
using System.IO;
using System.Runtime.InteropServices;

namespace DINOForge.Installer;

/// <summary>
/// Entry point for the DINOForge GUI installer application.
/// </summary>
internal static class Program
{
    private static readonly string LogPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "DINOForge", "installer-crash.log");

    /// <summary>
    /// Application entry point. Initializes Avalonia with desktop extensions.
    /// </summary>
    /// <param name="args">Command-line arguments.</param>
    [STAThread]
    public static void Main(string[] args)
    {
        // Catch unhandled exceptions from background threads / tasks
        AppDomain.CurrentDomain.UnhandledException += (_, e) =>
            LogAndShow("Unhandled exception", e.ExceptionObject as Exception);

        System.Threading.Tasks.TaskScheduler.UnobservedTaskException += (_, e) =>
        {
            LogAndShow("Unobserved task exception", e.Exception);
            e.SetObserved();
        };

        try
        {
            BuildAvaloniaApp().StartWithClassicDesktopLifetime(args);
        }
        catch (Exception ex)
        {
            LogAndShow("Startup crash", ex);
        }
    }

    /// <summary>
    /// Builds and configures the Avalonia application instance.
    /// </summary>
    /// <returns>Configured AppBuilder.</returns>
    public static AppBuilder BuildAvaloniaApp()
    {
        return AppBuilder.Configure<App>()
            .UsePlatformDetect()
            .LogToTrace();
    }

    /// <summary>
    /// Writes the exception to a log file and shows a Windows MessageBox so the
    /// crash is never silently swallowed (important for WinExe / elevated processes).
    /// </summary>
    private static void LogAndShow(string context, Exception? ex)
    {
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(LogPath)!);
            string message = $"[{DateTime.Now:u}] {context}:{Environment.NewLine}{ex}{Environment.NewLine}";
            // unbounded-log-ok: terminal crash handler in short-lived installer process — Pattern #232 allowlisted
            File.AppendAllText(LogPath, message);
        }
        catch { /* best-effort */ }

        // Show a native MessageBox — works even before Avalonia is initialised
        string text = $"DINOForge Installer crashed during {context}.\n\n{ex?.Message}\n\nFull log: {LogPath}";
        MessageBox(IntPtr.Zero, text, "DINOForge Installer — Startup Error", 0x10 /* MB_ICONERROR */);
    }

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern int MessageBox(IntPtr hWnd, string text, string caption, uint type);
}
