#nullable enable
using System;
using System.Collections.Generic;
using UnityEngine;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Simple LRU cache for AssetBundles with configurable max size and auto-unload on eviction.
    /// Netstandard2.0 compatible (no external caching library dependency).
    /// </summary>
    public class AssetBundleCache : IDisposable
    {
        private readonly int _maxSize;
        private readonly Dictionary<string, CacheEntry> _cache;
        private readonly LinkedList<string> _lruOrder;
        private bool _disposed;

        /// <summary>
        /// Entry in the cache with bundle reference and load time.
        /// </summary>
        private class CacheEntry
        {
            public AssetBundle Bundle { get; set; }
            public DateTime LoadedAt { get; set; }
            public LinkedListNode<string>? LruNode { get; set; }

            public CacheEntry(AssetBundle bundle)
            {
                Bundle = bundle;
                LoadedAt = DateTime.UtcNow;
            }
        }

        /// <summary>
        /// Creates a new AssetBundle cache with the specified max size.
        /// </summary>
        /// <param name="maxSize">Maximum number of bundles to cache (default 10).</param>
        public AssetBundleCache(int maxSize = 10)
        {
            if (maxSize <= 0)
                throw new ArgumentException("maxSize must be > 0", nameof(maxSize));

            _maxSize = maxSize;
            _cache = new Dictionary<string, CacheEntry>(StringComparer.OrdinalIgnoreCase);
            _lruOrder = new LinkedList<string>();
        }

        /// <summary>Gets a bundle from the cache, or null if not found.</summary>
        public AssetBundle? Get(string path)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(path))
                return null;

            if (!_cache.TryGetValue(path, out var entry))
                return null;

            // Move to end of LRU list (most recently used)
            if (entry.LruNode != null)
            {
                _lruOrder.Remove(entry.LruNode);
                entry.LruNode = _lruOrder.AddLast(path);
            }

            return entry.Bundle;
        }

        /// <summary>Adds or updates a bundle in the cache.</summary>
        public void Set(string path, AssetBundle bundle)
        {
            ThrowIfDisposed();

            if (string.IsNullOrEmpty(path))
                throw new ArgumentNullException(nameof(path));
            if (bundle == null)
                throw new ArgumentNullException(nameof(bundle));

            if (_cache.TryGetValue(path, out var existing))
            {
                // Update existing entry
                existing.Bundle = bundle;
                existing.LoadedAt = DateTime.UtcNow;

                // Move to end of LRU list
                if (existing.LruNode != null)
                {
                    _lruOrder.Remove(existing.LruNode);
                    existing.LruNode = _lruOrder.AddLast(path);
                }
            }
            else
            {
                // Add new entry
                var entry = new CacheEntry(bundle);
                entry.LruNode = _lruOrder.AddLast(path);
                _cache[path] = entry;

                // Evict LRU if over capacity
                if (_cache.Count > _maxSize)
                {
                    EvictOne();
                }
            }
        }

        /// <summary>Removes a bundle from the cache and unloads it.</summary>
        public void Remove(string path)
        {
            ThrowIfDisposed();

            if (!_cache.TryGetValue(path, out var entry))
                return;

            if (entry.LruNode != null)
            {
                _lruOrder.Remove(entry.LruNode);
            }

            entry.Bundle?.Unload(unloadAllLoadedObjects: true);
            _cache.Remove(path);
        }

        /// <summary>Clears all bundles and unloads them.</summary>
        public void Clear()
        {
            ThrowIfDisposed();

            foreach (var entry in _cache.Values)
            {
                entry.Bundle?.Unload(unloadAllLoadedObjects: true);
            }

            _cache.Clear();
            _lruOrder.Clear();
        }

        /// <summary>Returns the number of bundles currently cached.</summary>
        public int Count
        {
            get
            {
                ThrowIfDisposed();
                return _cache.Count;
            }
        }

        /// <summary>Disposes the cache and unloads all bundles.</summary>
        public void Dispose()
        {
            if (_disposed)
                return;

            Clear();
            _disposed = true;
        }

        /// <summary>Evicts the least recently used bundle from the cache.</summary>
        private void EvictOne()
        {
            if (_lruOrder.Count == 0)
                return;

            var lruPath = _lruOrder.First.Value;
            if (_cache.TryGetValue(lruPath, out var entry))
            {
                entry.Bundle?.Unload(unloadAllLoadedObjects: true);
                _cache.Remove(lruPath);
            }

            _lruOrder.RemoveFirst();
        }

        private void ThrowIfDisposed()
        {
            if (_disposed)
                throw new ObjectDisposedException(GetType().Name);
        }
    }
}
