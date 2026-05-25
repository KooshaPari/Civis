#nullable enable
using System;
using System.Text.Json;
using System.Text.Json.Serialization;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Assetctl
{
    /// <summary>
    /// Shared output formatting helpers for assetctl CLI commands.
    /// Supports both human-readable table output (default) and JSON output.
    /// </summary>
    public static class AssetctlOutput
    {
        private static readonly JsonSerializerOptions _jsonOptions = new(JsonSerializerDefaults.Web)
        {
            WriteIndented = true,
            DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        };

        /// <summary>Returns true when the given format string requests JSON output.</summary>
        public static bool IsJsonOutput(string? format) =>
            string.Equals(format, "json", StringComparison.OrdinalIgnoreCase);

        /// <summary>Serialises <paramref name="value"/> as indented JSON and writes it to stdout.</summary>
        public static void WriteJson(object value) =>
            Console.WriteLine(JsonSerializer.Serialize(value, _jsonOptions));

        /// <summary>
        /// Writes a human-readable error to stderr (table mode) or a JSON error payload to stdout (json mode).
        /// </summary>
        public static void WriteError(string command, string message, string? format)
        {
            if (IsJsonOutput(format))
            {
                WriteJson(new { success = false, command, error = message });
            }
            else
            {
                AnsiConsole.MarkupLine($"[red]Error ({command}):[/] {Markup.Escape(message)}");
            }
        }

        /// <summary>
        /// Writes a success heading followed by dim key/value detail lines (table mode only).
        /// </summary>
        /// <param name="heading">The success message displayed in green.</param>
        /// <param name="details">Key/value pairs to display as dim detail lines.</param>
        public static void WriteSuccessWithDim(string heading, params (string Label, string Value)[] details)
        {
            AnsiConsole.MarkupLine($"[green]{Markup.Escape(heading)}[/]");
            foreach ((string label, string value) in details)
            {
                AnsiConsole.MarkupLine($"  [dim]{Markup.Escape(label)}:[/] {Markup.Escape(value)}");
            }
        }
    }
}
