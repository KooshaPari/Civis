//! civ-ai — generic AI provider port generalizing `civ-research::LlmClient`.
//!
//! This crate extracts the provider / cache / event / worker-pool machinery
//! from `civ-research` into a **domain-agnostic** substrate (see
//! `docs/design/civ-ai-crate.md`). It knows providers, cache, pool, provenance;
//! it does **not** know about cultures, epochs, or tech cards. The five feature
//! services and `civ-research` are consumers (FR-CIV-AI-001).
//!
//! Per `CLAUDE.md` "Optionality and failure behavior", every required provider
//! that is missing fails **loud and named** (see [`preflight`] and
//! [`AiError::Unavailable`] / [`AiError::ModelMissing`]). Determinism is **not**
//! required (charter), but the blake3 cache is **mandatory** for cost/latency
//! (FR-CIV-AI-007, NFR-CIV-AI-003).
//!
//! ## What is working vs stubbed in P1 (S2.W3)
//! - **Working:** [`AiProvider`] trait, [`DummyAiProvider`], the blake3
//!   [`cache::AiCache`], [`provenance::AiEvent`] + [`ReplayMode`] reuse, the
//!   [`pool::AiWorkerPool`] skeleton, [`config::AiConfig`], loud [`preflight`],
//!   and the [`registry::ProviderRegistry`].
//! - **Wired (feature `cloud`):** [`providers::FirepassKimiProvider`] wraps the
//!   existing `civ-research::FirepassKimiClient`.
//! - **Stubbed (features `local` / `embed` / `dev`):** `LocalSlmProvider`,
//!   `EmbedProvider`, `OllamaDevProvider` advertise capabilities and return
//!   [`AiError::ModelMissing`] / [`AiError::Unavailable`] until full model
//!   loading lands in a later phase (see each module's `TODO`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cache;
pub mod config;
pub mod pool;
pub mod preflight;
pub mod provenance;
pub mod providers;
pub mod registry;

pub use cache::AiCache;
pub use config::AiConfig;
pub use pool::{AiPayload, AiResult, AiTask, AiWorkerPool, TaskId};
pub use provenance::{AiEvent, ReplayAdvanceOutcome, ReplayMode, ReplayRefusal};
pub use providers::DummyAiProvider;
pub use registry::{ProviderRegistry, ProviderRole};

use serde::{Deserialize, Serialize};

/// Schema version for `civ-ai`. Bumped on breaking changes.
pub const SCHEMA_VERSION: u32 = 0;

/// Generic AI provider port. Generalizes `civ-research::LlmClient`.
///
/// All impls are `Arc`-shared across the worker pool. A provider that only does
/// one operation declares the other [`AiError::Unsupported`] and advertises via
/// [`AiProvider::capabilities`] so the pool routes without a failed round-trip
/// (FR-CIV-AI-001).
///
/// Uses [`async_trait`](async_trait::async_trait) so the trait is
/// `dyn`-compatible and providers can be shared as `Arc<dyn AiProvider>` across
/// the worker pool / registry.
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    /// Free-form text generation for flavor (legends, chatter, headlines) and
    /// one-shot batch jobs. Returns [`AiError::Unsupported`] for embed-only
    /// providers.
    async fn generate(&self, req: &GenRequest) -> Result<GenOutput, AiError>;

    /// Embeddings for drift/speciation and log triage. Returns
    /// [`AiError::Unsupported`] for generate-only providers.
    async fn embed(&self, req: &EmbedRequest) -> Result<Vec<Vec<f32>>, AiError>;

    /// Stable provider model identifier — flows into [`AiEvent`] provenance +
    /// cache key.
    fn model_id(&self) -> &str;

    /// Provider model version — flows into [`AiEvent`] provenance + cache key.
    fn model_version(&self) -> &str;

    /// Declared capabilities so callers/pool route correctly.
    fn capabilities(&self) -> Capabilities;
}

