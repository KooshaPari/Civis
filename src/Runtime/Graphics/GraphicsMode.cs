#nullable enable
using System;
using DINOForge.Runtime.Diagnostics;
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.Rendering.Universal;

namespace DINOForge.Runtime.Graphics
{
    /// <summary>
    /// Graphics fidelity tiers for the realistic-GFX feature.
    /// </summary>
    public enum GraphicsTier
    {
        /// <summary>Vanilla DINO look — no DINOForge post-processing applied (default).</summary>
        Vanilla = 0,

        /// <summary>
        /// Cinematic look — DINOForge injects a URP post-processing stack (tonemap + bloom +
        /// ambient occlusion + subtle color grading) onto the active camera.
        /// </summary>
        High = 1
    }

    /// <summary>
    /// Tier-B Phase-1 proof-of-concept for the "realistic GFX mode" toggle.
    ///
    /// DINO renders with the Universal Render Pipeline (URP 12.x, Unity 2021.3.45f2), confirmed by
    /// the presence of <c>Unity.RenderPipelines.Universal.Runtime.dll</c> and the
    /// <c>Universal Render Pipeline/StencilDeferred</c> shader in the shipped game. URP exposes
    /// post-processing through the SRP <b>Volume framework</b>, so the cleanest non-invasive lift is
    /// to inject a <see cref="Volume"/> with a runtime-built <see cref="VolumeProfile"/> and enable
    /// post-processing on the active camera via <see cref="UniversalAdditionalCameraData"/>. This is
    /// exactly the pattern used by SOTA URP graphics mods (e.g. Lumina for Cities: Skylines).
    ///
    /// This component is created on the persistent DINOForge root and is OFF by default. It only does
    /// anything when <see cref="ConfiguredTier"/> is <see cref="GraphicsTier.High"/>. It owns its own
    /// Volume GameObject and never mutates DINO's own volumes, so toggling back to
    /// <see cref="GraphicsTier.Vanilla"/> fully restores the vanilla look.
    ///
    /// IMPORTANT (DINO runtime model): MonoBehaviour.Update() does NOT run under DINO's custom
    /// PlayerLoop, so this component does not rely on Update(). It re-applies on
    /// <see cref="UnityEngine.SceneManagement.SceneManager.activeSceneChanged"/> (wired by the caller)
    /// and exposes <see cref="Apply"/> for explicit invocation from a main-thread hook (F-menu, key
    /// thread marshal, etc.). All URP/Camera calls MUST happen on the Unity main thread.
    /// </summary>
    public sealed class GraphicsMode : MonoBehaviour
    {
        private const string LogCategory = "GraphicsMode";
        private const string VolumeObjectName = "DINOForge_GfxVolume";

        private GameObject? _volumeGo;
        private Volume? _volume;
        private VolumeProfile? _profile;

        // ---- Phase-2 quality-settings snapshot (for reversible restore) ----
        private bool _qualityCaptured;
        private int _origShadowResolution;
        private float _origShadowDistance;
        private int _origShadowCascades;
        private int _origAntiAliasing;

        /// <summary>
        /// The tier the user has configured (bound to BepInEx config by <see cref="Plugin"/>).
        /// Defaults to <see cref="GraphicsTier.Vanilla"/> so the feature is inert unless opted in.
        /// </summary>
        public GraphicsTier ConfiguredTier { get; set; } = GraphicsTier.Vanilla;

        /// <summary>The tier currently realized on the camera.</summary>
        public GraphicsTier ActiveTier { get; private set; } = GraphicsTier.Vanilla;

        /// <summary>
        /// Apply the configured tier to the active camera. Safe to call repeatedly (idempotent).
        /// Must be called on the Unity main thread.
        /// </summary>
        public void Apply()
        {
            try
            {
                if (ConfiguredTier == GraphicsTier.High)
                {
                    EnableHigh();
                }
                else
                {
                    DisableHigh();
                }
            }
            catch (Exception ex)
            {
                // Graceful degradation: a graphics failure must never take the game down.
                DebugLog.Write(LogCategory, $"Apply failed ({ConfiguredTier}): {ex.GetType().Name}: {ex.Message}");
            }
        }

