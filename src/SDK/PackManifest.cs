using System.Collections.Generic;
using DINOForge.SDK.Models;
using DINOForge.SDK.Patching;
using YamlDotNet.Serialization;

namespace DINOForge.SDK
{
    /// <summary>
    /// Strongly-typed representation of a DINOForge pack manifest (pack.yaml).
    /// Contains metadata about a content pack, including dependencies and version constraints.
    /// Corresponds to schemas/pack-manifest.schema.yaml.
    /// </summary>
    public sealed class PackManifest
    {
        /// <summary>
        /// Unique identifier for the pack.
        /// </summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>
        /// Human-readable name of the pack.
        /// </summary>
        [YamlMember(Alias = "name")]
        public string Name { get; set; } = "";

        /// <summary>
        /// Semantic version of the pack.
        /// </summary>
        [YamlMember(Alias = "version")]
        public string Version { get; set; } = "0.1.0";

        /// <summary>
        /// Framework version constraint for the pack (e.g., "&gt;=0.1.0").
        /// </summary>
        [YamlMember(Alias = "framework_version")]
        public string FrameworkVersion { get; set; } = ">=0.1.0 <1.0.0";

        /// <summary>
        /// Author or organization that created the pack.
        /// </summary>
        [YamlMember(Alias = "author")]
        public string Author { get; set; } = "";

        /// <summary>
        /// Pack type: content, balance, ruleset, total_conversion, or utility.
        /// </summary>
        [YamlMember(Alias = "type")]
        public string Type { get; set; } = "content";

        /// <summary>
        /// Pack classification tier: engine_extension, content, total_conversion, or baseline.
        /// Determines how the pack is displayed in mod menus and which features it can access.
        /// </summary>
        [YamlMember(Alias = "classification")]
        public string? Classification { get; set; }

        /// <summary>
        /// Optional description of the pack's purpose and content.
        /// </summary>
        [YamlMember(Alias = "description")]
        public string? Description { get; set; }

        /// <summary>
        /// List of pack IDs that this pack depends on.
        /// </summary>
        [YamlMember(Alias = "depends_on")]
        public List<string> DependsOn { get; set; } = new List<string>(); // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// List of pack IDs that conflict with this pack.
        /// </summary>
        [YamlMember(Alias = "conflicts_with")]
        public List<string> ConflictsWith { get; set; } = new List<string>(); // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Load order priority for the pack (higher loads later).
        /// </summary>
        [YamlMember(Alias = "load_order")]
        public int LoadOrder { get; set; } = 100;

        /// <summary>
        /// Game version constraint (e.g., "&gt;=0.0.0 &lt;2.0.0").
        /// </summary>
        [YamlMember(Alias = "game_version")]
        public string GameVersion { get; set; } = ">=0.0.0 <2.0.0";

        /// <summary>
        /// BepInEx version constraint (e.g., "&gt;=5.4.0").
        /// </summary>
        [YamlMember(Alias = "bepinex_version")]
        public string BepInExVersion { get; set; } = ">=5.4.0 <6.0.0";

        /// <summary>
        /// Unity version constraint (e.g., "&gt;=2021.3.0 &lt;2022.0.0").
        /// </summary>
        [YamlMember(Alias = "unity_version")]
        public string UnityVersion { get; set; } = ">=2021.3.0 <2022.0.0";

        /// <summary>
        /// Optional homepage URL for the pack (opened in browser from mod menu).
        /// </summary>
        [YamlMember(Alias = "homepage_url")]
        public string? HomepageUrl { get; set; }

        /// <summary>
        /// Optional GitHub repository URL for the pack.
        /// </summary>
        [YamlMember(Alias = "github_url")]
        public string? GithubUrl { get; set; }

        /// <summary>
        /// Optional Discord server/channel invite URL for the pack.
        /// </summary>
        [YamlMember(Alias = "discord_url")]
        public string? DiscordUrl { get; set; }

        /// <summary>
        /// SPDX license identifier (e.g., "CC0-1.0", "CC-BY-4.0", "MIT").
        /// Displayed as a badge in the mod menu detail pane.
        /// </summary>
        [YamlMember(Alias = "license")]
        public string? License { get; set; }

