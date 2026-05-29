using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;
using DINOForge.SDK.Validation;
using NJsonSchema;
using Newtonsoft.Json.Linq;
using Spectre.Console;

namespace DINOForge.Tools.PackCompiler.Services
{
    /// <summary>
    /// Enhances validation error reporting with schema-aware suggestions and helpful context.
    /// Converts generic validation errors into actionable guidance for pack developers.
    /// </summary>
    public class EnhancedValidationService
    {
        private readonly JsonSchema _schema;
        private readonly string _manifestYaml;
        private readonly string _packPath;

        public EnhancedValidationService(JsonSchema schema, string manifestYaml, string packPath)
        {
            _schema = schema ?? throw new ArgumentNullException(nameof(schema));
            _manifestYaml = manifestYaml ?? throw new ArgumentNullException(nameof(manifestYaml));
            _packPath = packPath ?? throw new ArgumentNullException(nameof(packPath));
        }

        /// <summary>
        /// Analyzes a validation error and returns enriched feedback with suggestions.
        /// </summary>
        public EnrichedValidationError EnrichError(ValidationError error)
        {
            var suggestions = new List<string>();
            var (lineNumber, column) = ExtractLineAndColumn(error.Path);

            // Parse path into field breadcrumbs
            var fieldPath = ParseFieldPath(error.Path);

            // Generate context-specific suggestions
            GenerateSuggestions(error, fieldPath, suggestions);

            return new EnrichedValidationError
            {
                OriginalError = error,
                Path = error.Path,
                LineNumber = lineNumber,
                Column = column,
                FieldPath = fieldPath,
                Suggestions = suggestions.AsReadOnly(),
                ContextualMessage = BuildContextualMessage(error, fieldPath)
            };
        }

        /// <summary>
        /// Generates repair suggestions for common validation failures.
        /// </summary>
        private void GenerateSuggestions(ValidationError error, List<string> fieldPath, List<string> suggestions)
        {
            string rule = error.Rule.ToLowerInvariant();
            string message = error.Message.ToLowerInvariant();

            // Missing required field
            if (rule.Contains("required") || message.Contains("is required"))
            {
                if (fieldPath.Count > 0)
                {
                    string field = fieldPath.Last();
                    suggestions.Add($"Add required field '{field}' to pack.yaml");

                    // Field-specific defaults
                    if (field == "framework_version")
                        suggestions.Add("Suggested value: \">=0.5.0 <1.0.0\"");
                    else if (field == "id")
                        suggestions.Add($"Suggested value: \"{DeriveIdFromPath(_packPath)}\"");
                }
            }

            // Enum mismatch
            if (rule.Contains("enum") || message.Contains("enum"))
            {
                if (fieldPath.Contains("type"))
                {
                    suggestions.Add("Valid types: content, balance, ruleset, scenario, total_conversion, utility");
                    suggestions.Add("Example: type: content");
                }
                else if (message.Contains("one of"))
                {
                    suggestions.Add("Check the field value against the allowed options in the error message");
                }
            }

            // Pattern validation failure
            if (rule.Contains("pattern"))
            {
                if (fieldPath.Contains("id"))
                {
                    suggestions.Add("Pack ID must be lowercase with hyphens/underscores only (e.g., 'my-cool-pack')");
                    suggestions.Add($"Suggested value: \"{DeriveIdFromPath(_packPath)}\"");
                }
                else if (fieldPath.Contains("version"))
                {
                    suggestions.Add("Version must follow semantic versioning: MAJOR.MINOR.PATCH (e.g., 1.2.3)");
                }
            }

            // Type mismatch
            if (rule.Contains("type"))
            {
                if (message.Contains("string") && message.Contains("array"))
                {
                    suggestions.Add("Use YAML array syntax: [item1, item2, item3]");
                }
                else if (message.Contains("object") && message.Contains("string"))
                {
                    suggestions.Add("Check YAML indentation — field may need child properties");
                }
            }

            // Min/max length
            if (rule.Contains("minlength"))
            {
                suggestions.Add("Field value is too short — provide a longer string");
            }

            // Additional properties
            if (rule.Contains("additional") || message.Contains("additional property"))
            {
                // Try to extract the unknown property
                var match = Regex.Match(message, @"'(\w+)'");
                if (match.Success)
                {
                    string unknownProp = match.Groups[1].Value;
                    suggestions.Add($"Unknown property '{unknownProp}' — did you mean one of the valid fields?");
                    suggestions.Add("Run 'dotnet run --project src/Tools/PackCompiler -- validate <pack>' to see valid fields");
                }
            }
        }

