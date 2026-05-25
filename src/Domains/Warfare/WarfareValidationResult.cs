using System.Collections.Generic;
using DINOForge.Domains.Warfare.Roles;

namespace DINOForge.Domains.Warfare
{
    /// <summary>
    /// Result of validating a complete warfare pack (faction, units, doctrines, waves).
    /// </summary>
    public sealed class WarfareValidationResult
    {
        /// <summary>
        /// The pack identifier that was validated.
        /// </summary>
        public string PackId { get; }

        /// <summary>
        /// True if all validations passed.
        /// </summary>
        public bool IsValid { get; }

        /// <summary>
        /// All validation errors found.
        /// </summary>
        public IReadOnlyList<string> Errors { get; }

        /// <summary>
        /// All validation warnings (non-blocking).
        /// </summary>
        public IReadOnlyList<string> Warnings { get; }

        /// <summary>
        /// Per-faction roster validation results.
        /// </summary>
        public IReadOnlyDictionary<string, RosterValidationResult> RosterResults { get; }

        /// <summary>
        /// Creates a new warfare validation result.
        /// </summary>
        public WarfareValidationResult(
            string packId,
            bool isValid,
            IReadOnlyList<string> errors,
            IReadOnlyList<string> warnings,
            IReadOnlyDictionary<string, RosterValidationResult> rosterResults)
        {
            PackId = packId;
            IsValid = isValid;
            Errors = errors;
            Warnings = warnings;
            RosterResults = rosterResults;
        }
    }
}
