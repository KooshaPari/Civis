using System;
using System.Collections.Generic;
using System.IO;

namespace DINOForge.SDK
{
    /// <summary>
    /// Service for discovering files and directories with configurable exclusion patterns.
    /// Provides centralized filesystem traversal with built-in support for common exclusions
    /// like archived/, export/, generated/, bin/, obj/, and node_modules/ directories.
    /// </summary>
    public interface IFileDiscoveryService
    {
        /// <summary>
        /// Gets all files in a directory matching the specified pattern.
        /// </summary>
        /// <param name="directory">The root directory to search.</param>
        /// <param name="searchPattern">The search pattern (e.g., "*.yaml", "*.json").</param>
        /// <param name="searchOption">Whether to search subdirectories.</param>
        /// <returns>Array of matching file paths.</returns>
        string[] GetFiles(string directory, string searchPattern, SearchOption searchOption = SearchOption.TopDirectoryOnly);

        /// <summary>
        /// Gets all subdirectories in a directory.
        /// </summary>
        /// <param name="directory">The root directory to search.</param>
        /// <param name="searchOption">Whether to search subdirectories.</param>
        /// <returns>Array of subdirectory paths.</returns>
        string[] GetDirectories(string directory, SearchOption searchOption = SearchOption.TopDirectoryOnly);

        /// <summary>
        /// Gets all files in a directory with support for multiple search patterns.
        /// </summary>
        /// <param name="directory">The root directory to search.</param>
        /// <param name="searchPatterns">Array of search patterns to match.</param>
        /// <param name="searchOption">Whether to search subdirectories.</param>
        /// <returns>Array of matching file paths.</returns>
        string[] GetFiles(string directory, string[] searchPatterns, SearchOption searchOption = SearchOption.TopDirectoryOnly);

        /// <summary>
        /// Discovers pack directories (directories containing pack.yaml).
        /// </summary>
        /// <param name="rootDirectory">The root packs directory.</param>
        /// <returns>Array of pack directory paths.</returns>
        string[] DiscoverPackDirectories(string rootDirectory);

        /// <summary>
        /// Gets the default exclusion patterns used by this service.
        /// </summary>
        IReadOnlyList<string> DefaultExclusions { get; }

        /// <summary>
        /// Adds a custom exclusion pattern.
        /// </summary>
        /// <param name="pattern">The exclusion pattern (e.g., "*.tmp", "test/").</param>
        void AddExclusion(string pattern);

        /// <summary>
        /// Removes a custom exclusion pattern.
        /// </summary>
        /// <param name="pattern">The exclusion pattern to remove.</param>
        void RemoveExclusion(string pattern);

        /// <summary>
        /// Clears all exclusion patterns.
        /// </summary>
        void ClearExclusions();

        /// <summary>
        /// Resets exclusions to the default set.
        /// </summary>
        void ResetToDefaults();
    }
}
