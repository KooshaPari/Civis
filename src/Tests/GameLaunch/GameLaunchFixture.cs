#nullable enable
using System;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
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
/// Gracefully skips (not throws) when DINO_GAME_PATH is not set AND the game is not running.
/// Tests are tagged [Trait("Category","GameLaunch")] and excluded from ci.yml.
/// They run via game-launch.yml on a self-hosted runner that has the game installed.
/// </summary>
public sealed class GameLaunchFixture : IAsyncLifetime
{
    private const int BootstrapTimeoutMs = 30_000;
    private const int PollIntervalMs = 500;
    
    private Process? _gameProcess;
    
    /// <summary>
    /// Indicates whether the fixture was initialized successfully.
    /// Tests should check this and skip if false.
    /// </summary>
    public bool IsInitialized { get; private set; }
    
    public GameClient? Client { get; private set; }
    
    public async Task InitializeAsync()
    {
        string? gamePath = Environment.GetEnvironmentVariable("DINO_GAME_PATH");
        
        // Gracefully skip if DINO_GAME_PATH is not set
        if (string.IsNullOrEmpty(gamePath))
        {
            IsInitialized = false;
            return;
        }
        
        if (!File.Exists(gamePath))
        {
            IsInitialized = false;
            return;
        }
        
        _gameProcess = Process.Start(new ProcessStartInfo
        {
            FileName = gamePath,
            UseShellExecute = false,
            CreateNoWindow = true
        }) ?? throw new InvalidOperationException($"Failed to start game at: {gamePath}");

        Client = new GameClient(new GameClientOptions());

        // Poll until the bridge is healthy or timeout
        using CancellationTokenSource cts = new(BootstrapTimeoutMs);
        while (!cts.IsCancellationRequested)
        {
            try
            {
                await Client.ConnectAsync(cts.Token);
                WaitResult worldResult = await Client.WaitForWorldAsync(BootstrapTimeoutMs, cts.Token);
                if (worldResult.Ready)
                {
                    IsInitialized = true;
                    return;
                }
            }
            catch
            {
                // Bridge not up yet — keep polling
            }

            await Task.Delay(PollIntervalMs, cts.Token).ConfigureAwait(false);
        }

        IsInitialized = false;
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
