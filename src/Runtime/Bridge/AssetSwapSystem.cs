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

        /// <summary>
        /// #992: Bundles (keyed by mod bundle path) that have PERMANENTLY failed to yield a usable
        /// swap — stub bundles (the 13 #986 90-byte stubs), bundles with no renderable mesh, or
        /// hard load errors. These can never succeed, yet AssetSwapRegistry keeps them pending and
        /// the swap retries them EVERY frame, re-loading the bundle each pass → the per-frame
        /// 'another AssetBundle with the same files is already loaded' flood. Once a bundle is in
        /// this set we skip it entirely (logged once, then silent), stopping the flood for all stub
        /// bundles while leaving working swaps (units/cims/buildings) untouched.
        /// </summary>
        private readonly HashSet<string> _permanentlyFailedBundles = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// Tracks bundles for which a non-URP/legacy material swap was already skipped.
        /// This prevents repeated logs while the same bundle is retried across frames.
        /// </summary>
        private static readonly HashSet<string> _reportedNonUrpMaterialBundles = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        private int _frameCount;

        /// <summary>Whether the one-shot vanilla mesh diagnostic dump has been emitted.</summary>
        private bool _vanillaMeshDumpDone;

        /// <summary>
        /// Maximum number of entities to swap per bundle invocation.
        /// Safety cap to prevent replacing the entire world with one mesh.
        /// </summary>
        private const int MaxSwapsPerBundle = 500;

        /// <summary>
        /// Declarative map from a pack <c>vanilla_mapping</c> value to the vanilla DINO mesh-name
        /// substrings it should replace. This is the real "bundle → vanilla mesh" mapping that exits
        /// DIAGNOSTIC MODE: the source of truth is the pack's <c>vanilla_mapping</c> field on every
        /// unit/building definition (carried through to <see cref="AssetSwapRequest.VanillaMapping"/>),
        /// NOT a hardcoded bundle-filename table.
        ///
        /// Targeting works in two layers:
        ///   1. PRIMARY (authoritative): the ECS entity query is already narrowed to the archetype
        ///      component resolved from <c>vanilla_mapping</c> (e.g. militia → Components.MeleeUnit)
        ///      via <see cref="PackStatMappings.TryResolveMapping"/>. This is what actually scopes
        ///      the swap to the correct vanilla units.
        ///   2. SECONDARY (optional refinement): the substrings below let the swap further filter by
        ///      the live mesh name when DINO's vanilla mesh names are known. When a mapping has no
        ///      substrings (or the value is empty) the swap relies on the archetype filter alone and
        ///      still proceeds (no longer DIAGNOSTIC MODE).
        ///
        /// Keyed by <c>vanilla_mapping</c> so it stays declarative and pack-driven: adding a new
        /// mapping value only requires a single entry here, and packs reference it by name.
        /// </summary>
        private static readonly Dictionary<string, string[]> VanillaMappingToMeshSubstrings =
            new Dictionary<string, string[]>(StringComparer.OrdinalIgnoreCase)
            {
                // Melee / infantry archetypes (mapped to Components.MeleeUnit).
                { "militia",         new[] { "Militia", "militia", "Peasant", "peasant" } },
                { "line_infantry",   new[] { "Swordsman", "swordsman", "Soldier", "soldier", "Infantry", "infantry" } },
                { "heavy_infantry",  new[] { "Heavy", "heavy", "Armored", "armored" } },
                { "elite",           new[] { "Elite", "elite", "Knight", "knight", "Veteran", "veteran" } },
                { "hero",            new[] { "Hero", "hero", "Champion", "champion", "Lord", "lord" } },
                { "wall_defender",   new[] { "Wall", "wall", "Guard", "guard", "Defender", "defender" } },
                { "support",         new[] { "Support", "support", "Healer", "healer", "Medic", "medic", "Priest", "priest" } },
                { "special",         new[] { "Special", "special", "Mage", "mage" } },

                // Ranged archetypes (mapped to Components.RangeUnit).
                { "ranged_infantry", new[] { "Archer", "archer", "Ranged", "ranged", "Crossbow", "crossbow", "Gun", "gun" } },
                { "skirmisher",      new[] { "Skirmisher", "skirmisher", "Slinger", "slinger", "Scout", "scout" } },
                { "scout",           new[] { "Scout", "scout", "Recon", "recon" } },

                // Mounted / mechanised archetypes.
                { "cavalry",         new[] { "Cavalry", "cavalry", "Rider", "rider", "Horse", "horse", "Mount", "mount" } },
                { "siege",           new[] { "Siege", "siege", "Catapult", "catapult", "Trebuchet", "trebuchet", "Ram", "ram", "Cannon", "cannon" } },

                // Aerial archetype (BUG B fix #101). Provides a mesh-name fallback so the swap
                // proceeds for sw-tri-fighter / sw-nantex-fighter even before AerialSpawnSystem
                // has tagged the entity with AerialUnitComponent. Covers DINO's airstrike/flyer
                // mesh names plus generic aerial tokens.
                { "aerial_fighter",  new[] { "Aerial", "aerial", "Air", "Flying", "flying", "Flyer", "flyer", "Bird", "bird", "Bomber", "bomber", "Airstrike", "airstrike", "Plane", "plane", "Fighter", "fighter" } },

                // #975 Phase 1 — CIMS (citizens/workers). DINO's roaming population renders with
                // bomj_* meshes (бомж = vagrant). When the Components.Worker archetype filter does
                // not resolve, these substrings let the swap proceed via the secondary mesh-name
                // filter so cims still exit DIAGNOSTIC MODE.
                { "cims",            new[] { "bomj", "Bomj", "cim", "Cim", "Citizen", "citizen", "Worker", "worker", "Peon", "peon", "Villager", "villager" } },
                { "worker",          new[] { "bomj", "Bomj", "Worker", "worker", "Peon", "peon", "Villager", "villager" } },
                { "citizen",         new[] { "bomj", "Bomj", "Citizen", "citizen", "cim", "Cim" } },

                // #975 Phase 1 — BUILDINGS. Vanilla DINO building mesh-name substrings, keyed by
                // building-type vanilla_mapping. The archetype filter (Components.BuildingBase) is
                // authoritative; these substrings are the optional secondary refinement that lets
                // distinct building meshes be swapped selectively. Empty/unknown DINO mesh names
                // fall back to archetype-only targeting.
                { "command",         new[] { "Castle", "castle", "Keep", "keep", "Command", "command", "TownHall", "townhall", "Hall", "hall" } },
                { "barracks",        new[] { "Barrack", "barrack", "Baraks", "baraks", "Stable", "stable", "Train", "train" } },
                { "resource",        new[] { "Farm", "farm", "Mine", "mine", "Mill", "mill", "Lumber", "lumber", "Quarry", "quarry", "Storage", "storage", "Warehouse", "warehouse" } },
                { "economy",         new[] { "Farm", "farm", "Mine", "mine", "Mill", "mill", "Market", "market", "House", "house", "Storage", "storage" } },
                { "defense",         new[] { "Tower", "tower", "Wall", "wall", "Gate", "gate", "Turret", "turret", "Fort", "fort" } },
                { "tower",           new[] { "Tower", "tower", "Turret", "turret", "Watchtower", "watchtower" } },
                { "wall",            new[] { "Wall", "wall", "Gate", "gate", "Palisade", "palisade", "Rampart", "rampart" } },
                { "research",        new[] { "Research", "research", "Lab", "lab", "Library", "library", "University", "university", "Academy", "academy" } },
                // Generic building archetype (#101 building fix). Pack structures default to the
                // "building" mapping when they omit vanilla_mapping; these substrings cover DINO's
                // common building mesh-name tokens so the swap can refine within the BuildingBase
                // archetype query (and so it never falls into DIAGNOSTIC MODE for lack of a signal).
                { "building",        new[] { "Building", "building", "House", "house", "Tower", "tower", "Barrack", "barrack", "Wall", "wall", "Gate", "gate", "Keep", "keep", "Castle", "castle", "Hall", "hall", "Factory", "factory", "Mill", "mill", "Mine", "mine", "Storage", "storage", "Depot", "depot", "Center", "center", "Centre", "centre", "Bay", "bay", "Lab", "lab", "Generator", "generator", "Foundry", "foundry", "Camp", "camp", "Tent", "tent", "Hut", "hut" } },
            };

        /// <summary>
        /// Component type name that aerial units (vanilla_mapping='aerial_fighter') are tagged
        /// with by DINOForge's AerialSpawnSystem. Used as the archetype filter for the visual
        /// mesh swap (BUG B fix #101). PackStatMappings deliberately maps aerial_fighter to null
        /// for STAT injection (AerialSpawnSystem owns behaviour), so the swap resolves the archetype
        /// here instead.
        /// </summary>
        /// <para>
        /// #986 (2026-05-31): the original value <c>DINOForge.Runtime.Aviation.AerialUnitComponent</c>
        /// resolved to 0 live entities (live entity dump, build 1BDC999C) — DINO never tags any
        /// entity with our custom aviation component, and DINO has no native flying units. The
        /// RenderMesh + AerialUnitComponent query therefore returned 0 → "0 succeeded" for every
        /// aerial bundle. DINO's renderable combat archetypes that carry RenderMesh directly are
        /// MeleeUnit/RangeUnit/CavalryUnit/SiegeUnit. We reskin SW aircraft onto
        /// <c>Components.SiegeUnit</c> (live count 162, RenderMesh on the same entity) — distinct
        /// from the melee/range archetypes used by infantry to reduce visual collision.
        /// </para>
        private const string AerialArchetypeTypeName = "Components.SiegeUnit";

        /// <summary>
        /// Vanilla DINO component every building entity carries. Used as the archetype filter for
        /// the generic <c>building</c> swap mapping (#101 building fix) so pack structures (e.g.
        /// sw-cis-command-center, sw-rep-vehicle-bay) reskin onto live building entities instead of
        /// falling into DIAGNOSTIC MODE and rendering as native royal/undead buildings. Resolved in
        /// the swap path only (NOT in PackStatMappings) so building stat injection is unaffected.
        /// </summary>
        private const string BuildingArchetypeTypeName = "Components.BuildingBase";

        /// <summary>
        /// Resolves a pack <c>vanilla_mapping</c> to the ECS component-type name the asset swap
        /// should narrow its EntityQuery to. Wraps <see cref="PackStatMappings.TryResolveMapping"/>
        /// but supplies the aerial archetype for <c>aerial_fighter</c> (which PackStatMappings
        /// intentionally leaves null for stat injection). Returns false only when the mapping is
        /// blank/unknown; a known mapping with a null component type still returns true with a null
        /// out value.
        /// </summary>
        private static bool TryResolveSwapArchetype(string? vanillaMapping, out string? archetypeTypeName)
        {
            if (!string.IsNullOrWhiteSpace(vanillaMapping)
                && string.Equals(vanillaMapping, "aerial_fighter", StringComparison.OrdinalIgnoreCase))
            {
                archetypeTypeName = AerialArchetypeTypeName;
                return true;
            }
            if (!string.IsNullOrWhiteSpace(vanillaMapping)
                && string.Equals(vanillaMapping, "building", StringComparison.OrdinalIgnoreCase))
            {
                archetypeTypeName = BuildingArchetypeTypeName;
                return true;
            }
            return PackStatMappings.TryResolveMapping(vanillaMapping, out archetypeTypeName);
        }

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

        /// <summary>
        /// #986: For parent archetypes that do not carry RenderMesh on themselves (DINO buildings:
        /// Components.BuildingBase has LinkedEntityGroup but RenderMesh lives on a child entity),
        /// query the archetype alone, then walk each match's LinkedEntityGroup buffer and collect
        /// the child entities that DO carry the RenderMesh shared component. Returns a NativeArray
        /// (caller owns disposal). Empty/uncreated array means no child meshes were found.
        /// </summary>
        private NativeArray<Entity> CollectRenderMeshChildren(
            EntityManager em, Type archetypeType, Type renderMeshType,
            string assetName, string? vanillaMapping)
        {
            ComponentType renderMeshCt = ComponentType.ReadOnly(renderMeshType);
            EntityQuery archetypeQuery = em.CreateEntityQuery(
                new EntityQueryDesc
                {
                    All = new[] { ComponentType.ReadOnly(archetypeType) },
                    Options = EntityQueryOptions.IncludePrefab,
                });
            NativeArray<Entity> parents = archetypeQuery.ToEntityArray(Allocator.Temp);

            var collected = new List<Entity>();
            var seen = new HashSet<int>();
            try
            {
                for (int i = 0; i < parents.Length; i++)
                {
                    Entity parent = parents[i];
                    // Walk LinkedEntityGroup (a DynamicBuffer<LinkedEntityGroup> of Entity-wrappers).
                    if (!em.HasComponent<LinkedEntityGroup>(parent))
                    {
                        // No child group — if the parent itself somehow has RenderMesh, take it.
                        if (em.HasComponent(parent, renderMeshCt) && seen.Add(parent.Index))
                            collected.Add(parent);
                        continue;
                    }

                    DynamicBuffer<LinkedEntityGroup> group = em.GetBuffer<LinkedEntityGroup>(parent);
                    for (int j = 0; j < group.Length; j++)
                    {
                        Entity child = group[j].Value;
                        if (child == parent) continue;
                        if (!em.Exists(child)) continue;
                        if (!em.HasComponent(child, renderMeshCt)) continue;
                        if (seen.Add(child.Index))
                            collected.Add(child);
                    }
                }
            }
            catch (Exception ex)
            {
                DebugLog.Write("AssetSwap",
                    $"CollectRenderMeshChildren: walk failed for archetype '{archetypeType.FullName}' " +
                    $"(asset='{assetName}', vanilla_mapping='{vanillaMapping ?? "<null>"}'): {ex.Message}");
            }
            finally
            {
                parents.Dispose();
                archetypeQuery.Dispose();
            }

            if (collected.Count == 0)
                return default;

            var result = new NativeArray<Entity>(collected.Count, Allocator.Temp);
            for (int i = 0; i < collected.Count; i++)
                result[i] = collected[i];
            return result;
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
        public override void OnCreate()
        {
            base.OnCreate();
            DebugLog.Write("AssetSwap", "AssetSwapSystem.OnCreate");
        }

        /// <inheritdoc/>
        public override void OnUpdate()
        {
            if (_resetPending)
            {
                _resetPending = false;
                _frameCount = 0;
                _reportedFailures.Clear();
                // #992: clear the negative-result cache on reset so a hot-reload that ships fixed
                // (non-stub) bundles gets a fresh chance to swap.
                _permanentlyFailedBundles.Clear();
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
            string modBundleFullPath = ResolveModBundlePath(request.ModBundlePath);
            if (!File.Exists(modBundleFullPath))
            {
                DebugLog.Write("AssetSwap", $"ApplySwap: mod bundle not found: {modBundleFullPath}");
                return false;
            }

            // #992 negative-result cache: a bundle that has permanently failed (stub / no usable
            // mesh / load error) must NOT be re-loaded and retried every frame — that re-load is
            // the source of the 'already loaded' flood. Skip silently (the permanent-failure was
            // logged once when first detected). Returning false keeps the entity-swap contract
            // unchanged for the registry; the bundle is never touched again.
            if (_permanentlyFailedBundles.Contains(modBundleFullPath))
            {
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
                modBundleFullPath, request.AssetName, request.VanillaMapping, bestEm);
            DebugLog.Write("AssetSwap", $"ApplySwap: entity swap result={entitySwapResult} for '{request.AssetAddress}'");
            if (!patchResult && !entitySwapResult)
            {
                DebugLog.Write("AssetSwap",
                    $"[AssetSwap] RESOLVE-FAIL {request.AssetAddress}: both patch and entity-swap failed; " +
                    $"diskPatched={patchResult}, entitySwap={entitySwapResult}, vanillaMapping='{request.VanillaMapping ?? "<null>"}'");
            }

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
            string requestBundleFileName = Path.GetFileName(modBundlePath);
            AssetBundle? bundle = LoadBundle(modBundlePath);
            if (bundle == null)
            {
                // #992: bundle failed to load and could not be recovered from the already-loaded
                // set — a hard load error that will recur every frame. Mark permanently failed so
                // we stop re-attempting LoadFromFile (the flood source). Log once.
                if (_permanentlyFailedBundles.Add(modBundlePath))
                {
                    DebugLog.Write("AssetSwap",
                        $"[AssetSwap] RESOLVE-FAIL {requestBundleFileName}: LoadBundle returned null for '{modBundlePath}'. " +
                        "failed to load and could not be recovered — marking permanently failed (#992).");
                }
                return false;
            }

            // #101 fix (Bug B): the mesh/material/prefab inside the bundle is named after the
            // source FBX/prefab (e.g. "sw-cis-commando-droid"), NOT after the bundle key / asset
            // address (e.g. "sw-bx-commando-droid"). Looking up by request.AssetName therefore
            // returns null for most bundles. Resolve the replacement robustly by loading ALL assets
            // of each type and taking the first viable one, instead of trusting the key string.
            (Mesh? replacementMesh, Material? replacementMat) = ResolveReplacementAssets(bundle, assetName);

            if (replacementMesh == null && replacementMat == null)
            {
                // #992: a bundle with no usable Mesh/Material (the 13 #986 90-byte stubs) can NEVER
                // yield a swap. Mark it permanently failed so ApplySwap skips it on every later
                // frame instead of re-loading it — this is the primary stop for the per-frame
                // 'already loaded' flood. Log once.
                _permanentlyFailedBundles.Add(modBundlePath);
                DebugLog.Write("AssetSwap",
                    $"[AssetSwap] RESOLVE-FAIL {requestBundleFileName}: no usable Mesh/Material/prefab found in bundle " +
                    $"'{requestBundleFileName}' (requested asset='{assetName}'). " +
                    "Bundle may be a stub or contain no renderable geometry. " +
                    "Marking permanently failed — will skip on subsequent frames (#992).");
                return false;
            }

            // GFX-mode Phase 2: when the High graphics tier is active, upgrade the swapped
            // material to URP/Lit with PBR slots (driven by pack PBR metadata when present).
            // Pure passthrough when tier is Vanilla or no upgrade is possible — never breaks the swap.
            if (replacementMat != null)
            {
                replacementMat = DINOForge.Runtime.Graphics.GraphicsMaterialUpgrader.Upgrade(replacementMat, assetName);
            }
            bool materialCompatible = replacementMat != null && IsUrpCompatibleMaterial(replacementMat, modBundlePath);

            Type? renderMeshType = ResolveRenderMeshType();
            if (renderMeshType == null)
            {
                DebugLog.Write("AssetSwap", $"[AssetSwap] RESOLVE-FAIL {requestBundleFileName}: Unity.Rendering.RenderMesh type not found");
                return false;
            }

            // #608 P2: HRV2 (RenderMeshUnmanaged / MaterialMeshInfo) uses blittable structs
            // with readonly mesh/material data — the legacy FieldInfo.SetValue path doesn't
            // apply. Bail out gracefully until HRV2 mesh-swap is implemented (separate task).
            if (IsHrv2Type(_renderMeshVariantName))
            {
                DebugLog.Write("AssetSwap", $"[AssetSwap] RESOLVE-FAIL {requestBundleFileName}: HRV2 mesh-swap not yet implemented (variant='{_renderMeshVariantName}') — falling back to no-op for entity swap. Bundle-disk patch (if successful) still applies.");
                return false;
            }

            // Resolve vanilla_mapping → ECS component type for targeted entity filtering.
            // When the mapping is absent or unrecognised we fall back to RenderMesh-only query,
            // which at minimum avoids modifying non-unit geometry in cases like buildings.
            ComponentType[] queryComponents;
            // #986: When the archetype carries RenderMesh on a CHILD entity (DINO buildings:
            // Components.BuildingBase has LinkedEntityGroup + Unity.Transforms.Child but NO
            // RenderMesh on the parent), the RenderMesh+archetype query returns 0. We capture
            // the resolved archetype type so we can re-query archetype-only and walk children.
            Type? resolvedArchetypeType = null;
            if (!string.IsNullOrWhiteSpace(vanillaMapping)
                && TryResolveSwapArchetype(vanillaMapping, out string? archetypeTypeName)
                && !string.IsNullOrEmpty(archetypeTypeName))
            {
                Type? archetypeType = ResolveTypeByName(archetypeTypeName!);
                if (archetypeType != null)
                {
                    resolvedArchetypeType = archetypeType;
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
                        $"[AssetSwap] RESOLVE-FAIL {requestBundleFileName}: archetype type '{archetypeTypeName}' not " +
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

            // #986: child-entity RenderMesh fallback. DINO buildings carry the archetype marker
            // (Components.BuildingBase) on a PARENT entity whose RenderMesh lives on a CHILD
            // entity referenced via LinkedEntityGroup. The RenderMesh+archetype query above
            // therefore returns 0. When that happens (and we resolved a parent archetype), query
            // archetype-only and collect the RenderMesh-bearing children. This is what makes
            // buildings (and any other parent-rendered DINO entity) swap instead of "0 succeeded".
            if (entities.Length == 0 && resolvedArchetypeType != null)
            {
                NativeArray<Entity> childMatches = CollectRenderMeshChildren(
                    em, resolvedArchetypeType, renderMeshType, assetName, vanillaMapping);
                if (childMatches.IsCreated && childMatches.Length > 0)
                {
                    entities.Dispose();
                    query.Dispose();
                    entities = childMatches;
                    // No EntityQuery to dispose for the child path; use a no-op query handle.
                    query = em.CreateEntityQuery(ComponentType.ReadOnly(renderMeshType));
                    DebugLog.Write("AssetSwap",
                        $"TrySwapRenderMeshFromBundle: parent archetype '{resolvedArchetypeType.FullName}' " +
                        $"carries no RenderMesh; resolved {entities.Length} RenderMesh child entities " +
                        $"via LinkedEntityGroup for asset='{assetName}'.");
                }
                else
                {
                    if (childMatches.IsCreated) childMatches.Dispose();
                    DebugLog.Write("AssetSwap",
                        $"[AssetSwap] RESOLVE-FAIL {requestBundleFileName}: archetype '{resolvedArchetypeType.FullName}' " +
                        $"resolved for vanilla_mapping='{vanillaMapping ?? "<null>"}' but no RenderMesh children were found.");
                }
            }

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
                        $"[AssetSwap] RESOLVE-FAIL {requestBundleFileName}: entity query returned 0 results for asset='{assetName}' " +
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
                        $"[AssetSwap] RESOLVE-FAIL {requestBundleFileName}: reflection lookup failed (#{_reflectionFailCount}) " +
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

            // BUG A fix (#101): DINO's Unity 2021.3 EntityManager exposes the non-generic
            // GetSharedComponentData in TWO overloads — (Entity, ComponentType) AND
            // (Entity, int typeIndex). The FirstOrDefault lookup above can bind the Int32
            // overload (it explicitly accepts either FullName). Invoking that overload with a
            // boxed ComponentType throws at runtime: "Object of type 'Unity.Entities.ComponentType'
            // cannot be converted to type 'System.Int32'" → result=False ("swapped 0/100" for
            // ground units like sw-cis-magna-guard). Detect which overload was bound and pass the
            // matching argument type: ComponentType.TypeIndex (an int) when the param wants Int32.
            bool getSharedWantsTypeIndex =
                getSharedNonGeneric.GetParameters()[1].ParameterType.FullName == "System.Int32";
            object renderMeshSharedArg = getSharedWantsTypeIndex
                ? (object)renderMeshComponentType.TypeIndex
                : renderMeshComponentType;

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

            // ---- SELECTIVE SWAP ----
            // The PRIMARY (authoritative) targeting was already applied above: the EntityQuery
            // was narrowed to the archetype component resolved from vanilla_mapping. The optional
            // SECONDARY refinement below further filters by live mesh name when DINO's vanilla
            // mesh names for this mapping are known (VanillaMappingToMeshSubstrings, keyed by the
            // pack's vanilla_mapping value — NOT by bundle filename).
            // preserve historical log key for lower-risk diagnostics
            string[]? targetMeshSubstrings = null;
            if (!string.IsNullOrWhiteSpace(vanillaMapping)
                && VanillaMappingToMeshSubstrings.TryGetValue(vanillaMapping!, out string[]? substrings)
                && substrings != null
                && substrings.Length > 0)
            {
                targetMeshSubstrings = substrings;
                DebugLog.Write("AssetSwap",
                    $"TrySwapRenderMeshFromBundle: bundle '{requestBundleFileName}' vanilla_mapping='{vanillaMapping}' " +
                    $"→ refining by mesh substrings: [{string.Join(", ", substrings)}]");
            }

            // DIAGNOSTIC MODE only when there is NO targeting signal at all: no archetype filter
            // (vanilla_mapping absent/unresolved) AND no mesh-name substrings. With either signal
            // present we proceed — the archetype-narrowed query is authoritative; substrings are an
            // optional refinement. A populated vanilla_mapping therefore exits DIAGNOSTIC MODE.
            // #986 FINAL FIX: use the ACTUAL query-narrowing result (resolvedArchetypeType),
            // NOT a separate re-resolution. The earlier code re-ran TryResolveSwapArchetype +
            // ResolveTypeByName here, which could (and did, live) disagree with the resolution
            // the EntityQuery actually used above — the query logged "filtering by
            // 'Components.MeleeUnit'" (resolvedArchetypeType != null, 193 entities matched) yet
            // this re-resolution returned false, leaving the hand-guessed mesh substrings active
            // as a reject filter that dropped all 193 matched entities ("swapped 0/100"). The
            // query already narrowed the entities to the right archetype, so trust that single
            // source of truth: if the query was archetype-narrowed, treat the filter as present.
            bool hasArchetypeFilter = resolvedArchetypeType != null;

            // CRITICAL ("units look native") fix: when an archetype filter IS present it is the
            // authoritative targeting — the EntityQuery is already narrowed to the right unit class.
            // The mesh-name substrings were hand-guessed (e.g. "Swordsman") and do NOT match DINO's
            // real mesh vocabulary (e.g. "swordsmen", "royal_sword_2", "bomj_*", "harpy_*",
            // "undead_*"), so applying them as a *reject* filter dropped 100% of entities
            // ("swapped 0/100 … skipped 100 non-matching meshes") and every unit stayed native.
            // Substrings are therefore ONLY used as a fallback targeting signal when there is no
            // archetype filter; with an archetype filter we swap every entity the query returned.
            if (hasArchetypeFilter)
            {
                targetMeshSubstrings = null;
            }

            if (!hasArchetypeFilter && (targetMeshSubstrings == null || targetMeshSubstrings.Length == 0))
            {
                DebugLog.Write("AssetSwap",
                    $"[DIAGNOSTIC MODE] No targeting signal for bundle '{requestBundleFileName}' " +
                    $"(vanilla_mapping='{vanillaMapping ?? "<null>"}' yielded neither an archetype filter " +
                    $"nor mesh-name substrings). Skipping entity swap for {entities.Length} entities. " +
                    $"Add a 'vanilla_mapping' to the pack visual_asset to exit DIAGNOSTIC MODE.");
                entities.Dispose();
                query.Dispose();
                return false;
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
                        continue;

                    // Use non-generic overload to avoid "Ambiguous match found" on multi-mesh entities.
                    // BUG A fix (#101): pass TypeIndex (int) when the bound overload is the
                    // (Entity, int) one — passing a ComponentType there throws the Mono
                    // "ComponentType cannot be converted to Int32" conversion error.
                    object? renderMesh = getSharedNonGeneric.Invoke(
                        em, new object[] { entity, renderMeshSharedArg });
                    if (renderMesh == null) continue;

                    // ---- Selective mesh-name check (SECONDARY refinement only) ----
                    // Skip when no substrings were resolved: the archetype-narrowed query is
                    // authoritative and we swap every entity it returned.
                    if (meshField != null && targetMeshSubstrings != null)
                    {
                        object? currentMeshObj = meshField.GetValue(renderMesh);
                        if (currentMeshObj is Mesh currentMesh && currentMesh != null)
                        {
                            string currentName = currentMesh.name ?? "";
                            bool nameMatches = false;
                            for (int s = 0; s < targetMeshSubstrings.Length; s++)
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
                    if (replacementMat != null && materialCompatible && materialField != null)
                    {
                        object? currentMat = materialField.GetValue(renderMesh);
                        if (currentMat == null)
                            continue;
                        materialField.SetValue(renderMesh, replacementMat);
                        changed = true;
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

        /// <summary>
        /// Returns true only when the material is safe for Hybrid Renderer V2/URP execution.
        /// Legacy built-in shaders such as "Standard"/"Legacy"/"Diffuse" are rejected.
        /// </summary>
        private static bool IsUrpCompatibleMaterial(Material material, string bundlePath)
        {
            if (material == null)
                return false;

            Shader? shader = material.shader;
            if (shader == null)
            {
                LogNonUrpMaterialSkip(bundlePath, "<null>");
                return false;
            }

            string shaderName = shader.name ?? string.Empty;
            if (shaderName.StartsWith("Universal Render Pipeline/", StringComparison.Ordinal))
                return true;

            if (HasSrpBatcherSupport(shader))
                return true;

            LogNonUrpMaterialSkip(bundlePath, shaderName);
            return false;
        }

        private static bool HasSrpBatcherSupport(Shader shader)
        {
            try
            {
                MethodInfo? findPassTagValue = typeof(Shader).GetMethod(
                    "FindPassTagValue",
                    BindingFlags.Public | BindingFlags.Instance,
                    null,
                    new[] { typeof(int), typeof(string) },
                    null);

                if (findPassTagValue == null)
                    return false;

                for (int pass = 0; pass < 32; pass++)
                {
                    object? tagValue = findPassTagValue.Invoke(shader, new object[] { pass, "SRPBatcher" });
                    try
                    {
                        if (tagValue != null && Convert.ToInt32(tagValue) != 0)
                            return true;
                    }
                    catch
                    {
                        // Some Unity versions may expose FindPassTagValue with a non-convertible return type.
                    }
                }
            }
            catch
            {
                // If FindPassTagValue probing fails, fallback to shader-name check.
            }

            return false;
        }

        private static void LogNonUrpMaterialSkip(string bundlePath, string shaderName)
        {
            string bundleName = Path.GetFileName(bundlePath);
            if (string.IsNullOrWhiteSpace(bundleName))
                bundleName = bundlePath;

            if (!_reportedNonUrpMaterialBundles.Add(bundleName))
                return;

            DebugLog.Write(
                "AssetSwap",
                $"[AssetSwap] skipped material swap for {bundleName}: shader {shaderName} not HRV2-compatible");
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
        /// Resolves a mod bundle path. Relative paths are joined against the BepInEx plugins dir.
        /// </summary>
        private static string ResolveModBundlePath(string path)
        {
            return Path.IsPathRooted(path)
                ? path
                : Path.Combine(BepInEx.Paths.PluginPath, path);
        }

        /// <summary>
        /// Robustly resolves a replacement (Mesh, Material) pair from a mod bundle without relying
        /// on the asset being named after the bundle key (#101 Bug B). Resolution order:
        ///   1. Try the requested name as a bare Mesh / Material (fast path for purpose-built bundles).
        ///   2. Load the first GameObject prefab and extract mesh+material from its renderer
        ///      (SkinnedMeshRenderer preferred, then MeshFilter/MeshRenderer) — the common case for
        ///      bundles built from prefabs, where the prefab is named after the FBX.
        ///   3. Fall back to the first bare Mesh / Material asset in the bundle.
        /// Mesh and material are always sourced from the same renderer where possible to avoid
        /// mesh/material mismatches.
        /// </summary>
        private static (Mesh? mesh, Material? material) ResolveReplacementAssets(AssetBundle bundle, string requestedName)
        {
            string[] allAssetNames;
            try
            {
                allAssetNames = bundle.GetAllAssetNames();
            }
            catch (Exception ex)
            {
                allAssetNames = Array.Empty<string>();
                DebugLog.Write("AssetSwap",
                    $"[AssetSwap] RESOLVE-FAIL {requestedName}: bundle.GetAllAssetNames() failed: {ex.Message}");
            }
            // 1. Fast path: exact-name bare assets.
            Mesh? mesh = bundle.LoadAsset<Mesh>(requestedName);
            Material? material = bundle.LoadAsset<Material>(requestedName);
            if (mesh != null || material != null)
            {
                DebugLog.Write("AssetSwap", $"ResolveReplacementAssets: resolved bare asset by name '{requestedName}'");
                return (mesh, material);
            }

            // 2. Prefab path: extract from the first GameObject's renderer.
            GameObject[] prefabs = bundle.LoadAllAssets<GameObject>();
            foreach (GameObject prefab in prefabs)
            {
                if (prefab == null) continue;

                SkinnedMeshRenderer? smr = prefab.GetComponentInChildren<SkinnedMeshRenderer>(true);
                if (smr != null && smr.sharedMesh != null)
                {
                    mesh = smr.sharedMesh;
                    if (smr.sharedMaterials != null && smr.sharedMaterials.Length > 0)
                        material = smr.sharedMaterials[0];
                }
                else
                {
                    MeshFilter? mf = prefab.GetComponentInChildren<MeshFilter>(true);
                    if (mf != null && mf.sharedMesh != null)
                        mesh = mf.sharedMesh;

                    MeshRenderer? mr = prefab.GetComponentInChildren<MeshRenderer>(true);
                    if (mr != null && mr.sharedMaterials != null && mr.sharedMaterials.Length > 0)
                        material = mr.sharedMaterials[0];
                }

                if (mesh != null || material != null)
                {
                    DebugLog.Write("AssetSwap",
                        $"ResolveReplacementAssets: extracted mesh='{mesh?.name ?? "<null>"}' " +
                        $"material='{material?.name ?? "<null>"}' from prefab '{prefab.name}'");
                    return (mesh, material);
                }
            }

            // 3. Fall back to the first bare Mesh / Material asset in the bundle.
            Mesh[] meshes = bundle.LoadAllAssets<Mesh>();
            if (meshes.Length > 0) mesh = meshes[0];

            Material[] materials = bundle.LoadAllAssets<Material>();
            if (materials.Length > 0) material = materials[0];

            if (mesh != null || material != null)
            {
                DebugLog.Write("AssetSwap",
                    $"ResolveReplacementAssets: fell back to first bare mesh='{mesh?.name ?? "<null>"}' " +
                    $"material='{material?.name ?? "<null>"}' in bundle");
            }
            else
            {
                DebugLog.Write("AssetSwap",
                    $"[AssetSwap] RESOLVE-FAIL {requestedName}: ResolveReplacementAssets returned null mesh/material. " +
                    $"bundle assets: [{string.Join(", ", allAssetNames)}]");
            }

            return (mesh, material);
        }

        /// <summary>
        /// Loads an AssetBundle from disk, caching the result (LRU with auto-eviction).
        /// </summary>
        private AssetBundle? LoadBundle(string path)
        {
            if (_permanentlyFailedBundles.Contains(path))
            {
                DebugLog.Write("AssetSwap",
                    $"[AssetSwap] RESOLVE-FAIL {Path.GetFileName(path)}: request skipped because this bundle is permanently failed.");
                return null;
            }

            AssetBundle? cached = _loadedBundles.Get(path);
            if (cached != null)
                return cached;

            string fullPath = ResolveModBundlePath(path);

            if (!File.Exists(fullPath))
            {
                DebugLog.Write("AssetSwap",
                    $"[AssetSwap] RESOLVE-FAIL {Path.GetFileName(path)}: LoadBundle file not found: {fullPath}");
                return null;
            }

            try
            {
                AssetBundle bundle = AssetBundle.LoadFromFile(fullPath);
                if (bundle != null)
                {
                    _loadedBundles.Set(path, bundle);
                    DebugLog.Write("AssetSwap", $"LoadBundle: loaded '{fullPath}'");
                    return bundle;
                }

                // #992 (null-return variant): when the LRU evicted+Unloaded a bundle but Unity's
                // Unload has not yet completed (Unload lag), a fresh LoadFromFile of the SAME file
                // returns NULL (not the documented 'already loaded' throw). With ~50 SW bundles and
                // a 10-slot LRU this silently fails ~48/50 swaps. Recover by reusing the handle
                // Unity still has loaded, keyed by bundle name. This is the load-bearing fix that
                // lets a freshly built bundle (e.g. the rigged sw-clone-trooper-republic) actually
                // swap instead of being marked permanently failed (#991).
                AssetBundle? stillLoaded = FindLoadedBundleByPath(fullPath);
                if (stillLoaded != null)
                {
                    _loadedBundles.Set(path, stillLoaded);
                    if (_reportedFailures.Add($"reuse-null:{path}"))
                        DebugLog.Write("AssetSwap",
                            $"LoadBundle: LoadFromFile returned null but Unity still has '{Path.GetFileName(fullPath)}' " +
                            "loaded — reusing existing handle (Unload-lag recovery, #992/#991).");
                    return stillLoaded;
                }

                DebugLog.Write("AssetSwap",
                    $"[AssetSwap] RESOLVE-FAIL {Path.GetFileName(path)}: LoadFromFile returned null and no recoverable loaded bundle existed for '{fullPath}'.");

                return null;
            }
            catch (Exception ex)
            {
                // #992 flood fix: AssetBundle.LoadFromFile THROWS (and Unity logs an error every
                // call) when "another AssetBundle with the same files is already loaded". This
                // happens when our LRU evicted+Unloaded a bundle but Unity still considers the
                // underlying file loaded (Unload lag / scene-transition skip), or when the same
                // physical bundle was reached via a different cache key. Recover by reusing the
                // already-loaded handle instead of re-loading — and re-cache it so subsequent
                // passes hit the cache and never call LoadFromFile again.
                if (ex.Message.IndexOf("already loaded", StringComparison.OrdinalIgnoreCase) >= 0)
                {
                    AssetBundle? existing = FindLoadedBundleByPath(fullPath);
                    if (existing != null)
                    {
                        _loadedBundles.Set(path, existing);
                        if (_reportedFailures.Add($"reuse:{path}"))
                            DebugLog.Write("AssetSwap", $"LoadBundle: reused already-loaded bundle for '{fullPath}' (recovered from 'already loaded')");
                        return existing;
                    }
                }
                DebugLog.Write("AssetSwap",
                    $"[AssetSwap] RESOLVE-FAIL {Path.GetFileName(path)}: LoadBundle failed '{fullPath}': {ex.Message}");
                return null;
            }
        }

        /// <summary>
        /// #992: Locates an AssetBundle Unity already has loaded that corresponds to
        /// <paramref name="fullPath"/>. Unity does not expose the source path on a loaded bundle,
        /// so match by bundle name (file name without extension) against
        /// <see cref="AssetBundle.GetAllLoadedAssetBundles"/>. Returns null if none match.
        /// </summary>
        private static AssetBundle? FindLoadedBundleByPath(string fullPath)
        {
            string wantName = Path.GetFileNameWithoutExtension(fullPath) ?? "";
            try
            {
                foreach (AssetBundle loaded in AssetBundle.GetAllLoadedAssetBundles())
                {
                    if (loaded == null) continue;
                    string loadedName = loaded.name ?? "";
                    if (string.Equals(loadedName, wantName, StringComparison.OrdinalIgnoreCase)
                        || (loadedName.Length > 0
                            && (loadedName.IndexOf(wantName, StringComparison.OrdinalIgnoreCase) >= 0
                                || wantName.IndexOf(loadedName, StringComparison.OrdinalIgnoreCase) >= 0)))
                    {
                        return loaded;
                    }
                }
            }
            catch (Exception ex)
            {
                DebugLog.Write("AssetSwap", $"FindLoadedBundleByPath: enumeration failed: {ex.Message}");
            }
            return null;
        }

        /// <inheritdoc/>
        public override void OnDestroy()
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
