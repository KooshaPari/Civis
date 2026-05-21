#nullable enable
using System.CommandLine;
using System.Text.Json;
using DINOForge.Tools.Installer;
using DINOForge.Tools.Installer.Json;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Offline install-health and repair commands for a game directory.
/// </summary>
internal static class InstallCommand
{
    /// <summary>
    /// Creates the <c>install</c> command with offline maintenance subcommands.
    /// </summary>
    public static Command Create()
    {
        Command command = new("install", "Inspect or repair the on-disk DINOForge installation");
        command.Add(CreateStatusCommand());
        command.Add(CreateRepairCommand());
        return command;
    }

    private static Command CreateStatusCommand()
    {
        Command command = new("status", "Inspect DINOForge installation health on disk");
        Option<string?> gameDirOption = new("--game-dir") { Description = "Path to the DINO game directory" };
        Option<string> formatOption = new("--format")
        {
            Description = "Output format: table or json",
            DefaultValueFactory = _ => "table"
        };

        command.Add(gameDirOption);
        command.Add(formatOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string? gameDir = parseResult.GetValue(gameDirOption);
            string format = parseResult.GetValue(formatOption) ?? "table";

            string resolvedPath = ResolveGamePath(gameDir);
            ct.ThrowIfCancellationRequested();
            InstallStatus status = await Task.Run(() => InstallVerifier.Verify(resolvedPath), ct).ConfigureAwait(false);
            ct.ThrowIfCancellationRequested();
            InstallInspection inspection = await Task.Run(() => InstallLifecycle.Inspect(resolvedPath), ct).ConfigureAwait(false);
            ct.ThrowIfCancellationRequested();

            if (string.Equals(format, "json", StringComparison.OrdinalIgnoreCase))
            {
                object payload = new
                {
                    gamePath = resolvedPath,
                    isFullyInstalled = status.IsFullyInstalled,
                    runtimeInstalled = status.RuntimeInstalled,
                    packsReady = status.PacksReady,
                    manifestPresent = status.ManifestPresent,
                    hasLegacyArtifacts = status.HasLegacyArtifacts,
                    installedVersion = inspection.InstalledVersion,
                    primaryRuntimePath = inspection.PrimaryRuntimePath,
                    issues = status.Issues,
                    warnings = status.Warnings,
                    legacyArtifacts = inspection.LegacyArtifacts,
                    managedFiles = inspection.ManagedFiles,
                };

                AnsiConsole.WriteLine(JsonSerializer.Serialize(payload, InstallerJsonOptions.Default));
                return;
            }

            Table table = new Table()
                .Border(TableBorder.Rounded)
                .Title("[bold]DINOForge Install Status[/]")
                .AddColumn("Property")
                .AddColumn("Value");

            table.AddRow("Game Path", Markup.Escape(resolvedPath));
            table.AddRow("Installed Version", Markup.Escape(inspection.InstalledVersion));
            table.AddRow("Runtime Installed", status.RuntimeInstalled ? "[green]Yes[/]" : "[red]No[/]");
            table.AddRow("Manifest Present", status.ManifestPresent ? "[green]Yes[/]" : "[yellow]No[/]");
            table.AddRow("Legacy Artifacts", status.HasLegacyArtifacts ? $"[red]{inspection.LegacyArtifacts.Count}[/]" : "[green]0[/]");
            table.AddRow("Packs Ready", status.PacksReady ? "[green]Yes[/]" : "[yellow]No[/]");
            table.AddRow("Primary Runtime", inspection.PrimaryRuntimePath is null ? "[dim]None[/]" : Markup.Escape(inspection.PrimaryRuntimePath));

            AnsiConsole.Write(table);

            if (status.Issues.Count > 0)
            {
                AnsiConsole.MarkupLine("\n[red]Issues[/]");
                foreach (string issue in status.Issues)
                {
                    AnsiConsole.MarkupLine($"  - {Markup.Escape(issue)}");
                }
            }

            if (status.Warnings.Count > 0)
            {
                AnsiConsole.MarkupLine("\n[yellow]Warnings[/]");
                foreach (string warning in status.Warnings)
                {
                    AnsiConsole.MarkupLine($"  - {Markup.Escape(warning)}");
                }
            }
        });

