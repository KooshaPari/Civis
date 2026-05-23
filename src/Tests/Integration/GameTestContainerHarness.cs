#nullable enable
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;

namespace DINOForge.Tests.Integration;

/// <summary>
/// Manages a pool of isolated game containers (DINOBox instances) for parallel testing.
///
/// Each container is a complete, isolated game instance with:
/// - Symlinked read-only assets (no 12GB duplication)
/// - Unique named pipe for bridge communication
/// - Independent save directory
/// - Independent BepInEx configuration
///
/// USAGE:
/// ```csharp
/// var harness = new GameTestContainerHarness();
/// var pool = await harness.CreatePoolAsync(4);
///
/// // Launch all instances in parallel
/// var launchTasks = pool.Select(container => harness.LaunchInstanceAsync(container));
/// await Task.WhenAll(launchTasks);
///
/// // Wait for all bridges to be ready
/// var bridgeTasks = pool.Select(container => harness.WaitForWorldAsync(container));
/// await Task.WhenAll(bridgeTasks);
/// ```
/// </summary>
public class GameTestContainerHarness : IAsyncDisposable
{
    private readonly string _baseDir;
    private readonly Dictionary<string, GameClient> _clients = new();
    private readonly List<Process> _gameProcesses = new();
    private bool _disposed;

    public GameTestContainerHarness(string? baseDir = null)
    {
        _baseDir = baseDir ?? @"G:\dino_boxes";
    }

    /// <summary>
    /// Container metadata for a game instance.
    /// </summary>
    public class GameContainer
    {
        public required int Index { get; init; }
        public required string BoxPath { get; init; }
        public required string PipeName { get; init; }
        public required string Uuid { get; init; }
        public required string ExePath { get; init; }
        public required string BepInExDir { get; init; }
        public required string SaveDir { get; init; }
        public required string DebugLogPath { get; init; }

        public GameClient? Client { get; set; }
    }

    /// <summary>
    /// Create a pool of N isolated game containers via PowerShell New-DINOBoxPool script.
    /// </summary>
    public async Task<List<GameContainer>> CreatePoolAsync(int count)
    {
        var scriptPath = FindScript("New-DINOBoxPool.ps1");
        if (!File.Exists(scriptPath))
        {
            throw new InvalidOperationException($"PowerShell script not found: {scriptPath}");
        }

        // Run PowerShell script to create pool
        var psScript = $"& '{scriptPath}' -Count {count} -BaseDir '{_baseDir}' | ConvertTo-Json";
        var result = await RunPowerShellAsync(psScript).ConfigureAwait(true);

        // Parse JSON result into containers
        var containers = ParsePoolJson(result);
        return containers;
    }

    /// <summary>
    /// Launch a game instance from a container with real bridge polling (not sleep).
    /// </summary>
    public async Task<bool> LaunchInstanceAsync(GameContainer container, int timeoutSeconds = 30)
    {
        var scriptPath = FindScript("Launch-DINOBoxInstance.ps1");
        if (!File.Exists(scriptPath))
        {
            throw new InvalidOperationException($"PowerShell script not found: {scriptPath}");
        }

        // Run launcher script
        var psScript = $"& '{scriptPath}' -BoxPath '{container.BoxPath}' -PipeName '{container.PipeName}' -Hidden -TimeoutSeconds {timeoutSeconds}";
        await RunPowerShellAsync(psScript).ConfigureAwait(true);

        // Now actively poll for the game process and bridge connection
        var startTime = DateTime.UtcNow;
        var timeout = TimeSpan.FromSeconds(timeoutSeconds);

        while (DateTime.UtcNow - startTime < timeout)
        {
            try
            {
                // Check if process is running
                var processName = "Diplomacy is Not an Option";
                var processes = Process.GetProcessesByName(processName);
                if (processes.Length > 0)
                {
                    _gameProcesses.AddRange(processes);
                }

                // Try to connect to bridge
                var client = new GameClient(new GameClientOptions { PipeName = container.PipeName });
                await client.ConnectAsync().ConfigureAwait(true);

                if (client.IsConnected)
                {
                    container.Client = client;
                    _clients[container.PipeName] = client;
                    return true;
                }
            }
            catch
            {
                // Not ready yet, continue polling
            }

            await Task.Delay(100).ConfigureAwait(true);
        }

        return false;
    }

    /// <summary>
    /// Wait for ECS world to be ready on an instance.
    /// Polls the bridge for entity count > 0 instead of sleeping.
    /// </summary>
    public async Task<bool> WaitForWorldAsync(GameContainer container, int timeoutSeconds = 60)
    {
        var client = container.Client ?? _clients.GetValueOrDefault(container.PipeName);
        if (client == null)
        {
            throw new InvalidOperationException($"Game client not connected for container {container.Index}");
        }

        var startTime = DateTime.UtcNow;
        var timeout = TimeSpan.FromSeconds(timeoutSeconds);

        while (DateTime.UtcNow - startTime < timeout)
        {
            try
            {
                var status = await client.StatusAsync().ConfigureAwait(true);
                if (status?.EntityCount > 0)
                {
                    return true;
                }
            }
            catch
            {
                // Bridge not fully ready yet
            }

            await Task.Delay(100).ConfigureAwait(true);
        }

        return false;
    }

