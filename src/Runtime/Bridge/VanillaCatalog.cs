using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Unity.Collections;
using Unity.Entities;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Information about a vanilla DINO entity archetype discovered at runtime.
    /// </summary>
    public sealed class VanillaEntityInfo
    {
        /// <summary>
        /// Inferred DINOForge registry ID (e.g. "vanilla:melee_unit", "vanilla:farm").
        /// </summary>
        public string InferredId { get; }

        /// <summary>
        /// Full type names of all ECS components on entities of this archetype.
        /// </summary>
        public string[] ComponentTypes { get; }

        /// <summary>
        /// Number of entities with this archetype found during the scan.
        /// </summary>
        public int EntityCount { get; }

        /// <summary>
        /// Primary category: "unit", "building", "resource", "projectile", "other".
        /// </summary>
        public string Category { get; }

        public VanillaEntityInfo(string inferredId, string[] componentTypes, int entityCount, string category)
        {
            InferredId = inferredId;
            ComponentTypes = componentTypes;
            EntityCount = entityCount;
            Category = category;
        }

        public override string ToString() => $"{InferredId} ({EntityCount}x, {ComponentTypes.Length} components)";
    }

    /// <summary>
    /// Maps vanilla DINO entities to DINOForge registry identifiers.
    /// Built at runtime by scanning the ECS world and classifying entities
    /// based on their component signatures.
    ///
    /// Manual testing:
    ///   1. Load a game and let entities spawn (wait for gameplay to start)
    ///   2. Call Build(entityManager) from Plugin or DumpSystem
    ///   3. Check Units / Buildings / Projectiles lists
    ///   4. Verify against entity dump data in BepInEx/dinoforge_dumps/
    /// </summary>
    public class VanillaCatalog
    {
        private readonly List<VanillaEntityInfo> _units = new List<VanillaEntityInfo>();
        private readonly List<VanillaEntityInfo> _buildings = new List<VanillaEntityInfo>();
        private readonly List<VanillaEntityInfo> _projectiles = new List<VanillaEntityInfo>();
        private readonly List<VanillaEntityInfo> _other = new List<VanillaEntityInfo>();

        private bool _isBuilt;

        /// <summary>All discovered vanilla unit archetypes.</summary>
        public IReadOnlyList<VanillaEntityInfo> Units => _units;

        /// <summary>All discovered vanilla building archetypes.</summary>
        public IReadOnlyList<VanillaEntityInfo> Buildings => _buildings;

        /// <summary>All discovered vanilla projectile archetypes.</summary>
        public IReadOnlyList<VanillaEntityInfo> Projectiles => _projectiles;

        /// <summary>Entities that don't fall into the above categories.</summary>
        public IReadOnlyList<VanillaEntityInfo> Other => _other;

        /// <summary>Whether the catalog has been built.</summary>
        public bool IsBuilt => _isBuilt;

        /// <summary>
        /// Scan the current ECS world and build the vanilla entity catalog.
        /// Groups entities by their component archetype signature and classifies
        /// them based on the presence of known marker components.
        /// </summary>
        /// <param name="em">The EntityManager to scan.</param>
        public void Build(EntityManager em)
        {
            _units.Clear();
            _buildings.Clear();
            _projectiles.Clear();
            _other.Clear();

            WriteDebug("VanillaCatalog.Build starting scan");

            NativeArray<Entity> allEntities = em.GetAllEntities(Allocator.Temp);
            WriteDebug($"VanillaCatalog: scanning {allEntities.Length} entities");

            // Skip if no entities (e.g., still on MainMenu scene)
            if (allEntities.Length == 0)
            {
                WriteDebug("VanillaCatalog.Build: No entities found. Skipping catalog build.");
                allEntities.Dispose();
                return;
            }

            // Group entities by archetype (component signature)
            Dictionary<string, ArchetypeGroup> archetypeGroups =
                new Dictionary<string, ArchetypeGroup>();

            foreach (Entity entity in allEntities)
            {
                try
                {
                    NativeArray<ComponentType> types = em.GetComponentTypes(entity, Allocator.Temp);

                    string[] typeNames = new string[types.Length];
                    for (int i = 0; i < types.Length; i++)
                    {
                        Type? managedType = types[i].GetManagedType();
                        typeNames[i] = managedType?.FullName ?? $"Unknown({types[i].TypeIndex})";
                    }

                    Array.Sort(typeNames, StringComparer.Ordinal);
                    string signature = string.Join("|", typeNames);

                    if (!archetypeGroups.TryGetValue(signature, out ArchetypeGroup? group))
                    {
                        group = new ArchetypeGroup(typeNames);
                        archetypeGroups[signature] = group;
                    }
                    group.Count++;

                    types.Dispose();
                }
                catch
                {
                    // Skip entities we can't inspect
                }
            }

            allEntities.Dispose();

            WriteDebug($"VanillaCatalog: found {archetypeGroups.Count} unique archetypes");

            // Classify each archetype group
            foreach (ArchetypeGroup group in archetypeGroups.Values)
            {
                VanillaEntityInfo info = ClassifyArchetype(group);

                switch (info.Category)
                {
                    case "unit":
                        _units.Add(info);
                        break;
                    case "building":
                        _buildings.Add(info);
                        break;
                    case "projectile":
                        _projectiles.Add(info);
                        break;
                    default:
                        _other.Add(info);
                        break;
                }
            }

            // Sort by entity count descending
            _units.Sort((a, b) => b.EntityCount.CompareTo(a.EntityCount));
            _buildings.Sort((a, b) => b.EntityCount.CompareTo(a.EntityCount));
            _projectiles.Sort((a, b) => b.EntityCount.CompareTo(a.EntityCount));

            _isBuilt = true;

            WriteDebug($"VanillaCatalog built: {_units.Count} unit types, " +
                        $"{_buildings.Count} building types, " +
                        $"{_projectiles.Count} projectile types, " +
                        $"{_other.Count} other");
        }

        /// <summary>
        /// Look up a vanilla entity info by its inferred ID.
        /// </summary>
        public VanillaEntityInfo? FindById(string inferredId)
        {
            foreach (VanillaEntityInfo info in _units)
                if (string.Equals(info.InferredId, inferredId, StringComparison.OrdinalIgnoreCase))
                    return info;
            foreach (VanillaEntityInfo info in _buildings)
                if (string.Equals(info.InferredId, inferredId, StringComparison.OrdinalIgnoreCase))
                    return info;
            foreach (VanillaEntityInfo info in _projectiles)
                if (string.Equals(info.InferredId, inferredId, StringComparison.OrdinalIgnoreCase))
                    return info;
            foreach (VanillaEntityInfo info in _other)
                if (string.Equals(info.InferredId, inferredId, StringComparison.OrdinalIgnoreCase))
                    return info;
            return null;
        }

        private VanillaEntityInfo ClassifyArchetype(ArchetypeGroup group)
        {
            HashSet<string> types = new HashSet<string>(group.TypeNames, StringComparer.Ordinal);

            // Classify based on marker components
            bool hasUnit = types.Contains("Components.Unit");
            bool hasBuilding = types.Contains("Components.BuildingBase");
            bool hasEnemy = types.Contains("Components.Enemy");

            if (hasUnit)
            {
                string unitType = InferUnitType(types);
                string factionPrefix = hasEnemy ? "enemy" : "player";
                string inferredId = $"vanilla:{factionPrefix}_{unitType}";
                return new VanillaEntityInfo(inferredId, group.TypeNames, group.Count, "unit");
            }

            if (hasBuilding)
            {
                string buildingType = InferBuildingType(types);
                string inferredId = $"vanilla:{buildingType}";
                return new VanillaEntityInfo(inferredId, group.TypeNames, group.Count, "building");
            }

            // Detect projectile archetypes via ProjectileDataBase or ProjectileFlyData.
            // Entity dumps confirm: Components.ProjectileDataBase (BlobAssetReference<ProjectileData>)
            // and Components.RawComponents.ProjectileFlyData (runtime fields: weaponType, damage, etc.)
            bool hasProjectile = types.Contains("Components.ProjectileDataBase") ||
                                 types.Contains("Components.RawComponents.ProjectileFlyData");
            if (hasProjectile)
            {
                string projectileType = InferProjectileType(types);
                string inferredId = $"vanilla:{projectileType}";
                return new VanillaEntityInfo(inferredId, group.TypeNames, group.Count, "projectile");
            }

            // Classify as resource entity if it has resource components
            if (types.Any(t => t.Contains("FoodSource") || t.Contains("IronSource") ||
                               t.Contains("StoneSource")))
            {
                string inferredId = $"vanilla:resource_node_{group.Count}";
                return new VanillaEntityInfo(inferredId, group.TypeNames, group.Count, "resource");
            }

            // Default: unknown
            string defaultId = $"vanilla:unknown_{Math.Abs(string.Join("", group.TypeNames).GetHashCode()) % 10000:D4}";
            return new VanillaEntityInfo(defaultId, group.TypeNames, group.Count, "other");
        }

        private static string InferUnitType(HashSet<string> types)
        {
            if (types.Contains("Components.MeleeUnit")) return "melee_unit";
            if (types.Contains("Components.RangeUnit")) return "ranged_unit";
            if (types.Contains("Components.CavalryUnit")) return "cavalry_unit";
            if (types.Contains("Components.SiegeUnit")) return "siege_unit";
            if (types.Contains("Components.Archer")) return "archer_unit";
            if (types.Contains("Components.CastOnlyUnit")) return "caster_unit";
            return "generic_unit";
        }

        private static string InferProjectileType(HashSet<string> types)
        {
            // ProjectileMultiHitBuffer indicates AoE/piercing projectiles
            if (types.Contains("Components.ProjectileMultiHitBuffer"))
                return "projectile_aoe";
            if (types.Contains("Components.ProjectileDataBase"))
                return "projectile_standard";
            return "projectile_generic";
        }

        private static string InferBuildingType(HashSet<string> types)
        {
            if (types.Contains("Components.Barraks")) return "barracks";
            if (types.Contains("Components.Farm")) return "farm";
            if (types.Contains("Components.House")) return "house";
            if (types.Contains("Components.Granary")) return "granary";
            if (types.Contains("Components.Hospital")) return "hospital";
            if (types.Contains("Components.ForesterHouse")) return "forester_house";
            if (types.Contains("Components.StoneCutter")) return "stone_cutter";
            if (types.Contains("Components.IronMine")) return "iron_mine";
            if (types.Contains("Components.InfiniteIronMine")) return "infinite_iron_mine";
            if (types.Contains("Components.SoulMine")) return "soul_mine";
            if (types.Contains("Components.BuilderHouse")) return "builder_house";
            if (types.Contains("Components.EngineerGuild")) return "engineer_guild";
            if (types.Contains("Components.GateBase")) return "gate";
            return "generic_building";
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = Path.Combine(
                    BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.Now}] {msg}\n");
            }
            catch { }
        }

        /// <summary>
        /// Internal grouping of entities by their component archetype.
        /// </summary>
        private class ArchetypeGroup
        {
            public string[] TypeNames { get; }
            public int Count { get; set; }

            public ArchetypeGroup(string[] typeNames)
            {
                TypeNames = typeNames;
            }
        }
    }
}
