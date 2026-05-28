#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using Newtonsoft.Json.Linq;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Displays or exports the live MetricsCollector snapshot from the running game bridge.
/// </summary>
internal static class MetricsCommand
{
    /// <summary>
    /// Creates the <c>metrics</c> command.
    /// Usage: <c>dinoforge metrics [--format json]</c>
    /// </summary>
    public static Command Create()
    {
        Command command = new("metrics", "Show runtime telemetry metrics from the running game");
        Option<string> formatOpt = CommandOutput.CreateFormatOption();
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            using GameClient? client = await CommandHelper.ConnectAsync(ct, writeErrors: !json).ConfigureAwait(false);
            if (client is null)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("game_not_running", "Game not running. Start DINO first.");
                }
                else
                {
                    AnsiConsole.MarkupLine("[red]Game not running.[/] Start DINO with DINOForge loaded first.");
                }

                return;
            }

            JObject metricsObj = await client.GetMetricsAsync(ct).ConfigureAwait(false);

            if (json)
            {
                AnsiConsole.WriteLine(metricsObj.ToString(Newtonsoft.Json.Formatting.Indented));
                return;
            }

            // Pretty-print as a Spectre table
            Table table = new Table()
                .Border(TableBorder.Rounded)
                .Title("[bold]DINOForge Runtime Metrics[/]")
                .AddColumn("Metric")
                .AddColumn("Value");

            bool any = false;
            foreach (System.Collections.Generic.KeyValuePair<string, JToken?> kv in metricsObj)
            {
                if (kv.Value is JObject entry)
                {
                    string typeVal = entry.Value<string>("type") ?? "?";
                    string rawVal  = entry.Value<string>("value") ?? entry.ToString();
                    string display = typeVal switch
                    {
                        "counter"  => $"[cyan]{Markup.Escape(rawVal)}[/]",
                        "gauge"    => $"[green]{Markup.Escape(rawVal)}[/]",
                        "duration" => $"[yellow]{Markup.Escape(rawVal)} ms[/]",
                        _          => Markup.Escape(rawVal)
                    };
                    table.AddRow(Markup.Escape(kv.Key), display);
                    any = true;
                }
            }

            if (!any)
            {
                table.AddRow("[dim]—[/]", "[dim]No metrics recorded yet[/]");
            }

            AnsiConsole.Write(table);
        });

        return command;
    }
}
