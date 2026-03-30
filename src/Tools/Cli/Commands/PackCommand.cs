#nullable enable
using System.CommandLine;
using DINOForge.SDK.Dependencies;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Manages pack repositories as git submodules with add, list, update, and lock commands.
/// </summary>
internal static class PackCommand
{
    /// <summary>
    /// Creates the <c>pack</c> command group with subcommands.
    /// </summary>
    public static Command Create()
    {
        Command packCommand = new("pack", "Manage pack repositories as git submodules");
        packCommand.Add(CreateAddCommand());
        packCommand.Add(CreateListCommand());
        packCommand.Add(CreateUpdateCommand());
        packCommand.Add(CreateLockCommand());
        return packCommand;
    }

    /// <summary>
    /// Creates the <c>pack add</c> subcommand.
    /// </summary>
    private static Command CreateAddCommand()
    {
        Argument<string> repoUrlArg = new("repoUrl") { Description = "Repository URL (HTTPS or SSH format)" };
        Option<string?> pathOpt = new("--path") { Description = "Optional submodule path (defaults to packs/{repo-name})" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        Command command = new("add", "Add a pack repository as a git submodule");
        command.Add(repoUrlArg);
        command.Add(pathOpt);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            string repoUrl = parseResult.GetRequiredValue(repoUrlArg);
            string? path = parseResult.GetValue(pathOpt);

            try
            {
                var manager = new PackSubmoduleManager();
                await manager.AddPackAsync(repoUrl, path);

                if (json)
                {
                    CommandOutput.WriteJson(new { status = "success", message = string.Format("Pack added from {0}", repoUrl) });
                }
                else
                {
                    string displayPath = path ?? string.Format("packs/{0}", repoUrl.Split('/').Last().TrimEnd('/'));
                    AnsiConsole.MarkupLine(string.Format("[green]✓[/] Pack added: [bold]{0}[/]", Markup.Escape(displayPath)));
                }
            }
            catch (Exception ex)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("git_error", ex.Message);
                }
                else
                {
                    AnsiConsole.MarkupLine(string.Format("[red]✗ Error:[/] {0}", ex.Message));
                }

                Environment.Exit(1);
            }
        });

        return command;
    }

    /// <summary>
    /// Creates the <c>pack list</c> subcommand.
    /// </summary>
    private static Command CreateListCommand()
    {
        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        Command command = new("list", "List installed pack submodules");
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);

            try
            {
                var manager = new PackSubmoduleManager();
                var packs = manager.ListPacks();

                if (json)
                {
                    var jsonPacks = packs.Select(p => new { path = p.Path, url = p.Url }).ToList();
                    CommandOutput.WriteJson(new { packs = jsonPacks });
                    return;
                }

                if (packs.Count == 0)
                {
                    AnsiConsole.MarkupLine("[yellow]No pack submodules found[/]");
                    return;
                }

                Table table = new Table()
                    .Border(TableBorder.Rounded)
                    .Title("[bold]Pack Submodules[/]")
                    .AddColumn("Path")
                    .AddColumn("Repository URL");

                foreach (var pack in packs)
                {
                    table.AddRow(Markup.Escape(pack.Path), Markup.Escape(pack.Url));
                }

                AnsiConsole.Write(table);
                AnsiConsole.MarkupLine(string.Format("\n[green]✓[/] Found [bold]{0}[/] pack(s)", packs.Count));
            }
            catch (Exception ex)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("error", ex.Message);
                }
                else
                {
                    AnsiConsole.MarkupLine(string.Format("[red]✗ Error:[/] {0}", ex.Message));
                }

                Environment.Exit(1);
            }
        });

        return command;
    }

    /// <summary>
    /// Creates the <c>pack update</c> subcommand.
    /// </summary>
    private static Command CreateUpdateCommand()
    {
        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        Command command = new("update", "Update all pack submodules to latest remote versions");
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);

            try
            {
                var manager = new PackSubmoduleManager();
                await manager.UpdateAllAsync();

                if (json)
                {
                    CommandOutput.WriteJson(new { status = "success", message = "All pack submodules updated" });
                }
                else
                {
                    AnsiConsole.MarkupLine("[green]✓[/] All pack submodules updated successfully");
                }
            }
            catch (Exception ex)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("git_error", ex.Message);
                }
                else
                {
                    AnsiConsole.MarkupLine(string.Format("[red]✗ Error:[/] {0}", ex.Message));
                }

                Environment.Exit(1);
            }
        });

        return command;
    }

    /// <summary>
    /// Creates the <c>pack lock</c> subcommand.
    /// </summary>
    private static Command CreateLockCommand()
    {
        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        Command command = new("lock", "Generate packs.lock file with current pack versions");
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);

            try
            {
                var manager = new PackSubmoduleManager();
                await manager.GenerateLockFile();

                var lockData = manager.ReadLockFile();

                if (json)
                {
                    var entries = lockData.Select(kvp => new { path = kvp.Key, sha = kvp.Value }).ToList();
                    CommandOutput.WriteJson(new { status = "success", entries = entries });
                }
                else
                {
                    AnsiConsole.MarkupLine(string.Format("[green]✓[/] packs.lock generated with [bold]{0}[/] entries", lockData.Count));

                    if (lockData.Count > 0)
                    {
                        Table table = new Table()
                            .Border(TableBorder.Rounded)
                            .Title("[bold]Lock File Contents[/]")
                            .AddColumn("Pack Path")
                            .AddColumn("Commit SHA");

                        foreach (var entry in lockData)
                        {
                            table.AddRow(Markup.Escape(entry.Key), Markup.Escape(entry.Value));
                        }

                        AnsiConsole.Write(table);
                    }
                }
            }
            catch (Exception ex)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("git_error", ex.Message);
                }
                else
                {
                    AnsiConsole.MarkupLine(string.Format("[red]✗ Error:[/] {0}", ex.Message));
                }

                Environment.Exit(1);
            }
        });

        return command;
    }
}
