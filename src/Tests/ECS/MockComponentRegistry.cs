using System;
using System.Collections.Generic;

namespace DINOForge.Tests.ECS
{
    /// <summary>
    /// Internal registry that tracks components on entities.
    /// Used by MockEntityManager to store and retrieve component data.
    /// </summary>
    internal class MockComponentRegistry
    {
        private readonly Dictionary<MockEntity, HashSet<Type>> _entityComponents = new();
        private readonly Dictionary<(MockEntity, Type), object?> _componentData = new();

        public void AddComponent(MockEntity entity, Type componentType)
        {
            if (!_entityComponents.ContainsKey(entity))
                _entityComponents[entity] = new HashSet<Type>();

            _entityComponents[entity].Add(componentType);
        }

        public void RemoveComponent(MockEntity entity, Type componentType)
        {
            if (_entityComponents.TryGetValue(entity, out var components))
            {
                components.Remove(componentType);
                _componentData.Remove((entity, componentType));
            }
        }

        public bool HasComponent(MockEntity entity, Type componentType)
        {
            return _entityComponents.TryGetValue(entity, out var components)
                && components.Contains(componentType);
        }

        public IReadOnlyCollection<Type> GetComponentsForEntity(MockEntity entity)
        {
            if (_entityComponents.TryGetValue(entity, out var components))
                return components;
            return new HashSet<Type>();
        }

        public void SetComponentData(MockEntity entity, Type componentType, object? data)
        {
            if (!HasComponent(entity, componentType))
                throw new InvalidOperationException(
                    $"Entity {entity} does not have component {componentType.Name}");

            _componentData[(entity, componentType)] = data;
        }

        public object? GetComponentData(MockEntity entity, Type componentType)
        {
            if (!HasComponent(entity, componentType))
                throw new InvalidOperationException(
                    $"Entity {entity} does not have component {componentType.Name}");

            return _componentData.TryGetValue((entity, componentType), out var data) ? data : null;
        }

        public IEnumerable<MockEntity> GetEntitiesWithComponent(Type componentType)
        {
            foreach (var kvp in _entityComponents)
            {
                if (kvp.Value.Contains(componentType))
                    yield return kvp.Key;
            }
        }

        public IEnumerable<MockEntity> GetEntitiesWithAllComponents(IEnumerable<Type> componentTypes)
        {
            var componentSet = new HashSet<Type>(componentTypes);

            foreach (var kvp in _entityComponents)
            {
                if (componentSet.IsSubsetOf(kvp.Value))
                    yield return kvp.Key;
            }
        }

        public IEnumerable<MockEntity> GetEntitiesWithoutComponent(Type componentType)
        {
            foreach (var kvp in _entityComponents)
            {
                if (!kvp.Value.Contains(componentType))
                    yield return kvp.Key;
            }
        }

        public void Clear()
        {
            _entityComponents.Clear();
            _componentData.Clear();
        }

        public int GetEntityCount() => _entityComponents.Count;
    }
}
