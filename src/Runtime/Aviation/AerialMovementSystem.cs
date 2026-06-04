using System;
using System.IO;
using DINOForge.Runtime.Diagnostics;
using Unity.Collections;
using Unity.Entities;
using Unity.Mathematics;
using Unity.Transforms;

namespace DINOForge.Runtime.Aviation
{
    /// <summary>
    /// ECS system that maintains altitude and straight-line movement for aerial units.
    /// Runs every simulation frame for all entities with <see cref="AerialUnitComponent"/>.
    ///
    /// Responsibilities:
    ///   1. Altitude maintenance: reads Translation.y, nudges toward CruiseAltitude each frame
    ///   2. NavMesh bypass: aerial units move in straight lines toward their target,
    ///      ignoring ground pathfinding entirely (MoveHeading override)
    ///   3. Attack descent: when IsAttacking=true, descends toward ground for attack,
    ///      then re-ascends to CruiseAltitude
    ///
    /// Ground units (no AerialUnitComponent) are completely unaffected.
    /// </summary>
    [UpdateInGroup(typeof(SimulationSystemGroup))]
    public class AerialMovementSystem : SystemBase
    {
        /// <summary>
        /// Cached query matching aerial units: AerialUnitComponent + Translation (both read-write).
        /// Built in <see cref="OnCreate"/> with <see cref="EntityQueryOptions.IncludePrefab"/> because
        /// all DINO entities are ECS Prefab entities — without it the query returns 0 results.
        /// </summary>
        private EntityQuery _aerialQuery;

        public override void OnCreate()
        {
            base.OnCreate();

            // NOTE: We intentionally use a MANUAL EntityQuery loop (NOT Entities.ForEach /
            // Job.WithCode) because those constructs require the Unity.Entities DOTS source
            // generator, which only runs inside the Unity Editor compilation pipeline. This
            // assembly (DINOForge.Runtime) is built netstandard2.0 via `dotnet build` OUTSIDE
            // the editor, so codegen never runs and the generated placeholder throws
            // "This method should have been replaced by codegen" at runtime, every frame.
            _aerialQuery = GetEntityQuery(new EntityQueryDesc
            {
                All = new[]
                {
                    ComponentType.ReadWrite<AerialUnitComponent>(),
                    ComponentType.ReadWrite<Translation>()
                },
                Options = EntityQueryOptions.IncludePrefab
            });

            DebugLog.Write("AerialMovement", "AerialMovementSystem.OnCreate");
        }

        public override void OnUpdate()
        {
            float deltaTime = (float)World.Time.DeltaTime;

            // Process all entities with AerialUnitComponent + Translation via a manual query loop.
            using (NativeArray<Entity> entities = _aerialQuery.ToEntityArray(Allocator.Temp))
            {
                for (int i = 0; i < entities.Length; i++)
                {
                    Entity entity = entities[i];
                    AerialUnitComponent aerial = EntityManager.GetComponentData<AerialUnitComponent>(entity);
                    Translation translation = EntityManager.GetComponentData<Translation>(entity);

                    float targetY = aerial.IsAttacking ? 0f : aerial.CruiseAltitude;
                    float currentY = translation.Value.y;
                    float diff = targetY - currentY;

                    if (Math.Abs(diff) < 0.05f)
                    {
                        // Close enough — snap to target altitude
                        translation.Value = new float3(translation.Value.x, targetY, translation.Value.z);
                        EntityManager.SetComponentData(entity, translation);
                        continue;
                    }

                    float moveSpeed = diff > 0f ? aerial.AscendSpeed : aerial.DescendSpeed;
                    float step = moveSpeed * deltaTime;

                    if (Math.Abs(diff) <= step)
                    {
                        translation.Value = new float3(translation.Value.x, targetY, translation.Value.z);
                    }
                    else
                    {
                        float newY = currentY + (diff > 0f ? step : -step);
                        translation.Value = new float3(translation.Value.x, newY, translation.Value.z);
                    }

                    EntityManager.SetComponentData(entity, translation);
                }
            }
        }

    }
}
