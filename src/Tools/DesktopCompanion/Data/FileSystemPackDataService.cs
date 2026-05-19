using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using DINOForge.SDK;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.DesktopCompanion.Data
{
    /// <summary>
    /// Reads pack manifests from the filesystem using YamlDotNet.
    /// Wraps <see cref="DINOForge.SDK.PackManifest"/> for deserialization;
    /// maps to the companion's <see cref="PackViewModel"/> DTO.
    /// </summary>
    public sealed class FileSystemPackDataService : IPackDataService
    {
        private static readonly IDeserializer YamlDeserializer =
            new DeserializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .IgnoreUnmatchedProperties()
                .Build();

        /// <inheritdoc />
        public Task<LoadResultViewModel> LoadPacksAsync(string packsDir)
        {
            return Task.Run(() => ScanDirectory(packsDir));
        }

        private LoadResultViewModel ScanDirectory(string packsDir)
        {
            List<PackViewModel> packs = new List<PackViewModel>();
            List<string> errors = new List<string>();
            int errorCount = 0;

            if (!Directory.Exists(packsDir))
            {
                return new LoadResultViewModel
                {
                    LoadedCount = 0,
                    ErrorCount = 1,
                    Errors = new[] { $"Packs directory not found: {packsDir}" },
                    Packs = Array.Empty<PackViewModel>()
                };
            }

            foreach (string dir in Directory.GetDirectories(packsDir))
            {
                // Skip _archived and hidden directories
                string dirName = Path.GetFileName(dir);
                if (dirName.StartsWith("_") || dirName.StartsWith("."))
                    continue;

                string manifestPath = Path.Combine(dir, "pack.yaml");
                if (!File.Exists(manifestPath))
                    continue;

                try
                {
                    string yaml = File.ReadAllText(manifestPath, Encoding.UTF8);
                    PackManifest manifest = YamlDeserializer.Deserialize<PackManifest>(yaml);

                    packs.Add(new PackViewModel
                    {
                        Id = manifest.Id,
                        Name = string.IsNullOrEmpty(manifest.Name) ? manifest.Id : manifest.Name,
                        Version = manifest.Version,
                        Author = manifest.Author,
                        Type = manifest.Type,
                        Description = manifest.Description,
                        DependsOn = manifest.DependsOn.AsReadOnly(),
                        ConflictsWith = manifest.ConflictsWith.AsReadOnly(),
                        LoadOrder = manifest.LoadOrder,
                        PackDirectory = dir,
                        Enabled = true,
                        ErrorCount = 0,
                        Errors = Array.Empty<string>()
                    });
                }
                catch (Exception ex)
                {
                    errorCount++;
                    string message = $"Failed to parse {manifestPath}: {ex.Message}";
                    errors.Add(message);

                    // Still add a placeholder so the UI shows it as errored
                    packs.Add(new PackViewModel
                    {
                        Id = dirName,
                        Name = dirName,
                        Version = "?",
                        Author = "?",
                        Type = "unknown",
                        PackDirectory = dir,
                        Enabled = false,
                        ErrorCount = 1,
                        Errors = new[] { message }
                    });
                }
            }

            return new LoadResultViewModel
            {
                LoadedCount = packs.Count - errorCount,
                ErrorCount = errorCount,
                Errors = errors.AsReadOnly(),
                Packs = packs.AsReadOnly()
            };
        }
    }
}
