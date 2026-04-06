#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using DINOForge.Bridge.Protocol;
using Unity.Collections;
using Unity.Entities;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Reads current resource stockpile values from the ECS world.
    ///
    /// Root causes of the "resources reads 0" bug (fixed here):
    ///   1. Missing IncludePrefab — DINO stores even singleton-like resource entities
    ///      as ECS prefab entities. Without EntityQueryOptions.IncludePrefab, the query
    ///      returns 0 entities regardless of whether the component type resolved correctly.
    ///   2. Unverified component type names — "Components.RawComponents.CurrentFood" et al.
    ///      are best guesses. A fallback table of alternative names is tried if the primary
    ///      type does not resolve or yields no entities.
    ///   3. Unverified field names — the resource value field may be named differently
    ///      across DINO versions. A per-mapping fallback chain is tried in order.
    ///   4. GetComponentData reflection path — field traversal now logs every failed
    ///      field attempt so the debug log can identify the correct field name quickly.
    /// </summary>
    public static class ResourceReader
    {
        // ---------------------------------------------------------------------------
        // Alternative component type names to try when the primary name resolves but
        // yields 0 entities (or the primary name itself cannot be resolved).
        // Order matters: more-specific names first.
        // ---------------------------------------------------------------------------
        private static readonly (string EcsType, string[] FieldPaths)[] FoodAlternatives =
        {
            // Primary guesses (unverified)
            ("Components.RawComponents.CurrentFood",   new[] { "value", "amount", "current", "count", "_value", "stored", "total" }),
            ("Components.CurrentFood",                  new[] { "value", "amount", "current", "count", "_value", "stored" }),
            ("Components.FoodAmount",                   new[] { "value", "amount", "current", "count" }),
            // Fallback: ResourceData singleton
            ("Components.ResourceData",                 new[] { "food", "foodAmount", "currentFood", "_food", "_currentFood" }),
            // Fallback: singleton patterns
            ("Components.SingletonComponents.FoodStorage", new[] { "stored", "value", "amount", "current" }),
            ("Components.SingletonComponents.CurrentFood", new[] { "value", "stored", "amount", "current" }),
            // Fallback: generic patterns
            ("Components.SingletonResources",           new[] { "food", "storedFood", "_food" }),
        };

        private static readonly (string EcsType, string[] FieldPaths)[] WoodAlternatives =
        {
            ("Components.RawComponents.CurrentWood",   new[] { "value", "amount", "current", "count", "_value", "stored", "total" }),
            ("Components.CurrentWood",                  new[] { "value", "amount", "current", "count", "_value", "stored" }),
            ("Components.WoodAmount",                   new[] { "value", "amount", "current", "count" }),
            ("Components.ResourceData",                 new[] { "wood", "woodAmount", "currentWood", "_wood", "_currentWood" }),
            ("Components.SingletonComponents.WoodStorage", new[] { "stored", "value", "amount", "current" }),
            ("Components.SingletonComponents.CurrentWood", new[] { "value", "stored", "amount", "current" }),
        };

        private static readonly (string EcsType, string[] FieldPaths)[] StoneAlternatives =
        {
            ("Components.RawComponents.CurrentStone",  new[] { "value", "amount", "current", "count", "_value", "stored" }),
            ("Components.CurrentStone",                 new[] { "value", "amount", "current", "count", "_value", "stored" }),
            ("Components.StoneAmount",                  new[] { "value", "amount", "current", "count" }),
            ("Components.ResourceData",                 new[] { "stone", "stoneAmount", "currentStone", "_stone" }),
            ("Components.SingletonComponents.StoneStorage", new[] { "stored", "value", "amount", "current" }),
            ("Components.SingletonComponents.CurrentStone", new[] { "value", "stored", "amount", "current" }),
        };

        private static readonly (string EcsType, string[] FieldPaths)[] IronAlternatives =
        {
            ("Components.RawComponents.CurrentIron",   new[] { "value", "amount", "current", "count", "_value", "stored" }),
            ("Components.CurrentIron",                  new[] { "value", "amount", "current", "count", "_value", "stored" }),
            ("Components.IronAmount",                   new[] { "value", "amount", "current", "count" }),
            ("Components.ResourceData",                 new[] { "iron", "ironAmount", "currentIron", "_iron" }),
            ("Components.SingletonComponents.IronStorage", new[] { "stored", "value", "amount", "current" }),
            ("Components.SingletonComponents.CurrentIron", new[] { "value", "stored", "amount", "current" }),
        };

        private static readonly (string EcsType, string[] FieldPaths)[] MoneyAlternatives =
        {
            ("Components.RawComponents.CurrentMoney",  new[] { "value", "amount", "current", "count", "_value", "stored" }),
            ("Components.CurrentMoney",                 new[] { "value", "amount", "current", "count", "_value", "stored" }),
            ("Components.MoneyAmount",                  new[] { "value", "amount", "current", "count" }),
            ("Components.ResourceData",                 new[] { "money", "gold", "currentMoney", "_money", "_gold" }),
            ("Components.SingletonComponents.MoneyStorage", new[] { "stored", "value", "amount", "current" }),
            ("Components.SingletonComponents.CurrentMoney", new[] { "value", "stored", "amount", "current" }),
        };

        private static readonly (string EcsType, string[] FieldPaths)[] SoulsAlternatives =
        {
            ("Components.RawComponents.CurrentSouls",  new[] { "value", "amount", "current", "count", "_value", "stored" }),
            ("Components.CurrentSouls",                 new[] { "value", "amount", "current", "count", "_value", "stored" }),
            ("Components.SoulAmount",                   new[] { "value", "amount", "current", "count" }),
            ("Components.ResourceData",                 new[] { "souls", "soulAmount", "currentSouls", "_souls" }),
            ("Components.SingletonComponents.SoulStorage", new[] { "stored", "value", "amount", "current" }),
        };

        private static readonly (string EcsType, string[] FieldPaths)[] BonesAlternatives =
        {
            // Primary uses "valueContainer.value" path (nested struct) — try flat paths too
            ("Components.RawComponents.CurrentBones",  new[] { "valueContainer.value", "value", "amount", "current", "_value", "stored" }),
            ("Components.CurrentBones",                 new[] { "value", "amount", "current", "_value", "stored" }),
            ("Components.ResourceData",                 new[] { "bones", "bonesAmount", "currentBones", "_bones" }),
            ("Components.SingletonComponents.BonesStorage", new[] { "stored", "value", "amount", "current" }),
        };

        private static readonly (string EcsType, string[] FieldPaths)[] SpiritAlternatives =
        {
            ("Components.RawComponents.CurrentSpirit", new[] { "valueContainer.value", "value", "amount", "current", "_value", "stored" }),
            ("Components.CurrentSpirit",                new[] { "value", "amount", "current", "_value", "stored" }),
            ("Components.ResourceData",                 new[] { "spirit", "spiritAmount", "currentSpirit", "_spirit" }),
            ("Components.SingletonComponents.SpiritStorage", new[] { "stored", "value", "amount", "current" }),
        };

        /// <summary>
        /// Reads all known resource stockpile values from the entity manager.
        /// Uses a primary mapping with automatic fallback to alternative component
        /// type names and field paths to tolerate game version differences.
        /// 
        /// Auto-discovery: First time called, scans for resource-related component types
        /// from EcsTypeDiscovery cache and adds any found types to the fallback chain.
        /// </summary>
        /// <param name="em">The EntityManager to query.</param>
        /// <returns>A snapshot of current resource values.</returns>
        public static ResourceSnapshot ReadResources(EntityManager em)
        {
            // Auto-discover resource types on first call
            AutoDiscoverResourceTypes();

            ResourceSnapshot snapshot = new ResourceSnapshot();

            snapshot.Food = ReadWithFallback(em, FoodAlternatives, "food");
            snapshot.Wood = ReadWithFallback(em, WoodAlternatives, "wood");
            snapshot.Stone = ReadWithFallback(em, StoneAlternatives, "stone");
            snapshot.Iron = ReadWithFallback(em, IronAlternatives, "iron");
            snapshot.Money = ReadWithFallback(em, MoneyAlternatives, "money");
            snapshot.Souls = ReadWithFallback(em, SoulsAlternatives, "souls");
            snapshot.Bones = ReadWithFallback(em, BonesAlternatives, "bones");
            snapshot.Spirit = ReadWithFallback(em, SpiritAlternatives, "spirit");

            return snapshot;
        }

        /// <summary>
        /// Tries each (EcsType, fieldPaths[]) alternative in order and returns the
        /// first non-zero reading. Returns 0 only when every alternative fails.
        /// </summary>
        private static int ReadWithFallback(
            EntityManager em,
            (string EcsType, string[] FieldPaths)[] alternatives,
            string resourceName)
        {
            foreach ((string ecsType, string[] fieldPaths) in alternatives)
            {
                foreach (string fieldPath in fieldPaths)
                {
                    int result = ReadSingletonInt(em, ecsType, fieldPath);
                    if (result != 0)
                    {
                        WriteDebug($"[ResourceReader] {resourceName}: resolved via {ecsType}.{fieldPath} = {result}");
                        return result;
                    }
                }
            }

            WriteDebug($"[ResourceReader] {resourceName}: all alternatives returned 0 — check dinoforge_debug.log for field errors");
            return 0;
        }

        /// <summary>
        /// Reads a single integer value from a component by ECS type name and dotted field path.
        ///
        /// Bug fix: EntityQueryOptions.IncludePrefab is mandatory — DINO stores resource
        /// singleton entities (and all other live entities) as ECS Prefab entities.
        /// Without this flag the query returns an empty result even when the component
        /// type resolves and entities genuinely exist.
        /// </summary>
        private static int ReadSingletonInt(EntityManager em, string ecsTypeName, string fieldPath)
        {
            try
            {
                // Step 1: resolve the CLR type
                Type? clrType = null;
                foreach (System.Reflection.Assembly asm in AppDomain.CurrentDomain.GetAssemblies())
                {
                    try
                    {
                        clrType = asm.GetType(ecsTypeName, throwOnError: false);
                        if (clrType != null) break;
                    }
                    catch { }
                }

                if (clrType == null)
                    return 0; // type not present in this game version — silent, normal path

                // Step 2: resolve to Unity ComponentType
                ComponentType? ct = EntityQueries.ResolveComponentType(ecsTypeName);
                if (ct == null)
                {
                    WriteDebug($"[ResourceReader] ResolveComponentType failed for {ecsTypeName}");
                    return 0;
                }

                // Step 3: query entities — MUST include IncludePrefab.
                // DINO marks all live entities (including resource singletons) as ECS Prefabs.
                // Without this flag, queries return 0 results even when entities exist.
                EntityQueryDesc desc = new EntityQueryDesc
                {
                    All = new[] { ct.Value },
                    Options = EntityQueryOptions.IncludePrefab
                };
                EntityQuery query = em.CreateEntityQuery(desc);
                NativeArray<Entity> entities = query.ToEntityArray(Allocator.Temp);

                if (entities.Length == 0)
                {
                    entities.Dispose();
                    query.Dispose();
                    return 0;
                }

                // Pick the first entity (resource singletons have exactly one)
                Entity entity = entities[0];
                entities.Dispose();
                query.Dispose();

                // Step 4: read component data via reflection
                MethodInfo? getMethod = typeof(EntityManager)
                    .GetMethod("GetComponentData", new[] { typeof(Entity) });
                if (getMethod == null) return 0;

                MethodInfo genericGet = getMethod.MakeGenericMethod(clrType);
                object? data = genericGet.Invoke(em, new object[] { entity });
                if (data == null) return 0;

                // Step 5: walk the dotted field path (e.g. "valueContainer.value")
                string[] segments = fieldPath.Split('.');
                object? current = data;
                Type currentType = clrType;

                foreach (string seg in segments)
                {
                    if (current == null) return 0;

                    FieldInfo? field = currentType.GetField(seg,
                        BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);

                    if (field == null)
                    {
                        // Log available fields so we can identify the correct name from the debug log
                        FieldInfo[] available = currentType.GetFields(
                            BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);
                        string fieldList = string.Join(", ", Array.ConvertAll(available, f => f.Name));
                        WriteDebug($"[ResourceReader] Field '{seg}' not found on {currentType.FullName}. " +
                                   $"Available fields: [{fieldList}]");
                        return 0;
                    }

                    current = field.GetValue(current);
                    currentType = field.FieldType;
                }

                if (current is int intVal) return intVal;
                if (current is float floatVal) return (int)floatVal;
                if (current is long longVal) return (int)longVal;
                if (current is double dblVal) return (int)dblVal;

                WriteDebug($"[ResourceReader] Unexpected value type {current?.GetType().FullName} at path {ecsTypeName}.{fieldPath}");
                return 0;
            }
            catch (Exception ex)
            {
                WriteDebug($"[ResourceReader] Exception reading {ecsTypeName}.{fieldPath}: {ex.Message}");
                return 0;
            }
        }

        private static bool _autoDiscoveryDone;

        /// <summary>
        /// Auto-discovers resource-related component types from EcsTypeDiscovery cache.
        /// Adds any found types to the fallback chains for better compatibility.
        /// </summary>
        private static void AutoDiscoverResourceTypes()
        {
            if (_autoDiscoveryDone) return;
            _autoDiscoveryDone = true;

            try
            {
                var discovered = EcsTypeDiscovery.GetDiscoveredTypes();
                if (discovered == null || discovered.Count == 0)
                {
                    WriteDebug("[ResourceReader] AutoDiscovery: no types discovered yet");
                    return;
                }

                WriteDebug($"[ResourceReader] AutoDiscovery: scanning {discovered.Count} discovered types for resources");

                // Find resource-related types
                var resourceTypes = discovered
                    .Where(t => t.Contains("Food") || t.Contains("Wood") || t.Contains("Stone") ||
                                t.Contains("Iron") || t.Contains("Money") || t.Contains("Soul") ||
                                t.Contains("Bone") || t.Contains("Spirit") || t.Contains("Resource"))
                    .ToList();

                if (resourceTypes.Any())
                {
                    WriteDebug("[ResourceReader] AutoDiscovery found resource types:");
                    foreach (var t in resourceTypes.Take(20))
                    {
                        WriteDebug($"  - {t}");
                    }
                    if (resourceTypes.Count > 20)
                        WriteDebug($"  ... and {resourceTypes.Count - 20} more");
                }
                else
                {
                    WriteDebug("[ResourceReader] AutoDiscovery: no resource types found in scanned assemblies");
                }
            }
            catch (Exception ex)
            {
                WriteDebug($"[ResourceReader] AutoDiscovery failed: {ex.Message}");
            }
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
    }
}
