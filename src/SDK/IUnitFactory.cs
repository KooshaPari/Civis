namespace DINOForge.SDK
{
    /// <summary>
    /// Factory interface for spawning units at runtime.
    /// Implementations should be registered with the DINOForge SDK to enable
    /// spawning of pack-defined units during gameplay.
    ///
    /// Usage:
    ///   1. Domain plugins (like Warfare) implement this interface in their spawner system
    ///   2. The SDK maintains a singleton instance available to pack loaders and the UI
    ///   3. Packs call RequestSpawn with a unit definition ID and world position
    ///   4. The spawner queues the request and processes it on the next ECS frame
    /// </summary>
    public interface IUnitFactory
    {
        /// <summary>
        /// Check whether this factory can spawn a unit by its definition ID.
        /// </summary>
        /// <param name="unitDefinitionId">The unit ID from UnitDefinition.Id.</param>
        /// <returns>True if the unit can be spawned, false otherwise.</returns>
        bool CanSpawn(string unitDefinitionId);

        /// <summary>
        /// Request a unit spawn at the given position.
        /// The spawner queues the request and may process it asynchronously.
        /// Spawning is not guaranteed (resource constraints, game state, etc).
        /// </summary>
        /// <param name="unitDefinitionId">The unit ID from UnitDefinition.Id.</param>
        /// <param name="x">World X coordinate.</param>
        /// <param name="z">World Z coordinate.</param>
        void RequestSpawn(string unitDefinitionId, float x, float z);
    }
}
