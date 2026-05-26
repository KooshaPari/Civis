#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using UnityEditor;
using UnityEngine;

namespace DINOForge.Tools.VFXPrefabGenerator
{
    /// <summary>
    /// VFX Prefab Generator — Creates binary Unity prefabs for the warfare-starwars pack.
    ///
    /// This tool generates 11 particle system prefabs used by the VFXPoolManager:
    /// - 2 blaster bolts (Republic, CIS)
    /// - 2 lightsaber VFX (Republic, CIS)
    /// - 2 blaster impacts (Republic, CIS)
    /// - 2 unit death effects (Republic, CIS)
    /// - 2 building collapse effects (Republic, CIS)
    /// - 1 large explosion (CIS)
    ///
    /// Usage: Unity Editor menu > DINOForge > Generate VFX Prefabs
    ///
    /// Output directory: Assets/warfare-starwars/vfx/
    /// (which maps to packs/warfare-starwars/assets/vfx/ in the final pack)
    ///
    /// Color Reference:
    /// - Republic: #4488FF (bright blue), #64A0DC (lighter accent)
    /// - CIS: #FF4400 (rust orange), #B35A00 (darker accent)
    ///
    /// Each prefab includes:
    /// - ParticleSystem component with faction-appropriate configuration
    /// - ParticleSystemRenderer with correct material/shader
    /// - LOD support via max particle count variation
    /// </summary>
    public static class VFXPrefabGenerator
    {
        // Faction colors (hex to Color)
        private static readonly Color RepublicBlue = new Color(0.267f, 0.533f, 1.0f, 1.0f);      // #4488FF
        private static readonly Color RepublicAccent = new Color(0.392f, 0.627f, 0.863f, 1.0f);   // #64A0DC
        private static readonly Color CISRed = new Color(1.0f, 0.267f, 0.0f, 1.0f);                // #FF4400
        private static readonly Color CISAccent = new Color(0.702f, 0.353f, 0.0f, 1.0f);           // #B35A00

        private const string OutputDirectory = "Assets/warfare-starwars/vfx";

        [MenuItem("DINOForge/Generate VFX Prefabs")]
        public static void GenerateAllPrefabs()
        {
            try
            {
                if (!EditorApplication.isPlaying)
                {
                    EditorUtility.DisplayDialog(
                        "VFX Prefab Generator",
                        "Generating 11 VFX prefabs...",
                        "OK");
                }

                // Ensure output directory exists
                if (!AssetDatabase.IsValidFolder(OutputDirectory))
                {
                    string parentDir = Path.GetDirectoryName(OutputDirectory);
                    string folderName = Path.GetFileName(OutputDirectory);
                    AssetDatabase.CreateFolder(parentDir, folderName);
                }

                // Generate all 11 prefabs
                GenerateBlasterBoltRep();
                GenerateBlasterBoltCIS();
                GenerateLightsaberVFXRep();
                GenerateLightsaberVFXCIS();
                GenerateBlasterImpactRep();
                GenerateBlasterImpactCIS();
                GenerateUnitDeathVFXRep();
                GenerateUnitDeathVFXCIS();
                GenerateBuildingCollapseRep();
                GenerateBuildingCollapseCIS();
                GenerateExplosionCIS();

                // Save all changes
                AssetDatabase.SaveAssets();
                AssetDatabase.Refresh();

                Debug.Log("[VFXPrefabGenerator] Successfully generated 11 VFX prefabs");
                EditorUtility.DisplayDialog(
                    "VFX Prefab Generator",
                    "Successfully generated 11 VFX prefabs in " + OutputDirectory,
                    "OK");
            }
            catch (Exception ex)
            {
                Debug.LogError($"[VFXPrefabGenerator] Failed to generate prefabs: {ex}");
                EditorUtility.DisplayDialog(
                    "VFX Prefab Generator Error",
                    $"Failed to generate prefabs:\n{ex.Message}",
                    "OK");
            }
        }

        /// <summary>
        /// Create a ParticleSystem prefab with faction-specific configuration.
        /// </summary>
        private static GameObject CreateParticleSystemPrefab(
            string prefabName,
            Color primaryColor,
            Color secondaryColor,
            VFXType type)
        {
            GameObject go = new GameObject(prefabName);
            ParticleSystem ps = go.AddComponent<ParticleSystem>();
            ParticleSystemRenderer psr = go.GetComponent<ParticleSystemRenderer>();

            // Configure particle system based on effect type
            ConfigureParticleSystem(ps, type, primaryColor);

            // Create and assign material
            Material mat = CreateParticleMaterial(primaryColor, secondaryColor, type);
            psr.material = mat;

            // Set standard renderer properties
            psr.renderMode = ParticleSystemRenderMode.Billboard;
            psr.maxParticleSize = 10f;
            psr.cameraVelocityScale = 0.0f;
            psr.velocityScale = 1.0f;

            return go;
        }

