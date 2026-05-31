#nullable enable
using System;
using System.Collections.Generic;
using BepInEx.Logging;
using UnityEngine;

namespace DINOForge.Runtime.Graphics
{
    /// <summary>
    /// Applies a skybox + GI refresh for gameplay scenes based on the active planet.
    /// The loader first prefers already-cached materials; if none exist, it creates
    /// procedural solid-color placeholders so gameplay still gets SW-themed atmosphere
    /// even before dedicated skybox asset bundles land.
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
                Material? skybox = GetOrCreateSkyboxMaterial(planet);
                if (skybox == null)
                {
                    _log.LogWarning($"[EnvironmentThemeSwap] No usable URP-compatible skybox material for '{planet}' scene '{sceneName}'.");
                    return false;
                }

                if (!ReferenceEquals(RenderSettings.skybox, skybox))
                {
                    RenderSettings.skybox = skybox;
                    DynamicGI.UpdateEnvironment();
                    _log.LogInfo($"[EnvironmentThemeSwap] Applied skybox '{skybox.name}' for scene '{sceneName}' (planet '{planet}').");
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

            return !sceneName.Equals("MainMenu", StringComparison.OrdinalIgnoreCase) &&
                   !sceneName.Equals("InitialGameLoader", StringComparison.OrdinalIgnoreCase);
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

        private Material? GetOrCreateSkyboxMaterial(string planet)
        {
            if (_cache.TryGetValue(planet, out Material? cached))
                return cached;

            if (!PlanetSkyColors.TryGetValue(planet, out Color tint))
                tint = Color.black;

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


