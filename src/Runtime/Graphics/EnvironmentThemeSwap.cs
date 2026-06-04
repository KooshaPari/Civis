#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using BepInEx.Logging;
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.SceneManagement;

namespace DINOForge.Runtime.Graphics
{
    /// <summary>
    /// Phase-2 spike (#975): applies pack-defined environment themes (skybox, ambient, fog)
    /// via Unity <see cref="RenderSettings"/> when a gameplay map scene becomes active.
    /// RenderSettings are scene-level and outside the ECS bridge — this is the dedicated hook.
    /// </summary>
    internal static class EnvironmentThemeSwap
    {
        private static ManualLogSource? _log;
        private static string _packsDirectory = string.Empty;
        private static bool _subscribed;
        private static string? _lastAppliedScene;
        private static Material? _loadedSkybox;

        /// <summary>Registers the scene-change hook. Call from plugin bootstrap when wired.</summary>
        public static void Initialize(ManualLogSource log, string packsDirectory)
        {
            _log = log;
            _packsDirectory = packsDirectory ?? string.Empty;
            EnsureSubscribed();
        }

        [RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.AfterSceneLoad)]
        private static void AutoRegisterSceneHook()
        {
            EnsureSubscribed();
        }

        private static void EnsureSubscribed()
        {
            if (_subscribed) return;
            SceneManager.activeSceneChanged += OnActiveSceneChanged;
            _subscribed = true;
        }

        private static void OnActiveSceneChanged(Scene previous, Scene next)
        {
            if (!next.IsValid() || !next.isLoaded) return;
            if (string.Equals(_lastAppliedScene, next.name, StringComparison.Ordinal)) return;
            if (!IsGameplayScene(next.name)) return;

            EnvironmentThemeData? theme = ResolveActiveEnvironmentTheme();
            if (theme == null) return;

            if (TryApply(theme))
            {
                _lastAppliedScene = next.name;
                _log?.LogInfo($"[EnvironmentThemeSwap] Applied planet '{theme.PlanetId}' for scene '{next.name}'.");
            }
        }

        internal static bool TryApply(EnvironmentThemeData theme)
        {
            if (theme == null) return false;

            try
            {
                ApplyAmbient(theme);
                ApplyFog(theme);
                ApplySkybox(theme);
                return true;
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[EnvironmentThemeSwap] Apply failed: {ex.Message}");
                return false;
            }
        }

        private static void ApplyAmbient(EnvironmentThemeData theme)
        {
            if (theme.AmbientModeSetting.HasValue)
            {
                RenderSettings.ambientMode = theme.AmbientModeSetting.Value;
            }

            if (theme.AmbientLight != null && ColorUtility.TryParseHtmlString(theme.AmbientLight, out Color ambient))
            {
                RenderSettings.ambientLight = ambient;
            }
        }

        private static void ApplyFog(EnvironmentThemeData theme)
        {
            if (theme.FogEnabled.HasValue)
            {
                RenderSettings.fog = theme.FogEnabled.Value;
            }

            if (theme.FogColor != null && ColorUtility.TryParseHtmlString(theme.FogColor, out Color fogColor))
            {
                RenderSettings.fogColor = fogColor;
            }

            if (theme.FogDensity.HasValue)
            {
                RenderSettings.fogDensity = theme.FogDensity.Value;
            }

            if (theme.FogModeSetting.HasValue)
            {
                RenderSettings.fogMode = theme.FogModeSetting.Value;
            }
        }

        private static void ApplySkybox(EnvironmentThemeData theme)
        {
            if (string.IsNullOrWhiteSpace(theme.SkyboxMaterial)) return;

            Material? material = LoadSkyboxMaterial(theme.SkyboxMaterial, theme.PackId);
            if (material == null) return;

            if (_loadedSkybox != null && _loadedSkybox != material)
            {
                UnityEngine.Object.Destroy(_loadedSkybox);
            }

            _loadedSkybox = material;
            RenderSettings.skybox = material;
        }

        private static Material? LoadSkyboxMaterial(string reference, string? packId)
        {
            if (string.IsNullOrWhiteSpace(reference)) return null;

            if (reference.StartsWith("bundle:", StringComparison.OrdinalIgnoreCase))
            {
                return LoadSkyboxFromBundle(reference.Substring("bundle:".Length), packId);
            }

            return Resources.Load<Material>(reference);
        }

        private static Material? LoadSkyboxFromBundle(string bundleRef, string? packId)
        {
            if (string.IsNullOrWhiteSpace(packId) || string.IsNullOrWhiteSpace(_packsDirectory)) return null;

            string[] parts = bundleRef.Split(new[] { '/' }, 2);
            if (parts.Length != 2) return null;

            string bundleName = parts[0];
            string assetName = parts[1];
            string bundlePath = Path.Combine(_packsDirectory, packId, "assets", "bundles", bundleName);
            if (!File.Exists(bundlePath)) return null;

            AssetBundle? bundle = AssetBundle.LoadFromFile(bundlePath);
            if (bundle == null) return null;

            try
            {
                return bundle.LoadAsset<Material>(assetName);
            }
            finally
            {
                bundle.Unload(unloadAllLoadedObjects: false);
            }
        }

