#nullable enable
using System.CommandLine;
using System.CommandLine.Parsing;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

internal static class NewPackCommand
{
    public static Command Create()
    {
        var nameArg = new Argument<string>("name") { Description = "Pack ID (kebab-case, e.g. my-awesome-mod)" };
        var authorOpt = new Option<string>("--author") { Description = "Author name (default: Anonymous)" };
        var typeOpt = new Option<string>("--type") { Description = "Pack type: content, balance, ruleset, scenario, total_conversion, utility (default: content)" };
        var outputOpt = new Option<string>("--output") { Description = "Output directory (default: packs/)" };

        var command = new Command("new", "Scaffold a new mod pack from the built-in template")
        {
            nameArg,
            authorOpt,
            typeOpt,
            outputOpt,
        };

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            await System.Threading.Tasks.Task.CompletedTask.ConfigureAwait(false);
            string name = parseResult.GetValue(nameArg)!;
            string author = parseResult.GetValue(authorOpt) ?? "Anonymous";
            string type = parseResult.GetValue(typeOpt) ?? "content";
            string output = parseResult.GetValue(outputOpt) ?? "packs";

            string targetDir = Path.Combine(output, name);
            if (Directory.Exists(targetDir))
            {
                AnsiConsole.MarkupLine($"[red]Error:[/] Directory '{targetDir}' already exists.");
                return;
            }

            string? templateDir = FindTemplateDir();
            if (templateDir == null)
            {
                AnsiConsole.MarkupLine("[red]Error:[/] Pack template not found. Expected src/Templates/PackTemplate/");
                return;
            }

            Directory.CreateDirectory(targetDir);
            CopyTemplateRecursive(templateDir, targetDir, name, author, type);

            AnsiConsole.MarkupLine($"[green]Created pack '{name}' at {targetDir}/[/]");
            AnsiConsole.MarkupLine("");
            AnsiConsole.MarkupLine("[dim]Next steps:[/]");
            AnsiConsole.MarkupLine($"  1. Edit [cyan]{targetDir}/pack.yaml[/] to customize metadata");
            AnsiConsole.MarkupLine($"  2. Add units/buildings/factions YAML in subdirectories");
            AnsiConsole.MarkupLine($"  3. Validate: [cyan]dinoforge verify-pack {targetDir}[/]");
            AnsiConsole.MarkupLine($"  4. Deploy:   [cyan]dotnet build -p:DeployToGame=true[/]");
        });

        return command;
    }

    private static string? FindTemplateDir()
    {
        string[] candidates = new[]
        {
            Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "Templates", "PackTemplate"),
            Path.Combine(Directory.GetCurrentDirectory(), "src", "Templates", "PackTemplate"),
            Path.Combine(Directory.GetCurrentDirectory(), "Templates", "PackTemplate"),
        };

        foreach (string candidate in candidates)
        {
            string resolved = Path.GetFullPath(candidate);
            if (Directory.Exists(resolved) && File.Exists(Path.Combine(resolved, "pack.yaml")))
                return resolved;
        }

        return null;
    }

    private static void CopyTemplateRecursive(string sourceDir, string targetDir, string packName, string author, string type)
    {
        foreach (string file in Directory.GetFiles(sourceDir))
        {
            string fileName = Path.GetFileName(file);
            if (string.Equals(fileName, "template.json", StringComparison.Ordinal)) continue;

            string content = File.ReadAllText(file, System.Text.Encoding.UTF8);
            content = content
                .Replace("pack-template", packName, StringComparison.Ordinal)
                .Replace("Pack Template", packName, StringComparison.Ordinal)
                .Replace("DINOForge Agents", author, StringComparison.Ordinal)
                .Replace("type: content", $"type: {type}", StringComparison.Ordinal);

            string targetPath = Path.Combine(targetDir, fileName);
            File.WriteAllText(targetPath, content, System.Text.Encoding.UTF8);
        }

        foreach (string dir in Directory.GetDirectories(sourceDir))
        {
            string dirName = Path.GetFileName(dir);
            if (string.Equals(dirName, ".template.config", StringComparison.Ordinal)) continue;

            string targetSubDir = Path.Combine(targetDir, dirName);
            Directory.CreateDirectory(targetSubDir);
            CopyTemplateRecursive(dir, targetSubDir, packName, author, type);
        }
    }
}
