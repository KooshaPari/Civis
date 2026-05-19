using System;
using System.IO;
using DINOForge.SDK.IO;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.SDK.Universe
{
    /// <summary>
    /// Loads UniverseBible instances from universe pack directories on disk.
    /// Supports loading from a single universe.yaml or from a directory with
    /// separate crosswalk.yaml, factions.yaml, naming.yaml, and style.yaml files.
    /// </summary>
    public sealed class UniverseLoader
    {
        private readonly IDeserializer _deserializer;
        private readonly Action<string> _log;

        /// <summary>
        /// Initializes a new <see cref="UniverseLoader"/>.
        /// </summary>
        /// <param name="log">Optional logging callback.</param>
        public UniverseLoader(Action<string>? log = null)
        {
            _log = log ?? (_ => { });
            _deserializer = new DeserializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .IgnoreUnmatchedProperties()
                .Build();
        }

        /// <summary>
        /// Loads a UniverseBible from a universe pack directory.
        /// Expects a universe.yaml file. Optionally loads crosswalk.yaml, factions.yaml,
        /// naming.yaml, and style.yaml if present as separate files.
        /// </summary>
        /// <param name="universeDirectory">Path to the universe pack directory.</param>
        /// <returns>The loaded UniverseBible.</returns>
        /// <exception cref="FileNotFoundException">If universe.yaml is not found.</exception>
        /// <exception cref="InvalidOperationException">If YAML parsing fails.</exception>
        public UniverseBible LoadFromDirectory(string universeDirectory)
        {
            string universePath = Path.Combine(universeDirectory, "universe.yaml");
            if (!File.Exists(universePath))
            {
                throw new FileNotFoundException(
                    $"Universe manifest not found: {universePath}",
                    universePath);
            }

            _log($"[UniverseLoader] Loading universe from {universeDirectory}");

            // Load base universe manifest
            string universeYaml = SafeFileIO.ReadText(universePath);
            UniverseBible bible = _deserializer.Deserialize<UniverseBible>(universeYaml);
            // Task #319 — IValidatable semantic check at the deserialize site.
            JsonGuard.ValidateOrThrow(bible, universePath);

            // Load separate files if they exist and override the embedded data
            LoadCrosswalk(universeDirectory, bible);
            LoadFactions(universeDirectory, bible);
            LoadNaming(universeDirectory, bible);
            LoadStyle(universeDirectory, bible);

            _log($"[UniverseLoader] Loaded universe '{bible.Id}' with {bible.CrosswalkDictionary.Entries.Count} crosswalk entries");
            return bible;
        }

        /// <summary>
        /// Loads a UniverseBible from a single YAML string.
        /// </summary>
        /// <param name="yaml">YAML content containing the full UniverseBible.</param>
        /// <returns>The parsed UniverseBible.</returns>
        public UniverseBible LoadFromYaml(string yaml)
        {
            UniverseBible bible = _deserializer.Deserialize<UniverseBible>(yaml);
            // Task #319 — IValidatable semantic check at the deserialize site.
            JsonGuard.ValidateOrThrow(bible);
            return bible;
        }

        private void LoadCrosswalk(string directory, UniverseBible bible)
        {
            string path = Path.Combine(directory, "crosswalk.yaml");
            if (!File.Exists(path))
                return;

            string yaml = SafeFileIO.ReadText(path);
            CrosswalkDictionary crosswalk = _deserializer.Deserialize<CrosswalkDictionary>(yaml);
            // Task #319 — IValidatable semantic check at the deserialize site.
            JsonGuard.ValidateOrThrow(crosswalk, path);
            bible.CrosswalkDictionary = crosswalk;
            _log($"[UniverseLoader] Loaded crosswalk with {crosswalk.Entries.Count} entries");
        }

        private void LoadFactions(string directory, UniverseBible bible)
        {
            string path = Path.Combine(directory, "factions.yaml");
            if (!File.Exists(path))
                return;

            string yaml = SafeFileIO.ReadText(path);
            FactionTaxonomy taxonomy = _deserializer.Deserialize<FactionTaxonomy>(yaml);
            // Task #319 — IValidatable semantic check at the deserialize site.
            JsonGuard.ValidateOrThrow(taxonomy, path);
            bible.FactionTaxonomy = taxonomy;
            _log($"[UniverseLoader] Loaded {taxonomy.Factions.Count} factions");
        }

        private void LoadNaming(string directory, UniverseBible bible)
        {
            string path = Path.Combine(directory, "naming.yaml");
            if (!File.Exists(path))
                return;

            string yaml = SafeFileIO.ReadText(path);
            NamingGuide naming = _deserializer.Deserialize<NamingGuide>(yaml);
            // Task #319 — IValidatable semantic check at the deserialize site.
            JsonGuard.ValidateOrThrow(naming, path);
            bible.NamingGuide = naming;
            _log("[UniverseLoader] Loaded naming guide");
        }

        private void LoadStyle(string directory, UniverseBible bible)
        {
            string path = Path.Combine(directory, "style.yaml");
            if (!File.Exists(path))
                return;

            string yaml = SafeFileIO.ReadText(path);
            StyleGuide style = _deserializer.Deserialize<StyleGuide>(yaml);
            // Task #319 — IValidatable semantic check at the deserialize site.
            JsonGuard.ValidateOrThrow(style, path);
            bible.StyleGuide = style;
            _log("[UniverseLoader] Loaded style guide");
        }
    }
}