        internal static EnvironmentThemeData? ResolveActiveEnvironmentTheme()
        {
            if (string.IsNullOrWhiteSpace(_packsDirectory) || !Directory.Exists(_packsDirectory)) return null;

            foreach (string packDir in Directory.EnumerateDirectories(_packsDirectory))
            {
                string packYaml = Path.Combine(packDir, "pack.yaml");
                if (!File.Exists(packYaml)) continue;

                string yaml = File.ReadAllText(packYaml, Encoding.UTF8);
                if (yaml.IndexOf("type: total_conversion", StringComparison.OrdinalIgnoreCase) < 0) continue;

                string packId = Path.GetFileName(packDir);
                EnvironmentThemeData? fromSidecar = ReadEnvironmentSidecar(packId);
                if (fromSidecar != null) return fromSidecar;

                EnvironmentThemeData? inline = ReadInlineEnvironmentBlock(yaml, packId);
                if (inline != null) return inline;
            }

            return null;
        }

        private static EnvironmentThemeData? ReadEnvironmentSidecar(string packId)
        {
            string sidecarPath = Path.Combine(_packsDirectory, packId, "ui_theme.environment.yaml");
            if (!File.Exists(sidecarPath)) return null;

            string yaml = File.ReadAllText(sidecarPath, Encoding.UTF8);
            return ParsePlanetEntry(yaml, ResolveScenePlanetId(SceneManager.GetActiveScene().name), packId);
        }

        private static EnvironmentThemeData? ReadInlineEnvironmentBlock(string packYaml, string packId)
        {
            int envIdx = packYaml.IndexOf("environment:", StringComparison.Ordinal);
            if (envIdx < 0) return null;

            return ParsePlanetEntry(packYaml, ResolveScenePlanetId(SceneManager.GetActiveScene().name), packId);
        }

        private static EnvironmentThemeData? ParsePlanetEntry(string yaml, string planetId, string packId)
        {
            int planetsIdx = yaml.IndexOf("planets:", StringComparison.Ordinal);
            if (planetsIdx < 0) return null;

            int planetIdx = yaml.IndexOf(planetId + ":", planetsIdx, StringComparison.OrdinalIgnoreCase);
            if (planetIdx < 0) return null;

            return new EnvironmentThemeData
            {
                PackId = packId,
                PlanetId = planetId,
                SkyboxMaterial = ExtractYamlScalar(yaml, planetIdx, "skybox_material"),
                AmbientLight = ExtractYamlScalar(yaml, planetIdx, "ambient_light"),
                AmbientModeSetting = ParseAmbientMode(ExtractYamlScalar(yaml, planetIdx, "ambient_mode")),
                FogEnabled = ParseNullableBool(ExtractYamlScalar(yaml, planetIdx, "fog_enabled")),
                FogColor = ExtractYamlScalar(yaml, planetIdx, "fog_color"),
                FogDensity = ParseNullableFloat(ExtractYamlScalar(yaml, planetIdx, "fog_density")),
                FogModeSetting = ParseFogMode(ExtractYamlScalar(yaml, planetIdx, "fog_mode")),
            };
        }

        private static string ResolveScenePlanetId(string sceneName)
        {
            if (string.IsNullOrWhiteSpace(sceneName)) return "default";

            string lower = sceneName.ToLowerInvariant();
            if (lower.Contains("tatooine")) return "tatooine";
            if (lower.Contains("naboo")) return "naboo";
            if (lower.Contains("umbara")) return "umbara";
            if (lower.Contains("coruscant")) return "coruscant";
            return "default";
        }

        private static bool IsGameplayScene(string sceneName)
        {
            if (string.IsNullOrWhiteSpace(sceneName)) return false;

            string lower = sceneName.ToLowerInvariant();
            if (lower.Contains("mainmenu") || lower.Contains("main_menu")) return false;
            if (lower.Contains("loading")) return false;
            return true;
        }

        private static string? ExtractYamlScalar(string yaml, int blockStart, string key)
        {
            string searchKey = key + ":";
            int keyIdx = yaml.IndexOf(searchKey, blockStart, StringComparison.Ordinal);
            if (keyIdx < 0) return null;

            int valueStart = keyIdx + searchKey.Length;
            int lineEnd = yaml.IndexOf('\n', valueStart);
            if (lineEnd < 0) lineEnd = yaml.Length;

            string raw = yaml.Substring(valueStart, lineEnd - valueStart).Trim();
            if (raw.Length >= 2 && (raw[0] == '"' || raw[0] == '\''))
            {
                raw = raw.Substring(1, raw.Length - 2);
            }

            return string.IsNullOrWhiteSpace(raw) ? null : raw;
        }

        private static AmbientMode? ParseAmbientMode(string? value)
        {
            if (string.IsNullOrWhiteSpace(value)) return null;
            return Enum.TryParse(value, ignoreCase: true, out AmbientMode mode) ? mode : null;
        }

        private static FogMode? ParseFogMode(string? value)
        {
            if (string.IsNullOrWhiteSpace(value)) return null;
            return Enum.TryParse(value, ignoreCase: true, out FogMode mode) ? mode : null;
        }

        private static bool? ParseNullableBool(string? value)
        {
            if (string.IsNullOrWhiteSpace(value)) return null;
            return bool.TryParse(value, out bool parsed) ? parsed : null;
        }

        private static float? ParseNullableFloat(string? value)
        {
            if (string.IsNullOrWhiteSpace(value)) return null;
            return float.TryParse(value, out float parsed) ? parsed : null;
        }

        internal sealed class EnvironmentThemeData
        {
            public string? PackId;
            public string? PlanetId;
            public string? SkyboxMaterial;
            public string? AmbientLight;
            public AmbientMode? AmbientModeSetting;
            public bool? FogEnabled;
            public string? FogColor;
            public float? FogDensity;
            public FogMode? FogModeSetting;
        }
    }
}
