using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Shared resource cost model used by units, buildings, and other definitions.
    /// </summary>
    public class ResourceCost : IValidatable
    {
        /// <summary>Food resource cost.</summary>
        [YamlMember(Alias = "food")]
        public int Food { get; set; } = 0;

        /// <summary>Wood resource cost.</summary>
        [YamlMember(Alias = "wood")]
        public int Wood { get; set; } = 0;

        /// <summary>Stone resource cost.</summary>
        [YamlMember(Alias = "stone")]
        public int Stone { get; set; } = 0;

        /// <summary>Iron resource cost.</summary>
        [YamlMember(Alias = "iron")]
        public int Iron { get; set; } = 0;

        /// <summary>Gold resource cost.</summary>
        [YamlMember(Alias = "gold")]
        public int Gold { get; set; } = 0;

        /// <summary>Population cost (housing slots consumed).</summary>
        [YamlMember(Alias = "population")]
        public int Population { get; set; } = 0;

        /// <summary>
        /// Validates that the resource cost is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new List<ValidationError>();
            if (Food < 0) errors.Add(new ValidationError("food", "Food cost cannot be negative", "min-value"));
            if (Wood < 0) errors.Add(new ValidationError("wood", "Wood cost cannot be negative", "min-value"));
            if (Stone < 0) errors.Add(new ValidationError("stone", "Stone cost cannot be negative", "min-value"));
            if (Iron < 0) errors.Add(new ValidationError("iron", "Iron cost cannot be negative", "min-value"));
            if (Gold < 0) errors.Add(new ValidationError("gold", "Gold cost cannot be negative", "min-value"));
            if (Population < 0) errors.Add(new ValidationError("population", "Population cost cannot be negative", "min-value"));

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }
}