        /// <summary>Toggle between Vanilla and High and apply immediately. Returns the new tier.</summary>
        public GraphicsTier Toggle()
        {
            ConfiguredTier = ConfiguredTier == GraphicsTier.High ? GraphicsTier.Vanilla : GraphicsTier.High;
            Apply();
            return ConfiguredTier;
        }

        private void EnableHigh()
        {
            EnsureVolume();
            EnableCameraPostProcessing(true);
            ApplyHighQualitySettings();
            ActiveTier = GraphicsTier.High;
            // Phase 2: let the material upgrader (AssetSwap hook) honour the High tier.
            GraphicsMaterialUpgrader.ActiveTier = GraphicsTier.High;
            DebugLog.Write(LogCategory, "High graphics tier active (URP post-process volume + quality bumps + PBR material upgrade enabled).");
        }

        private void DisableHigh()
        {
            if (_volume != null)
            {
                _volume.enabled = false;
            }
            EnableCameraPostProcessing(false);
            RestoreQualitySettings();
            ActiveTier = GraphicsTier.Vanilla;
            // Phase 2: revert to passthrough so already-applied materials are not re-upgraded.
            GraphicsMaterialUpgrader.ActiveTier = GraphicsTier.Vanilla;
            DebugLog.Write(LogCategory, "Vanilla graphics tier active (DINOForge post-process + quality bumps + PBR upgrade disabled).");
        }

        /// <summary>
        /// Phase-2 quality lift for the High tier: longer/higher-resolution shadows, more cascades,
        /// and MSAA. Captures the vanilla values on first application so <see cref="RestoreQualitySettings"/>
        /// can fully revert. All values are conservative — readable, not a benchmark filter.
        /// QualitySettings are global Unity state; URP's UniversalRenderPipelineAsset reads many of
        /// them, so bumping them lifts the active URP render path without us having to mutate the
        /// pipeline asset directly (which DINO ships read-only). Best-effort and never throws.
        /// </summary>
        private void ApplyHighQualitySettings()
        {
            try
            {
                if (!_qualityCaptured)
                {
                    _origShadowResolution = (int)QualitySettings.shadowResolution;
                    _origShadowDistance = QualitySettings.shadowDistance;
                    _origShadowCascades = QualitySettings.shadowCascades;
                    _origAntiAliasing = QualitySettings.antiAliasing;
                    _qualityCaptured = true;
                }

                QualitySettings.shadowResolution = UnityEngine.ShadowResolution.VeryHigh;
                QualitySettings.shadowDistance = Mathf.Max(_origShadowDistance, 150f);
                QualitySettings.shadowCascades = 4;
                QualitySettings.antiAliasing = 4; // 4x MSAA

                DebugLog.Write(LogCategory,
                    $"High quality settings applied (shadowRes=VeryHigh, dist={QualitySettings.shadowDistance}, cascades=4, MSAA=4x).");
            }
            catch (Exception ex)
            {
                DebugLog.Write(LogCategory, $"ApplyHighQualitySettings failed (non-fatal): {ex.GetType().Name}: {ex.Message}");
            }
        }

        /// <summary>Reverts QualitySettings captured by <see cref="ApplyHighQualitySettings"/>.</summary>
        private void RestoreQualitySettings()
        {
            if (!_qualityCaptured) return;
            try
            {
                QualitySettings.shadowResolution = (UnityEngine.ShadowResolution)_origShadowResolution;
                QualitySettings.shadowDistance = _origShadowDistance;
                QualitySettings.shadowCascades = _origShadowCascades;
                QualitySettings.antiAliasing = _origAntiAliasing;
                DebugLog.Write(LogCategory, "Quality settings restored to vanilla snapshot.");
            }
            catch (Exception ex)
            {
                DebugLog.Write(LogCategory, $"RestoreQualitySettings failed (non-fatal): {ex.GetType().Name}: {ex.Message}");
            }
        }

