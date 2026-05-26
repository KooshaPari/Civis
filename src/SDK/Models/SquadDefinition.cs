using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Strongly-typed representation of a DINOForge squad definition (squads/*.yaml).
    /// Maps to DINO's Components.RawComponents.SquadMarker, Components.FormationViewParameters,
    /// Components.FormationArrowData, Components.FormationDrawParameters,
    /// Components.FormationViewMainColor, and Components.FormationViewBackColor ECS components.
    /// </summary>
    public sealed class SquadDefinition : IValidatable
    {
        /// <summary>Unique squad identifier.</summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>Human-readable squad name shown in-game.</summary>
        [YamlMember(Alias = "display_name")]
        public string DisplayName { get; set; } = "";

        /// <summary>Optional description of the squad.</summary>
        [YamlMember(Alias = "description")]
        public string? Description { get; set; }

        /// <summary>
        /// Reference to a UnitDefinition ID that populates this squad.
        /// </summary>
        [YamlMember(Alias = "unit_id")]
        public string UnitId { get; set; } = "";

        /// <summary>
        /// Minimum number of units in the squad.
        /// </summary>
        [YamlMember(Alias = "min_size")]
        public int MinSize { get; set; } = 1;

        /// <summary>
        /// Maximum number of units in the squad.
        /// </summary>
        [YamlMember(Alias = "max_size")]
        public int MaxSize { get; set; } = 20;

        /// <summary>
        /// Default formation layout. Valid values: line, column, wedge, circle, scattered.
        /// Maps to Components.FormationDrawParameters.
        /// </summary>
        [YamlMember(Alias = "default_formation")]
        public string DefaultFormation { get; set; } = "line";

        /// <summary>
        /// Spacing between units in formation (world units).
        /// Maps to Components.FormationViewParameters.
        /// </summary>
        [YamlMember(Alias = "formation_spacing")]
        public float FormationSpacing { get; set; } = 1.0f;

        /// <summary>
        /// Primary color as hex string (e.g. "#FF0000").
        /// Maps to Components.FormationViewMainColor.
        /// </summary>
        [YamlMember(Alias = "color_primary")]
        public string? ColorPrimary { get; set; }

        /// <summary>
        /// Secondary/back color as hex string (e.g. "#0000FF").
        /// Maps to Components.FormationViewBackColor.
        /// </summary>
        [YamlMember(Alias = "color_secondary")]
        public string? ColorSecondary { get; set; }

        /// <summary>
        /// Behavior tags for AI control. Valid values: hold_position, aggressive, defensive, patrol.
        /// </summary>
        [YamlMember(Alias = "behavior_tags")]
        public List<string> BehaviorTags { get; set; } = new List<string>(); // public-mutable-ok: YAML deserializer requires mutable List<T> for YamlDotNet

        /// <summary>
        /// Validates that the squad definition is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new List<ValidationError>();

            if (string.IsNullOrWhiteSpace(Id))
            {
                errors.Add(new ValidationError("id", "Squad id must not be empty", "required"));
            }

            if (string.IsNullOrWhiteSpace(DisplayName))
            {
                errors.Add(new ValidationError("display_name", "Squad display_name must not be empty", "required"));
            }

            if (MaxSize < MinSize)
            {
                errors.Add(new ValidationError("max_size", "MaxSize cannot be less than MinSize", "constraint-violation"));
            }

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }
}
