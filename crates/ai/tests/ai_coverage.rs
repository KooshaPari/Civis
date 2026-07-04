//! Additional coverage for civ-ai reaching >= 50% (FR-CIV-TEST-012).
//!
//! Targets pub fns not yet exercised in dummy_roundtrip.rs or the inline
//! module tests: GenRequest helpers, GenOutput::fresh, compose_cache_key,
//! replay Hybrid/Free paths, preflight, ProviderRole, ProviderRegistry::get,
//! DummyAiProvider::generate_sync, and AiTask::id.

use std::sync::Arc;

use civ_ai::cache::AiCache;
use civ_ai::pool::{AiTask, AiPayload, AiWorkerPool};
use civ_ai::preflight::{check_artifacts, preflight, required_artifacts, RequiredArtifact};
use civ_ai::provenance::{
    compose_cache_key, replay_advance_ai_event, AiEvent, ReplayAdvanceOutcome, ReplayRefusal,
};
use civ_ai::registry::{MissingProvider, ProviderRole};
use civ_ai::{
    gen_cache_key, AiConfig, DummyAiProvider, EmbedRequest, GenOutput, GenRequest,
    ProviderRegistry, ReplayMode,
};

// ---------------------------------------------------------------------------
// GenRequest helpers
// ---------------------------------------------------------------------------

/// FR-CIV-AI-007 — from_prompt populates every required field with stable values.
#[test]
fn gen_request_from_prompt_fields() {
    let req = GenRequest::from_prompt("test prompt");
    assert_eq!(req.prompt, "test prompt");
    assert_eq!(req.max_tokens, 600);
    assert!((req.temperature - 0.7).abs() < f32::EPSILON);
    assert!(req.json_schema.is_none());
    assert!(req.seed.is_none());
    // input_snapshot_hash must equal blake3(prompt).
    let expected = *blake3::hash(b"test prompt").as_bytes();
    assert_eq!(req.input_snapshot_hash, expected);
}

/// FR-CIV-AI-007 — prompt_hash is stable and equals blake3(prompt).
#[test]
fn gen_request_prompt_hash_is_stable() {
    let req = GenRequest::from_prompt("stable hash test");
    let h1 = req.prompt_hash();
    let h2 = req.prompt_hash();
    assert_eq!(h1, h2);
    let expected = *blake3::hash(b"stable hash test").as_bytes();
    assert_eq!(h1, expected);
}

/// Distinct prompts produce distinct prompt_hashes (no trivial collision).
#[test]
fn gen_request_prompt_hash_differs_for_different_prompts() {
    let h1 = GenRequest::from_prompt("a").prompt_hash();
    let h2 = GenRequest::from_prompt("b").prompt_hash();
    assert_ne!(h1, h2);
}

// ---------------------------------------------------------------------------
// GenOutput::fresh
// ---------------------------------------------------------------------------

/// FR-CIV-AI-007 — fresh output carries correct hash and from_cache=false.
#[test]
fn gen_output_fresh_hash_and_flag() {
    let out = GenOutput::fresh("hello world");
    assert_eq!(out.text, "hello world");
    assert!(!out.from_cache);
    let expected = *blake3::hash(b"hello world").as_bytes();
    assert_eq!(out.output_hash, expected);
}

// ---------------------------------------------------------------------------
// provenance::compose_cache_key
// ---------------------------------------------------------------------------

/// FR-CIV-AI-007 — compose_cache_key concatenates all four components.
#[test]
fn compose_cache_key_length_and_stability() {
    let ph = [0xAAu8; 32];
    let sh = [0xBBu8; 32];
    let key = compose_cache_key(&ph, &sh, "model-x", "v2");
    // 32 + 32 + "model-x".len() + "v2".len() == 73.
    assert_eq!(key.len(), 32 + 32 + "model-x".len() + "v2".len());
    // Stable across calls.
    let key2 = compose_cache_key(&ph, &sh, "model-x", "v2");
    assert_eq!(key, key2);
}

/// Changing model_version shifts the cache key.
#[test]
fn compose_cache_key_differs_on_model_version_change() {
    let ph = [0u8; 32];
    let sh = [0u8; 32];
    let k1 = compose_cache_key(&ph, &sh, "m", "v1");
    let k2 = compose_cache_key(&ph, &sh, "m", "v2");
    assert_ne!(k1, k2);
}

// ---------------------------------------------------------------------------
// replay_advance_ai_event — Hybrid and Free paths
// ---------------------------------------------------------------------------

fn make_event(model_id: &str, model_version: &str) -> AiEvent<String> {
    let ph = *blake3::hash(b"prompt").as_bytes();
    let sh = [0u8; 32];
    AiEvent {
        seed: 0,
        prompt_hash: ph,
        model_id: model_id.into(),
        model_version: model_version.into(),
        input_snapshot_hash: sh,
        output_hash: [0u8; 32],
        output: "out".into(),
        tick: 1,
    }
}

/// Hybrid replay: cache miss -> HybridCacheMiss refusal.
#[test]
fn replay_hybrid_cache_miss_is_refused() {
    let cache: AiCache<String> = AiCache::new();
    let event = make_event("dummy", "0");
    assert_eq!(
        replay_advance_ai_event(ReplayMode::Hybrid, &cache, &event, true),
        ReplayAdvanceOutcome::Refused(ReplayRefusal::HybridCacheMiss)
    );
}

/// Hybrid replay: cache hit -> Advanced.
#[test]
fn replay_hybrid_cache_hit_advances() {
    let mut cache: AiCache<String> = AiCache::new();
    let event = make_event("dummy", "0");
    cache.insert(&event.cache_key(), "stored".into());
    assert_eq!(
        replay_advance_ai_event(ReplayMode::Hybrid, &cache, &event, true),
        ReplayAdvanceOutcome::Advanced
    );
}

