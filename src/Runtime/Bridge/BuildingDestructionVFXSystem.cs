#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using Unity.Collections;
using Unity.Entities;
using UnityEngine;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// ECS SystemBase that spawns building collapse VFX when structures are destroyed.
    /// </summary>
    [UpdateInGroup(typeof(PresentationSystemGroup))]
    public class BuildingDestructionVFXSystem : SystemBase
    {
        private static ParticlePoolManager? _poolManager;
        private readonly HashSet<Entity> _processedDestructions = new HashSet<Entity>();
        private readonly Dictionary<GameObject, float> _activeVFX = new Dictionary<GameObject, float>();
        private int _frameCount;
        private const int MinFrameDelay = 600;
        private const float VFXLifetime = 5.0f;
        private const float MinSizeMultiplier = 0.8f;
        private const float MaxSizeMultiplier = 1.2f;

        public static void SetPoolManager(ParticlePoolManager? poolManager)
        {
            _poolManager = poolManager;
            WriteDebug("BuildingDestructionVFXSystem.SetPoolManager: Pool initialized");
        }

        protected override void OnCreate()
        {
            base.OnCreate();
            WriteDebug("BuildingDestructionVFXSystem.OnCreate");
        }

        protected override void OnUpdate()
        {
            _frameCount++;

            if (_frameCount < MinFrameDelay)
                return;

            if (_poolManager == null)
            {
                if (_frameCount == MinFrameDelay + 1)
                    WriteDebug("BuildingDestructionVFXSystem: Pool manager not initialized, skipping");
                return;
            }

            UpdateActiveVFX();

            EntityManager em = World.DefaultGameObjectInjectionWorld.EntityManager;

            ComponentType? buildingType = global::DINOForge.Runtime.Bridge.EntityQueries.ResolveComponentType("Components.BuildingBase");
            ComponentType? healthType = global::DINOForge.Runtime.Bridge.EntityQueries.ResolveComponentType("Components.Health");

            if (buildingType == null || healthType == null)
                return;

            EntityQueryDesc desc = new EntityQueryDesc
            {
                All = new[]
                {
                    ComponentType.ReadOnly(buildingType.Value.TypeIndex),
                    ComponentType.ReadOnly(healthType.Value.TypeIndex)
                }
            };

            EntityQuery query = em.CreateEntityQuery(desc);
            using NativeArray<Entity> buildings = query.ToEntityArray(Allocator.Temp);

            try
            {
                foreach (Entity building in buildings)
                {
                    if (_processedDestructions.Contains(building))
                        continue;

                    try
                    {
                        var health = em.GetComponentData<Components.Health>(building);

                        if (health.currentHealth > 0)
                            continue;

                        _processedDestructions.Add(building);

                        bool isEnemy = em.HasComponent<Components.Enemy>(building);
                        string vfxPoolKey = isEnemy ? "BuildingCollapse_CIS" : "BuildingCollapse_Rep";

                        var position = em.GetComponentData<Unity.Transforms.Translation>(building);
                        Vector3 buildingPos = position.Value;

                        float sizeMultiplier = GetBuildingSizeMultiplier(building, em);

                        GameObject? vfxInstance = _poolManager.Get(vfxPoolKey);
                        if (vfxInstance == null)
                        {
                            WriteDebug($"BuildingDestructionVFXSystem: Pool returned null for '{vfxPoolKey}'");
                            continue;
                        }

                        vfxInstance.transform.position = buildingPos;

                        ParticleSystem? ps = vfxInstance.GetComponent<ParticleSystem>();
                        if (ps != null)
                        {
                            ps.Play();

                            var emission = ps.emission;
                            emission.rateOverTime = emission.rateOverTime.constant * sizeMultiplier;

                            _activeVFX[vfxInstance] = VFXLifetime;
                            WriteDebug($"BuildingDestructionVFXSystem: Spawned {vfxPoolKey} at {buildingPos} (scale: {sizeMultiplier:F2}x)");
                        }
                    }
                    catch (Exception ex)
                    {
                        WriteDebug($"BuildingDestructionVFXSystem: Error processing building: {ex.Message}");
                    }
                }
            }
            finally
            {
                buildings.Dispose();
            }
        }

        private float GetBuildingSizeMultiplier(Entity building, EntityManager em)
        {
            try
            {
                ComponentType? scaleType = global::DINOForge.Runtime.Bridge.EntityQueries.ResolveComponentType("Components.Scale");
                if (scaleType != null && em.HasComponent<Unity.Transforms.Scale>(building))
                {
                    var scale = em.GetComponentData<Unity.Transforms.Scale>(building);
                    float magnitude = scale.Value;
                    return Mathf.Clamp(magnitude, MinSizeMultiplier, MaxSizeMultiplier);
                }

                return 1.0f;
            }
            catch
            {
                return 1.0f;
            }
        }

        private void UpdateActiveVFX()
        {
            List<GameObject> expired = new List<GameObject>();
            float deltaTime = Time.DeltaTime;

            foreach (var kvp in _activeVFX)
            {
                GameObject vfxInstance = kvp.Key;
                float remainingLifetime = kvp.Value - deltaTime;

                if (remainingLifetime <= 0)
                {
                    expired.Add(vfxInstance);
                }
                else
                {
                    _activeVFX[vfxInstance] = remainingLifetime;
                }
            }

            foreach (GameObject vfxInstance in expired)
            {
                try
                {
                    ParticleSystem? ps = vfxInstance.GetComponent<ParticleSystem>();
                    if (ps != null)
                        ps.Stop();

                    string poolKey = vfxInstance.name.Replace("(Clone)", "").Trim();
                    _poolManager?.Return(vfxInstance, poolKey);
                    _activeVFX.Remove(vfxInstance);
                    WriteDebug($"BuildingDestructionVFXSystem: Returned {poolKey} to pool");
                }
                catch (Exception ex)
                {
                    WriteDebug($"BuildingDestructionVFXSystem: Error returning VFX to pool: {ex.Message}");
                }
            }
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.UtcNow:yyyy-MM-ddTHH:mm:ss.fffZ}] [BuildingDestructionVFXSystem] {msg}\n");
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"BuildingDestructionVFXSystem debug log write failed: {ex.Message}");
            }
        }
    }
}
