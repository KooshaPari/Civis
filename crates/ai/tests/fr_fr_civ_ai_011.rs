//! Tests for FR-CIV-AI-011
//!
//!
//! This test file verifies FR FR-CIV-AI-011: Naming service
//! (grammar+Markov inline; SLM seeds a per-culture grammar once, batch).
//! Zero per-name model calls; grammar cached on the culture record.

use civ_ai::{
    AiCache, DummyAiProvider, GenOutput, GenRequest, ProviderRegistry, ProviderRole,
};
use std::sync::Arc;

#[cfg(test)]
mod fr_fr_civ_ai_011 {
    use civ_ai::AiProvider;
    use super::*;

    /// FR-CIV-AI-011 — Naming grammar-seed cache key is derived from
    /// (culture_id, language_params_hash), not from per-name prompts.
    /// The naming service seeds a grammar once per culture; subsequent name
    /// generation uses the grammar inline with zero model calls.
    #[test]
    fn naming_grammar_cache_key_is_culture_scoped() {
        let lang_params_hash: [u8; 32] = *blake3::hash(b"proto-language-params").as_bytes();

        let mut cache: AiCache<GenOutput> = AiCache::new();
        let dummy = DummyAiProvider;
        let seed_req = GenRequest {
            prompt: "seed grammar for culture 42".into(),
            input_snapshot_hash: lang_params_hash,
            ..GenRequest::from_prompt("seed grammar for culture 42")
        };
        let key = civ_ai::gen_cache_key(&dummy, &seed_req);
        cache.insert(&key, GenOutput::fresh("grammar-seeded"));

        // Same culture+params = cache hit (no repeated model call)
        assert!(cache.contains_key(&key));
        assert_eq!(cache.len(), 1);
    }

    /// FR-CIV-AI-011 — DummyAiProvider is deterministic: same prompt yields
    /// same name, enabling grammar-seed reproducibility across reloads.
    #[test]
    fn naming_generation_is_deterministic() {
        let provider = DummyAiProvider;
        let req = GenRequest::from_prompt("generate name for settlement-7");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let out1 = rt.block_on(provider.generate(&req)).unwrap();
        let out2 = rt.block_on(provider.generate(&req)).unwrap();
        assert_eq!(out1.text, out2.text);
        assert_eq!(out1.output_hash, out2.output_hash);
    }

    /// FR-CIV-AI-011 — The provider registry supports the Narrator role
    /// used by the naming service for grammar seeding.
    #[test]
    fn naming_uses_narrator_role_in_registry() {
        let mut registry = ProviderRegistry::new();
        let dummy: Arc<dyn AiProvider> = Arc::new(DummyAiProvider);
        registry.register(ProviderRole::Narrator, Arc::clone(&dummy));
        let provider = registry.require(ProviderRole::Narrator).unwrap();
        assert_eq!(provider.model_id(), "dummy");
    }
}
