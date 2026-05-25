#nullable enable
using System.CommandLine;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Shared output helpers for dual human-readable and machine-readable CLI commands.
/// </summary>
internal static class CommandOutput
{
    private static readonly JsonSerializerOptions JsonOptions = new(JsonSerializerDefaults.Web)
    {
        WriteIndented = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
    };

    /// <summary>
    /// Creates the standard output format option for commands that support JSON responses.
    /// </summary>
    public static Option<string> CreateFormatOption() => new("--format")
    {
        Description = "Output format: table or json",
        DefaultValueFactory = _ => "table"
    };

    /// <summary>
    /// Returns whether the caller requested JSON output.
    /// </summary>
    public static bool IsJson(ParseResult parseResult, Option<string> formatOption) =>
        string.Equals(parseResult.GetValue(formatOption), "json", StringComparison.OrdinalIgnoreCase);

    /// <summary>
    /// Writes a JSON payload to stdout.
    /// </summary>
    public static void WriteJson<T>(T value) =>
        Console.WriteLine(JsonSerializer.Serialize(value, JsonOptions));

    /// <summary>
    /// Writes a consistent JSON error payload and marks the process as failed.
    /// </summary>
    public static void WriteJsonError(string error, string message)
    {
        Environment.ExitCode = 1;
        WriteJson(new
        {
            success = false,
            error,
            message
        });
    }
}
