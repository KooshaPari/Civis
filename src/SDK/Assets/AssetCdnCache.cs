// STUB IMPLEMENTATION — Download + HTTP client layer pending v0.26.0
// This file manages local asset caching, LRU eviction, and SHA256 verification.
// See docs/architecture/asset-cdn-lazy-load.md for design rationale.

using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace DINOForge.SDK.Assets
{
    /// <summary>
    /// Manages local asset cache directory for a single pack.
    /// Handles:
    ///   - LRU eviction when cache exceeds max size
    ///   - SHA256 verification on read/write
    ///   - Atomic file operations (partial downloads not cached)
    ///   - Thread-safe concurrent access
    ///
    /// Cache directory: BepInEx/dinoforge_pack_cache/{pack_id}/
    /// </summary>
    [ExcludeFromCodeCoverage] // Stub: network layer (HttpClient download) pending
    public sealed class AssetCdnCache : IDisposable
    {
        private readonly string _packId;
        private readonly string _cacheDir;
        private readonly long _maxCacheSizeBytes;
        private readonly object _lockObj = new object();
        private bool _disposed;

        // TODO (v0.26.0): Replace with SQLite assets.db for persistent cache metadata
        private readonly Dictionary<string, CachedAssetMetadata> _metadata =
            new Dictionary<string, CachedAssetMetadata>(StringComparer.Ordinal);

        /// <summary>
        /// Metadata for a single cached asset (in-memory representation).
        /// TODO: Move to SQLite assets.db
        /// </summary>
        [ExcludeFromCodeCoverage]
        private sealed class CachedAssetMetadata
        {
            public string AssetId { get; set; } = string.Empty;
            public string Sha256 { get; set; } = string.Empty;
            public long SizeBytes { get; set; }
            public DateTime CachedAt { get; set; } = DateTime.UtcNow;
            public DateTime AccessedAt { get; set; } = DateTime.UtcNow;
        }

        /// <summary>
        /// Cache statistics snapshot.
        /// </summary>
        [ExcludeFromCodeCoverage]
        public sealed class CacheStats
        {
            /// <summary>Total size of cached assets.</summary>
            public long TotalBytes { get; set; }

            /// <summary>Maximum allowed cache size.</summary>
            public long MaxBytes { get; set; }

            /// <summary>Number of cached assets.</summary>
            public int AssetCount { get; set; }

            /// <summary>Percentage of cache utilization.</summary>
            public int UtilizationPercent =>
                MaxBytes > 0 ? (int)(100L * TotalBytes / MaxBytes) : 0;

            /// <summary>Age of oldest cached asset.</summary>
            public TimeSpan OldestAssetAge { get; set; } = TimeSpan.Zero;

            /// <summary>Human-readable summary.</summary>
            public string Summary =>
                $"{AssetCount} assets, {TotalBytes / 1024 / 1024} MB / {MaxBytes / 1024 / 1024} MB ({UtilizationPercent}%)";
        }

        /// <summary>
        /// Initialize cache manager for a pack.
        /// </summary>
        /// <param name="packId">Pack identifier.</param>
        /// <param name="cacheDir">Root cache directory (e.g., BepInEx/dinoforge_pack_cache).</param>
        /// <param name="maxCacheSizeBytes">Maximum cache size (default 2GB).</param>
        public AssetCdnCache(
            string packId,
            string cacheDir,
            long maxCacheSizeBytes = 2L * 1024 * 1024 * 1024)
        {
            if (string.IsNullOrWhiteSpace(packId))
                throw new ArgumentException("Pack ID cannot be empty.", nameof(packId));
            if (string.IsNullOrWhiteSpace(cacheDir))
                throw new ArgumentException("Cache directory cannot be empty.", nameof(cacheDir));

            _packId = packId;
            _cacheDir = Path.Combine(cacheDir, packId);
            _maxCacheSizeBytes = maxCacheSizeBytes;

            // Ensure cache directory exists
            Directory.CreateDirectory(_cacheDir);

            // TODO (v0.26.0): Load existing cache metadata from assets.db
            // For now, start fresh
        }

        /// <summary>
        /// Ensure an asset is cached locally, downloading if necessary.
        /// </summary>
        /// <param name="assetId">Asset identifier.</param>
        /// <param name="cdnUrl">Full CDN URL for download.</param>
        /// <param name="expectedSha256">Expected SHA256 hash.</param>
        /// <param name="cancellationToken">Cancellation token.</param>
        /// <returns>Path to cached asset file.</returns>
        /// <exception cref="ArgumentException">Thrown if SHA256 is invalid format.</exception>
        /// <exception cref="OperationCanceledException">Thrown if cancelled during download.</exception>
        /// <remarks>
        /// TODO (v0.26.0): Implement actual HTTP download using HttpClient.
        /// Current stub: checks local cache, raises if missing.
        ///
        /// Download flow:
        ///   1. Check if already cached → return path
        ///   2. Download to temp file with timeout (30s default)
        ///   3. Verify SHA256 matches → delete if mismatch
        ///   4. Atomically move to cache
        ///   5. Update metadata + access time
        ///   6. Check cache size → evict LRU if exceeds max
        /// </remarks>
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
                // Check if already cached
                if (File.Exists(cachePath))
                {
                    // TODO (v0.26.0): Verify SHA256 on disk matches expected
                    // For now, trust the file exists

                    // Update access time for LRU tracking
                    if (_metadata.TryGetValue(assetId, out var meta))
                    {
                        meta.AccessedAt = DateTime.UtcNow;
                    }

                    return cachePath;
                }
            }

            // TODO (v0.26.0): Download from cdnUrl using HttpClient
            // For now, stub: throw if not cached
            throw new FileNotFoundException(
                $"Asset '{assetId}' not cached and download not yet implemented. " +
                $"URL: {cdnUrl}");
        }

        /// <summary>
        /// Manually remove an asset from cache.
        /// </summary>
        /// <returns>True if asset was removed; false if not found.</returns>
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

        /// <summary>
        /// Get current cache statistics.
        /// </summary>
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

        /// <summary>
        /// Clear all cached assets for this pack.
        /// </summary>
        /// <remarks>
        /// TODO (v0.26.0): Delete cache directory recursively using safe Windows API.
        /// </remarks>
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
                        // Stub: swallow errors during cleanup
                    }
                }
            });
        }

        /// <summary>
        /// Apply LRU eviction when cache exceeds max size.
        /// </summary>
        /// <remarks>
        /// TODO (v0.26.0): Implement full eviction logic.
        ///
        /// Algorithm:
        ///   1. If total cache < max size: return (no eviction needed)
        ///   2. Calculate target size = max size * 90%
        ///   3. Sort cached assets by AccessedAt (oldest first)
        ///   4. Delete oldest assets until total < target
        ///   5. Update assets.db to reflect deletions
        /// </remarks>
        internal void EvictLruIfNeeded()
        {
            lock (_lockObj)
            {
                long totalBytes = _metadata.Values.Sum(m => m.SizeBytes);
                if (totalBytes < _maxCacheSizeBytes)
                    return;

                // TODO: Implement eviction
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

        /// <summary>
        /// Verify SHA256 hash of a file.
        /// </summary>
        /// <remarks>
        /// TODO (v0.26.0): Integrate into download + cache write flow.
        /// </remarks>
        private static string ComputeSha256(string filePath)
        {
            using (var sha256 = SHA256.Create())
            using (var stream = File.OpenRead(filePath))
            {
                byte[] hash = sha256.ComputeHash(stream);
                return BitConverter.ToString(hash).Replace("-", string.Empty).ToLowerInvariant();
            }
        }
    }
}
