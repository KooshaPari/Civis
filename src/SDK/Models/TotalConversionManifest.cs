#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Manifest for a total conversion pack — replaces all vanilla game content
    /// with themed alternatives (e.g. Star Wars, Modern Warfare).
    /// </summary>
    public sealed class TotalConversionManifest : IValidatable
    {
        /// <summary>
        /// Unique identifier for the total conversion pack.
        /// </summary>
        public string Id { get; set; } = "";

        /// <summary>
        /// Human-readable name of the total conversion.
        /// </summary>
        public string Name { get; set; } = "";

        /// <summary>
        /// Semantic version of the total conversion pack.
        /// </summary>
        public string Version { get; set; } = "0.1.0";

        /// <summary>
        /// Pack type identifier (always "total_conversion").
        /// </summary>
        public string Type { get; set; } = "total_conversion";

        /// <summary>
        /// Author or organization that created the total conversion.
        /// </summary>
        public string Author { get; set; } = "";

        /// <summary>
        /// Visual theme name (e.g., "starwars", "modern", "medieval").
        /// </summary>
        public string? Theme { get; set; }

        /// <summary>
        /// Optional description of the total conversion's content and changes.
        /// </summary>
        public string? Description { get; set; }

        /// <summary>
        /// Framework version constraint for the total conversion.
        /// </summary>
        [YamlMember(Alias = "framework_version")]
        public string FrameworkVersion { get; set; } = ">=0.1.0 <1.0.0";

        /// <summary>Maps vanilla faction IDs to replacement faction IDs.</summary>
        [YamlMember(Alias = "replaces_vanilla")]
        public Dictionary<string, string> ReplacesVanilla { get; set; } = new(StringComparer.Ordinal); // public-mutable-ok: YAML deserializer requires mutable Dictionary for YamlDotNet

        /// <summary>
        /// List of faction replacements defined in this total conversion.
        /// </summary>
        public List<TcFactionEntry> Factions { get; set; } = new(); // public-mutable-ok: YAML deserializer requires mutable List<T> for YamlDotNet

        /// <summary>
        /// Asset replacement mappings (textures, audio, UI).
        /// </summary>
        [YamlMember(Alias = "asset_replacements")]
        public TcAssetReplacements AssetReplacements { get; set; } = new();

        /// <summary>
        /// List of pack IDs that conflict with this total conversion.
        /// </summary>
        [YamlMember(Alias = "conflicts_with")]
        public List<string> ConflictsWith { get; set; } = new(); // public-mutable-ok: YAML deserializer requires mutable List<T> for YamlDotNet

        /// <summary>If true, only one total conversion can be active at a time.</summary>
        public bool Singleton { get; set; } = true;

        /// <summary>
        /// Validates that the total conversion manifest is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new List<ValidationError>();

            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "Id is required.", "validation"));
            if (string.IsNullOrWhiteSpace(Name))
                errors.Add(new ValidationError("name", "Name is required.", "validation"));
            if (string.IsNullOrWhiteSpace(Version))
                errors.Add(new ValidationError("version", "Version is required.", "validation"));
            if (string.IsNullOrWhiteSpace(Type))
                errors.Add(new ValidationError("type", "Type is required.", "validation"));
            else if (Type != "total_conversion")
                errors.Add(new ValidationError("type", "Type must be 'total_conversion'.", "validation"));
            if (string.IsNullOrWhiteSpace(FrameworkVersion))
                errors.Add(new ValidationError("framework_version", "FrameworkVersion is required.", "validation"));

            if (Factions != null)
            {
                for (int i = 0; i < Factions.Count; i++)
                {
                    var f = Factions[i];
                    if (f == null) continue;
                    if (string.IsNullOrWhiteSpace(f.Id))
                        errors.Add(new ValidationError($"factions[{i}].id", "Faction entry Id is required.", "validation"));
                    if (string.IsNullOrWhiteSpace(f.Replaces))
                        errors.Add(new ValidationError($"factions[{i}].replaces", "Faction entry Replaces is required.", "validation"));
                    if (string.IsNullOrWhiteSpace(f.Name))
                        errors.Add(new ValidationError($"factions[{i}].name", "Faction entry Name is required.", "validation"));
                }
            }

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }

    /// <summary>A single faction replacement entry in a total conversion.</summary>
    public sealed class TcFactionEntry
    {
        /// <summary>
        /// Unique identifier for this faction entry.
        /// </summary>
        public string Id { get; set; } = "";

        /// <summary>Vanilla faction ID being replaced.</summary>
        public string Replaces { get; set; } = "";

        /// <summary>
        /// Human-readable name of the replacement faction.
        /// </summary>
        public string Name { get; set; } = "";

        /// <summary>
        /// Visual theme identifier for this faction (e.g., "rebel", "empire").
        /// </summary>
        public string? Theme { get; set; }

        /// <summary>
        /// Faction archetype (determines unit roster and behavior).
        /// </summary>
        public string Archetype { get; set; } = "custom";

        /// <summary>
        /// List of unit IDs that belong to this faction.
        /// </summary>
        public List<string> Units { get; set; } = new(); // public-mutable-ok: YAML deserializer requires mutable List<T> for YamlDotNet

        /// <summary>
        /// List of building IDs that this faction can construct.
        /// </summary>
        public List<string> Buildings { get; set; } = new(); // public-mutable-ok: YAML deserializer requires mutable List<T> for YamlDotNet
    }

    /// <summary>Asset replacement mappings for a total conversion.</summary>
    public sealed class TcAssetReplacements
    {
        /// <summary>
        /// Maps vanilla texture asset paths to replacement texture asset paths.
        /// </summary>
        public Dictionary<string, string> Textures { get; set; } = new(StringComparer.Ordinal); // public-mutable-ok: YAML deserializer requires mutable Dictionary for YamlDotNet

        /// <summary>
        /// Maps vanilla audio asset paths to replacement audio asset paths.
        /// </summary>
        public Dictionary<string, string> Audio { get; set; } = new(StringComparer.Ordinal); // public-mutable-ok: YAML deserializer requires mutable Dictionary for YamlDotNet

        /// <summary>
        /// Maps vanilla UI asset paths to replacement UI asset paths.
        /// </summary>
        public Dictionary<string, string> Ui { get; set; } = new(StringComparer.Ordinal); // public-mutable-ok: YAML deserializer requires mutable Dictionary for YamlDotNet
    }
}
