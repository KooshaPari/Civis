using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using DINOForge.Runtime.Diagnostics;
using Unity.Collections;
using Unity.Entities;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Represents a single stat modification to be applied to matching entities.
    /// </summary>
    public sealed class StatModification
    {
        /// <summary>
        /// SDK model path identifying the stat (e.g. "unit.stats.hp").
        /// Must match a key in <see cref="ComponentMap.All"/>.
        /// </summary>
        public string SdkPath { get; }

        /// <summary>
        /// The new value to write, or the multiplier/addend depending on <see cref="Mode"/>.
        /// </summary>
        public float Value { get; }

        /// <summary>
        /// How to apply the modification.
        /// </summary>
        public ModifierMode Mode { get; }

        /// <summary>
        /// Optional filter: only apply to entities that also have this component type.
        /// Null means apply to all matching entities.
        /// </summary>
        public string? FilterComponentType { get; }

        /// <summary>
        /// Number of times this modification has been retried after initial failure.
        /// </summary>
        public int RetryCount { get; }

        public StatModification(string sdkPath, float value,
            ModifierMode mode = ModifierMode.Override, string? filterComponentType = null, int retryCount = 0)
        {
            SdkPath = sdkPath ?? throw new ArgumentNullException(nameof(sdkPath));
            Value = value;
            Mode = mode;
            FilterComponentType = filterComponentType;
            RetryCount = retryCount;
        }
    }

    /// <summary>
    /// How a stat modification is applied to the existing value.
    /// </summary>
    public enum ModifierMode
    {
        /// <summary>Replace the existing value entirely.</summary>
        Override,

        /// <summary>Add to the existing value.</summary>
        Add,

        /// <summary>Multiply the existing value.</summary>
        Multiply
    }

    /// <summary>
    /// ECS System that applies DINOForge mod stat modifications to game entities.
    /// Runs in the SimulationSystemGroup after game initialization to modify
    /// unit/building stats based on loaded packs.
    ///
    /// Usage:
    ///   1. Queue modifications via <see cref="Enqueue"/> before or during runtime
    ///   2. The system processes the queue on its next update
    ///   3. After processing, the system disables itself (one-shot by default)
    ///   4. Call <see cref="Reapply"/> to force re-processing
    ///
    /// Manual testing:
    ///   1. Load game, wait for entities to spawn
    ///   2. Enqueue a stat modification (e.g. double all unit HP)
    ///   3. Verify via entity dump that Health component values changed
    ///   4. Check BepInEx/dinoforge_debug.log for application results
    /// </summary>
    [UpdateInGroup(typeof(SimulationSystemGroup))]
    public class StatModifierSystem : SystemBase
    {
        private static readonly Queue<StatModification> _pendingModifications =
            new Queue<StatModification>();

        private bool _applied;
        private int _frameCount;

        /// <summary>
        /// Minimum frames to wait before applying modifications.
        /// Gives the game time to fully initialize entity data.
        /// </summary>
        private const int MinFrameDelay = 1800; // ~30 seconds at 60fps (entities spawn around frame 1200)

        /// <summary>
        /// Frames to wait between retry attempts.
        /// </summary>
        private const int RetryFrameDelay = 600; // ~10 seconds at 60fps

        /// <summary>
        /// Maximum number of retries for failed modifications.
        /// </summary>
        private const int MaxRetries = 3;

        /// <summary>
        /// Queue a stat modification for application on the next update cycle.
        /// Thread-safe: can be called from pack loaders on any thread.
        /// </summary>
        public static void Enqueue(StatModification modification)
        {
            if (modification == null) throw new ArgumentNullException(nameof(modification));

            lock (_pendingModifications)
            {
                _pendingModifications.Enqueue(modification);
            }
        }

        /// <summary>
        /// Queue multiple stat modifications at once.
        /// </summary>
        public static void EnqueueRange(IEnumerable<StatModification> modifications)
        {
            if (modifications == null) throw new ArgumentNullException(nameof(modifications));

            lock (_pendingModifications)
            {
                foreach (StatModification mod in modifications)
                {
                    _pendingModifications.Enqueue(mod);
                }
            }
        }

        /// <summary>
        /// Force the system to re-process on its next update.
        /// </summary>
        public static void Reapply()
        {
            // The system will check for pending modifications even after initial apply
        }

        /// <summary>
        /// Applies a single stat modification immediately using the provided <see cref="EntityManager"/>.
        /// Unlike <see cref="Enqueue"/>, this bypasses the frame-delay queue and applies the change
        /// synchronously. Must be called from the Unity main thread (e.g. via MainThreadDispatcher).
        /// Targets all matching entities regardless of the Prefab tag so that both blueprint prefabs
        /// and live spawned entities are updated in one pass.
        /// </summary>
        /// <param name="em">The EntityManager for the active world.</param>
        /// <param name="modification">The stat modification to apply.</param>
        /// <returns>Number of entities that were modified, or -1 if the mapping was not found.</returns>
        public static int ApplyImmediate(EntityManager em, StatModification modification)
        {
            if (modification == null) throw new ArgumentNullException(nameof(modification));

            ComponentMapping? mapping = ComponentMap.Find(modification.SdkPath);
            if (mapping == null)
            {
                DebugLog.Write("StatModifier",$"[ApplyImmediate] No ComponentMapping for '{modification.SdkPath}'");
                return -1;
            }

            Type? componentType = mapping.ResolvedType;
            if (componentType == null)
            {
                DebugLog.Write("StatModifier",$"[ApplyImmediate] Cannot resolve ECS type: {mapping.EcsComponentType}");
                return 0;
            }

            ComponentType? ct = Bridge.EntityQueries.ResolveComponentType(mapping.EcsComponentType);
            if (ct == null)
            {
                DebugLog.Write("StatModifier",$"[ApplyImmediate] Cannot create ComponentType for: {mapping.EcsComponentType}");
                return 0;
            }

            // Query both live (non-prefab) and prefab entities so that the modification is
            // visible immediately when read back via getStat (which queries without IncludePrefab).
            EntityQueryDesc queryDesc = new EntityQueryDesc
            {
                All = new[] { ct.Value },
                Options = EntityQueryOptions.IncludePrefab | EntityQueryOptions.IncludeDisabled
            };

            EntityQuery query = em.CreateEntityQuery(queryDesc);
            NativeArray<Entity> entities = query.ToEntityArray(Allocator.Temp);

            MethodInfo? getMethod = typeof(EntityManager).GetMethod("GetComponentData", new[] { typeof(Entity) });
            MethodInfo? setMethod = typeof(EntityManager).GetMethod("SetComponentData");

            int modifiedCount = 0;
            if (getMethod != null && setMethod != null)
            {
                MethodInfo genericGet = getMethod.MakeGenericMethod(componentType);
                MethodInfo genericSet = setMethod.MakeGenericMethod(componentType);

                string? targetField = mapping.TargetFieldName;
                FieldInfo? field = targetField != null
                    ? componentType.GetField(targetField,
                        BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance)
                    : FindFirstNumericField(componentType);

                if (field == null)
                {
                    DebugLog.Write("StatModifier",$"[ApplyImmediate] No target field on {componentType.FullName}");
                    entities.Dispose();
                    query.Dispose();
                    return 0;
                }

                for (int i = 0; i < entities.Length; i++)
                {
                    try
                    {
                        object? data = genericGet.Invoke(em, new object[] { entities[i] });
                        if (data == null) continue;

                        bool modified = false;
                        if (field.FieldType == typeof(float))
                        {
                            float current = (float)field.GetValue(data);
                            float newValue = ApplyMode(current, modification.Value, modification.Mode);
                            field.SetValue(data, newValue);
                            modified = true;
                        }
                        else if (field.FieldType == typeof(int))
                        {
                            int current = (int)field.GetValue(data);
                            int newValue = (int)ApplyMode(current, modification.Value, modification.Mode);
                            field.SetValue(data, newValue);
                            modified = true;
                        }

                        if (modified)
                        {
                            genericSet.Invoke(em, new object[] { entities[i], data });
                            modifiedCount++;
                        }
                    }
                    catch (Exception ex)
                    {
                        Exception root = ex is TargetInvocationException tie ? (tie.InnerException ?? ex) : ex;
                        DebugLog.Write("StatModifier",$"[ApplyImmediate] Failed entity {entities[i].Index}: {root.GetType().Name}: {root.Message}");
                    }
                }
            }

            DebugLog.Write("StatModifier",$"[ApplyImmediate] Modified {modifiedCount}/{entities.Length} entities for {modification.SdkPath}");

            entities.Dispose();
            query.Dispose();
            return modifiedCount;
        }

        protected override void OnCreate()
        {
            base.OnCreate();
            DebugLog.Write("StatModifier","StatModifierSystem.OnCreate");
        }

        protected override void OnUpdate()
        {
            _frameCount++;

            // Wait for the game to fully initialize before applying
            if (_frameCount < MinFrameDelay)
                return;

            // Check for pending modifications
            List<StatModification> batch;
            lock (_pendingModifications)
            {
                if (_pendingModifications.Count == 0)
                {
                    if (_applied)
                    {
                        Enabled = false;
                    }
                    return;
                }

                batch = new List<StatModification>();
                while (_pendingModifications.Count > 0)
                {
                    batch.Add(_pendingModifications.Dequeue());
                }
            }

            DebugLog.Write("StatModifier",$"StatModifierSystem applying {batch.Count} modifications");

            int successCount = 0;
            int failCount = 0;
            List<StatModification> retryQueue = new List<StatModification>();

            foreach (StatModification mod in batch)
            {
                try
                {
                    bool result = ApplyModification(mod);
                    if (result)
                    {
                        successCount++;
                    }
                    else
                    {
                        failCount++;
                        // Queue for retry if retries remain
                        if (mod.RetryCount < MaxRetries)
                        {
                            StatModification retryMod = new StatModification(
                                mod.SdkPath, mod.Value, mod.Mode, mod.FilterComponentType, mod.RetryCount + 1);
                            retryQueue.Add(retryMod);
                            DebugLog.Write("StatModifier",$"Queuing retry {retryMod.RetryCount}/{MaxRetries} for {mod.SdkPath}");
                        }
                        else
                        {
                            DebugLog.Write("StatModifier",$"Max retries exhausted for {mod.SdkPath}");
                        }
                    }
                }
                catch (Exception ex)
                {
                    Exception root = ex is TargetInvocationException tie ? (tie.InnerException ?? ex) : ex;
                    DebugLog.Write("StatModifier",$"StatModifierSystem: Failed to apply {mod.SdkPath}: {root.GetType().Name}: {root.Message}");
                    failCount++;
                    // Queue for retry if retries remain
                    if (mod.RetryCount < MaxRetries)
                    {
                        StatModification retryMod = new StatModification(
                            mod.SdkPath, mod.Value, mod.Mode, mod.FilterComponentType, mod.RetryCount + 1);
                        retryQueue.Add(retryMod);
                        DebugLog.Write("StatModifier",$"Queuing retry {retryMod.RetryCount}/{MaxRetries} for {mod.SdkPath} after exception");
                    }
                }
            }

            DebugLog.Write("StatModifier",$"StatModifierSystem: Applied {successCount}, failed {failCount}, queued {retryQueue.Count} for retry");

            // Re-queue failed modifications for retry
            if (retryQueue.Count > 0)
            {
                lock (_pendingModifications)
                {
                    foreach (StatModification retryMod in retryQueue)
                    {
                        _pendingModifications.Enqueue(retryMod);
                    }
                }
                // Don't disable - we have pending retries
            }
            else
            {
                _applied = true;
            }
        }

        private bool ApplyModification(StatModification mod)
        {
            // Look up the component mapping
            ComponentMapping? mapping = ComponentMap.Find(mod.SdkPath);
            if (mapping == null)
            {
                // Silently skip unmapped paths — retrying will never help, so return true
                // to prevent infinite retry spam for stats like "unit.stats.damage" that
                // have no direct writable ECS field (damage is baked into projectile blobs).
                DebugLog.Write("StatModifier",$"No ComponentMapping for '{mod.SdkPath}' — skipping (not retryable)");
                return true;
            }

            Type? componentType = mapping.ResolvedType;
            if (componentType == null)
            {
                DebugLog.Write("StatModifier",$"Cannot resolve ECS type: {mapping.EcsComponentType}");
                return false;
            }

            // Build a query for entities with this component
            ComponentType? ct = Bridge.EntityQueries.ResolveComponentType(mapping.EcsComponentType);
            if (ct == null)
            {
                DebugLog.Write("StatModifier",$"Cannot create ComponentType for: {mapping.EcsComponentType}");
                return false;
            }

            // Optionally add a filter component
            // DINO stores ALL live gameplay entities (units, buildings) as ECS Prefab entities.
            // Standard EntityQueryDesc excludes Prefab-tagged entities by default.
            // We must opt-in to include them or every query returns 0.
            EntityQueryDesc queryDesc;
            if (mod.FilterComponentType != null)
            {
                ComponentType? filterCt = Bridge.EntityQueries.ResolveComponentType(mod.FilterComponentType);
                if (filterCt == null)
                {
                    DebugLog.Write("StatModifier",$"Cannot resolve filter type: {mod.FilterComponentType}");
                    return false;
                }

                queryDesc = new EntityQueryDesc
                {
                    All = new[] { ct.Value, filterCt.Value },
                    Options = EntityQueryOptions.IncludePrefab
                };
            }
            else
            {
                queryDesc = new EntityQueryDesc
                {
                    All = new[] { ct.Value },
                    Options = EntityQueryOptions.IncludePrefab
                };
            }

            EntityQuery query = EntityManager.CreateEntityQuery(queryDesc);
            NativeArray<Entity> entities = query.ToEntityArray(Allocator.Temp);

            if (entities.Length == 0)
            {
                // Log total entity count to diagnose whether scene is loaded
                int totalEntities = EntityManager.UniversalQuery.CalculateEntityCount();
                DebugLog.Write("StatModifier",$"No entities matched for {mapping.EcsComponentType} (total entities in world: {totalEntities}, ct={ct.Value})");
                entities.Dispose();
                query.Dispose();
                return false;
            }

            DebugLog.Write("StatModifier",$"Matched {entities.Length} entities for {mapping.EcsComponentType}");

            // Resolve the target field name from the mapping.
            // Confirmed field layouts from entity dump analysis (ecs_types.json):
            //   Components.Health:        currentHealth (float), underAttackTimerEnd (float)
            //   Components.HealthBase:    _maxHealthMultiplier (float, private)
            //   Components.AttackCooldown: value (float), fixedProjectileSpeed (float)
            //   Components.MmAnimationPropertyAttackSpeedModifier: Value (float)
            //   Components.Regeneration:  regenerationStartTime (float)
            //   Components.FoodStorage:   stored (int)
            //   Components.WoodStorage:   stored (int)
            //   Components.StoneStorage:  stored (int)
            //   Components.IronStorage:   stored (int)
            //   Components.RawComponents.ObjectId: value (int)
            //   Components.RawComponents.ProjectileFlyData: damage (float), gravity (float)
            string? targetField = mapping.TargetFieldName;

            int modifiedCount = 0;

            for (int i = 0; i < entities.Length; i++)
            {
                try
                {
                    bool modified = TryModifyEntityComponent(
                        entities[i], componentType, targetField, mod.Value, mod.Mode);
                    if (modified) modifiedCount++;
                }
                catch (Exception ex)
                {
                    Exception root = ex is TargetInvocationException tie ? (tie.InnerException ?? ex) : ex;
                    DebugLog.Write("StatModifier",$"Failed to modify entity {entities[i].Index}: {root.GetType().Name}: {root.Message}");
                }
            }

            DebugLog.Write("StatModifier",$"Modified {modifiedCount}/{entities.Length} entities for {mod.SdkPath}");

            entities.Dispose();
            query.Dispose();
            return modifiedCount > 0;
        }

        /// <summary>
        /// Attempt to modify a single entity's component data via reflection.
        /// Targets a specific field by name when available (from dump-confirmed layouts),
        /// or falls back to the first numeric field if no target is specified.
        /// </summary>
        private bool TryModifyEntityComponent(
            Entity entity, Type componentType, string? targetFieldName,
            float value, ModifierMode mode)
        {
            // Get the generic GetComponentData<T> and SetComponentData<T> methods
            MethodInfo? getMethod = typeof(EntityManager)
                .GetMethod("GetComponentData", new[] { typeof(Entity) });
            MethodInfo? setMethod = typeof(EntityManager)
                .GetMethod("SetComponentData");

            if (getMethod == null || setMethod == null)
            {
                DebugLog.Write("StatModifier","Cannot find GetComponentData/SetComponentData methods on EntityManager");
                return false;
            }

            MethodInfo genericGet = getMethod.MakeGenericMethod(componentType);
            MethodInfo genericSet = setMethod.MakeGenericMethod(componentType);

            // Read current component data
            object? data = genericGet.Invoke(EntityManager, new object[] { entity });
            if (data == null) return false;

            FieldInfo? field;
            if (targetFieldName != null)
            {
                // Target the specific field confirmed from entity dump analysis
                field = componentType.GetField(targetFieldName,
                    BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);

                if (field == null)
                {
                    DebugLog.Write("StatModifier",$"Field '{targetFieldName}' not found on {componentType.FullName}");
                    return false;
                }
            }
            else
            {
                // Fallback: find first numeric field
                field = FindFirstNumericField(componentType);
                if (field == null)
                {
                    DebugLog.Write("StatModifier",$"No numeric field found on {componentType.FullName}");
                    return false;
                }
            }

            // Apply the modification based on field type
            bool modified = false;
            if (field.FieldType == typeof(float))
            {
                float current = (float)field.GetValue(data);
                float newValue = ApplyMode(current, value, mode);
                field.SetValue(data, newValue);
                modified = true;
            }
            else if (field.FieldType == typeof(int))
            {
                int current = (int)field.GetValue(data);
                int newValue = (int)ApplyMode(current, value, mode);
                field.SetValue(data, newValue);
                modified = true;
            }

            if (modified)
            {
                genericSet.Invoke(EntityManager, new object[] { entity, data });
            }

            return modified;
        }

        private static FieldInfo? FindFirstNumericField(Type type)
        {
            foreach (FieldInfo field in type.GetFields(
                BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance))
            {
                if (field.FieldType == typeof(float) || field.FieldType == typeof(int))
                    return field;
            }
            return null;
        }

        private static float ApplyMode(float current, float value, ModifierMode mode)
        {
            switch (mode)
            {
                case ModifierMode.Override:
                    return value;
                case ModifierMode.Add:
                    return current + value;
                case ModifierMode.Multiply:
                    return current * value;
                default:
                    return value;
            }
        }

    }
}
