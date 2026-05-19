#nullable enable
using System;
using UnityEngine;

namespace DINOForge.Runtime.VFX
{
    /// <summary>
    /// Runtime factory for constructing VFX prefabs from descriptors.
    /// Used when binary .prefab files are not available (e.g., development builds).
    ///
    /// Creates GameObject + ParticleSystem + Material + Renderer at runtime,
    /// matching the configuration defined in VFXPrefabDescriptor.
    ///
    /// This is a fallback mechanism; production builds should use pre-built binary prefabs.
    /// However, this ensures VFX can always be instantiated even if assets are missing.
    /// </summary>
    public static class VFXPrefabFactory
    {
        /// <summary>
        /// Create a VFX prefab GameObject from a descriptor.
        /// The returned GameObject is not parented and is disabled by default.
        /// </summary>
        public static GameObject CreatePrefabFromDescriptor(VFXPrefabDescriptor descriptor)
        {
            try
            {
                // Create root GameObject
                GameObject go = new GameObject(descriptor.Id);
                go.SetActive(false);

                // Add ParticleSystem component
                ParticleSystem ps = go.AddComponent<ParticleSystem>();
                ApplyParticleSystemConfig(ps, descriptor.ParticleConfig);

                // Get renderer and configure
                ParticleSystemRenderer psr = go.GetComponent<ParticleSystemRenderer>();
                ApplyRendererConfig(psr, descriptor.MaterialConfig);

                // Create and assign material with faction colors
                Material mat = CreateParticleMaterial(descriptor.MaterialConfig, descriptor.ParticleConfig.Color);
                psr.material = mat;

                return go;
            }
            catch (Exception ex)
            {
                Debug.LogError($"[VFXPrefabFactory] Failed to create prefab from descriptor '{descriptor.Id}': {ex}");
                return null!;
            }
        }

        /// <summary>
        /// Apply ParticleSystemConfig settings to a ParticleSystem component.
        /// </summary>
        private static void ApplyParticleSystemConfig(ParticleSystem ps, ParticleSystemConfig config)
        {
            var main = ps.main;
            main.duration = config.Duration;
            main.loop = config.Loop;
            main.startLifetime = config.StartLifetime;
            main.startSpeed = config.StartSpeed;
            main.startSize = config.StartSize;
            main.gravityModifier = config.GravityModifier;
            main.maxParticles = config.MaxParticles;

            // Emission
            var emission = ps.emission;
            emission.enabled = true;
            emission.rateOverTime = config.EmissionRateOverTime;

            // Shape
            var shape = ps.shape;
            shape.enabled = true;
            shape.shapeType = config.Shape.ShapeType;
            shape.angle = config.Shape.Angle;
            shape.radius = config.Shape.Radius;

            // Color over lifetime
            var colorByLife = ps.colorOverLifetime;
            colorByLife.enabled = true;

            Gradient grad = new Gradient();
            GradientColorKey[] colorKeys = new GradientColorKey[2];
            colorKeys[0].color = config.Color.PrimaryColor;
            colorKeys[0].time = 0.0f;
            colorKeys[1].color = config.Color.SecondaryColor;
            colorKeys[1].time = 1.0f;

            GradientAlphaKey[] alphaKeys = new GradientAlphaKey[2];
            alphaKeys[0].alpha = 1.0f;
            alphaKeys[0].time = 0.0f;
            alphaKeys[1].alpha = 0.0f;
            alphaKeys[1].time = 1.0f;

            grad.SetKeys(colorKeys, alphaKeys);
            colorByLife.color = new ParticleSystem.MinMaxGradient(grad);
        }

        private static void ApplyRendererConfig(ParticleSystemRenderer psr, MaterialConfig config)
        {
            psr.renderMode = ParticleSystemRenderMode.Billboard;
            psr.maxParticleSize = 10f;
            psr.cameraVelocityScale = 0.0f;
            psr.velocityScale = 1.0f;
            psr.lengthScale = 1.0f;
            psr.alignment = ParticleSystemRenderSpace.View;
        }

        private static Material CreateParticleMaterial(MaterialConfig config, ParticleColorConfig colorConfig)
        {
            // Find or create particle shader
            Shader shader = Shader.Find(config.ShaderName);
            if (shader == null)
            {
                // Fallback to standard particle shader
                shader = Shader.Find("Particles/Standard Unlit");
            }

            if (shader == null)
            {
                // Last resort: use standard surface shader
                shader = Shader.Find("Standard");
            }

            Material mat = new Material(shader);

            // Set colors
            mat.SetColor("_Color", colorConfig.PrimaryColor);
            mat.SetColor("_Emission", colorConfig.PrimaryColor * config.EmissionIntensity);
            mat.SetColor("_EmissionColor", colorConfig.PrimaryColor * config.EmissionIntensity);

            // Set render queue for transparency/additive blending
            mat.renderQueue = config.RenderQueue;

            // Enable emission if possible
            if (mat.HasProperty("_EMISSION"))
            {
                mat.EnableKeyword("_EMISSION");
            }

            return mat;
        }

        /// <summary>
        /// Create all 11 VFX prefabs from the catalog and parent them under a root.
        /// Returns the root GameObject (disabled, for pooling).
        /// </summary>
        public static GameObject CreateAllPrefabsInPool()
        {
            GameObject poolRoot = new GameObject("DINOForge_VFXPrefabPool_Runtime");
            poolRoot.SetActive(false);

            VFXPrefabDescriptor[] allDescriptors = VFXPrefabCatalog.GetAllPrefabs();

            foreach (var descriptor in allDescriptors)
            {
                try
                {
                    GameObject prefab = CreatePrefabFromDescriptor(descriptor);
                    if (prefab != null)
                    {
                        prefab.transform.parent = poolRoot.transform;
                        Debug.Log($"[VFXPrefabFactory] Created runtime prefab: {descriptor.Id}");
                    }
                }
                catch (Exception ex)
                {
                    Debug.LogError($"[VFXPrefabFactory] Failed to create prefab '{descriptor.Id}': {ex}");
                }
            }

            return poolRoot;
        }
    }
}
