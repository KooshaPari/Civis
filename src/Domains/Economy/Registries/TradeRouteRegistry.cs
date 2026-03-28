using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Economy.Models;

namespace DINOForge.Domains.Economy.Registries
{
    /// <summary>
    /// Registry of trade route definitions. Supports custom trade route registration and lookup.
    /// </summary>
    public class TradeRouteRegistry
    {
        private readonly Dictionary<string, TradeRouteDefinition> _routes =
            new Dictionary<string, TradeRouteDefinition>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// All registered trade routes.
        /// </summary>
        public IReadOnlyList<TradeRouteDefinition> All => _routes.Values.ToList().AsReadOnly();

        /// <summary>
        /// All enabled trade routes.
        /// </summary>
        public IReadOnlyList<TradeRouteDefinition> Enabled =>
            _routes.Values.Where(r => r.Enabled).ToList().AsReadOnly();

        /// <summary>
        /// Number of registered trade routes.
        /// </summary>
        public int Count => _routes.Count;

        /// <summary>
        /// Retrieve a trade route by its identifier.
        /// </summary>
        /// <param name="id">Trade route identifier.</param>
        /// <returns>The matching trade route definition.</returns>
        /// <exception cref="KeyNotFoundException">Thrown when no route with the given id exists.</exception>
        public TradeRouteDefinition GetRoute(string id)
        {
            if (_routes.TryGetValue(id, out TradeRouteDefinition? route))
                return route;

            throw new KeyNotFoundException($"No trade route registered with id '{id}'.");
        }

        /// <summary>
        /// Try to retrieve a trade route by its identifier.
        /// </summary>
        /// <param name="id">Trade route identifier.</param>
        /// <param name="route">The matching trade route definition, or null if not found.</param>
        /// <returns>True if found.</returns>
        public bool TryGetRoute(string id, out TradeRouteDefinition? route)
        {
            return _routes.TryGetValue(id, out route);
        }

        /// <summary>
        /// Check if a trade route with the given identifier is registered.
        /// </summary>
        /// <param name="id">Trade route identifier.</param>
        /// <returns>True if registered.</returns>
        public bool Contains(string id)
        {
            return _routes.ContainsKey(id);
        }

        /// <summary>
        /// Register a custom trade route.
        /// </summary>
        /// <param name="route">The trade route definition to register.</param>
        public void Register(TradeRouteDefinition route)
        {
            if (route == null) throw new ArgumentNullException(nameof(route));
            if (string.IsNullOrWhiteSpace(route.Id)) throw new ArgumentException("Trade route ID cannot be empty.", nameof(route));
            _routes[route.Id] = route;
        }

        /// <summary>
        /// Unregister a trade route by identifier.
        /// </summary>
        /// <param name="id">Trade route identifier.</param>
        /// <returns>True if a route was removed; false if not found.</returns>
        public bool Unregister(string id)
        {
            return _routes.Remove(id);
        }

        /// <summary>
        /// Get all routes for a specific source resource.
        /// </summary>
        /// <param name="sourceResource">The source resource ID.</param>
        /// <returns>All routes where the source matches.</returns>
        public IReadOnlyList<TradeRouteDefinition> GetRoutesBySource(string sourceResource)
        {
            return _routes.Values
                .Where(r => string.Equals(r.SourceResource, sourceResource, StringComparison.OrdinalIgnoreCase))
                .ToList()
                .AsReadOnly();
        }

        /// <summary>
        /// Get all routes for a specific target resource.
        /// </summary>
        /// <param name="targetResource">The target resource ID.</param>
        /// <returns>All routes where the target matches.</returns>
        public IReadOnlyList<TradeRouteDefinition> GetRoutesByTarget(string targetResource)
        {
            return _routes.Values
                .Where(r => string.Equals(r.TargetResource, targetResource, StringComparison.OrdinalIgnoreCase))
                .ToList()
                .AsReadOnly();
        }
    }
}
