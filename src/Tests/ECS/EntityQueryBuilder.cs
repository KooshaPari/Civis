using System;
using System.Collections.Generic;
using System.Linq;

namespace DINOForge.Tests.ECS
{
    /// <summary>
    /// Fluent builder for constructing MockEntityQuery instances.
    /// Supports method chaining to build complex component filters.
    /// </summary>
    internal class EntityQueryBuilder
    {
        private readonly MockComponentRegistry _registry;
        private readonly List<Type> _withAll = new();
        private readonly List<Type> _withoutAll = new();
        private readonly List<Type> _withoutAny = new();

        internal EntityQueryBuilder(MockComponentRegistry registry)
        {
            _registry = registry ?? throw new ArgumentNullException(nameof(registry));
        }

        /// <summary>
        /// Adds a required component type (entity must have all WithAll components).
        /// </summary>
        public EntityQueryBuilder WithAll(params Type[] componentTypes)
        {
            _withAll.AddRange(componentTypes ?? Enumerable.Empty<Type>());
            return this;
        }

        /// <summary>
        /// Adds a required component type using generic syntax.
        /// </summary>
        public EntityQueryBuilder WithAll<T>()
        {
            _withAll.Add(typeof(T));
            return this;
        }

        /// <summary>
        /// Adds a forbidden component type (entity must NOT have any WithoutAll components).
        /// </summary>
        public EntityQueryBuilder WithoutAll(params Type[] componentTypes)
        {
            _withoutAll.AddRange(componentTypes ?? Enumerable.Empty<Type>());
            return this;
        }

        /// <summary>
        /// Adds a forbidden component type using generic syntax.
        /// </summary>
        public EntityQueryBuilder WithoutAll<T>()
        {
            _withoutAll.Add(typeof(T));
            return this;
        }

        /// <summary>
        /// Adds an excluded component type (if any entity has a WithoutAny component, exclude it).
        /// </summary>
        public EntityQueryBuilder WithoutAny(params Type[] componentTypes)
        {
            _withoutAny.AddRange(componentTypes ?? Enumerable.Empty<Type>());
            return this;
        }

        /// <summary>
        /// Adds an excluded component type using generic syntax.
        /// </summary>
        public EntityQueryBuilder WithoutAny<T>()
        {
            _withoutAny.Add(typeof(T));
            return this;
        }

        /// <summary>
        /// Builds and returns the MockEntityQuery.
        /// </summary>
        public MockEntityQuery Build()
        {
            return new MockEntityQuery(_registry, _withAll, _withoutAll, _withoutAny);
        }

        /// <summary>
        /// Executes the query and returns all matching entities.
        /// Convenience method that calls Build().ToEntityArray().
        /// </summary>
        public IReadOnlyList<MockEntity> Execute()
        {
            return Build().ToEntityArray();
        }

        /// <summary>
        /// Executes the query and returns the count of matching entities.
        /// Convenience method that calls Build().CalculateEntityCount().
        /// </summary>
        public int Count()
        {
            return Build().CalculateEntityCount();
        }
    }
}
