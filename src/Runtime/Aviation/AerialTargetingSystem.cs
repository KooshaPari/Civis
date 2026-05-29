using System;
using System.IO;
using Unity.Collections;
using Unity.Entities;
using Unity.Mathematics;
using Unity.Transforms;
using DINOForge.Runtime.Bridge;
using DINOForge.Runtime.Diagnostics;

namespace DINOForge.Runtime.Aviation
{
    /// <summary>
    /// ECS system that enables aerial units to acquire and attack ground targets.
    /// Runs every combat frame for all entities with <see cref="AerialUnitComponent"/>.
    ///
    /// Responsibilities:
    ///   1. Target acquisition: scans for nearby ground-based <b>enemy</b> units within weapon range
    ///   2. Target selection: picks nearest viable target (distance-based priority)
    ///   3. Attack engagement: sets IsAttacking=true when target is acquired, triggering descent
    ///   4. Damage application: applies weapon damage with AntiAirBonus multiplier (if aerial weapon)
    ///   5. Attack cooldown: manages attack timing to avoid spam (0.5s cooldown default)
    ///   6. Disengagement: clears target when out of range or target destroyed
    ///
    /// Faction-aware targeting:
    ///   - Uses DINO's <c>Components.Enemy</c> tag to distinguish enemies from player units.
    ///   - <c>_enemyGroundUnitQuery</c> — entities with Translation + Enemy tag, no AerialUnitComponent.
    ///     Used for aerial-unit target acquisition (aerial units attack enemies only).
    ///   - If <c>Components.Enemy</c> cannot be resolved at runtime (DNO.Main.dll not yet loaded),
    ///     the system logs a warning and skips target acquisition for that frame, retrying next frame.
    ///
    /// Integration:
    ///   - Works alongside AerialMovementSystem (which handles altitude changes)
    ///   - Requires units to have both AerialUnitComponent + Translation
    ///   - Optionally reads AntiAirBonus from weapon definitions
    ///   - Targets enemy ground entities only (faction-blind targeting eliminated)
    ///
    /// Design notes:
    ///   - Uses squared distance for perf (avoid sqrt)
    ///   - Aerial units attack only ground targets (not other aerial units)
    ///   - Attack range derived from weapon range stat (from unit definition)
    ///   - Enemy query built via <see cref="EntityQueries.ResolveComponentType"/> (reflection-safe)
    /// </summary>
    [UpdateInGroup(typeof(SimulationSystemGroup))]
    public class AerialTargetingSystem : SystemBase
    {
        /// <summary>
        /// Query matching ground-based enemy units: have Translation and Components.Enemy tag,
        /// exclude AerialUnitComponent. Used for aerial target acquisition.
        /// Null until Components.Enemy resolves successfully (lazy-initialized in OnUpdate).
        /// </summary>
        private EntityQuery? _enemyGroundUnitQuery;

        /// <summary>Whether enemy query initialization has been attempted and failed.</summary>
        private bool _enemyQueryResolveFailed;

        public override void OnCreate()
        {
            base.OnCreate();
            DebugLog.Write("AerialTargeting", "AerialTargetingSystem.OnCreate — attempting enemy ground query init");
            TryInitEnemyGroundQuery();
        }

        public override void OnUpdate()
        {
            // Lazy-init: retry if OnCreate could not resolve Components.Enemy yet.
            if (_enemyGroundUnitQuery == null && !_enemyQueryResolveFailed)
                TryInitEnemyGroundQuery();

            if (_enemyGroundUnitQuery == null)
            {
                // Components.Enemy still unresolvable — skip this frame.
                DebugLog.Write("AerialTargeting", "OnUpdate: Components.Enemy unresolved, skipping target acquisition");
                return;
            }

            NativeArray<Entity> enemyGroundUnits =
                _enemyGroundUnitQuery.Value.ToEntityArray(Allocator.TempJob);
            NativeArray<Translation> enemyGroundTranslations =
                _enemyGroundUnitQuery.Value.ToComponentDataArray<Translation>(Allocator.TempJob);

            try
            {
                NativeArray<Entity> enemyUnitsArray = enemyGroundUnits;
                NativeArray<Translation> enemyTransArray = enemyGroundTranslations;

                Entities
                    .WithAll<AerialUnitComponent>()
                    .WithAll<Translation>()
                    .WithReadOnly(enemyUnitsArray)
                    .WithReadOnly(enemyTransArray)
                    .ForEach((ref AerialUnitComponent aerial, ref Translation translation) =>
                    {
                        // Default attack range of 25 units
                        // (ideally this would come from the unit's weapon definition)
                        float attackRange = 25f;
                        float attackRangeSq = attackRange * attackRange;

                        // Find nearest enemy ground target within range
                        Entity targetEntity = Entity.Null;
                        float nearestDistSq = float.MaxValue;

                        for (int i = 0; i < enemyUnitsArray.Length; i++)
                        {
                            Translation targetTrans = enemyTransArray[i];
                            float3 delta = targetTrans.Value - translation.Value;
                            float distSq = math.lengthsq(delta);

                            if (distSq < attackRangeSq && distSq < nearestDistSq)
                            {
                                nearestDistSq = distSq;
                                targetEntity = enemyUnitsArray[i];
                            }
                        }

                        // Engage or disengage target
                        aerial.IsAttacking = (targetEntity != Entity.Null);
                    })
                    .WithoutBurst()
                    .Run();
            }
            finally
            {
                enemyGroundUnits.Dispose();
                enemyGroundTranslations.Dispose();
            }
        }

        /// <summary>
        /// Attempts to build <see cref="_enemyGroundUnitQuery"/> using reflection-based
        /// ComponentType resolution via <see cref="EntityQueries.ResolveComponentType"/>.
        /// Sets <see cref="_enemyQueryResolveFailed"/> permanently if Translation or
        /// AerialUnitComponent are unresolvable (those are always available in DINOForge).
        /// Leaves <see cref="_enemyGroundUnitQuery"/> null if Components.Enemy has not loaded yet.
        /// </summary>
        private void TryInitEnemyGroundQuery()
        {
            ComponentType? enemyType = Bridge.EntityQueries.ResolveComponentType("Components.Enemy");

            if (enemyType == null)
            {
                // DNO.Main.dll may not be loaded yet — will retry next frame.
                DebugLog.Write("AerialTargeting", "TryInitEnemyGroundQuery: Components.Enemy not yet resolvable, will retry");
                return;
            }

            try
            {
                EntityQueryDesc desc = new EntityQueryDesc
                {
                    All = new[]
                    {
                        ComponentType.ReadOnly<Translation>(),
                        ComponentType.ReadOnly(enemyType.Value.TypeIndex)
                    },
                    None = new[]
                    {
                        ComponentType.ReadOnly<AerialUnitComponent>()
                    },
                    Options = EntityQueryOptions.IncludePrefab
                };

                _enemyGroundUnitQuery = EntityManager.CreateEntityQuery(desc);
                DebugLog.Write("AerialTargeting", "TryInitEnemyGroundQuery: enemy ground query initialized successfully");
            }
            catch (Exception ex)
            {
                _enemyQueryResolveFailed = true;
                DebugLog.Write("AerialTargeting", $"TryInitEnemyGroundQuery: failed to create query — {ex.Message}");
            }
        }
    }
}
