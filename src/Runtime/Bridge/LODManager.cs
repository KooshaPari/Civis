#nullable enable
using System;
using System.IO;
using Unity.Entities;
using UnityEngine;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Manages distance-based level-of-detail (LOD) culling for VFX particles.
    /// Reduces particle count and emission rates based on camera distance to improve
    /// performance in large battles with 200+ units on screen.
    ///
    /// LOD Tiers:
    /// - FULL (0–100m):     Full quality, all particles (1.0× multiplier)
    /// - MEDIUM (100–200m): Reduced particles, 60% emission (0.6× multiplier)
    /// - CULLED (200m+):    No particles spawned (0.0× multiplier)
    ///
    /// Usage:
    ///   float distance = Vector3.Distance(cameraPos, effectPos);
    ///   LODTier tier = LODManager.Instance.GetLODTier(distance);
    ///   if (tier != LODTier.CULLED)
    ///   {
    ///       ParticleSystem ps = pool.Get(prefabPath);
    ///       ps.emission.rateOverTime = baseEmissionRate * LODManager.GetEmissionMultiplier(tier);
    ///   }
    ///
    /// Singleton pattern: Access via <see cref="Instance"/>.
    /// Camera reference: Automatically queries World.DefaultGameObjectInjectionWorld.
    /// </summary>
    public class LODManager
    {
        private static LODManager? _instance;

        // Distance thresholds in world units (100 units ≈ 100 meters in DINO)
        private const float FullQualityDistance = 100f;
        private const float MediumQualityDistance = 200f;

        /// <summary>
        /// Lazy singleton accessor.
        /// </summary>
        public static LODManager Instance
        {
            get
            {
                _instance ??= new LODManager();
                return _instance;
            }
        }

        /// <summary>
        /// LOD tier enumeration.
        /// </summary>
        public enum LODTier
        {
            /// <summary>Full quality, all particles spawned (1.0× emission multiplier).</summary>
            FULL = 0,

            /// <summary>Reduced quality, 60% particles (0.6× emission multiplier).</summary>
            MEDIUM = 1,

            /// <summary>Culled, no particles spawned (0.0× emission multiplier).</summary>
            CULLED = 2
        }

        /// <summary>
        /// Determine the LOD tier for a given camera distance.
        /// </summary>
        public LODTier GetLODTier(float distance)
        {
            if (distance < FullQualityDistance)
                return LODTier.FULL;
            if (distance < MediumQualityDistance)
                return LODTier.MEDIUM;
            return LODTier.CULLED;
        }

        /// <summary>
        /// Get the emission rate multiplier for a LOD tier.
        /// Used to scale ParticleSystem.emission.rateOverTime.
        /// </summary>
        public float GetEmissionMultiplier(LODTier tier)
        {
            return tier switch
            {
                LODTier.FULL => 1.0f,
                LODTier.MEDIUM => 0.6f,
                LODTier.CULLED => 0.0f,
                _ => 1.0f
            };
        }

        /// <summary>
        /// Get the camera position from the main camera or ECS world.
        /// Falls back to Vector3.zero if camera not available.
        /// </summary>
        public Vector3 GetCameraPosition()
        {
            try
            {
                // Prefer main camera
                Camera? mainCam = Camera.main;
                if (mainCam != null)
                {
                    return mainCam.transform.position;
                }

                // Fallback: try to find a camera via ECS world
                World? world = World.DefaultGameObjectInjectionWorld;
                if (world != null && world.IsCreated)
                {
                    // Query for any active camera in the scene
                    foreach (Camera cam in Camera.allCameras)
                    {
                        if (cam.isActiveAndEnabled)
                        {
                            return cam.transform.position;
                        }
                    }
                }

                WriteDebug("LODManager.GetCameraPosition: No camera found, returning zero");
                return Vector3.zero;
            }
            catch (Exception ex)
            {
                WriteDebug($"LODManager.GetCameraPosition failed: {ex.Message}");
                return Vector3.zero;
            }
        }

        /// <summary>
        /// Calculate LOD tier for an effect at a given world position.
        /// Convenience method: combines GetCameraPosition() and GetLODTier().
        /// </summary>
        public LODTier GetLODTierForPosition(Vector3 effectPosition)
        {
            Vector3 cameraPos = GetCameraPosition();
            float distance = Vector3.Distance(cameraPos, effectPosition);
            return GetLODTier(distance);
        }

        /// <summary>
        /// Get LOD tier as an integer (0=FULL, 1=MEDIUM, 2=CULLED).
        /// Useful for shader parameters or UI displays.
        /// </summary>
        public int GetLODTierIndex(float distance)
        {
            return (int)GetLODTier(distance);
        }

        /// <summary>
        /// Get human-readable LOD tier name.
        /// </summary>
        public string GetLODTierName(LODTier tier)
        {
            return tier switch
            {
                LODTier.FULL => "FULL",
                LODTier.MEDIUM => "MEDIUM",
                LODTier.CULLED => "CULLED",
                _ => "UNKNOWN"
            };
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.UtcNow:HH:mm:ss.fff}] [LODManager] {msg}\n");
            }
            catch { } // safe-swallow: best-effort debug I/O, non-critical
        }
    }
}
