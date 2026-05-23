using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace DINOForge.SDK
{
    /// <summary>
    /// Default implementation of <see cref="IFileDiscoveryService"/> that provides
    /// centralized filesystem traversal with configurable exclusion patterns.
    /// </summary>
    public class FileDiscoveryService : IFileDiscoveryService
    {
        /// <summary>
        /// Default exclusion patterns for build artifacts and generated content.
        /// </summary>
        private static readonly string[] DefaultExclusionsArray = new[]
        {
            "archived",
            "_archived",
            "_temp",
            "_scratch",
            "export",
            "generated",
            "bin",
            "obj",
            "node_modules",
            ".git",
            ".vs",
            ".idea",
            "packages",
            "node_modules"
        };

        private readonly HashSet<string> _exclusions;
        private readonly bool _useDefaults;

        /// <summary>
        /// Initializes a new <see cref="FileDiscoveryService"/> with default exclusions.
        /// </summary>
        public FileDiscoveryService() : this(useDefaults: true)
        {
        }

        /// <summary>
        /// Initializes a new <see cref="FileDiscoveryService"/> with optional default exclusions.
        /// </summary>
        /// <param name="useDefaults">Whether to include default exclusions.</param>
        public FileDiscoveryService(bool useDefaults)
        {
            _useDefaults = useDefaults;
            _exclusions = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            if (useDefaults)
            {
                foreach (var exclusion in DefaultExclusionsArray)
                {
                    _exclusions.Add(exclusion);
                }
            }
        }

        /// <summary>
        /// Initializes a new <see cref="FileDiscoveryService"/> with custom exclusions.
        /// </summary>
        /// <param name="customExclusions">Custom exclusion patterns.</param>
        public FileDiscoveryService(IEnumerable<string> customExclusions)
        {
            _useDefaults = false;
            _exclusions = new HashSet<string>(customExclusions, StringComparer.OrdinalIgnoreCase);
        }

        /// <inheritdoc />
        public IReadOnlyList<string> DefaultExclusions => Array.AsReadOnly(DefaultExclusionsArray);

        /// <inheritdoc />
        public string[] GetFiles(string directory, string searchPattern, SearchOption searchOption = SearchOption.TopDirectoryOnly)
        {
            return GetFiles(directory, new[] { searchPattern }, searchOption);
        }

        /// <inheritdoc />
        public string[] GetFiles(string directory, string[] searchPatterns, SearchOption searchOption = SearchOption.TopDirectoryOnly)
        {
            if (string.IsNullOrEmpty(directory) || !Directory.Exists(directory))
            {
                return Array.Empty<string>();
            }

            if (searchPatterns == null || searchPatterns.Length == 0)
            {
                return Array.Empty<string>();
            }

            var results = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

            foreach (var pattern in searchPatterns)
            {
                var files = GetFilesInternal(directory, pattern, searchOption);
                foreach (var file in files)
                {
                    results.Add(file);
                }
            }

            var sortedResults = results.ToList();
            sortedResults.Sort(StringComparer.OrdinalIgnoreCase);
            return sortedResults.ToArray();
        }

        /// <inheritdoc />
        public string[] GetDirectories(string directory, SearchOption searchOption = SearchOption.TopDirectoryOnly)
        {
            if (string.IsNullOrEmpty(directory) || !Directory.Exists(directory))
            {
                return Array.Empty<string>();
            }

            var results = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

            if (searchOption == SearchOption.TopDirectoryOnly)
            {
                foreach (var dir in Directory.GetDirectories(directory))
                {
                    string dirName = Path.GetFileName(dir);
                    if (!ShouldExclude(dirName))
                    {
                        results.Add(dir);
                    }
                }
            }
            else
            {
                // Recursive search
                GetDirectoriesRecursive(directory, results);
            }

            var sortedResults = results.ToList();
            sortedResults.Sort(StringComparer.OrdinalIgnoreCase);
            return sortedResults.ToArray();
        }

        /// <inheritdoc />
        public string[] DiscoverPackDirectories(string rootDirectory)
        {
            if (string.IsNullOrEmpty(rootDirectory) || !Directory.Exists(rootDirectory))
            {
                return Array.Empty<string>();
            }

            var packDirectories = new List<string>();
            var allDirs = GetDirectories(rootDirectory, SearchOption.TopDirectoryOnly);

            foreach (var dir in allDirs)
            {
                string manifestPath = Path.Combine(dir, "pack.yaml");
                if (File.Exists(manifestPath))
                {
                    packDirectories.Add(dir);
                }
            }

            return packDirectories.ToArray();
        }

        /// <inheritdoc />
        public void AddExclusion(string pattern)
        {
            if (!string.IsNullOrWhiteSpace(pattern))
            {
                _exclusions.Add(pattern.Trim());
            }
        }

        /// <inheritdoc />
        public void RemoveExclusion(string pattern)
        {
            if (!string.IsNullOrWhiteSpace(pattern))
            {
                _exclusions.Remove(pattern.Trim());
            }
        }

        /// <inheritdoc />
        public void ClearExclusions()
        {
            _exclusions.Clear();
        }

        /// <inheritdoc />
        public void ResetToDefaults()
        {
            _exclusions.Clear();
            foreach (var exclusion in DefaultExclusionsArray)
            {
                _exclusions.Add(exclusion);
            }
        }

        private string[] GetFilesInternal(string directory, string searchPattern, SearchOption searchOption)
        {
            if (searchOption == SearchOption.TopDirectoryOnly)
            {
                return Directory.GetFiles(directory, searchPattern)
                    .Where(f => !ShouldExclude(Path.GetDirectoryName(f) ?? ""))
                    .ToArray();
            }

            // For recursive search, we need to filter out excluded directories
            var results = new List<string>();
            GetFilesRecursive(directory, searchPattern, results);
            return results.ToArray();
        }

        private void GetFilesRecursive(string directory, string searchPattern, List<string> results)
        {
            try
            {
                // Get files in current directory
                foreach (var file in Directory.GetFiles(directory, searchPattern))
                {
                    results.Add(file);
                }

                // Recurse into subdirectories (excluding excluded ones)
                foreach (var subDir in Directory.GetDirectories(directory))
                {
                    string dirName = Path.GetFileName(subDir);
                    if (!ShouldExclude(dirName))
                    {
                        GetFilesRecursive(subDir, searchPattern, results);
                    }
                }
            }
            catch (UnauthorizedAccessException)
            {
                // safe-swallow: directory may be inaccessible during traversal; skip it
            }
            catch (DirectoryNotFoundException)
            {
                // safe-swallow: directory may be deleted during traversal; skip it
            }
        }

        private void GetDirectoriesRecursive(string directory, HashSet<string> results)
        {
            try
            {
                foreach (var dir in Directory.GetDirectories(directory))
                {
                    string dirName = Path.GetFileName(dir);
                    if (!ShouldExclude(dirName))
                    {
                        results.Add(dir);
                        GetDirectoriesRecursive(dir, results);
                    }
                }
            }
            catch (UnauthorizedAccessException)
            {
                // safe-swallow: directory may be inaccessible during traversal; skip it
            }
            catch (DirectoryNotFoundException)
            {
                // safe-swallow: directory may be deleted during traversal; skip it
            }
        }

        private bool ShouldExclude(string path)
        {
            // Check each path segment for exclusions
            var segments = path.Split(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
            foreach (var segment in segments)
            {
                if (string.IsNullOrEmpty(segment))
                {
                    continue;
                }

                if (_exclusions.Contains(segment))
                {
                    return true;
                }

                // Convention: any directory segment starting with "_" is treated as
                // an internal/archived/scratch area and is excluded from discovery.
                // This catches _archived, _temp, _scratch, _wip, etc. uniformly
                // (fix #852: packs/_archived/ was being silently re-discovered
                // because only the literal "archived" was excluded).
                if (_useDefaults && segment[0] == '_')
                {
                    return true;
                }
            }
            return false;
        }
    }
}
