#nullable enable
using System;
using System.Diagnostics;
using System.IO;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using FluentAssertions;

namespace DINOForge.Tests.Integration;

/// <summary>
/// Base class for integration tests that run against a real game instance.
/// 
/// USAGE:
/// ```csharp
/// public class MyGameTest : GameTestRunner
/// {
///     [Fact]
///     public async Task MyTest()
///     {
///         await LaunchGameAsync();
///         await WaitForWorldAsync();
///         
///         // Test game functionality
///         var status = await StatusAsync();
///         status.EntityCount.Should().BeGreaterThan(1000);
///     }
/// }
/// ```
/// 
/// POLICY: Integration tests MUST use this base class, NOT mocks.
/// Mocks are only allowed for unit tests in src/Tests/*.cs
/// </summary>
public abstract class GameTestRunner : IAsyncDisposable, IDisposable
{
    private GameClient? _client;
    private Process? _gameProcess;
    private readonly string _testInstancePath;
    private readonly bool _useHiddenDesktop;
    private bool _disposed;

    /// <summary>
    /// Path to game executable.
    /// Override in tests that need a specific instance.
    /// </summary>
    protected virtual string GameExePath => GetDefaultGamePath();

    /// <summary>
    /// Timeout for game launch in milliseconds.
    /// </summary>
    protected virtual int LaunchTimeoutMs => 60000;

    /// <summary>
    /// Timeout for world readiness in milliseconds.
    /// </summary>
    protected virtual int WorldTimeoutMs => 60000;

    protected GameTestRunner(string? testInstancePath = null, bool useHiddenDesktop = true)
    {
        _testInstancePath = testInstancePath ?? GetTestInstancePath();
        _useHiddenDesktop = useHiddenDesktop;
    }

    private static string GetDefaultGamePath()
    {
        var envPath = Environment.GetEnvironmentVariable("DINO_GAME_DIR");
        if (!string.IsNullOrEmpty(envPath))
        {
            var exePath = Path.Combine(envPath, "Diplomacy is Not an Option.exe");
            if (File.Exists(exePath)) return exePath;
        }

        // Default Steam path
        var defaultPath = @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe";
        if (File.Exists(defaultPath)) return defaultPath;

        throw new InvalidOperationException($"Game executable not found. Set DINO_GAME_DIR environment variable or install game to default location.");
    }

    private static string GetTestInstancePath()
    {
        var configFile = Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "..", ".dino_test_instance_path");
        if (File.Exists(configFile))
        {
            var path = File.ReadAllText(configFile).Trim();
            if (Directory.Exists(path)) return path;
        }

        // Default test instance path
        var defaultTest = @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST";
        if (Directory.Exists(defaultTest)) return defaultTest;