/// Declared provider capabilities, consulted before dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    /// Provider can serve [`AiProvider::generate`].
    pub generate: bool,
    /// Provider can serve [`AiProvider::embed`].
    pub embed: bool,
    /// Provider talks to a remote/cloud service (opt-in only).
    pub cloud: bool,
}

/// Request for [`AiProvider::generate`]. Carries everything needed to form a
/// deterministic cache key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenRequest {
    /// Fully-rendered prompt (template + variables).
    pub prompt: String,
    /// Tight token budget: 60–600 typical.
    pub max_tokens: u32,
    /// Randomness welcome (determinism not required).
    pub temperature: f32,
    /// JSON schema, when structured output is required.
    pub json_schema: Option<String>,
    /// blake3 of the sim region observed → cache key + provenance.
    pub input_snapshot_hash: [u8; 32],
    /// Optional seed; recorded for provenance, not required for replay.
    pub seed: Option<u64>,
}

impl GenRequest {
    /// Build a minimal request from a prompt; hashes the prompt for the
    /// snapshot field when no sim region applies (tests, one-shot jobs).
    #[must_use]
    pub fn from_prompt(prompt: impl Into<String>) -> Self {
        let prompt = prompt.into();
        Self {
            input_snapshot_hash: *blake3::hash(prompt.as_bytes()).as_bytes(),
            prompt,
            max_tokens: 600,
            temperature: 0.7,
            json_schema: None,
            seed: None,
        }
    }

    /// blake3 of the rendered prompt — the `prompt_hash` for provenance/cache.
    #[must_use]
    pub fn prompt_hash(&self) -> [u8; 32] {
        *blake3::hash(self.prompt.as_bytes()).as_bytes()
    }
}

/// Output of [`AiProvider::generate`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenOutput {
    /// Generated text.
    pub text: String,
    /// blake3(text) for [`AiEvent::output_hash`].
    pub output_hash: [u8; 32],
    /// Whether this result was served from cache.
    pub from_cache: bool,
}

impl GenOutput {
    /// Build a fresh (non-cached) output, hashing `text` for provenance.
    #[must_use]
    pub fn fresh(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            output_hash: *blake3::hash(text.as_bytes()).as_bytes(),
            text,
            from_cache: false,
        }
    }
}

/// Request for [`AiProvider::embed`]. Batched by construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbedRequest {
    /// Texts to embed in one batch.
    pub texts: Vec<String>,
    /// blake3 of the sim region observed → cache key + provenance.
    pub input_snapshot_hash: [u8; 32],
}

/// Error taxonomy (generalizes `civ-research::LlmError`; loud and named).
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum AiError {
    /// Cloud key missing, server down — LOUD at the call site (FR-CIV-AI-004).
    #[error("provider unavailable: {0}")]
    Unavailable(String),
    /// Embed-only provider asked to generate, or vice versa (FR-CIV-AI-005).
    #[error("operation unsupported by provider {0}")]
    Unsupported(String),
    /// Provider was rate limited.
    #[error("rate limited")]
    RateLimited,
    /// Provider returned an uninterpretable response.
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    /// Required model artifact missing — surfaced by [`preflight`].
    #[error("model artifact missing: {0}")]
    ModelMissing(String),
}

/// Compute the composite cache key for a generate request against a provider.
///
/// Mirrors `civ-research::LlmEvent::cache_key` verbatim:
/// `prompt_hash ‖ input_snapshot_hash ‖ model_id ‖ model_version`
/// (FR-CIV-AI-007).
#[must_use]
pub fn gen_cache_key(provider: &dyn AiProvider, req: &GenRequest) -> Vec<u8> {
    provenance::compose_cache_key(
        &req.prompt_hash(),
        &req.input_snapshot_hash,
        provider.model_id(),
        provider.model_version(),
    )
}

