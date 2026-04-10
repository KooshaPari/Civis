#nullable enable
using System.Diagnostics;
using DINOForge.Bridge.Client;
using Xunit;

namespace DINOForge.Tests.Integration.Fixtures;

/// <summary>
/// xUnit fixture that manages the game process and client connection
/// for integration tests. Skips gracefully if the game is not installed.
/// </summary>
public sealed class GameFixture : IAsyncLifetime
{
    private static readonly string[] KnownGamePaths =
    [
        @"C:\Program Files (x86)\Steam\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
        @"C:\Program Files\Steam\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
        @"D:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
        @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe",
    ];

    private Process? _gameProcess;
    private const int ConnectTimeoutMs = 30_000;
    private const int PollIntervalMs = 500;

    /// <summary>Gets the connected game client.</summary>
    public GameClient Client { get; } = new();

    /// <summary>Gets the game process manager.</summary>
    public GameProcessManager ProcessManager { get; } = new();

    /// <summary>Gets whether the game was found and is available for testing.</summary>
    public bool GameAvailable { get; private set; }

    /// <summary>
    /// Launches the game, connects the client, and waits for the ECS world.
    /// If the game is not installed, sets <see cref="GameAvailable"/> to false
    /// so tests can skip gracefully.
    /// </summary>
    public async Task InitializeAsync()
    {
        // Build list of possible game paths including DINO_GAME_PATH env var
        var allGamePaths = KnownGamePaths.ToList();
        var gamePathEnv = Environment.GetEnvironmentVariable("DINO_GAME_PATH");
        if (!string.IsNullOrEmpty(gamePathEnv))
        {
            var exePath = Path.Combine(gamePathEnv, "Diplomacy is Not an Option.exe");
            if (File.Exists(exePath))
                allGamePaths.Add(exePath);
        }

        // Find existing game or check if already running
        string? gameExePath = allGamePaths.FirstOrDefault(File.Exists);
        bool isRunning = ProcessManager.IsRunning;

        if (gameExePath == null && !isRunning)
        {
            GameAvailable = false;
            return;
        }

        try
        {
            // Launch game if not already running
            if (!isRunning && gameExePath != null)
            {
                _gameProcess = Process.Start(new ProcessStartInfo
                {
                    FileName = gameExePath,
                    UseShellExecute = false,
                    CreateNoWindow = true
                });
            }

            // Poll until bridge is connected (up to ConnectTimeoutMs)
            using CancellationTokenSource cts = new(ConnectTimeoutMs);
            while (!cts.Token.IsCancellationRequested && !Client.IsConnected)
            {
                try
                {
                    await Client.ConnectAsync(cts.Token);
                    if (Client.IsConnected) break;
                }
                catch
                {
                    // Bridge not up yet - keep polling
                }
                await Task.Delay(PollIntervalMs, cts.Token).ConfigureAwait(false);
            }

            if (!Client.IsConnected)
            {
                GameAvailable = false;
                return;
            }

            // Wait for the ECS world to be ready (up to 60 seconds)
            using CancellationTokenSource worldCts = new(TimeSpan.FromSeconds(60));
            Bridge.Protocol.WaitResult waitResult = await Client.WaitForWorldAsync(60000, worldCts.Token);

            GameAvailable = waitResult.Ready && Client.IsConnected;
        }
        catch
        {
            GameAvailable = false;
        }
        finally
        {
            // If game is not available, disconnect the client to avoid stale connections
            if (!GameAvailable)
            {
                try { Client.Disconnect(); } catch { /* best effort */ }
            }
        }
    }

    /// <summary>
    /// Disconnects the client and kills the launched game process.
    /// </summary>
    public Task DisposeAsync()
    {
        try { Client.Disconnect(); } catch { /* best effort */ }
        Client.Dispose();
        try { _gameProcess?.Kill(entireProcessTree: true); } catch { /* best effort */ }
        _gameProcess?.Dispose();
        return Task.CompletedTask;
    }
}
