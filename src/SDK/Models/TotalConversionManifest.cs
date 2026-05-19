#nullable enable
using System.Collections.Generic;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Manifest for a total conversion pack — replaces all vanilla game content
    /// with themed alternatives (e.g. Star Wars, Modern Warfare).
    /// </summary>
    public class TotalConversionManifest
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
        public string FrameworkVersion { get; set; } = "*";

        /// <summary>Maps vanilla faction IDs to replacement faction IDs.</summary>
        [YamlMember(Alias = "replaces_vanilla")]
        public Dictionary<string, string> ReplacesVanilla { get; set; } = new();

        /// <summary>
        /// List of faction replacements defined in this total conversion.
        /// </summary>
        // public-mutable-ok: YAML deserialization requires mutable List<T> for YamlDotNet
        public List<TcFactionEntry> Factions { get; set; } = new();

        /// <summary>
        /// Asset replacement mappings (textures, audio, UI).
        /// </summary>
        [YamlMember(Alias = "asset_replacements")]
        public TcAssetReplacements AssetReplacements { get; set; } = new();

        /// <summary>
        /// List of pack IDs that conflict with this total conversion.
        /// </summary>
        [YamlMember(Alias = "conflicts_with")]
        // public-mutable-ok: YAML deserialization requires mutable List<T> for YamlDotNet
        public List<string> ConflictsWith { get; set; } = new();

        /// <summary>If true, only one total conversion can be active at a time.</summary>
        public bool Singleton { get; set; } = true;
    }

    /// <summary>A single faction replacement entry in a total conversion.</summary>
    public class TcFactionEntry
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
        // public-mutable-ok: YAML deserialization requires mutable List<T> for YamlDotNet
        public List<string> Units { get; set; } = new();

        /// <summary>
        /// List of building IDs that this faction can construct.
        /// </summary>
        // public-mutable-ok: YAML deserialization requires mutable List<T> for YamlDotNet
        public List<string> Buildings { get; set; } = new();
    }

    /// <summary>Asset replacement mappings for a total conversion.</summary>
    public class TcAssetReplacements
    {
        /// <summary>
        /// Maps vanilla texture asset paths to replacement texture asset paths.
        /// </summary>
        public Dictionary<string, string> Textures { get; set; } = new();

        /// <summary>
        /// Maps vanilla audio asset paths to replacement audio asset paths.
        /// </summary>
        public Dictionary<string, string> Audio { get; set; } = new();

        /// <summary>
        /// Maps vanilla UI asset paths to replacement UI asset paths.
        /// </summary>
        public Dictionary<string, string> Ui { get; set; } = new();
    }
}
