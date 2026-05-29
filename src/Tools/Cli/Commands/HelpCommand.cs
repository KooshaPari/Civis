#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.CommandLine;
using DINOForge.SDK.Json;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// CLI command to display FAQ/help content from faq.json.
/// Usage: dinoforge help [category] [question-index]
/// </summary>
internal static class HelpCommand
{
    // ── FAQ Data Structure ───────────────────────────────────────────────────
    [Serializable]
    private sealed class FaqCategory
    {
        [JsonPropertyName("name")]
        public string Name { get; set; } = "";

        [JsonPropertyName("items")]
        public List<FaqItem> Items { get; set; } = new();
    }

    [Serializable]
    private sealed class FaqItem
    {
        [JsonPropertyName("q")]
        public string Question { get; set; } = "";

        [JsonPropertyName("a")]
        public string Answer { get; set; } = "";
    }

    [Serializable]
    private sealed class FaqRoot
    {
        [JsonPropertyName("categories")]
        public List<FaqCategory> Categories { get; set; } = new();
    }

    // ── Cache ────────────────────────────────────────────────────────────
    private static FaqRoot? _faqData;

    /// <summary>
    /// Creates the <c>help</c> command.
    /// </summary>
    public static Command Create()
    {
        Command command = new("help", "Display FAQ and help topics");
        Argument<string?> categoryArg = new("category") { Description = "FAQ category name (optional)", Arity = ArgumentArity.ZeroOrOne };
        Argument<int?> indexArg = new("index") { Description = "Question index within category (optional)", Arity = ArgumentArity.ZeroOrOne };

        command.Add(categoryArg);
        command.Add(indexArg);
        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            string? category = parseResult.GetValue(categoryArg);
            int? index = parseResult.GetValue(indexArg);

            try
            {
                // Load FAQ data
            if (_faqData == null && !LoadFaqData())
            {
                AnsiConsole.MarkupLine("[red]Error: Could not load FAQ data.[/]");
                return;
            }

            if (_faqData?.Categories == null || _faqData.Categories.Count == 0)
            {
                AnsiConsole.MarkupLine("[yellow]No FAQ data available.[/]");
                return;
            }

            // No arguments: show all categories
            if (string.IsNullOrEmpty(category))
            {
                ShowAllCategories();
                return;
            }

            // Find the category (case-insensitive)
            FaqCategory? foundCategory = _faqData.Categories
                .Find(c => c.Name.Equals(category, StringComparison.OrdinalIgnoreCase));

            if (foundCategory == null)
            {
                AnsiConsole.MarkupLine($"[red]Category not found: {Markup.Escape(category)}[/]");
                AnsiConsole.MarkupLine("[dim]Available categories:[/]");
                foreach (var cat in _faqData.Categories)
                {
                    AnsiConsole.MarkupLine($"  - {Markup.Escape(cat.Name)}");
                }

                return;
            }

            // Category only: show all Q/A in category
            if (index == null)
            {
                ShowCategory(foundCategory);
                return;
            }

            // Category + index: show single Q/A
            if (index.Value < 0 || index.Value >= foundCategory.Items.Count)
            {
                AnsiConsole.MarkupLine($"[red]Question index out of range: {index}[/]");
                return;
            }

            ShowQuestion(foundCategory.Items[index.Value]);
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]Error: {Markup.Escape(ex.Message)}[/]");
        }
        });

        return command;
    }

    // ── Display Methods ──────────────────────────────────────────────────

    private static void ShowAllCategories()
    {
        var table = new Table()
            .Border(TableBorder.Rounded)
            .Title("[bold]DINOForge FAQ Categories[/]")
            .AddColumn("Category")
            .AddColumn("Questions");

        if (_faqData?.Categories != null)
        {
            foreach (var cat in _faqData.Categories)
            {
                table.AddRow(
                    Markup.Escape(cat.Name),
                    cat.Items.Count.ToString()
                );
            }
        }

        AnsiConsole.Write(table);
        AnsiConsole.MarkupLine("");
        AnsiConsole.MarkupLine("[dim]Usage: dinoforge help <category> [index][/]");
        AnsiConsole.MarkupLine("[dim]Example: dinoforge help 'Getting Started'[/]");
    }

    private static void ShowCategory(FaqCategory category)
    {
        AnsiConsole.MarkupLine($"[bold yellow]{Markup.Escape(category.Name)}[/]");
        AnsiConsole.MarkupLine("");

        var panel = new Panel("");
        var tree = new Tree($"[bold]{Markup.Escape(category.Name)}[/]");

        for (int i = 0; i < category.Items.Count; i++)
        {
            var item = category.Items[i];
            string qText = $"[bold cyan]{i}. {Markup.Escape(item.Question)}[/]";
            var qNode = tree.AddNode(qText);
            qNode.AddNode($"[green]{Markup.Escape(item.Answer)}[/]");
        }

        AnsiConsole.Write(tree);
        AnsiConsole.MarkupLine("");
        AnsiConsole.MarkupLine("[dim]Usage: dinoforge help 'Getting Started' <index>[/]");
    }

    private static void ShowQuestion(FaqItem item)
    {
        var table = new Table()
            .Border(TableBorder.Rounded)
            .AddColumn(new TableColumn("[bold]Question[/]").NoWrap())
            .AddColumn(new TableColumn("[bold]Answer[/]"));

        table.AddRow(
            $"[cyan]{Markup.Escape(item.Question)}[/]",
            $"[green]{Markup.Escape(item.Answer)}[/]"
        );

        AnsiConsole.Write(table);
    }

    // ── FAQ Loading ──────────────────────────────────────────────────────

    private static bool LoadFaqData()
    {
        try
        {
            // Try to load from assets/help/faq.json
            string? faqPath = null;

            // Check relative to current directory first
            var relPath = Path.Combine("assets", "help", "faq.json");
            if (File.Exists(relPath))
            {
                faqPath = relPath;
            }
            else
            {
                // Check in repo root
                var rootPath = Path.Combine("..", "..", "assets", "help", "faq.json");
                if (File.Exists(rootPath))
                {
                    faqPath = rootPath;
                }
            }

            if (faqPath == null || !File.Exists(faqPath))
            {
                AnsiConsole.MarkupLine("[yellow]Warning: FAQ file not found (assets/help/faq.json)[/]");
                return false;
            }

            string json = File.ReadAllText(faqPath);
            _faqData = JsonSerializer.Deserialize<FaqRoot>(json, JsonOptions.Compact);
            return _faqData?.Categories != null && _faqData.Categories.Count > 0;
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]Error loading FAQ: {Markup.Escape(ex.Message)}[/]");
            return false;
        }
    }
}
