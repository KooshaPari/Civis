#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using DINOForge.Runtime.Diagnostics;
using DINOForge.SDK.Assets;
// #613 dedup: was alias to DINOForge.Runtime.Assets.AssetService (retired); now points to SDK canonical impl.
using RuntimeAssetService = DINOForge.SDK.Assets.AssetService;
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
        // Iter-144: dump EntityManager shared-component methods once on reflection failure
        // so we can diagnose Pattern #101 against DINO's actual Unity 2021.3 EntityManager surface.
        private static bool _dumpedEmMethods;

        // Iter-146 #881: throttle reflection-failure logs (Pattern #232 — was firing every retry).
        // Counter is per-instance; ScheduleReset() clears _dumpedEmMethods but this counter
        // continues across resets to keep the periodic-warn cadence stable.
        private int _reflectionFailCount;
        private const int ReflectionFailLogEvery = 50;

        /// <summary>Requests a full asset swap reset on next OnUpdate cycle (thread-safe).</summary>
        public static void ScheduleReset()
        {
            _resetPending = true;
            // #608 P2: reset diagnostic dump latch so retries re-dump EntityManager methods
            // (HRV1↔HRV2 transitions, Unity-version upgrades, or scene-reload-induced surface
            // changes all benefit from a fresh dump on the next failure).
            _dumpedEmMethods = false;
            // Also clear the RenderMesh type cache so a different DOTS variant
            // (HRV1 RenderMesh → HRV2 RenderMeshUnmanaged / MaterialMeshInfo) is re-resolved
            // after a scene transition or hot-reload.
            _renderMeshResolved = false;
            _renderMeshType = null;
            _renderMeshVariantLogged = false;
        }

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
            DebugLog.Write("AssetSwap", "AssetSwapSystem.OnCreate");
        }

        /// <inheritdoc/>
        protected override void OnUpdate()
        {
            if (_resetPending)
            {
                _resetPending = false;
                _frameCount = 0;
                _reportedFailures.Clear();
                DebugLog.Write("AssetSwap", "AssetSwapSystem.ScheduleReset: frame counter reset, will re-apply swaps after delay.");
            }

            _frameCount++;

            if (_frameCount < MinFrameDelay)
                return;

            IReadOnlyList<AssetSwapRequest> pending = AssetSwapRegistry.GetPending();
            if (pending.Count == 0)
                return;

            DebugLog.Write("AssetSwap", $"AssetSwapSystem: processing {pending.Count} pending swap(s)");

            string patchDir = Path.Combine(BepInEx.Paths.BepInExRootPath, PatchedBundlesDir);
            RuntimeAssetService assetService = new RuntimeAssetService(BepInEx.Paths.GameRootPath);

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
                        DebugLog.Write("AssetSwap", $"AssetSwapSystem: swap applied — address='{request.AssetAddress}' " +
                                   $"asset='{request.AssetName}'");
                    }
                    else
                    {
                        AssetSwapRegistry.MarkFailed(request.AssetAddress);
                        failed++;
                        int newCount = request.FailCount;
                        if (newCount >= AssetSwapRegistry.MaxRetries)
                        {
                            DebugLog.Write("AssetSwap", $"AssetSwapSystem: giving up on '{request.AssetAddress}' " +
                                       $"after {newCount} failures");
                        }
                        else if (_reportedFailures.Add(request.AssetAddress))
                        {
                            DebugLog.Write("AssetSwap", $"AssetSwapSystem: swap failed — address='{request.AssetAddress}' " +
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
                        DebugLog.Write("AssetSwap", $"AssetSwapSystem: swap exception for '{request.AssetAddress}': {ex.Message}");
                    }
                }
            }

            assetService.Dispose();
            DebugLog.Write("AssetSwap", $"AssetSwapSystem: batch complete — {succeeded} succeeded, {failed} failed");
        }

        /// <summary>
        /// Applies a single asset swap: patches the vanilla bundle on disk and,
        /// if the mod bundle contains a Unity Mesh or Material, attempts a live
        /// RenderMesh swap on matched ECS entities.
        /// </summary>
        private bool ApplySwap(AssetSwapRequest request, string patchDir, RuntimeAssetService assetService)
        {
            // Resolve the mod bundle path (relative paths against BepInEx plugins dir).
            string modBundleFullPath = ResolveModBundlePath(request.ModBundlePath);
            if (!File.Exists(modBundleFullPath))
            {
                DebugLog.Write("AssetSwap", $"ApplySwap: mod bundle not found: {modBundleFullPath}");
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

                    if (string.IsNullOrEmpty(vanillaBundlePath))
                    {
                        if (_reportedFailures.Add($"resolve:{request.AssetAddress}"))
                            DebugLog.Write("AssetSwap", $"ApplySwap: unable to resolve vanilla bundle path for '{request.AssetAddress}'");
                    }
                    else if (File.Exists(vanillaBundlePath))
                    {
                        string patchedFileName = Path.GetFileName(vanillaBundlePath);
                        string outputPath = Path.Combine(patchDir, patchedFileName);

                        patchResult = assetService.ReplaceAsset(
                            vanillaBundlePath,
                            request.AssetAddress,
                            modAssetBytes,
                            outputPath);

                        if (patchResult)
                            DebugLog.Write("AssetSwap", $"ApplySwap: patched bundle written to '{outputPath}'");
                        else
                            DebugLog.Write("AssetSwap", $"ApplySwap: bundle patch failed for '{request.AssetAddress}'");
                    }
                }
                else if (_reportedFailures.Add($"catalog:{request.AssetAddress}"))
                {
                    DebugLog.Write("AssetSwap", $"ApplySwap: address '{request.AssetAddress}' not in catalog — skipping disk patch, using entity swap only");
                }
            }
            else if (_reportedFailures.Add($"extract:{request.AssetAddress}"))
            {
                DebugLog.Write("AssetSwap", $"ApplySwap: could not extract '{request.AssetName}' from '{modBundleFullPath}' — using entity swap only");
            }

            // Best-effort live RenderMesh swap on ECS entities.
            bool entitySwapResult = TrySwapRenderMeshFromBundle(
                modBundleFullPath, request.AssetName, request.VanillaMapping);
            DebugLog.Write("AssetSwap", $"ApplySwap: entity swap result={entitySwapResult} for '{request.AssetAddress}'");

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
                        DebugLog.Write("AssetSwap", $"TrySwapRenderMeshFromBundle: extracted from prefab '{assetName}'");
                }
            }

            if (replacementMesh == null && replacementMat == null)
            {
                DebugLog.Write("AssetSwap",
                    $"TrySwapRenderMeshFromBundle: no Mesh/Material named '{assetName}' in bundle");
                return false;
            }

            Type? renderMeshType = ResolveRenderMeshType();
            if (renderMeshType == null)
            {
                DebugLog.Write("AssetSwap", "TrySwapRenderMeshFromBundle: Unity.Rendering.RenderMesh type not found");
                return false;
            }

            // #608 P2: HRV2 (RenderMeshUnmanaged / MaterialMeshInfo) uses blittable structs
            // with readonly mesh/material data — the legacy FieldInfo.SetValue path doesn't
            // apply. Bail out gracefully until HRV2 mesh-swap is implemented (separate task).
            if (IsHrv2Type(_renderMeshVariantName))
            {
                DebugLog.Write("AssetSwap", $"TrySwapRenderMeshFromBundle: HRV2 mesh-swap not yet implemented (variant='{_renderMeshVariantName}') — falling back to no-op for entity swap. Bundle-disk patch (if successful) still applies.");
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
                    DebugLog.Write("AssetSwap",
                        $"TrySwapRenderMeshFromBundle: filtering by '{archetypeTypeName}' " +
                        $"for vanilla_mapping='{vanillaMapping}'");
                }
                else
                {
                    DebugLog.Write("AssetSwap",
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
            // Iter-144 fix: GetMethod(name, types[]) returns null at runtime against DINO's
            // Unity 2021.3 EntityManager (overload-resolution mismatch). Mirror the arity-filter
            // pattern used below for SetSharedComponentData: enumerate methods, filter on name,
            // non-generic, arity=2, first param Entity, second param ComponentType.
            // Mono 4.x type-identity bug: typeof(Entity) != param.ParameterType across
            // assembly boundaries. Use FullName string comparison instead.
            MethodInfo? getSharedNonGeneric = typeof(EntityManager).GetMethods(
                    BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance)
                .FirstOrDefault(m =>
                    m.Name == "GetSharedComponentData"
                    && !m.IsGenericMethodDefinition
                    && m.GetParameters().Length == 2
                    && m.GetParameters()[0].ParameterType.FullName == "Unity.Entities.Entity"
                    && (m.GetParameters()[1].ParameterType.FullName == "Unity.Entities.ComponentType"
                        || m.GetParameters()[1].ParameterType.FullName == "System.Int32"));
            MethodInfo? setSharedGeneric = typeof(EntityManager).GetMethods(
                    BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance)
                .FirstOrDefault(m =>
                    m.Name == "SetSharedComponentData"
                    && m.IsGenericMethodDefinition
                    && m.GetParameters().Length == 2
                    && m.GetParameters()[0].ParameterType.FullName == "Unity.Entities.Entity");

            // Also resolve generic GetSharedComponentData<T>(Entity) — primary path
            // since the non-generic overload doesn't exist in DINO's Unity.Entities.
            MethodInfo? getSharedGeneric = typeof(EntityManager).GetMethods(
                    BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance)
                .FirstOrDefault(m =>
                    m.Name == "GetSharedComponentData"
                    && m.IsGenericMethodDefinition
                    && m.GetParameters().Length == 1
                    && m.GetParameters()[0].ParameterType.FullName == "Unity.Entities.Entity");
            MethodInfo? boundGet = getSharedGeneric?.MakeGenericMethod(renderMeshType);

            if (setSharedGeneric == null)
            {
                _reflectionFailCount++;
                if (_reflectionFailCount == 1 || (_reflectionFailCount % ReflectionFailLogEvery) == 0)
                {
                    DebugLog.Write("AssetSwap",
                        $"WARN: TrySwapRenderMeshFromBundle: reflection lookup failed (#{_reflectionFailCount}) " +
                        $"(boundGet={boundGet != null}, setSharedGeneric=False).");
                }
                if (!_dumpedEmMethods)
                {
                    _dumpedEmMethods = true;
                    DebugLog.Write("AssetSwap",
                        "[AssetSwap] EntityManager methods dump (one-shot until ScheduleReset). Triggered by reflection-lookup failure...");
                    var allMethods = typeof(EntityManager).GetMethods(
                        BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Static);
                    foreach (var m in allMethods)
                    {
                        if (m.Name == "GetSharedComponentData" || m.Name == "SetSharedComponentData")
                        {
                            var ps = string.Join(", ", m.GetParameters().Select(p => $"{p.ParameterType.Name} {p.Name}"));
                            DebugLog.Write("AssetSwap", $"  EM.{m.Name}(<{ps}>) generic={m.IsGenericMethodDefinition} retType={m.ReturnType.Name}");
                        }
                    }
                }
                entities.Dispose();
                query.Dispose();
                return false;
            }

            MethodInfo genericSet = setSharedGeneric.MakeGenericMethod(renderMeshType);
            FieldInfo? meshField = renderMeshType.GetField("mesh");
            FieldInfo? materialField = renderMeshType.GetField("material");

            // Use generic GetSharedComponentData<RenderMesh>(Entity) — the non-generic overload
            // doesn't exist in DINO's Unity.Entities version.
            MethodInfo? boundGetForSwap = boundGet;
            if (boundGetForSwap == null)
            {
                DebugLog.Write("AssetSwap", "FATAL: boundGet (generic GetSharedComponentData<T>) is null — cannot swap.");
                entities.Dispose();
                query.Dispose();
                return false;
            }

            DebugLog.Write("AssetSwap",
                $"TrySwapRenderMeshFromBundle: reflection OK — boundGet={boundGetForSwap != null}, " +
                $"entities={entities.Length}, renderMeshType={renderMeshType.FullName}");

            int swapCount = 0;
            int nullMeshSkips = 0;
            for (int i = 0; i < entities.Length; i++)
            {
                Entity entity = entities[i];
                try
                {
                    object? renderMesh = boundGetForSwap.Invoke(EntityManager, new object[] { entity });
                    if (renderMesh == null) continue;

                    bool changed = false;
                    if (replacementMesh != null && meshField != null)
                    {
                        object? currentMesh = meshField.GetValue(renderMesh);
                        if (currentMesh == null)
                        {
                            nullMeshSkips++;
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
                            nullMeshSkips++;
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
                    DebugLog.Write("AssetSwap",
                        $"TrySwapRenderMeshFromBundle: entity {entity.Index} has multiple " +
                        $"RenderMesh instances — skipping (ambiguous match)");
                }
                catch (Exception ex)
                {
                    DebugLog.Write("AssetSwap",
                        $"TrySwapRenderMeshFromBundle: failed on entity {entity.Index}: {ex.Message}");
                }
            }

            DebugLog.Write("AssetSwap", $"TrySwapRenderMeshFromBundle: swapped {swapCount}/{entities.Length} entities (nullSkips={nullMeshSkips})");
            entities.Dispose();
            query.Dispose();

            return swapCount > 0;
        }

        // ------------------------------------------------------------------ helpers

        private static Type? _renderMeshType;
        private static bool _renderMeshResolved;
        // #608 P2: tracks which HRV variant was resolved so callers (mesh-swap path) can
        // bail out gracefully on HRV2 (MaterialMeshInfo/RenderMeshUnmanaged) where the
        // public-mutable-field path doesn't apply.
        private static string? _renderMeshVariantName;
        private static bool _renderMeshVariantLogged;
        private static bool _unityRenderingVersionLogged;

        /// <summary>
        /// HRV2 type names (RenderMeshUnmanaged, MaterialMeshInfo). When the resolved
        /// RenderMesh type matches one of these, the legacy FieldInfo.SetValue("mesh"/"material")
        /// swap path is NOT supported and must be skipped (see TrySwapRenderMeshFromBundle).
        /// </summary>
        private static readonly string[] Hrv2TypeNames =
        {
            "Unity.Rendering.RenderMeshUnmanaged",
            "Unity.Rendering.MaterialMeshInfo",
        };

        /// <summary>
        /// Resolves the Unity.Rendering RenderMesh shared-component type from loaded assemblies.
        /// Tries HRV1 ("Unity.Rendering.RenderMesh") first, then falls back to HRV2 variants
        /// ("RenderMeshUnmanaged", "MaterialMeshInfo") so newer DOTS installations are detected
        /// (#608 P2). Caller is responsible for checking <see cref="IsHrv2Type"/> before
        /// attempting field mutation — HRV2 has readonly properties / blittable structs and
        /// requires a different swap strategy (not yet implemented).
        /// </summary>
        private static Type? ResolveRenderMeshType()
        {
            if (_renderMeshResolved) return _renderMeshType;
            _renderMeshResolved = true;

            LogUnityRenderingVersionOnce();

            // Try HRV1 first (DINO 2021.3 baseline), then HRV2 variants.
            string[] candidates =
            {
                "Unity.Rendering.RenderMesh",
                "Unity.Rendering.RenderMeshUnmanaged",
                "Unity.Rendering.MaterialMeshInfo",
            };

            foreach (string typeName in candidates)
            {
                foreach (Assembly asm in AppDomain.CurrentDomain.GetAssemblies())
                {
                    try
                    {
                        Type? t = asm.GetType(typeName, throwOnError: false);
                        if (t != null)
                        {
                            _renderMeshType = t;
                            _renderMeshVariantName = typeName;
                            if (!_renderMeshVariantLogged)
                            {
                                _renderMeshVariantLogged = true;
                                DebugLog.Write("AssetSwap", $"ResolveRenderMeshType: resolved '{typeName}' from assembly '{asm.GetName().Name}' v{asm.GetName().Version}");
                                if (IsHrv2Type(typeName))
                                {
                                    DebugLog.Write("AssetSwap", $"ResolveRenderMeshType: HRV2 variant detected ('{typeName}') — HRV2 mesh-swap not yet implemented, falling back to no-op for entity swaps. Bundle-disk patching (Phase 1) remains functional.");
                                }
                            }
                            return _renderMeshType;
                        }
                    }
                    catch (Exception ex)
                    {
                        /* safe-swallow: type resolution failure is expected when assembly does not contain Unity.Rendering */
                        System.Diagnostics.Debug.WriteLine($"RenderMesh type lookup for '{typeName}' in {asm.GetName().Name} failed: {ex.Message}");
                    }
                }
            }

            DebugLog.Write("AssetSwap", "ResolveRenderMeshType: no HRV1 or HRV2 RenderMesh type found in any loaded assembly. Entity mesh-swap disabled.");
            return null;
        }

        /// <summary>Returns true if the resolved type is an HRV2 variant (no mutable mesh/material fields).</summary>
        private static bool IsHrv2Type(string? typeName)
        {
            if (string.IsNullOrEmpty(typeName)) return false;
            for (int i = 0; i < Hrv2TypeNames.Length; i++)
            {
                if (Hrv2TypeNames[i] == typeName) return true;
            }
            return false;
        }

        /// <summary>
        /// Logs the Unity.Rendering assembly version once at first init. Helps future agents
        /// diagnose Unity-version-dependent reflection bugs (#608 P2). Scans both the currently
        /// loaded assemblies and the executing assembly's referenced assemblies so we capture
        /// the version regardless of whether Unity.Rendering has been JIT-loaded yet.
        /// </summary>
        private static void LogUnityRenderingVersionOnce()
        {
            if (_unityRenderingVersionLogged) return;
            _unityRenderingVersionLogged = true;

            try
            {
                // Pass 1: loaded assemblies (most reliable — actual runtime version).
                foreach (Assembly asm in AppDomain.CurrentDomain.GetAssemblies())
                {
                    AssemblyName name = asm.GetName();
                    if (name.Name == "Unity.Rendering" || name.Name == "Unity.Rendering.Hybrid")
                    {
                        DebugLog.Write("AssetSwap", $"Unity.Rendering assembly: name='{name.Name}' version={name.Version} (loaded)");
                        return;
                    }
                }

                // Pass 2: referenced (but not yet loaded) assemblies from this DLL.
                foreach (AssemblyName refName in Assembly.GetExecutingAssembly().GetReferencedAssemblies())
                {
                    if (refName.Name == "Unity.Rendering" || refName.Name == "Unity.Rendering.Hybrid")
                    {
                        DebugLog.Write("AssetSwap", $"Unity.Rendering assembly: name='{refName.Name}' version={refName.Version} (referenced, not yet loaded)");
                        return;
                    }
                }

                DebugLog.Write("AssetSwap", "Unity.Rendering assembly: NOT FOUND in loaded or referenced assemblies. RenderMesh resolution will fail.");
            }
            catch (Exception ex)
            {
                /* safe-swallow: diagnostic logging must never throw */
                System.Diagnostics.Debug.WriteLine($"LogUnityRenderingVersionOnce failed: {ex.Message}");
            }
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
                catch (Exception ex)
                {
                    /* safe-swallow: type resolution failure is expected when assembly does not contain the target type */
                    System.Diagnostics.Debug.WriteLine($"Type '{typeName}' lookup in {asm.GetName().Name} failed: {ex.Message}");
                }
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
                DebugLog.Write("AssetSwap", $"LoadBundle: file not found: {fullPath}");
                return null;
            }

            try
            {
                AssetBundle bundle = AssetBundle.LoadFromFile(fullPath);
                if (bundle != null)
                {
                    _loadedBundles.Set(path, bundle);
                    DebugLog.Write("AssetSwap", $"LoadBundle: loaded '{fullPath}'");
                }
                return bundle;
            }
            catch (Exception ex)
            {
                DebugLog.Write("AssetSwap", $"LoadBundle: failed '{fullPath}': {ex.Message}");
                return null;
            }
        }

        /// <inheritdoc/>
        protected override void OnDestroy()
        {
            // Iter-144 #543 fix: skip bundle unload when RuntimeDriver is being destroyed as part
            // of a scene transition (NeedsResurrection / s_skipBundleUnload). AssetBundle.Unload(false)
            // mid-swap orphans chicken-sprite placeholders — bundles must survive the scene
            // transition so swapped sprites continue resolving until the new RuntimeDriver +
            // AssetSwapSystem rehydrate the cache.
            //
            // Iter-144 #547 H6 gray-freeze fix: bundle Dispose() walks LRU cache and calls
            // AssetBundle.Unload(false) on each entry. Unity's AssetBundle.Unload can stall
            // when called during scene-transition asset-loading collisions (the new scene's
            // pack-load may have loaded bundles while we're tearing down). Wrap in try and
            // bound the disposal so OnDestroy can return to Unity even if a Unload wedges.
            // Iter-144 #543: AssetSwapSystem.OnDestroy is ONLY invoked when DINO is tearing down
            // the ECS World — which in DINO always happens during scene transitions, NOT during
            // normal gameplay. Bundles MUST survive these transitions to keep chicken-sprite swaps
            // resolved while the new RuntimeDriver + AssetSwapSystem reconstruct the cache. The
            // flag-OR check (NeedsResurrection / s_skipBundleUnload) was unreliable because ECS
            // system OnDestroy fires BEFORE the MonoBehaviour OnDestroy that sets those flags
            // (observed in-log: AssetSwap@14.0004327Z, RuntimeDriver@14.1274287Z = 127ms gap).
            // We also proactively set the companion flag so any other system observing it
            // (e.g. defensive bundle-unload guards elsewhere) behaves consistently.
            Plugin.s_skipBundleUnload = true;
            Plugin.NeedsResurrection = true;
            DebugLog.Write("AssetSwap", $"[AssetSwapSystem] OnDestroy SKIPPED bundle unload — bundles preserved across scene transition (NeedsResurrection={Plugin.NeedsResurrection} s_skipBundleUnload={Plugin.s_skipBundleUnload}). Companion flags set for downstream observers.");

            try
            {
                base.OnDestroy();
            }
            catch (Exception ex)
            {
                DebugLog.Write("AssetSwap", $"AssetSwapSystem.OnDestroy - base.OnDestroy threw {ex.GetType().Name}: {ex.Message}");
            }
        }

    }
}
