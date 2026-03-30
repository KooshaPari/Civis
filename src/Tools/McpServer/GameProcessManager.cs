#nullable enable
using System.Diagnostics;

namespace DINOForge.Tools.McpServer;

/// <summary>
/// Manages the Diplomacy is Not an Option game process lifecycle.
/// Supports launching from the main install path or the TEST (isolated) instance path.
/// Uses hidden window style for TEST launches to avoid cluttering the taskbar.
/// </summary>
public sealed class GameProcessManager : IDisposable
{
    private Process? _process;
    private readonly string _exeName = "Diplomacy is Not an Option.exe";
    private readonly string? _testInstancePath;

    /// <summary>Returns true if a game process is currently running.</summary>
    public bool IsRunning => _process != null && !_process.HasExited;

    /// <summary>
    /// Creates a new GameProcessManager.
    /// Reads the TEST instance path from .dino_test_instance_path if present.
    /// </summary>
    public GameProcessManager()
    {
        string? testPath = TryReadTestInstancePath();
        if (!string.IsNullOrEmpty(testPath) && Directory.Exists(testPath))
        {
            string testExe = Path.Combine(testPath, _exeName);
            if (File.Exists(testExe))
                _testInstancePath = testPath;
        }
    }

    /// <summary>
    /// Launches the game from the TEST instance directory (hidden window).
    /// The TEST path bypasses Unity's native single-instance mutex.
    /// </summary>
    /// <param name="hidden">If true, launch with hidden window style.</param>
    /// <returns>True if the process was launched successfully.</returns>
    public bool LaunchTestInstance(bool hidden = true)
    {
        if (_testInstancePath == null)
            return false;

        if (IsRunning)
            return true; // already running

        string exePath = Path.Combine(_testInstancePath, _exeName);
        if (!File.Exists(exePath))
            return false;

        var psi = new ProcessStartInfo(exePath)
        {
            WorkingDirectory = _testInstancePath,
            UseShellExecute = false,
            RedirectStandardOutput = false,
            RedirectStandardError = false,
        };

        if (hidden)
            psi.WindowStyle = ProcessWindowStyle.Hidden;

        try
        {
            _process = Process.Start(psi);
            return _process != null;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Launches the game from the default/install path (visible window).
    /// </summary>
    /// <param name="gamePath">Optional explicit path to the game executable.</param>
    /// <returns>True if the process was launched successfully.</returns>
    public Task<bool> LaunchAsync(string? gamePath = null)
    {
        return Task.FromResult(LaunchSync(gamePath));
    }

    private bool LaunchSync(string? gamePath)
    {
        if (IsRunning)
            return true;

        string? exePath = gamePath;
        if (string.IsNullOrEmpty(exePath))
        {
            // Probe common install locations
            exePath = ProbeGamePath();
        }

        if (string.IsNullOrEmpty(exePath) || !File.Exists(exePath))
            return false;

        var psi = new ProcessStartInfo(exePath)
        {
            WorkingDirectory = Path.GetDirectoryName(exePath) ?? "",
            UseShellExecute = false,
            RedirectStandardOutput = false,
            RedirectStandardError = false,
        };

        try
        {
            _process = Process.Start(psi);
            return _process != null;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>Returns the path to the TEST instance directory, or null if not found.</summary>
    public string? TestInstancePath => _testInstancePath;

    /// <summary>Returns true if a TEST instance path is configured.</summary>
    public bool HasTestInstance => _testInstancePath != null;

    private static string? TryReadTestInstancePath()
    {
        // Walk up from the executing assembly looking for .dino_test_instance_path
        string? dir = AppContext.BaseDirectory;
        for (int i = 0; i < 10; i++)
        {
            string markerPath = Path.Combine(dir, ".dino_test_instance_path");
            if (File.Exists(markerPath))
            {
                string? path = File.ReadAllText(markerPath).Trim();
                return Directory.Exists(path) ? path : null;
            }

            dir = Path.GetDirectoryName(dir);
            if (string.IsNullOrEmpty(dir))
                break;
        }

        // Also check current working directory
        string cwdMarker = Path.Combine(Environment.CurrentDirectory, ".dino_test_instance_path");
        if (File.Exists(cwdMarker))
        {
            string? path = File.ReadAllText(cwdMarker).Trim();
            return Directory.Exists(path) ? path : null;
        }

        return null;
    }

    private static string? ProbeGamePath()
    {
        string exeName = "Diplomacy is Not an Option.exe";

        // Check Steam library locations
        string[] steamRoots =
        [
            @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option",
            @"C:\Program Files (x86)\Steam\steamapps\common\Diplomacy is Not an Option",
        ];

        foreach (string root in steamRoots)
        {
            string candidate = Path.Combine(root, exeName);
            if (File.Exists(candidate))
                return candidate;
        }

        return null;
    }

    public void Dispose()
    {
        if (_process != null)
        {
            try { _process.Dispose(); } catch { /* ignore */ }
            _process = null;
        }
    }
}
