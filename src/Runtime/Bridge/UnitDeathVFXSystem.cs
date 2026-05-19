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
    /// ECS SystemBase that spawns death VFX when units die (faction-aware).
    /// </summary>
    [UpdateInGroup(typeof(PresentationSystemGroup))]
    public class UnitDeathVFXSystem : SystemBase
    {
        private static ParticlePoolManager? _poolManager;
        private readonly HashSet<Entity> _processedDeaths = new HashSet<Entity>();
        private readonly Dictionary<GameObject, float> _activeVFX = new Dictionary<GameObject, float>();
        private int _frameCount;
        private const int MinFrameDelay = 600;
        private const float VFXLifetime = 2.5f;

        public static void SetPoolManager(ParticlePoolManager? poolManager)
        {
            _poolManager = poolManager;
            WriteDebug("UnitDeathVFXSystem.SetPoolManager: Pool initialized");
        }

        protected override void OnCreate()
        {
            base.OnCreate();
            WriteDebug("UnitDeathVFXSystem.OnCreate");
        }

        protected override void OnUpdate()
        {
            _frameCount++;

            if (_frameCount < MinFrameDelay)
                return;

            if (_poolManager == null)
            {
                if (_frameCount == MinFrameDelay + 1)
                    WriteDebug("UnitDeathVFXSystem: Pool manager not initialized, skipping");
                return;
            }

            UpdateActiveVFX();

            EntityManager em = World.DefaultGameObjectInjectionWorld.EntityManager;

            ComponentType? unitType = global::DINOForge.Runtime.Bridge.EntityQueries.ResolveComponentType("Components.Unit");
            ComponentType? healthType = global::DINOForge.Runtime.Bridge.EntityQueries.ResolveComponentType("Components.Health");

            if (unitType == null || healthType == null)
                return;

            EntityQueryDesc desc = new EntityQueryDesc
            {
                All = new[]
                {
                    ComponentType.ReadOnly(unitType.Value.TypeIndex),
                    ComponentType.ReadOnly(healthType.Value.TypeIndex)
                }
            };

            EntityQuery query = em.CreateEntityQuery(desc);
            using NativeArray<Entity> units = query.ToEntityArray(Allocator.Temp);

            try
            {
                foreach (Entity unit in units)
                {
                    if (_processedDeaths.Contains(unit))
                        continue;

                    try
                    {
                        var health = em.GetComponentData<Components.Health>(unit);

                        if (health.currentHealth > 0)
                            continue;

                        _processedDeaths.Add(unit);

                        bool isEnemy = em.HasComponent<Components.Enemy>(unit);
                        string vfxPoolKey = isEnemy ? "UnitDeathVFX_CIS" : "UnitDeathVFX_Rep";

                        var position = em.GetComponentData<Unity.Transforms.Translation>(unit);
                        Vector3 unitPos = position.Value;

                        GameObject? vfxInstance = _poolManager.Get(vfxPoolKey);
                        if (vfxInstance == null)
                        {
                            WriteDebug($"UnitDeathVFXSystem: Pool returned null for '{vfxPoolKey}'");
                            continue;
                        }

                        vfxInstance.transform.position = unitPos;

                        ParticleSystem? ps = vfxInstance.GetComponent<ParticleSystem>();
                        if (ps != null)
                        {
                            ps.Play();
                            _activeVFX[vfxInstance] = VFXLifetime;
                            WriteDebug($"UnitDeathVFXSystem: Spawned {vfxPoolKey} at {unitPos}");
                        }
                    }
                    catch (Exception ex)
                    {
                        WriteDebug($"UnitDeathVFXSystem: Error processing unit: {ex.Message}");
                    }
                }
            }
            finally
            {
                units.Dispose();
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
                    WriteDebug($"UnitDeathVFXSystem: Returned {poolKey} to pool");
                }
                catch (Exception ex)
                {
                    WriteDebug($"UnitDeathVFXSystem: Error returning VFX to pool: {ex.Message}");
                }
            }
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.UtcNow:yyyy-MM-ddTHH:mm:ss.fffZ}] [UnitDeathVFXSystem] {msg}\n");
            }
            catch { } // safe-swallow: best-effort debug I/O, non-critical
        }
    }
}