/// Free replay: cache hit -> Advanced.
#[test]
fn replay_free_cache_hit_advances() {
    let mut cache: AiCache<String> = AiCache::new();
    let event = make_event("dummy", "0");
    cache.insert(&event.cache_key(), "stored".into());
    assert_eq!(
        replay_advance_ai_event(ReplayMode::Free, &cache, &event, true),
        ReplayAdvanceOutcome::Advanced
    );
}

/// Free replay: cache miss -> HybridCacheMiss.
#[test]
fn replay_free_cache_miss_is_refused() {
    let cache: AiCache<String> = AiCache::new();
    let event = make_event("dummy", "0");
    assert_eq!(
        replay_advance_ai_event(ReplayMode::Free, &cache, &event, true),
        ReplayAdvanceOutcome::Refused(ReplayRefusal::HybridCacheMiss)
    );
}

// ---------------------------------------------------------------------------
// preflight
// ---------------------------------------------------------------------------

/// FR-CIV-AI-009 — check_artifacts with no required artifacts succeeds.
#[test]
fn preflight_no_artifacts_is_ok() {
    let result = check_artifacts(&[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Vec::<String>::new());
}

/// FR-CIV-AI-009 — missing artifact produces loud named failure.
#[test]
fn preflight_missing_artifact_fails_loud() {
    let artifact = RequiredArtifact {
        name: "test-model".into(),
        path: "/nonexistent/path/to/model.gguf".into(),
    };
    let result = check_artifacts(&[artifact]);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("test-model"), "error must name the model: {msg}");
    assert!(msg.contains("missing model"), "error must use named-failure format: {msg}");
}

/// FR-CIV-AI-009 — required_artifacts with no local_model_path returns empty.
#[test]
fn required_artifacts_empty_when_no_local_path() {
    let config = AiConfig::default();
    // default has local_model_path = None
    assert!(config.local_model_path.is_none());
    let arts = required_artifacts(&config);
    assert!(arts.is_empty());
}

/// FR-CIV-AI-009 — preflight convenience fn succeeds when no artifacts required.
#[test]
fn preflight_convenience_ok_for_no_local_path() {
    let config = AiConfig::default();
    assert!(preflight(&config).is_ok());
}

// ---------------------------------------------------------------------------
// ProviderRole::as_str
// ---------------------------------------------------------------------------

#[test]
fn provider_role_as_str_returns_stable_names() {
    assert_eq!(ProviderRole::Narrator.as_str(), "narrator");
    assert_eq!(ProviderRole::Embedder.as_str(), "embedder");
    assert_eq!(ProviderRole::Heavy.as_str(), "heavy");
}

// ---------------------------------------------------------------------------
// ProviderRegistry::get
// ---------------------------------------------------------------------------

#[test]
fn registry_get_returns_none_for_unregistered_role() {
    let reg = ProviderRegistry::new();
    assert!(reg.get(ProviderRole::Narrator).is_none());
}

#[test]
fn registry_get_returns_provider_after_register() {
    let mut reg = ProviderRegistry::new();
    reg.register(ProviderRole::Narrator, Arc::new(DummyAiProvider));
    assert!(reg.get(ProviderRole::Narrator).is_some());
    // Unregistered roles still return None.
    assert!(reg.get(ProviderRole::Embedder).is_none());
}

/// require() loud failure message includes the role name.
#[test]
fn registry_require_error_message_names_role() {
    let reg = ProviderRegistry::new();
    let err = reg.require(ProviderRole::Heavy).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("heavy"), "error must name the role: {msg}");
}

// ---------------------------------------------------------------------------
// DummyAiProvider::generate_sync
// ---------------------------------------------------------------------------

/// generate_sync is a deterministic, synchronous shortcut.
#[test]
fn dummy_generate_sync_is_deterministic() {
    let p = DummyAiProvider;
    let req = GenRequest::from_prompt("build a rail line");
    let a = p.generate_sync(&req);
    let b = p.generate_sync(&req);
    assert_eq!(a.text, b.text);
    assert!(!a.from_cache);
    assert!(a.text.starts_with("dummy-generation-"));
}

/// generate_sync output differs from a different prompt.
#[test]
fn dummy_generate_sync_differs_per_prompt() {
    let p = DummyAiProvider;
    let a = p.generate_sync(&GenRequest::from_prompt("alpha"));
    let b = p.generate_sync(&GenRequest::from_prompt("beta"));
    assert_ne!(a.text, b.text);
}

// ---------------------------------------------------------------------------
// AiTask::id
// ---------------------------------------------------------------------------

#[test]
fn ai_task_id_echoed_for_generate_and_embed() {
    let provider = Arc::new(DummyAiProvider) as Arc<dyn civ_ai::AiProvider>;
    let gen_task = AiTask::Generate {
        id: 42,
        provider: Arc::clone(&provider),
        req: GenRequest::from_prompt("p"),
    };
    assert_eq!(gen_task.id(), 42);

    let embed_task = AiTask::Embed {
        id: 99,
        provider,
        req: EmbedRequest {
            texts: vec!["t".into()],
            input_snapshot_hash: [0u8; 32],
        },
    };
    assert_eq!(embed_task.id(), 99);
}

// ---------------------------------------------------------------------------
// gen_cache_key integration
// ---------------------------------------------------------------------------

/// gen_cache_key changes when the snapshot hash changes.
#[test]
fn gen_cache_key_changes_with_snapshot_hash() {
    let p = DummyAiProvider;
    let mut req = GenRequest::from_prompt("same prompt");
    let k1 = gen_cache_key(&p, &req);
    req.input_snapshot_hash = [0xFF; 32];
    let k2 = gen_cache_key(&p, &req);
    assert_ne!(k1, k2);
}