using System.Collections.Generic;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;

namespace DINOForge.SDK
{
    /// <summary>
    /// Service for importing and registering content into registries.
    /// Handles YAML reading, schema validation, deserialization, and registry population.
    /// </summary>
    internal interface IRegistryImportService
    {
        /// <summary>
        /// Gets all loaded stat override definitions.
        /// </summary>
        IReadOnlyList<StatOverrideDefinition> LoadedOverrides { get; }

        /// <summary>
        /// Loads and registers content from a YAML file.
        /// </summary>
        /// <param name="yamlFilePath">Path to the YAML file.</param>
        /// <param name="contentType">Content type (e.g., "units", "buildings").</param>
        /// <param name="manifest">Pack manifest for source tracking.</param>
        /// <param name="errors">List to accumulate errors.</param>
        void LoadAndRegisterContent(
            string yamlFilePath,
            string contentType,
            PackManifest manifest,
            IList<string> errors);

        /// <summary>
        /// Stores a patched YAML string that will be used in place of reading the
        /// specified file path from disk during subsequent
        /// <see cref="LoadAndRegisterContent"/> calls.
        /// Populated by <see cref="ContentLoader"/> after the patch phase runs.
        /// </summary>
        /// <param name="filePath">Absolute file path acting as the cache key.</param>
        /// <param name="patchedYaml">Patched YAML content to use instead of the file.</param>
        void SetPatchedYaml(string filePath, string patchedYaml);
    }
}