    /// <summary>
    /// Kill all game instances in the pool.
    /// </summary>
    public async Task KillAllAsync()
    {
        // Close all bridge clients
        foreach (var client in _clients.Values)
        {
            try
            {
                client.Dispose();
            }
            catch { }
        }
        _clients.Clear();

        // Kill game processes
        var processName = "Diplomacy is Not an Option";
        var processes = Process.GetProcessesByName(processName);
        foreach (var proc in processes)
        {
            try
            {
                proc.Kill(true);
                await Task.Delay(100).ConfigureAwait(true);
            }
            catch { }
        }

        // Verify all are dead
        var deadline = DateTime.UtcNow.AddSeconds(5);
        while (DateTime.UtcNow < deadline)
        {
            if (Process.GetProcessesByName(processName).Length == 0)
            {
                break;
            }
            await Task.Delay(100).ConfigureAwait(true);
        }
    }

    /// <summary>
    /// Get a game client for a container.
    /// </summary>
    public GameClient? GetClient(GameContainer container)
    {
        return container.Client ?? _clients.GetValueOrDefault(container.PipeName);
    }

    // ===== Private Helpers =====

    private static string FindScript(string scriptName)
    {
        // Search relative to test assembly
        var baseDir = AppContext.BaseDirectory;
        var searchDir = new DirectoryInfo(baseDir);

        while (searchDir != null)
        {
            var scriptPath = Path.Combine(searchDir.FullName, "scripts", "game", scriptName);
            if (File.Exists(scriptPath))
            {
                return scriptPath;
            }

            searchDir = searchDir.Parent;
        }

        throw new InvalidOperationException($"Could not find script {scriptName} in repository");
    }

    private static async Task<string> RunPowerShellAsync(string script)
    {
        var psi = new ProcessStartInfo
        {
            FileName = "powershell.exe",
            Arguments = $"-NoProfile -Command \"{script.Replace("\"", "\\\"")}\"",
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true
        };

        using var process = Process.Start(psi);
        if (process == null)
        {
            throw new InvalidOperationException("Failed to start PowerShell process");
        }

        var output = await process.StandardOutput.ReadToEndAsync().ConfigureAwait(true);
        var error = await process.StandardError.ReadToEndAsync().ConfigureAwait(true);

        await process.WaitForExitAsync().ConfigureAwait(true);

        if (process.ExitCode != 0)
        {
            throw new InvalidOperationException($"PowerShell script failed: {error}");
        }

        return output;
    }

    private static List<GameContainer> ParsePoolJson(string json)
    {
        // Simple JSON parsing (pool is a hashtable converted to JSON)
        var containers = new List<GameContainer>();

        // Expected format: @{ 1 = @{Index, BoxPath, PipeName, ...}, 2 = ... }
        // PowerShell ConvertTo-Json produces nested objects
        // For now, parse via simple string matching or use manual construction

        // Parse each box entry
        var lines = json.Split('\n');
        var currentIndex = 0;
        var currentBox = new Dictionary<string, string>();

        foreach (var line in lines)
        {
            var trimmed = line.Trim();

            // Check for index change (e.g., "1" : {)
            if (trimmed.StartsWith("\"Index\""))
            {
                if (currentBox.Count > 0 && currentBox.ContainsKey("Index"))
                {
                    // Save previous box
                    var container = BuildContainer(currentBox);
                    if (container != null) containers.Add(container);
                    currentBox.Clear();
                }

                // Extract index value
                var match = System.Text.RegularExpressions.Regex.Match(trimmed, @":\s*(\d+)");
                if (match.Success && int.TryParse(match.Groups[1].Value, out var idx))
                {
                    currentIndex = idx;
                    currentBox["Index"] = idx.ToString();
                }
            }

            // Extract key-value pairs
            var kvMatch = System.Text.RegularExpressions.Regex.Match(trimmed, @"""(\w+)""\s*:\s*""([^""]*)""");
            if (kvMatch.Success)
            {
                currentBox[kvMatch.Groups[1].Value] = kvMatch.Groups[2].Value;
            }
        }

        // Save last box
        if (currentBox.Count > 0)
        {
            var container = BuildContainer(currentBox);
            if (container != null) containers.Add(container);
        }

        return containers;
    }

    private static GameContainer? BuildContainer(Dictionary<string, string> data)
    {
        if (!data.ContainsKey("Index") || !data.ContainsKey("BoxPath"))
        {
            return null;
        }

        return new GameContainer
        {
            Index = int.Parse(data["Index"]),
            BoxPath = data.GetValueOrDefault("BoxPath", ""),
            PipeName = data.GetValueOrDefault("PipeName", ""),
            Uuid = data.GetValueOrDefault("Uuid", ""),
            ExePath = data.GetValueOrDefault("ExePath", ""),
            BepInExDir = data.GetValueOrDefault("BepInExDir", ""),
            SaveDir = data.GetValueOrDefault("SaveDir", ""),
            DebugLogPath = data.GetValueOrDefault("DebugLogPath", "")
        };
    }

    // ===== IAsyncDisposable =====

    public async ValueTask DisposeAsync()
    {
        if (_disposed)
            return;

        await KillAllAsync().ConfigureAwait(true);

        foreach (var client in _clients.Values)
        {
            try
            {
                client.Dispose();
            }
            catch { }
        }

        _disposed = true;
    }
}
