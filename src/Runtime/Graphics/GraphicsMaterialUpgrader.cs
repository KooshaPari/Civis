#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Runtime.Diagnostics;
using UnityEngine;

namespace DINOForge.Runtime.Graphics
{
    /// <summary>
    /// Describes the PBR texture set a pack may declare for a given <c>visual_asset</c> key.
    /// All slots are optional — the upgrader copies across whatever is present and leaves the
    /// rest at URP/Lit defaults. Textures are <see cref="Texture2D"/> handles that the caller
    /// is responsible for loading (e.g. from a pack AssetBundle); the upgrader never does I/O.
    ///
    /// This is the data contract for the "PBR metadata" a pack declares (today via the
    /// <c>visual_asset</c> + a future <c>pbr:</c> block in unit/building YAML; see
    /// <see cref="PbrMaterialRegistry"/>). It is intentionally a thin DTO so it can be populated
    /// from any source (pack loader, asset bundle, or procedural generation) without coupling
    /// the upgrader to the pack schema.
    /// </summary>
    public sealed class PbrTextureSet
    {
        /// <summary>Base color / albedo map (URP/Lit <c>_BaseMap</c>).</summary>
        public Texture2D? Albedo { get; set; }

        /// <summary>Metallic+smoothness packed map (URP/Lit <c>_MetallicGlossMap</c>).</summary>
        public Texture2D? Metallic { get; set; }

        /// <summary>Tangent-space normal map (URP/Lit <c>_BumpMap</c>).</summary>
        public Texture2D? Normal { get; set; }

        /// <summary>Ambient-occlusion map (URP/Lit <c>_OcclusionMap</c>).</summary>
        public Texture2D? Occlusion { get; set; }

        /// <summary>Emission map (URP/Lit <c>_EmissionMap</c>).</summary>
        public Texture2D? Emission { get; set; }

        /// <summary>Scalar metallic factor used when no metallic map is supplied (0..1).</summary>
        public float Metallic01 { get; set; } = 0f;

        /// <summary>Scalar smoothness/roughness factor (0..1). Higher = smoother (URP convention).</summary>
        public float Smoothness01 { get; set; } = 0.35f;

        /// <summary>Returns true if at least one PBR texture slot is populated.</summary>
        public bool HasAnyTexture =>
            Albedo != null || Metallic != null || Normal != null ||
            Occlusion != null || Emission != null;
    }

    /// <summary>
    /// Thread-safe registry mapping a pack <c>visual_asset</c> key (or material name) to its
    /// declared <see cref="PbrTextureSet"/>. Pack loaders / the asset pipeline register entries
    /// here once the BF2/#973 PBR textures are available; until then the registry is empty and
    /// the upgrader degrades to a pure shader-swap (URP/Lit with the existing base color), or, if
    /// even that is undesirable, a no-op passthrough.
    /// </summary>
    public static class PbrMaterialRegistry
    {
        private static readonly object _lock = new object();
        private static readonly Dictionary<string, PbrTextureSet> _sets =
            new Dictionary<string, PbrTextureSet>(StringComparer.OrdinalIgnoreCase);

        /// <summary>Registers (or replaces) the PBR texture set for a visual-asset / material key.</summary>
        public static void Register(string key, PbrTextureSet set)
        {
            if (string.IsNullOrEmpty(key) || set == null) return;
            lock (_lock) { _sets[key] = set; }
        }

        /// <summary>Attempts to resolve the PBR texture set for a key.</summary>
        public static bool TryGet(string key, out PbrTextureSet? set)
        {
            if (string.IsNullOrEmpty(key)) { set = null; return false; }
            lock (_lock) { return _sets.TryGetValue(key, out set); }
        }

        /// <summary>Number of registered PBR texture sets.</summary>
        public static int Count { get { lock (_lock) { return _sets.Count; } } }

        /// <summary>Clears the registry (test teardown / hot-reload).</summary>
        public static void Clear() { lock (_lock) { _sets.Clear(); } }
    }