/// Cache-wrapping generate: returns on hit, else calls the provider and stores.
///
/// Providers stay cache-agnostic; this wrapper owns the cache path
/// (FR-CIV-AI-007, NFR-CIV-AI-003).
pub async fn cached_generate(
    provider: &dyn AiProvider,
    cache: &mut AiCache<GenOutput>,
    req: &GenRequest,
) -> Result<GenOutput, AiError> {
    let key = gen_cache_key(provider, req);
    if let Some(hit) = cache.get(&key) {
        let mut out = hit.clone();
        out.from_cache = true;
        return Ok(out);
    }
    let out = provider.generate(req).await?;
    cache.insert(&key, out.clone());
    Ok(out)
}

#[cfg(test)]
mod tests {
    //! Unit tests for core types in `civ-ai`:
    //! - `GenRequest` builder + `prompt_hash` determinism (FR-CIV-AI-001, FR-CIV-AI-007)
    //! - `GenOutput::fresh` builder + output hash correctness (FR-CIV-AI-006)
    //! - `gen_cache_key` composite key determinism (FR-CIV-AI-007)
    //! - `AiError` display / Debug impls (FR-CIV-AI-004, FR-CIV-AI-005)
    //! - `Capabilities` defaults and field access

    use super::*;

    // ------------------------------------------------------------------
    // GenRequest builder + prompt_hash determinism
    // ------------------------------------------------------------------

    /// `from_prompt` populates sensible defaults and derives
    /// `input_snapshot_hash` from the prompt content (FR-CIV-AI-001).
    #[test]
    fn gen_request_from_prompt_defaults() {
        let req = GenRequest::from_prompt("test prompt");
        assert_eq!(req.prompt, "test prompt");
        assert_eq!(req.max_tokens, 600);
        assert!((req.temperature - 0.7).abs() < f32::EPSILON);
        assert!(req.json_schema.is_none());
        assert!(req.seed.is_none());
        // input_snapshot_hash is blake3 of the prompt bytes.
        let expected_hash = *blake3::hash(b"test prompt").as_bytes();
        assert_eq!(req.input_snapshot_hash, expected_hash);
    }

    /// `prompt_hash` is stable across repeated calls for the same text,
    /// and different for different texts (FR-CIV-AI-007).
    #[test]
    fn gen_request_prompt_hash_determinism() {
        let a = GenRequest::from_prompt("forge a blade");
        let b = GenRequest::from_prompt("forge a blade");
        assert_eq!(a.prompt_hash(), b.prompt_hash(), "same prompt → same hash");

        let c = GenRequest::from_prompt("forge a shield");
        assert_ne!(
            a.prompt_hash(),
            c.prompt_hash(),
            "different prompt → different hash"
        );
    }

    /// Round-trip: `prompt_hash` on a cloned request is deterministic.
    #[test]
    fn gen_request_prompt_hash_clone() {
        let req = GenRequest::from_prompt("chronicle of the iron age");
        let hash_a = req.prompt_hash();
        let cloned = req.clone();
        let hash_b = cloned.prompt_hash();
        assert_eq!(hash_a, hash_b);
        // Mutating the clone's prompt changes the hash.
        let mut changed = cloned;
        changed.prompt.push('.');
        assert_ne!(hash_a, changed.prompt_hash());
    }

    // ------------------------------------------------------------------
    // GenOutput builder + output_hash correctness
    // ------------------------------------------------------------------

    /// `GenOutput::fresh` creates a non-cached output whose `output_hash`
    /// is blake3 of the text (FR-CIV-AI-006).
    #[test]
    fn gen_output_fresh_builder() {
        let out = GenOutput::fresh("hello world");
        assert_eq!(out.text, "hello world");
        assert!(!out.from_cache);
        let expected_hash = *blake3::hash(b"hello world").as_bytes();
        assert_eq!(out.output_hash, expected_hash);
    }

    /// `GenOutput` PartialEq/Eq consistency.
    #[test]
    fn gen_output_eq_and_clone() {
        let a = GenOutput::fresh("same");
        let b = GenOutput::fresh("same");
        assert_eq!(a, b);
        let c = GenOutput::fresh("different");
        assert_ne!(a, c);
    }

