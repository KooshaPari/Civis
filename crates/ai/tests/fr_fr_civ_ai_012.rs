//! Tests for FR-CIV-AI-012
//!
//!
//! This test file verifies FR FR-CIV-AI-012: Legends narration service
//! (epoch-digest -> SLM prose; digest-hash cached). One <=1.5B call per
//! epoch; unchanged epoch is a cache hit on reload.

use civ_ai::{
    AiCache, AiEvent, DummyAiProvider, GenOutput, GenRequest,
};

#[cfg(test)]
mod fr_fr_civ_ai_012 {
    use civ_ai::AiProvider;
    use super::*;

    /// FR-CIV-AI-012 — Legends narration generates prose from an epoch digest.
    /// The digest-hash is the cache key; an unchanged epoch yields a cache hit.
    #[test]
    fn legends_narration_generates_prose_from_digest() {
        let provider = DummyAiProvider;
        let digest = "Epoch 3: faction 7 achieved Bronze age, population 42.";
        let req = GenRequest::from_prompt(digest);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let out = rt.block_on(provider.generate(&req)).unwrap();
        // Output must be non-empty prose
        assert!(!out.text.is_empty());
        // Output hash must match the text (provenance)
        let expected_hash = *blake3::hash(out.text.as_bytes()).as_bytes();
        assert_eq!(out.output_hash, expected_hash);
    }

    /// FR-CIV-AI-012 — Same epoch digest yields a cache hit (no repeated call).
    #[test]
    fn unchanged_epoch_is_cache_hit() {
        let provider = DummyAiProvider;
        let mut cache: AiCache<GenOutput> = AiCache::new();
        let digest = "Epoch 7: golden age, population 120.";
        let req = GenRequest::from_prompt(digest);

        let rt = tokio::runtime::Runtime::new().unwrap();
        let out1 = rt.block_on(civ_ai::cached_generate(&provider, &mut cache, &req));
        let out1 = out1.unwrap();
        assert!(!out1.from_cache);
        assert_eq!(cache.len(), 1);

        // Second call with same digest = cache hit
        let out2 = rt.block_on(civ_ai::cached_generate(&provider, &mut cache, &req));
        let out2 = out2.unwrap();
        assert!(out2.from_cache);
        assert_eq!(out1.text, out2.text);
    }

    /// FR-CIV-AI-012 — Different epoch digests produce different cache entries.
    #[test]
    fn different_epochs_are_different_cache_entries() {
        let provider = DummyAiProvider;
        let mut cache: AiCache<GenOutput> = AiCache::new();
        let rt = tokio::runtime::Runtime::new().unwrap();

        let req1 = GenRequest::from_prompt("Epoch 1: dawn.");
        let req2 = GenRequest::from_prompt("Epoch 2: rise.");

        let out1 = rt.block_on(civ_ai::cached_generate(&provider, &mut cache, &req1));
        let out1 = out1.unwrap();
        let out2 = rt.block_on(civ_ai::cached_generate(&provider, &mut cache, &req2));
        let out2 = out2.unwrap();

        assert_eq!(cache.len(), 2);
        assert_ne!(out1.text, out2.text);
    }

    /// FR-CIV-AI-012 — AiEvent provenance records the epoch narration for replay.
    #[test]
    fn legends_event_records_provenance() {
        let event: AiEvent<String> = AiEvent {
            seed: 12345,
            prompt_hash: *blake3::hash(b"epoch-7-digest").as_bytes(),
            model_id: "dummy".into(),
            model_version: "0".into(),
            input_snapshot_hash: *blake3::hash(b"epoch-7-state").as_bytes(),
            output_hash: *blake3::hash(b"legendary prose").as_bytes(),
            output: "legendary prose".into(),
            tick: 7,
        };
        // Cache key is composite: prompt_hash || snapshot_hash || model_id || model_version
        let key = event.cache_key();
        assert!(key.len() > 64); // at least two 32-byte hashes
    }
}
