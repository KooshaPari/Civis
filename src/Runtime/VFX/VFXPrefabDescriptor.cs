#nullable enable
using System;
using System.Collections.Generic;
using UnityEngine;

namespace DINOForge.Runtime.VFX
{
    /// <summary>
    /// Immutable descriptor for VFX prefab configuration.
    /// Allows VFX prefabs to be created at runtime without the Unity Editor,
    /// by storing all ParticleSystem and material configuration as data.
    ///
    /// This is used as a fallback when prefab binary files are not available,
    /// allowing the VFXPoolManager to reconstruct prefabs from metadata.
    ///
    /// Design: Serializable, can be persisted as JSON or YAML for pack distribution.
    /// </summary>
    [System.Serializable]
    public class VFXPrefabDescriptor
    {
        /// <summary>
        /// Unique identifier matching the prefab filename (e.g., "BlasterBolt_Rep").
        /// </summary>
        public string Id { get; private set; }

        /// <summary>
        /// Human-readable name for debugging.
        /// </summary>
        public string DisplayName { get; private set; }

        /// <summary>
        /// Faction tag: "rep" (Republic) or "cis" (CIS).
        /// </summary>
        public string Faction { get; private set; }

        /// <summary>
        /// VFX effect category.
        /// </summary>
        public VFXEffectType EffectType { get; private set; }

        /// <summary>
        /// ParticleSystem configuration.
        /// </summary>
        public ParticleSystemConfig ParticleConfig { get; private set; }

        /// <summary>
        /// Material and color configuration.
        /// </summary>
        public MaterialConfig MaterialConfig { get; private set; }

        /// <summary>
        /// Optional LOD configuration for performance scaling.
        /// </summary>
        public LODConfig? LODConfig { get; private set; }

        public VFXPrefabDescriptor(
            string id,
            string displayName,
            string faction,
            VFXEffectType effectType,
            ParticleSystemConfig particleConfig,
            MaterialConfig materialConfig,
            LODConfig? lodConfig = null)
        {
            Id = id;
            DisplayName = displayName;
            Faction = faction;
            EffectType = effectType;
            ParticleConfig = particleConfig;
            MaterialConfig = materialConfig;
            LODConfig = lodConfig;
        }

        public enum VFXEffectType
        {
            BlasterBolt,      // Projectile trail
            LightsaberVFX,    // Melee swing
            BlasterImpact,    // Hit burst
            UnitDeathVFX,     // Unit destruction
            BuildingCollapse, // Building destruction
            Explosion         // Large explosion
        }
    }

    /// <summary>
    /// ParticleSystem component configuration.
    /// </summary>
    [System.Serializable]
    public class ParticleSystemConfig
    {
        public float Duration = 0.5f;
        public bool Loop = false;
        public float StartLifetime = 0.3f;
        public float StartSpeed = 10.0f;
        public float StartSize = 0.1f;
        public float GravityModifier = 0f;
        public int MaxParticles = 80;

        public float EmissionRateOverTime = 50f;
        public ParticleShapeConfig Shape = new ParticleShapeConfig();
        public ParticleColorConfig Color = new ParticleColorConfig();

        public ParticleSystemConfig() { }

        public ParticleSystemConfig(
            float duration,
            float startLifetime,
            float startSpeed,
            float startSize,
            int maxParticles,
            float emissionRate,
            float gravityMod = 0f)
        {
            Duration = duration;
            StartLifetime = startLifetime;
            StartSpeed = startSpeed;
            StartSize = startSize;
            MaxParticles = maxParticles;
            EmissionRateOverTime = emissionRate;
            GravityModifier = gravityMod;
        }
    }

    [System.Serializable]
    public class ParticleShapeConfig
    {
        public ParticleSystemShapeType ShapeType = ParticleSystemShapeType.Cone;
        public float Angle = 45f;
        public float Radius = 0.1f;
    }

    [System.Serializable]
    public class ParticleColorConfig
    {
        /// <summary>
        /// Primary faction color (start of lifetime).
        /// </summary>
        public Color PrimaryColor = Color.white;

