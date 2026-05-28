#nullable enable
using System.CommandLine;
using System.Diagnostics;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Builds the DINOForge Runtime DLL targeting netstandard2.0.
/// </summary>
internal static class BuildCommand
{
    /// <summary>
    /// Creates the <c>build</c> command.
    /// </summary>
    public static Command Create()
    {
        Option<string> configOpt = new("--configuration")
        {
            Description = "Build configuration (Release or Debug)",
            DefaultValueFactory = _ => "Release"
        };

        Command command = new("build", "Build the DINOForge Runtime DLL");
        command.Add(configOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            string config = parseResult.GetValue(configOpt) ?? "Release";
            int exitCode = await RunBuildAsync(config, ct).ConfigureAwait(false);
            Environment.ExitCode = exitCode;
        });

        return command;
    }

    /// <summary>
    /// Runs the dotnet build for the Runtime project. Returns the process exit code.
    /// </summary>
    internal static async Task<int> RunBuildAsync(string configuration, CancellationToken ct)
    {
        // Resolve the Runtime csproj relative to the repo root.
        string repoRoot = FindRepoRoot();
        string csproj = Path.Combine(repoRoot, "src", "Runtime", "DINOForge.Runtime.csproj");

        if (!File.Exists(csproj))
        {
            AnsiConsole.MarkupLine($"[red]Error:[/] Runtime project not found at {Markup.Escape(csproj)}");
            return 1;
        }

        AnsiConsole.MarkupLine($"[bold]Building[/] DINOForge.Runtime ({Markup.Escape(configuration)}) ...");

        Stopwatch sw = Stopwatch.StartNew();

        ProcessStartInfo psi = new("dotnet", $"build \"{csproj}\" -c {configuration} -p:TargetFramework=netstandard2.0")
        {
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true
        };

        using Process? proc = Process.Start(psi);
        if (proc is null)
        {
            AnsiConsole.MarkupLine("[red]Error:[/] Failed to start dotnet build process.");
            return 1;
        }

        // Stream output
        Task readOut = Task.Run(async () =>
        {
            while (await proc.StandardOutput.ReadLineAsync(ct).ConfigureAwait(false) is { } line)
            {
                if (line.Contains("error", StringComparison.OrdinalIgnoreCase))
                {
                    AnsiConsole.MarkupLine($"  [red]{Markup.Escape(line)}[/]");
                }
                else if (line.Contains("warning", StringComparison.OrdinalIgnoreCase))
                {
                    AnsiConsole.MarkupLine($"  [yellow]{Markup.Escape(line)}[/]");
                }
                else
                {
                    AnsiConsole.MarkupLine($"  [dim]{Markup.Escape(line)}[/]");
                }
            }
        }, ct);

        Task readErr = Task.Run(async () =>
        {
            while (await proc.StandardError.ReadLineAsync(ct).ConfigureAwait(false) is { } line)
            {
                AnsiConsole.MarkupLine($"  [red]{Markup.Escape(line)}[/]");
            }
        }, ct);

        await proc.WaitForExitAsync(ct).ConfigureAwait(false);
        await Task.WhenAll(readOut, readErr).ConfigureAwait(false);
        sw.Stop();

        int exitCode = proc.ExitCode;

        if (exitCode == 0)
        {
            string dllPath = Path.Combine(repoRoot, "src", "Runtime", "bin", configuration, "netstandard2.0", "DINOForge.Runtime.dll");
            if (File.Exists(dllPath))
            {
                long sizeBytes = new FileInfo(dllPath).Length;
                AnsiConsole.MarkupLine($"\n[green]Build succeeded[/] in {sw.Elapsed.TotalSeconds:F1}s");
                AnsiConsole.MarkupLine($"  Output: {Markup.Escape(dllPath)}");
                AnsiConsole.MarkupLine($"  Size:   {sizeBytes / 1024.0:F1} KB");
            }
            else
            {
                AnsiConsole.MarkupLine($"\n[green]Build succeeded[/] in {sw.Elapsed.TotalSeconds:F1}s");
                AnsiConsole.MarkupLine("[yellow]Warning:[/] Output DLL not found at expected path.");
            }
        }
        else
        {
            AnsiConsole.MarkupLine($"\n[red]Build FAILED[/] (exit code {exitCode}) after {sw.Elapsed.TotalSeconds:F1}s");
        }

        return exitCode;
    }

    /// <summary>
    /// Walks up from the current directory to find the repo root (contains .git or DINOForge.sln).
    /// </summary>
    private static string FindRepoRoot()
    {
        string? dir = Directory.GetCurrentDirectory();
        while (dir is not null)
        {
            if (Directory.Exists(Path.Combine(dir, ".git")) ||
                File.Exists(Path.Combine(dir, "src", "DINOForge.sln")))
            {
                return dir;
            }

            dir = Directory.GetParent(dir)?.FullName;
        }

        // Fallback to current directory
        return Directory.GetCurrentDirectory();
    }
}
