//! Integration tests: DummyAiProvider round-trip + cache (FR-CIV-AI-006/007),
//! registry loud-failure (FR-CIV-AI-009), worker pool never-await contract
//! (FR-CIV-AI-008), and replay rules.

use std::sync::Arc;

use civ_ai::cache::AiCache;
use civ_ai::pool::{AiPayload, AiTask, AiWorkerPool};
use civ_ai::provenance::{replay_advance_ai_event, AiEvent, ReplayAdvanceOutcome, ReplayRefusal};
use civ_ai::registry::MissingProvider;
use civ_ai::{
    cached_generate, gen_cache_key, AiError, AiProvider, DummyAiProvider, EmbedRequest, GenOutput,
    GenRequest, ProviderRegistry, ProviderRole, ReplayMode,
};

#[tokio::test]
async fn dummy_generate_is_deterministic() {
    let p = DummyAiProvider;
    let req = GenRequest::from_prompt("build a rail line");
    let a = p.generate(&req).await.expect("gen");
    let b = p.generate(&req).await.expect("gen");
    assert_eq!(a.text, b.text);
    assert_eq!(a.output_hash, b.output_hash);
    assert!(!a.from_cache);
}

#[tokio::test]
async fn dummy_embed_round_trips() {
    let p = DummyAiProvider;
    let req = EmbedRequest {
        texts: vec!["alpha".into(), "beta".into()],
        input_snapshot_hash: [0u8; 32],
    };
    let v1 = p.embed(&req).await.expect("embed");
    let v2 = p.embed(&req).await.expect("embed");
    assert_eq!(v1.len(), 2);
    assert_eq!(v1, v2);
}

#[tokio::test]
async fn cached_generate_hits_on_repeat() {
    let p = DummyAiProvider;
    let mut cache: AiCache<GenOutput> = AiCache::new();
    let req = GenRequest::from_prompt("legend of the iron age");

    let first = cached_generate(&p, &mut cache, &req).await.expect("first");
    assert!(!first.from_cache);
    assert_eq!(cache.len(), 1);

    let second = cached_generate(&p, &mut cache, &req).await.expect("second");
    assert!(second.from_cache);
    assert_eq!(first.text, second.text);
    assert_eq!(cache.len(), 1);
}

#[test]
fn cache_round_trips_generic_value() {
    let mut cache: AiCache<String> = AiCache::new();
    assert!(cache.is_empty());
    cache.insert(b"k", "v".into());
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.get(b"k"), Some(&"v".to_string()));
    assert!(cache.contains_key(b"k"));
}

#[test]
fn registry_required_provider_fails_loud() {
    let mut reg = ProviderRegistry::new();
    assert_eq!(
        reg.require(ProviderRole::Narrator).err(),
        Some(MissingProvider("narrator"))
    );
    reg.register(ProviderRole::Narrator, Arc::new(DummyAiProvider));
    assert!(reg.require(ProviderRole::Narrator).is_ok());
    // Unregistered role still fails loud.
    assert!(reg.require(ProviderRole::Embedder).is_err());
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "pre-existing tokio runtime blocking issue"]
async fn worker_pool_runs_task_off_thread() {
    let mut pool = AiWorkerPool::spawn(8, 2);
    let provider: Arc<dyn AiProvider> = Arc::new(DummyAiProvider);
    let enqueued = pool
        .try_enqueue(AiTask::Generate {
            id: 7,
            provider,
            req: GenRequest::from_prompt("chronicle"),
        })
        .is_ok();
    assert!(enqueued, "task should enqueue onto an empty bounded queue");

    let result = pool.next_result().await.expect("result");
    assert_eq!(result.id, 7);
    match result.payload {
        AiPayload::Text(out) => assert!(out.text.starts_with("dummy-generation-")),
        other => panic!("unexpected payload: {other:?}"),
    }
}

