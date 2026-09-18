//! Oracles for the generic AI provider port and its per-provider contracts.
//!
//! Requirement text (docs/design/civ-ai-crate.md §2):
//!
//! - Covers FR-CIV-AI-001: "A generic `AiProvider` port with `generate` +
//!   `embed`, `Send + Sync`, async, `model_id`/`model_version` for provenance."
//!   Acceptance: "trait object usable behind `Arc`."
//! - Covers FR-CIV-AI-002: "`LocalSlmProvider` ... as the **default in-game**
//!   generator; in-process, no sidecar."
//! - Covers FR-CIV-AI-003: "`OllamaDevProvider` — dev-only, OpenAI-compatible
//!   ... **not** a shipping dependency." Acceptance: "Compiled behind a
//!   `dev`/feature flag; never selected in release config."
//! - Covers FR-CIV-AI-004: "`FirepassKimiProvider` — cloud heavy-reasoning
//!   **fallback only**." Acceptance: "Behind `CIVAI_ENABLE_CLOUD`; missing
//!   `KIMI_API_KEY` → loud unavailable at call site."
//! - Covers FR-CIV-AI-005: "`EmbedProvider` ... for `embed`; generation
//!   unsupported (loud error)." Acceptance: "`generate` returns `Unsupported`."
//!
//! Replaces placeholders (`crates/ai/tests/fr_fr_civ_ai_001.rs` … `_005.rs`)
//! whose entire body was
//! `assert_eq!(SCHEMA_VERSION, 0); let _ = AiConfig::default();` — identical
//! across all five files, and asserting nothing about any provider. The
//! `SCHEMA_VERSION` check pinned a constant that the crate's own doc comment
//! says is bumped on breaking changes, so those tests would also have failed on
//! the next such change without testing anything.
//!
//! The per-provider tests are feature-gated because the crate declares
//! `default = []`: `DummyAiProvider` is the only impl compiled unconditionally.
//! The gates are the requirement (FR-CIV-AI-003's acceptance criterion is
//! literally "compiled behind a feature flag"), so they are asserted rather than
//! worked around.

use std::sync::Arc;

use civ_ai::{
    AiError, AiProvider, Capabilities, DummyAiProvider, EmbedRequest, GenRequest,
};

/// Compile-time proof that the port is object-safe and shareable.
///
/// This is the substance of FR-CIV-AI-001's acceptance criterion: the pool
/// stores providers as `Arc<dyn AiProvider>` and dispatches from several
/// threads, so both bounds must hold for the trait object, not just for a
/// concrete impl.
fn assert_send_sync<T: Send + Sync>() {}

// ---------------------------------------------------------------------------
// FR-CIV-AI-001 — the generic port
// ---------------------------------------------------------------------------

/// Covers FR-CIV-AI-001.
#[test]
fn fr_civ_ai_001_port_is_object_safe_and_send_sync() {
    assert_send_sync::<Arc<dyn AiProvider>>();
    assert_send_sync::<Box<dyn AiProvider>>();

    // Usable behind Arc, which is how the worker pool stores it.
    let provider: Arc<dyn AiProvider> = Arc::new(DummyAiProvider);
    assert_eq!(provider.model_id(), "dummy");
}

/// Covers FR-CIV-AI-001.
///
/// Both port operations must be reachable through the trait object.
#[tokio::test]
async fn fr_civ_ai_001_both_operations_work_through_a_trait_object() {
    let provider: Arc<dyn AiProvider> = Arc::new(DummyAiProvider);

    let generated = provider
        .generate(&GenRequest::from_prompt("a legend about the first harvest"))
        .await
        .expect("dummy provider serves generate");
    assert!(
        !generated.text.is_empty(),
        "generate must return text through the trait object"
    );

    let embedded = provider
        .embed(&EmbedRequest {
            texts: vec!["one".into(), "two".into()],
            input_snapshot_hash: [0u8; 32],
        })
        .await
        .expect("dummy provider serves embed");
    assert_eq!(embedded.len(), 2, "embed must return one vector per input");
    assert_eq!(
        embedded[0].len(),
        embedded[1].len(),
        "all vectors in a batch must share a dimension"
    );
}

/// Covers FR-CIV-AI-001.
///
/// `model_id` and `model_version` exist so results carry provenance, and
/// `capabilities` lets the pool route without a failed round-trip. A provider
/// that only embeds must therefore advertise `generate: false` rather than
/// rely on the caller discovering it from an error.
#[test]
fn fr_civ_ai_001_identity_and_capabilities_are_declared() {
    let provider = DummyAiProvider;

    assert!(
        !provider.model_id().is_empty(),
        "provenance requires a non-empty model id"
    );
    assert!(
        !provider.model_version().is_empty(),
        "provenance requires a non-empty model version"
    );

    let caps: Capabilities = provider.capabilities();
    assert!(caps.generate && caps.embed, "dummy serves both operations");
    assert!(
        !caps.cloud,
        "the default/test provider must not claim to be a cloud service"
    );
}

