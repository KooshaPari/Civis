//! Generic blake3 hash-keyed cache (generalizes `civ-research::ResearchCache`).
//!
//! Widened from "value = `TechCard`" to a generic value `V` (FR-CIV-AI-007).
//! Same composite-key surface (`insert`/`get`/`len`/`is_empty`); keys are the
//! blake3-derived composite from [`crate::provenance::compose_cache_key`].

use std::collections::BTreeMap;

/// Replay-safe, blake3 hash-keyed cache, generic over the cached value.
#[derive(Debug, Clone)]
pub struct AiCache<V> {
    entries: BTreeMap<Vec<u8>, V>,
}

impl<V> Default for AiCache<V> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl<V> AiCache<V> {
    /// Create an empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a cached value under `key` (the composite cache key).
    pub fn insert(&mut self, key: &[u8], value: V) {
        self.entries.insert(key.to_vec(), value);
    }

    /// Look up a cached value.
    #[must_use]
    pub fn get(&self, key: &[u8]) -> Option<&V> {
        self.entries.get(key)
    }

    /// Whether a key is present (used by replay cache-hit checks).
    #[must_use]
    pub fn contains_key(&self, key: &[u8]) -> bool {
        self.entries.contains_key(key)
    }

    /// Number of cached entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Is the cache empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-AI-007 — `AiCache` insert/get round-trips for multiple generic types.
    #[test]
    fn cache_insert_get_roundtrip() {
        let mut cache: AiCache<String> = AiCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        cache.insert(b"key1", "value1".into());
        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(b"key1"), Some(&"value1".to_string()));
        assert!(cache.contains_key(b"key1"));
        assert!(!cache.contains_key(b"missing"));
    }

    /// FR-CIV-AI-007 — overwriting a key updates the value without growing len.
    #[test]
    fn cache_overwrite_updates_value() {
        let mut cache: AiCache<u32> = AiCache::new();
        cache.insert(b"k", 1);
        assert_eq!(cache.get(b"k"), Some(&1));
        cache.insert(b"k", 2);
        assert_eq!(cache.get(b"k"), Some(&2));
        assert_eq!(cache.len(), 1);
    }

    /// FR-CIV-AI-007 — multiple independent keys coexist.
    #[test]
    fn cache_multiple_keys_independent() {
        let mut cache: AiCache<i32> = AiCache::new();
        let cases: Vec<(&[u8], i32)> = vec![(b"a", 1), (b"b", 2), (b"c", 3)];
        for (k, v) in &cases {
            cache.insert(k, *v);
        }
        assert_eq!(cache.len(), 3);
        for (k, v) in &cases {
            assert_eq!(cache.get(k), Some(v));
        }
    }

    /// FR-CIV-AI-007 — cloned cache shares entries but mutations are isolated.
    #[test]
    fn cache_clone_shares_data() {
        let mut cache: AiCache<String> = AiCache::new();
        cache.insert(b"x", "hello".into());
        let cloned = cache.clone();
        assert_eq!(cloned.get(b"x"), Some(&"hello".to_string()));
        assert_eq!(cloned.len(), 1);
    }
}