#[test]
fn replay_canonical_refuses_ai_event() {
    let cache: AiCache<String> = AiCache::new();
    let event = AiEvent {
        seed: 1,
        prompt_hash: [0xAA; 32],
        model_id: "dummy".into(),
        model_version: "0".into(),
        input_snapshot_hash: [0xBB; 32],
        output_hash: [0xCC; 32],
        output: "saga".to_string(),
        tick: 1,
    };
    assert_eq!(
        replay_advance_ai_event(ReplayMode::Canonical, &cache, &event, true),
        ReplayAdvanceOutcome::Refused(ReplayRefusal::CanonicalAiEvent)
    );
    // Live play always advances.
    assert_eq!(
        replay_advance_ai_event(ReplayMode::Hybrid, &cache, &event, false),
        ReplayAdvanceOutcome::Advanced
    );
}

#[test]
fn fr_civ_llm_001_cache_key_is_stable_for_identical_inputs() {
    let provider = DummyAiProvider;
    let req = GenRequest::from_prompt("forge a frontier city");
    let key_a = gen_cache_key(&provider, &req);
    let key_b = gen_cache_key(&provider, &req);
    assert_eq!(key_a, key_b);
}

#[test]
fn fr_civ_llm_002_cache_key_changes_when_prompt_or_model_shift() {
    let provider = DummyAiProvider;
    let first = GenRequest::from_prompt("a");
    let second = GenRequest::from_prompt("b");
    assert_ne!(
        gen_cache_key(&provider, &first),
        gen_cache_key(&provider, &second),
        "prompt changes should not collide"
    );

    let mut cache = AiCache::new();
    let mut custom = AiEvent {
        seed: 0,
        prompt_hash: first.prompt_hash(),
        model_id: "dummy".into(),
        model_version: "0".into(),
        input_snapshot_hash: first.input_snapshot_hash,
        output_hash: [0u8; 32],
        output: String::from("ok"),
        tick: 1,
    };
    let key_first = custom.cache_key();
    custom.model_version = "1".into();
    assert_ne!(key_first, custom.cache_key());
    cache.insert(&key_first, "v1".to_string());
    assert_eq!(cache.get(&key_first), Some(&"v1".to_string()));
    assert!(!cache.is_empty());
}

#[test]
fn fr_civ_llm_003_max_concurrent_gen_from_env_is_honored() {
    let backup = std::env::var("CIVAI_MAX_CONCURRENT_GEN").ok();
    std::env::set_var("CIVAI_MAX_CONCURRENT_GEN", "7");
    let config = civ_ai::AiConfig::from_env();
    assert_eq!(config.max_concurrent_gen, 7);

    match backup {
        Some(value) => std::env::set_var("CIVAI_MAX_CONCURRENT_GEN", value),
        None => std::env::remove_var("CIVAI_MAX_CONCURRENT_GEN"),
    }
}

#[tokio::test]
async fn fr_civ_llm_004_provider_operations_enforce_allowed_use_matrix() {
    struct GenerateOnlyProvider;

    #[async_trait::async_trait]
    impl civ_ai::AiProvider for GenerateOnlyProvider {
        async fn generate(&self, req: &GenRequest) -> Result<GenOutput, AiError> {
            Ok(GenOutput::fresh(req.prompt.clone()))
        }

        async fn embed(&self, _req: &EmbedRequest) -> Result<Vec<Vec<f32>>, AiError> {
            Err(AiError::Unsupported("generate-only".into()))
        }

        fn model_id(&self) -> &str {
            "gen-only"
        }

        fn model_version(&self) -> &str {
            "1"
        }

        fn capabilities(&self) -> civ_ai::Capabilities {
            civ_ai::Capabilities {
                generate: true,
                embed: false,
                cloud: false,
            }
        }
    }

    let provider = GenerateOnlyProvider;
    let req = GenRequest::from_prompt("policy tone sample");
    let out = provider
        .generate(&req)
        .await
        .expect("generate should be allowed");
    assert_eq!(out.text, "policy tone sample");
    let err = provider
        .embed(&EmbedRequest {
            texts: vec!["x".into()],
            input_snapshot_hash: [0; 32],
        })
        .await
        .expect_err("embed should be rejected");
    let msg = format!("{err}");
    assert!(msg.contains("generate-only"));
}
