using System;
using System.Collections.Generic;
using System.Linq;

namespace DINOForge.Tests.ECS
{
    /// <summary>
    /// Mock implementation of Unity.Entities.EntityQuery for testing.
    /// Supports WithAll, WithoutAll, and WithoutAny component filters.
    /// </summary>
    public class MockEntityQuery : IDisposable
    {
        private readonly MockComponentRegistry _registry;
        private readonly List<Type> _withAll;
        private readonly List<Type> _withoutAll;
        private readonly List<Type> _withoutAny;

        internal MockEntityQuery(
            MockComponentRegistry registry,
            IEnumerable<Type> withAll,
            IEnumerable<Type> withoutAll,
            IEnumerable<Type> withoutAny)
        {
            _registry = registry ?? throw new ArgumentNullException(nameof(registry));
            _withAll = new List<Type>(withAll ?? Enumerable.Empty<Type>());
            _withoutAll = new List<Type>(withoutAll ?? Enumerable.Empty<Type>());
            _withoutAny = new List<Type>(withoutAny ?? Enumerable.Empty<Type>());
        }

        /// <summary>
        /// Returns all entities matching this query's component filters.
        /// </summary>
        public IReadOnlyList<MockEntity> ToEntityArray()
        {
            var results = new List<MockEntity>();

            if (_withAll.Count == 0)
                return results;

            foreach (var entity in _registry.GetEntitiesWithAllComponents(_withAll))
            {
                // Check withoutAll constraint
                if (_withoutAll.Any(ct => _registry.HasComponent(entity, ct)))
                    continue;

                // Check withoutAny constraint
                if (_withoutAny.Any(ct => _registry.HasComponent(entity, ct)))
                    continue;

                results.Add(entity);
            }

            return results;
        }

        /// <summary>
        /// Returns the count of entities matching this query.
        /// </summary>
        public int CalculateEntityCount() => ToEntityArray().Count;

        /// <summary>
        /// Creates a new query by combining this query's filters with additional WithAll constraints.
        /// </summary>
        public MockEntityQuery AddWithAll(params Type[] additionalTypes)
        {
            var combinedWithAll = new List<Type>(_withAll);
            combinedWithAll.AddRange(additionalTypes ?? Enumerable.Empty<Type>());
            return new MockEntityQuery(_registry, combinedWithAll, _withoutAll, _withoutAny);
        }

        /// <summary>
        /// Creates a new query by combining this query's filters with additional WithoutAll constraints.
        /// </summary>
        public MockEntityQuery AddWithoutAll(params Type[] additionalTypes)
        {
            var combinedWithoutAll = new List<Type>(_withoutAll);
            combinedWithoutAll.AddRange(additionalTypes ?? Enumerable.Empty<Type>());
            return new MockEntityQuery(_registry, _withAll, combinedWithoutAll, _withoutAny);
        }

        public void Dispose()
        {
            // No resources to clean up in mock implementation
        }
    }
}
