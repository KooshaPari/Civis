#nullable enable
using System.CommandLine;
using System.Diagnostics;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Manages optional DINOForge development tools (UnityExplorer, etc.).
/// </summary>
internal static class DevToolsCommand
{
    /// <summary>
    /// Creates the <c>dev-tools</c> command group.
    /// </summary>
    public static Command Create()
    {
        Command group = new("dev-tools", "Manage optional development tools");
        group.Add(CreateInstallCommand());
        return group;
    }

    /// <summary>
    /// Creates the <c>dev-tools install</c> subcommand.
    /// </summary>
    private static Command CreateInstallCommand()
    {
        Command command = new("install", "Install UnityExplorer and other dev tools");

        Option<string?> gamePathOpt = new("--game-path")
        {
            Description = "Path to DINO game installation (optional; auto-detected if not specified)"
        };

        Option<bool> forceOpt = new("--force")
        {
            Description = "Reinstall even if already present"
        };

        command.Add(gamePathOpt);
        command.Add(forceOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string? gamePath = parseResult.GetValue(gamePathOpt);
            bool force = parseResult.GetValue(forceOpt);
            await InstallDevToolsAsync(gamePath, force, ct).ConfigureAwait(false);
        });

        return command;
    }

    /// <summary>
    /// Executes the install action for dev tools.
    /// </summary>
    private static async Task InstallDevToolsAsync(string? gamePath, bool force, CancellationToken ct)
    {
        try
        {
            string resolvedGamePath = GamePathHelper.Detect(gamePath);
            string scriptPath = GetInstallScript();

            if (!File.Exists(scriptPath))
            {
                AnsiConsole.MarkupLine($"[red]Install script not found:[/] {Markup.Escape(scriptPath)}");
                Environment.ExitCode = 1;
                return;
            }

            List<string> args = new();
            string gamePathArgument = OperatingSystem.IsWindows()
                ? $"\"{resolvedGamePath}\""
                : $"\"{resolvedGamePath}\"";
            args.Add(OperatingSystem.IsWindows()
                ? $"-GamePath {gamePathArgument}"
                : $"--game-path {gamePathArgument}");

            if (force)
            {
                args.Add(OperatingSystem.IsWindows() ? "-Force" : "--force");
            }

            AnsiConsole.MarkupLine($"[bold]Installing dev tools[/] for {Markup.Escape(resolvedGamePath)}");
            await RunInstallScriptAsync(scriptPath, args, ct).ConfigureAwait(false);
        }
        catch (OperationCanceledException)
        {
            AnsiConsole.MarkupLine("[yellow]Dev tools installation cancelled.[/]");
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]Dev tools installation failed:[/] {Markup.Escape(ex.Message)}");
            Environment.ExitCode = 1;
        }
    }

    /// <summary>
    /// Gets the path to the appropriate install script based on the current platform.
    /// </summary>
    private static string GetInstallScript()
    {
        string repoRoot = FindRepoRoot();

        if (OperatingSystem.IsWindows())
        {
            return Path.Combine(repoRoot, "scripts", "install-dev-tools.ps1");
        }
        else
        {
            return Path.Combine(repoRoot, "scripts", "install-dev-tools.sh");
        }
    }

    private static string FindRepoRoot()
    {
        string current = AppContext.BaseDirectory;
        while (!string.IsNullOrWhiteSpace(current))
        {
            string propsPath = Path.Combine(current, "Directory.Build.props");
            if (File.Exists(propsPath))
            {
                return current;
            }

            DirectoryInfo? parent = Directory.GetParent(current);
            if (parent is null)
            {
                break;
            }

            current = parent.FullName;
        }

        return Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", ".."));
    }

    /// <summary>
    /// Runs the install script and streams its output.
    /// </summary>
    private static async Task RunInstallScriptAsync(string scriptPath, List<string> args, CancellationToken ct)
    {
        ProcessStartInfo psi = new();

        if (OperatingSystem.IsWindows())
        {
            // PowerShell on Windows
            psi.FileName = "powershell.exe";
            psi.Arguments = $"-NoProfile -ExecutionPolicy Bypass -File \"{scriptPath}\" {string.Join(" ", args)}";
        }
        else
        {
            // Bash on Unix-like systems
            psi.FileName = "/bin/bash";
            psi.Arguments = $"\"{scriptPath}\" {string.Join(" ", args)}";
        }

        psi.UseShellExecute = false;
        psi.RedirectStandardOutput = true;
        psi.RedirectStandardError = true;
        psi.CreateNoWindow = false;

        using Process? process = Process.Start(psi);
        if (process is null)
        {
            throw new InvalidOperationException("Failed to start install script process.");
        }

        bool exited = await Task.Run(() => process.WaitForExit(TimeSpan.FromMinutes(5)), ct).ConfigureAwait(false);

        if (!exited)
        {
            AnsiConsole.MarkupLine("[yellow]Warning: Installation script timed out after 5 minutes.[/]");
            process.Kill();
        }

        if (process.ExitCode != 0)
        {
            AnsiConsole.MarkupLine($"[yellow]Install script exited with code {process.ExitCode}.[/]");
            throw new InvalidOperationException(
                $"Installation script exited with code {process.ExitCode}."
            );
        }

        AnsiConsole.MarkupLine("[green]Dev tools installation completed successfully.[/]");
    }
}