/// Covers FR-CIV-AI-001.
///
/// A request must be a pure function of its inputs: the same prompt yields the
/// same output, so provider calls are reproducible under test and a cache key
/// derived from the prompt is meaningful.
#[tokio::test]
async fn fr_civ_ai_001_identical_requests_yield_identical_output() {
    let provider = DummyAiProvider;
    let req = GenRequest::from_prompt("the river changed course");

    let first = provider.generate(&req).await.expect("generate");
    let second = provider.generate(&req).await.expect("generate");
    assert_eq!(first, second, "identical requests must give identical output");

    let other = GenRequest::from_prompt("the river changed course twice");
    let different = provider.generate(&other).await.expect("generate");
    assert_ne!(
        first.text, different.text,
        "a different prompt must not collide with the first"
    );
    assert_ne!(
        first.output_hash, different.output_hash,
        "distinct outputs must carry distinct provenance hashes"
    );
    // The hash is over the text, so it must be stable for identical text.
    assert_eq!(
        first.output_hash, second.output_hash,
        "output_hash must be a pure function of the generated text"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-AI-002 — local in-process provider
// ---------------------------------------------------------------------------

/// Covers FR-CIV-AI-002.
///
/// The compatibility constructor defers loading ("in-process, no sidecar" is
/// what keeps this loadable at all). Before a GGUF is loaded the provider must
/// declare no generation capability and must fail *loudly* naming the missing
/// artifact, rather than silently returning empty prose.
#[cfg(feature = "local")]
#[tokio::test]
async fn fr_civ_ai_002_unloaded_local_provider_fails_loudly() {
    use civ_ai::providers::LocalSlmProvider;

    let provider = LocalSlmProvider::new("mistral-7b-instruct", "/nonexistent/model.gguf");

    assert!(
        !provider.capabilities().generate,
        "an unloaded local provider must not advertise generation"
    );
    assert!(
        !provider.capabilities().cloud,
        "the in-game default must be local, not cloud"
    );

    let err = provider
        .generate(&GenRequest::from_prompt("narrate the epoch"))
        .await
        .expect_err("an unloaded provider must not silently succeed");
    assert!(
        matches!(err, AiError::ModelMissing(_)),
        "missing artifact must be a loud ModelMissing, got {err:?}"
    );
    assert!(
        err.to_string().contains("model"),
        "the error must name the missing artifact: {err}"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-AI-003 — dev-only Ollama provider
// ---------------------------------------------------------------------------

/// Covers FR-CIV-AI-003.
///
/// The dev provider is HTTP-backed and must not be a shipping dependency. Its
/// contract when no Ollama server is running is a loud `Unavailable` naming the
/// endpoint, never a fabricated completion.
#[cfg(feature = "dev")]
#[tokio::test]
async fn fr_civ_ai_003_ollama_dev_is_loud_when_no_server_answers() {
    use civ_ai::providers::OllamaDevProvider;

    // A port nothing is listening on, so the request cannot succeed.
    let provider = OllamaDevProvider::new("http://127.0.0.1:1", "llama3.2");

    let caps = provider.capabilities();
    assert!(caps.generate, "the dev provider serves generate");
    assert!(
        !caps.embed,
        "the dev provider is generate-only and must say so"
    );
    assert!(caps.cloud, "Ollama is reached over HTTP, so it is not local");

    let err = provider
        .generate(&GenRequest::from_prompt("say hello"))
        .await
        .expect_err("no server is listening, so generate must fail");
    assert!(
        matches!(err, AiError::Unavailable(_)),
        "an unreachable dev server must surface as a loud Unavailable, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-AI-004 — cloud fallback
// ---------------------------------------------------------------------------

/// Covers FR-CIV-AI-004.
///
/// Acceptance: "missing `KIMI_API_KEY` → loud unavailable at call site". The
/// failure must happen at construction with a named cause, so a misconfigured
/// cloud fallback cannot masquerade as a working provider.
///
/// If the key *is* present in the environment the constructor legitimately
/// succeeds, and there is no failure path to assert; the test then documents
/// that rather than pretending otherwise.
#[cfg(feature = "cloud")]
#[test]
fn fr_civ_ai_004_cloud_provider_is_loud_without_a_key() {
    use civ_ai::providers::FirepassKimiProvider;

    let key = std::env::var("KIMI_API_KEY").unwrap_or_default();
    if !key.trim().is_empty() {
        eprintln!(
            "skipping the missing-key assertion: KIMI_API_KEY is set in this \
             environment, so from_env() may legitimately succeed"
        );
        return;
    }

    let err = FirepassKimiProvider::from_env()
        .err()
        .expect("from_env must fail when no cloud key is configured");
    assert!(
        matches!(err, AiError::Unavailable(_)),
        "a missing cloud key must be a loud Unavailable, got {err:?}"
    );
    assert!(
        err.to_string().contains("KIMI_API_KEY"),
        "the error must name the missing key so the cause is actionable: {err}"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-AI-005 — embed-only provider
// ---------------------------------------------------------------------------

/// Covers FR-CIV-AI-005.
///
/// Acceptance: "`generate` returns `Unsupported`". An embed-only provider must
/// refuse generation explicitly (and advertise `generate: false`) instead of
/// returning empty text that a caller might render as flavour.
#[cfg(feature = "embed")]
#[tokio::test]
async fn fr_civ_ai_005_embed_provider_refuses_generate() {
    use civ_ai::providers::EmbedProvider;

    let provider = EmbedProvider::new("all-MiniLM-L6-v2");

    assert!(
        !provider.capabilities().generate,
        "an embed-only provider must declare generate: false"
    );

    let err = provider
        .generate(&GenRequest::from_prompt("write a legend"))
        .await
        .expect_err("generate must be refused rather than faked");
    assert!(
        matches!(err, AiError::Unsupported(_)),
        "generation on an embed-only provider must be Unsupported, got {err:?}"
    );
    assert!(
        err.to_string().contains("unsupported"),
        "the error must say the operation is unsupported: {err}"
    );
}
