using System.Collections.Generic;
using DINOForge.SDK.Validation;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Aerial flight parameters for a unit definition.
    /// Deserialized from the <c>aerial:</c> block in unit YAML files.
    /// </summary>
    public sealed class AerialProperties : IValidatable
    {
        /// <summary>
        /// Target altitude in world units above ground (Y axis).
        /// Example: 15.0f = low flight (balloon), 25.0f = high flight (bomber).
        /// </summary>
        public float CruiseAltitude { get; set; } = 15f;

        /// <summary>
        /// Speed at which the unit climbs toward its cruise altitude (world units/second).
        /// </summary>
        public float AscendSpeed { get; set; } = 5f;

        /// <summary>
        /// Speed at which the unit descends when attacking or landing (world units/second).
        /// </summary>
        public float DescendSpeed { get; set; } = 3f;

        /// <summary>
        /// Whether this aerial unit can engage other aerial targets (e.g. interceptor role).
        /// </summary>
        public bool AntiAir { get; set; } = false;

        /// <summary>
        /// Validates that aerial parameters are semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new List<ValidationError>();

            if (CruiseAltitude < 0f)
                errors.Add(new ValidationError("cruise_altitude", "CruiseAltitude must be >= 0.", "validation"));
            if (AscendSpeed < 0f)
                errors.Add(new ValidationError("ascend_speed", "AscendSpeed must be >= 0.", "validation"));
            if (DescendSpeed < 0f)
                errors.Add(new ValidationError("descend_speed", "DescendSpeed must be >= 0.", "validation"));

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }
}
