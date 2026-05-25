using System;
using System.Collections.Generic;
using System.IO;

namespace DINOForge.SDK
{
    /// <summary>
    /// Implementation of <see cref="IContentDiscoveryService"/> for filesystem traversal.
    /// </summary>
    public sealed class ContentDiscoveryService : IContentDiscoveryService
    {
        private readonly IFileDiscoveryService _fileDiscovery;

        /// <summary>
        /// Creates a new ContentDiscoveryService with default FileDiscoveryService.
        /// </summary>
        public ContentDiscoveryService() : this(new FileDiscoveryService())
        {
        }

        /// <summary>
        /// Creates a new ContentDiscoveryService with the specified FileDiscoveryService.
        /// </summary>
        /// <param name="fileDiscovery">The file discovery service to use.</param>
        public ContentDiscoveryService(IFileDiscoveryService fileDiscovery)
        {
            _fileDiscovery = fileDiscovery ?? throw new ArgumentNullException(nameof(fileDiscovery));
        }

        /// <inheritdoc />
        public IReadOnlyList<string> DiscoverPackDirectories(string packsRootDirectory)
        {
            return _fileDiscovery.DiscoverPackDirectories(packsRootDirectory);
        }

        /// <inheritdoc />
        public IReadOnlyList<string> DiscoverYamlFiles(
            string packDirectory,
            string contentType,
            IReadOnlyList<string>? declaredPaths)
        {
            List<string> yamlFiles = new List<string>();

            if (declaredPaths != null && declaredPaths.Count > 0)
            {
                foreach (string path in declaredPaths)
                {
                    string fullPath = Path.Combine(packDirectory, path);
                    if (Directory.Exists(fullPath))
                    {
                        AddYamlFiles(yamlFiles, fullPath);
                    }
                    else if (File.Exists(fullPath))
                    {
                        yamlFiles.Add(fullPath);
                    }
                    else
                    {
                        string withExtension = fullPath.EndsWith(".yaml", StringComparison.OrdinalIgnoreCase) ||
                                               fullPath.EndsWith(".yml", StringComparison.OrdinalIgnoreCase)
                            ? fullPath
                            : fullPath + ".yaml";

                        if (File.Exists(withExtension))
                        {
                            yamlFiles.Add(withExtension);
                        }
                    }
                }

                return yamlFiles;
            }

            string conventionalDirectory = Path.Combine(packDirectory, contentType);
            if (Directory.Exists(conventionalDirectory))
            {
                AddYamlFiles(yamlFiles, conventionalDirectory);
            }

            return yamlFiles;
        }

        private void AddYamlFiles(List<string> yamlFiles, string directory)
        {
            yamlFiles.AddRange(_fileDiscovery.GetFiles(directory, new[] { "*.yaml", "*.yml" }, SearchOption.TopDirectoryOnly));
        }
    }
}
