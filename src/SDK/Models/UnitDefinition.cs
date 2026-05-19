using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Strongly-typed representation of a DINOForge unit definition (units/*.yaml).
    /// Corresponds to schemas/unit.schema.yaml.
    /// </summary>
    public class UnitDefinition : IValidatable
    {
        /// <summary>Unique unit identifier.</summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>Human-readable unit name shown in-game.</summary>
        [YamlMember(Alias = "display_name")]
        public string DisplayName { get; set; } = "";

        /// <summary>Optional flavor or tooltip text for the unit.</summary>
        [YamlMember(Alias = "description")]
        public string? Description { get; set; }

        /// <summary>
        /// Unit class. Valid values: MilitiaLight, CoreLineInfantry, EliteLineInfantry,
        /// HeavyInfantry, Skirmisher, AntiArmor, ShockMelee, SwarmFodder, FastVehicle,
        /// MainBattleVehicle, HeavySiege, Artillery, WalkerHeavy, StaticMG, StaticAT,
        /// StaticArtillery, SupportEngineer, Recon, HeroCommander, AirstrikeProxy, ShieldedElite.
        /// </summary>
        [YamlMember(Alias = "unit_class")]
        public string UnitClass { get; set; } = "";

        /// <summary>ID of the faction this unit belongs to.</summary>
        [YamlMember(Alias = "faction_id")]
        public string FactionId { get; set; } = "";

        /// <summary>
        /// Tech tier: 1 = cheap/early, 2 = mid, 3 = late/expensive.
        /// </summary>
        [YamlMember(Alias = "tier")]
        public int? Tier { get; set; }

        /// <summary>Combat and movement statistics for this unit.</summary>
        [YamlMember(Alias = "stats")]
        public UnitStats Stats { get; set; } = new UnitStats();

        /// <summary>
        /// Weapon class ID referencing a weapon in the weapon registry.
        /// </summary>
        [YamlMember(Alias = "weapon")]
        public string? Weapon { get; set; }

        /// <summary>
        /// Defense tags. Valid values: Unarmored, InfantryArmor, HeavyArmor,
        /// Fortified, Shielded, Mechanical, Biological, Heroic.
        /// </summary>
        [YamlMember(Alias = "defense_tags")]
        // public-mutable-ok: YAML deserialization requires mutable List<T> for YamlDotNet
        public List<string> DefenseTags { get; set; } = new List<string>();

        /// <summary>
        /// Behavior tags. Valid values: HoldLine, AdvanceFire, Charge, Kite,
        /// Swarm, SiegePriority, AntiStructure, AntiMass, AntiArmor, MoralePressure.
        /// </summary>
        [YamlMember(Alias = "behavior_tags")]
        // public-mutable-ok: YAML deserialization requires mutable List<T> for YamlDotNet
        public List<string> BehaviorTags { get; set; } = new List<string>();

        /// <summary>
        /// Addressables key for the unit's 3D visual asset (LOD0 prefab).
        /// Set by the asset pipeline; resolved at runtime via the pack's addressables.yaml catalog.
        /// </summary>
        [YamlMember(Alias = "visual_asset")]
        public string? VisualAsset { get; set; }

        /// <summary>Optional visual overrides (icon, portrait, model, VFX).</summary>
        [YamlMember(Alias = "visuals")]
        public UnitVisuals? Visuals { get; set; }

        /// <summary>Optional audio overrides (attack, death, select, move sounds).</summary>
        [YamlMember(Alias = "audio")]
        public UnitAudio? Audio { get; set; }

        /// <summary>
        /// The vanilla DINO unit ID this unit replaces or maps to.
        /// </summary>
        [YamlMember(Alias = "vanilla_mapping")]
        public string? VanillaMapping { get; set; }

        /// <summary>
        /// Tech node ID required to unlock this unit.
        /// </summary>
        [YamlMember(Alias = "tech_requirement")]
        public string? TechRequirement { get; set; }

        /// <summary>
        /// Aerial flight parameters. Only relevant for units with behavior_tag "Aerial".
        /// </summary>
        [YamlMember(Alias = "aerial")]
        public AerialProperties? Aerial { get; set; }

        /// <summary>
        /// Validates that the unit definition is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new System.Collections.Generic.List<ValidationError>();
            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "Id is required.", "validation"));
            if (string.IsNullOrWhiteSpace(DisplayName))
                errors.Add(new ValidationError("display_name", "DisplayName is required.", "validation"));
            if (string.IsNullOrWhiteSpace(FactionId))
                errors.Add(new ValidationError("faction_id", "FactionId is required.", "validation"));
            if (Stats?.Hp <= 0)
                errors.Add(new ValidationError("stats.hp", "Stats.Hp must be greater than 0.", "validation"));
            if (Stats?.Accuracy < 0 || Stats?.Accuracy > 1)
                errors.Add(new ValidationError("stats.accuracy", "Stats.Accuracy must be between 0 and 1.", "validation"));

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }

    /// <summary>
    /// Combat and movement statistics for a unit definition.
    /// </summary>
    public class UnitStats
    {
        /// <summary>Hit points. Default 1.</summary>
        [YamlMember(Alias = "hp")]
        public float Hp { get; set; } = 1f;

        /// <summary>Base damage per attack. Default 0.</summary>
        [YamlMember(Alias = "damage")]
        public float Damage { get; set; } = 0f;

        /// <summary>Armor value reducing incoming damage. Default 0.</summary>
        [YamlMember(Alias = "armor")]
        public float Armor { get; set; } = 0f;

        /// <summary>Attack range in world units. Default 0 (melee).</summary>
        [YamlMember(Alias = "range")]
        public float Range { get; set; } = 0f;

        /// <summary>Movement speed in world units per second. Default 0.</summary>
        [YamlMember(Alias = "speed")]
        public float Speed { get; set; } = 0f;

        /// <summary>Resource cost to produce this unit.</summary>
        [YamlMember(Alias = "cost")]
        public ResourceCost Cost { get; set; } = new ResourceCost();

        /// <summary>
        /// Hit chance, clamped 0.0 - 1.0. Default 0.7.
        /// </summary>
        [YamlMember(Alias = "accuracy")]
        public float Accuracy { get; set; } = 0.7f;

        /// <summary>
        /// Attacks per second. Default 1.0.
        /// </summary>
        [YamlMember(Alias = "fire_rate")]
        public float FireRate { get; set; } = 1.0f;

        /// <summary>
        /// Base morale value. Default 100.
        /// </summary>
        [YamlMember(Alias = "morale")]
        public float Morale { get; set; } = 100f;
    }

    /// <summary>
    /// Visual asset overrides for a unit (icon, portrait, model, VFX).
    /// </summary>
    public class UnitVisuals
    {
        /// <summary>Icon asset path for the unit selection panel.</summary>
        [YamlMember(Alias = "icon")]
        public string? Icon { get; set; }

        /// <summary>Portrait asset path for the unit info panel.</summary>
        [YamlMember(Alias = "portrait")]
        public string? Portrait { get; set; }

        /// <summary>3D model override asset ID.</summary>
        [YamlMember(Alias = "model_override")]
        public string? ModelOverride { get; set; }

        /// <summary>Projectile visual effect asset ID.</summary>
        [YamlMember(Alias = "projectile_vfx")]
        public string? ProjectileVfx { get; set; }

        /// <summary>Muzzle flash visual effect asset ID.</summary>
        [YamlMember(Alias = "muzzle_vfx")]
        public string? MuzzleVfx { get; set; }
    }

    /// <summary>
    /// Audio asset overrides for a unit (attack, death, selection, movement).
    /// </summary>
    public class UnitAudio
    {
        /// <summary>Sound effect played when the unit attacks.</summary>
        [YamlMember(Alias = "attack_sound")]
        public string? AttackSound { get; set; }

        /// <summary>Sound effect played when the unit dies.</summary>
        [YamlMember(Alias = "death_sound")]
        public string? DeathSound { get; set; }

        /// <summary>Sound effect played when the unit is selected.</summary>
        [YamlMember(Alias = "select_sound")]
        public string? SelectSound { get; set; }

        /// <summary>Sound effect played when the unit moves.</summary>
        [YamlMember(Alias = "move_sound")]
        public string? MoveSound { get; set; }
    }
}
