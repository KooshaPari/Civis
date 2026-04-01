using System;
using System.Collections.Generic;

namespace DINOForge.Tests.ECS
{
    /// <summary>
    /// Mock implementation of Unity.Entities.World for unit testing.
    /// Provides a minimal wrapper around MockEntityManager and system collection tracking.
    /// </summary>
    public class MockWorld : IDisposable
    {
        private readonly MockEntityManager _entityManager = new();
        private readonly List<object> _systems = new();

        /// <summary>
        /// Gets the underlying entity manager for this world.
        /// </summary>
        public MockEntityManager EntityManager => _entityManager;

        /// <summary>
        /// Gets the list of systems registered in this world.
        /// Simulates World.Systems (NoAllocReadOnlyCollection in real ECS).
        /// </summary>
        public IReadOnlyList<object> Systems => _systems.AsReadOnly();

        /// <summary>
        /// Registers a system instance in this world.
        /// </summary>
        public void AddSystem(object system)
        {
            if (system == null)
                throw new ArgumentNullException(nameof(system));

            _systems.Add(system);
        }

        /// <summary>
        /// Removes a system instance from this world.
        /// </summary>
        public bool RemoveSystem(object system)
        {
            return _systems.Remove(system);
        }

        /// <summary>
        /// Clears all entities and systems.
        /// </summary>
        public void Clear()
        {
            _entityManager.Clear();
            _systems.Clear();
        }

        /// <summary>
        /// Gets the total number of entities in this world.
        /// </summary>
        public int GetEntityCount() => _entityManager.GetEntityCount();

        public void Dispose()
        {
            Clear();
        }
    }
}
