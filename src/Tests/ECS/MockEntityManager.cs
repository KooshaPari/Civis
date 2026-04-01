using System;
using System.Collections.Generic;
using System.Linq;

namespace DINOForge.Tests.ECS
{
    /// <summary>
    /// Mock implementation of Unity.Entities.EntityManager for unit testing ECS systems.
    /// Provides minimal support for entity creation, component operations, and queries.
    /// </summary>
    public class MockEntityManager
    {
        private readonly MockComponentRegistry _registry = new();
        private int _nextEntityIndex;

        /// <summary>
        /// Creates a new entity and returns its identifier.
        /// </summary>
        public MockEntity CreateEntity()
        {
            var entity = new MockEntity(_nextEntityIndex++, version: 0);
            _registry.AddComponent(entity, typeof(Entity)); // Implicit base component
            return entity;
        }

        /// <summary>
        /// Creates multiple entities and returns their identifiers.
        /// </summary>
        public IReadOnlyList<MockEntity> CreateEntity(int count)
        {
            var entities = new List<MockEntity>(count);
            for (int i = 0; i < count; i++)
            {
                entities.Add(CreateEntity());
            }
            return entities;
        }

        /// <summary>
        /// Adds a component of the specified type to an entity.
        /// </summary>
        public void AddComponent<T>(MockEntity entity) where T : new()
        {
            _registry.AddComponent(entity, typeof(T));
            _registry.SetComponentData(entity, typeof(T), new T());
        }

        /// <summary>
        /// Adds a component of the specified type to an entity with provided data.
        /// </summary>
        public void AddComponent<T>(MockEntity entity, T componentData)
        {
            _registry.AddComponent(entity, typeof(T));
            _registry.SetComponentData(entity, typeof(T), componentData);
        }

        /// <summary>
        /// Removes a component of the specified type from an entity.
        /// </summary>
        public void RemoveComponent<T>(MockEntity entity)
        {
            _registry.RemoveComponent(entity, typeof(T));
        }

        /// <summary>
        /// Checks if an entity has a specific component type.
        /// </summary>
        public bool HasComponent<T>(MockEntity entity)
        {
            return _registry.HasComponent(entity, typeof(T));
        }

        /// <summary>
        /// Gets component data from an entity (read-only).
        /// </summary>
        public T GetComponentData<T>(MockEntity entity)
        {
            var data = _registry.GetComponentData(entity, typeof(T));
            if (data is not T typedData)
                throw new InvalidOperationException(
                    $"Component {typeof(T).Name} on entity {entity} is not of expected type");
            return typedData;
        }

        /// <summary>
        /// Sets component data on an entity.
        /// </summary>
        public void SetComponentData<T>(MockEntity entity, T componentData)
        {
            _registry.SetComponentData(entity, typeof(T), componentData);
        }

        /// <summary>
        /// Creates an entity query using the fluent builder pattern.
        /// </summary>
        internal EntityQueryBuilder CreateEntityQueryBuilder()
        {
            return new EntityQueryBuilder(_registry);
        }

        /// <summary>
        /// Creates an entity query from component types (simulating Unity.Entities.EntityQueryDesc).
        /// All provided types are treated as required (WithAll).
        /// </summary>
        public MockEntityQuery CreateEntityQuery(params Type[]? componentTypes)
        {
            return new MockEntityQuery(_registry, componentTypes ?? Enumerable.Empty<Type>(), Enumerable.Empty<Type>(), Enumerable.Empty<Type>());
        }

        /// <summary>
        /// Creates an entity query with explicit WithAll and WithoutAll constraints.
        /// </summary>
        public MockEntityQuery CreateEntityQuery(
            IEnumerable<Type> withAll,
            IEnumerable<Type> withoutAll = null)
        {
            return new MockEntityQuery(
                _registry,
                withAll ?? Enumerable.Empty<Type>(),
                withoutAll ?? Enumerable.Empty<Type>(),
                Enumerable.Empty<Type>());
        }

        /// <summary>
        /// Returns the underlying component registry (internal use for testing).
        /// </summary>
        internal MockComponentRegistry GetRegistry() => _registry;

        /// <summary>
        /// Clears all entities and components (useful between tests).
        /// </summary>
        public void Clear()
        {
            _registry.Clear();
            _nextEntityIndex = 0;
        }

        /// <summary>
        /// Returns the total number of created entities.
        /// </summary>
        public int GetEntityCount() => _registry.GetEntityCount();
    }

    /// <summary>
    /// Marker type representing an entity in ECS.
    /// </summary>
    internal class Entity { }
}
