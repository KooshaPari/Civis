using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using DINOForge.SDK.IO;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.SDK
{
    /// <summary>
    /// Loads and validates pack manifest files (pack.yaml).
    /// Wraps YamlDotNet for deserialization (ADR-008).
    /// </summary>
    public sealed class PackLoader
    {
        private readonly IDeserializer _deserializer;

        /// <summary>
        /// Initializes a new <see cref="PackLoader"/> with a configured YAML deserializer.
        /// </summary>
        public PackLoader()
        {
            _deserializer = new DeserializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .IgnoreUnmatchedProperties()
                .Build();
        }

        /// <summary>
        /// Load a pack manifest from a YAML file.
        /// </summary>
        public PackManifest LoadFromFile(string filePath)
        {
            if (!File.Exists(filePath))
                throw new FileNotFoundException($"Pack manifest not found: {filePath}");

            string yaml = SafeFileIO.ReadText(filePath);
            return LoadFromString(yaml);
        }

        /// <summary>
        /// Load a pack manifest from a YAML string.
        /// </summary>
        public PackManifest LoadFromString(string yaml)
        {
            PackManifest manifest = _deserializer.Deserialize<PackManifest>(yaml);

            if (string.IsNullOrWhiteSpace(manifest.Id))
                throw new InvalidOperationException("Pack manifest missing required field: id");

            if (string.IsNullOrWhiteSpace(manifest.Name))
                throw new InvalidOperationException("Pack manifest missing required field: name");

            if (string.IsNullOrWhiteSpace(manifest.Version))
                throw new InvalidOperationException("Pack manifest missing required field: version");

            return manifest;
        }

        /// <summary>
        /// Load all packs from a directory (scans subdirectories for pack.yaml files).
        /// </summary>
        /// <param name="directory">Root directory to scan for packs.</param>
        /// <returns>List of loaded pack manifests.</returns>
        public List<PackManifest> LoadPacksFromDirectory(string directory)
        {
            if (!Directory.Exists(directory))
                return new List<PackManifest>();

            var packs = new List<PackManifest>();
            foreach (string subDir in Directory.GetDirectories(directory))
            {
                string packYaml = Path.Combine(subDir, "pack.yaml");
                if (File.Exists(packYaml))
                {
                    try
                    {
                        packs.Add(LoadFromFile(packYaml));
                    }
                    catch (Exception ex)
                    {
                        System.Diagnostics.Debug.WriteLine($"[PackLoader] Skipping invalid pack manifest at {packYaml}: {ex.Message}");
                    }
                }
            }
            return packs;
        }

        /// <summary>
        /// Load all packs from a directory asynchronously.
        /// </summary>
        public Task<List<PackManifest>> LoadPacksFromDirectoryAsync(string directory)
        {
            return Task.Run(() => LoadPacksFromDirectory(directory));
        }
    }
}
