// MIRROR OF src/SDK/Assets/AssetCdnCatalog.cs
// See src/SDK/Assets/AssetCdnCatalog.cs for canonical source.
// This file exists for netstandard2.0 BepInEx plugin compatibility (Pattern #233).
//
// When changing the SDK version, mirror the changes here.
// (In future: DLL reference instead of source duplication.)

using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace DINOForge.Runtime.Assets
{
    /// <summary>
    /// MIRROR — see SDK version for canonical docs.
    /// </summary>
    [ExcludeFromCodeCoverage]
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

        [ExcludeFromCodeCoverage]
        public sealed class CacheStats
        {
            public long TotalCacheBytes { get; set; }
            public long MaxCacheBytes { get; set; }
            public int CachedAssetCount { get; set; }
            public int TotalAssetCount { get; set; }
            public double EstimatedHitRate { get; set; } = 0.0;
            public TimeSpan OldestEntryAge { get; set; } = TimeSpan.Zero;

            public string Summary =>
                $"{CachedAssetCount}/{TotalAssetCount} assets cached ({(int)(100 * CachedAssetCount / (double)TotalAssetCount)}%), " +
                $"{TotalCacheBytes / 1024 / 1024} MB / {MaxCacheBytes / 1024 / 1024} MB " +
                $"({(int)(100 * TotalCacheBytes / (double)MaxCacheBytes)}%)";
        }

        public AssetCdnCatalog(
            string packId,
            string packVersion,
            string baseUrl,
            long maxCacheSizeBytes = 2L * 1024 * 1024 * 1024,
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

        public static AssetCdnCatalog? TryLoadFromFile(string manifestPath)
        {
            if (!File.Exists(manifestPath))
                return null;

            return null;
        }

        internal void RegisterAsset(string assetId, AssetDescriptor descriptor)
        {
            lock (_lockObj)
            {
                _assets[assetId] = descriptor ?? throw new ArgumentNullException(nameof(descriptor));
            }
        }

        public bool TryGetAssetUrl(string assetId, out string url, out string sha256, out long sizeBytes)
        {
            url = string.Empty;
            sha256 = string.Empty;
            sizeBytes = 0L;

            lock (_lockObj)
            {
                if (!_assets.TryGetValue(assetId, out var descriptor))
                    return false;

                url = _baseUrl + descriptor.Url;
                sha256 = descriptor.Sha256;
                sizeBytes = descriptor.SizeBytes;
                return true;
            }
        }

        public string GetLocalCachePath(string assetId)
        {
            string bepinexDir = Path.Combine(
                AppDomain.CurrentDomain.BaseDirectory,
                "..",
                "BepInEx");
            return Path.Combine(bepinexDir, "dinoforge_pack_cache", _packId, assetId);
        }

        public bool IsCached(string assetId)
        {
            string cachePath = GetLocalCachePath(assetId);
            return File.Exists(cachePath);
        }

        public int GetAssetCount()
        {
            lock (_lockObj)
            {
                return _assets.Count;
            }
        }

        public IReadOnlyList<string> GetAssetIds()
        {
            lock (_lockObj)
            {
                return _assets.Keys.ToList().AsReadOnly();
            }
        }

        public CacheStats GetCacheStats()
        {
            lock (_lockObj)
            {
                return new CacheStats
                {
                    TotalCacheBytes = 0L,
                    MaxCacheBytes = _maxCacheSizeBytes,
                    CachedAssetCount = 0,
                    TotalAssetCount = _assets.Count,
                    EstimatedHitRate = 0.0,
                    OldestEntryAge = TimeSpan.Zero,
                };
            }
        }

        public bool IsLruEnabled => _lruEnabled;
        public bool ShouldPrefetchOnEnable => _prefetchOnPackEnable;
        public string PackId => _packId;
        public string PackVersion => _packVersion;
        public string BaseUrl => _baseUrl;
    }
}
