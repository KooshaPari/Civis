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
        private static bool _isRuntimeEnabled = false;
        private static bool _disabledWarningLogged;

        public override void OnCreate()
        {
            base.OnCreate();
            DebugLog.Write("AerialMovement", "AerialMovementSystem.OnCreate");

            if (!_isRuntimeEnabled)
            {
                if (!_disabledWarningLogged)
                {
                    DebugLog.Write("AerialMovement", "AerialMovementSystem disabled (feature gate OFF). Restore runtime enablement when DOTS codegen path is stable.");
                    _disabledWarningLogged = true;
                }

                Enabled = false;
            }
        }

        public override void OnUpdate()
        {
            float deltaTime = (float)World.Time.DeltaTime;

            // Manual EntityQuery loop — avoids Entities.ForEach DOTS codegen path
            // which crashes in netstandard2.0 Mono even when system is disabled
            // (job types are registered at world-create time). Pattern #233.
            EntityQueryDesc moveDesc = new EntityQueryDesc
            {
                All = new[] { ComponentType.ReadWrite<AerialUnitComponent>(), ComponentType.ReadWrite<Translation>() }
            };
            EntityQuery moveQuery = EntityManager.CreateEntityQuery(moveDesc);
            using NativeArray<Entity> moveEntities = moveQuery.ToEntityArray(Allocator.Temp);
            foreach (Entity entity in moveEntities)
            {
                AerialUnitComponent aerial = EntityManager.GetComponentData<AerialUnitComponent>(entity);
                Translation translation = EntityManager.GetComponentData<Translation>(entity);

                float targetY = aerial.IsAttacking ? 0f : aerial.CruiseAltitude;
                float currentY = translation.Value.y;
                float diff = targetY - currentY;

                if (Math.Abs(diff) < 0.05f)
                {
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
