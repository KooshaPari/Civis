#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using DINOForge.Runtime.Diagnostics;
using DINOForge.Runtime.Telemetry;
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

        /// <summary>Whether the one-shot vanilla mesh diagnostic dump has been emitted.</summary>
        private bool _vanillaMeshDumpDone;

        /// <summary>
        /// Maximum number of entities to swap per bundle invocation.
        /// Safety cap to prevent replacing the entire world with one mesh.
        /// </summary>
        private const int MaxSwapsPerBundle = 500;

        /// <summary>
        /// Maps mod bundle name prefixes to vanilla mesh name substrings for selective matching.
        /// Key = substring found in the bundle filename (case-insensitive).
        /// Value = list of vanilla mesh name substrings that should be replaced.
        /// When a bundle's filename contains the key, only entities whose current mesh name
        /// contains one of the value substrings will be swapped.
        ///
        /// This mapping is populated after the first diagnostic dump reveals vanilla mesh names.
        /// Until the mapping is known, the system runs in DIAGNOSTIC MODE (logs mesh names, skips swap).
        /// </summary>
        private static readonly Dictionary<string, string[]> BundleToVanillaMeshMap =
            new Dictionary<string, string[]>(StringComparer.OrdinalIgnoreCase)
            {
                // Derived from DINO vanilla mesh survey (iter-153, 2026-06-03).
                // Keys are substrings of bundle filenames; values are substrings of vanilla mesh names.
                // Infantry / melee units (royal_sword_1=1686, royal_spear=657, royal_tough_guy=636 etc)
                { "b1-battle-droid",      new[] { "royal_sword", "royal_spear", "royal_shortsword", "royal_tough" } },
                { "b2-super-droid",       new[] { "royal_sword", "royal_spear", "royal_shortsword", "royal_tough" } },
                { "bx-commando",          new[] { "royal_sword", "royal_spear", "royal_shortsword", "royal_tough" } },
                { "commando-droid",       new[] { "royal_sword", "royal_spear", "royal_shortsword", "royal_tough" } },
                { "clone-trooper",        new[] { "royal_sword", "royal_spear", "royal_shortsword", "royal_tough" } },
                { "clone-heavy",          new[] { "royal_tough_guy", "royal_sword_2" } },
                { "clone-medic",          new[] { "royal_sword", "royal_spear", "royal_shortsword" } },
                { "clone-militia",        new[] { "royal_sword", "royal_spear", "royal_shortsword" } },
                { "clone-commander",      new[] { "royal_ban", "royal_horseman" } },
                { "clone-engineer",       new[] { "royal_tough_guy", "royal_sword_2" } },
                { "clone-jet",            new[] { "royal_spear", "royal_horseman" } },
                { "clone-mortar",         new[] { "royal_spear", "royal_tough" } },
                { "clone-pilot",          new[] { "royal_horseman_spear", "royal_horseman_sword" } },
                { "arc-trooper",          new[] { "royal_ban", "royal_horseman" } },
                { "cis-droideka",         new[] { "royal_tough_guy", "royal_sword_2" } },
                { "cis-magna",            new[] { "royal_ban", "royal_horseman" } },
                { "cis-spider",           new[] { "royal_tough_guy", "royal_horseman" } },
                { "general-grievous",     new[] { "royal_ban", "royal_horseman" } },
                { "jedi",                 new[] { "royal_ban", "royal_horseman" } },
                { "sniper-droid",         new[] { "royal_spear", "royal_tough" } },
                { "octuptarra",           new[] { "royal_tough_guy", "royal_horseman" } },
                { "probe-droid",          new[] { "royal_spear", "royal_shortsword" } },
                { "grapple-droid",        new[] { "royal_tough_guy", "royal_sword_2" } },
                { "rocket-droid",         new[] { "royal_spear", "royal_tough" } },
                // Aerial units — matched by aerial archetype filter; no mesh substrings needed
                // Building meshes — broad match; archetype filter handles selectivity
                { "clone-barracks",       new[] { "b1_", "ruins_building", "soul_stone" } },
                { "assembly-line",        new[] { "b1_", "ruins_building" } },
                { "weapons-factory",      new[] { "b1_", "ruins_building" } },
                { "guard-tower",          new[] { "b1_", "ruins_building" } },
                { "command-center",       new[] { "b1_", "ruins_building" } },
                { "hangar-bay",           new[] { "b1_", "ruins_building" } },
                { "cis-aa-tower",         new[] { "b1_", "ruins_building" } },
            };

        private static volatile bool _resetPending;
        // Iter-144: dump EntityManager shared-component methods once on reflection failure
        // so we can diagnose Pattern #101 against DINO's actual Unity 2021.3 EntityManager surface.
        private static bool _dumpedEmMethods;

        // Iter-146 #881: throttle reflection-failure logs (Pattern #232 — was firing every retry).
        // Counter is per-instance; ScheduleReset() clears _dumpedEmMethods but this counter
        // continues across resets to keep the periodic-warn cadence stable.
        private int _reflectionFailCount;
        private const int ReflectionFailLogEvery = 50;

        // Iter-148 #912: track which world we selected so we log the switch only once.
        private string _loggedWorldSelection = "";

        /// <summary>
        /// Iter-148 #912: AssetSwapSystem may run in a World that has only prefab templates
        /// (Default World ~25 entities) while gameplay entities live in a separate World.
        /// Scan all worlds and return the EntityManager from the one with the most entities.
        /// </summary>
        private EntityManager FindBestEntityManager(out int bestCount, out string bestName)
        {
            EntityManager best = EntityManager;
            bestCount = -1;
            bestName = World.Name;
            try
            {
                foreach (World w in World.All)
                {
                    if (w == null || !w.IsCreated) continue;
                    int c;
                    try
                    {
                        c = w.EntityManager.UniversalQuery.CalculateEntityCount();
                    }
                    catch
                    {
                        continue;
                    }
                    if (c > bestCount)
                    {
                        bestCount = c;
                        best = w.EntityManager;
                        bestName = w.Name;
                    }
                }
            }
            catch
            {
                // World.All access failed — keep our own EntityManager as fallback
            }
            if (bestCount < 0) bestCount = 0;
            return best;
        }

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
#if NET8_0
        public override void OnCreate()
