#nullable enable
using System.CommandLine;
using System.Security.Cryptography;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Deploys the DINOForge Runtime DLL and packs to the game directory.
/// </summary>
internal static class DeployCommand
{
    /// <summary>
    /// Creates the <c>deploy</c> command.
    /// </summary>
    public static Command Create()
    {
        Option<string?> gamePathOpt = new("--game-path")
        {
            Description = "Game installation path (auto-detected if omitted)"
        };

        Option<bool> buildOpt = new("--build")
        {
            Description = "Build before deploying",
            DefaultValueFactory = _ => true
        };

        Option<string> configOpt = new("--configuration")
        {
            Description = "Build configuration",
            DefaultValueFactory = _ => "Release"
        };

        Command command = new("deploy", "Deploy Runtime DLL and packs to the game directory");
        command.Add(gamePathOpt);
        command.Add(buildOpt);
        command.Add(configOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            string? gamePathVal = parseResult.GetValue(gamePathOpt);
            bool doBuild = parseResult.GetValue(buildOpt);
            string config = parseResult.GetValue(configOpt) ?? "Release";

            string? gamePath = GamePathHelper.Detect(gamePathVal);
            if (gamePath is null)
            {
                Environment.ExitCode = 1;
                return;
            }

            AnsiConsole.MarkupLine($"[bold]Game path:[/] {Markup.Escape(gamePath)}");

            // Step 1: Build if requested
            if (doBuild)
            {
                int buildExit = await BuildCommand.RunBuildAsync(config, ct).ConfigureAwait(false);
                if (buildExit != 0)
                {
                    AnsiConsole.MarkupLine("[red]Build failed — aborting deploy.[/]");
                    Environment.ExitCode = buildExit;
                    return;
                }
            }

            // Step 2: Deploy DLL
            string repoRoot = Directory.GetCurrentDirectory();
            // Walk up to find repo root
            string? dir = repoRoot;
            while (dir is not null && !Directory.Exists(Path.Combine(dir, ".git")))
            {
                dir = Directory.GetParent(dir)?.FullName;
            }

            repoRoot = dir ?? repoRoot;

            string sourceDll = Path.Combine(repoRoot, "src", "Runtime", "bin", config, "netstandard2.0", "DINOForge.Runtime.dll");
            string pluginsDir = GamePathHelper.GetPluginsDir(gamePath);
            string destDll = Path.Combine(pluginsDir, "DINOForge.Runtime.dll");

            if (!File.Exists(sourceDll))
            {
                AnsiConsole.MarkupLine($"[red]Error:[/] Built DLL not found at {Markup.Escape(sourceDll)}");
                Environment.ExitCode = 1;
                return;
            }

            Directory.CreateDirectory(pluginsDir);
            File.Copy(sourceDll, destDll, overwrite: true);

            // Compute SHA256
            string hash = ComputeSha256(destDll);
            long sizeBytes = new FileInfo(destDll).Length;

            AnsiConsole.MarkupLine($"\n[green]DLL deployed[/]");
            AnsiConsole.MarkupLine($"  Destination: {Markup.Escape(destDll)}");
            AnsiConsole.MarkupLine($"  Size:        {sizeBytes / 1024.0:F1} KB");
            AnsiConsole.MarkupLine($"  SHA256:      {hash}");

            // Step 3: Deploy packs
            string packsSource = Path.Combine(repoRoot, "packs");
            string packsTarget = GamePathHelper.GetPacksDir(gamePath);

            if (Directory.Exists(packsSource))
            {
                int packCount = DeployPacks(packsSource, packsTarget);
                AnsiConsole.MarkupLine($"\n[green]Packs deployed:[/] {packCount} pack(s) to {Markup.Escape(packsTarget)}");
            }
            else
            {
                AnsiConsole.MarkupLine("[yellow]Warning:[/] No packs/ directory found — skipping pack deploy.");
            }
        });

        return command;
    }

    /// <summary>
    /// Copies pack directories (mirroring structure) from source to target.
    /// Returns the number of packs deployed.
    /// </summary>
    private static int DeployPacks(string source, string target)
    {
        Directory.CreateDirectory(target);
        int count = 0;

        foreach (string packDir in Directory.GetDirectories(source))
        {
            string packName = Path.GetFileName(packDir);
            string destPack = Path.Combine(target, packName);
            Directory.CreateDirectory(destPack);

            // Copy all files in the pack directory (shallow — pack.yaml + content)
            foreach (string file in Directory.GetFiles(packDir, "*", SearchOption.AllDirectories))
            {
                string relativePath = Path.GetRelativePath(packDir, file);
                string destFile = Path.Combine(destPack, relativePath);
                string? destDir = Path.GetDirectoryName(destFile);
                if (destDir is not null)
                {
                    Directory.CreateDirectory(destDir);
                }

                File.Copy(file, destFile, overwrite: true);
            }

            count++;
        }

        return count;
    }

    private static string ComputeSha256(string filePath)
    {
        using FileStream fs = File.OpenRead(filePath);
        byte[] hashBytes = SHA256.HashData(fs);
        return Convert.ToHexString(hashBytes).ToLowerInvariant();
    }
}