        private enum VFXType
        {
            BlasterBolt,      // Fast projectile with glow
            LightsaberVFX,    // Melee swing trail
            BlasterImpact,    // Hit burst
            UnitDeathVFX,     // Unit destruction
            BuildingCollapse, // Building destruction
            Explosion         // Large explosion
        }

        private static void ConfigureParticleSystem(ParticleSystem ps, VFXType type, Color color)
        {
            var main = ps.main;
            var emission = ps.emission;
            var shape = ps.shape;
            var velocity = ps.velocityOverLifetime;
            var size = ps.sizeOverLifetime;
            var colorByLife = ps.colorOverLifetime;
            var trails = ps.trails;

            switch (type)
            {
                case VFXType.BlasterBolt:
                    // Fast projectile with trailing glow
                    main.duration = 0.5f;
                    main.loop = false;
                    main.startLifetime = 0.3f;
                    main.startSpeed = 20.0f;
                    main.startSize = 0.1f;
                    main.gravityModifier = 0f;

                    emission.enabled = true;
                    emission.rateOverTime = new ParticleSystem.MinMaxCurve(50);

                    shape.enabled = true;
                    shape.shapeType = ParticleSystemShapeType.Cone;
                    shape.angle = 5f;

                    main.maxParticles = 80;
                    break;

                case VFXType.LightsaberVFX:
                    // Melee swing with trailing energy
                    main.duration = 0.6f;
                    main.loop = false;
                    main.startLifetime = 0.4f;
                    main.startSpeed = 5.0f;
                    main.startSize = new ParticleSystem.MinMaxCurve(0.05f, 0.15f);
                    main.gravityModifier = 0f;

                    emission.enabled = true;
                    emission.rateOverTime = new ParticleSystem.MinMaxCurve(40);

                    shape.enabled = true;
                    shape.shapeType = ParticleSystemShapeType.Sphere;

                    main.maxParticles = 100;
                    break;

                case VFXType.BlasterImpact:
                    // Burst on impact — quick, bright
                    main.duration = 0.3f;
                    main.loop = false;
                    main.startLifetime = 0.25f;
                    main.startSpeed = new ParticleSystem.MinMaxCurve(2.0f, 4.0f);
                    main.startSize = new ParticleSystem.MinMaxCurve(0.05f, 0.1f);
                    main.gravityModifier = 0.1f;

                    emission.enabled = true;
                    emission.rateOverTime = new ParticleSystem.MinMaxCurve(100);

                    shape.enabled = true;
                    shape.shapeType = ParticleSystemShapeType.Cone;
                    shape.angle = 45f;

                    main.maxParticles = 50;
                    break;

                case VFXType.UnitDeathVFX:
                    // Ascending disintegration or explosive burst
                    main.duration = 0.8f;
                    main.loop = false;
                    main.startLifetime = 0.6f;
                    main.startSpeed = new ParticleSystem.MinMaxCurve(1.0f, 3.0f);
                    main.startSize = new ParticleSystem.MinMaxCurve(0.1f, 0.25f);
                    main.gravityModifier = -0.05f; // Slight upward bias

                    emission.enabled = true;
                    emission.rateOverTime = new ParticleSystem.MinMaxCurve(80);

                    shape.enabled = true;
                    shape.shapeType = ParticleSystemShapeType.Sphere;
                    shape.radius = 0.3f;

                    main.maxParticles = 120;
                    break;

                case VFXType.BuildingCollapse:
                    // Dust cloud rising
                    main.duration = 1.0f;
                    main.loop = false;
                    main.startLifetime = 0.8f;
                    main.startSpeed = new ParticleSystem.MinMaxCurve(0.5f, 1.5f);
                    main.startSize = new ParticleSystem.MinMaxCurve(0.2f, 0.5f);
                    main.gravityModifier = -0.3f; // Upward movement

                    emission.enabled = true;
                    emission.rateOverTime = new ParticleSystem.MinMaxCurve(60);

                    shape.enabled = true;
                    shape.shapeType = ParticleSystemShapeType.Sphere;
                    shape.radius = 1.0f;

                    main.maxParticles = 150;
                    break;

                case VFXType.Explosion:
                    // Large explosion burst
                    main.duration = 0.6f;
                    main.loop = false;
                    main.startLifetime = 0.5f;
                    main.startSpeed = new ParticleSystem.MinMaxCurve(5.0f, 15.0f);
                    main.startSize = new ParticleSystem.MinMaxCurve(0.15f, 0.3f);
                    main.gravityModifier = 0.1f;

                    emission.enabled = true;
                    emission.rateOverTime = new ParticleSystem.MinMaxCurve(150);

                    shape.enabled = true;
                    shape.shapeType = ParticleSystemShapeType.Sphere;
                    shape.radius = 0.5f;

                    main.maxParticles = 200;
                    break;
            }

            // Color over lifetime: fade from color to transparent
            if (colorByLife.enabled)
            {
                Gradient grad = new Gradient();
                GradientColorKey[] colorKeys = new GradientColorKey[2];
                colorKeys[0].color = color;
                colorKeys[0].time = 0.0f;
                colorKeys[1].color = new Color(color.r, color.g, color.b, 0.0f);
                colorKeys[1].time = 1.0f;

                GradientAlphaKey[] alphaKeys = new GradientAlphaKey[2];
                alphaKeys[0].alpha = 1.0f;
                alphaKeys[0].time = 0.0f;
                alphaKeys[1].alpha = 0.0f;
                alphaKeys[1].time = 1.0f;

                grad.SetKeys(colorKeys, alphaKeys);
                colorByLife.color = new ParticleSystem.MinMaxGradient(grad);
            }
        }

