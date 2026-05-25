using System.Collections.Generic;

namespace DINOForge.Domains.Scenario
{
    /// <summary>
    /// Result of validating a complete scenario pack, containing per-scenario errors and warnings.
    /// </summary>
    public sealed class ScenarioValidationResult
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
        /// All validation errors found across all scenarios in the pack.
        /// </summary>
        public IReadOnlyList<string> Errors { get; }

        /// <summary>
        /// All validation warnings (non-blocking) found across all scenarios.
        /// </summary>
        public IReadOnlyList<string> Warnings { get; }

        /// <summary>
        /// Number of scenarios validated in the pack.
        /// </summary>
        public int ScenarioCount { get; }

        /// <summary>
        /// Creates a new scenario validation result.
        /// </summary>
        /// <param name="packId">The pack identifier that was validated.</param>
        /// <param name="isValid">Whether all validations passed.</param>
        /// <param name="errors">List of error messages.</param>
        /// <param name="warnings">List of warning messages.</param>
        /// <param name="scenarioCount">Number of scenarios validated.</param>
        public ScenarioValidationResult(
            string packId,
            bool isValid,
            IReadOnlyList<string> errors,
            IReadOnlyList<string> warnings,
            int scenarioCount)
        {
            PackId = packId;
            IsValid = isValid;
            Errors = errors;
            Warnings = warnings;
            ScenarioCount = scenarioCount;
        }
    }
}
