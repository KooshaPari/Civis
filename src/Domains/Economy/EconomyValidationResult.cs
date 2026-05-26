using System.Collections.Generic;

namespace DINOForge.Domains.Economy
{
    /// <summary>
    /// Result of validating a complete economy pack (profiles, trade routes, buildings).
    /// </summary>
    public sealed class EconomyValidationResult
    {
        /// <summary>
        /// The pack identifier that was validated.
        /// </summary>
        public string PackId { get; }

        /// <summary>
        /// True if all validations passed with no errors.
        /// </summary>
        public bool IsValid { get; }

        /// <summary>
        /// All validation errors found. Errors indicate problems that must be fixed.
        /// </summary>
        public IReadOnlyList<string> Errors { get; }

        /// <summary>
        /// All validation warnings (non-blocking). Warnings indicate potential issues
        /// that may affect gameplay balance but are not structurally invalid.
        /// </summary>
        public IReadOnlyList<string> Warnings { get; }

        /// <summary>
        /// Creates a new economy validation result.
        /// </summary>
        /// <param name="packId">The pack identifier that was validated.</param>
        /// <param name="isValid">Whether all validations passed.</param>
        /// <param name="errors">List of validation errors.</param>
        /// <param name="warnings">List of validation warnings.</param>
        public EconomyValidationResult(
            string packId,
            bool isValid,
            IReadOnlyList<string> errors,
            IReadOnlyList<string> warnings)
        {
            PackId = packId;
            IsValid = isValid;
            Errors = errors;
            Warnings = warnings;
        }
    }
}