        /// <summary>
        /// Secondary color (end of lifetime, transparent).
        /// </summary>
        public Color SecondaryColor = Color.white;

        public ParticleColorConfig() { }

        public ParticleColorConfig(Color primary, Color secondary)
        {
            PrimaryColor = primary;
            SecondaryColor = secondary;
        }
    }

    /// <summary>
    /// Material and shader configuration.
    /// </summary>
    [System.Serializable]
    public class MaterialConfig
    {
        public string ShaderName = "Particles/Standard Unlit";
        public Color BaseColor = Color.white;
        public float EmissionIntensity = 2.0f;
        public int RenderQueue = 3000;
    }

    /// <summary>
    /// LOD configuration for particle system scaling.
    /// </summary>
    [System.Serializable]
    public class LODConfig
    {
        /// <summary>
        /// Max particles for MEDIUM LOD level (percentage of base).
        /// </summary>
        public float MediumLODScale = 0.6f;

        /// <summary>
        /// Max particles for LOW LOD level (percentage of base).
        /// </summary>
        public float LowLODScale = 0.3f;

        public LODConfig() { }

        public LODConfig(float mediumScale, float lowScale)
        {
            MediumLODScale = mediumScale;
            LowLODScale = lowScale;
        }
    }

    /// <summary>
    /// Static catalog of all 11 VFX prefab descriptors.
    /// Provides design-time metadata for runtime prefab reconstruction.
    /// </summary>
    public static class VFXPrefabCatalog
    {
        // Faction colors (matching ASSET_PIPELINE.md)
        private static readonly Color RepublicBlue = new Color(0.267f, 0.533f, 1.0f, 1.0f);      // #4488FF
        private static readonly Color RepublicAccent = new Color(0.392f, 0.627f, 0.863f, 1.0f);   // #64A0DC
        private static readonly Color CISRed = new Color(1.0f, 0.267f, 0.0f, 1.0f);                // #FF4400
        private static readonly Color CISAccent = new Color(0.702f, 0.353f, 0.0f, 1.0f);           // #B35A00

        public static readonly VFXPrefabDescriptor BlasterBoltRep = new VFXPrefabDescriptor(
            id: "BlasterBolt_Rep",
            displayName: "Republic Blaster Bolt",
            faction: "rep",
            effectType: VFXPrefabDescriptor.VFXEffectType.BlasterBolt,
            particleConfig: new ParticleSystemConfig(
                duration: 0.5f,
                startLifetime: 0.3f,
                startSpeed: 20.0f,
                startSize: 0.1f,
                maxParticles: 80,
                emissionRate: 50f,
                gravityMod: 0f),
            materialConfig: new MaterialConfig
            {
                BaseColor = RepublicBlue,
                EmissionIntensity = 2.0f
            },
            lodConfig: new LODConfig(0.6f, 0.3f));

        public static readonly VFXPrefabDescriptor BlasterBoltCIS = new VFXPrefabDescriptor(
            id: "BlasterBolt_CIS",
            displayName: "CIS Blaster Bolt",
            faction: "cis",
            effectType: VFXPrefabDescriptor.VFXEffectType.BlasterBolt,
            particleConfig: new ParticleSystemConfig(
                duration: 0.5f,
                startLifetime: 0.3f,
                startSpeed: 20.0f,
                startSize: 0.1f,
                maxParticles: 80,
                emissionRate: 50f,
                gravityMod: 0f),
            materialConfig: new MaterialConfig
            {
                BaseColor = CISRed,
                EmissionIntensity = 2.0f
            },
            lodConfig: new LODConfig(0.6f, 0.3f));

