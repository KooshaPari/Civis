using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Strongly-typed representation of a DINOForge faction definition (factions/*.yaml).
    /// Corresponds to schemas/faction.schema.yaml.
    /// </summary>
    public class FactionDefinition : IValidatable
    {
        /// <summary>Core faction identity (id, name, theme, archetype).</summary>
        [YamlMember(Alias = "faction")]
        public FactionInfo Faction { get; set; } = new FactionInfo();

        /// <summary>Economy modifiers for this faction.</summary>
        [YamlMember(Alias = "economy")]
        public FactionEconomy Economy { get; set; } = new FactionEconomy();

        /// <summary>Army-wide modifiers for this faction.</summary>
        [YamlMember(Alias = "army")]
        public FactionArmy Army { get; set; } = new FactionArmy();

        /// <summary>
        /// Maps abstract role slots to concrete unit IDs.
        /// Keys: cheap_infantry, line_infantry, elite_infantry, anti_armor, support_weapon,
        /// recon, light_vehicle, heavy_vehicle, artillery, hero_commander, spike_unit.
        /// </summary>
        [YamlMember(Alias = "roster")]
        public FactionRoster Roster { get; set; } = new FactionRoster();

        /// <summary>
        /// Maps abstract building roles to concrete building IDs.
        /// Keys: barracks, workshop, artillery_foundry, tower_mg, heavy_defense,
        /// command_center, economy_primary, economy_secondary, research_facility, wall_segment.
        /// </summary>
        [YamlMember(Alias = "buildings")]
        public FactionBuildings Buildings { get; set; } = new FactionBuildings();

        /// <summary>Optional visual theme overrides (colors, projectile pack, UI skin).</summary>
        [YamlMember(Alias = "visuals")]
        public FactionVisuals? Visuals { get; set; }

        /// <summary>Optional audio pack overrides (weapons, structures, ambient, music).</summary>
        [YamlMember(Alias = "audio")]
        public FactionAudio? Audio { get; set; }

        /// <summary>
        /// Validates that the faction definition is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new System.Collections.Generic.List<ValidationError>();
            if (Faction == null || string.IsNullOrWhiteSpace(Faction.Id))
                errors.Add(new ValidationError("faction.id", "Faction.Id is required.", "validation"));
            if (Faction == null || string.IsNullOrWhiteSpace(Faction.DisplayName))
                errors.Add(new ValidationError("faction.display_name", "Faction.DisplayName is required.", "validation"));
            if (Visuals?.PrimaryColor != null && !IsValidHexColor(Visuals.PrimaryColor))
                errors.Add(new ValidationError("visuals.primary_color", "PrimaryColor must be a valid hex color (e.g., #RRGGBB).", "validation"));
            if (Visuals?.AccentColor != null && !IsValidHexColor(Visuals.AccentColor))
                errors.Add(new ValidationError("visuals.accent_color", "AccentColor must be a valid hex color (e.g., #RRGGBB).", "validation"));

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }

        private static bool IsValidHexColor(string color)
        {
            if (string.IsNullOrEmpty(color))
                return false;
            if (!color.StartsWith("#"))
                return false;
            if (color.Length != 7)
                return false;
            for (int i = 1; i < color.Length; i++)
            {
                char c = color[i];
                if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F')))
                    return false;
            }
            return true;
        }
    }

    /// <summary>
    /// Core identity block for a faction definition.
    /// </summary>
    public class FactionInfo
    {
        /// <summary>Unique faction identifier.</summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>Human-readable faction name shown in-game.</summary>
        [YamlMember(Alias = "display_name")]
        public string DisplayName { get; set; } = "";

        /// <summary>Optional faction lore or description text.</summary>
        [YamlMember(Alias = "description")]
        public string? Description { get; set; }

        /// <summary>
        /// Theme. Valid values: starwars, modern, futuristic, fantasy, custom.
        /// </summary>
        [YamlMember(Alias = "theme")]
        public string Theme { get; set; } = "";

        /// <summary>
        /// Archetype. Valid values: order, industrial_swarm, asymmetric, custom.
        /// </summary>
        [YamlMember(Alias = "archetype")]
        public string Archetype { get; set; } = "";

        /// <summary>
        /// Doctrine identifier (e.g. elite_discipline, mechanized_attrition).
        /// </summary>
        [YamlMember(Alias = "doctrine")]
        public string? Doctrine { get; set; }

        /// <summary>Optional icon asset path for this faction.</summary>
        [YamlMember(Alias = "icon")]
        public string? Icon { get; set; }
    }

    /// <summary>
    /// Economy modifiers applied to a faction's resource gathering, upkeep, research, and building.
    /// </summary>
    public class FactionEconomy
    {
        /// <summary>
        /// Resource gather rate multiplier. Default 1.0.
        /// </summary>
        [YamlMember(Alias = "gather_bonus")]
        public float GatherBonus { get; set; } = 1.0f;

        /// <summary>
        /// Unit upkeep cost multiplier. Default 1.0.
        /// </summary>
        [YamlMember(Alias = "upkeep_modifier")]
        public float UpkeepModifier { get; set; } = 1.0f;

        /// <summary>
        /// Research speed multiplier. Default 1.0.
        /// </summary>
        [YamlMember(Alias = "research_speed")]
        public float ResearchSpeed { get; set; } = 1.0f;

        /// <summary>
        /// Building construction speed multiplier. Default 1.0.
        /// </summary>
        [YamlMember(Alias = "build_speed")]
        public float BuildSpeed { get; set; } = 1.0f;
    }

    /// <summary>
    /// Army-wide modifiers applied to a faction's military capabilities.
    /// </summary>
    public class FactionArmy
    {
        /// <summary>
        /// Morale style. Valid values: disciplined, mechanical, fanatical, irregular, custom.
        /// </summary>
        [YamlMember(Alias = "morale_style")]
        public string? MoraleStyle { get; set; }

        /// <summary>
        /// Unit cap multiplier. Default 1.0.
        /// </summary>
        [YamlMember(Alias = "unit_cap_modifier")]
        public float UnitCapModifier { get; set; } = 1.0f;

        /// <summary>
        /// Elite unit cost multiplier. Default 1.0.
        /// </summary>
        [YamlMember(Alias = "elite_cost_modifier")]
        public float EliteCostModifier { get; set; } = 1.0f;

        /// <summary>
        /// Unit spawn rate multiplier. Default 1.0.
        /// </summary>
        [YamlMember(Alias = "spawn_rate_modifier")]
        public float SpawnRateModifier { get; set; } = 1.0f;
    }

    /// <summary>
    /// Maps abstract unit role slots to concrete unit definition IDs for a faction.
    /// </summary>
    public class FactionRoster
    {
        /// <summary>Unit ID for the cheap infantry role slot.</summary>
        [YamlMember(Alias = "cheap_infantry")]
        public string? CheapInfantry { get; set; }

        /// <summary>Unit ID for the line infantry role slot.</summary>
        [YamlMember(Alias = "line_infantry")]
        public string? LineInfantry { get; set; }

        /// <summary>Unit ID for the elite infantry role slot.</summary>
        [YamlMember(Alias = "elite_infantry")]
        public string? EliteInfantry { get; set; }

        /// <summary>Unit ID for the anti-armor role slot.</summary>
        [YamlMember(Alias = "anti_armor")]
        public string? AntiArmor { get; set; }

        /// <summary>Unit ID for the support weapon role slot.</summary>
        [YamlMember(Alias = "support_weapon")]
        public string? SupportWeapon { get; set; }

        /// <summary>Unit ID for the recon role slot.</summary>
        [YamlMember(Alias = "recon")]
        public string? Recon { get; set; }

        /// <summary>Unit ID for the light vehicle role slot.</summary>
        [YamlMember(Alias = "light_vehicle")]
        public string? LightVehicle { get; set; }

        /// <summary>Unit ID for the heavy vehicle role slot.</summary>
        [YamlMember(Alias = "heavy_vehicle")]
        public string? HeavyVehicle { get; set; }

        /// <summary>Unit ID for the artillery role slot.</summary>
        [YamlMember(Alias = "artillery")]
        public string? Artillery { get; set; }

        /// <summary>Unit ID for the hero/commander role slot.</summary>
        [YamlMember(Alias = "hero_commander")]
        public string? HeroCommander { get; set; }

        /// <summary>Unit ID for the spike unit role slot.</summary>
        [YamlMember(Alias = "spike_unit")]
        public string? SpikeUnit { get; set; }
    }

    /// <summary>
    /// Maps abstract building roles to concrete building definition IDs for a faction.
    /// </summary>
    public class FactionBuildings
    {
        /// <summary>Building ID for the barracks role.</summary>
        [YamlMember(Alias = "barracks")]
        public string? Barracks { get; set; }

        /// <summary>Building ID for the workshop role.</summary>
        [YamlMember(Alias = "workshop")]
        public string? Workshop { get; set; }

        /// <summary>Building ID for the artillery foundry role.</summary>
        [YamlMember(Alias = "artillery_foundry")]
        public string? ArtilleryFoundry { get; set; }

        /// <summary>Building ID for the machine gun tower role.</summary>
        [YamlMember(Alias = "tower_mg")]
        public string? TowerMg { get; set; }

        /// <summary>Building ID for the heavy defense role.</summary>
        [YamlMember(Alias = "heavy_defense")]
        public string? HeavyDefense { get; set; }

        /// <summary>Building ID for the command center role.</summary>
        [YamlMember(Alias = "command_center")]
        public string? CommandCenter { get; set; }

        /// <summary>Building ID for the primary economy building role.</summary>
        [YamlMember(Alias = "economy_primary")]
        public string? EconomyPrimary { get; set; }

        /// <summary>Building ID for the secondary economy building role.</summary>
        [YamlMember(Alias = "economy_secondary")]
        public string? EconomySecondary { get; set; }

        /// <summary>Building ID for the research facility role.</summary>
        [YamlMember(Alias = "research_facility")]
        public string? ResearchFacility { get; set; }

        /// <summary>Building ID for the wall segment role.</summary>
        [YamlMember(Alias = "wall_segment")]
        public string? WallSegment { get; set; }
    }

    /// <summary>
    /// Visual theme overrides for a faction (colors, projectile effects, UI skin).
    /// </summary>
    public class FactionVisuals
    {
        /// <summary>
        /// Hex color string, e.g. "#FF0000".
        /// </summary>
        [YamlMember(Alias = "primary_color")]
        public string? PrimaryColor { get; set; }

        /// <summary>
        /// Hex color string, e.g. "#FF0000".
        /// </summary>
        [YamlMember(Alias = "accent_color")]
        public string? AccentColor { get; set; }

        /// <summary>Projectile VFX pack asset ID.</summary>
        [YamlMember(Alias = "projectile_pack")]
        public string? ProjectilePack { get; set; }

        /// <summary>UI skin asset ID for faction-specific interface.</summary>
        [YamlMember(Alias = "ui_skin")]
        public string? UiSkin { get; set; }
    }

    /// <summary>
    /// Audio pack overrides for a faction (weapons, structures, ambient, music).
    /// </summary>
    public class FactionAudio
    {
        /// <summary>Weapon sound effects pack asset ID.</summary>
        [YamlMember(Alias = "weapon_pack")]
        public string? WeaponPack { get; set; }

        /// <summary>Structure sound effects pack asset ID.</summary>
        [YamlMember(Alias = "structure_pack")]
        public string? StructurePack { get; set; }

        /// <summary>Ambient sound effects pack asset ID.</summary>
        [YamlMember(Alias = "ambient_pack")]
        public string? AmbientPack { get; set; }

        /// <summary>Music pack asset ID.</summary>
        [YamlMember(Alias = "music_pack")]
        public string? MusicPack { get; set; }
    }
}
