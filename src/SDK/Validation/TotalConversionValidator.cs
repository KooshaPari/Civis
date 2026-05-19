#nullable enable
using System.Collections.Generic;
using System.Linq;
using DINOForge.SDK.Models;

namespace DINOForge.SDK.Validation
{
    /// <summary>
    /// Validates total conversion packs for completeness and consistency.
    /// Checks faction coverage, asset references, and singleton enforcement.
    /// </summary>
    public static class TotalConversionValidator
    {
        /// <summary>Known vanilla faction IDs in the base game.</summary>
        public static readonly IReadOnlyList<string> VanillaFactionIds = new[]
        {
            "player", "enemy_classic", "enemy_guerrilla"
        };

        /// <summary>
        /// Validates a total conversion manifest.
        /// </summary>
        public static TcValidationResult Validate(TotalConversionManifest manifest)
        {
            var result = new TcValidationResult { ManifestId = manifest.Id };

            // Must be total_conversion type
            if (manifest.Type != "total_conversion")
                result.Errors.Add($"Pack type must be 'total_conversion', got '{manifest.Type}'");

            // Must replace at least one vanilla faction
            if (manifest.ReplacesVanilla.Count == 0)
                result.Errors.Add("total_conversion must define replaces_vanilla mappings");

            // Check all declared vanilla replacements are known
            foreach (string vanillaId in manifest.ReplacesVanilla.Keys)
            {
                if (!VanillaFactionIds.Contains(vanillaId))
                    result.Warnings.Add($"Unknown vanilla faction '{vanillaId}' in replaces_vanilla (may be valid for modded bases)");
            }

            // Each faction entry must have a valid archetype
            var validArchetypes = new[] { "order", "industrial_swarm", "asymmetric", "custom" };
            foreach (TcFactionEntry faction in manifest.Factions)
            {
                if (string.IsNullOrEmpty(faction.Id))
                    result.Errors.Add("Faction entry missing id");

                if (string.IsNullOrEmpty(faction.Replaces))
                    result.Errors.Add($"Faction '{faction.Id}' missing replaces field");

                if (!validArchetypes.Contains(faction.Archetype))
                    result.Warnings.Add($"Faction '{faction.Id}' has unknown archetype '{faction.Archetype}'");

                if (faction.Units.Count == 0)
                    result.Warnings.Add($"Faction '{faction.Id}' has no units defined");
            }

            // Faction IDs in replaces_vanilla should map to real faction entries
            foreach (string replacementId in manifest.ReplacesVanilla.Values)
            {
                bool found = manifest.Factions.Any(f => f.Id == replacementId);
                if (!found)
                    result.Errors.Add($"replaces_vanilla maps to '{replacementId}' but no faction with that id exists");
            }

            // Singleton enforcement reminder
            if (!manifest.Singleton)
                result.Warnings.Add("Singleton=false allows multiple total conversions — ensure compatibility is tested");

            result.IsValid = result.Errors.Count == 0;
            return result;
        }

        /// <summary>
        /// Checks completeness: does this TC cover all vanilla factions?
        /// </summary>
        public static List<string> GetUnreplacedFactions(TotalConversionManifest manifest)
        {
            return VanillaFactionIds
                .Where(vf => !manifest.ReplacesVanilla.ContainsKey(vf))
                .ToList();
        }
    }

    /// <summary>Result of a total conversion validation check.</summary>
    public sealed class TcValidationResult
    {
        /// <summary>
        /// The identifier of the total conversion manifest being validated.
        /// </summary>
        public string ManifestId { get; set; } = "";

        /// <summary>
        /// Whether the total conversion passed all validation checks.
        /// </summary>
        public bool IsValid { get; set; }

        /// <summary>
        /// List of validation errors that prevent the total conversion from loading.
        /// </summary>
        public List<string> Errors { get; set; } = new(); // public-mutable-ok: deserialization requires mutable List

        /// <summary>
        /// List of validation warnings that don't prevent loading but indicate issues.
        /// </summary>
        public List<string> Warnings { get; set; } = new(); // public-mutable-ok: deserialization requires mutable List
    }
}