        return command;
    }

    private static Command CreateRepairCommand()
    {
        Command command = new("repair", "Clean legacy install artifacts and refresh the install manifest");
        Option<string?> gameDirOption = new("--game-dir") { Description = "Path to the DINO game directory" };
        Option<bool> dryRunOption = new("--dry-run") { Description = "Report actions without changing files" };
        Option<string> formatOption = new("--format")
        {
            Description = "Output format: table or json",
            DefaultValueFactory = _ => "table"
        };

        command.Add(gameDirOption);
        command.Add(dryRunOption);
        command.Add(formatOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string? gameDir = parseResult.GetValue(gameDirOption);
            bool dryRun = parseResult.GetValue(dryRunOption);
            string format = parseResult.GetValue(formatOption) ?? "table";

            string resolvedPath = ResolveGamePath(gameDir);
            ct.ThrowIfCancellationRequested();
            InstallInspection before = await Task.Run(() => InstallLifecycle.Inspect(resolvedPath), ct).ConfigureAwait(false);
            int removedLegacy = 0;
            string? manifestPath = null;

            if (!dryRun)
            {
                ct.ThrowIfCancellationRequested();
                removedLegacy = await Task.Run(() => InstallLifecycle.CleanupLegacyArtifacts(resolvedPath), ct).ConfigureAwait(false);
                ct.ThrowIfCancellationRequested();
                await Task.Run(() => InstallLifecycle.MigrateLegacyPacks(resolvedPath), ct).ConfigureAwait(false);
                if (before.RuntimeInstalled)
                {
                    ct.ThrowIfCancellationRequested();
                    manifestPath = await Task.Run(() => InstallLifecycle.WriteManifest(resolvedPath, InstallDetector.GetInstalledVersion(resolvedPath)), ct).ConfigureAwait(false);
                }
            }

            ct.ThrowIfCancellationRequested();
            InstallStatus afterStatus = await Task.Run(() => InstallVerifier.Verify(resolvedPath), ct).ConfigureAwait(false);
            ct.ThrowIfCancellationRequested();
            InstallInspection after = await Task.Run(() => InstallLifecycle.Inspect(resolvedPath), ct).ConfigureAwait(false);

            if (string.Equals(format, "json", StringComparison.OrdinalIgnoreCase))
            {
                object payload = new
                {
                    gamePath = resolvedPath,
                    dryRun,
                    removedLegacyArtifacts = dryRun ? before.LegacyArtifacts.Count : removedLegacy,
                    manifestPath,
                    issues = afterStatus.Issues,
                    warnings = afterStatus.Warnings,
                    isHealthy = after.IsHealthy,
                };

                AnsiConsole.WriteLine(JsonSerializer.Serialize(payload, InstallerJsonOptions.Default));
                return;
            }

            AnsiConsole.MarkupLine(dryRun
                ? "[yellow]Dry run only[/]"
                : $"[green]Repair complete[/] Removed {removedLegacy} legacy artifact(s).");

            if (!string.IsNullOrEmpty(manifestPath))
            {
                AnsiConsole.MarkupLine($"Manifest: [dim]{Markup.Escape(manifestPath)}[/]");
            }

            if (afterStatus.Issues.Count > 0)
            {
                AnsiConsole.MarkupLine("\n[red]Remaining issues[/]");
                foreach (string issue in afterStatus.Issues)
                {
                    AnsiConsole.MarkupLine($"  - {Markup.Escape(issue)}");
                }
            }
            else
            {
                AnsiConsole.MarkupLine("[green]Install health looks clean.[/]");
            }

            if (afterStatus.Warnings.Count > 0)
            {
                AnsiConsole.MarkupLine("\n[yellow]Warnings[/]");
                foreach (string warning in afterStatus.Warnings)
                {
                    AnsiConsole.MarkupLine($"  - {Markup.Escape(warning)}");
                }
            }
        });

        return command;
    }

    private static string ResolveGamePath(string? explicitGamePath)
    {
        if (!string.IsNullOrWhiteSpace(explicitGamePath))
        {
            return explicitGamePath;
        }

        string? detected = SteamLocator.FindDinoInstallPath();
        if (!string.IsNullOrWhiteSpace(detected))
        {
            return detected;
        }

        throw new InvalidOperationException("Could not determine game path automatically. Pass --game-dir.");
    }
}
