#nullable enable
using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// xUnit collection fixture: launches the DINO game process, waits for the DINOForge
/// bridge to become healthy, and tears down the process after all tests in the
/// <see cref="GameLaunchCollection"/> finish.
///
/// Required environment variables:
///   DINO_GAME_PATH — path to the DINO game executable (or auto-detected from Steam)
///
/// Tests are tagged [Trait("Category","GameLaunch")] and excluded from ci.yml.
/// They run via game-launch.yml on a self-hosted runner that has the game installed.
/// </summary>
public sealed class GameLaunchFixture : IAsyncLifetime
{
    private const int BootstrapTimeoutMs = 30_000;
    private const int PollIntervalMs = 500;

    private Process? _gameProcess;

    public GameClient? Client { get; private set; }

    public async Task InitializeAsync()
    {
        string? gamePath = Environment.GetEnvironmentVariable("DINO_GAME_PATH");
        var processManager = new GameProcessManager();

        if (gamePath == null)
        {
            if (processManager.IsRunning)
            {
                Client = new GameClient();
                await ConnectAndWaitAsync();
                return;
            }

            throw new InvalidOperationException(
                "DINO_GAME_PATH environment variable is required for game-launch tests, " +
                "or the game must already be running.");
        }

        if (!File.Exists(gamePath))
        {
            throw new InvalidOperationException($"Game executable not found: {gamePath}");
        }

        // Launch the game if not already running
        if (!processManager.IsRunning)
        {
            bool launched = await processManager.LaunchAsync(gamePath);
            if (!launched)
            {
                throw new InvalidOperationException($"Failed to launch game at: {gamePath}");
            }
        }

        _gameProcess = Process.GetProcessesByName("Diplomacy is Not an Option").FirstOrDefault();
        Client = new GameClient();

        await ConnectAndWaitAsync();
    }

    private async Task ConnectAndWaitAsync()
    {
        // Poll until the bridge is healthy or timeout
        using CancellationTokenSource cts = new(BootstrapTimeoutMs);
        while (!cts.IsCancellationRequested)
        {
            try
            {
                var status = await Client!.StatusAsync();
                if (status.WorldReady)
                    return;
            }
            catch
            {
                // Bridge not up yet — keep polling
            }

            await Task.Delay(PollIntervalMs, cts.Token).ConfigureAwait(false);
        }

        throw new TimeoutException(
            $"DINOForge bridge did not become healthy within {BootstrapTimeoutMs / 1000}s.");
    }

    public Task DisposeAsync()
    {
        try { _gameProcess?.Kill(entireProcessTree: true); }
        catch { /* best-effort */ }
        _gameProcess?.Dispose();
        Client?.Dispose();
        return Task.CompletedTask;
    }
}

[CollectionDefinition(GameLaunchCollection.Name)]
public sealed class GameLaunchCollection : ICollectionFixture<GameLaunchFixture>
{
    public const string Name = "GameLaunch";
}
