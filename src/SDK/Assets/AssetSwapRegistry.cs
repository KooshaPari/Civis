#nullable enable
using System;
using System.Collections.Generic;

namespace DINOForge.SDK.Assets
{
    /// <summary>
    /// Describes a pending asset swap: which Addressables asset address to target,
    /// which mod bundle contains the replacement, and which asset name within that bundle.
    /// </summary>
    public sealed class AssetSwapRequest
    {
        /// <summary>
        /// Addressables asset address (from catalog.json) to override.
        /// e.g. "Assets/Prefabs/Units/Swordsman.prefab"
        /// </summary>
        public string AssetAddress { get; }

        /// <summary>
        /// Absolute (or BepInEx-plugins-relative) path to the mod bundle containing the replacement asset.
        /// </summary>
        public string ModBundlePath { get; }

        /// <summary>
        /// Asset name (or numeric PathID) within the mod bundle to use as the replacement.
        /// </summary>
        public string AssetName { get; }

        /// <summary>Whether this swap has already been applied in the current session.</summary>
        public bool Applied { get; set; }

        /// <summary>Number of times this swap has failed. After <see cref="AssetSwapRegistry.MaxRetries"/> failures it is permanently skipped.</summary>
        public int FailCount { get; set; }

        /// <summary>
        /// Pack <c>vanilla_mapping</c> value (e.g. "line_infantry", "ranged_infantry") that
        /// identifies which ECS archetype to target when swapping live RenderMesh components.
        /// Null means the swap targets all entities with RenderMesh (legacy / building swaps).
        /// </summary>
        public string? VanillaMapping { get; }

        /// <summary>
        /// Initializes a new <see cref="AssetSwapRequest"/>.
        /// </summary>
        /// <param name="assetAddress">Addressables key identifying the vanilla asset to replace.</param>
        /// <param name="modBundlePath">Path to the mod bundle that supplies the replacement.</param>
        /// <param name="assetName">Asset name or PathID within the mod bundle.</param>
        /// <param name="vanillaMapping">
        /// Optional pack <c>vanilla_mapping</c> used to narrow entity targeting during live mesh swap.
        /// When non-null, only entities with the matching ECS archetype component receive the swap.
        /// </param>
        public AssetSwapRequest(string assetAddress, string modBundlePath, string assetName,
            string? vanillaMapping = null)
        {
            AssetAddress = assetAddress ?? throw new ArgumentNullException(nameof(assetAddress));
            ModBundlePath = modBundlePath ?? throw new ArgumentNullException(nameof(modBundlePath));
            AssetName = assetName ?? throw new ArgumentNullException(nameof(assetName));
            VanillaMapping = vanillaMapping;
        }
    }

    /// <summary>
    /// Static, thread-safe registry of pending asset swap requests.
    /// Mod packs register swaps here during their manifest-load phase;
    /// <c>AssetSwapSystem</c> (Runtime layer) drains and applies them at runtime.
    /// </summary>
    public static class AssetSwapRegistry
    {
        /// <summary>
        /// Maximum number of retry attempts before a swap is permanently skipped.
        /// </summary>
        public const int MaxRetries = 3;

        private static readonly object _lock = new object();
        private static readonly Dictionary<string, AssetSwapRequest> _requests =
            new Dictionary<string, AssetSwapRequest>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// Registers an asset swap. If a swap for the same address already exists it is replaced.
        /// </summary>
        /// <param name="request">The swap request to register.</param>
        /// <exception cref="ArgumentNullException">Thrown when <paramref name="request"/> is null.</exception>
        public static void Register(AssetSwapRequest request)
        {
            if (request == null) throw new ArgumentNullException(nameof(request));

            lock (_lock)
            {
                _requests[request.AssetAddress] = request;
            }
        }

        /// <summary>
        /// Returns all registered swaps that have not yet been applied.
        /// </summary>
        /// <returns>Snapshot list of pending swap requests.</returns>
        public static IReadOnlyList<AssetSwapRequest> GetPending()
        {
            lock (_lock)
            {
                var pending = new List<AssetSwapRequest>(_requests.Count);
                foreach (AssetSwapRequest req in _requests.Values)
                {
                    if (!req.Applied && req.FailCount < MaxRetries)
                    {
                        pending.Add(req);
                    }
                }
                return pending;
            }
        }

        /// <summary>
        /// Increments the fail count for the swap at <paramref name="assetAddress"/>.
        /// After <see cref="MaxRetries"/> failures, <see cref="GetPending"/> will no longer return it.
        /// </summary>
        /// <param name="assetAddress">The Addressables key whose swap failed.</param>
        public static void MarkFailed(string assetAddress)
        {
            if (string.IsNullOrEmpty(assetAddress)) return;

            lock (_lock)
            {
                if (_requests.TryGetValue(assetAddress, out AssetSwapRequest? req))
                {
                    req.FailCount++;
                }
            }
        }

        /// <summary>
        /// Marks the swap for <paramref name="assetAddress"/> as applied so it is not re-processed.
        /// </summary>
        /// <param name="assetAddress">The Addressables key whose swap was applied.</param>
        public static void MarkApplied(string assetAddress)
        {
            if (string.IsNullOrEmpty(assetAddress)) return;

            lock (_lock)
            {
                if (_requests.TryGetValue(assetAddress, out AssetSwapRequest? req))
                {
                    req.Applied = true;
                }
            }
        }

        /// <summary>
        /// Returns the total number of registered swap requests (applied + pending).
        /// </summary>
        public static int Count
        {
            get
            {
                lock (_lock) { return _requests.Count; }
            }
        }

        /// <summary>
        /// Marks all registered swaps as not applied so they will be re-processed.
        /// Call this after a hot-reload that may have changed bundle files, so the
        /// next asset swap system update re-patches and re-swaps everything.
        /// </summary>
        public static void ResetApplied()
        {
            lock (_lock)
            {
                foreach (AssetSwapRequest req in _requests.Values)
                {
                    req.Applied = false;
                    req.FailCount = 0;
                }
            }
        }

        /// <summary>
        /// Clears all registered swaps. Intended for test teardown only.
        /// </summary>
        internal static void Clear()
        {
            lock (_lock) { _requests.Clear(); }
        }
    }
}
