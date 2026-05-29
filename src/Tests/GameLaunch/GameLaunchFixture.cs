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
/// xUnit collection fixture: launches the DINO game process (or attaches to an already-running
/// instance), waits for the DINOForge bridge to become healthy, and tears down the process after
/// all tests in the <see cref="GameLaunchCollection"/> finish (only when this fixture started it).
///
/// Environment variables:
///   DINO_GAME_PATH — (required to launch) path to <c>Diplomacy is Not an Option.exe</c>, or the
///                    Steam install directory; directories are resolved to the .exe automatically.
///                    Optional when <c>DINO_GAME_ALREADY_RUNNING=1</c> (attach-only).
///   DINO_GAME_ALREADY_RUNNING — set to <c>1</c> or <c>true</c> to skip launching the game and
///                               connect to an existing process with the bridge already up.
///   DINO_GAME_BOOTSTRAP_TIMEOUT_MS — total milliseconds to poll for bridge readiness (default
///                                    90000). Headless cold starts often exceed 30s.
///   DINOFORGE_PIPE_NAME — (optional) named pipe for the in-game bridge (default
///                         <c>dinoforge-game-bridge</c>).
///   DINO_BRIDGE_PORT — (optional, informational) legacy CI marker; bridge uses named pipes, not TCP.
///
/// Gracefully skips (not throws) when the game path is invalid, launch fails, or the bridge is
/// unavailable after bootstrap. Tests use <see cref="SkipIfNotInitialized"/>.
/// Tagged [Trait("Category","GameLaunch")] and run on self-hosted runners via game-launch.yml.
/// </summary>
public sealed class GameLaunchFixture : IAsyncLifetime
{
    private const string GameExecutableName = GameLaunchProcessCleanup.GameProcessBaseName + ".exe";

    // Cold starts often need ~80s for the named pipe; mod platform + pack load needs more headroom after connect.
    private const int DefaultBootstrapTimeoutMs = 180_000;
    private const int PollIntervalMs = 500;
    private const int PerAttemptConnectTimeoutMs = 10_000;
    private const int PerAttemptWorldWaitMs = 15_000;
    private const int PerAttemptModPlatformWaitMs = 30_000;

    private Process? _gameProcess;
    private bool _launchedGame;

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
        bool attachOnly = IsTruthyEnv("DINO_GAME_ALREADY_RUNNING");
        string? gamePath = Environment.GetEnvironmentVariable("DINO_GAME_PATH");
        int bootstrapTimeoutMs = ResolveBootstrapTimeoutMs();

        if (!attachOnly && string.IsNullOrEmpty(gamePath))
        {
            IsInitialized = false;
            return;
        }

        string? gameExePath = null;
        string? gameDir = null;
        if (!string.IsNullOrEmpty(gamePath))
        {
            gameExePath = GameLaunchPaths.ResolveGameExecutablePath(gamePath);
            if (gameExePath is null && !attachOnly)
            {
                IsInitialized = false;
                return;
            }

            if (gameExePath is not null)
            {
                gameDir = Path.GetDirectoryName(gameExePath);
            }
        }

        if (attachOnly)
        {
            if (!IsGameProcessRunning())
            {
                IsInitialized = false;
                return;
            }
        }
        else
        {
            if (gameExePath is null)
            {
                IsInitialized = false;
                return;
            }

            // Game Launch Protocol: avoid attaching to a zombie instance without a healthy bridge.
            GameLaunchProcessCleanup.StopStrayGameProcesses();

            _gameProcess = Process.Start(new ProcessStartInfo
            {
                FileName = gameExePath,
                WorkingDirectory = gameDir ?? Path.GetDirectoryName(gameExePath) ?? "",
                UseShellExecute = false,
                CreateNoWindow = true,
            }) ?? throw new InvalidOperationException($"Failed to start game at: {gameExePath}");

            _launchedGame = true;
        }

        Client = new GameClient(BuildClientOptions());

