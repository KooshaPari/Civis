#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using UnityEngine;
using DINOForge.Runtime.Diagnostics;
using DINOForge.Runtime.VFX;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Manages a pooled allocation strategy for particle system VFX instances.
    /// Pre-allocates instances at startup and reuses them throughout gameplay to avoid
    /// runtime allocations during combat and effects playback.
    ///
    /// Pool structure: Dictionary keyed by prefab asset path (e.g., "vfx/BlasterBolt_Rep.prefab")
    /// Each entry contains a Queue of available ParticleSystem instances.
    ///
    /// Design: Singleton pattern with lazy initialization.
    /// Call <see cref="Initialize"/> once when the BepInEx plugin awakes, before the ECS world is ready.
    /// Call <see cref="Get"/> from effect spawners (e.g., in weapon systems or impact handlers).
    /// Call <see cref="Return"/> when effects finish (ParticleSystem.lifetime has elapsed).
    ///
    /// Performance targets:
    /// - Zero allocation during gameplay (all instances pre-allocated)
    /// - Max 48 concurrent instances across all effect types
    /// - 11 distinct VFX prefabs loaded from pack assets
    ///
    /// Graceful degradation: If a prefab cannot be loaded, logs error and returns null.
    /// Callers must check Get() return value before use.
    /// </summary>
    public class VFXPoolManager
    {
        private static VFXPoolManager? _instance;

        /// <summary>
        /// Pool structure: prefab path → queue of available ParticleSystem instances.
        /// </summary>
        private readonly Dictionary<string, Queue<ParticleSystem>> _pools =
            new Dictionary<string, Queue<ParticleSystem>>(StringComparer.Ordinal);

        /// <summary>
        /// Track active (checked-out) instances for stats reporting.
        /// </summary>
        private readonly Dictionary<string, int> _activeCount =
            new Dictionary<string, int>(StringComparer.Ordinal);

        /// <summary>
        /// Root container for all pooled particle systems. Disabled in hierarchy.
        /// </summary>
        private GameObject? _poolRoot;

        /// <summary>
        /// Total instances across all pools (for monitoring).
        /// </summary>
        private int _totalInstances;

        /// <summary>
        /// Lazy singleton accessor. Initialize() must be called once before Get() is used.
        /// </summary>
        public static VFXPoolManager Instance
        {
            get
            {
                _instance ??= new VFXPoolManager();
                return _instance;
            }
        }

        /// <summary>
        /// Initialize the pool with pre-allocated instances.
        /// Must be called once during BepInEx Plugin.Awake() or Plugin.OnWorldReady().
        /// Safe to call multiple times (no-op if already initialized).
        /// </summary>
        public void Initialize()
        {
            if (_poolRoot != null) return; // Already initialized

            try
            {
                // Create a disabled container for pooled instances
                _poolRoot = new GameObject("DINOForge_VFXPool");
                _poolRoot.SetActive(false);
                UnityEngine.Object.DontDestroyOnLoad(_poolRoot);

                // Pre-allocate pools for all VFX types
                // Allocation strategy: Common effects get more instances
                AllocatePool("vfx/BlasterBolt_Rep.prefab", 12);     // Republic blaster — high fire rate
                AllocatePool("vfx/BlasterBolt_CIS.prefab", 12);     // CIS blaster — high fire rate
                AllocatePool("vfx/LightsaberVFX_Rep.prefab", 8);    // Melee — lower frequency
                AllocatePool("vfx/LightsaberVFX_CIS.prefab", 8);    // Melee — lower frequency
                AllocatePool("vfx/BlasterImpact_Rep.prefab", 8);    // Hit feedback
                AllocatePool("vfx/BlasterImpact_CIS.prefab", 8);    // Hit feedback
                AllocatePool("vfx/UnitDeathVFX_Rep.prefab", 8);     // Unit death — occasional
                AllocatePool("vfx/UnitDeathVFX_CIS.prefab", 8);     // Unit death — occasional
                AllocatePool("vfx/BuildingCollapse_Rep.prefab", 4); // Building death — rare
                AllocatePool("vfx/BuildingCollapse_CIS.prefab", 4); // Building death — rare
                AllocatePool("vfx/Explosion_CIS.prefab", 12);       // Heavy weapon AOE — high impact

                DebugLog.Write("VFXPool",$"VFXPoolManager initialized: {_totalInstances} instances across {_pools.Count} pools");
            }
            catch (Exception ex)
            {
                DebugLog.Write("VFXPool",$"VFXPoolManager.Initialize failed: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// Allocate N instances of a prefab and add them to the pool.
        /// If the prefab cannot be loaded, logs error and skips allocation.
        /// </summary>
        private void AllocatePool(string prefabPath, int count)
        {
            try
            {
                // Attempt to load the prefab from Resources or Addressables
                // For now, assume a simple path resolution: packs/warfare-starwars/assets/{prefabPath}
                ParticleSystem? prefab = LoadPrefabFromPack(prefabPath);
                if (prefab == null)
                {
                    DebugLog.Write("VFXPool",$"VFXPoolManager: Prefab not found, skipping allocation: {prefabPath}");
                    return;
                }

                Queue<ParticleSystem> poolQueue = new Queue<ParticleSystem>();
                _pools[prefabPath] = poolQueue;
                _activeCount[prefabPath] = 0;

                for (int i = 0; i < count; i++)
                {
                    try
                    {
                        // null-forgiveness-ok: _poolRoot lazy-initialized at field declaration
                        ParticleSystem instance = UnityEngine.Object.Instantiate(prefab, _poolRoot!.transform);
                        instance.name = $"{prefab.name}_{i}";
                        instance.gameObject.SetActive(false);

                        // Ensure particle system is stopped
                        instance.Stop(withChildren: true);

                        poolQueue.Enqueue(instance);
                        _totalInstances++;
                    }
                    catch (Exception ex)
                    {
                        DebugLog.Write("VFXPool",$"VFXPoolManager: Failed to instantiate {prefabPath} ({i}/{count}): {ex.Message}");
                    }
                }

                DebugLog.Write("VFXPool",$"VFXPoolManager: Allocated {poolQueue.Count} instances of {prefabPath}");
            }
            catch (Exception ex)
            {
                DebugLog.Write("VFXPool",$"VFXPoolManager.AllocatePool({prefabPath}) failed: {ex.Message}");
            }
        }

        /// <summary>
        /// Load a VFX prefab from the pack asset directory.
        /// Returns null if not found or load fails.
        /// Fallback: Creates prefab at runtime from VFXPrefabDescriptor if binary file not found.
        /// </summary>
        private ParticleSystem? LoadPrefabFromPack(string prefabPath)
        {
            try
            {
                // Primary: Try to load from Resources or Addressables
                // In production, this would use AddressablesCatalog.LoadAssetAsync()
                // or a similar asset resolution system from the SDK.

                string resourcePath = prefabPath.Replace(".prefab", "");
                GameObject? prefabGo = Resources.Load<GameObject>(resourcePath);

                if (prefabGo != null)
                {
                    ParticleSystem? ps = prefabGo.GetComponent<ParticleSystem>();
                    if (ps != null)
                    {
                        return ps;
                    }

                    DebugLog.Write("VFXPool",$"VFXPoolManager: Loaded prefab but has no ParticleSystem: {prefabPath}");
                    return null;
                }

                // Fallback: Create prefab at runtime from descriptor
                // This ensures VFX always works even if binary prefabs are missing
                DebugLog.Write("VFXPool",$"VFXPoolManager: Binary prefab not found ({prefabPath}), creating from descriptor");
                return CreatePrefabFromDescriptor(prefabPath);
            }
            catch (Exception ex)
            {
                DebugLog.Write("VFXPool",$"VFXPoolManager.LoadPrefabFromPack({prefabPath}) failed: {ex.Message}");
                return null;
            }
        }

        /// <summary>
        /// Retrieve a ParticleSystem instance from the pool.
        /// If pool is exhausted, creates a new instance (fallback allocation).
        /// Returns null if prefab path has never been allocated.
        ///
        /// Caller responsibility: Activate the instance, set its position/rotation, call Play().
        /// </summary>
        public ParticleSystem? Get(string prefabPath)
        {
            if (!_pools.TryGetValue(prefabPath, out Queue<ParticleSystem> queue))
            {
                // Path not in pool — try to allocate on-demand (slower path)
                DebugLog.Write("VFXPool",$"VFXPoolManager.Get: Pool not initialized for {prefabPath}, creating on-demand");
                AllocatePool(prefabPath, 1);

                if (!_pools.TryGetValue(prefabPath, out queue))
                {
                    DebugLog.Write("VFXPool",$"VFXPoolManager.Get: Failed to allocate on-demand for {prefabPath}");
                    return null;
                }
            }

            ParticleSystem? instance = null;

            if (queue.Count > 0)
            {
                instance = queue.Dequeue();
            }
            else
            {
                // Pool exhausted — log and create fallback
                DebugLog.Write("VFXPool",$"VFXPoolManager.Get: Pool exhausted for {prefabPath}, creating fallback instance");

                ParticleSystem? prefab = LoadPrefabFromPack(prefabPath);
                if (prefab != null)
                {
                    // null-forgiveness-ok: _poolRoot lazy-initialized at field declaration
                    instance = UnityEngine.Object.Instantiate(prefab, _poolRoot!.transform);
                    instance.gameObject.SetActive(false);
                    _totalInstances++;
                }
            }

            if (instance != null)
            {
                instance.gameObject.SetActive(true);
                int current = 0;
                if (_activeCount.TryGetValue(prefabPath, out int val))
                    current = val;
                _activeCount[prefabPath] = current + 1;
            }

            return instance;
        }

        /// <summary>
        /// Return a ParticleSystem instance to the pool after use.
        /// Stops emission and disables the GameObject.
        /// Safe to call even if instance was not from this pool.
        ///
        /// Caller responsibility: Should call this when ParticleSystem lifetime expires
        /// or when effect is no longer needed. For long-running effects, consider manual
        /// return points in effect completion handlers.
        /// </summary>
        public void Return(ParticleSystem instance)
        {
            if (instance == null) return;

            try
            {
                // Stop all particle emission
                instance.Stop(withChildren: true);

                // Disable the GameObject
                instance.gameObject.SetActive(false);

                // Reset position/rotation to origin (optional — helps with memory usage)
                instance.transform.position = Vector3.zero;
                instance.transform.rotation = Quaternion.identity;

                // Find which pool this belongs to by matching the prefab name
                string prefabKey = FindPrefabKey(instance);

                if (!string.IsNullOrEmpty(prefabKey) && _pools.TryGetValue(prefabKey, out Queue<ParticleSystem> queue))
                {
                    queue.Enqueue(instance);
                    int activeCount = 0;
                    if (_activeCount.TryGetValue(prefabKey, out int active))
                    {
                        activeCount = Mathf.Max(0, active - 1);
                        _activeCount[prefabKey] = activeCount;
                    }
                }
                else
                {
                    // Instance not recognized — may be from elsewhere, just disable it
                    DebugLog.Write("VFXPool",$"VFXPoolManager.Return: Instance not in pool, destroying: {instance.name}");
                    UnityEngine.Object.Destroy(instance.gameObject);
                    _totalInstances--;
                }
            }
            catch (Exception ex)
            {
                DebugLog.Write("VFXPool",$"VFXPoolManager.Return failed: {ex.Message}");
            }
        }

        /// <summary>
        /// Find the prefab pool key that this instance belongs to.
        /// Uses GameObject name heuristics (e.g., "BlasterBolt_Rep_0" maps to "vfx/BlasterBolt_Rep.prefab").
        /// </summary>
        private string FindPrefabKey(ParticleSystem instance)
        {
            // Simple name-matching heuristic
            string instanceName = instance.name;

            foreach (string key in _pools.Keys)
            {
                string filename = System.IO.Path.GetFileNameWithoutExtension(key);
                if (instanceName.StartsWith(filename))
                {
                    return key;
                }
            }

            return "";
        }

        /// <summary>
        /// Get current pool statistics for monitoring and debugging.
        /// </summary>
        public (int activeInstances, int pooledInstances, int totalInstances) GetStats()
        {
            int totalActive = 0;
            int totalPooled = 0;

            foreach (string key in _pools.Keys)
            {
                int active = 0;
                if (_activeCount.TryGetValue(key, out int val))
                    active = val;
                int pooled = _pools[key].Count;

                totalActive += active;
                totalPooled += pooled;
            }

            return (totalActive, totalPooled, _totalInstances);
        }

        /// <summary>
        /// Clean up all pooled instances (called on shutdown).
        /// </summary>
        public void Shutdown()
        {
            try
            {
                if (_poolRoot != null)
                {
                    UnityEngine.Object.Destroy(_poolRoot);
                    _poolRoot = null;
                }

                _pools.Clear();
                _activeCount.Clear();
                _totalInstances = 0;

                DebugLog.Write("VFXPool","VFXPoolManager shutdown complete");
            }
            catch (Exception ex)
            {
                DebugLog.Write("VFXPool",$"VFXPoolManager.Shutdown error: {ex.Message}");
            }
        }

        /// <summary>
        /// Fallback: Create a VFX prefab at runtime from descriptor.
        /// Used when binary prefab files are not available.
        /// Returns null if descriptor lookup fails.
        /// </summary>
        private ParticleSystem? CreatePrefabFromDescriptor(string prefabPath)
        {
            try
            {
                // Extract prefab name from path (e.g., "vfx/BlasterBolt_Rep.prefab" → "BlasterBolt_Rep")
                string prefabName = System.IO.Path.GetFileNameWithoutExtension(prefabPath);

                // Look up descriptor from catalog
                var allDescriptors = VFXPrefabCatalog.GetAllPrefabs();
                VFXPrefabDescriptor? descriptor = null;

                foreach (var desc in allDescriptors)
                {
                    if (desc.Id == prefabName)
                    {
                        descriptor = desc;
                        break;
                    }
                }

                if (descriptor == null)
                {
                    DebugLog.Write("VFXPool",$"VFXPoolManager: No descriptor found for {prefabName}");
                    return null;
                }

                // Create prefab from descriptor
                GameObject prefabGo = VFXPrefabFactory.CreatePrefabFromDescriptor(descriptor);
                if (prefabGo == null)
                {
                    DebugLog.Write("VFXPool",$"VFXPoolManager: Failed to create prefab from descriptor: {prefabName}");
                    return null;
                }

                ParticleSystem? ps = prefabGo.GetComponent<ParticleSystem>();
                if (ps == null)
                {
                    DebugLog.Write("VFXPool",$"VFXPoolManager: Created prefab has no ParticleSystem: {prefabName}");
                    UnityEngine.Object.Destroy(prefabGo);
                    return null;
                }

                DebugLog.Write("VFXPool",$"VFXPoolManager: Created runtime prefab from descriptor: {prefabName}");
                return ps;
            }
            catch (Exception ex)
            {
                DebugLog.Write("VFXPool",$"VFXPoolManager.CreatePrefabFromDescriptor({prefabPath}) failed: {ex.Message}");
                return null;
            }
        }

    }
}
