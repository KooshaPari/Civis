namespace DINOForge.SDK.Validation
{
    /// <summary>
    /// Validates raw YAML content against a named schema.
    /// </summary>
    public interface ISchemaValidator
    {
        /// <summary>
        /// Validates <paramref name="yamlContent"/> against the schema identified by <paramref name="schemaName"/>.
        /// </summary>
        /// <param name="schemaName">The logical schema name (e.g. "pack-manifest").</param>
        /// <param name="yamlContent">Raw YAML text to validate.</param>
        /// <returns>A <see cref="ValidationResult"/> describing whether validation passed.</returns>
        ValidationResult Validate(string schemaName, string yamlContent);
    }
}