        DateTime deadline = DateTime.UtcNow.AddMilliseconds(bootstrapTimeoutMs);
        while (DateTime.UtcNow < deadline)
        {
            using CancellationTokenSource attemptCts = new(
                TimeSpan.FromMilliseconds(Math.Min(PerAttemptConnectTimeoutMs, RemainingMs(deadline))));

            try
            {
                if (Client.IsConnected)
                {
                    Client.Disconnect();
                }

                await Client.ConnectAsync(
                    TimeSpan.FromMilliseconds(PerAttemptConnectTimeoutMs),
                    attemptCts.Token).ConfigureAwait(true);

                int worldWaitMs = (int)Math.Min(
                    PerAttemptWorldWaitMs,
                    RemainingMs(deadline));
                if (worldWaitMs <= 0)
                {
                    break;
                }

                using CancellationTokenSource worldCts = new(TimeSpan.FromMilliseconds(worldWaitMs));
                WaitResult worldResult = await Client
                    .WaitForWorldAsync(worldWaitMs, worldCts.Token)
                    .ConfigureAwait(true);
                if (worldResult.Ready && await WaitForModPlatformReadyAsync(deadline).ConfigureAwait(true))
                {
                    IsInitialized = true;
                    return;
                }
            }
            catch
            {
                // Bridge not up yet — keep polling
            }

            int delayMs = (int)Math.Min(PollIntervalMs, RemainingMs(deadline));
            if (delayMs <= 0)
            {
                break;
            }

            try
            {
                await Task.Delay(delayMs).ConfigureAwait(false);
            }
            catch
            {
                break;
            }
        }

        IsInitialized = false;
    }

    private static GameClientOptions BuildClientOptions()
    {
        string pipeName = Environment.GetEnvironmentVariable("DINOFORGE_PIPE_NAME")
            ?? "dinoforge-game-bridge";

        return new GameClientOptions
        {
            UseMessageFraming = false,
            PipeName = pipeName,
            ConnectTimeoutMs = PerAttemptConnectTimeoutMs,
            SendTimeoutMs = 30_000,
            ReadTimeoutMs = 60_000,
        };
    }

    private static int ResolveBootstrapTimeoutMs()
    {
        string? raw = Environment.GetEnvironmentVariable("DINO_GAME_BOOTSTRAP_TIMEOUT_MS");
        if (int.TryParse(raw, out int ms) && ms > 0)
        {
            return ms;
        }

        return DefaultBootstrapTimeoutMs;
    }

    private static bool IsTruthyEnv(string name)
    {
        string? value = Environment.GetEnvironmentVariable(name);
        return value is "1"
            || string.Equals(value, "true", StringComparison.OrdinalIgnoreCase)
            || string.Equals(value, "yes", StringComparison.OrdinalIgnoreCase);
    }

    private static bool IsGameProcessRunning()
    {
        try
        {
            return Process.GetProcessesByName(GameLaunchProcessCleanup.GameProcessBaseName).Length > 0;
        }
        catch
        {
            return false;
        }
    }

    private async Task<bool> WaitForModPlatformReadyAsync(DateTime deadline)
    {
        if (Client is null)
        {
            return false;
        }

        while (DateTime.UtcNow < deadline)
        {
            int remainingMs = (int)RemainingMs(deadline);
            if (remainingMs <= 0)
            {
                break;
            }

            try
            {
                using CancellationTokenSource statusCts = new(
                    TimeSpan.FromMilliseconds(Math.Min(PerAttemptModPlatformWaitMs, remainingMs)));
                GameStatus status = await Client.StatusAsync(statusCts.Token).ConfigureAwait(true);
                if (status.ModPlatformReady && status.LoadedPacks.Count > 0)
                {
                    return true;
                }
            }
            catch
            {
                // Platform still bootstrapping — keep polling until bootstrap deadline.
            }

            int delayMs = (int)Math.Min(PollIntervalMs, RemainingMs(deadline));
            if (delayMs <= 0)
            {
                break;
            }

            try
            {
                await Task.Delay(delayMs).ConfigureAwait(false);
            }
            catch
            {
                break;
            }
        }

        return false;
    }

    private static long RemainingMs(DateTime deadline) =>
        Math.Max(0, (long)(deadline - DateTime.UtcNow).TotalMilliseconds);

    public Task DisposeAsync()
    {
        if (_launchedGame)
        {
            try { _gameProcess?.Kill(entireProcessTree: true); }
            catch { /* best-effort */ }

            GameLaunchProcessCleanup.StopStrayGameProcesses();
        }

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
