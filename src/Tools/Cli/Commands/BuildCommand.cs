#nullable enable
using System.CommandLine;
using System.Diagnostics;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Builds the DINOForge Runtime DLL via <c>dotnet build</c>.
/// </summary>
internal static class BuildCommand
{
    private const string RuntimeProject = "src/Runtime/DINOForge.Runtime.csproj";
    private const string TargetFramework = "netstandard2.0";

    /// <summary>
    /// Creates the <c>build</c> command.
    /// </summary>
    public static Command Create()
    {
        Option<string> configOpt = new("--configuration", "-c")
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
    /// Runs <c>dotnet build</c> for the Runtime project, streaming colorized output.
    /// </summary>
    /// <param name="config">Configuration (e.g. Release).</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>Process exit code (0 on success).</returns>
    internal static async Task<int> RunBuildAsync(string config, CancellationToken ct)
    {
        AnsiConsole.MarkupLine($"[bold]Building[/] {Markup.Escape(RuntimeProject)} ([cyan]{Markup.Escape(config)}[/], TFM=[cyan]{TargetFramework}[/])");

        Stopwatch sw = Stopwatch.StartNew();

        ProcessStartInfo psi = new("dotnet")
        {
            UseShellExecute = false,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            CreateNoWindow = true
        };
        psi.ArgumentList.Add("build");
        psi.ArgumentList.Add(RuntimeProject);
        psi.ArgumentList.Add("-c");
        psi.ArgumentList.Add(config);
        psi.ArgumentList.Add($"-p:TargetFramework={TargetFramework}");

        using Process process = new() { StartInfo = psi };

        process.OutputDataReceived += (_, e) =>
        {
            if (e.Data is null) return;
            AnsiConsole.MarkupLine(ColorizeLine(e.Data));
        };
        process.ErrorDataReceived += (_, e) =>
        {
            if (e.Data is null) return;
            AnsiConsole.MarkupLine($"[red]{Markup.Escape(e.Data)}[/]");
        };

        try
        {
            process.Start();
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]Failed to start dotnet:[/] {Markup.Escape(ex.Message)}");
            return 1;
        }

        process.BeginOutputReadLine();
        process.BeginErrorReadLine();

        try
        {
            await process.WaitForExitAsync(ct).ConfigureAwait(false);
        }
        catch (OperationCanceledException)
        {
            try { process.Kill(entireProcessTree: true); } catch { /* best-effort */ }
            throw;
        }

        sw.Stop();

        if (process.ExitCode == 0)
        {
            AnsiConsole.MarkupLine($"[green]Build succeeded[/] in {sw.Elapsed.TotalSeconds:F2}s");
            string dllPath = GetOutputDllPath(config);
            if (File.Exists(dllPath))
            {
                long size = new FileInfo(dllPath).Length;
                AnsiConsole.MarkupLine($"[green]Output:[/] {Markup.Escape(dllPath)} ([cyan]{size:N0} bytes[/])");
            }
        }
        else
        {
            AnsiConsole.MarkupLine($"[red]Build failed[/] (exit code {process.ExitCode}) after {sw.Elapsed.TotalSeconds:F2}s");
        }

        return process.ExitCode;
    }

    /// <summary>
    /// Returns the expected path to the built Runtime DLL.
    /// </summary>
    internal static string GetOutputDllPath(string config) =>
        Path.Combine("src", "Runtime", "bin", config, TargetFramework, "DINOForge.Runtime.dll");

    private static string ColorizeLine(string line)
    {
        string escaped = Markup.Escape(line);
        if (line.Contains(": error", StringComparison.OrdinalIgnoreCase) || line.Contains("Build FAILED", StringComparison.OrdinalIgnoreCase))
        {
            return $"[red]{escaped}[/]";
        }

        if (line.Contains(": warning", StringComparison.OrdinalIgnoreCase))
        {
            return $"[yellow]{escaped}[/]";
        }

        return escaped;
    }
}
