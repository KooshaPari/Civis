// MIRROR OF src/SDK/Assets/AssetCdnCache.cs
// See src/SDK/Assets/AssetCdnCache.cs for canonical source.
// This file exists for netstandard2.0 BepInEx plugin compatibility (Pattern #233).

using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Threading;
using System.Threading.Tasks;

namespace DINOForge.Runtime.Assets
{
    /// <summary>
    /// MIRROR — see SDK version for canonical docs.
    /// </summary>
    [ExcludeFromCodeCoverage]
    public sealed class AssetCdnCache : IDisposable
    {
        private readonly string _packId;
        private readonly string _cacheDir;
        private readonly long _maxCacheSizeBytes;
        private readonly object _lockObj = new object();
        private bool _disposed;

        private readonly Dictionary<string, CachedAssetMetadata> _metadata =
            new Dictionary<string, CachedAssetMetadata>(StringComparer.Ordinal);

        [ExcludeFromCodeCoverage]
        private sealed class CachedAssetMetadata
        {
            public string AssetId { get; set; } = string.Empty;
            public string Sha256 { get; set; } = string.Empty;
            public long SizeBytes { get; set; }
            public DateTime CachedAt { get; set; } = DateTime.UtcNow;
            public DateTime AccessedAt { get; set; } = DateTime.UtcNow;
        }

        [ExcludeFromCodeCoverage]
        public sealed class CacheStats
        {
            public long TotalBytes { get; set; }
            public long MaxBytes { get; set; }
            public int AssetCount { get; set; }
            public int UtilizationPercent =>
                MaxBytes > 0 ? (int)(100L * TotalBytes / MaxBytes) : 0;
            public TimeSpan OldestAssetAge { get; set; } = TimeSpan.Zero;

            public string Summary =>
                $"{AssetCount} assets, {TotalBytes / 1024 / 1024} MB / {MaxBytes / 1024 / 1024} MB ({UtilizationPercent}%)";
        }

        public AssetCdnCache(
            string packId,
            string cacheDir,
            long maxCacheSizeBytes = 2 * 1024 * 1024 * 1024)
        {
            if (string.IsNullOrWhiteSpace(packId))
                throw new ArgumentException("Pack ID cannot be empty.", nameof(packId));
            if (string.IsNullOrWhiteSpace(cacheDir))
                throw new ArgumentException("Cache directory cannot be empty.", nameof(cacheDir));

            _packId = packId;
            _cacheDir = Path.Combine(cacheDir, packId);
            _maxCacheSizeBytes = maxCacheSizeBytes;

            Directory.CreateDirectory(_cacheDir);
        }

        public Task<string> EnsureCachedAsync(
            string assetId,
            string cdnUrl,
            string expectedSha256,
            CancellationToken cancellationToken = default)
        {
            if (string.IsNullOrWhiteSpace(assetId))
                throw new ArgumentException("Asset ID cannot be empty.", nameof(assetId));
            if (string.IsNullOrWhiteSpace(cdnUrl))
                throw new ArgumentException("CDN URL cannot be empty.", nameof(cdnUrl));
            if (!IsValidSha256(expectedSha256))
                throw new ArgumentException("Invalid SHA256 format.", nameof(expectedSha256));

            ThrowIfDisposed();

            return EnsureCachedAsyncCore(assetId, cdnUrl, expectedSha256, cancellationToken);
        }

        private async Task<string> EnsureCachedAsyncCore(
            string assetId,
            string cdnUrl,
            string expectedSha256,
            CancellationToken cancellationToken)
        {
            string cachePath = Path.Combine(_cacheDir, assetId);

            lock (_lockObj)
            {
                if (File.Exists(cachePath))
                {
                    if (_metadata.TryGetValue(assetId, out var meta))
                    {
                        meta.AccessedAt = DateTime.UtcNow;
                    }

                    return cachePath;
                }
            }

            throw new FileNotFoundException(
                $"Asset '{assetId}' not cached and download not yet implemented. " +
                $"URL: {cdnUrl}");
        }

        public bool TryRemove(string assetId)
        {
            ThrowIfDisposed();

            lock (_lockObj)
            {
                string cachePath = Path.Combine(_cacheDir, assetId);

                if (!File.Exists(cachePath))
                    return false;

                try
                {
                    File.Delete(cachePath);
                    _metadata.Remove(assetId);
                    return true;
                }
                catch (Exception)
                {
                    return false;
                }
            }
        }

        public CacheStats GetStats()
        {
            ThrowIfDisposed();

            lock (_lockObj)
            {
                long totalBytes = 0L;
                TimeSpan oldestAge = TimeSpan.Zero;

                foreach (var file in Directory.GetFiles(_cacheDir))
                {
                    var info = new FileInfo(file);
                    totalBytes += info.Length;

                    var age = DateTime.UtcNow - info.LastAccessTime;
                    if (oldestAge == TimeSpan.Zero || age > oldestAge)
                        oldestAge = age;
                }

                return new CacheStats
                {
                    TotalBytes = totalBytes,
                    MaxBytes = _maxCacheSizeBytes,
                    AssetCount = _metadata.Count,
                    OldestAssetAge = oldestAge,
                };
            }
        }

        public Task ClearAsync()
        {
            ThrowIfDisposed();

            return Task.Run(() =>
            {
                lock (_lockObj)
                {
                    try
                    {
                        if (Directory.Exists(_cacheDir))
                        {
                            Directory.Delete(_cacheDir, recursive: true);
                        }
                        _metadata.Clear();
                    }
                    catch (Exception)
                    {
                    }
                }
            });
        }

        internal void EvictLruIfNeeded()
        {
            lock (_lockObj)
            {
                long totalBytes = _metadata.Values.Sum(m => m.SizeBytes);
                if (totalBytes < _maxCacheSizeBytes)
                    return;
            }
        }

        public void Dispose()
        {
            _disposed = true;
        }

        private void ThrowIfDisposed()
        {
            if (_disposed)
                throw new ObjectDisposedException(nameof(AssetCdnCache));
        }

        private static bool IsValidSha256(string sha256)
        {
            return !string.IsNullOrWhiteSpace(sha256) &&
                   sha256.Length == 64 &&
                   sha256.All(c => "0123456789abcdefABCDEF".Contains(c));
        }

        private static string ComputeSha256(string filePath)
        {
            using (var sha256 = SHA256.Create())
            using (var stream = File.OpenRead(filePath))
            {
                byte[] hash = sha256.ComputeHash(stream);
                return Convert.ToHexString(hash).ToLowerInvariant();
            }
        }
    }
}