    /// <summary>
    /// GFX-mode Phase 2: upgrades materials to the URP/Lit physically-based shader when the
    /// active graphics tier is <see cref="GraphicsTier.High"/>.
    ///
    /// Why this exists: DINO ships low-poly units lit by a flat URP/Lit-or-Simple-Lit material.
    /// The Manor-Lords-style lift comes from (a) the post-process volume (Phase 1) and (b) giving
    /// surfaces real metallic/normal/AO response so light reads as material. This class performs
    /// (b): given a base material (typically the one produced by <c>AssetSwapSystem</c>'s
    /// RenderMesh swap), it produces a NEW URP/Lit material that carries over the base color/map
    /// and layers in any PBR maps declared for the asset.
    ///
    /// Contract / safety:
    ///   - Tier == Vanilla → pure passthrough; the original material is returned unchanged.
    ///   - Tier == High but no <c>Universal Render Pipeline/Lit</c> shader found, or no upgrade
    ///     is possible → the original material is returned (graceful degradation, never throws).
    ///   - With no PBR textures registered for the key, the upgrade still swaps to URP/Lit and
    ///     carries the base color/albedo across so the metallic/smoothness response improves even
    ///     before BF2/#973 textures arrive. If even the shader-only upgrade is unavailable, the
    ///     base material is returned.
    ///
    /// All Unity Material/Shader calls MUST run on the Unity main thread. AssetSwapSystem invokes
    /// the upgrader from <c>OnUpdate</c> (PresentationSystemGroup), which is main-thread.
    /// </summary>
    public static class GraphicsMaterialUpgrader
    {
        private const string LogCategory = "GfxMaterial";
        private const string UrpLitShaderName = "Universal Render Pipeline/Lit";

        // URP/Lit shader property IDs (resolved lazily, once).
        private static readonly int BaseMapId = Shader.PropertyToID("_BaseMap");
        private static readonly int BaseColorId = Shader.PropertyToID("_BaseColor");
        private static readonly int MetallicGlossMapId = Shader.PropertyToID("_MetallicGlossMap");
        private static readonly int MetallicId = Shader.PropertyToID("_Metallic");
        private static readonly int SmoothnessId = Shader.PropertyToID("_Smoothness");
        private static readonly int BumpMapId = Shader.PropertyToID("_BumpMap");
        private static readonly int OcclusionMapId = Shader.PropertyToID("_OcclusionMap");
        private static readonly int EmissionMapId = Shader.PropertyToID("_EmissionMap");
        private static readonly int EmissionColorId = Shader.PropertyToID("_EmissionColor");

        private static Shader? _urpLitShader;
        private static bool _urpLitResolved;
        private static bool _shaderMissingLogged;

        /// <summary>
        /// The tier the upgrader currently honours. Set by <see cref="GraphicsMode"/> when it
        /// applies a tier so the upgrade path tracks the post-process toggle in lock-step
        /// (reversible: flipping back to Vanilla makes <see cref="Upgrade"/> a passthrough).
        /// </summary>
        public static GraphicsTier ActiveTier { get; set; } = GraphicsTier.Vanilla;

        /// <summary>
        /// Upgrade <paramref name="baseMaterial"/> for the active tier.
        /// </summary>
        /// <param name="baseMaterial">The material to upgrade (e.g. the swapped RenderMesh material).</param>
        /// <param name="assetKey">
        /// The pack <c>visual_asset</c> key (or material/mesh name) used to look up declared PBR
        /// textures in <see cref="PbrMaterialRegistry"/>. May be null/empty — the shader-only
        /// upgrade still applies.
        /// </param>
        /// <returns>
        /// A new URP/Lit material when an upgrade was applied, otherwise the original
        /// <paramref name="baseMaterial"/> (including when tier is Vanilla or on any failure).
        /// </returns>
        public static Material? Upgrade(Material? baseMaterial, string? assetKey)
        {
            if (baseMaterial == null) return baseMaterial;

            // Vanilla tier (or anything other than High) is a strict passthrough.
            if (ActiveTier != GraphicsTier.High) return baseMaterial;

            try
            {
                Shader? lit = ResolveUrpLit();
                if (lit == null)
                {
                    if (!_shaderMissingLogged)
                    {
                        _shaderMissingLogged = true;
                        DebugLog.Write(LogCategory,
                            $"URP/Lit shader '{UrpLitShaderName}' not found; PBR upgrade disabled (passthrough).");
                    }
                    return baseMaterial;
                }

                PbrMaterialRegistry.TryGet(assetKey ?? string.Empty, out PbrTextureSet? pbr);

                Material upgraded = new Material(lit)
                {
                    name = $"{baseMaterial.name}__DINOForge_PBR"
                };

                CarryOverBaseColor(baseMaterial, upgraded);
                ApplyPbr(upgraded, pbr);

                DebugLog.Write(LogCategory,
                    $"Upgraded material '{baseMaterial.name}' → URP/Lit (key='{assetKey ?? "<null>"}', " +
                    $"pbrTextures={(pbr?.HasAnyTexture == true)}).");
                return upgraded;
            }
            catch (Exception ex)
            {
                // Graceful degradation: never break a swap because of a material upgrade.
                DebugLog.Write(LogCategory,
                    $"Upgrade failed for '{baseMaterial.name}' ({ex.GetType().Name}: {ex.Message}); returning original.");
                return baseMaterial;
            }
        }

