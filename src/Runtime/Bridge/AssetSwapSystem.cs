#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using DINOForge.SDK.Assets;
using Unity.Collections;
using Unity.Entities;
using UnityEngine;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// ECS System that applies pending asset swaps registered via <see cref="AssetSwapRegistry"/>.
    ///
    /// Lifecycle:
    ///   1. Mod pack loaders call <see cref="AssetSwapRegistry.Register"/> (SDK layer, any thread).
    ///   2. On the first <see cref="OnUpdate"/> tick that has pending requests, vanilla bundles
    ///      are patched on disk via <see cref="AssetService.ReplaceAsset"/> (phase 1, no ECS dep).
    ///   3. On each subsequent update, once <see cref="EntityQueries.GetRenderMeshEntities"/>
    ///      returns a non-empty result, pending swaps are drained from
    ///      <see cref="AssetSwapRegistry"/>, patched bundles are written to
    ///      <c>BepInEx/dinoforge_patched_bundles/</c> via <see cref="AssetService.ReplaceAsset"/>,
    ///      and <see cref="AssetSwapRegistry.MarkApplied"/> is called on success.
    ///   4. The system also applies RenderMesh visual swaps for ECS entities matching
    ///      the source asset address - bridging the bundle write path to live entities.
    ///
    /// Thread safety:
    ///   - <see cref="AssetSwapRegistry"/> is thread-safe; this system only reads from the
    ///     main Unity thread (ECS SystemBase guarantee).
    ///
    /// Architecture notes:
    ///   - DINO uses Unity's Hybrid Renderer V2 (or similar) for ECS rendering.
    ///   - Visual data is stored in RenderMesh shared components.
    ///   - Asset replacement works by (a) patching the vanilla bundle file with the mod's bytes
    ///     and (b) swapping Mesh/Material references on matched entities so the live game sees
    ///     the new assets without a scene reload.
    ///
    /// Manual testing:
    ///   1. Build a test AssetBundle with a replacement mesh/material.
    ///   2. Register a swap via <see cref="AssetSwapRegistry.Register"/>.
    ///   3. Load game and verify visual change on target entities.
    ///   4. Check <c>BepInEx/dinoforge_debug.log</c> for swap results.
    ///
    /// Entity dump analysis confirms DINO uses Unity.Rendering.RenderMesh shared
    /// components (Hybrid Renderer V1 style). Static environment archetypes show
    /// RenderMesh + BuiltinMaterialPropertyColor + RenderBounds + PerInstanceCullingTag.
    /// The swap targets the RenderMesh shared component to replace mesh/material refs.
    /// </summary>
    [UpdateInGroup(typeof(PresentationSystemGroup))]
    public class AssetSwapSystem : SystemBase
    {
        /// <summary>
        /// Cache of loaded AssetBundles keyed by file path (used for RenderMesh visual swap).
        /// </summary>
        private readonly Dictionary<string, AssetBundle> _loadedBundles =
            new Dictionary<string, AssetBundle>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// Tracks which asset addresses have had their disk bundle patched (phase 1).
        /// Must use OrdinalIgnoreCase to match asset address lookups elsewhere in the system.
        /// </summary>
        private readonly HashSet<string> _patchedAddresses =
            new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// Cached entity query for detecting when RenderMesh entities exist.
        /// Created lazily on first <see cref="OnUpdate"/> via <see cref="EntityQueries.GetRenderMeshEntities"/>.
        /// </summary>
        private EntityQuery _renderMeshProbeQuery;
        private bool _probeQueryCreated;

        /// <summary>
        /// Subdirectory under BepInEx root where patched bundles are written.
        /// </summary>
        private const string PatchedBundlesDir = "dinoforge_patched_bundles";

        /// <inheritdoc/>
        protected override void OnCreate()
        {
            base.OnCreate();
            WriteDebug("AssetSwapSystem.OnCreate — awaiting pack load before patching");
            // NOTE: packs are not loaded yet at OnCreate time (LoadPacks() is called from
            // RuntimeDriver.Update() after the ECS world becomes available). Bundle patching
            // and entity swaps both happen in OnUpdate once pending registrations appear.
        }

        /// <inheritdoc/>
        protected override void OnUpdate()
        {
            IReadOnlyList<AssetSwapRequest> pending = AssetSwapRegistry.GetPending();
            if (pending.Count == 0)
                return;

            // Phase 1: patch vanilla bundles on disk for any address not yet patched.
            // No ECS dependency — run as soon as packs are loaded (i.e. pending.Count > 0).
            // Read the catalog once outside the loop to avoid repeated file I/O.
            PatchUnpatchedBundles(pending);

            // Phase 2: live RenderMesh entity swap — only when entities exist.
            // Use EntityQueries helper to keep ECS query patterns centralized in the Bridge layer.
            if (!_probeQueryCreated)
            {
                EntityQuery? probeQuery = DINOForge.Runtime.Bridge.EntityQueries.GetRenderMeshEntities(EntityManager);
                if (probeQuery == null)
                    return; // RenderMesh type not loaded yet

                _renderMeshProbeQuery = probeQuery.Value;
                _probeQueryCreated = true;
            }

            if (_renderMeshProbeQuery.CalculateEntityCount() == 0)
                return;

            WriteDebug($"AssetSwapSystem: RenderMesh entities present — " +
                       $"attempting live swap for {pending.Count} request(s)");

            foreach (AssetSwapRequest request in pending)
            {
                try
                {
                    string modBundleFullPath = ResolveModBundlePath(request.ModBundlePath);
                    bool entitySwapped = TrySwapRenderMeshFromBundle(
                        modBundleFullPath, request.AssetName, request.VanillaMapping);

                    // Only mark applied once the live entity swap succeeds. The disk patch
                    // alone is not sufficient — entities using the vanilla address must be
                    // updated in-memory for the current session. If entity swap fails (e.g.
                    // no matching entities yet) the request remains pending for the next frame.
                    if (entitySwapped)
                    {
                        AssetSwapRegistry.MarkApplied(request.AssetAddress);
                        WriteDebug($"AssetSwapSystem: swap complete — address='{request.AssetAddress}' " +
                                   $"bundlePatched={_patchedAddresses.Contains(request.AssetAddress)}");
                    }
                    else
                    {
                        WriteDebug($"AssetSwapSystem: live swap pending — address='{request.AssetAddress}' " +
                                   $"(no matching entities yet)");
                    }
                }
                catch (Exception ex)
                {
                    WriteDebug($"AssetSwapSystem: swap exception for '{request.AssetAddress}': {ex.Message}");
                }
            }
        }

        /// <summary>
        /// Patches vanilla bundle files on disk for all pending requests not yet patched.
        /// Called from <see cref="OnUpdate"/> on every frame that has pending swaps, but
        /// each address is patched at most once (tracked via <see cref="_patchedAddresses"/>).
        /// The catalog is read once per call rather than once per request to minimise file I/O.
        /// </summary>
        private void PatchUnpatchedBundles(IReadOnlyList<AssetSwapRequest> pending)
        {
            // Quick check: if all pending addresses are already patched, nothing to do.
            bool anyUnpatched = false;
            foreach (AssetSwapRequest r in pending)
            {
                if (!_patchedAddresses.Contains(r.AssetAddress)) { anyUnpatched = true; break; }
            }
            if (!anyUnpatched) return;

            string patchDir = Path.Combine(BepInEx.Paths.BepInExRootPath, PatchedBundlesDir);
            int patched = 0;
            int skipped = 0;

            using AssetService assetService = new AssetService(BepInEx.Paths.GameRootPath);

            // Read catalog once — it doesn't change between requests.
            // Guard against catalog read failures so phase 2 entity swaps still run this frame.
            IReadOnlyDictionary<string, string> catalog;
            try
            {
                catalog = assetService.ReadCatalog();
            }
            catch (Exception ex)
            {
                WriteDebug($"PatchUnpatchedBundles: catalog read failed — {ex.Message}");
                return;
            }

            foreach (AssetSwapRequest request in pending)
            {
                if (_patchedAddresses.Contains(request.AssetAddress))
                    continue;

                try
                {
                    string modBundleFullPath = ResolveModBundlePath(request.ModBundlePath);
                    if (!File.Exists(modBundleFullPath))
                    {
                        WriteDebug($"PatchUnpatchedBundles: mod bundle not found: {modBundleFullPath}");
                        skipped++;
                        continue;
                    }

                    byte[]? modAssetBytes = assetService.ExtractAsset(modBundleFullPath, request.AssetName);
                    if (modAssetBytes == null || modAssetBytes.Length == 0)
                    {
                        WriteDebug($"PatchUnpatchedBundles: could not extract '{request.AssetName}' " +
                                   $"from '{modBundleFullPath}'");
                        skipped++;
                        continue;
                    }

                    if (!catalog.TryGetValue(request.AssetAddress, out string? vanillaBundleRelPath)
                        || string.IsNullOrEmpty(vanillaBundleRelPath))
                    {
                        WriteDebug($"PatchUnpatchedBundles: address '{request.AssetAddress}' not in catalog");
                        skipped++;
                        continue;
                    }

                    string vanillaBundlePath = AddressablesCatalog.ResolveBundlePath(
                        vanillaBundleRelPath, BepInEx.Paths.GameRootPath);

                    if (!File.Exists(vanillaBundlePath))
                    {
                        WriteDebug($"PatchUnpatchedBundles: vanilla bundle not found: {vanillaBundlePath}");
                        skipped++;
                        continue;
                    }

                    string outputPath = Path.Combine(patchDir, Path.GetFileName(vanillaBundlePath));
                    bool ok = assetService.ReplaceAsset(
                        vanillaBundlePath, request.AssetAddress, modAssetBytes, outputPath);

                    if (ok)
                    {
                        _patchedAddresses.Add(request.AssetAddress);
                        patched++;
                        WriteDebug($"PatchUnpatchedBundles: patched '{request.AssetAddress}' → '{outputPath}'");
                    }
                    else
                    {
                        skipped++;
                    }
                }
                catch (Exception ex)
                {
                    skipped++;
                    WriteDebug($"PatchUnpatchedBundles: exception for '{request.AssetAddress}': {ex.Message}");
                }
            }

            if (patched > 0 || skipped > 0)
                WriteDebug($"PatchUnpatchedBundles: {patched} patched, {skipped} skipped");
        }

        /// <summary>
        /// Attempts to load a Mesh or Material from the mod bundle and apply it to ECS entities
        /// carrying a RenderMesh shared component.
        /// When <paramref name="vanillaMapping"/> is provided the entity query is narrowed to only
        /// entities that also carry the corresponding unit-archetype component (e.g.
        /// <c>Components.MeleeUnit</c>), preventing the replacement from touching unrelated geometry.
        /// </summary>
        private bool TrySwapRenderMeshFromBundle(
            string modBundlePath, string assetName, string? vanillaMapping)
        {
            AssetBundle? bundle = LoadBundle(modBundlePath);
            if (bundle == null) return false;

            Mesh? replacementMesh = bundle.LoadAsset<Mesh>(assetName);
            Material? replacementMat = bundle.LoadAsset<Material>(assetName);

            // Bundles built from Unity prefabs store a GameObject hierarchy, not a bare Mesh/Material.
            // Fall back to loading the prefab and extracting its mesh and material.
            // Prefer SkinnedMeshRenderer (animated characters) so mesh+material always come from
            // the same component — avoids mismatches when both SMR and static MF/MR exist.
            if (replacementMesh == null && replacementMat == null)
            {
                GameObject? prefab = bundle.LoadAsset<GameObject>(assetName);
                if (prefab != null)
                {
                    SkinnedMeshRenderer? smr = prefab.GetComponentInChildren<SkinnedMeshRenderer>();
                    if (smr != null && smr.sharedMesh != null)
                    {
                        replacementMesh = smr.sharedMesh;
                        if (smr.sharedMaterials.Length > 0)
                            replacementMat = smr.sharedMaterials[0];
                    }
                    else
                    {
                        // Static mesh fallback — extract from the same object to stay consistent.
                        MeshFilter? mf = prefab.GetComponentInChildren<MeshFilter>();
                        if (mf != null)
                            replacementMesh = mf.sharedMesh;

                        MeshRenderer? mr = prefab.GetComponentInChildren<MeshRenderer>();
                        if (mr != null && mr.sharedMaterials.Length > 0)
                            replacementMat = mr.sharedMaterials[0];
                    }

                    if (replacementMesh != null || replacementMat != null)
                        WriteDebug($"TrySwapRenderMeshFromBundle: extracted from prefab '{assetName}'");
                }
            }

            if (replacementMesh == null && replacementMat == null)
            {
                WriteDebug(
                    $"TrySwapRenderMeshFromBundle: no Mesh/Material named '{assetName}' in bundle");
                return false;
            }

            Type? renderMeshType = ResolveRenderMeshType();
            if (renderMeshType == null)
            {
                WriteDebug("TrySwapRenderMeshFromBundle: Unity.Rendering.RenderMesh type not found");
                return false;
            }

            // Resolve vanilla_mapping → ECS component type for targeted entity filtering.
            // When the mapping is absent or unrecognised we fall back to RenderMesh-only query,
            // which at minimum avoids modifying non-unit geometry in cases like buildings.
            ComponentType[] queryComponents;
            if (!string.IsNullOrWhiteSpace(vanillaMapping)
                && PackStatMappings.TryResolveMapping(vanillaMapping, out string? archetypeTypeName)
                && !string.IsNullOrEmpty(archetypeTypeName))
            {
                Type? archetypeType = ResolveTypeByName(archetypeTypeName!);
                if (archetypeType != null)
                {
                    queryComponents = new[]
                    {
                        ComponentType.ReadOnly(renderMeshType),
                        ComponentType.ReadOnly(archetypeType),
                    };
                    WriteDebug(
                        $"TrySwapRenderMeshFromBundle: filtering by '{archetypeTypeName}' " +
                        $"for vanilla_mapping='{vanillaMapping}'");
                }
                else
                {
                    WriteDebug(
                        $"TrySwapRenderMeshFromBundle: archetype type '{archetypeTypeName}' not " +
                        $"found in assemblies; falling back to RenderMesh-only query");
                    queryComponents = new[] { ComponentType.ReadOnly(renderMeshType) };
                }
            }
            else
            {
                queryComponents = new[] { ComponentType.ReadOnly(renderMeshType) };
            }

            EntityQuery query = EntityManager.CreateEntityQuery(
                new EntityQueryDesc { All = queryComponents });
            NativeArray<Entity> entities = query.ToEntityArray(Allocator.Temp);

            MethodInfo? getShared = typeof(EntityManager).GetMethod(
                "GetSharedComponentData", new[] { typeof(Entity) });
            MethodInfo? setShared = typeof(EntityManager).GetMethod("SetSharedComponentData");

            if (getShared == null || setShared == null)
            {
                WriteDebug(
                    "TrySwapRenderMeshFromBundle: GetSharedComponentData/SetSharedComponentData not found");
                entities.Dispose();
                query.Dispose();
                return false;
            }

            MethodInfo genericGet = getShared.MakeGenericMethod(renderMeshType);
            MethodInfo genericSet = setShared.MakeGenericMethod(renderMeshType);

            FieldInfo? meshField = renderMeshType.GetField("mesh");
            FieldInfo? materialField = renderMeshType.GetField("material");

            int swapCount = 0;
            for (int i = 0; i < entities.Length; i++)
            {
                try
                {
                    if (!EntityManager.HasComponent(entities[i], ComponentType.ReadOnly(renderMeshType)))
                        continue;

                    object? renderMesh = genericGet.Invoke(EntityManager, new object[] { entities[i] });
                    if (renderMesh == null) continue;

                    bool changed = false;
                    if (replacementMesh != null && meshField != null)
                    {
                        meshField.SetValue(renderMesh, replacementMesh);
                        changed = true;
                    }
                    if (replacementMat != null && materialField != null)
                    {
                        materialField.SetValue(renderMesh, replacementMat);
                        changed = true;
                    }

                    if (changed)
                    {
                        genericSet.Invoke(EntityManager, new object[] { entities[i], renderMesh });
                        swapCount++;
                    }
                }
                catch (Exception ex)
                {
                    WriteDebug(
                        $"TrySwapRenderMeshFromBundle: failed on entity {entities[i].Index}: {ex.Message}");
                }
            }

            WriteDebug($"TrySwapRenderMeshFromBundle: swapped {swapCount}/{entities.Length} entities");
            entities.Dispose();
            query.Dispose();

            return swapCount > 0;
        }

        // ------------------------------------------------------------------ helpers

        private static Type? _renderMeshType;
        private static bool _renderMeshResolved;

        /// <summary>
        /// Resolves the Unity.Rendering.RenderMesh type from loaded assemblies.
        /// DINO uses Hybrid Renderer V1 which provides RenderMesh as a shared component.
        /// </summary>
        private static Type? ResolveRenderMeshType()
        {
            if (_renderMeshResolved) return _renderMeshType;
            _renderMeshResolved = true;

            foreach (Assembly asm in AppDomain.CurrentDomain.GetAssemblies())
            {
                try
                {
                    _renderMeshType = asm.GetType("Unity.Rendering.RenderMesh", throwOnError: false);
                    if (_renderMeshType != null) return _renderMeshType;
                }
                catch { }
            }
            return null;
        }

        private static readonly Dictionary<string, Type?> _resolvedTypeCache =
            new Dictionary<string, Type?>(StringComparer.Ordinal);

        /// <summary>
        /// Resolves a fully-qualified type name (e.g. "Components.MeleeUnit") from any loaded assembly.
        /// Results are cached to avoid repeated assembly scans.
        /// </summary>
        private static Type? ResolveTypeByName(string typeName)
        {
            if (_resolvedTypeCache.TryGetValue(typeName, out Type? cached))
                return cached;

            Type? found = null;
            foreach (Assembly asm in AppDomain.CurrentDomain.GetAssemblies())
            {
                try
                {
                    found = asm.GetType(typeName, throwOnError: false);
                    if (found != null) break;
                }
                catch { }
            }

            _resolvedTypeCache[typeName] = found;
            return found;
        }

        /// <summary>
        /// Resolves a mod bundle path. Relative paths are joined against the BepInEx plugins dir.
        /// </summary>
        private static string ResolveModBundlePath(string path)
        {
            return Path.IsPathRooted(path)
                ? path
                : Path.Combine(BepInEx.Paths.PluginPath, path);
        }

        /// <summary>
        /// Loads an AssetBundle from disk, caching the result.
        /// </summary>
        private AssetBundle? LoadBundle(string path)
        {
            if (_loadedBundles.TryGetValue(path, out AssetBundle? cached))
                return cached;

            string fullPath = ResolveModBundlePath(path);

            if (!File.Exists(fullPath))
            {
                WriteDebug($"LoadBundle: file not found: {fullPath}");
                return null;
            }

            try
            {
                AssetBundle bundle = AssetBundle.LoadFromFile(fullPath);
                if (bundle != null)
                {
                    _loadedBundles[path] = bundle;
                    WriteDebug($"LoadBundle: loaded '{fullPath}'");
                }
                return bundle;
            }
            catch (Exception ex)
            {
                WriteDebug($"LoadBundle: failed '{fullPath}': {ex.Message}");
                return null;
            }
        }

        /// <inheritdoc/>
        protected override void OnDestroy()
        {
            foreach (AssetBundle bundle in _loadedBundles.Values)
            {
                try { bundle.Unload(false); }
                catch { }
            }
            _loadedBundles.Clear();

            if (_probeQueryCreated)
                _renderMeshProbeQuery.Dispose();

            base.OnDestroy();
            WriteDebug("AssetSwapSystem.OnDestroy - bundles unloaded");
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = Path.Combine(
                    BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.Now:u}] {msg}\n");
            }
            catch { }
        }
    }
}
