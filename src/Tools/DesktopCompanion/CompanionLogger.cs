using System;
using System.IO;
using System.Text;

namespace DINOForge.Tools.DesktopCompanion;

internal static class CompanionLogger
{
    private const long MaxLogBytes = 10 * 1024 * 1024; // 10 MB rotation threshold
    private static readonly string LogPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "DINOForge.DesktopCompanion", "app.log");
    private static readonly object _lock = new();

    public static void Append(string message)
    {
        try
        {
            Directory.CreateDirectory(Path.GetDirectoryName(LogPath)!);
            lock (_lock)
            {
                if (File.Exists(LogPath) && new FileInfo(LogPath).Length >= MaxLogBytes)
                {
                    var rotated = LogPath + ".1";
                    if (File.Exists(rotated)) File.Delete(rotated);
                    File.Move(LogPath, rotated);
                }
                File.AppendAllText(LogPath, $"[{DateTimeOffset.UtcNow:O}] {message}{Environment.NewLine}", Encoding.UTF8);
            }
        }
        catch { /* safe-swallow: logger must not crash app startup */ }
    }
}
