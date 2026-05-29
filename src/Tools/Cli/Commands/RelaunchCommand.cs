#nullable enable
using System.CommandLine;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Kills any running DINO instances, launches a fresh one, and verifies it started.
/// </summary>
internal static class RelaunchCommand
{
    private const string ProcessName = "Diplomacy is Not an Option";

    /// <summary>
    /// Creates the <c>relaunch</c> command.
    /// </summary>
    public static Command Create()
    {
        Option<string?> gamePathOpt = new("--game-path")
        {
            Description = "Path to the DINO install root (overrides DINO_GAME_PATH)"
        };
        Option<int> waitOpt = new("--wait")
        {
            Description = "Seconds to wait after launch before verification",
            DefaultValueFactory = _ => 15
        };

        Command command = new("relaunch", "Kill, wait, launch, and verify the DINO game");
        command.Add(gamePathOpt);
        command.Add(waitOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            string? gamePath = parseResult.GetValue(gamePathOpt);
            int wait = parseResult.GetValue(waitOpt);
            int exitCode = await RunRelaunchAsync(gamePath, wait, ct).ConfigureAwait(false);
            Environment.ExitCode = exitCode;
        });

        return command;
    }

    /// <summary>
    /// Kills running instances, launches a new one, waits, and verifies window state.
    /// </summary>
    internal static async Task<int> RunRelaunchAsync(string? gamePath, int waitSeconds, CancellationToken ct)
    {
        string resolvedPath = GamePathHelper.Detect(gamePath);
        string exePath = GamePathHelper.GetExePath(resolvedPath);

        if (!File.Exists(exePath))
        {
            AnsiConsole.MarkupLine($"[red]Game executable not found:[/] {Markup.Escape(exePath)}");
            return 1;
        }

        // 1. Kill existing instances.
        Process[] existing = Process.GetProcessesByName(ProcessName);
        if (existing.Length > 0)
        {
            AnsiConsole.MarkupLine($"[yellow]Killing {existing.Length} existing DINO instance(s)...[/]");
            foreach (Process p in existing)
            {
                try { p.Kill(entireProcessTree: true); } catch { /* best-effort */ }
                finally { p.Dispose(); }
            }
        }

        // 2. Wait 3 seconds, verify none remain.
        await Task.Delay(TimeSpan.FromSeconds(3), ct).ConfigureAwait(false);
        Process[] remaining = Process.GetProcessesByName(ProcessName);
        try
        {
            if (remaining.Length > 0)
            {
                AnsiConsole.MarkupLine($"[red]Failed to kill all DINO instances ({remaining.Length} remain).[/]");
                return 1;
            }
        }
        finally
        {
            foreach (Process p in remaining) p.Dispose();
        }

        // 3. Launch.
        AnsiConsole.MarkupLine($"[bold]Launching[/] {Markup.Escape(exePath)}");
        Process? launched;
        try
        {
            ProcessStartInfo psi = new(exePath)
            {
                WorkingDirectory = resolvedPath,
                UseShellExecute = true
            };
            launched = Process.Start(psi);
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]Failed to launch game:[/] {Markup.Escape(ex.Message)}");
            return 1;
        }

        if (launched is null)
        {
            AnsiConsole.MarkupLine("[red]Process.Start returned null.[/]");
            return 1;
        }

        // 4. Wait the configured time.
        AnsiConsole.MarkupLine($"[cyan]Waiting {waitSeconds}s for startup...[/]");
        try
        {
            await Task.Delay(TimeSpan.FromSeconds(waitSeconds), ct).ConfigureAwait(false);
        }
        finally
        {
            // Refresh handle state, do not dispose yet.
        }

        // 5. Verify the process is still alive.
        Process[] postLaunch = Process.GetProcessesByName(ProcessName);
        try
        {
            if (postLaunch.Length == 0)
            {
                AnsiConsole.MarkupLine("[red]Game process exited during startup.[/]");
                return 1;
            }

            Process game = postLaunch[0];
            AnsiConsole.MarkupLine($"[green]Game running:[/] PID [cyan]{game.Id}[/]");

            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                game.Refresh();
                string title = game.MainWindowTitle ?? string.Empty;
                if (!string.IsNullOrEmpty(title))
                {
                    AnsiConsole.MarkupLine($"  Window: [cyan]{Markup.Escape(title)}[/]");

                    if (title.Contains("Fatal", StringComparison.OrdinalIgnoreCase) ||
                        title.Contains("error", StringComparison.OrdinalIgnoreCase) ||
                        title.Contains("another instance", StringComparison.OrdinalIgnoreCase))
                    {
                        AnsiConsole.MarkupLine($"[red]Launch FAILED:[/] error window detected ({Markup.Escape(title)})");
                        return 1;
                    }
                }
            }

            AnsiConsole.MarkupLine("[green]Relaunch successful.[/]");
            return 0;
        }
        finally
        {
            foreach (Process p in postLaunch) p.Dispose();
            launched?.Dispose();
        }
    }
}