        return GetDefaultGamePath(); // Fall back to main instance
    }

    /// <summary>
    /// Launch the game and wait for the bridge to be ready.
    /// </summary>
    public async Task LaunchGameAsync(CancellationToken ct = default)
    {
        if (_gameProcess != null)
            throw new InvalidOperationException("Game already launched. Call Dispose() first.");

        // Kill any existing game processes
        await KillExistingGameProcessesAsync();

        // Wait for any lingering processes to exit
        await Task.Delay(2000, ct);

        // Launch the game
        var gameDir = Path.GetDirectoryName(GameExePath) ?? throw new InvalidOperationException("Invalid game path");
        var startInfo = new ProcessStartInfo
        {
            FileName = GameExePath,
            WorkingDirectory = gameDir,
            UseShellExecute = true,
            // No window if hidden
            CreateNoWindow = _useHiddenDesktop,
        };

        if (_useHiddenDesktop)
        {
            // Use CreateDesktop for headless testing
            startInfo.Arguments = "--disable-gpu --nographics";
            startInfo.EnvironmentVariables["DISPLAY"] = ":99"; // Xvfb style
        }

        _gameProcess = Process.Start(startInfo);
        if (_gameProcess == null)
            throw new InvalidOperationException("Failed to start game process");

        // Wait for game to initialize bridge
        await Task.Delay(5000, ct); // Give game time to start BepInEx

        // Connect to the bridge
        _client = new GameClient(new GameClientOptions
        {
            ConnectTimeoutMs = 30000,
            ReadTimeoutMs = 10000,
        });

        // Retry connection for up to 30 seconds
        var connected = false;
        for (int i = 0; i < 30; i++)
        {
            try
            {
                await _client.ConnectAsync(ct);
                connected = true;
                break;
            }
            catch
            {
                await Task.Delay(1000, ct);
            }
        }

        if (!connected)
            throw new InvalidOperationException("Failed to connect to game bridge after 30 seconds");
    }

    /// <summary>
    /// Wait for the ECS world to be ready.
    /// </summary>
    public async Task WaitForWorldAsync(CancellationToken ct = default)
    {
        EnsureConnected();

        var result = await _client!.WaitForWorldAsync(WorldTimeoutMs, ct);
        result.Ready.Should().BeTrue("ECS world must be ready for this test");
    }

    /// <summary>
    /// Get current game status.
    /// </summary>
    public async Task<GameStatus> StatusAsync(CancellationToken ct = default)
    {
        EnsureConnected();
        return await _client!.StatusAsync(ct);
    }

    /// <summary>
    /// Query entities matching a component type.
    /// </summary>
    public async Task<QueryResult> QueryEntitiesAsync(string? componentType = null, string? category = null, CancellationToken ct = default)
    {
        EnsureConnected();
        return await _client!.QueryEntitiesAsync(componentType, category, ct);
    }

    /// <summary>
    /// Get a stat value by SDK path.
    /// </summary>
    public async Task<StatResult> GetStatAsync(string sdkPath, int? entityIndex = null, CancellationToken ct = default)
    {
        EnsureConnected();
        return await _client!.GetStatAsync(sdkPath, entityIndex, ct);
    }

    /// <summary>
    /// Apply a stat override.
    /// </summary>
    public async Task<OverrideResult> ApplyOverrideAsync(string sdkPath, float value, string? mode = null, string? filter = null, CancellationToken ct = default)
    {
        EnsureConnected();
        return await _client!.ApplyOverrideAsync(sdkPath, value, mode, filter, ct);
    }

    /// <summary>
    /// Get the catalog snapshot.
    /// </summary>
    public async Task<CatalogSnapshot> GetCatalogAsync(CancellationToken ct = default)
    {
        EnsureConnected();
        return await _client!.GetCatalogAsync(ct);
    }

    /// <summary>
    /// Reload all packs.
    /// </summary>
    public async Task<ReloadResult> ReloadPacksAsync(string? path = null, CancellationToken ct = default)
    {
        EnsureConnected();
        return await _client!.ReloadPacksAsync(path, ct);
    }

    /// <summary>
    /// Get resources snapshot.
    /// </summary>
    public async Task<ResourceSnapshot> GetResourcesAsync(CancellationToken ct = default)
    {
        EnsureConnected();
        return await _client!.GetResourcesAsync(ct);
    }

    /// <summary>
    /// Get the component map.
    /// </summary>
    public async Task<ComponentMapResult> GetComponentMapAsync(string? sdkPath = null, CancellationToken ct = default)
    {
        EnsureConnected();
        return await _client!.GetComponentMapAsync(sdkPath, ct);
    }

    /// <summary>
    /// Wait for the ECS world to be ready.
    /// </summary>
    public async Task<WaitResult> WaitForWorldAsync(int? timeoutMs = null, CancellationToken ct = default)
    {
        EnsureConnected();
        return await _client!.WaitForWorldAsync(timeoutMs, ct);
    }

    private void EnsureConnected()
    {
        if (_client == null)
            throw new InvalidOperationException("Call LaunchGameAsync() first");
        if (!_client.IsConnected)
            throw new InvalidOperationException("Not connected to game bridge");
    }

    private static async Task KillExistingGameProcessesAsync()
    {
        var gameName = "Diplomacy is Not an Option";
        var processes = Process.GetProcessesByName(gameName);
        foreach (var p in processes)
        {
            try
            {
                p.Kill();
                await p.WaitForExitAsync();
            }
            catch { /* Ignore */ }
        }
    }

    public async ValueTask DisposeAsync()
    {
        if (_disposed) return;
        _disposed = true;

        if (_client != null)
        {
            try
            {
                _client.Disconnect();
                _client.Dispose();
            }
            catch { /* Ignore */ }
            _client = null;
        }

        if (_gameProcess != null)
        {
            try
            {
                if (!_gameProcess.HasExited)
                {
                    _gameProcess.Kill();
                    await _gameProcess.WaitForExitAsync();
                }
                _gameProcess.Dispose();
            }
            catch { /* Ignore */ }
            _gameProcess = null;
        }

        GC.SuppressFinalize(this);
    }

    public void Dispose()
    {
        DisposeAsync().AsTask().Wait();
        GC.SuppressFinalize(this);
    }
}
