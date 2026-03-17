using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using Unity.Entities;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Provides ECS entity queries mapped to DINOForge content types.
    /// All queries use reflection-based ComponentType lookup so that the bridge
    /// works even if exact game struct layouts change between DINO versions.
    ///
    /// Manual testing:
    ///   1. Load a game with units and buildings spawned
    ///   2. Call GetPlayerUnits / GetBuildings from the debug console or DumpSystem
    ///   3. Verify entity counts match the debug overlay archetype counts
    /// </summary>
    public static class EntityQueries
    {
        // Cache resolved ComponentTypes to avoid repeated reflection
        private static readonly Dictionary<string, ComponentType?> _typeCache =
            new Dictionary<string, ComponentType?>(StringComparer.Ordinal);

        /// <summary>
        /// Get an EntityQuery for all player-owned units (have Unit tag, lack Enemy tag).
        /// </summary>
        public static EntityQuery GetPlayerUnits(EntityManager em)
        {
            ComponentType? unitType = ResolveComponentType("Components.Unit");
            ComponentType? enemyType = ResolveComponentType("Components.Enemy");

            if (unitType == null)
                throw new InvalidOperationException(
                    "Cannot resolve Components.Unit — is DNO.Main.dll loaded?");

            // DINO stores all live entities (units, buildings) as ECS Prefab entities.
            // Must include IncludePrefab option or queries return 0 results.
            EntityQueryDesc desc = new EntityQueryDesc
            {
                All = new[] { ComponentType.ReadOnly(unitType.Value.TypeIndex) },
                None = enemyType.HasValue
                    ? new[] { ComponentType.ReadOnly(enemyType.Value.TypeIndex) }
                    : Array.Empty<ComponentType>(),
                Options = EntityQueryOptions.IncludePrefab
            };

            return em.CreateEntityQuery(desc);
        }

        /// <summary>
        /// Get an EntityQuery for all enemy units (have both Unit and Enemy tags).
        /// </summary>
        public static EntityQuery GetEnemyUnits(EntityManager em)
        {
            ComponentType? unitType = ResolveComponentType("Components.Unit");
            ComponentType? enemyType = ResolveComponentType("Components.Enemy");

            if (unitType == null || enemyType == null)
                throw new InvalidOperationException(
                    "Cannot resolve Components.Unit or Components.Enemy — is DNO.Main.dll loaded?");

            EntityQueryDesc desc = new EntityQueryDesc
            {
                All = new[]
                {
                    ComponentType.ReadOnly(unitType.Value.TypeIndex),
                    ComponentType.ReadOnly(enemyType.Value.TypeIndex)
                },
                Options = EntityQueryOptions.IncludePrefab
            };

            return em.CreateEntityQuery(desc);
        }

        /// <summary>
        /// Get an EntityQuery for all buildings (have BuildingBase component).
        /// </summary>
        public static EntityQuery GetBuildings(EntityManager em)
        {
            ComponentType? buildingType = ResolveComponentType("Components.BuildingBase");

            if (buildingType == null)
                throw new InvalidOperationException(
                    "Cannot resolve Components.BuildingBase — is DNO.Main.dll loaded?");

            EntityQueryDesc desc = new EntityQueryDesc
            {
                All = new[] { ComponentType.ReadOnly(buildingType.Value.TypeIndex) },
                Options = EntityQueryOptions.IncludePrefab
            };

            return em.CreateEntityQuery(desc);
        }

        /// <summary>
        /// Get an EntityQuery for units of a specific class.
        /// Valid class names: "melee", "ranged", "cavalry", "archer".
        /// </summary>
        /// <param name="em">The EntityManager to query against.</param>
        /// <param name="unitClass">Unit class name (case-insensitive).</param>
        /// <exception cref="ArgumentException">If unitClass is not recognized.</exception>
        public static EntityQuery GetUnitsByClass(EntityManager em, string unitClass)
        {
            string ecsTypeName;
            switch (unitClass.ToLowerInvariant())
            {
                case "melee":
                    ecsTypeName = "Components.MeleeUnit";
                    break;
                case "ranged":
                case "range":
                    ecsTypeName = "Components.RangeUnit";
                    break;
                case "cavalry":
                    ecsTypeName = "Components.CavalryUnit";
                    break;
                case "archer":
                    ecsTypeName = "Components.Archer";
                    break;
                default:
                    throw new ArgumentException(
                        $"Unknown unit class '{unitClass}'. " +
                        "Valid values: melee, ranged, cavalry, archer.",
                        nameof(unitClass));
            }

            ComponentType? classType = ResolveComponentType(ecsTypeName);
            ComponentType? unitType = ResolveComponentType("Components.Unit");

            if (classType == null || unitType == null)
                throw new InvalidOperationException(
                    $"Cannot resolve {ecsTypeName} or Components.Unit — is DNO.Main.dll loaded?");

            EntityQueryDesc desc = new EntityQueryDesc
            {
                All = new[]
                {
                    ComponentType.ReadOnly(unitType.Value.TypeIndex),
                    ComponentType.ReadOnly(classType.Value.TypeIndex)
                },
                Options = EntityQueryOptions.IncludePrefab
            };

            return em.CreateEntityQuery(desc);
        }

        /// <summary>
        /// Get an EntityQuery for buildings of a specific type.
        /// </summary>
        /// <param name="em">The EntityManager to query against.</param>
        /// <param name="buildingTypeName">
        /// Full ECS component type name (e.g. "Components.Barraks", "Components.Farm").
        /// </param>
        public static EntityQuery GetBuildingsByType(EntityManager em, string buildingTypeName)
        {
            ComponentType? buildingBase = ResolveComponentType("Components.BuildingBase");
            ComponentType? specificType = ResolveComponentType(buildingTypeName);

            if (buildingBase == null || specificType == null)
                throw new InvalidOperationException(
                    $"Cannot resolve Components.BuildingBase or {buildingTypeName} — is DNO.Main.dll loaded?");

            EntityQueryDesc desc = new EntityQueryDesc
            {
                All = new[]
                {
                    ComponentType.ReadOnly(buildingBase.Value.TypeIndex),
                    ComponentType.ReadOnly(specificType.Value.TypeIndex)
                },
                Options = EntityQueryOptions.IncludePrefab
            };

            return em.CreateEntityQuery(desc);
        }

        /// <summary>
        /// Get an EntityQuery for units of a specific archetype component type.
        /// Used by PackUnitSpawner to find vanilla template entities for cloning.
        /// Returns a query, or throws an InvalidOperationException if the component type cannot be resolved.
        /// </summary>
        /// <param name="em">The EntityManager to query against.</param>
        /// <param name="componentTypeName">
        /// Full ECS component type name identifying the archetype
        /// (e.g. "Components.MeleeUnit", "Components.RangeUnit", "Components.CavalryUnit", "Components.SiegeUnit", "Components.Archer").
        /// </param>
        /// <returns>An EntityQuery matching units with that archetype.</returns>
        /// <exception cref="InvalidOperationException">If the component type cannot be resolved.</exception>
        public static EntityQuery GetUnitsByComponentType(EntityManager em, string componentTypeName)
        {
            ComponentType? archetype = ResolveComponentType(componentTypeName);
            if (archetype == null)
                throw new InvalidOperationException(
                    $"Cannot resolve archetype component type '{componentTypeName}' — is DNO.Main.dll loaded?");

            ComponentType? unitType = ResolveComponentType("Components.Unit");
            if (unitType == null)
                throw new InvalidOperationException(
                    "Cannot resolve Components.Unit — is DNO.Main.dll loaded?");

            EntityQueryDesc desc = new EntityQueryDesc
            {
                All = new[]
                {
                    ComponentType.ReadOnly(unitType.Value.TypeIndex),
                    ComponentType.ReadOnly(archetype.Value.TypeIndex)
                },
                Options = EntityQueryOptions.IncludePrefab
            };

            return em.CreateEntityQuery(desc);
        }

        /// <summary>
        /// Resolve a DINO ECS component type by full name from loaded assemblies,
        /// then convert it to a Unity.Entities.ComponentType.
        /// Results are cached for the lifetime of the AppDomain.
        /// </summary>
        /// <param name="fullTypeName">Full CLR type name (e.g. "Components.Health").</param>
        /// <returns>The ComponentType, or null if the type cannot be found.</returns>
        public static ComponentType? ResolveComponentType(string fullTypeName)
        {
            if (_typeCache.TryGetValue(fullTypeName, out ComponentType? cached))
                return cached;

            Type? clrType = null;
            foreach (Assembly assembly in AppDomain.CurrentDomain.GetAssemblies())
            {
                try
                {
                    clrType = assembly.GetType(fullTypeName, throwOnError: false);
                    if (clrType != null) break;
                }
                catch
                {
                    // Assembly may fail to enumerate types — skip
                }
            }

            if (clrType == null)
            {
                _typeCache[fullTypeName] = null;
                return null;
            }

            try
            {
                ComponentType ct = ComponentType.ReadOnly(clrType);
                _typeCache[fullTypeName] = ct;
                return ct;
            }
            catch (Exception)
            {
                // Type exists but is not a valid ECS component
                _typeCache[fullTypeName] = null;
                return null;
            }
        }

        /// <summary>
        /// Get an EntityQuery for all entities that carry a Unity.Rendering.RenderMesh
        /// shared component. Used by <see cref="AssetSwapSystem"/> to detect when the
        /// game's presentation layer has populated the ECS world with renderable geometry.
        /// Returns null if the RenderMesh type cannot be resolved (renderer not loaded yet).
        /// </summary>
        public static EntityQuery? GetRenderMeshEntities(EntityManager em)
        {
            ComponentType? renderMeshType = ResolveComponentType("Unity.Rendering.RenderMesh");
            if (renderMeshType == null)
                return null;

            return em.CreateEntityQuery(
                new EntityQueryDesc { All = new[] { renderMeshType.Value } });
        }

        /// <summary>
        /// Clear the component type cache. Useful if assemblies are loaded/unloaded at runtime.
        /// </summary>
        public static void ClearCache()
        {
            _typeCache.Clear();
        }
    }
}
