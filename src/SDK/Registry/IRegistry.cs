using System.Collections.Generic;

namespace DINOForge.SDK.Registry
{
    /// <summary>
    /// Defines the contract for a typed content registry that supports registration,
    /// lookup, override, and conflict detection.
    /// </summary>
    /// <typeparam name="T">The type of content managed by this registry.</typeparam>
    public interface IRegistry<T>
    {
        /// <summary>
        /// Registers an entry with the given ID, source, and load order.
        /// </summary>
        /// <param name="id">Unique content identifier.</param>
        /// <param name="entry">The content data to register.</param>
        /// <param name="source">Which layer contributed this entry.</param>
        /// <param name="sourcePackId">The pack that owns this entry.</param>
        /// <param name="loadOrder">Intra-tier priority value (higher = loaded later).</param>
        void Register(string id, T entry, RegistrySource source, string sourcePackId, int loadOrder = 100);

        /// <summary>
        /// Retrieves the highest-priority entry for the given ID, or default if not found.
        /// </summary>
        /// <param name="id">The content identifier to look up.</param>
        /// <returns>The content data, or default if no entry exists.</returns>
        T? Get(string id);

        /// <summary>
        /// Returns true if an entry with the given ID exists in the registry.
        /// </summary>
        /// <param name="id">The content identifier to check.</param>
        /// <returns>True if the registry contains an entry for this ID.</returns>
        bool Contains(string id);

        /// <summary>
        /// Gets a snapshot of all entries, returning only the highest-priority entry per ID.
        /// </summary>
        IReadOnlyDictionary<string, RegistryEntry<T>> All { get; }

        /// <summary>
        /// Registers an override entry, logging the override for auditing purposes.
        /// </summary>
        /// <param name="id">Unique content identifier to override.</param>
        /// <param name="entry">The replacement content data.</param>
        /// <param name="source">Which layer contributed this override.</param>
        /// <param name="sourcePackId">The pack that owns this override.</param>
        /// <param name="loadOrder">Intra-tier priority value.</param>
        void Override(string id, T entry, RegistrySource source, string sourcePackId, int loadOrder = 100);

        /// <summary>
        /// Detects entries where multiple registrations share the same top priority,
        /// indicating an unresolvable conflict.
        /// </summary>
        /// <returns>A list of detected conflicts.</returns>
        IReadOnlyList<RegistryConflict> DetectConflicts();
    }
}
