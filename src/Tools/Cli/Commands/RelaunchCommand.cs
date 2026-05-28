#nullable enable
using System.CommandLine;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Kills and relaunches the DINO game process.
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
            Description = "Game installation path (auto-detected if omitted)"
        };

        Option<int> waitOpt = new("--wait")
        {
            Description = "Seconds to wait after launch for readiness check",
            DefaultValueFactory = _ => 15
        };

        Command command = new("relaunch", "Kill and relaunch the DINO game");
        command.Add(gamePathOpt);
        command.Add(waitOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            string? gamePathVal = parseResult.GetValue(gamePathOpt);
            int waitSeconds = parseResult.GetValue(waitOpt);

            string? gamePath = GamePathHelper.Detect(gamePathVal);
            if (gamePath is null)
            {
                Environment.ExitCode = 1;
                return;
            }

            string exePath = GamePathHelper.GetExePath(gamePath);
            if (!File.Exists(exePath))
            {
                AnsiConsole.MarkupLine($"[red]Error:[/] Game executable not found at {Markup.Escape(exePath)}");
                Environment.ExitCode = 1;
                return;
            }

            int exitCode = await RunRelaunchAsync(gamePath, exePath, waitSeconds, ct).ConfigureAwait(false);
            Environment.ExitCode = exitCode;
        });

        return command;
    }

    /// <summary>
    /// Kills existing game processes, relaunches, and waits for readiness.
    /// Returns 0 on success, 1 on failure.
    /// </summary>
    internal static async Task<int> RunRelaunchAsync(string gamePath, string exePath, int waitSeconds, CancellationToken ct)
    {
        // Step 1: Kill existing processes
        Process[] existing = Process.GetProcessesByName(ProcessName);
        if (existing.Length > 0)
        {
            AnsiConsole.MarkupLine($"[yellow]Killing {existing.Length} existing game process(es)...[/]");
            foreach (Process p in existing)
            {
                try
                {
                    p.Kill(entireProcessTree: true);
                }
                catch (Exception ex)
                {
                    AnsiConsole.MarkupLine($"[yellow]Warning:[/] Could not kill PID {p.Id}: {Markup.Escape(ex.Message)}");
                }
                finally
                {
                    p.Dispose();
                }
            }

            // Wait 3 seconds for processes to terminate
            AnsiConsole.MarkupLine("[dim]Waiting 3s for processes to exit...[/]");
            await Task.Delay(TimeSpan.FromSeconds(3), ct).ConfigureAwait(false);

            // Verify no processes remain
            Process[] remaining = Process.GetProcessesByName(ProcessName);
            if (remaining.Length > 0)
            {
                AnsiConsole.MarkupLine($"[red]Error:[/] {remaining.Length} game process(es) still running after kill attempt.");
                foreach (Process p in remaining) p.Dispose();
                return 1;
            }
        }
        else
        {
            AnsiConsole.MarkupLine("[dim]No existing game processes found.[/]");
        }

        // Step 2: Launch game
        AnsiConsole.MarkupLine($"[bold]Launching:[/] {Markup.Escape(exePath)}");
        ProcessStartInfo psi = new(exePath)
        {
            WorkingDirectory = gamePath,
            UseShellExecute = true
        };

        Process? gameProc = Process.Start(psi);
        if (gameProc is null)
        {
            AnsiConsole.MarkupLine("[red]Error:[/] Failed to start game process.");
            return 1;
        }

        AnsiConsole.MarkupLine($"[green]Launched[/] (PID {gameProc.Id})");

        // Step 3: Wait for readiness
        AnsiConsole.MarkupLine($"[dim]Waiting {waitSeconds}s for game to initialize...[/]");
        await Task.Delay(TimeSpan.FromSeconds(waitSeconds), ct).ConfigureAwait(false);

        // Step 4: Check process status
        gameProc.Refresh();
        if (gameProc.HasExited)
        {
            AnsiConsole.MarkupLine($"[red]Game exited[/] with code {gameProc.ExitCode} before readiness check.");
            gameProc.Dispose();
            return 1;
        }

        AnsiConsole.MarkupLine($"[green]Game running[/] (PID {gameProc.Id})");

        // Windows-specific: check MainWindowTitle for error dialogs
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            try
            {
                string title = gameProc.MainWindowTitle ?? "";
                if (title.Contains("Fatal", StringComparison.OrdinalIgnoreCase) ||
                    title.Contains("error", StringComparison.OrdinalIgnoreCase) ||
                    title.Contains("another instance", StringComparison.OrdinalIgnoreCase))
                {
                    AnsiConsole.MarkupLine($"[red]Launch FAILED[/] — window title: \"{Markup.Escape(title)}\"");
                    gameProc.Dispose();
                    return 1;
                }

                if (!string.IsNullOrWhiteSpace(title))
                {
                    AnsiConsole.MarkupLine($"  Window title: {Markup.Escape(title)}");
                }
            }
            catch
            {
                // Access denied or process already exited — non-fatal
            }
        }

        gameProc.Dispose();
        return 0;
    }
}
