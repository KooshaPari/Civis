#nullable enable
using System;
using System.Collections.Generic;
using BepInEx.Logging;
using UnityEngine;

namespace DINOForge.Runtime.Graphics
{
    /// <summary>
    /// Applies a skybox + GI refresh for gameplay scenes based on the active planet.
    /// The loader first prefers bundled skybox materials (for future SW bundle support);
    /// if no SW bundle exists, it creates procedural solid-color placeholders so gameplay
    /// still gets a readable SW atmosphere until real bundles land.
    /// TODO: Replace fallback placeholders with loaded SW skybox bundles (tatooine/naboo/umbara/coruscant).
    /// </summary>
    internal sealed class EnvironmentThemeSwap
    {
        private const string LogCategory = "EnvironmentThemeSwap";
        private static readonly string[] PlanetKeywords =
        {
            "tatooine",
            "naboo",
            "umbara",
            "coruscant"
        };

        private static readonly Dictionary<string, string> PlanetLabelByKeyword = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
        {
            ["tatooine"] = "tatooine",
            ["naboo"] = "naboo",
            ["umbara"] = "umbara",
            ["coruscant"] = "coruscant"
        };

        private static readonly Dictionary<string, string[]> PlanetSkyboxBundleKeys = new Dictionary<string, string[]>(StringComparer.OrdinalIgnoreCase)
        {
            ["tatooine"] = new[] { "skybox_tatooine", "tatooine_skybox", "sw_tatooine_skybox", "Environment/skybox_tatooine" },
            ["naboo"] = new[] { "skybox_naboo", "naboo_skybox", "sw_naboo_skybox", "Environment/skybox_naboo" },
            ["umbara"] = new[] { "skybox_umbara", "umbara_skybox", "sw_umbara_skybox", "Environment/skybox_umbara" },
            ["coruscant"] = new[] { "skybox_coruscant", "coruscant_skybox", "sw_coruscant_skybox", "Environment/skybox_coruscant" }
        };

        private static readonly Dictionary<string, Color> PlanetSkyColors = new Dictionary<string, Color>(StringComparer.OrdinalIgnoreCase)
        {
            ["tatooine"] = new Color(0.89f, 0.74f, 0.49f, 1f),
            ["naboo"] = new Color(0.22f, 0.46f, 0.78f, 1f),
            ["umbara"] = new Color(0.12f, 0.12f, 0.14f, 1f),
            ["coruscant"] = new Color(0.98f, 0.56f, 0.24f, 1f)
        };

        private static readonly string[] CandidateSkyboxShaders =
        {
            "Skybox/Procedural",
            "Skybox/6 Sided",
            "Universal Render Pipeline/Lit",
            "Universal Render Pipeline/Unlit"
        };

        private readonly ManualLogSource _log;
        private readonly Dictionary<string, Material> _cache = new(StringComparer.OrdinalIgnoreCase);

        public EnvironmentThemeSwap(ManualLogSource log)
        {
            _log = log;
        }

        public bool TryApplyForScene(string sceneName)
        {
            if (string.IsNullOrWhiteSpace(sceneName))
                return false;

            string? planet = ResolvePlanet(sceneName);
            if (planet == null)
            {
                _log.LogDebug($"[EnvironmentThemeSwap] No planet mapping for scene '{sceneName}'.");
                return false;
            }

            try
            {
                bool usedPlaceholder;
                Material? skybox = GetOrCreateSkyboxMaterial(planet, out usedPlaceholder);
                if (skybox == null)
                {
                    _log.LogWarning($"[EnvironmentThemeSwap] No usable URP-compatible skybox material for '{planet}' scene '{sceneName}'.");
                    return false;
                }

                if (!ReferenceEquals(RenderSettings.skybox, skybox))
                {
                    RenderSettings.skybox = skybox;
                    DynamicGI.UpdateEnvironment();
                    string source = usedPlaceholder ? "placeholder" : "bundled";
                    _log.LogInfo($"[EnvironmentThemeSwap] Applied {source} skybox '{skybox.name}' for scene '{sceneName}' (planet '{planet}').");
                }
                else
                {
                    _log.LogDebug($"[EnvironmentThemeSwap] Scene '{sceneName}' already using skybox '{skybox.name}'.");
                }

                return true;
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[EnvironmentThemeSwap] Apply failed for scene '{sceneName}': {ex.Message}");
                return false;
            }
        }

