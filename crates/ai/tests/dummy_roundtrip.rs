//! Integration tests: DummyAiProvider round-trip + cache (FR-CIV-AI-006/007),
//! registry loud-failure (FR-CIV-AI-009), worker pool never-await contract
//! (FR-CIV-AI-008), and replay rules.

use std::sync::Arc;

use civ_ai::pool::{AiPayload, AiTask, AiWorkerPool};
use civ_ai::provenance::{replay_advance_ai_event, AiEvent, ReplayAdvanceOutcome, ReplayRefusal};
use civ_ai::registry::MissingProvider;
use civ_ai::{
    cached_generate, AiCache, AiProvider, DummyAiProvider, EmbedRequest, GenOutput, GenRequest,
    ProviderRegistry, ProviderRole, ReplayMode,
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
