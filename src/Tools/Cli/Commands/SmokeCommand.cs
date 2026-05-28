#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// End-to-end smoke test: build, deploy, relaunch, and verify game status via bridge RPC.
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
            Description = "Game installation path (auto-detected if omitted)"
        };

        Option<int> timeoutOpt = new("--timeout")
        {
            Description = "Total timeout in seconds for the smoke pipeline",
            DefaultValueFactory = _ => 45
        };

        Option<string> configOpt = new("--configuration")
        {
            Description = "Build configuration",
            DefaultValueFactory = _ => "Release"
        };

        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        Command command = new("smoke", "End-to-end smoke test: build, deploy, relaunch, verify");
        command.Add(gamePathOpt);
        command.Add(timeoutOpt);
        command.Add(configOpt);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            string? gamePathVal = parseResult.GetValue(gamePathOpt);
            int timeoutSeconds = parseResult.GetValue(timeoutOpt);
            string config = parseResult.GetValue(configOpt) ?? "Release";
            bool json = CommandOutput.IsJson(parseResult, formatOpt);

            string? gamePath = GamePathHelper.Detect(gamePathVal);
            if (gamePath is null)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("no_game_path", "Could not detect game installation path.");
                }

                Environment.ExitCode = 1;
                return;
            }

            string exePath = GamePathHelper.GetExePath(gamePath);
            if (!File.Exists(exePath))
            {
                string msg = $"Game executable not found at {exePath}";
                if (json)
                {
                    CommandOutput.WriteJsonError("exe_not_found", msg);
                }
                else
                {
                    AnsiConsole.MarkupLine($"[red]Error:[/] {Markup.Escape(msg)}");
                }

                Environment.ExitCode = 1;
                return;
            }

            using CancellationTokenSource cts = CancellationTokenSource.CreateLinkedTokenSource(ct);
            cts.CancelAfter(TimeSpan.FromSeconds(timeoutSeconds));
            CancellationToken token = cts.Token;

            bool buildOk = false;
            bool deployOk = false;
            bool launchOk = false;
            GameStatus? status = null;
            string? failStep = null;

            if (!json)
            {
                await AnsiConsole.Progress()
                    .AutoClear(false)
                    .Columns(
                        new TaskDescriptionColumn(),
                        new ProgressBarColumn(),
                        new SpinnerColumn())
                    .StartAsync(async ctx =>
                    {
                        // Step 1: Build
                        ProgressTask buildTask = ctx.AddTask("[bold]Build[/]", maxValue: 1);
                        int buildExit = await BuildCommand.RunBuildAsync(config, token).ConfigureAwait(false);
                        buildOk = buildExit == 0;
                        buildTask.Value = 1;

                        if (!buildOk)
                        {
                            failStep = "build";
                            buildTask.Description = "[red]Build FAILED[/]";
                            return;
                        }

                        buildTask.Description = "[green]Build OK[/]";

                        // Step 2: Deploy (without rebuild)
                        ProgressTask deployTask = ctx.AddTask("[bold]Deploy[/]", maxValue: 1);
                        try
                        {
                            DeployFiles(gamePath, config);
                            deployOk = true;
                            deployTask.Description = "[green]Deploy OK[/]";
                        }
                        catch (Exception ex)
                        {
                            failStep = "deploy";
                            deployTask.Description = $"[red]Deploy FAILED: {Markup.Escape(ex.Message)}[/]";
                        }

                        deployTask.Value = 1;
                        if (!deployOk) return;

                        // Step 3: Relaunch
                        ProgressTask launchTask = ctx.AddTask("[bold]Relaunch[/]", maxValue: 1);
                        int launchExit = await RelaunchCommand.RunRelaunchAsync(gamePath, exePath, 15, token)
                            .ConfigureAwait(false);
                        launchOk = launchExit == 0;
                        launchTask.Value = 1;

                        if (!launchOk)
                        {
                            failStep = "relaunch";
                            launchTask.Description = "[red]Relaunch FAILED[/]";
                            return;
                        }

                        launchTask.Description = "[green]Relaunch OK[/]";

                        // Step 4: Verify via GameClient
                        ProgressTask verifyTask = ctx.AddTask("[bold]Verify[/]", maxValue: 1);
                        status = await TryGetStatusAsync(token).ConfigureAwait(false);
                        verifyTask.Value = 1;

                        if (status is null)
                        {
                            failStep = "verify";
                            verifyTask.Description = "[red]Verify FAILED — could not connect to bridge[/]";
                        }
                        else
                        {
                            verifyTask.Description = "[green]Verify OK[/]";
                        }
                    }).ConfigureAwait(false);

                // Summary
                AnsiConsole.WriteLine();
                if (failStep is not null)
                {
                    AnsiConsole.MarkupLine($"[red bold]Smoke test FAILED[/] at step: [red]{failStep}[/]");
                    Environment.ExitCode = 1;
                }
                else if (status is not null)
                {
                    Table table = new Table()
                        .Border(TableBorder.Rounded)
                        .Title("[bold green]Smoke Test Passed[/]")
                        .AddColumn("Property")
                        .AddColumn("Value");

                    table.AddRow("Build", "[green]OK[/]");
                    table.AddRow("Deploy", "[green]OK[/]");
                    table.AddRow("Launch", "[green]OK[/]");
                    table.AddRow("Running", status.Running ? "[green]Yes[/]" : "[red]No[/]");
                    table.AddRow("World Ready", status.WorldReady ? "[green]Yes[/]" : "[yellow]No[/]");
                    table.AddRow("Entity Count", status.EntityCount.ToString("N0"));
                    table.AddRow("Loaded Packs", status.LoadedPacks.Count > 0
                        ? Markup.Escape(string.Join(", ", status.LoadedPacks))
                        : "[dim]None[/]");

                    AnsiConsole.Write(table);
                }
            }
            else
            {
                // JSON mode: run pipeline without Spectre progress
                int buildExit = await BuildCommand.RunBuildAsync(config, token).ConfigureAwait(false);
                buildOk = buildExit == 0;

                if (buildOk)
                {
                    try
                    {
                        DeployFiles(gamePath, config);
                        deployOk = true;
                    }
                    catch
                    {
                        failStep = "deploy";
                    }
                }
                else
                {
                    failStep = "build";
                }

                if (deployOk)
                {
                    int launchExit = await RelaunchCommand.RunRelaunchAsync(gamePath, exePath, 15, token)
                        .ConfigureAwait(false);
                    launchOk = launchExit == 0;
                    if (!launchOk) failStep = "relaunch";
                }

                if (launchOk)
                {
                    status = await TryGetStatusAsync(token).ConfigureAwait(false);
                    if (status is null) failStep = "verify";
                }

                CommandOutput.WriteJson(new
                {
                    success = failStep is null,
                    failStep,
                    build = buildOk,
                    deploy = deployOk,
                    launch = launchOk,
                    status
                });

                if (failStep is not null)
                {
                    Environment.ExitCode = 1;
                }
            }
        });

        return command;
    }

    /// <summary>
    /// Deploys DLL and packs without building. Throws on failure.
    /// </summary>
    private static void DeployFiles(string gamePath, string config)
    {
        string repoRoot = FindRepoRoot();
        string sourceDll = Path.Combine(repoRoot, "src", "Runtime", "bin", config, "netstandard2.0", "DINOForge.Runtime.dll");

        if (!File.Exists(sourceDll))
        {
            throw new FileNotFoundException($"Built DLL not found at {sourceDll}");
        }

        string pluginsDir = GamePathHelper.GetPluginsDir(gamePath);
        Directory.CreateDirectory(pluginsDir);
        File.Copy(sourceDll, Path.Combine(pluginsDir, "DINOForge.Runtime.dll"), overwrite: true);

        // Deploy packs
        string packsSource = Path.Combine(repoRoot, "packs");
        if (Directory.Exists(packsSource))
        {
            string packsTarget = GamePathHelper.GetPacksDir(gamePath);
            Directory.CreateDirectory(packsTarget);

            foreach (string packDir in Directory.GetDirectories(packsSource))
            {
                string packName = Path.GetFileName(packDir);
                string destPack = Path.Combine(packsTarget, packName);
                Directory.CreateDirectory(destPack);

                foreach (string file in Directory.GetFiles(packDir, "*", SearchOption.AllDirectories))
                {
                    string relativePath = Path.GetRelativePath(packDir, file);
                    string destFile = Path.Combine(destPack, relativePath);
                    string? destDir = Path.GetDirectoryName(destFile);
                    if (destDir is not null) Directory.CreateDirectory(destDir);
                    File.Copy(file, destFile, overwrite: true);
                }
            }
        }
    }

    private static async Task<GameStatus?> TryGetStatusAsync(CancellationToken ct)
    {
        try
        {
            using GameClient? client = await CommandHelper.ConnectAsync(ct, writeErrors: false).ConfigureAwait(false);
            if (client is null) return null;
            return await client.StatusAsync(ct).ConfigureAwait(false);
        }
        catch
        {
            return null;
        }
    }

    private static string FindRepoRoot()
    {
        string? dir = Directory.GetCurrentDirectory();
        while (dir is not null)
        {
            if (Directory.Exists(Path.Combine(dir, ".git"))) return dir;
            dir = Directory.GetParent(dir)?.FullName;
        }

        return Directory.GetCurrentDirectory();
    }
}