        public static readonly VFXPrefabDescriptor LightsaberVFXRep = new VFXPrefabDescriptor(
            id: "LightsaberVFX_Rep",
            displayName: "Republic Lightsaber VFX",
            faction: "rep",
            effectType: VFXPrefabDescriptor.VFXEffectType.LightsaberVFX,
            particleConfig: new ParticleSystemConfig(
                duration: 0.6f,
                startLifetime: 0.4f,
                startSpeed: 5.0f,
                startSize: 0.1f,
                maxParticles: 100,
                emissionRate: 40f,
                gravityMod: 0f),
            materialConfig: new MaterialConfig
            {
                BaseColor = RepublicBlue,
                EmissionIntensity = 2.0f
            },
            lodConfig: new LODConfig(0.6f, 0.3f));

        public static readonly VFXPrefabDescriptor LightsaberVFXCIS = new VFXPrefabDescriptor(
            id: "LightsaberVFX_CIS",
            displayName: "CIS Lightsaber VFX",
            faction: "cis",
            effectType: VFXPrefabDescriptor.VFXEffectType.LightsaberVFX,
            particleConfig: new ParticleSystemConfig(
                duration: 0.6f,
                startLifetime: 0.4f,
                startSpeed: 5.0f,
                startSize: 0.1f,
                maxParticles: 100,
                emissionRate: 40f,
                gravityMod: 0f),
            materialConfig: new MaterialConfig
            {
                BaseColor = CISRed,
                EmissionIntensity = 2.0f
            },
            lodConfig: new LODConfig(0.6f, 0.3f));

        public static readonly VFXPrefabDescriptor BlasterImpactRep = new VFXPrefabDescriptor(
            id: "BlasterImpact_Rep",
            displayName: "Republic Blaster Impact",
            faction: "rep",
            effectType: VFXPrefabDescriptor.VFXEffectType.BlasterImpact,
            particleConfig: new ParticleSystemConfig(
                duration: 0.3f,
                startLifetime: 0.25f,
                startSpeed: 3.0f,
                startSize: 0.075f,
                maxParticles: 50,
                emissionRate: 100f,
                gravityMod: 0.1f),
            materialConfig: new MaterialConfig
            {
                BaseColor = RepublicBlue,
                EmissionIntensity = 2.0f
            },
            lodConfig: new LODConfig(0.6f, 0.4f));

        public static readonly VFXPrefabDescriptor BlasterImpactCIS = new VFXPrefabDescriptor(
            id: "BlasterImpact_CIS",
            displayName: "CIS Blaster Impact",
            faction: "cis",
            effectType: VFXPrefabDescriptor.VFXEffectType.BlasterImpact,
            particleConfig: new ParticleSystemConfig(
                duration: 0.3f,
                startLifetime: 0.25f,
                startSpeed: 3.0f,
                startSize: 0.075f,
                maxParticles: 50,
                emissionRate: 100f,
                gravityMod: 0.1f),
            materialConfig: new MaterialConfig
            {
                BaseColor = CISRed,
                EmissionIntensity = 2.0f
            },
            lodConfig: new LODConfig(0.6f, 0.4f));

        public static readonly VFXPrefabDescriptor UnitDeathVFXRep = new VFXPrefabDescriptor(
            id: "UnitDeathVFX_Rep",
            displayName: "Republic Unit Death",
            faction: "rep",
            effectType: VFXPrefabDescriptor.VFXEffectType.UnitDeathVFX,
            particleConfig: new ParticleSystemConfig(
                duration: 0.8f,
                startLifetime: 0.6f,
                startSpeed: 2.0f,
                startSize: 0.175f,
                maxParticles: 120,
                emissionRate: 80f,
                gravityMod: -0.05f),
            materialConfig: new MaterialConfig
            {
                BaseColor = RepublicBlue,
                EmissionIntensity = 2.0f
            },
            lodConfig: new LODConfig(0.6f, 0.3f));

        public static readonly VFXPrefabDescriptor UnitDeathVFXCIS = new VFXPrefabDescriptor(
            id: "UnitDeathVFX_CIS",
            displayName: "CIS Unit Death",
            faction: "cis",
            effectType: VFXPrefabDescriptor.VFXEffectType.UnitDeathVFX,
            particleConfig: new ParticleSystemConfig(
                duration: 0.8f,
                startLifetime: 0.6f,
                startSpeed: 2.0f,
                startSize: 0.175f,
                maxParticles: 120,
                emissionRate: 80f,
                gravityMod: -0.05f),
            materialConfig: new MaterialConfig
            {
                BaseColor = CISRed,
                EmissionIntensity = 2.0f
            },
            lodConfig: new LODConfig(0.6f, 0.3f));

