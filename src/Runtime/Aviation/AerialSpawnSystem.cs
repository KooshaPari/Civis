using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.Runtime.Diagnostics;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using Unity.Collections;
using Unity.Entities;
using Unity.Mathematics;
using Unity.Transforms;

namespace DINOForge.Runtime.Aviation
{
    /// <summary>
    /// ECS system that performs two aerial-related startup tasks:
    ///
    /// 1. <b>Aerial unit altitude initialisation</b> — detects newly spawned aerial units and
    ///    sets their Y position to <see cref="AerialUnitComponent.CruiseAltitude"/> so they
    ///    don't pop up from the ground.  Controlled by <see cref="SpawnAtAltitude"/>.
    ///
    /// 2. <b>Anti-air building sweep</b> — runs once after the world is populated and attaches
    ///    <see cref="AntiAirComponent"/> to every building entity whose pack definition carries
    ///    <c>defense_tags: [AntiAir]</c>.  Buildings in DINO are baked into the game world at
    ///    scene load; there is no runtime building-spawner, so this sweep is the canonical
    ///    call-site for <see cref="AerialBuildingMapper.ApplyAntiAirComponent"/>.
    ///
    ///    The sweep strategy:
    ///    - Query all live entities with <c>Components.BuildingBase</c>.
    ///    - For each loaded pack building definition that has the "AntiAir" defense_tag,
    ///      apply <see cref="AntiAirComponent"/> to <i>all</i> currently-untagged building
    ///      entities.  (DINO building entities do not carry pack IDs at runtime, so we apply
    ///      the component broadly.  Future work: correlate by vanilla_mapping or a custom
    ///      DINOForge tag component.)
    ///    - The sweep is guarded by <see cref="_buildingSweepDone"/> so it runs only once.
    ///
    /// Call <see cref="Initialize"/> from ModPlatform after packs are loaded to supply the
    /// building registry.
    /// </summary>
    [UpdateInGroup(typeof(SimulationSystemGroup))]
    public class AerialSpawnSystem : SystemBase
    {
        /// <summary>
        /// When true, newly detected aerial units are teleported to CruiseAltitude instantly.
        /// When false, units start at Y=0 and ascend naturally (takeoff animation feel).
        /// </summary>
        public static bool SpawnAtAltitude = true;

        private static RegistryManager? _registry;

        /// <summary>Minimum frames to wait before running the building sweep.</summary>
        private const int MinFrameDelay = 1800; // ~30 seconds at 60 fps

        private int _frameCount;
        private bool _buildingSweepDone;

        // -------------------------------------------------------------------------
        // Initialisation
        // -------------------------------------------------------------------------

        /// <summary>
        /// Supplies the building registry used by the anti-air building sweep.
        /// Call this from ModPlatform after packs have been loaded.
        /// </summary>
        /// <param name="registry">The RegistryManager containing loaded pack definitions.</param>
        public static void Initialize(RegistryManager? registry)
        {
            // Iter-144 H9 probe: ENTER/EXIT timing around mod-side pack-recreation entry point.
            var __sw = System.Diagnostics.Stopwatch.StartNew();
            DebugLog.Write("AerialSpawn", $"[AerialSpawnSystem.Initialize] ENTER thread={System.Threading.Thread.CurrentThread.ManagedThreadId}");
            try
            {
                _registry = registry;
                DebugLog.Write("AerialSpawn", "AerialSpawnSystem.Initialize: Registry set");
            }
            finally
            {
                DebugLog.Write("AerialSpawn", $"[AerialSpawnSystem.Initialize] EXIT elapsed={__sw.ElapsedMilliseconds}ms");
            }
        }

        // -------------------------------------------------------------------------
        // ECS lifecycle
        // -------------------------------------------------------------------------

        /// <summary>
        /// Cached query matching aerial units: AerialUnitComponent + Translation (both read-write).
        /// Built in <see cref="OnCreate"/> with <see cref="EntityQueryOptions.IncludePrefab"/> because
        /// all DINO entities are ECS Prefab entities — without it the query returns 0 results.
        /// </summary>
        private EntityQuery _aerialQuery;

        public override void OnCreate()
        {
            base.OnCreate();

            // NOTE: Manual EntityQuery loop (NOT Entities.ForEach / Job.WithCode) — those require the
            // Unity.Entities DOTS source generator, which only runs inside the Unity Editor. This
            // assembly is built netstandard2.0 via `dotnet build` outside the editor, so codegen never
            // runs and the placeholder throws "This method should have been replaced by codegen" every frame.
            _aerialQuery = GetEntityQuery(new EntityQueryDesc
            {
                All = new[]
                {
                    ComponentType.ReadWrite<AerialUnitComponent>(),
                    ComponentType.ReadWrite<Translation>()
                },
                Options = EntityQueryOptions.IncludePrefab
            });
        }

