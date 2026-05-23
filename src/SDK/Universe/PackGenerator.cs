using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.SDK.Universe
{
    /// <summary>
    /// Generates a complete mod pack from a UniverseBible and faction selection.
    /// Produces pack.yaml manifest, unit definitions, faction definitions, and weapon definitions
    /// by applying crosswalk mappings to vanilla DINO entities.
    /// </summary>
    public sealed class PackGenerator
    {
        private readonly ISerializer _serializer;
        private readonly Action<string> _log;

        /// <summary>
        /// Initializes a new <see cref="PackGenerator"/>.
        /// </summary>
        /// <param name="log">Optional logging callback.</param>
        public PackGenerator(Action<string>? log = null)
        {
            _log = log ?? (_ => { });
            _serializer = new SerializerBuilder()
                .WithNamingConvention(UnderscoredNamingConvention.Instance)
                .DisableAliases()
                .Build();
        }

        /// <summary>
        /// Generates a complete mod pack directory from a UniverseBible.
        /// </summary>
        /// <param name="bible">The universe bible to generate from.</param>
        /// <param name="factionIds">Optional filter: only generate content for these faction IDs.
        /// If null or empty, generates for all factions.</param>
        /// <param name="outputDirectory">Directory where the pack will be written.</param>
        /// <returns>Result containing the generated file paths.</returns>
        public PackGeneratorResult Generate(UniverseBible bible, IReadOnlyList<string>? factionIds, string outputDirectory)
        {
            List<string> generatedFiles = new List<string>();
            List<string> warnings = new List<string>();

            Directory.CreateDirectory(outputDirectory);

            // Determine which factions to generate
            List<TaxonomyFaction> factions = factionIds != null && factionIds.Count > 0
                ? bible.FactionTaxonomy.Factions
                    .Where(f => factionIds.Contains(f.Id, StringComparer.OrdinalIgnoreCase))
                    .ToList()
                : bible.FactionTaxonomy.Factions;

            if (factions.Count == 0)
            {
                warnings.Add("No factions matched the selection criteria.");
            }

            // 1. Generate pack.yaml
            string manifestPath = GenerateManifest(bible, factions, outputDirectory);
            generatedFiles.Add(manifestPath);

            // 2. Generate faction definitions
            string factionsDir = Path.Combine(outputDirectory, "factions");
            Directory.CreateDirectory(factionsDir);
            foreach (TaxonomyFaction faction in factions)
            {
                string factionPath = GenerateFactionDefinition(bible, faction, factionsDir);
                generatedFiles.Add(factionPath);
            }

            // 3. Generate unit definitions from crosswalk
            string unitsDir = Path.Combine(outputDirectory, "units");
            Directory.CreateDirectory(unitsDir);
            foreach (KeyValuePair<string, CrosswalkEntry> entry in bible.CrosswalkDictionary.Entries)
            {
                string unitPath = GenerateUnitDefinition(bible, entry.Value, factions, unitsDir);
                if (unitPath != null)
                    generatedFiles.Add(unitPath);
            }

            _log($"[PackGenerator] Generated {generatedFiles.Count} files for universe '{bible.Id}'");

            return new PackGeneratorResult(generatedFiles, warnings);
        }

        /// <summary>
        /// Generates the pack.yaml manifest.
        /// </summary>
        internal string GenerateManifest(UniverseBible bible, List<TaxonomyFaction> factions, string outputDir)
        {
            PackManifest manifest = new PackManifest
            {
                Id = bible.Id,
                Name = bible.Name,
                Version = bible.Version,
                Author = bible.Author ?? "DINOForge Generator",
                Type = "total_conversion",
                Description = bible.Description,
                Loads = new PackLoads
                {
                    Factions = factions.Select(f => $"factions/{f.Id}.yaml").ToList(),
                    Units = new List<string> { "units/" }
                }
            };

            string path = Path.Combine(outputDir, "pack.yaml");
            string yaml = _serializer.Serialize(manifest);
            File.WriteAllBytes(path, System.Text.Encoding.UTF8.GetBytes(yaml));
            _log($"[PackGenerator] Generated manifest: {path}");
            return path;
        }

        /// <summary>
        /// Generates a faction YAML definition from taxonomy data.
        /// </summary>
        internal string GenerateFactionDefinition(UniverseBible bible, TaxonomyFaction faction, string factionsDir)
        {
            // Look up style for this faction
            FactionStyle? style = null;
            bible.StyleGuide.FactionStyles.TryGetValue(faction.Id, out style);

            Dictionary<string, object> factionDef = new Dictionary<string, object>(StringComparer.Ordinal)
            {
                ["faction"] = new Dictionary<string, object?>(StringComparer.Ordinal)
                {
                    ["id"] = faction.Id,
                    ["display_name"] = faction.Name,
                    ["description"] = faction.Description,
                    ["theme"] = bible.Id,
                    ["archetype"] = faction.Archetype
                }
            };

            if (style != null)
            {
                factionDef["visuals"] = new Dictionary<string, object?>(StringComparer.Ordinal)
                {
                    ["primary_color"] = style.Colors.Primary,
                    ["accent_color"] = style.Colors.Secondary
                };
            }

            if (faction.UnitRoster != null)
            {
                factionDef["roster"] = faction.UnitRoster;
            }

            string path = Path.Combine(factionsDir, $"{faction.Id}.yaml");
            string yaml = _serializer.Serialize(factionDef);
            File.WriteAllBytes(path, System.Text.Encoding.UTF8.GetBytes(yaml));
            return path;
        }

        /// <summary>
        /// Generates a unit YAML definition from a crosswalk entry.
        /// </summary>
        internal string GenerateUnitDefinition(UniverseBible bible, CrosswalkEntry entry, List<TaxonomyFaction> factions, string unitsDir)
        {
            // Determine faction for this unit (first faction by default)
            string factionId = factions.Count > 0 ? factions[0].Id : "unknown";

            // Apply naming guide
            string displayName = entry.ThemedName
                ?? bible.NamingGuide.ApplyNaming(factionId, "unit", entry.ThemedId);

            Dictionary<string, object?> unitDef = new Dictionary<string, object?>(StringComparer.Ordinal)
            {
                ["id"] = entry.ThemedId,
                ["display_name"] = displayName,
                ["description"] = entry.ThemedDescription,
                ["faction_id"] = factionId,
                ["vanilla_mapping"] = entry.VanillaId
            };

            if (entry.StatModifiers != null && entry.StatModifiers.Count > 0)
            {
                unitDef["stats"] = entry.StatModifiers;
            }

            if (entry.SpriteOverride != null)
            {
                unitDef["visuals"] = new Dictionary<string, string?>(StringComparer.Ordinal)
                {
                    ["icon"] = entry.SpriteOverride
                };
            }

            string safeFileName = entry.ThemedId.Replace("*", "_").Replace("/", "_");
            string path = Path.Combine(unitsDir, $"{safeFileName}.yaml");
            string yaml = _serializer.Serialize(unitDef);
            File.WriteAllBytes(path, System.Text.Encoding.UTF8.GetBytes(yaml));
            return path;
        }
    }

    /// <summary>
    /// Result of a pack generation operation.
    /// </summary>
    public sealed class PackGeneratorResult
    {
        /// <summary>
        /// List of all generated file paths.
        /// </summary>
        public IReadOnlyList<string> GeneratedFiles { get; }

        /// <summary>
        /// Warnings encountered during generation.
        /// </summary>
        public IReadOnlyList<string> Warnings { get; }

        /// <summary>
        /// Whether generation completed without warnings.
        /// </summary>
        public bool IsClean => Warnings.Count == 0;

        /// <summary>
        /// Initializes a new <see cref="PackGeneratorResult"/> with generated files and warnings.
        /// </summary>
        /// <param name="generatedFiles">List of file paths that were generated.</param>
        /// <param name="warnings">List of warnings encountered during generation.</param>
        public PackGeneratorResult(IReadOnlyList<string> generatedFiles, IReadOnlyList<string> warnings)
        {
            GeneratedFiles = generatedFiles;
            Warnings = warnings;
        }
    }
}
