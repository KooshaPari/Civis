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
        // Check if the game executable exists anywhere
        bool gameInstalled = KnownGamePaths.Any(File.Exists) || ProcessManager.IsRunning;
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

            // Wait for the ECS world to be ready (up to 60 seconds)
            using CancellationTokenSource worldCts = new(TimeSpan.FromSeconds(60));
            Bridge.Protocol.WaitResult waitResult = await Client.WaitForWorldAsync(60000, worldCts.Token);

            GameAvailable = waitResult.Ready;
        }
        catch
        {
            GameAvailable = false;
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
