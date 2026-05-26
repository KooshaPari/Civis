#nullable enable
using System.Collections.Generic;
using DINOForge.SDK.Models;

namespace DINOForge.SDK
{
    /// <summary>
    /// Public interface for faction management exposed by the Runtime to domain plugins.
    /// Allows packs and plugins to query faction state and register factions without
    /// directly depending on Runtime internals.
    /// </summary>
    public interface IFactionSystem
    {
        /// <summary>
        /// Read-only collection of registered faction IDs.
        /// </summary>
        IReadOnlyCollection<string> RegisteredFactions { get; }

        /// <summary>
        /// Check if a faction with the given ID is registered in the system.
        /// </summary>
        /// <param name="factionId">The faction ID to check.</param>
        /// <returns>True if the faction is registered, false otherwise.</returns>
        bool IsFactionRegistered(string factionId);

        /// <summary>
        /// Register a faction definition and initialize its runtime state.
        /// </summary>
        /// <param name="faction">The faction definition to register.</param>
        /// <param name="isEnemy">Whether this faction should be marked as enemy (true) or player-owned (false).</param>
        void RegisterFaction(FactionDefinition faction, bool isEnemy);
    }
}
