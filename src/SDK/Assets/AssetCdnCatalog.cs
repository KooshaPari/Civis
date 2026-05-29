// STUB IMPLEMENTATION — Full network layer pending v0.26.0
// This file defines the CDN asset registry and cache location resolution.
// See docs/architecture/asset-cdn-lazy-load.md for design rationale.

using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace DINOForge.SDK.Assets
{
    /// <summary>
    /// Manages the catalog of CDN-hosted assets for a single pack.
    /// Loads asset metadata from asset_cdn_manifest.yaml and provides
    /// URL/hash resolution for lazy-load downloads.
    ///
    /// Thread-safe for concurrent asset lookups.
    /// </summary>
    [ExcludeFromCodeCoverage] // Stub: network layer pending
    public sealed class AssetCdnCatalog
    {
        private readonly string _packId;
        private readonly string _packVersion;
        private readonly string _baseUrl;
        private readonly Dictionary<string, AssetDescriptor> _assets;
        private readonly long _maxCacheSizeBytes;
        private readonly bool _lruEnabled;
        private readonly bool _prefetchOnPackEnable;
        private readonly object _lockObj = new object();

        /// <summary>
        /// Asset descriptor from asset_cdn_manifest.yaml.
        /// </summary>
        [ExcludeFromCodeCoverage]
        internal sealed class AssetDescriptor
        {
            [JsonPropertyName("url")]
            public string Url { get; set; } = string.Empty;

            [JsonPropertyName("sha256")]
            public string Sha256 { get; set; } = string.Empty;

            [JsonPropertyName("size_bytes")]
            public long SizeBytes { get; set; }

            [JsonPropertyName("tags")]
            public List<string> Tags { get; set; } = new();

            [JsonPropertyName("depends_on")]
            public List<string> DependsOn { get; set; } = new();
        }

        /// <summary>
        /// Cache statistics snapshot.
        /// </summary>
        [ExcludeFromCodeCoverage]
        public sealed class CacheStats
        {
            /// <summary>Total size of cached assets in bytes.</summary>
            public long TotalCacheBytes { get; set; }

            /// <summary>Maximum allowed cache size in bytes.</summary>
            public long MaxCacheBytes { get; set; }

            /// <summary>Number of cached assets.</summary>
            public int CachedAssetCount { get; set; }

            /// <summary>Total number of known assets for this pack.</summary>
            public int TotalAssetCount { get; set; }

            /// <summary>Estimated cache hit rate (0–1), if available.</summary>
            public double EstimatedHitRate { get; set; } = 0.0;

            /// <summary>Age of oldest cached asset.</summary>
            public TimeSpan OldestEntryAge { get; set; } = TimeSpan.Zero;

            /// <summary>Human-readable cache summary.</summary>
            public string Summary =>
                $"{CachedAssetCount}/{TotalAssetCount} assets cached ({(int)(100 * CachedAssetCount / (double)TotalAssetCount)}%), " +
                $"{TotalCacheBytes / 1024 / 1024} MB / {MaxCacheBytes / 1024 / 1024} MB " +
                $"({(int)(100 * TotalCacheBytes / (double)MaxCacheBytes)}%)";
        }

        /// <summary>
        /// Initialize CDN catalog with pack configuration.
        /// </summary>
        /// <param name="packId">Pack identifier (e.g., "warfare-starwars").</param>
        /// <param name="packVersion">Pack version (e.g., "0.1.0").</param>
        /// <param name="baseUrl">CDN base URL for asset downloads.</param>
        /// <param name="maxCacheSizeBytes">Maximum cache size (default 2GB).</param>
        /// <param name="lruEnabled">Whether to apply LRU eviction when cache exceeds limit.</param>
        /// <param name="prefetchOnPackEnable">Whether to prefetch all assets when pack is enabled.</param>
        public AssetCdnCatalog(
            string packId,
            string packVersion,
            string baseUrl,
            long maxCacheSizeBytes = 2L * 1024 * 1024 * 1024,  // 2 GB default
            bool lruEnabled = true,
            bool prefetchOnPackEnable = false)
        {
            if (string.IsNullOrWhiteSpace(packId))
                throw new ArgumentException("Pack ID cannot be empty.", nameof(packId));
            if (string.IsNullOrWhiteSpace(baseUrl))
                throw new ArgumentException("Base URL cannot be empty.", nameof(baseUrl));

            _packId = packId;
            _packVersion = packVersion;
            _baseUrl = baseUrl.EndsWith("/") ? baseUrl : baseUrl + "/";
            _maxCacheSizeBytes = maxCacheSizeBytes;
            _lruEnabled = lruEnabled;
            _prefetchOnPackEnable = prefetchOnPackEnable;
            _assets = new Dictionary<string, AssetDescriptor>(StringComparer.Ordinal);
        }

        /// <summary>
        /// Load CDN manifest from a YAML file and initialize catalog.
        /// </summary>
        /// <param name="manifestPath">Path to asset_cdn_manifest.yaml.</param>
        /// <returns>Initialized catalog, or null if manifest not found.</returns>
        /// <remarks>
        /// TODO (v0.26.0): Parse YAML manifest file and populate _assets dictionary.
        /// Current stub: returns empty catalog. Requires YamlDotNet integration.
        /// </remarks>
        public static AssetCdnCatalog? TryLoadFromFile(string manifestPath)
        {
            if (!File.Exists(manifestPath))
                return null;

            // TODO: Parse YAML, extract pack_id, pack_version, asset_cdn config
            // For now, return null (stub behavior)
            return null;
        }

        /// <summary>
        /// Register an asset in the catalog.
        /// </summary>
        /// <remarks>Internal API; used by manifest loader.</remarks>
        internal void RegisterAsset(string assetId, AssetDescriptor descriptor)
        {
            lock (_lockObj)
            {
                _assets[assetId] = descriptor ?? throw new ArgumentNullException(nameof(descriptor));
            }
        }

        /// <summary>
        /// Resolve a CDN asset URL and expected SHA256 hash.
        /// </summary>
        /// <param name="assetId">Asset identifier (e.g., "sw-rep-clone-trooper").</param>
        /// <param name="url">Full CDN URL where asset can be downloaded.</param>
        /// <param name="sha256">Expected SHA256 hash of the downloaded asset.</param>
        /// <param name="sizeBytes">Expected size in bytes.</param>
        /// <returns>True if asset found in catalog; false if unknown.</returns>
        public bool TryGetAssetUrl(string assetId, out string url, out string sha256, out long sizeBytes)
        {
            url = string.Empty;
            sha256 = string.Empty;
            sizeBytes = 0L;

            lock (_lockObj)
            {
                if (!_assets.TryGetValue(assetId, out var descriptor))
                    return false;

                // TODO (v0.26.0): Validate URL structure (https://, no path traversal)
                url = _baseUrl + descriptor.Url;
                sha256 = descriptor.Sha256;
                sizeBytes = descriptor.SizeBytes;
                return true;
            }
        }

        /// <summary>
        /// Get the local cache path where an asset should be stored.
        /// </summary>
        /// <remarks>
        /// Cache layout: BepInEx/dinoforge_pack_cache/{pack_id}/{asset_id}
        /// Returns path even if asset not yet cached (for checking existence).
        /// </remarks>
        public string GetLocalCachePath(string assetId)
        {
            // TODO (v0.26.0): Resolve BepInEx directory from RuntimeContext
            // For now, return a stub path
            string bepinexDir = Path.Combine(
                AppDomain.CurrentDomain.BaseDirectory,
                "..",
                "BepInEx");
            return Path.Combine(bepinexDir, "dinoforge_pack_cache", _packId, assetId);
        }

        /// <summary>
        /// Check if an asset is already cached locally.
        /// </summary>
        public bool IsCached(string assetId)
        {
            string cachePath = GetLocalCachePath(assetId);
            return File.Exists(cachePath);
        }

        /// <summary>
        /// Get the total number of assets declared for this pack.
        /// </summary>
        public int GetAssetCount()
        {
            lock (_lockObj)
            {
                return _assets.Count;
            }
        }

        /// <summary>
        /// Get all asset IDs in the catalog.
        /// </summary>
        public IReadOnlyList<string> GetAssetIds()
        {
            lock (_lockObj)
            {
                return _assets.Keys.ToList().AsReadOnly();
            }
        }

        /// <summary>
        /// Get current cache statistics.
        /// </summary>
        /// <remarks>
        /// TODO (v0.26.0): Scan dinoforge_pack_cache directory and SQLite assets.db
        /// to calculate actual cache size, hit rate, and LRU metadata.
        /// Current stub: returns placeholder values.
        /// </remarks>
        public CacheStats GetCacheStats()
        {
            lock (_lockObj)
            {
                return new CacheStats
                {
                    TotalCacheBytes = 0L,  // TODO: scan filesystem
                    MaxCacheBytes = _maxCacheSizeBytes,
                    CachedAssetCount = 0,  // TODO: count from assets.db
                    TotalAssetCount = _assets.Count,
                    EstimatedHitRate = 0.0,  // TODO: calculate from access logs
                    OldestEntryAge = TimeSpan.Zero,  // TODO: from cache metadata
                };
            }
        }

        /// <summary>
        /// Whether LRU eviction is enabled for this catalog's cache.
        /// </summary>
        public bool IsLruEnabled => _lruEnabled;

        /// <summary>
        /// Whether all assets should be prefetched when the pack is enabled.
        /// </summary>
        public bool ShouldPrefetchOnEnable => _prefetchOnPackEnable;

        /// <summary>
        /// Pack identifier for this catalog.
        /// </summary>
        public string PackId => _packId;

        /// <summary>
        /// Pack version for this catalog.
        /// </summary>
        public string PackVersion => _packVersion;

        /// <summary>
        /// CDN base URL for resolving asset download URLs.
        /// </summary>
        public string BaseUrl => _baseUrl;
    }
}
