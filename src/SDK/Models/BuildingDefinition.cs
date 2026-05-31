using System;
using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Strongly-typed representation of a DINOForge building definition (buildings/*.yaml).
    /// </summary>
    public sealed class BuildingDefinition : IValidatable
    {
        /// <summary>Unique building identifier.</summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>Human-readable building name shown in-game.</summary>
        [YamlMember(Alias = "display_name")]
        public string DisplayName { get; set; } = "";

        /// <summary>Optional flavor or tooltip text for the building.</summary>
        [YamlMember(Alias = "description")]
        public string? Description { get; set; }

        /// <summary>
        /// Functional category of this building
        /// (e.g. barracks, economy, defense, research, command).
        /// </summary>
        [YamlMember(Alias = "building_type")]
        public string? BuildingType { get; set; }

        /// <summary>
        /// Optional vanilla DINO <c>BuildingType</c> enum name this pack building aliases
        /// into the live build menu (e.g. "Stables", "Tower", "Port"). DINO's
        /// <c>BuildingType</c> enum is a closed compiled set — a genuinely new building
        /// type cannot be added to the native build menu at runtime, so pack buildings ride
        /// on an existing buildable slot, then get reskinned (mesh-swap) and re-targeted
        /// (UnitsShop production) by the runtime. When omitted, the runtime
        /// <c>BuildMenuInjector</c> auto-maps by <see cref="BuildingType"/>
        /// (Production→Stables, Defense→Tower, naval→Port). See
        /// docs/sessions/dino-build-catalog-20260530.md.
        /// </summary>
        [YamlMember(Alias = "build_alias")]
        public string? BuildAlias { get; set; }

        /// <summary>Resource cost to construct this building.</summary>
        [YamlMember(Alias = "cost")]
        public ResourceCost Cost { get; set; } = new ResourceCost();

        /// <summary>
        /// Total hit points of the building.
        /// </summary>
        [YamlMember(Alias = "health")]
        public int Health { get; set; } = 0;

        /// <summary>
        /// Production rates for this building.
        /// Keys are resource or unit identifiers; values are production amounts per tick.
        /// </summary>
        [YamlMember(Alias = "production")]
        public Dictionary<string, int> Production { get; set; } = new Dictionary<string, int>(StringComparer.Ordinal); // public-mutable-ok: YAML deserializer requires mutable Dictionary for YamlDotNet

        /// <summary>
        /// Addressables key for the building's 3D visual asset (LOD0 prefab).
        /// Set by the asset pipeline; resolved at runtime via the pack's addressables.yaml catalog.
        /// </summary>
        [YamlMember(Alias = "visual_asset")]
        public string? VisualAsset { get; set; }

        /// <summary>
        /// Pack <c>vanilla_mapping</c> value (e.g. <c>command</c>, <c>barracks</c>, <c>resource</c>,
        /// <c>wall</c>, <c>tower</c>) identifying which vanilla DINO building archetype the
        /// runtime <c>AssetSwapSystem</c> should target when swapping the building's live
        /// RenderMesh. This is the building-side analogue of <see cref="UnitDefinition.VanillaMapping"/>.
        ///
        /// Gap A (#975): prior to this field, <c>ContentLoader.RegisterAssetSwaps</c> registered
        /// building swaps WITHOUT a <c>vanilla_mapping</c>, so every building swap ran in the
        /// system's "no targeting signal" DIAGNOSTIC MODE and was skipped. Carrying this value
        /// through to <see cref="DINOForge.SDK.Assets.AssetSwapRequest.VanillaMapping"/> lets the
        /// swap narrow to the building archetype (or, when the archetype type cannot be resolved,
        /// fall back to mesh-name substring filtering keyed by the same mapping value).
        /// When omitted the building swap targets all <c>Components.BuildingBase</c> entities via
        /// the default <c>building</c> mapping (see <c>PackStatMappings</c>).
        /// </summary>
        [YamlMember(Alias = "vanilla_mapping")]
        public string? VanillaMapping { get; set; }

        /// <summary>
        /// Defense tags controlling how this building interacts with combat systems.
        /// Supported values: AntiAir, Fortified, Shielded.
        /// A building with "AntiAir" will have <c>AntiAirComponent</c>
        /// attached to its ECS entity on world load by <c>AerialBuildingMapper</c>.
        /// </summary>
        [YamlMember(Alias = "defense_tags")]
        public List<string> DefenseTags { get; set; } = new List<string>(); // public-mutable-ok: YAML deserializer requires mutable List<T> for YamlDotNet

        /// <summary>
        /// Anti-air parameters. Only relevant when <see cref="DefenseTags"/> contains "AntiAir".
        /// If omitted, defaults in <c>AerialBuildingMapper</c> apply (range 20f, damage bonus 1.5f).
        /// </summary>
        [YamlMember(Alias = "anti_air")]
        public BuildingAntiAirProperties? AntiAir { get; set; }

        /// <summary>
        /// Validates that the building definition is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new System.Collections.Generic.List<ValidationError>();
            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "Id is required.", "validation"));
            if (string.IsNullOrWhiteSpace(DisplayName))
                errors.Add(new ValidationError("display_name", "DisplayName is required.", "validation"));
            if (Health < 0)
                errors.Add(new ValidationError("health", "Health must be non-negative.", "validation"));

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }

    /// <summary>
    /// Anti-air combat parameters for a building definition.
    /// Deserialized from the <c>anti_air:</c> block in building YAML files.
    ///
    /// These values are mapped to <c>AntiAirComponent</c>
    /// when a building with the <c>AntiAir</c> defense_tag is initialised on world load.
    /// </summary>
    public sealed class BuildingAntiAirProperties
    {
        /// <summary>
        /// Maximum engagement range in world units at which this building can
        /// target aerial entities.  Default 20f.
        /// </summary>
        [YamlMember(Alias = "range")]
        public float Range { get; set; } = 20f;

        /// <summary>
        /// Damage multiplier applied when attacking aerial targets
        /// (e.g. 1.5f = +50 % damage vs aerial).  Default 1.5f.
        /// </summary>
        [YamlMember(Alias = "damage_bonus")]
        public float DamageBonus { get; set; } = 1.5f;
    }
}
