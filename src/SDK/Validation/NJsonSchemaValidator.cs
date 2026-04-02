using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using NJsonSchema;
using Newtonsoft.Json.Linq;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Validation
{
    /// <summary>
    /// Implements <see cref="ISchemaValidator"/> using the NJsonSchema library.
    /// Validates YAML content against YAML-defined JSON schemas by converting both
    /// to JSON and leveraging NJsonSchema's validation capabilities.
    ///
    /// This is a thin wrapper around NJsonSchema per ADR-008 (wrap, don't handroll).
    /// </summary>
    public class NJsonSchemaValidator : ISchemaValidator
    {
        private readonly Dictionary<string, string> _schemaSources;
        private readonly Dictionary<string, JsonSchema> _cachedSchemas;

        /// <summary>
        /// Initializes a new instance of <see cref="NJsonSchemaValidator"/>.
        /// </summary>
        /// <param name="schemaSources">
        /// Dictionary mapping schema names (e.g., "pack-manifest") to their YAML schema content.
        /// </param>
        public NJsonSchemaValidator(Dictionary<string, string> schemaSources)
        {
            _schemaSources = schemaSources ?? throw new ArgumentNullException(nameof(schemaSources));
            _cachedSchemas = new Dictionary<string, JsonSchema>();
        }

        /// <summary>
        /// Validates YAML content against a named schema.
        /// </summary>
        /// <param name="schemaName">The logical schema name (e.g., "pack-manifest").</param>
        /// <param name="yamlContent">Raw YAML text to validate.</param>
        /// <returns>A validation result describing whether validation passed and any errors.</returns>
        public ValidationResult Validate(string schemaName, string yamlContent)
        {
            if (string.IsNullOrWhiteSpace(schemaName))
                throw new ArgumentException("Schema name cannot be null or empty.", nameof(schemaName));
            if (string.IsNullOrWhiteSpace(yamlContent))
                throw new ArgumentException("YAML content cannot be null or empty.", nameof(yamlContent));

            // Load and cache the schema
            if (!_cachedSchemas.TryGetValue(schemaName, out JsonSchema? schema))
            {
                if (!_schemaSources.TryGetValue(schemaName, out string? schemaYaml))
                    throw new InvalidOperationException($"Schema '{schemaName}' not found.");

                schema = LoadSchema(schemaYaml);
                _cachedSchemas[schemaName] = schema;
            }

            // Convert YAML content to JSON
            string jsonContent;
            try
            {
                jsonContent = YamlSchemaConverter.ConvertYamlToJson(yamlContent);
            }
            catch (Exception ex)
            {
                // Return validation failure for malformed YAML instead of throwing
                List<ValidationError> yamlErrors = new List<ValidationError>
                {
                    new ValidationError(
                        path: "",
                        message: $"Invalid YAML: {ex.Message}",
                        rule: "yaml-parse-error")
                };
                return ValidationResult.Failure(yamlErrors.AsReadOnly());
            }

            // Parse JSON for validation
            JToken jToken = JToken.Parse(jsonContent);

            // Perform validation
            ICollection<NJsonSchema.Validation.ValidationError> errors = schema.Validate(jToken);

            if (errors.Count == 0)
                return ValidationResult.Success();

            List<ValidationError> validationErrors = errors
                .Select(e => new ValidationError(
                    path: e.Path ?? "",
                    message: e.ToString(),
                    rule: GetRuleKind(e)))
                .ToList();

            return ValidationResult.Failure(validationErrors.AsReadOnly());
        }

        /// <summary>
        /// Extracts the rule kind from a validation error.
        /// </summary>
        private static string GetRuleKind(object validationError)
        {
            // Get the 'Kind' property from the validation error
            System.Reflection.PropertyInfo? kindProperty = validationError.GetType().GetProperty("Kind");
            if (kindProperty != null)
            {
                object? kindValue = kindProperty.GetValue(validationError);
                return kindValue?.ToString() ?? "unknown";
            }
            return "unknown";
        }

        /// <summary>
        /// Loads a JSON schema from YAML schema content.
        /// </summary>
        private static JsonSchema LoadSchema(string yamlSchemaContent)
        {
            string jsonSchemaContent = YamlSchemaConverter.ConvertYamlToJson(yamlSchemaContent);
            JsonSchema schema = JsonSchema.FromJsonAsync(jsonSchemaContent).GetAwaiter().GetResult();
            return schema;
        }
    }
}
