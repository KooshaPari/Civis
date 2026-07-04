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
