#nullable enable
using System.CommandLine;
using DINOForge.Tools.Cli.Commands;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Syncs content packs between repository and game directories.
/// </summary>
internal static class SyncCommand
{
    private const string DefaultGamePath = @"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option";
    private const string GamePacksFolder = "BepInEx/dinoforge_packs";

    /// <summary>
    /// Creates the <c>sync</c> command.
    /// </summary>
    public static Command Create()
    {
        var repoPathOption = new Option<string>("--repo");
        repoPathOption.Description = "Path to the packs repository directory";

        var gamePathOption = new Option<string>("--game");
        gamePathOption.Description = "Path to the game installation directory";

        var watchOption = new Option<bool>("--watch");
        watchOption.Description = "Watch for changes and auto-sync";

        var command = new Command("sync", "Sync content packs between repository and game");
        command.Add(repoPathOption);
        command.Add(gamePathOption);
        command.Add(watchOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            string repoPath = parseResult.GetValue(repoPathOption) ?? "packs";
            string gamePath = parseResult.GetValue(gamePathOption) ?? DefaultGamePath;
            bool watch = parseResult.GetValue(watchOption);

            string gamePacksPath = Path.Combine(gamePath, GamePacksFolder);

            if (watch)
            {
                await WatchAndSyncAsync(repoPath, gamePacksPath, ct).ConfigureAwait(false);
            }
            else
            {
                await SyncPacksAsync(repoPath, gamePacksPath).ConfigureAwait(false);
            }
        });

        return command;
    }

    private static async Task SyncPacksAsync(string repoPath, string gamePacksPath)
    {
        AnsiConsole.MarkupLine("[cyan]Syncing packs...[/]");

        if (!Directory.Exists(repoPath))
        {
            AnsiConsole.MarkupLine($"[red]Error:[/] Repository path not found: {repoPath}");
            return;
        }

        if (!Directory.Exists(gamePacksPath))
        {
            AnsiConsole.MarkupLine($"[yellow]Creating packs directory:[/] {gamePacksPath}");
            Directory.CreateDirectory(gamePacksPath);
        }

        var repoPacks = Directory.GetDirectories(repoPath)
            .Select(d => new DirectoryInfo(d))
            .Where(d => File.Exists(Path.Combine(d.FullName, "pack.yaml")))
            .ToList();

        var gamePacks = Directory.GetDirectories(gamePacksPath)
            .Select(d => new DirectoryInfo(d))
            .ToList();

        // Find packs to add (in repo but not in game)
        var toAdd = repoPacks
            .Where(r => !gamePacks.Any(g => g.Name == r.Name))
            .ToList();

        // Find packs to update (in both, repo is newer)
        var toUpdate = repoPacks
            .Where(r => gamePacks.Any(g => g.Name == r.Name && g.LastWriteTime < r.LastWriteTime))
            .ToList();

        // Find packs to remove (in game but not in repo)
        var toRemove = gamePacks
            .Where(g => !repoPacks.Any(r => r.Name == g.Name))
            .ToList();

        int changes = 0;

        // Skip removing packs for safety - just sync from repo to game
        // User can manually delete if needed

        // Add/Update packs
        foreach (var pack in toAdd)
        {
            AnsiConsole.MarkupLine($"[green]ADD:[/] {pack.Name}");
            await CopyDirectoryAsync(pack.FullName, Path.Combine(gamePacksPath, pack.Name)).ConfigureAwait(false);
            changes++;
        }

        foreach (var pack in toUpdate)
        {
            AnsiConsole.MarkupLine($"[yellow]UPDATE:[/] {pack.Name}");
            string dest = Path.Combine(gamePacksPath, pack.Name);
            if (Directory.Exists(dest))
                Directory.Delete(dest, true);
            await CopyDirectoryAsync(pack.FullName, dest).ConfigureAwait(false);
            changes++;
        }

        if (changes == 0)
        {
            AnsiConsole.MarkupLine("[dim]All packs up to date.[/]");
        }
        else
        {
            AnsiConsole.MarkupLine($"[green]Synced {changes} pack(s).[/]");

            // Trigger reload in game if running
            await TriggerInGameReloadAsync().ConfigureAwait(false);
        }
    }

    private static async Task WatchAndSyncAsync(string repoPath, string gamePacksPath, CancellationToken ct)
    {
        AnsiConsole.MarkupLine($"[cyan]Watching for changes...[/]");

        using var watcher = new FileSystemWatcher(repoPath)
        {
            IncludeSubdirectories = true,
            NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.DirectoryName
        };

        watcher.Changed += async (s, e) =>
        {
            if (e.Name?.EndsWith(".yaml") == true || e.Name?.EndsWith(".json") == true)
            {
                AnsiConsole.MarkupLine($"[cyan]Change detected:[/] {e.Name}");
                await SyncPacksAsync(repoPath, gamePacksPath).ConfigureAwait(false);
            }
        };

        watcher.Created += async (s, e) =>
        {
            if (e.Name?.EndsWith(".yaml") == true || e.Name?.EndsWith(".json") == true)
            {
                AnsiConsole.MarkupLine($"[cyan]New file:[/] {e.Name}");
                await SyncPacksAsync(repoPath, gamePacksPath).ConfigureAwait(false);
            }
        };

        watcher.EnableRaisingEvents = true;

        try
        {
            await Task.Delay(Timeout.Infinite, ct).ConfigureAwait(false);
        }
        catch (OperationCanceledException)
        {
            AnsiConsole.MarkupLine("[yellow]Stopped watching.[/]");
        }
    }

    private static async Task TriggerInGameReloadAsync()
    {
        try
        {
            using var client = new GameClient(new GameClientOptions { UseMessageFraming = false });
            await client.ConnectAsync().ConfigureAwait(false);

            AnsiConsole.MarkupLine("[cyan]Triggering in-game pack reload...[/]");
            var result = await client.ReloadPacksAsync().ConfigureAwait(false);

            if (result.Success)
            {
                AnsiConsole.MarkupLine($"[green]Reloaded {result.LoadedPacks.Count} pack(s).[/]");
            }
            else
            {
                AnsiConsole.MarkupLine($"[yellow]Reload completed with errors.[/]");
            }
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[dim]Could not trigger reload: {ex.Message}[/]");
        }
    }

    private static async Task CopyDirectoryAsync(string source, string destination)
    {
        Directory.CreateDirectory(destination);

        foreach (var file in Directory.GetFiles(source))
        {
            var destFile = Path.Combine(destination, Path.GetFileName(file));
            await Task.Run(() => File.Copy(file, destFile, true)).ConfigureAwait(false);
        }

        foreach (var dir in Directory.GetDirectories(source))
        {
            var destDir = Path.Combine(destination, Path.GetFileName(dir));
            await CopyDirectoryAsync(dir, destDir).ConfigureAwait(false);
        }
    }
}