        public override void OnUpdate()
        {
            _frameCount++;

            // --- 1. Aerial unit altitude initialisation (manual query loop; codegen-free) ---
            if (SpawnAtAltitude)
            {
                using (NativeArray<Entity> entities = _aerialQuery.ToEntityArray(Allocator.Temp))
                {
                    for (int i = 0; i < entities.Length; i++)
                    {
                        Entity entity = entities[i];
                        AerialUnitComponent aerial = EntityManager.GetComponentData<AerialUnitComponent>(entity);
                        Translation translation = EntityManager.GetComponentData<Translation>(entity);

                        if (translation.Value.y < 1f && aerial.CruiseAltitude > 0f)
                        {
                            translation.Value = new float3(
                                translation.Value.x,
                                aerial.CruiseAltitude,
                                translation.Value.z);
                            EntityManager.SetComponentData(entity, translation);
                        }
                    }
                }
            }

            // --- 2. Anti-air building sweep (runs once after world is populated) ---
            if (!_buildingSweepDone && _frameCount >= MinFrameDelay)
            {
                RunBuildingAntiAirSweep();
                _buildingSweepDone = true;
            }
        }

        // -------------------------------------------------------------------------
        // Building anti-air sweep
        // -------------------------------------------------------------------------

        /// <summary>
        /// Scans the live ECS world for building entities and attaches
        /// <see cref="AntiAirComponent"/> based on pack building definitions.
        ///
        /// Because DINO building entities do not carry pack definition IDs at runtime,
        /// the sweep applies <see cref="AntiAirComponent"/> to all untagged building
        /// entities whenever <i>any</i> loaded pack building definition has the "AntiAir"
        /// defense_tag.  A future refinement should correlate via <c>vanilla_mapping</c>
        /// or a DINOForge-added tag component (e.g. PackBuildingTag) so that per-building
        /// range / damage_bonus values can be applied selectively.
        /// </summary>
        private void RunBuildingAntiAirSweep()
        {
            if (_registry == null)
            {
                DebugLog.Write("AerialSpawn", "BuildingAntiAirSweep: registry not initialised, skipping.");
                return;
            }

            // Collect all pack building definitions that are flagged AntiAir.
            List<BuildingDefinition> antiAirDefs = new List<BuildingDefinition>();
            foreach (KeyValuePair<string, RegistryEntry<BuildingDefinition>> kvp in _registry.Buildings.All)
            {
                BuildingDefinition def = kvp.Value.Data;
                if (def?.DefenseTags != null && def.DefenseTags.Contains("AntiAir"))
                    antiAirDefs.Add(def);
            }

            if (antiAirDefs.Count == 0)
            {
                DebugLog.Write("AerialSpawn", "BuildingAntiAirSweep: no AntiAir building definitions found, nothing to do.");
                return;
            }

            DebugLog.Write("AerialSpawn", $"BuildingAntiAirSweep: {antiAirDefs.Count} AntiAir building definition(s) found. Querying live building entities.");

            // Resolve Components.BuildingBase via the EntityQueries helper.
            ComponentType? buildingBaseType = Bridge.EntityQueries.ResolveComponentType("Components.BuildingBase");
            if (buildingBaseType == null)
            {
                DebugLog.Write("AerialSpawn", "BuildingAntiAirSweep: Components.BuildingBase could not be resolved. Skipping sweep.");
                return;
            }

            EntityManager em = World.DefaultGameObjectInjectionWorld.EntityManager;

            EntityQueryDesc desc = new EntityQueryDesc
            {
                All = new[] { ComponentType.ReadOnly(buildingBaseType.Value.TypeIndex) },
                None = new[] { ComponentType.ReadOnly<AntiAirComponent>() }
            };

            EntityQuery query = em.CreateEntityQuery(desc);
            using NativeArray<Entity> buildings = query.ToEntityArray(Allocator.Temp);

            if (buildings.Length == 0)
            {
                DebugLog.Write("AerialSpawn", "BuildingAntiAirSweep: no untagged building entities found.");
                return;
            }

            DebugLog.Write("AerialSpawn", $"BuildingAntiAirSweep: {buildings.Length} building entit(ies) without AntiAirComponent. " +
                       $"Applying components from {antiAirDefs.Count} AntiAir definition(s).");

            // Use the first AntiAir definition's parameters as the authoritative values.
            // When vanilla_mapping correlation is added this should be per-entity.
            BuildingDefinition representative = antiAirDefs[0];
            int applied = 0;

            foreach (Entity building in buildings)
            {
                try
                {
                    AerialBuildingMapper.ApplyAntiAirComponent(em, building, representative);
                    applied++;
                }
                catch (Exception ex)
                {
                    DebugLog.Write("AerialSpawn", $"BuildingAntiAirSweep: error on entity {building.Index}: {ex.Message}");
                }
            }

            DebugLog.Write("AerialSpawn", $"BuildingAntiAirSweep: applied AntiAirComponent to {applied}/{buildings.Length} building entities.");
        }
    }
}