        /// <summary>Copies the base albedo map + tint from the source material into the URP/Lit upgrade.</summary>
        private static void CarryOverBaseColor(Material source, Material target)
        {
            // Prefer URP's _BaseMap, fall back to legacy _MainTex if the source used Built-in/Simple-Lit.
            Texture? mainTex = null;
            if (source.HasProperty(BaseMapId)) mainTex = source.GetTexture(BaseMapId);
            if (mainTex == null && source.HasProperty("_MainTex")) mainTex = source.GetTexture("_MainTex");
            if (mainTex != null && target.HasProperty(BaseMapId)) target.SetTexture(BaseMapId, mainTex);

            Color tint = Color.white;
            if (source.HasProperty(BaseColorId)) tint = source.GetColor(BaseColorId);
            else if (source.HasProperty("_Color")) tint = source.GetColor("_Color");
            if (target.HasProperty(BaseColorId)) target.SetColor(BaseColorId, tint);
        }

        /// <summary>Applies the declared PBR maps + scalars onto the URP/Lit target material.</summary>
        private static void ApplyPbr(Material target, PbrTextureSet? pbr)
        {
            // Always set a sensible metallic/smoothness baseline so even texture-less assets
            // gain proper specular response under the High tier.
            float metallic = pbr?.Metallic01 ?? 0f;
            float smoothness = pbr?.Smoothness01 ?? 0.35f;
            if (target.HasProperty(MetallicId)) target.SetFloat(MetallicId, Mathf.Clamp01(metallic));
            if (target.HasProperty(SmoothnessId)) target.SetFloat(SmoothnessId, Mathf.Clamp01(smoothness));

            if (pbr == null || !pbr.HasAnyTexture) return;

            if (pbr.Albedo != null && target.HasProperty(BaseMapId))
                target.SetTexture(BaseMapId, pbr.Albedo);

            if (pbr.Metallic != null && target.HasProperty(MetallicGlossMapId))
            {
                target.SetTexture(MetallicGlossMapId, pbr.Metallic);
                target.EnableKeyword("_METALLICSPECGLOSSMAP");
            }

            if (pbr.Normal != null && target.HasProperty(BumpMapId))
            {
                target.SetTexture(BumpMapId, pbr.Normal);
                target.EnableKeyword("_NORMALMAP");
            }

            if (pbr.Occlusion != null && target.HasProperty(OcclusionMapId))
            {
                target.SetTexture(OcclusionMapId, pbr.Occlusion);
                target.EnableKeyword("_OCCLUSIONMAP");
            }

            if (pbr.Emission != null && target.HasProperty(EmissionMapId))
            {
                target.SetTexture(EmissionMapId, pbr.Emission);
                if (target.HasProperty(EmissionColorId)) target.SetColor(EmissionColorId, Color.white);
                target.EnableKeyword("_EMISSION");
                target.globalIlluminationFlags = MaterialGlobalIlluminationFlags.RealtimeEmissive;
            }
        }

        /// <summary>Resolves the URP/Lit shader once, caching the result (may be null on non-URP).</summary>
        private static Shader? ResolveUrpLit()
        {
            if (_urpLitResolved) return _urpLitShader;
            _urpLitResolved = true;
            _urpLitShader = Shader.Find(UrpLitShaderName);
            return _urpLitShader;
        }

        /// <summary>Resets cached shader resolution (hot-reload / test teardown).</summary>
        public static void ResetCache()
        {
            _urpLitResolved = false;
            _urpLitShader = null;
            _shaderMissingLogged = false;
        }
    }
}
