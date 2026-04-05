#nullable enable
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
        // Also check DINO_GAME_PATH environment variable
    ];

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
        // Check if the game executable exists anywhere (including DINO_GAME_PATH env var)
        var gamePath = Environment.GetEnvironmentVariable("DINO_GAME_PATH");
        var allGamePaths = KnownGamePaths.ToList();
        if (!string.IsNullOrEmpty(gamePath))
        {
            var exePath = Path.Combine(gamePath, "Diplomacy is Not an Option.exe");
            if (File.Exists(exePath))
                allGamePaths.Add(exePath);
        }

        bool gameInstalled = allGamePaths.Any(File.Exists) || ProcessManager.IsRunning;
        if (!gameInstalled)
        {
            GameAvailable = false;
            return;
        }

        try
        {
            // Launch or detect the game
            bool launched = await ProcessManager.LaunchAsync();
            if (!launched)
            {
                GameAvailable = false;
                return;
            }

            // Connect the client
            using CancellationTokenSource connectCts = new(TimeSpan.FromSeconds(15));
            await Client.ConnectAsync(connectCts.Token);

            // Check if connected before proceeding
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
    /// Disconnects the client. Does not kill the game process
    /// to avoid disrupting interactive sessions.
    /// </summary>
    public Task DisposeAsync()
    {
        Client.Disconnect();
        Client.Dispose();
        return Task.CompletedTask;
    }
}