        /// <summary>
        /// Builds a contextual error message that explains what went wrong in plain English.
        /// </summary>
        private string BuildContextualMessage(ValidationError error, List<string> fieldPath)
        {
            var msg = new StringBuilder();
            string rule = error.Rule.ToLowerInvariant();

            if (fieldPath.Count > 0)
                msg.Append($"Field '[{string.Join(".", fieldPath)}]' ");
            else
                msg.Append("Value ");

            if (rule.Contains("required"))
                msg.Append("is required but was not provided");
            else if (rule.Contains("enum"))
                msg.Append("has an invalid value — it must be one of the allowed options");
            else if (rule.Contains("pattern"))
                msg.Append("does not match the required format");
            else if (rule.Contains("type"))
                msg.Append("has the wrong type");
            else if (rule.Contains("minlength"))
                msg.Append("is too short");
            else if (rule.Contains("maxlength"))
                msg.Append("is too long");
            else if (rule.Contains("additional"))
                msg.Append("is not a valid property");
            else
                msg.Append(error.Message);

            return msg.ToString();
        }

        /// <summary>
        /// Extracts line and column numbers from a YAML path string.
        /// </summary>
        private (int line, int column) ExtractLineAndColumn(string path)
        {
            // Try to find line number in the path (e.g., "line:12,col:5")
            var lineMatch = Regex.Match(path, @"line:(\d+)", RegexOptions.IgnoreCase);
            var colMatch = Regex.Match(path, @"col:(\d+)", RegexOptions.IgnoreCase);

            int line = lineMatch.Success ? int.Parse(lineMatch.Groups[1].Value) : -1;
            int col = colMatch.Success ? int.Parse(colMatch.Groups[1].Value) : -1;

            // If not in path, try to find it by scanning the YAML
            if (line < 0)
            {
                line = FindLineNumberInYaml(path);
            }

            return (line, col);
        }

        /// <summary>
        /// Scans YAML content to find the line number of a given field path.
        /// </summary>
        private int FindLineNumberInYaml(string fieldPath)
        {
            if (string.IsNullOrEmpty(fieldPath))
                return 1;

            var lines = _manifestYaml.Split(new[] { "\r\n", "\r", "\n" }, StringSplitOptions.None);
            var key = fieldPath.Split(new[] { '.', '[', ']' }, StringSplitOptions.RemoveEmptyEntries)
                .FirstOrDefault()?
                .Trim();

            if (string.IsNullOrEmpty(key))
                return 1;

            for (int i = 0; i < lines.Length; i++)
            {
                if (lines[i].Contains(key + ":") || lines[i].Contains(key))
                    return i + 1;
            }

            return 1;
        }

        /// <summary>
        /// Parses a dot-separated field path into component parts.
        /// </summary>
        private List<string> ParseFieldPath(string path)
        {
            if (string.IsNullOrEmpty(path))
                return new List<string>();

            // Remove array indices and split on dots
            var parts = Regex.Split(path, @"[\.\[\]]+")
                .Where(p => !string.IsNullOrEmpty(p) && !int.TryParse(p, out _))
                .ToList();

            return parts;
        }

        /// <summary>
        /// Derives a valid pack ID from the directory name.
        /// </summary>
        private string DeriveIdFromPath(string packPath)
        {
            string dirName = new DirectoryInfo(packPath).Name;
            // Convert to lowercase, replace spaces/underscores with hyphens
            string id = Regex.Replace(dirName, @"[^\w-]", "").ToLowerInvariant();
            // Ensure it starts with a letter/digit
            id = Regex.Replace(id, @"^[^a-z0-9]+", "");
            return string.IsNullOrEmpty(id) ? "new-pack" : id;
        }
    }

    /// <summary>
    /// Enriched validation error with additional context and suggestions.
    /// </summary>
    public class EnrichedValidationError
    {
        public ValidationError OriginalError { get; set; } = null!;
        public string Path { get; set; } = "";
        public int LineNumber { get; set; }
        public int Column { get; set; }
        public List<string> FieldPath { get; set; } = new();
        public IReadOnlyList<string> Suggestions { get; set; } = new List<string>().AsReadOnly();
        public string ContextualMessage { get; set; } = "";
    }

    /// <summary>
    /// Validation report summarizing multiple pack validations.
    /// </summary>
    public class ValidationReport
    {
        public int TotalPacks { get; set; }
        public int SuccessfulPacks { get; set; }
        public int WarningPacks { get; set; }
        public int FailedPacks { get; set; }
        public List<string> Warnings { get; set; } = new();
        public List<string> Failures { get; set; } = new();

        public override string ToString()
        {
            var sb = new StringBuilder();
            sb.AppendLine($"[bold]Validation Report[/]");
            sb.AppendLine($"  [green]✓ Passed:[/] {SuccessfulPacks}");
            if (WarningPacks > 0)
                sb.AppendLine($"  [yellow]⚠ Warnings:[/] {WarningPacks}");
            if (FailedPacks > 0)
                sb.AppendLine($"  [red]✗ Failed:[/] {FailedPacks}");

            return sb.ToString();
        }
    }

    /// <summary>
    /// Auto-repair suggestions for common validation issues.
    /// </summary>
    public class AutoRepairSuggestion
    {
        public string Field { get; set; } = "";
        public string CurrentValue { get; set; } = "";
        public string SuggestedValue { get; set; } = "";
        public string Reason { get; set; } = "";
        public bool IsAutoFixable { get; set; }
    }
}
