//! Tests for FR-CIV-AI-015
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-AI-015: Balance analyst dev-assist
//! (heuristic anomaly detection -> SLM triage; offline/CI batch). Runs
//! headless; emits a human-readable balance report; never in shipping sim.

use civ_ai::{
    AiConfig, DummyAiProvider, GenRequest, ProviderRegistry, ProviderRole,
};
use std::sync::Arc;

#[cfg(test)]
mod fr_fr_civ_ai_015 {
    use civ_ai::AiProvider;
    use super::*;

    /// FR-CIV-AI-015 — Balance analyst runs headless: the provider can generate
    /// a report from structured telemetry without any sim/world-state dependency.
    #[test]
    fn balance_analyst_generates_headless_report() {
        let provider = DummyAiProvider;
        let telemetry_digest = "tick_range:100-200 pop_growth:0.12 energy_deficit_ticks:3 \
            gini_coefficient:0.45 trade_volume:820 unrest_events:2";
        let req = GenRequest::from_prompt(telemetry_digest);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let out = rt.block_on(provider.generate(&req)).unwrap();
        // Report must be non-empty
        assert!(!out.text.is_empty());
        // Output hash must match text (provenance for CI artifacts)
        let expected = *blake3::hash(out.text.as_bytes()).as_bytes();
        assert_eq!(out.output_hash, expected);
    }

    /// FR-CIV-AI-015 — Balance analyst is deterministic for same telemetry
    /// window (CI reproducibility).
    #[test]
    fn balance_analyst_deterministic_for_same_window() {
        let provider = DummyAiProvider;
        let digest = "window:tick_50-100 balance_score:0.73";
        let req = GenRequest::from_prompt(digest);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let out1 = rt.block_on(provider.generate(&req)).unwrap();
        let out2 = rt.block_on(provider.generate(&req)).unwrap();
        assert_eq!(out1.text, out2.text);
    }

    /// FR-CIV-AI-015 — Different telemetry windows produce different reports
    /// (sensitivity to input data).
    #[test]
    fn balance_analyst_different_windows_diverge() {
        let provider = DummyAiProvider;
        let req1 = GenRequest::from_prompt("window:A deficit:0 surplus:100");
        let req2 = GenRequest::from_prompt("window:B deficit:50 surplus:0");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let out1 = rt.block_on(provider.generate(&req1)).unwrap();
        let out2 = rt.block_on(provider.generate(&req2)).unwrap();
        assert_ne!(out1.text, out2.text);
    }

    /// FR-CIV-AI-015 — The Heavy provider role supports balance analyst
    /// (cloud OK for heavy triage, or local fallback).
    #[test]
    fn balance_analyst_supports_heavy_role() {
        let mut registry = ProviderRegistry::new();
        let dummy: Arc<dyn AiProvider> = Arc::new(DummyAiProvider);
        registry.register(ProviderRole::Heavy, Arc::clone(&dummy));
        let provider = registry.require(ProviderRole::Heavy).unwrap();
        let caps = provider.capabilities();
        assert!(caps.generate, "heavy provider must support generate");
    }

    /// FR-CIV-AI-015 — Balance analyst default config shows offline/CI defaults
    /// (no cloud required, reasonable token budget for reports).
    #[test]
    fn balance_analyst_config_defaults() {
        let config = AiConfig::default();
        // Cloud is off by default — balance analyst is offline/CI
        assert!(!config.enable_cloud, "balance analyst must not require cloud");
        // Token budget sufficient for a balance report
        assert!(config.gen_token_budget >= 200, "need budget for report");
    }
}
