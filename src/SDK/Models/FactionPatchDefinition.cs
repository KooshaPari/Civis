using System.Collections.Generic;
using DINOForge.SDK.Validation;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Defines additions to an existing vanilla faction.
    /// Used to extend player/enemy factions with new units, buildings, and doctrines
    /// without creating new factions or modifying vanilla data.
    ///
    /// Deserialized from YAML files in a pack's <c>patches/</c> or <c>faction_patches/</c> directory.
    ///
    /// The <c>target_faction</c> field accepts:
    ///   - <c>"player"</c> — the vanilla player faction (entities without Components.Enemy)
    ///   - <c>"enemy"</c>  — the vanilla enemy faction (entities with Components.Enemy)
    ///   - A specific faction ID from a loaded faction definition
    /// </summary>
    public class FactionPatchDefinition : IValidatable
    {
        /// <summary>
        /// ID of the faction to extend. Use "player" or "enemy" for vanilla factions.
        /// </summary>
        public string TargetFaction { get; set; } = "";

        /// <summary>
        /// Content to add to the target faction.
        /// </summary>
        public FactionPatchAdditions Add { get; set; } = new FactionPatchAdditions();

        /// <summary>
        /// Optional: override the role of specific unit IDs within the faction.
        /// Key = unit ID, Value = new role string.
        /// </summary>
        public Dictionary<string, string> RoleOverrides { get; set; } = new Dictionary<string, string>();

        /// <summary>
        /// Validates that the faction patch definition is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new System.Collections.Generic.List<ValidationError>();
            if (string.IsNullOrWhiteSpace(TargetFaction))
                errors.Add(new ValidationError("target_faction", "TargetFaction is required.", "validation"));
            if ((Add?.Units?.Count ?? 0) == 0 && (Add?.Buildings?.Count ?? 0) == 0 && (Add?.Doctrines?.Count ?? 0) == 0)
                errors.Add(new ValidationError("add", "At least one addition (units, buildings, or doctrines) is required.", "validation"));

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }

    /// <summary>
    /// The content additions for a faction patch.
    /// </summary>
    public class FactionPatchAdditions
    {
        /// <summary>Unit definition IDs to add to the faction.</summary>
        // public-mutable-ok: YAML deserialization requires mutable List<T> for YamlDotNet
        public List<string> Units { get; set; } = new List<string>();

        /// <summary>Building definition IDs to add to the faction.</summary>
        // public-mutable-ok: YAML deserialization requires mutable List<T> for YamlDotNet
        public List<string> Buildings { get; set; } = new List<string>();

        /// <summary>Doctrine definition IDs to add to the faction.</summary>
        // public-mutable-ok: YAML deserialization requires mutable List<T> for YamlDotNet
        public List<string> Doctrines { get; set; } = new List<string>();
    }
}
