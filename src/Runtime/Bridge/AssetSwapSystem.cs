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
    ///   2. This system waits <see cref="MinFrameDelay"/> frames for the game world to fully load.
    ///   3. On each update cycle after the delay, pending swaps are drained from
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
        /// Cache of loaded AssetBundles (LRU with max 10 bundles, auto-unload on eviction).
        /// </summary>
        private readonly AssetBundleCache _loadedBundles = new AssetBundleCache(maxSize: 10);

        /// <summary>
        /// Tracks asset addresses that have already had their first failure logged.
        /// Subsequent failures for the same address are logged at Debug level to reduce noise.
        /// </summary>
        private readonly HashSet<string> _reportedFailures = new HashSet<string>(StringComparer.Ordinal);

        private int _frameCount;

        private static volatile bool _resetPending;

        /// <summary>Requests a full asset swap reset on next OnUpdate cycle (thread-safe).</summary>
        public static void ScheduleReset() => _resetPending = true;

        /// <summary>
        /// Minimum frames to wait before applying swaps.
        /// Must wait for entities to be fully initialized with render data.
        /// </summary>
        private const int MinFrameDelay = 600; // ~10 seconds at 60 fps

        /// <summary>
        /// Subdirectory under BepInEx root where patched bundles are written.
        /// </summary>
        private const string PatchedBundlesDir = "dinoforge_patched_bundles";

        /// <inheritdoc/>
        protected override void OnCreate()
        {
            base.OnCreate();
            WriteDebug("AssetSwapSystem.OnCreate");
        }

        /// <inheritdoc/>
        protected override void OnUpdate()
        {
            if (_resetPending)
            {
                _resetPending = false;
                _frameCount = 0;
                _reportedFailures.Clear();
                WriteDebug("AssetSwapSystem.ScheduleReset: frame counter reset, will re-apply swaps after delay.");
            }

            _frameCount++;

            if (_frameCount < MinFrameDelay)
                return;

            IReadOnlyList<AssetSwapRequest> pending = AssetSwapRegistry.GetPending();
            if (pending.Count == 0)
                return;

            WriteDebug($"AssetSwapSystem: processing {pending.Count} pending swap(s)");

            string patchDir = Path.Combine(BepInEx.Paths.BepInExRootPath, PatchedBundlesDir);
            AssetService assetService = new AssetService(BepInEx.Paths.GameRootPath);

            int succeeded = 0;
            int failed = 0;

            foreach (AssetSwapRequest request in pending)
            {
                try
                {
                    bool result = ApplySwap(request, patchDir, assetService);
                    if (result)
                    {
                        AssetSwapRegistry.MarkApplied(request.AssetAddress);
                        succeeded++;
                        WriteDebug($"AssetSwapSystem: swap applied — address='{request.AssetAddress}' " +
                                   $"asset='{request.AssetName}'");
                    }
                    else
                    {
                        AssetSwapRegistry.MarkFailed(request.AssetAddress);
                        failed++;
                        int newCount = request.FailCount;
                        if (newCount >= AssetSwapRegistry.MaxRetries)
                        {
                            WriteDebug($"AssetSwapSystem: giving up on '{request.AssetAddress}' " +
                                       $"after {newCount} failures");
                        }
                        else if (_reportedFailures.Add(request.AssetAddress))
                        {
                            WriteDebug($"AssetSwapSystem: swap failed — address='{request.AssetAddress}' " +
                                       $"(attempt {newCount}/{AssetSwapRegistry.MaxRetries})");
                        }
                    }
                }
                catch (Exception ex)
                {
                    AssetSwapRegistry.MarkFailed(request.AssetAddress);
                    failed++;
                    if (_reportedFailures.Add(request.AssetAddress))
                    {
                        WriteDebug($"AssetSwapSystem: swap exception for '{request.AssetAddress}': {ex.Message}");
                    }
                }
            }

            assetService.Dispose();
            WriteDebug($"AssetSwapSystem: batch complete — {succeeded} succeeded, {failed} failed");
        }

        /// <summary>
        /// Applies a single asset swap: patches the vanilla bundle on disk and,
        /// if the mod bundle contains a Unity Mesh or Material, attempts a live
        /// RenderMesh swap on matched ECS entities.
        /// </summary>
        private bool ApplySwap(AssetSwapRequest request, string patchDir, AssetService assetService)
        {
            // Resolve the mod bundle path (relative paths against BepInEx plugins dir).
            string modBundleFullPath = ResolveModBundlePath(request.ModBundlePath);
            if (!File.Exists(modBundleFullPath))
            {
                WriteDebug($"ApplySwap: mod bundle not found: {modBundleFullPath}");
                return false;
            }

            // Phase 1 (optional): Patch the vanilla bundle on disk.
            // This only works when the AssetAddress matches a real Addressables catalog key.
            // Mod packs typically use bundle filenames as AssetAddress, so catalog lookup
            // may fail — that's expected. Phase 2 (entity swap) is the primary mechanism.
            bool patchResult = false;
            byte[]? modAssetBytes = assetService.ExtractAsset(modBundleFullPath, request.AssetName);

            if (modAssetBytes != null && modAssetBytes.Length > 0)
            {
                IReadOnlyDictionary<string, string> catalog = assetService.ReadCatalog();
                if (catalog.TryGetValue(request.AssetAddress, out string? vanillaBundleRelPath)
                    && !string.IsNullOrEmpty(vanillaBundleRelPath))
                {
                    string vanillaBundlePath = AddressablesCatalog.ResolveBundlePath(
                        vanillaBundleRelPath, BepInEx.Paths.GameRootPath);

                    if (File.Exists(vanillaBundlePath))
                    {
                        string patchedFileName = Path.GetFileName(vanillaBundlePath);
                        string outputPath = Path.Combine(patchDir, patchedFileName);

                        patchResult = assetService.ReplaceAsset(
                            vanillaBundlePath,
                            request.AssetAddress,
                            modAssetBytes,
                            outputPath);

                        if (patchResult)
                            WriteDebug($"ApplySwap: patched bundle written to '{outputPath}'");
                        else
                            WriteDebug($"ApplySwap: bundle patch failed for '{request.AssetAddress}'");
                    }
                }
                else if (_reportedFailures.Add($"catalog:{request.AssetAddress}"))
                {
                    WriteDebug($"ApplySwap: address '{request.AssetAddress}' not in catalog — skipping disk patch, using entity swap only");
                }
            }
            else if (_reportedFailures.Add($"extract:{request.AssetAddress}"))
            {
                WriteDebug($"ApplySwap: could not extract '{request.AssetName}' from '{modBundleFullPath}' — using entity swap only");
            }

            // Best-effort live RenderMesh swap on ECS entities.
            bool entitySwapResult = TrySwapRenderMeshFromBundle(
                modBundleFullPath, request.AssetName, request.VanillaMapping);
            WriteDebug($"ApplySwap: entity swap result={entitySwapResult} for '{request.AssetAddress}'");

            return patchResult || entitySwapResult;
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

            // Use the non-generic GetSharedComponentData(Entity, ComponentType) overload.
            // The generic GetSharedComponentData<T>(Entity) throws "Ambiguous match found"
            // for entities that have multiple instances of T (e.g. a unit with shadow+main mesh).
            MethodInfo? getSharedNonGeneric = typeof(EntityManager).GetMethod(
                "GetSharedComponentData",
                new[] { typeof(Entity), typeof(ComponentType) });
            MethodInfo? setSharedGeneric = typeof(EntityManager).GetMethod(
                "SetSharedComponentData");

            if (getSharedNonGeneric == null || setSharedGeneric == null)
            {
                WriteDebug(
                    "TrySwapRenderMeshFromBundle: GetSharedComponentData/SetSharedComponentData not found");
                entities.Dispose();
                query.Dispose();
                return false;
            }

            MethodInfo genericSet = setSharedGeneric.MakeGenericMethod(renderMeshType);
            FieldInfo? meshField = renderMeshType.GetField("mesh");
            FieldInfo? materialField = renderMeshType.GetField("material");

            ComponentType renderMeshComponentType = ComponentType.ReadOnly(renderMeshType);
            int swapCount = 0;
            for (int i = 0; i < entities.Length; i++)
            {
                Entity entity = entities[i];
                try
                {
                    if (!EntityManager.HasComponent(entity, renderMeshComponentType))
                        continue;

                    // Use non-generic overload to avoid "Ambiguous match found" on multi-mesh entities.
                    object? renderMesh = getSharedNonGeneric.Invoke(
                        EntityManager, new object[] { entity, renderMeshComponentType });
                    if (renderMesh == null) continue;

                    bool changed = false;
                    if (replacementMesh != null && meshField != null)
                    {
                        object? currentMesh = meshField.GetValue(renderMesh);
                        if (currentMesh == null)
                        {
                            WriteDebug(
                                $"TrySwapRenderMeshFromBundle: mesh field is null on entity {entity.Index} " +
                                $"(building/entity not yet loaded — skipping)");
                            continue;
                        }
                        meshField.SetValue(renderMesh, replacementMesh);
                        changed = true;
                    }
                    if (replacementMat != null && materialField != null)
                    {
                        object? currentMat = materialField.GetValue(renderMesh);
                        if (currentMat == null)
                        {
                            WriteDebug(
                                $"TrySwapRenderMeshFromBundle: material field is null on entity {entity.Index} " +
                                $"(building/entity not yet loaded — skipping)");
                            continue;
                        }
                        materialField.SetValue(renderMesh, replacementMat);
                        changed = true;
                    }

                    if (changed)
                    {
                        genericSet.Invoke(EntityManager, new object[] { entity, renderMesh });
                        swapCount++;
                    }
                }
                catch (TargetInvocationException ex) when (
                    ex.InnerException?.Message.Contains("Ambiguous match found") == true)
                {
                    // Entity has multiple RenderMesh instances (shadow + main mesh).
                    // Skip — unit swaps only need one mesh visible anyway.
                    WriteDebug(
                        $"TrySwapRenderMeshFromBundle: entity {entity.Index} has multiple " +
                        $"RenderMesh instances — skipping (ambiguous match)");
                }
                catch (Exception ex)
                {
                    WriteDebug(
                        $"TrySwapRenderMeshFromBundle: failed on entity {entity.Index}: {ex.Message}");
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
        /// Loads an AssetBundle from disk, caching the result (LRU with auto-eviction).
        /// </summary>
        private AssetBundle? LoadBundle(string path)
        {
            AssetBundle? cached = _loadedBundles.Get(path);
            if (cached != null)
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
                    _loadedBundles.Set(path, bundle);
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
            _loadedBundles.Dispose();

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