        /// <summary>
        /// Searchable tags for the pack (e.g., ["warfare", "sci-fi", "total-conversion"]).
        /// Displayed as colored chips in the mod menu detail pane.
        /// </summary>
        [YamlMember(Alias = "tags")]
        public List<string>? Tags { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Content types and files to load from this pack.
        /// </summary>
        [YamlMember(Alias = "loads")]
        public PackLoads? Loads { get; set; }

        /// <summary>
        /// Override definitions for vanilla game content.
        /// </summary>
        [YamlMember(Alias = "overrides")]
        public PackOverrides? Overrides { get; set; }

        /// <summary>
        /// Optional GitHub release update-check configuration.
        /// When set, the runtime polls the specified repository for newer releases and
        /// displays an "Update available" row in the F10 mod menu.
        /// </summary>
        [YamlMember(Alias = "update_check")]
        public PackUpdateCheck? UpdateCheck { get; set; }

        /// <summary>
        /// RimWorld-style cross-pack patch operations.
        /// Each entry targets another loaded pack and applies a sequence of field mutations
        /// (replace, add, remove, multiply) AFTER all packs are loaded but BEFORE registries
        /// are populated. Patches are applied by <see cref="DINOForge.SDK.Patching.PatchApplicator"/>.
        /// </summary>
        [YamlMember(Alias = "patches")]
        public List<PatchSet>? Patches { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// User-configurable runtime settings for this pack.
        /// Exposed in the F10 mod menu's detail pane, allowing players to customize
        /// difficulty, enabled features, and other mod behaviors.
        /// </summary>
        [YamlMember(Alias = "settings")]
        public List<PackSetting>? Settings { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Author-declared badges for this pack.
        /// Valid author-declared values: <c>early-access</c>, <c>total-conversion</c>.
        /// Curated values (<c>verified-author</c>, <c>editors-choice</c>) are assigned externally
        /// via a signed list and will be silently stripped if declared directly in pack.yaml.
        /// Auto-computed values (<c>popular</c>, <c>compatibility-tested</c>) are appended at runtime
        /// based on download counters and CI results — do not declare them here.
        /// </summary>
        [YamlMember(Alias = "badges")]
        public List<string>? Badges { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet
    }

    /// <summary>
    /// GitHub release update-check configuration for a pack.
    /// Maps to the <c>update_check</c> key in pack.yaml.
    /// </summary>
    public sealed class PackUpdateCheck
    {
        /// <summary>
        /// GitHub repository owner (user or organisation), e.g. "KooshaPari".
        /// </summary>
        [YamlMember(Alias = "owner")]
        public string Owner { get; set; } = "";

        /// <summary>
        /// GitHub repository name, e.g. "warfare-starwars".
        /// </summary>
        [YamlMember(Alias = "repo")]
        public string Repo { get; set; } = "";
    }

    /// <summary>
    /// Specifies which content types to load from the pack.
    /// Each property lists file paths or directories to load.
    /// </summary>
    public sealed class PackLoads
    {
        /// <summary>
        /// Paths to faction definition files.
        /// </summary>
        [YamlMember(Alias = "factions")]
        public List<string>? Factions { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to unit definition files.
        /// </summary>
        [YamlMember(Alias = "units")]
        public List<string>? Units { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to building definition files.
        /// </summary>
        [YamlMember(Alias = "buildings")]
        public List<string>? Buildings { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to weapon definition files.
        /// </summary>
        [YamlMember(Alias = "weapons")]
        public List<string>? Weapons { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to doctrine definition files.
        /// </summary>
        [YamlMember(Alias = "doctrines")]
        public List<string>? Doctrines { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to audio asset files or directories.
        /// </summary>
        [YamlMember(Alias = "audio")]
        public List<string>? Audio { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to visual asset files or directories.
        /// </summary>
        [YamlMember(Alias = "visuals")]
        public List<string>? Visuals { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to localization data files.
        /// </summary>
        [YamlMember(Alias = "localization")]
        public List<string>? Localization { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to wave template definition files.
        /// </summary>
        [YamlMember(Alias = "wave_templates")]
        public List<string>? WaveTemplates { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to technology node definition files.
        /// </summary>
        [YamlMember(Alias = "tech_nodes")]
        public List<string>? TechNodes { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to scenario definition files.
        /// </summary>
        [YamlMember(Alias = "scenarios")]
        public List<string>? Scenarios { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to faction patch definition files.
        /// </summary>
        [YamlMember(Alias = "faction_patches")]
        public List<string>? FactionPatches { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>Paths to resource definition files.</summary>
        [YamlMember(Alias = "resources")]
        public List<string>? Resources { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>Paths to economy profile definition files.</summary>
        [YamlMember(Alias = "economy_profiles")]
        public List<string>? EconomyProfiles { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>Paths to trade route definition files.</summary>
        [YamlMember(Alias = "trade_routes")]
        public List<string>? TradeRoutes { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>Paths to HUD element definition files.</summary>
        [YamlMember(Alias = "hud_elements")]
        public List<string>? HudElements { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>Paths to menu definition files.</summary>
        [YamlMember(Alias = "menus")]
        public List<string>? Menus { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>Paths to UI theme definition files.</summary>
        [YamlMember(Alias = "ui_themes")]
        public List<string>? UiThemes { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>Paths to wave definition files.</summary>
        [YamlMember(Alias = "waves")]
        public List<string>? Waves { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>Paths to stat definition files.</summary>
        [YamlMember(Alias = "stats")]
        public List<string>? Stats { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet
    }

    /// <summary>
    /// Specifies vanilla game content to override with custom definitions.
    /// </summary>
    public sealed class PackOverrides
    {
        /// <summary>
        /// Paths to unit override definition files.
        /// </summary>
        [YamlMember(Alias = "units")]
        public List<string>? Units { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to building override definition files.
        /// </summary>
        [YamlMember(Alias = "buildings")]
        public List<string>? Buildings { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet

        /// <summary>
        /// Paths to stat override definition files.
        /// </summary>
        [YamlMember(Alias = "stats")]
        public List<string>? Stats { get; set; } // public-mutable-ok: YAML deserializer requires mutable List for YamlDotNet
    }
}
