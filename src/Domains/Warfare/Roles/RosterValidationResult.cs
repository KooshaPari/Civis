using System.Collections.Generic;

namespace DINOForge.Domains.Warfare.Roles
{
    /// <summary>
    /// Result of validating a faction's unit roster against the required role slots.
    /// </summary>
    public sealed class RosterValidationResult
    {
        /// <summary>
        /// True if all required roles are filled with valid unit references.
        /// </summary>
        public bool IsComplete { get; }

        /// <summary>
        /// Roles that have no unit assigned or reference a non-existent unit.
        /// </summary>
        public IReadOnlyList<string> MissingRoles { get; }

        /// <summary>
        /// Roles that are properly filled.
        /// </summary>
        public IReadOnlyList<string> FilledRoles { get; }

        /// <summary>
        /// Map from role name to the unit ID filling that role.
        /// Only contains entries for filled roles.
        /// </summary>
        public IReadOnlyDictionary<string, string> RoleToUnitMap { get; }

        /// <summary>
        /// Creates a new roster validation result.
        /// </summary>
        public RosterValidationResult(
            bool isComplete,
            IReadOnlyList<string> missingRoles,
            IReadOnlyList<string> filledRoles,
            IReadOnlyDictionary<string, string> roleToUnitMap)
        {
            IsComplete = isComplete;
            MissingRoles = missingRoles;
            FilledRoles = filledRoles;
            RoleToUnitMap = roleToUnitMap;
        }
    }
}
