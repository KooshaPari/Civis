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
    /// ECS SystemBase that spawns impact VFX when projectiles hit targets.
    /// </summary>
    [UpdateInGroup(typeof(PresentationSystemGroup))]
    public class ProjectileVFXSystem : SystemBase
    {
        private static ParticlePoolManager? _poolManager;
        private EntityQuery _projectileQuery;
        private bool _queryInitialized;
        private int _frameCount;
        private const int MinFrameDelay = 600;

        public static void SetPoolManager(ParticlePoolManager? poolManager)
        {
            _poolManager = poolManager;
            DebugLog.Write("ProjectileVFX", "SetPoolManager: Pool initialized");
        }

        public override void OnCreate()
        {
            base.OnCreate();
            DebugLog.Write("ProjectileVFX", "OnCreate");
        }

        public override void OnUpdate()
        {
            _frameCount++;

            if (_frameCount < MinFrameDelay)
                return;

            if (_poolManager == null)
            {
                if (_frameCount == MinFrameDelay + 1)
                    DebugLog.Write("ProjectileVFX", "Pool manager not initialized, skipping");
                return;
            }

            EntityManager em = World.DefaultGameObjectInjectionWorld.EntityManager;

            if (!_queryInitialized)
            {
                // Note: EntityQueries is in the same namespace (DINOForge.Runtime.Bridge)
                ComponentType? projectileType = global::DINOForge.Runtime.Bridge.EntityQueries.ResolveComponentType("Components.ProjectileDataBase");
                if (projectileType == null)
                    return;

                EntityQueryDesc desc = new EntityQueryDesc
                {
                    All = new[] { ComponentType.ReadOnly(projectileType.Value.TypeIndex) }
                };

                _projectileQuery = em.CreateEntityQuery(desc);
                _queryInitialized = true;
            }

            using NativeArray<Entity> projectiles = _projectileQuery.ToEntityArray(Allocator.Temp);

            try
            {
                foreach (Entity projectile in projectiles)
                {
                    try
                    {
                        ComponentType? translationType = global::DINOForge.Runtime.Bridge.EntityQueries.ResolveComponentType("Components.Translation");
                        if (translationType == null) continue;

                        bool isOwnerEnemy = em.HasComponent<Components.Enemy>(projectile);
                        string vfxPoolKey = isOwnerEnemy ? "BlasterImpact_CIS" : "BlasterImpact_Rep";

                        var translationData = em.GetComponentData<Unity.Transforms.Translation>(projectile);
                        Vector3 impactPos = translationData.Value;

                        GameObject? vfxInstance = _poolManager.Get(vfxPoolKey);
                        if (vfxInstance == null)
                        {
                            DebugLog.Write("ProjectileVFX", $"Pool returned null for '{vfxPoolKey}'");
                            continue;
                        }

                        vfxInstance.transform.position = impactPos;

                        ParticleSystem? ps = vfxInstance.GetComponent<ParticleSystem>();
                        if (ps != null)
                        {
                            ps.Play();
                            DebugLog.Write("ProjectileVFX", $"Spawned {vfxPoolKey} at {impactPos}");
                        }
                    }
                    catch (Exception ex)
                    {
                        DebugLog.Write("ProjectileVFX", $"Error processing projectile: {ex.Message}");
                    }
                }
            }
            finally
            {
                projectiles.Dispose();
            }
        }

    }

    public interface IParticlePoolManager
    {
        GameObject? Get(string poolKey);
        void Return(GameObject instance, string poolKey);
    }

    public class ParticlePoolManager : IParticlePoolManager
    {
        private readonly Dictionary<string, Queue<GameObject>> _pools =
            new Dictionary<string, Queue<GameObject>>(StringComparer.OrdinalIgnoreCase);

        public void Register(string poolKey, GameObject prefab, int poolSize)
        {
            if (string.IsNullOrEmpty(poolKey)) throw new ArgumentNullException(nameof(poolKey));
            if (prefab == null) throw new ArgumentNullException(nameof(prefab));

            _pools[poolKey] = new Queue<GameObject>(poolSize);

            for (int i = 0; i < poolSize; i++)
            {
                GameObject instance = UnityEngine.Object.Instantiate(prefab);
                instance.SetActive(false);
                _pools[poolKey].Enqueue(instance);
            }
        }

        public GameObject? Get(string poolKey)
        {
            if (!_pools.TryGetValue(poolKey, out Queue<GameObject>? pool))
                return null;

            if (pool.Count > 0)
            {
                GameObject instance = pool.Dequeue();
                instance.SetActive(true);
                return instance;
            }

            return null;
        }

        public void Return(GameObject instance, string poolKey)
        {
            if (instance == null || !_pools.TryGetValue(poolKey, out Queue<GameObject>? pool))
                return;

            instance.SetActive(false);
            instance.transform.position = Vector3.zero;
            pool.Enqueue(instance);
        }

        public void Dispose()
        {
            foreach (Queue<GameObject> pool in _pools.Values)
            {
                foreach (GameObject instance in pool)
                {
                    if (instance != null)
                        UnityEngine.Object.Destroy(instance);
                }
            }
            _pools.Clear();
        }
    }
}