        public static bool IsGameplayScene(string sceneName)
        {
            if (string.IsNullOrWhiteSpace(sceneName))
                return false;

            string lowered = sceneName.ToLowerInvariant();
            if (lowered.Equals("mainmenu", StringComparison.OrdinalIgnoreCase) ||
                lowered.Equals("initialgameload", StringComparison.OrdinalIgnoreCase) ||
                lowered.Contains("menu"))
            {
                return false;
            }

            return true;
        }

        private static string? ResolvePlanet(string sceneName)
        {
            string lowered = sceneName.ToLowerInvariant();
            foreach (string keyword in PlanetKeywords)
            {
                if (lowered.Contains(keyword))
                    return PlanetLabelByKeyword[keyword];
            }

            return null;
        }

        private Material? GetOrCreateSkyboxMaterial(string planet, out bool usedPlaceholder)
        {
            usedPlaceholder = false;
            if (_cache.TryGetValue(planet, out Material? cached))
            {
                return cached;
            }

            if (!PlanetSkyColors.TryGetValue(planet, out Color tint))
                tint = Color.black;

            if (TryLoadBundledSkyboxMaterial(planet, out Material? bundledMaterial))
            {
                _cache[planet] = bundledMaterial;
                return bundledMaterial;
            }

            usedPlaceholder = true;
            Shader? skyboxShader = null;
            for (int i = 0; i < CandidateSkyboxShaders.Length; i++)
            {
                skyboxShader = Shader.Find(CandidateSkyboxShaders[i]);
                if (skyboxShader != null)
                    break;
            }

            if (skyboxShader == null)
            {
                _log.LogWarning($"[EnvironmentThemeSwap] No skybox shader found in candidate set: {string.Join(", ", CandidateSkyboxShaders)}");
                return null;
            }

            Material material = new(skyboxShader)
            {
                name = $"DINOForge_{planet}_skybox"
            };

            ApplyColorTint(material, tint);
            _cache[planet] = material;
            return material;
        }

        private bool TryLoadBundledSkyboxMaterial(string planet, out Material? material)
        {
            material = null;

            if (!PlanetSkyboxBundleKeys.TryGetValue(planet, out string[]? bundleKeys))
            {
                _log.LogDebug($"[EnvironmentThemeSwap] No skybox bundle key map for planet '{planet}'.");
                return false;
            }

            for (int i = 0; i < bundleKeys.Length; i++)
            {
                string candidatePath = bundleKeys[i];
                material = Resources.Load<Material>(candidatePath);
                if (material != null)
                {
                    _log.LogInfo($"[EnvironmentThemeSwap] Loaded bundled skybox material '{material.name}' using Resources key '{candidatePath}' for planet '{planet}'.");
                    return true;
                }
            }

            _log.LogDebug($"[EnvironmentThemeSwap] No SW skybox bundle material found for planet '{planet}' from keys: {string.Join(", ", bundleKeys)}. Using procedural placeholder.");
            return false;
        }

        private static void ApplyColorTint(Material material, Color tint)
        {
            if (material.HasProperty("_Tint"))
                material.SetColor("_Tint", tint);
            if (material.HasProperty("_SkyTint"))
                material.SetColor("_SkyTint", tint);
            if (material.HasProperty("_BaseColor"))
                material.SetColor("_BaseColor", tint);
            if (material.HasProperty("_Color"))
                material.SetColor("_Color", tint);
            if (material.HasProperty("_UnlitColor"))
                material.SetColor("_UnlitColor", tint);
        }
    }
}
