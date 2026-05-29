#nullable enable
using System.CommandLine;
using System.Security.Cryptography;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Builds (optionally) and deploys the DINOForge Runtime DLL plus packs to a DINO install.
/// </summary>
internal static class DeployCommand
{
    private const string RuntimeDllName = "DINOForge.Runtime.dll";

    /// <summary>
    /// Creates the <c>deploy</c> command.
    /// </summary>
    public static Command Create()
    {
        Option<string?> gamePathOpt = new("--game-path")
        {
            Description = "Path to the DINO install root (overrides DINO_GAME_PATH)"
        };
        Option<bool> buildOpt = new("--build")
        {
            Description = "Build the Runtime DLL before deploying",
            DefaultValueFactory = _ => true
        };
        Option<string> configOpt = new("--configuration", "-c")
        {
            Description = "Build configuration (Release or Debug)",
            DefaultValueFactory = _ => "Release"
        };

        Command command = new("deploy", "Deploy the DINOForge Runtime DLL and packs to a DINO install");
        command.Add(gamePathOpt);
        command.Add(buildOpt);
        command.Add(configOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            string? gamePath = parseResult.GetValue(gamePathOpt);
            bool doBuild = parseResult.GetValue(buildOpt);
            string config = parseResult.GetValue(configOpt) ?? "Release";

            int exitCode = await RunDeployAsync(gamePath, doBuild, config, ct).ConfigureAwait(false);
            Environment.ExitCode = exitCode;
        });

        return command;
    }

    /// <summary>
    /// Runs the deploy pipeline: optional build, copy DLL, mirror packs.
    /// </summary>
    internal static async Task<int> RunDeployAsync(string? gamePath, bool doBuild, string config, CancellationToken ct)
    {
        if (doBuild)
        {
            int buildExit = await BuildCommand.RunBuildAsync(config, ct).ConfigureAwait(false);
            if (buildExit != 0)
            {
                AnsiConsole.MarkupLine("[red]Deploy aborted: build failed.[/]");
                return buildExit;
            }
        }

        string resolvedGamePath = GamePathHelper.Detect(gamePath);
        AnsiConsole.MarkupLine($"[bold]Deploying to[/] {Markup.Escape(resolvedGamePath)}");

        string sourceDll = BuildCommand.GetOutputDllPath(config);
        if (!File.Exists(sourceDll))
        {
            AnsiConsole.MarkupLine($"[red]Runtime DLL not found at[/] {Markup.Escape(sourceDll)}");
            return 1;
        }

        string pluginsDir = GamePathHelper.GetPluginsDir(resolvedGamePath);
        try
        {
            Directory.CreateDirectory(pluginsDir);
            string destDll = Path.Combine(pluginsDir, RuntimeDllName);
            File.Copy(sourceDll, destDll, overwrite: true);

            long size = new FileInfo(destDll).Length;
            string hash = await ComputeSha256Async(destDll, ct).ConfigureAwait(false);
            AnsiConsole.MarkupLine($"[green]Copied DLL:[/] {Markup.Escape(destDll)}");
            AnsiConsole.MarkupLine($"  Size:   [cyan]{size:N0} bytes[/]");
            AnsiConsole.MarkupLine($"  SHA256: [cyan]{hash}[/]");
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]Failed to copy DLL:[/] {Markup.Escape(ex.Message)}");
            return 1;
        }

        // Mirror packs/ → BepInEx/dinoforge_packs/
        int packsCopied = 0;
        const string packsRoot = "packs";
        if (Directory.Exists(packsRoot))
        {
            string destPacksDir = GamePathHelper.GetPacksDir(resolvedGamePath);
            try
            {
                Directory.CreateDirectory(destPacksDir);
                foreach (string packDir in Directory.EnumerateDirectories(packsRoot))
                {
                    ct.ThrowIfCancellationRequested();
                    string packName = Path.GetFileName(packDir);
                    if (packName.StartsWith("test-", StringComparison.OrdinalIgnoreCase) ||
                        packName.StartsWith("Test", StringComparison.Ordinal))
                    {
                        continue;
                    }

                    string destPack = Path.Combine(destPacksDir, packName);
                    MirrorDirectory(packDir, destPack);
                    packsCopied++;
                }

                AnsiConsole.MarkupLine($"[green]Mirrored {packsCopied} pack(s) to[/] {Markup.Escape(destPacksDir)}");
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[yellow]Pack mirroring failed:[/] {Markup.Escape(ex.Message)}");
            }
        }
        else
        {
            AnsiConsole.MarkupLine($"[yellow]No packs/ directory found at[/] {Markup.Escape(Path.GetFullPath(packsRoot))}");
        }

        AnsiConsole.MarkupLine("[green]Deploy complete.[/]");
        return 0;
    }

    private static void MirrorDirectory(string source, string dest)
    {
        Directory.CreateDirectory(dest);
        foreach (string dir in Directory.EnumerateDirectories(source, "*", SearchOption.AllDirectories))
        {
            string rel = Path.GetRelativePath(source, dir);
            Directory.CreateDirectory(Path.Combine(dest, rel));
        }

        foreach (string file in Directory.EnumerateFiles(source, "*", SearchOption.AllDirectories))
        {
            string rel = Path.GetRelativePath(source, file);
            string destFile = Path.Combine(dest, rel);
            Directory.CreateDirectory(Path.GetDirectoryName(destFile)!);
            File.Copy(file, destFile, overwrite: true);
        }
    }

    private static async Task<string> ComputeSha256Async(string path, CancellationToken ct)
    {
        await using FileStream fs = File.OpenRead(path);
        byte[] hash = await SHA256.HashDataAsync(fs, ct).ConfigureAwait(false);
        return Convert.ToHexString(hash);
    }
}
