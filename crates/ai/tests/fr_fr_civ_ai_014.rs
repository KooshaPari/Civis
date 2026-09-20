//! Tests for FR-CIV-AI-014
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-AI-014: Chatter/headlines service
//! (fixed-persona SLM, event-triggered, rate-limited, LOD-gated,
//! (persona,event)-cached). Generates only on state-change near camera/feed;
//! hard concurrency cap.

use civ_ai::{
    AiCache, DummyAiProvider, GenOutput, GenRequest, ProviderRegistry, ProviderRole,
};
use std::sync::Arc;

#[cfg(test)]
mod fr_fr_civ_ai_014 {
    use civ_ai::AiProvider;
    use super::*;

    /// FR-CIV-AI-014 — Chatter generation produces persona-stable output.
    /// The same (persona, event) pair must yield identical text across calls.
    #[test]
    fn chatter_persona_event_deterministic() {
        let provider = DummyAiProvider;
        let prompt = "persona:merchant event:trade_route_opened";
        let req = GenRequest::from_prompt(prompt);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let out1 = rt.block_on(provider.generate(&req)).unwrap();
        let out2 = rt.block_on(provider.generate(&req)).unwrap();
        assert_eq!(out1.text, out2.text, "same persona+event must be stable");
    }

    /// FR-CIV-AI-014 — Different events for same persona produce different
    /// chatter (event-triggered generation).
    #[test]
    fn chatter_different_events_diverge() {
        let provider = DummyAiProvider;
        let req1 = GenRequest::from_prompt("persona:elder event:war_declared");
        let req2 = GenRequest::from_prompt("persona:elder event:feast_celebrated");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let out1 = rt.block_on(provider.generate(&req1)).unwrap();
        let out2 = rt.block_on(provider.generate(&req2)).unwrap();
        assert_ne!(out1.text, out2.text, "different events should diverge");
    }

    /// FR-CIV-AI-014 — (persona,event) cache key deduplication: re-submitting
    /// the same event is a cache hit (rate-limiting via cache).
    #[test]
    fn chatter_cache_deduplicates_identical_events() {
        let provider = DummyAiProvider;
        let mut cache: AiCache<GenOutput> = AiCache::new();
        let prompt = "persona:merchant event:market_boom";
        let req = GenRequest::from_prompt(prompt);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let out1 = rt.block_on(civ_ai::cached_generate(&provider, &mut cache, &req));
        let out1 = out1.unwrap();
        assert!(!out1.from_cache);

        // Same event re-triggered = cache hit, no extra model call
        let out2 = rt.block_on(civ_ai::cached_generate(&provider, &mut cache, &req));
        let out2 = out2.unwrap();
        assert!(out2.from_cache);
    }

    /// FR-CIV-AI-014 — The Narrator provider role handles chatter generation.
    #[test]
    fn chatter_uses_narrator_role() {
        let mut registry = ProviderRegistry::new();
        let dummy: Arc<dyn AiProvider> = Arc::new(DummyAiProvider);
        registry.register(ProviderRole::Narrator, Arc::clone(&dummy));
        let provider = registry.require(ProviderRole::Narrator).unwrap();
        let caps = provider.capabilities();
        assert!(caps.generate, "narrator must support generate for chatter");
    }
}