        private static Material CreateParticleMaterial(Color primaryColor, Color secondaryColor, VFXType type)
        {
            // Create a simple unlit additive material
            Material mat = new Material(Shader.Find("Particles/Standard Unlit"));

            mat.SetColor("_Color", primaryColor);
            mat.SetColor("_EmissionColor", primaryColor * 2.0f); // Doubled for glow effect
            mat.renderQueue = 3000; // Transparent queue for additive blending

            return mat;
        }

        // ===== Prefab Generation Methods =====

        private static void GenerateBlasterBoltRep()
        {
            GameObject go = CreateParticleSystemPrefab("BlasterBolt_Rep", RepublicBlue, RepublicAccent, VFXType.BlasterBolt);
            SavePrefab(go, "BlasterBolt_Rep.prefab");
        }

        private static void GenerateBlasterBoltCIS()
        {
            GameObject go = CreateParticleSystemPrefab("BlasterBolt_CIS", CISRed, CISAccent, VFXType.BlasterBolt);
            SavePrefab(go, "BlasterBolt_CIS.prefab");
        }

        private static void GenerateLightsaberVFXRep()
        {
            GameObject go = CreateParticleSystemPrefab("LightsaberVFX_Rep", RepublicBlue, RepublicAccent, VFXType.LightsaberVFX);
            SavePrefab(go, "LightsaberVFX_Rep.prefab");
        }

        private static void GenerateLightsaberVFXCIS()
        {
            GameObject go = CreateParticleSystemPrefab("LightsaberVFX_CIS", CISRed, CISAccent, VFXType.LightsaberVFX);
            SavePrefab(go, "LightsaberVFX_CIS.prefab");
        }

        private static void GenerateBlasterImpactRep()
        {
            GameObject go = CreateParticleSystemPrefab("BlasterImpact_Rep", RepublicBlue, RepublicAccent, VFXType.BlasterImpact);
            SavePrefab(go, "BlasterImpact_Rep.prefab");
        }

        private static void GenerateBlasterImpactCIS()
        {
            GameObject go = CreateParticleSystemPrefab("BlasterImpact_CIS", CISRed, CISAccent, VFXType.BlasterImpact);
            SavePrefab(go, "BlasterImpact_CIS.prefab");
        }

        private static void GenerateUnitDeathVFXRep()
        {
            GameObject go = CreateParticleSystemPrefab("UnitDeathVFX_Rep", RepublicBlue, RepublicAccent, VFXType.UnitDeathVFX);
            SavePrefab(go, "UnitDeathVFX_Rep.prefab");
        }

        private static void GenerateUnitDeathVFXCIS()
        {
            GameObject go = CreateParticleSystemPrefab("UnitDeathVFX_CIS", CISRed, CISAccent, VFXType.UnitDeathVFX);
            SavePrefab(go, "UnitDeathVFX_CIS.prefab");
        }

        private static void GenerateBuildingCollapseRep()
        {
            GameObject go = CreateParticleSystemPrefab("BuildingCollapse_Rep", RepublicBlue, RepublicAccent, VFXType.BuildingCollapse);
            SavePrefab(go, "BuildingCollapse_Rep.prefab");
        }

        private static void GenerateBuildingCollapseCIS()
        {
            GameObject go = CreateParticleSystemPrefab("BuildingCollapse_CIS", CISRed, CISAccent, VFXType.BuildingCollapse);
            SavePrefab(go, "BuildingCollapse_CIS.prefab");
        }

        private static void GenerateExplosionCIS()
        {
            GameObject go = CreateParticleSystemPrefab("Explosion_CIS", CISRed, CISAccent, VFXType.Explosion);
            SavePrefab(go, "Explosion_CIS.prefab");
        }

        private static void SavePrefab(GameObject go, string fileName)
        {
            string path = $"{OutputDirectory}/{fileName}";

            // Save as prefab
            PrefabUtility.SaveAsPrefabAsset(go, path);

            // Destroy the temporary GameObject
            GameObject.DestroyImmediate(go);

            Debug.Log($"[VFXPrefabGenerator] Created prefab: {path}");
        }
    }
}