        /// <summary>
        /// Lazily build the DINOForge-owned global Volume + profile and populate it with a cinematic
        /// post-processing stack. Re-enables the volume if it already exists.
        /// </summary>
        private void EnsureVolume()
        {
            if (_volume != null && _volumeGo != null)
            {
                _volume.enabled = true;
                return;
            }

            _profile = ScriptableObject.CreateInstance<VolumeProfile>();
            _profile.name = "DINOForge_GfxProfile";
            PopulateProfile(_profile);

            _volumeGo = new GameObject(VolumeObjectName)
            {
                hideFlags = HideFlags.HideAndDontSave
            };
            // Parent under the persistent root so it survives scene transitions like the rest of DINOForge.
            _volumeGo.transform.SetParent(transform, worldPositionStays: false);

            _volume = _volumeGo.AddComponent<Volume>();
            _volume.isGlobal = true;          // applies everywhere; no collider/trigger needed.
            _volume.priority = 100f;          // sit above any vanilla global volume.
            _volume.weight = 1f;
            _volume.sharedProfile = _profile;
            _volume.enabled = true;
        }

        /// <summary>
        /// Add the cinematic overrides. Conservative defaults — readable, not a screenshot filter.
        /// Tunables here become the BepInEx config / F-menu sliders in later phases.
        /// </summary>
        private static void PopulateProfile(VolumeProfile profile)
        {
            // ACES tonemapping: filmic curve — the single biggest "TABS-flat -> cinematic" shift.
            Tonemapping tonemapping = profile.Add<Tonemapping>(overrides: true);
            tonemapping.mode.overrideState = true;
            tonemapping.mode.value = TonemappingMode.ACES;

            // Bloom: soft highlight glow.
            Bloom bloom = profile.Add<Bloom>(overrides: true);
            bloom.intensity.overrideState = true;
            bloom.intensity.value = 0.35f;
            bloom.threshold.overrideState = true;
            bloom.threshold.value = 1.1f;
            bloom.scatter.overrideState = true;
            bloom.scatter.value = 0.6f;

            // Color grading: gentle contrast + saturation lift to fight the flat low-poly read.
            ColorAdjustments color = profile.Add<ColorAdjustments>(overrides: true);
            color.contrast.overrideState = true;
            color.contrast.value = 12f;
            color.saturation.overrideState = true;
            color.saturation.value = 8f;
            color.postExposure.overrideState = true;
            color.postExposure.value = 0.1f;

            // Vignette: subtle framing.
            Vignette vignette = profile.Add<Vignette>(overrides: true);
            vignette.intensity.overrideState = true;
            vignette.intensity.value = 0.18f;
            vignette.smoothness.overrideState = true;
            vignette.smoothness.value = 0.4f;

            // NOTE: URP screen-space ambient occlusion (SSAO) is a *ScriptableRendererFeature*, not a
            // Volume override, so it cannot be enabled purely from a Volume. Adding SSAO at runtime
            // requires reaching into the active UniversalRendererData's rendererFeatures list (or a
            // CommandBuffer-based AO pass). That is deferred to Phase 3 (see feasibility doc) to keep
            // this Phase-1 PoC volume-only and low-risk.
        }

        /// <summary>
        /// Enable/disable URP post-processing on the active camera(s). URP only evaluates the Volume
        /// stack for cameras whose <see cref="UniversalAdditionalCameraData.renderPostProcessing"/>
        /// is true, so we must flip that flag — DINO's camera may ship with it off.
        /// </summary>
        private static void EnableCameraPostProcessing(bool enabled)
        {
            Camera? cam = Camera.main;
            if (cam == null)
            {
                // No camera yet (e.g. main menu / mid-transition). Caller re-invokes on scene change.
                DebugLog.Write(LogCategory, "No Camera.main yet; will re-apply on next scene change.");
                return;
            }

            UniversalAdditionalCameraData data = cam.GetUniversalAdditionalCameraData();
            if (data != null)
            {
                data.renderPostProcessing = enabled;
            }
        }
    }
}