        public static readonly VFXPrefabDescriptor BuildingCollapseRep = new VFXPrefabDescriptor(
            id: "BuildingCollapse_Rep",
            displayName: "Republic Building Collapse",
            faction: "rep",
            effectType: VFXPrefabDescriptor.VFXEffectType.BuildingCollapse,
            particleConfig: new ParticleSystemConfig(
                duration: 1.0f,
                startLifetime: 0.8f,
                startSpeed: 1.0f,
                startSize: 0.35f,
                maxParticles: 150,
                emissionRate: 60f,
                gravityMod: -0.3f),
            materialConfig: new MaterialConfig
            {
                BaseColor = RepublicBlue,
                EmissionIntensity = 1.5f
            },
            lodConfig: new LODConfig(0.6f, 0.3f));

        public static readonly VFXPrefabDescriptor BuildingCollapseCIS = new VFXPrefabDescriptor(
            id: "BuildingCollapse_CIS",
            displayName: "CIS Building Collapse",
            faction: "cis",
            effectType: VFXPrefabDescriptor.VFXEffectType.BuildingCollapse,
            particleConfig: new ParticleSystemConfig(
                duration: 1.0f,
                startLifetime: 0.8f,
                startSpeed: 1.0f,
                startSize: 0.35f,
                maxParticles: 150,
                emissionRate: 60f,
                gravityMod: -0.3f),
            materialConfig: new MaterialConfig
            {
                BaseColor = CISRed,
                EmissionIntensity = 1.5f
            },
            lodConfig: new LODConfig(0.6f, 0.3f));

        public static readonly VFXPrefabDescriptor ExplosionCIS = new VFXPrefabDescriptor(
            id: "Explosion_CIS",
            displayName: "CIS Large Explosion",
            faction: "cis",
            effectType: VFXPrefabDescriptor.VFXEffectType.Explosion,
            particleConfig: new ParticleSystemConfig(
                duration: 0.6f,
                startLifetime: 0.5f,
                startSpeed: 10.0f,
                startSize: 0.225f,
                maxParticles: 200,
                emissionRate: 150f,
                gravityMod: 0.1f),
            materialConfig: new MaterialConfig
            {
                BaseColor = CISRed,
                EmissionIntensity = 2.5f
            },
            lodConfig: new LODConfig(0.6f, 0.3f));

        // Memoized init-once list of all 11 prefab descriptors (avoids per-call allocation
        // in VFX hot-path; see Pattern #121 / #123 — public mutability removed by exposing
        // IReadOnlyList<T> instead of the mutable backing array).
        private static readonly VFXPrefabDescriptor[] _allPrefabs = new[]
        {
            BlasterBoltRep,
            BlasterBoltCIS,
            LightsaberVFXRep,
            LightsaberVFXCIS,
            BlasterImpactRep,
            BlasterImpactCIS,
            UnitDeathVFXRep,
            UnitDeathVFXCIS,
            BuildingCollapseRep,
            BuildingCollapseCIS,
            ExplosionCIS
        };

        /// <summary>
        /// All 11 prefab descriptors as an init-once read-only list. Preferred accessor
        /// in hot-paths (VFX spawn, pool warmup) — no allocation per call.
        /// </summary>
        public static IReadOnlyList<VFXPrefabDescriptor> AllPrefabs { get; } = _allPrefabs;

        /// <summary>
        /// Get all 11 prefab descriptors. Returns the same cached <see cref="IReadOnlyList{T}"/>
        /// as <see cref="AllPrefabs"/>; no allocation per call. Callers must not cast to a
        /// mutable list.
        /// </summary>
        public static IReadOnlyList<VFXPrefabDescriptor> GetAllPrefabs() => _allPrefabs;
    }
}
