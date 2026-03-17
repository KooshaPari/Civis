#nullable enable
using System;
using System.Diagnostics;
using System.IO;
using System.Threading;
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
///   DINO_GAME_PATH   — path to the DINO game executable
///   DINO_BRIDGE_PORT — (optional) bridge listen port, defaults to 7474
///
/// Tests are tagged [Trait("Category","GameLaunch")] and excluded from ci.yml.
/// They run via game-launch.yml on a self-hosted runner that has the game installed.
/// </summary>
public sealed class GameLaunchFixture : IAsyncLifetime
{
    private const int DefaultBridgePort = 7474;
    private const int BootstrapTimeoutMs = 30_000;
    private const int PollIntervalMs = 500;

    private Process? _gameProcess;

    public GameClient? Client { get; private set; }

    public async Task InitializeAsync()
    {
        string gamePath = Environment.GetEnvironmentVariable("DINO_GAME_PATH")
            ?? throw new InvalidOperationException(
                "DINO_GAME_PATH environment variable is required for game-launch tests.");

        int port = int.TryParse(
            Environment.GetEnvironmentVariable("DINO_BRIDGE_PORT"), out int p) ? p : DefaultBridgePort;

        _gameProcess = Process.Start(new ProcessStartInfo
        {
            FileName = gamePath,
            UseShellExecute = false,
            CreateNoWindow = true
        }) ?? throw new InvalidOperationException($"Failed to start game at: {gamePath}");

        Client = new GameClient($"http://localhost:{port}");

        // Poll until the bridge is healthy or timeout
        using CancellationTokenSource cts = new(BootstrapTimeoutMs);
        while (!cts.IsCancellationRequested)
        {
            try
            {
                DINOForge.Bridge.Protocol.StatusResult status = await Client.GetStatusAsync();
                if (status.Ready)
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
        return Task.CompletedTask;
    }
}

[CollectionDefinition(GameLaunchCollection.Name)]
public sealed class GameLaunchCollection : ICollectionFixture<GameLaunchFixture>
{
    public const string Name = "GameLaunch";
}
