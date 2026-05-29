#nullable enable
using System.CommandLine;
using System.Diagnostics;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Full smoke pipeline: build → deploy → relaunch → verify via bridge.
/// </summary>
internal static class SmokeCommand
{
    /// <summary>
    /// Creates the <c>smoke</c> command.
    /// </summary>
    public static Command Create()
    {
        Option<string?> gamePathOpt = new("--game-path")
        {
            Description = "Path to the DINO install root (overrides DINO_GAME_PATH)"
        };
        Option<int> timeoutOpt = new("--timeout")
        {
            Description = "Seconds to wait between launch and bridge verification",
            DefaultValueFactory = _ => 45
        };
        Option<string> configOpt = new("--configuration", "-c")
        {
            Description = "Build configuration (Release or Debug)",
            DefaultValueFactory = _ => "Release"
        };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        Command command = new("smoke", "Run full smoke test: build, deploy, relaunch, verify");
        command.Add(gamePathOpt);
        command.Add(timeoutOpt);
        command.Add(configOpt);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            string? gamePath = parseResult.GetValue(gamePathOpt);
            int timeout = parseResult.GetValue(timeoutOpt);
            string config = parseResult.GetValue(configOpt) ?? "Release";
            bool json = CommandOutput.IsJson(parseResult, formatOpt);

            int exitCode = await RunSmokeAsync(gamePath, timeout, config, json, ct).ConfigureAwait(false);
            Environment.ExitCode = exitCode;
        });

        return command;
    }

    private static async Task<int> RunSmokeAsync(string? gamePath, int timeoutSeconds, string config, bool json, CancellationToken ct)
    {
        SmokeResult result = new();
        Stopwatch sw = Stopwatch.StartNew();

        if (json)
        {
            // No live progress UI in JSON mode; run sequentially and emit at end.
            result.FailedStep = await RunStepsAsync(gamePath, timeoutSeconds, config, result, progress: null, ct).ConfigureAwait(false);
            sw.Stop();
            result.TotalSeconds = sw.Elapsed.TotalSeconds;
            result.Success = result.FailedStep is null;
            CommandOutput.WriteJson(result);
            return result.Success ? 0 : 1;
        }

        string? failedStep = null;
        await AnsiConsole.Progress()
            .Columns(new ProgressColumn[]
            {
                new TaskDescriptionColumn(),
                new ProgressBarColumn(),
                new PercentageColumn(),
                new SpinnerColumn()
            })
            .StartAsync(async progress =>
            {
                failedStep = await RunStepsAsync(gamePath, timeoutSeconds, config, result, progress, ct).ConfigureAwait(false);
            }).ConfigureAwait(false);

        sw.Stop();
        result.FailedStep = failedStep;
        result.TotalSeconds = sw.Elapsed.TotalSeconds;
        result.Success = failedStep is null;

        if (result.Success)
        {
            AnsiConsole.MarkupLine($"[green]Smoke succeeded[/] in {result.TotalSeconds:F2}s");
        }
        else
        {
            AnsiConsole.MarkupLine($"[red]Smoke failed at step:[/] {Markup.Escape(failedStep ?? "unknown")}");
        }

        return result.Success ? 0 : 1;
    }

    private static async Task<string?> RunStepsAsync(string? gamePath, int timeoutSeconds, string config, SmokeResult result, ProgressContext? progress, CancellationToken ct)
    {
        ProgressTask? buildTask = progress?.AddTask("[cyan]Build[/]");
        ProgressTask? deployTask = progress?.AddTask("[cyan]Deploy[/]", autoStart: false);
        ProgressTask? relaunchTask = progress?.AddTask("[cyan]Relaunch[/]", autoStart: false);
        ProgressTask? verifyTask = progress?.AddTask("[cyan]Verify[/]", autoStart: false);

        // 1. Build
        buildTask?.StartTask();
        int buildExit = await BuildCommand.RunBuildAsync(config, ct).ConfigureAwait(false);
        buildTask?.Increment(100);
        buildTask?.StopTask();
        result.BuildExitCode = buildExit;
        if (buildExit != 0) return "build";

        // 2. Deploy (skip rebuild since we just did it)
        deployTask?.StartTask();
        int deployExit = await DeployCommand.RunDeployAsync(gamePath, doBuild: false, config, ct).ConfigureAwait(false);
        deployTask?.Increment(100);
        deployTask?.StopTask();
        result.DeployExitCode = deployExit;
        if (deployExit != 0) return "deploy";

        // 3. Relaunch
        relaunchTask?.StartTask();
        int relaunchExit = await RelaunchCommand.RunRelaunchAsync(gamePath, waitSeconds: Math.Min(timeoutSeconds, 15), ct).ConfigureAwait(false);
        relaunchTask?.Increment(100);
        relaunchTask?.StopTask();
        result.RelaunchExitCode = relaunchExit;
        if (relaunchExit != 0) return "relaunch";

        // 4. Verify via bridge
        verifyTask?.StartTask();
        bool verified = await VerifyViaBridgeAsync(timeoutSeconds, result, ct).ConfigureAwait(false);
        verifyTask?.Increment(100);
        verifyTask?.StopTask();
        if (!verified) return "verify";

        return null;
    }

    private static async Task<bool> VerifyViaBridgeAsync(int timeoutSeconds, SmokeResult result, CancellationToken ct)
    {
        // Poll the bridge for up to timeoutSeconds, retrying connection every 2s.
        DateTime deadline = DateTime.UtcNow.AddSeconds(timeoutSeconds);
        while (DateTime.UtcNow < deadline)
        {
            ct.ThrowIfCancellationRequested();
            using GameClient? client = await CommandHelper.ConnectAsync(ct, writeErrors: false).ConfigureAwait(false);
            if (client is not null)
            {
                try
                {
                    GameStatus status = await client.StatusAsync(ct).ConfigureAwait(false);
                    result.Running = status.Running;
                    result.WorldReady = status.WorldReady;
                    result.EntityCount = status.EntityCount;
                    result.PackCount = status.LoadedPacks?.Count ?? 0;

                    AnsiConsole.MarkupLine($"[green]Bridge verified:[/] Running={status.Running}, WorldReady={status.WorldReady}, Entities={status.EntityCount:N0}, Packs={result.PackCount}");
                    return true;
                }
                catch (Exception ex)
                {
                    AnsiConsole.MarkupLine($"[yellow]Bridge status call failed:[/] {Markup.Escape(ex.Message)}");
                }
            }

            try
            {
                await Task.Delay(TimeSpan.FromSeconds(2), ct).ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
                throw;
            }
        }

        AnsiConsole.MarkupLine($"[red]Bridge verification timed out after {timeoutSeconds}s.[/]");
        return false;
    }

    private sealed class SmokeResult
    {
        public bool Success { get; set; }
        public string? FailedStep { get; set; }
        public double TotalSeconds { get; set; }
        public int BuildExitCode { get; set; }
        public int DeployExitCode { get; set; }
        public int RelaunchExitCode { get; set; }
        public bool Running { get; set; }
        public bool WorldReady { get; set; }
        public int EntityCount { get; set; }
        public int PackCount { get; set; }
    }
}
