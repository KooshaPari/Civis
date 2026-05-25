using System.Collections.Generic;

namespace DINOForge.SDK
{
    /// <summary>
    /// Service for discovering content files on the filesystem.
    /// Handles pack directory discovery and YAML file enumeration.
    /// </summary>
    internal interface IContentDiscoveryService
    {
        /// <summary>
        /// Discovers all pack directories in a root directory.
        /// </summary>
        /// <param name="packsRootDirectory">Root directory containing pack subdirectories.</param>
        /// <returns>List of pack directory paths that contain a pack.yaml manifest.</returns>
        IReadOnlyList<string> DiscoverPackDirectories(string packsRootDirectory);

        /// <summary>
        /// Discovers YAML files for a specific content type in a pack directory.
        /// </summary>
        /// <param name="packDirectory">Path to the pack directory.</param>
        /// <param name="contentType">Content type (e.g., "units", "buildings").</param>
        /// <param name="declaredPaths">Optional explicit paths declared in manifest; null to use conventional subdirectory.</param>
        /// <returns>List of YAML file paths.</returns>
        IReadOnlyList<string> DiscoverYamlFiles(
            string packDirectory,
            string contentType,
            IReadOnlyList<string>? declaredPaths);
    }
}
