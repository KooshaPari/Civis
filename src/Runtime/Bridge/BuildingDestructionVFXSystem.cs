#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.Runtime.Diagnostics;
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
        private EntityQuery _buildingQuery;
        private bool _queryInitialized;
        private int _frameCount;
        private const int MinFrameDelay = 600;
        private const float VFXLifetime = 5.0f;
        private const float MinSizeMultiplier = 0.8f;
        private const float MaxSizeMultiplier = 1.2f;

        public static void SetPoolManager(ParticlePoolManager? poolManager)
        {
            _poolManager = poolManager;
            DebugLog.Write("BuildingDestructionVFX", "SetPoolManager: Pool initialized");
        }

        protected override void OnCreate()
        {
            base.OnCreate();
            DebugLog.Write("BuildingDestructionVFX", "OnCreate");
        }

        protected override void OnUpdate()
        {
            _frameCount++;

            if (_frameCount < MinFrameDelay)
                return;

            if (_poolManager == null)
            {
                if (_frameCount == MinFrameDelay + 1)
                    DebugLog.Write("BuildingDestructionVFX", "Pool manager not initialized, skipping");
                return;
            }

            UpdateActiveVFX();

            EntityManager em = World.DefaultGameObjectInjectionWorld.EntityManager;

            if (!_queryInitialized)
            {
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

                _buildingQuery = em.CreateEntityQuery(desc);
                _queryInitialized = true;
            }

            using NativeArray<Entity> buildings = _buildingQuery.ToEntityArray(Allocator.Temp);

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
                            DebugLog.Write("BuildingDestructionVFX", $"Pool returned null for '{vfxPoolKey}'");
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
                            DebugLog.Write("BuildingDestructionVFX", $"Spawned {vfxPoolKey} at {buildingPos} (scale: {sizeMultiplier:F2}x)");
                        }
                    }
                    catch (Exception ex)
                    {
                        DebugLog.Write("BuildingDestructionVFX", $"Error processing building: {ex.Message}");
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
            catch (Exception)
            {
                // safe-swallow: lifetime read failed, default to full life
                return 1.0f;
            }
        }

        private void UpdateActiveVFX()
        {
            List<GameObject> expired = new List<GameObject>(_activeVFX.Count);
            List<KeyValuePair<GameObject, float>> updates = new List<KeyValuePair<GameObject, float>>(_activeVFX.Count);
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
                    updates.Add(new KeyValuePair<GameObject, float>(vfxInstance, remainingLifetime));
                }
            }

            // Apply updates after iteration to avoid mid-enumeration dictionary mutation (Pattern #51)
            foreach (var update in updates)
            {
                _activeVFX[update.Key] = update.Value;
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
                    DebugLog.Write("BuildingDestructionVFX", $"Returned {poolKey} to pool");
                }
                catch (Exception ex)
                {
                    DebugLog.Write("BuildingDestructionVFX", $"Error returning VFX to pool: {ex.Message}");
                }
            }
        }

    }
}
