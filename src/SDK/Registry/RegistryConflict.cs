using System.Collections.Generic;

namespace DINOForge.SDK.Registry
{
    /// <summary>
    /// Describes a priority conflict where multiple registry entries share the same
    /// top priority for a given ID, making winner selection ambiguous.
    /// </summary>
    public sealed class RegistryConflict
    {
        /// <summary>The registry entry ID that has conflicting registrations.</summary>
        public string EntryId { get; }

        /// <summary>IDs of the packs whose entries are tied at the same priority.</summary>
        public IReadOnlyList<string> ConflictingPackIds { get; }

        /// <summary>Human-readable description of the conflict.</summary>
        public string Message { get; }

        /// <summary>
        /// Creates a new registry conflict record.
        /// </summary>
        /// <param name="entryId">The conflicting entry ID.</param>
        /// <param name="conflictingPackIds">Pack IDs involved in the conflict.</param>
        /// <param name="message">Descriptive message.</param>
        public RegistryConflict(string entryId, IReadOnlyList<string> conflictingPackIds, string message)
        {
            EntryId = entryId;
            ConflictingPackIds = conflictingPackIds;
            Message = message;
        }
    }
}