    // ------------------------------------------------------------------
    // gen_cache_key determinism
    // ------------------------------------------------------------------

    /// The composite key `prompt_hash ‖ input_snapshot_hash ‖ model_id ‖
    /// model_version` is stable for identical inputs and changes when
    /// any component shifts (FR-CIV-AI-007).
    #[test]
    fn gen_cache_key_identical_inputs() {
        let provider = super::DummyAiProvider;
        let req = GenRequest::from_prompt("forge a frontier city");
        let key_a = gen_cache_key(&provider, &req);
        let key_b = gen_cache_key(&provider, &req);
        assert_eq!(key_a, key_b, "identical inputs → identical cache key");
    }

    #[test]
    fn gen_cache_key_changes_on_prompt_shift() {
        let provider = super::DummyAiProvider;
        let req_a = GenRequest::from_prompt("a");
        let req_b = GenRequest::from_prompt("b");
        assert_ne!(
            gen_cache_key(&provider, &req_a),
            gen_cache_key(&provider, &req_b),
        );
    }

    #[test]
    fn gen_cache_key_changes_on_snapshot_shift() {
        let provider = super::DummyAiProvider;
        let mut req = GenRequest::from_prompt("prompt");
        let key_original = gen_cache_key(&provider, &req);
        req.input_snapshot_hash = [0xFF; 32];
        assert_ne!(key_original, gen_cache_key(&provider, &req));
    }

    /// A custom provider with different model_id produces a different key.
    struct CustomModelProvider;
    #[async_trait::async_trait]
    impl AiProvider for CustomModelProvider {
        async fn generate(&self, _req: &GenRequest) -> Result<GenOutput, AiError> {
            Err(AiError::Unsupported("custom".into()))
        }
        async fn embed(&self, _req: &EmbedRequest) -> Result<Vec<Vec<f32>>, AiError> {
            Err(AiError::Unsupported("custom".into()))
        }
        fn model_id(&self) -> &str { "custom-model" }
        fn model_version(&self) -> &str { "1.0" }
        fn capabilities(&self) -> Capabilities {
            Capabilities { generate: true, embed: false, cloud: false }
        }
    }

    #[test]
    fn gen_cache_key_changes_on_model_shift() {
        let dummy = super::DummyAiProvider;
        let custom = CustomModelProvider;
        let req = GenRequest::from_prompt("same prompt");
        assert_ne!(
            gen_cache_key(&dummy, &req),
            gen_cache_key(&custom, &req),
            "different provider model_id/version → different key",
        );
    }

    // ------------------------------------------------------------------
    // AiError display + Debug (loud and named; FR-CIV-AI-004)
    // ------------------------------------------------------------------

    #[test]
    fn ai_error_unavailable_display() {
        let err = AiError::Unavailable("API key missing".into());
        let msg = format!("{err}");
        assert!(msg.contains("API key missing"), "msg = {msg:?}");
    }

    #[test]
    fn ai_error_unsupported_display() {
        let err = AiError::Unsupported("embed-only".into());
        let msg = format!("{err}");
        assert!(msg.contains("embed-only"), "msg = {msg:?}");
    }

    #[test]
    fn ai_error_rate_limited_display() {
        let err = AiError::RateLimited;
        assert_eq!(format!("{err}"), "rate limited");
    }

    #[test]
    fn ai_error_model_missing_display() {
        let err = AiError::ModelMissing("mistral-7b.gguf".into());
        let msg = format!("{err}");
        assert!(msg.contains("mistral-7b"), "msg = {msg:?}");
    }

    // ------------------------------------------------------------------
    // Capabilities
    // ------------------------------------------------------------------

    #[test]
    fn capabilities_defaults() {
        let caps = Capabilities { generate: true, embed: false, cloud: false };
        assert!(caps.generate);
        assert!(!caps.embed);
        assert!(!caps.cloud);
    }
}