#else
        protected override void OnCreate()
#endif
        {
            base.OnCreate();
            DebugLog.Write("AssetSwap", "AssetSwapSystem.OnCreate");
        }

        /// <inheritdoc/>
#if NET8_0
        public override void OnUpdate()
#else
        protected override void OnUpdate()
#endif
        {
            if (_resetPending)
            {
                _resetPending = false;
                _frameCount = 0;
                _reportedFailures.Clear();
                DebugLog.Write("AssetSwap", "AssetSwapSystem.ScheduleReset: frame counter reset, will re-apply swaps after delay.");
            }

            _frameCount++;

            // #920: Telemetry — count every OnUpdate invocation.
            try { MetricsCollector.Instance.IncrementCounter("asset_swap.update_calls"); } catch { /* best-effort */ }

            if (_frameCount < MinFrameDelay)
                return;

            IReadOnlyList<AssetSwapRequest> pending = AssetSwapRegistry.GetPending();
            if (pending.Count == 0)
                return;

            // Gate at OnUpdate level: skip the entire swap pass until the world is populated.
            // Iter-148 #912: AssetSwapSystem may run in Default World which only has prefab
            // templates (~25 entities). The actual gameplay world has 49K+ entities. Find
            // the world with the most entities (likely the gameplay world) and use its EM.
            EntityManager bestEm = FindBestEntityManager(out int bestCount, out string bestName);

            // #920: Telemetry — record current best world entity count.
            try { MetricsCollector.Instance.RecordValue("asset_swap.world_entity_count", bestCount); } catch { /* best-effort */ }
            if (bestCount < 1000)
            {
                if (_frameCount % 60 == 0)
                {
                    DebugLog.Write("AssetSwap",
                        $"AssetSwapSystem: waiting for entities (best world='{bestName}' count={bestCount}, need>=1000, frame={_frameCount})");
                }
                return;
            }

            if (_loggedWorldSelection != bestName)
            {
                _loggedWorldSelection = bestName;
                DebugLog.Write("AssetSwap",
                    $"AssetSwapSystem: using world '{bestName}' with {bestCount} entities (was running in '{World.Name}')");
            }

            DebugLog.Write("AssetSwap", $"AssetSwapSystem: processing {pending.Count} pending swap(s)");

            string patchDir = Path.Combine(BepInEx.Paths.BepInExRootPath, PatchedBundlesDir);
            RuntimeAssetService assetService = new RuntimeAssetService(BepInEx.Paths.GameRootPath);

            int succeeded = 0;
            int failed = 0;

            foreach (AssetSwapRequest request in pending)
            {
                try
                {
                    bool result = ApplySwap(request, patchDir, assetService, bestEm);
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
        private bool ApplySwap(AssetSwapRequest request, string patchDir, RuntimeAssetService assetService, EntityManager bestEm)
        {
            // Resolve the mod bundle path (relative paths against BepInEx plugins dir).
            // A null/empty ModBundlePath means Phase 1 (disk patch) is skipped — but Phase 2
            // (entity swap) must still run because the bundle file may be located via pack
            // registration even when ModBundlePath is absent (#992 regression fix).
            bool hasBundlePath = !string.IsNullOrEmpty(request.ModBundlePath);
            string modBundleFullPath = hasBundlePath ? ResolveModBundlePath(request.ModBundlePath) : string.Empty;
            bool bundleFileExists = hasBundlePath && File.Exists(modBundleFullPath);

            if (!hasBundlePath)
            {
                DebugLog.Write("AssetSwap", $"ApplySwap: ModBundlePath is null/empty for address='{request.AssetAddress}' — skipping Phase 1 (disk patch), proceeding to Phase 2 (entity swap)");
            }
            else if (!bundleFileExists)
            {
                DebugLog.Write("AssetSwap", $"ApplySwap: mod bundle not found: {modBundleFullPath} — skipping Phase 1, proceeding to Phase 2");
            }

            // Phase 1 (optional): Patch the vanilla bundle on disk.
            // This only works when the AssetAddress matches a real Addressables catalog key.
            // Mod packs typically use bundle filenames as AssetAddress, so catalog lookup
            // may fail — that's expected. Phase 2 (entity swap) is the primary mechanism.
            // Phase 1 is skipped entirely when the mod bundle file is missing/unavailable.
            bool patchResult = false;
            byte[]? modAssetBytes = null;
            if (bundleFileExists)
            {
                try
                {
                    modAssetBytes = assetService.ExtractAsset(modBundleFullPath, request.AssetName);
                }
                catch (Exception ex)
                {
                    DebugLog.Write("AssetSwap", $"ApplySwap: failed to extract '{request.AssetName}' from '{modBundleFullPath}': {ex.Message} — skipping Phase 1 and continuing with entity swap");
                }
            }

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
                        // Path.GetFileName can return null on some Mono versions when the path
                        // ends with a directory separator — guard to avoid path2 null crash.
                        if (string.IsNullOrEmpty(patchedFileName))
                        {
                            if (_reportedFailures.Add($"filename:{request.AssetAddress}"))
                                DebugLog.Write("AssetSwap", $"ApplySwap: could not extract filename from vanilla bundle path '{vanillaBundlePath}'");
                        }
                        else
                        {
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
            // Phase 2 requires a resolvable bundle path.
            // Fallback: if ModBundlePath was null/empty at registration time, probe the standard
            // pack layout (<BepInEx>/dinoforge_packs/<pack>/assets/bundles/<assetAddress>) across
            // all deployed packs. This covers late-registered swaps and hot-reload scenarios where
            // the bundle file exists but the registry entry was created without a path.
            bool entitySwapResult = false;
            string resolvedBundlePath = modBundleFullPath;
            if (string.IsNullOrEmpty(resolvedBundlePath))
            {
                resolvedBundlePath = FindBundleInPacksDir(request.AssetAddress);
                if (!string.IsNullOrEmpty(resolvedBundlePath))
                {
                    DebugLog.Write("AssetSwap",
                        $"ApplySwap: fallback bundle lookup found '{resolvedBundlePath}' for address='{request.AssetAddress}'");
                }
            }

            if (!string.IsNullOrEmpty(resolvedBundlePath))
            {
                entitySwapResult = TrySwapRenderMeshFromBundle(
                    resolvedBundlePath, request.AssetName, request.VanillaMapping, bestEm);
            }
            else if (_reportedFailures.Add($"phase2-nopath:{request.AssetAddress}"))
            {
                DebugLog.Write("AssetSwap", $"ApplySwap: skipping Phase 2 — no bundle path for address='{request.AssetAddress}' (fallback scan also found nothing)");
            }
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
            string modBundlePath, string assetName, string? vanillaMapping, EntityManager em)
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

            // Final fallback: bundle asset name doesn't match the key (e.g. bundle built with
            // prefab name 'Clone_Heavy_Republic' but key is 'sw-clone-heavy'). Load the first
            // available asset from the bundle and extract mesh/material from it.
            if (replacementMesh == null && replacementMat == null)
            {
                string[] allNames = bundle.GetAllAssetNames();
                if (allNames.Length > 0)
                {
                    DebugLog.Write("AssetSwap",
                        $"TrySwapRenderMeshFromBundle: name '{assetName}' not found in bundle, " +
                        $"trying first asset '{allNames[0]}' (bundle has {allNames.Length} assets)");
                    replacementMesh = bundle.LoadAsset<Mesh>(allNames[0]);
                    replacementMat = bundle.LoadAsset<Material>(allNames[0]);
                    if (replacementMesh == null && replacementMat == null)
                    {
                        GameObject? fallbackPrefab = bundle.LoadAsset<GameObject>(allNames[0]);
                        if (fallbackPrefab != null)
                        {
                            SkinnedMeshRenderer? smr = fallbackPrefab.GetComponentInChildren<SkinnedMeshRenderer>();
                            if (smr != null && smr.sharedMesh != null)
                            {
                                replacementMesh = smr.sharedMesh;
                                if (smr.sharedMaterials.Length > 0) replacementMat = smr.sharedMaterials[0];
                            }
                            else
                            {
                                MeshFilter? mf = fallbackPrefab.GetComponentInChildren<MeshFilter>();
                                if (mf != null) replacementMesh = mf.sharedMesh;
                                MeshRenderer? mr = fallbackPrefab.GetComponentInChildren<MeshRenderer>();
                                if (mr != null && mr.sharedMaterials.Length > 0) replacementMat = mr.sharedMaterials[0];
                            }
                            if (replacementMesh != null || replacementMat != null)
                                DebugLog.Write("AssetSwap", $"TrySwapRenderMeshFromBundle: extracted from fallback prefab '{allNames[0]}'");
                        }
                    }
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

            EntityQuery query = em.CreateEntityQuery(
                new EntityQueryDesc { All = queryComponents, Options = EntityQueryOptions.IncludePrefab });
            NativeArray<Entity> entities = query.ToEntityArray(Allocator.Temp);

            // Iter-148 timing fix: ToEntityArray() may return 0 entities when the swap
            // fires before gameplay-scene entity population completes (observed ~9s gap
            // between AssetSwapSystem first OnUpdate at frame 600 and entities arriving
            // around frame 1000+). Returning false (not true) causes AssetSwapRegistry
            // to MarkFailed, which keeps the request in the pending queue for the next
            // OnUpdate retry. MaxRetries=200 gives a wide retry window past warmup.
            if (entities.Length == 0)
            {
                if (_reportedFailures.Add($"empty-query:{assetName}"))
                {
                    DebugLog.Write("AssetSwap",
                        $"TrySwapRenderMeshFromBundle: entity query returned 0 results for asset='{assetName}' " +
                        $"(vanillaMapping='{vanillaMapping ?? "<null>"}') — entities not yet populated, will retry next frame.");
                }
                entities.Dispose();
                query.Dispose();
                return false;
            }

            // Use the non-generic GetSharedComponentData(Entity, ComponentType) overload.
            // The generic GetSharedComponentData<T>(Entity) throws "Ambiguous match found"
            // for entities that have multiple instances of T (e.g. a unit with shadow+main mesh).
            // Iter-144 fix: GetMethod(name, types[]) returns null at runtime against DINO's
            // Unity 2021.3 EntityManager (overload-resolution mismatch). Mirror the arity-filter
            // pattern used below for SetSharedComponentData: enumerate methods, filter on name,
            // non-generic, arity=2, first param Entity, second param ComponentType.
            // Mono 4.x type-identity bug: typeof(Entity) != param.ParameterType across assembly
            // boundaries. Use FullName string comparison instead of reference equality.
            // Unity 2021.3 EntityManager has GetSharedComponentData(Entity, int typeIndex)
            // as the only non-generic 2-parameter overload — NOT (Entity, ComponentType).
            // We must pass ComponentType.TypeIndex (int) as the argument.
            MethodInfo? getSharedNonGeneric = typeof(EntityManager).GetMethods(
                    BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance)
                .FirstOrDefault(m =>
                    m.Name == "GetSharedComponentData"
                    && !m.IsGenericMethodDefinition
                    && m.GetParameters().Length == 2
                    && m.GetParameters()[0].ParameterType.FullName == "Unity.Entities.Entity"
                    && (m.GetParameters()[1].ParameterType.FullName == "Unity.Entities.ComponentType"
                        || m.GetParameters()[1].ParameterType.FullName == "System.Int32"));
            // #101: SetSharedComponentData<T> has multiple overloads (Entity, EntityQuery,
            // NativeArray<Entity>), so plain GetMethod("SetSharedComponentData") throws
            // AmbiguousMatchException. GetMethod(name, types[]) also can't disambiguate
            // open generics (parameter type is T, not a concrete Type). Filter by arity +
            // first-parameter Entity to pin the (Entity, T) overload.
            MethodInfo? setSharedGeneric = typeof(EntityManager).GetMethods(
                    BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance)
                .FirstOrDefault(m =>
                    m.Name == "SetSharedComponentData"
                    && m.IsGenericMethodDefinition
                    && m.GetParameters().Length == 2
                    && m.GetParameters()[0].ParameterType.FullName == "Unity.Entities.Entity");

            if (getSharedNonGeneric == null || setSharedGeneric == null)
            {
                // Iter-146 #881: throttle ungated log — was firing every retry (Pattern #232).
                // Emit a single LogWarning every Nth failure to preserve actionable signal.
                _reflectionFailCount++;
                if (_reflectionFailCount == 1 || (_reflectionFailCount % ReflectionFailLogEvery) == 0)
                {
                    DebugLog.Write("AssetSwap",
                        $"WARN: TrySwapRenderMeshFromBundle: reflection lookup failed (#{_reflectionFailCount}) " +
                        $"(getSharedNonGeneric={getSharedNonGeneric != null}, setSharedGeneric={setSharedGeneric != null}).");
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

            ComponentType renderMeshComponentType = ComponentType.ReadOnly(renderMeshType);

            // ---- DIAGNOSTIC PASS: log unique vanilla mesh names (one-shot) ----
            // Uses generic GetSharedComponentData<RenderMesh>(Entity) via MakeGenericMethod
            // to avoid the Mono type-identity bug with non-generic overload parameter matching.
            if (!_vanillaMeshDumpDone && meshField != null && setSharedGeneric != null)
            {
                _vanillaMeshDumpDone = true;
                var uniqueMeshNames = new Dictionary<string, int>(StringComparer.Ordinal);
                int scanned = 0;
                int errors = 0;

                // Find the generic GetSharedComponentData<T>(Entity) for reading
                MethodInfo? genericGet = typeof(EntityManager).GetMethods(
                        BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance)
                    .FirstOrDefault(m =>
                        m.Name == "GetSharedComponentData"
                        && m.IsGenericMethodDefinition
                        && m.GetParameters().Length == 1
                        && m.GetParameters()[0].ParameterType.FullName == "Unity.Entities.Entity");

                MethodInfo? boundGet = genericGet?.MakeGenericMethod(renderMeshType);

                if (boundGet != null)
                {
                    for (int i = 0; i < entities.Length && uniqueMeshNames.Count < 100; i++)
                    {
                        try
                        {
                            object? rm = boundGet.Invoke(EntityManager, new object[] { entities[i] });
                            if (rm == null) continue;
                            scanned++;
                            object? meshObj = meshField.GetValue(rm);
                            if (meshObj is Mesh diagMesh && diagMesh != null)
                            {
                                string meshName = diagMesh.name ?? "(null)";
                                if (uniqueMeshNames.ContainsKey(meshName))
                                    uniqueMeshNames[meshName]++;
                                else
                                    uniqueMeshNames[meshName] = 1;
                            }
                        }
                        catch { errors++; if (errors > 10) break; }
                    }
                }

                DebugLog.Write("AssetSwap",
                    $"[DIAGNOSTIC] Vanilla mesh name survey: scanned {scanned}/{entities.Length} entities, " +
                    $"found {uniqueMeshNames.Count} unique mesh names (errors={errors}, genericGet={boundGet != null}):");
                foreach (var kvp in uniqueMeshNames.OrderByDescending(x => x.Value))
                {
                    DebugLog.Write("AssetSwap", $"  mesh=\"{kvp.Key}\"  count={kvp.Value}");
                }
            }

            // ---- SELECTIVE SWAP: match by mesh name ----
            // Determine which vanilla mesh names this bundle should target.
            string bundleFileName = Path.GetFileNameWithoutExtension(modBundlePath) ?? "";
            string[]? targetMeshSubstrings = null;
            foreach (var kvp in BundleToVanillaMeshMap)
            {
                if (bundleFileName.IndexOf(kvp.Key, StringComparison.OrdinalIgnoreCase) >= 0)
                {
                    targetMeshSubstrings = kvp.Value;
                    DebugLog.Write("AssetSwap",
                        $"TrySwapRenderMeshFromBundle: bundle '{bundleFileName}' matched mapping key '{kvp.Key}' " +
                        $"→ targeting mesh substrings: [{string.Join(", ", kvp.Value)}]");
                    break;
                }
            }

            // If no mapping exists for this bundle, proceed with NO mesh-name filter.
            // The vanillaMapping archetype component filter (EntityQuery) already scopes the
            // candidate set to the right unit type — an additional mesh-name filter is only
            // needed when one bundle must target a strict subset of that archetype.
            // Logging here is intentional: surfaces which bundles are running unfiltered so
            // an operator can later populate BundleToVanillaMeshMap for tighter targeting.
            bool meshFilterActive = targetMeshSubstrings != null && targetMeshSubstrings.Length > 0;
            if (!meshFilterActive)
            {
                DebugLog.Write("AssetSwap",
                    $"[SWAP] No BundleToVanillaMeshMap entry for bundle '{bundleFileName}'. " +
                    $"Proceeding with no mesh-name filter — will swap all {entities.Length} entities " +
                    $"matching vanillaMapping archetype. Add an entry to BundleToVanillaMeshMap for finer targeting.");
            }

            int swapCount = 0;

            // Iter-148 safety cap: bundle-to-archetype swaps can match tens of thousands of
            // entities (observed 25,713 RenderMesh entities matching a single bundle). Applying
            // the same replacement mesh+material to every match makes every unit look identical
            // — the "everything looks the same" disaster. Cap at 100 entities per swap call
            // until proper per-unit selective targeting lands. Logged once per bundle so the
            // operator knows the cap was hit and additional matches were skipped.
            const int MaxEntitiesPerSwap = 100;
            int swapBudget = Math.Min(entities.Length, MaxEntitiesPerSwap);
            if (entities.Length > MaxEntitiesPerSwap && _reportedFailures.Add($"cap:{assetName}"))
            {
                DebugLog.Write("AssetSwap",
                    $"TrySwapRenderMeshFromBundle: capping swap at {MaxEntitiesPerSwap}/{entities.Length} entities for asset='{assetName}' " +
                    $"(vanillaMapping='{vanillaMapping ?? "<null>"}') — prevents 'everything looks the same' disaster. " +
                    "Selective targeting is a separate concern; tracked as follow-up.");
            }

            int skippedNoMatch = 0;
            for (int i = 0; i < swapBudget; i++)
            {
                if (swapCount >= MaxSwapsPerBundle)
                {
                    DebugLog.Write("AssetSwap",
                        $"TrySwapRenderMeshFromBundle: hit MaxSwapsPerBundle cap ({MaxSwapsPerBundle}), stopping.");
                    break;
                }

                Entity entity = entities[i];
                try
                {
                    if (!em.HasComponent(entity, renderMeshComponentType))
                    {
                        skippedNoMatch++;
                        continue;
                    }

                    // Use non-generic overload to avoid "Ambiguous match found" on multi-mesh entities.
                    // Unity 2021.3 EntityManager exposes (Entity, int typeIndex) not (Entity, ComponentType).
                    // If the resolved overload takes int, pass TypeIndex; otherwise pass ComponentType directly.
                    object getSharedArg = (getSharedNonGeneric.GetParameters()[1].ParameterType == typeof(int))
                        ? (object)renderMeshComponentType.TypeIndex
                        : (object)renderMeshComponentType;
                    object? renderMesh = getSharedNonGeneric.Invoke(
                        em, new object[] { entity, getSharedArg });
                    if (renderMesh == null)
                    {
                        skippedNoMatch++;
                        DebugLog.Write("AssetSwap",
                            $"[AssetSwap] entity has RenderMesh component but GetSharedComponentData returned null for asset='{assetName}'");
                        continue;
                    }

                    // ---- Selective mesh-name check (only when BundleToVanillaMeshMap has an entry) ----
                    if (meshFilterActive && meshField != null)
                    {
                        object? currentMeshObj = meshField.GetValue(renderMesh);
                        if (currentMeshObj is Mesh currentMesh && currentMesh != null)
                        {
                            string currentName = currentMesh.name ?? "";
                            bool nameMatches = false;
                            for (int s = 0; s < targetMeshSubstrings!.Length; s++)
                            {
                                if (currentName.IndexOf(targetMeshSubstrings[s], StringComparison.OrdinalIgnoreCase) >= 0)
                                {
                                    nameMatches = true;
                                    break;
                                }
                            }
                            if (!nameMatches)
                            {
                                skippedNoMatch++;
                                continue;
                            }
                        }
                        else
                        {
                            // Mesh field is null — entity not yet loaded, skip.
                            skippedNoMatch++;
                            continue;
                        }
                    }

                    bool changed = false;
                    if (replacementMesh != null && meshField != null)
                    {
                        object? currentMeshObj = meshField.GetValue(renderMesh);
                        if (currentMeshObj is Mesh currentMesh
                            && !IsSkinnedMeshCompatible(currentMesh, replacementMesh, out string? skinReason))
                        {
                            if (_reportedFailures.Add($"skinning:{assetName}"))
                            {
                                DebugLog.Write("AssetSwap",
                                    $"TrySwapRenderMeshFromBundle: skipping entity {entity.Index} — {skinReason}");
                            }
                            continue;
                        }

                        meshField.SetValue(renderMesh, replacementMesh);
                        changed = true;
                    }
                    if (materialField != null)
                    {
                        object? currentMat = materialField.GetValue(renderMesh);
                        if (replacementMat != null)
                        {
                            if (currentMat == null)
                            {
                                // Vanilla entity has no current material — skip material SET but
                                // keep the mesh swap already applied. Option C: log so we can
                                // track which entities/bundles lack a compatible material slot.
                                if (_reportedFailures.Add($"nomat:{assetName}:{entity.Index}"))
                                {
                                    DebugLog.Write("AssetSwap",
                                        $"TrySwapRenderMeshFromBundle: entity {entity.Index} — " +
                                        $"no current material slot for '{assetName}', keeping vanilla material (mesh swapped)");
                                }
                            }
                            else
                            {
                                materialField.SetValue(renderMesh, replacementMat);
                                changed = true;
                            }
                        }
                        else if (replacementMesh != null && currentMat == null)
                        {
                            // No replacement material and no current material — log once per asset.
                            if (_reportedFailures.Add($"nomatsrc:{assetName}"))
                            {
                                DebugLog.Write("AssetSwap",
                                    $"TrySwapRenderMeshFromBundle: bundle '{assetName}' has no material; " +
                                    $"mesh swapped with vanilla material retained");
                            }
                        }
                    }

                    if (changed)
                    {
                        genericSet.Invoke(em, new object[] { entity, renderMesh });
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

            DebugLog.Write("AssetSwap",
                $"TrySwapRenderMeshFromBundle: swapped {swapCount}/{swapBudget} entities (total matching={entities.Length}, " +
                $"skipped {skippedNoMatch} non-matching meshes, cap={MaxEntitiesPerSwap})");
            entities.Dispose();
            query.Dispose();

            return swapCount > 0;
        }

        // ------------------------------------------------------------------ helpers

        /// <summary>
        /// #991 / #973: DINO infantry uses skinned <see cref="Mesh"/> assets (bindposes + bone weights)
        /// driven by procedural animation. Swapping a static mesh onto those entities freezes the pose.
        /// </summary>
        private static bool IsSkinnedMeshCompatible(Mesh current, Mesh replacement, out string? reason)
        {
            reason = null;
            int currentBindposes = current.bindposes?.Length ?? 0;
            int replacementBindposes = replacement.bindposes?.Length ?? 0;
            bool currentSkinned = currentBindposes > 0;
            bool replacementSkinned = replacementBindposes > 0;

            if (currentSkinned && !replacementSkinned)
            {
                reason =
                    $"vanilla mesh '{current.name}' has {currentBindposes} bindpose(s) but replacement " +
                    $"'{replacement.name}' is static (0 bindposes) — swap would freeze procedural animation (#973)";
                return false;
            }

            if (currentSkinned && replacementSkinned && currentBindposes != replacementBindposes)
            {
                reason =
                    $"bindpose count mismatch for '{replacement.name}': vanilla={currentBindposes} " +
                    $"replacement={replacementBindposes} — retarget to DINO reference skeleton before swap";
                return false;
            }

            return true;
        }

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
        /// Fallback bundle discovery: scans all deployed packs under
        /// <c>&lt;BepInEx&gt;/dinoforge_packs/</c> for a bundle file whose name matches
        /// <paramref name="assetAddress"/> (the standard pack layout is
        /// <c>&lt;packDir&gt;/assets/bundles/&lt;assetAddress&gt;</c>).
        /// Returns the full path of the first match, or <see cref="string.Empty"/> if none found.
        /// Called only when <c>ModBundlePath</c> is null/empty on the registered swap request.
        /// </summary>
        private static string FindBundleInPacksDir(string assetAddress)
        {
            if (string.IsNullOrEmpty(assetAddress))
                return string.Empty;

            try
            {
                string packsRoot = Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge_packs");
                if (!Directory.Exists(packsRoot))
                    return string.Empty;

                foreach (string packDir in Directory.GetDirectories(packsRoot))
                {
                    string candidate = Path.Combine(packDir, "assets", "bundles", assetAddress);
                    if (File.Exists(candidate))
                        return candidate;
                }
            }
            catch
            {
                // Best-effort: filesystem errors during scan are non-fatal.
            }

            return string.Empty;
        }

        /// <summary>
        /// Resolves a mod bundle path. Relative paths are joined against the BepInEx plugins dir.
        /// </summary>
        private static string ResolveModBundlePath(string path)
        {
            // Defensive: null/empty path would throw ArgumentNullException(path2) in Path.Combine
            // on Mono (netstandard2.0 does not enforce C# nullable annotations at runtime).
            if (string.IsNullOrEmpty(path))
                return string.Empty;
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
#if NET8_0
        public override void OnDestroy()
#else
        protected override void OnDestroy()
#endif
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
