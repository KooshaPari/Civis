#nullable enable
using System;
using System.Diagnostics;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Xunit;
using Xunit.Abstractions;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// xUnit collection fixture: launches the DINO game process, waits for the DINOForge
/// bridge to become healthy, and tears down the process after all tests in the
/// <see cref="GameLaunchCollection"/> finish.
///
/// Required environment variables:
///   DINO_GAME_PATH   — path to Diplomacy is Not an Option.exe, or the Steam install
///                      directory (e.g. ...\common\Diplomacy is Not an Option); directories
///                      are resolved to Diplomacy is Not an Option.exe inside automatically
///   DINO_BRIDGE_PORT — (optional) bridge listen port, defaults to 7474
///
/// Gracefully skips (not throws) when DINO_GAME_PATH is not set, the path is invalid, or the
/// game bridge is unavailable after bootstrap.
/// Tests are tagged [Trait("Category","GameLaunch")] and excluded from ci.yml.
/// They run via game-launch.yml on a self-hosted runner that has the game installed.
/// </summary>
public sealed class GameLaunchFixture : IAsyncLifetime
{
    private const string GameExecutableName = "Diplomacy is Not an Option.exe";

    private const int BootstrapTimeoutMs = 30_000;
    private const int PollIntervalMs = 500;

    private Process? _gameProcess;

    /// <summary>
    /// Indicates whether the fixture was initialized successfully.
    /// Tests should check this and skip if false.
    /// </summary>
    public bool IsInitialized { get; private set; }

    public GameClient? Client { get; private set; }

    private const string SkipReason =
        "DINO_GAME_PATH not set or game bridge unavailable — GameLaunch test skipped.";

    /// <summary>
    /// Skips the current test when <see cref="IsInitialized"/> is false (records SKIP in xUnit).
    /// </summary>
    public void SkipIfNotInitialized(ITestOutputHelper? output = null)
    {
        if (!IsInitialized)
        {
            output?.WriteLine(SkipReason);
        }

        Skip.IfNot(IsInitialized, SkipReason);
    }

    public async Task InitializeAsync()
    {
        string? gamePath = Environment.GetEnvironmentVariable("DINO_GAME_PATH");

        // Gracefully skip if DINO_GAME_PATH is not set
        if (string.IsNullOrEmpty(gamePath))
        {
            IsInitialized = false;
            return;
        }

        string? gameExePath = ResolveGameExecutablePath(gamePath);
        if (gameExePath is null)
        {
            IsInitialized = false;
            return;
        }

        _gameProcess = Process.Start(new ProcessStartInfo
        {
            FileName = gameExePath,
            UseShellExecute = false,
            CreateNoWindow = true
        }) ?? throw new InvalidOperationException($"Failed to start game at: {gameExePath}");

        Client = new GameClient(new GameClientOptions { UseMessageFraming = false });

        // Poll until the bridge is healthy or timeout
        using CancellationTokenSource cts = new(BootstrapTimeoutMs);
        while (!cts.IsCancellationRequested)
        {
            try
            {
                await Client.ConnectAsync(cts.Token).ConfigureAwait(true);
                WaitResult worldResult = await Client.WaitForWorldAsync(BootstrapTimeoutMs, cts.Token).ConfigureAwait(true);
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

            try
            {
                await Task.Delay(PollIntervalMs, cts.Token).ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
                break;
            }
        }

        IsInitialized = false;
    }

    /// <summary>
    /// Resolves <paramref name="gamePath"/> to the game executable: accepts either the .exe
    /// file or the Steam install directory containing <see cref="GameExecutableName"/>.
    /// </summary>
    private static string? ResolveGameExecutablePath(string gamePath)
    {
        if (File.Exists(gamePath))
        {
            return gamePath;
        }

        if (Directory.Exists(gamePath))
        {
            string exeInDir = Path.Combine(gamePath, GameExecutableName);
            return File.Exists(exeInDir) ? exeInDir : null;
        }

        return null;
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
