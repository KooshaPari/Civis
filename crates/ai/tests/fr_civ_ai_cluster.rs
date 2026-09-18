//! Real behavioural oracles for the FR-CIV-AI-001..005 cluster of `civ-ai`
//! (the generic `AiProvider` port and its five provider adapters).
//!
//! Requirement source: `docs/design/civ-ai-crate.md` §2 (FR-CIV-AI-001..005).
//!
//! | ID | Requirement (abridged) | Oracles here |
//! |---|---|---|
//! | FR-CIV-AI-001 | generic `AiProvider` port (`generate` + `embed`, `Send + Sync`, async, `model_id`/`model_version` for provenance), usable behind `Arc` | dyn/`Send + Sync`/`Arc` sharing, trait-object dispatch, cache-key byte layout driven by model identity, request defaults matching the configured token budget |
//! | FR-CIV-AI-002 | `LocalSlmProvider` is the local in-game generator; honors the model path from `.env` | `CIVAI_LOCAL_MODEL_PATH` becomes the *required* preflight artifact, missing artifact fails loud and named, local-first defaults keep cloud unselected |
//! | FR-CIV-AI-003 | `OllamaDevProvider` — dev-only, OpenAI-compatible HTTP, never a shipping dependency | loopback-HTTP oracles: route normalization, request shape (model/messages/budgets/response_format), typed error mapping, unsupported `embed`, plus the feature-gate/`default = []` invariant |
//! | FR-CIV-AI-004 | `FirepassKimiProvider` — wraps the existing cloud client, opt-in fallback only, missing key fails loud | missing-key loud failure, proxied prompt/budget/schema to the cloud chat route, typed error mapping, unsupported `embed`, opt-in-only + not-a-startup-requirement invariants |
//! | FR-CIV-AI-005 | `EmbedProvider` — MiniLM embeddings; generation unsupported (loud) | artifact-backed load with deterministic named missing-file chain (never downloads), unloaded provider refuses loudly and claims no capability, zero-dimension rejection, `generate` unsupported, batched order-preserving embed contract, env-gated real-model dimensional check |
//!
//! Provider-specific oracles for FR-CIV-AI-003/004/005 live in `#[cfg(feature =
//! ...)]` modules because those adapters only exist behind the `dev` / `cloud` /
//! `embed` features. Their gating is itself asserted by the always-on tests, and
//! each gated test is run with its feature enabled (see the verification
//! commands in the task report). Everything not gated runs under the crate's
//! default feature set (which is empty).

use std::sync::{Arc, Mutex, MutexGuard};

use civ_ai::{
    gen_cache_key, AiConfig, AiError, AiProvider, Capabilities, DummyAiProvider, EmbedRequest,
    GenOutput, GenRequest,
};

// ---------------------------------------------------------------------------
// Test support
// ---------------------------------------------------------------------------

/// Serializes tests that mutate process-global environment variables (the test
/// binary runs its tests on parallel threads).
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Env var names this cluster touches.
const ENV_KEYS: &[&str] = &[
    "CIVAI_LOCAL_MODEL_PATH",
    "CIVAI_NARRATOR_MODEL",
    "CIVAI_EMBED_MODEL",
    "CIVAI_MAX_CONCURRENT_GEN",
    "CIVAI_GEN_TOKEN_BUDGET",
    "CIVAI_ENABLE_CLOUD",
    "CIVAI_OLLAMA_URL",
    "KIMI_API_KEY",
    "FIREPASS_BASE_URL",
];

/// Saves and clears the `civ-ai`-relevant env vars, restoring them on drop.
struct EnvGuard {
    _lock: MutexGuard<'static, ()>,
    saved: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn new() -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let saved = ENV_KEYS
            .iter()
            .map(|key| (*key, std::env::var(key).ok()))
            .collect();
        for key in ENV_KEYS {
            std::env::remove_var(key);
        }
        Self { _lock: lock, saved }
    }

    fn set(&self, key: &str, value: &str) {
        std::env::set_var(key, value);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.saved {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }
}

/// A unique scratch directory for filesystem oracles.
struct ScratchDir(std::path::PathBuf);

impl ScratchDir {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "civ-ai-cluster-{label}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("create scratch dir");
        Self(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }

    fn write(&self, name: &str, contents: &str) -> std::path::PathBuf {
        let file = self.0.join(name);
        std::fs::write(&file, contents).expect("write scratch file");
        file
    }

    /// Path string for config/env assertions.
    fn path_str(&self) -> &str {
        self.0.to_str().expect("utf-8 scratch path")
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Compile-time `Send + Sync` assertion helper (used on `dyn AiProvider`).
fn assert_send_sync<T: Send + Sync + ?Sized>() {}

/// Reads a file shipped next to this test target's package.
fn crate_file(relative: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

/// The `key = ...` value line for `key` in a TOML manifest, trimmed.
fn manifest_value(manifest: &str, key: &str) -> String {
    manifest
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with(&format!("{key} =")))
        .unwrap_or_else(|| panic!("manifest key `{key}` not found"))
        .to_string()
}

/// True when `mod <module>;` in `source` is directly preceded by
/// `#[cfg(feature = "<feature>")]`.
fn is_feature_gated(source: &str, module: &str, feature: &str) -> bool {
    let lines: Vec<&str> = source.lines().map(str::trim).collect();
    let needle = format!("mod {module};");
    let Some(index) = lines.iter().position(|line| *line == needle) else {
        return false;
    };
    let attribute = format!("#[cfg(feature = \"{feature}\")]");
    lines[..index]
        .iter()
        .rev()
        .take(2)
        .any(|line| *line == attribute)
}

/// A provider whose only distinctive behaviour is its declared model identity.
struct StaticProvider {
    model_id: &'static str,
    model_version: &'static str,
    cloud: bool,
}

#[async_trait::async_trait]
impl AiProvider for StaticProvider {
    async fn generate(&self, req: &GenRequest) -> Result<GenOutput, AiError> {
        Ok(GenOutput::fresh(format!("static:{}", req.prompt)))
    }

    async fn embed(&self, _req: &EmbedRequest) -> Result<Vec<Vec<f32>>, AiError> {
        Err(AiError::Unsupported("static-provider".into()))
    }

    fn model_id(&self) -> &str {
        self.model_id
    }

    fn model_version(&self) -> &str {
        self.model_version
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            generate: true,
            embed: false,
            cloud: self.cloud,
        }
    }
}

// ===========================================================================
// FR-CIV-AI-001 — generic AiProvider port (generate + embed, Send + Sync,
// async, model_id/model_version for provenance, usable behind Arc)
// ===========================================================================

/// Covers FR-CIV-AI-001.
#[test]
fn fr_civ_ai_001_trait_object_is_dyn_safe_send_sync_and_arc_shareable() {
    // The port must be object-safe and usable across threads as `Arc<dyn ...>`.
    assert_send_sync::<dyn AiProvider>();

    let shared: Arc<dyn AiProvider> = Arc::new(DummyAiProvider);
    let request = GenRequest::from_prompt("epoch one digest");
    let expected = DummyAiProvider
        .generate_sync(&request)
        .text
        .clone();
    assert!(
        !expected.is_empty(),
        "generate_sync must produce a non-empty deterministic string"
    );

    let moved = Arc::clone(&shared);
    let moved_request = request.clone();
    let worker = std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build runtime");
        let text = runtime.block_on(async {
            moved
                .generate(&moved_request)
                .await
                .expect("generate through Arc<dyn AiProvider>")
                .text
        });
        (text, moved.model_id().to_string(), moved.model_version().to_string())
    });

    let (thread_text, model_id, model_version) = worker.join().expect("worker thread");
    assert_eq!(
        thread_text, expected,
        "deterministic provider must agree across threads through the trait object"
    );
    assert_eq!(model_id, "dummy");
    assert_eq!(model_version, "0");
    assert_eq!(Arc::strong_count(&shared), 1, "worker clone must be dropped");
}

/// Covers FR-CIV-AI-001.
#[tokio::test]
async fn fr_civ_ai_001_generate_and_embed_dispatch_through_trait_objects() {
    let provider: Arc<dyn AiProvider> = Arc::new(DummyAiProvider);
    let request = GenRequest::from_prompt("rail line between two hubs");

    let concrete = DummyAiProvider.generate(&request).await.expect("concrete gen");
    let dyn_ref: &dyn AiProvider = provider.as_ref();
    let via_dyn = dyn_ref.generate(&request).await.expect("dyn gen");
    let via_arc = provider.generate(&request).await.expect("arc gen");
    assert_eq!(concrete, via_dyn);
    assert_eq!(concrete, via_arc);

    let embed = EmbedRequest {
        texts: vec!["alpha".into(), "beta".into()],
        input_snapshot_hash: [3u8; 32],
    };
    let via_dyn_embed = dyn_ref.embed(&embed).await.expect("dyn embed");
    assert_eq!(via_dyn_embed.len(), 2, "one vector per batched input");

    // The port is usable as a heterogeneous collection of providers.
    let providers: Vec<Arc<dyn AiProvider>> = vec![
        Arc::new(DummyAiProvider),
        Arc::new(StaticProvider {
            model_id: "static",
            model_version: "1",
            cloud: false,
        }),
    ];
    let mut texts = Vec::new();
    for entry in &providers {
        texts.push(entry.generate(&request).await.expect("gen").text);
    }
    assert_eq!(texts.len(), 2);
    assert_ne!(texts[0], texts[1], "distinct providers must be dispatched distinctly");
}

/// Covers FR-CIV-AI-001.
#[test]
fn fr_civ_ai_001_model_identity_flows_into_provenance_cache_key() {
    let request = GenRequest {
        prompt: "narrate the bronze collapse".into(),
        max_tokens: 128,
        temperature: 0.2,
        json_schema: None,
        input_snapshot_hash: [0x5Au8; 32],
        seed: Some(9),
    };

    let narrator_v1 = StaticProvider {
        model_id: "narrator",
        model_version: "1",
        cloud: false,
    };
    let narrator_v2 = StaticProvider {
        model_id: "narrator",
        model_version: "2",
        cloud: false,
    };

    let key_v1 = gen_cache_key(&narrator_v1, &request);
    let key_v2 = gen_cache_key(&narrator_v2, &request);

    // Exact composite layout: prompt_hash ‖ input_snapshot_hash ‖ model_id ‖ model_version.
    assert_eq!(key_v1.len(), 32 + 32 + "narrator".len() + "1".len());
    assert_eq!(&key_v1[..32], &request.prompt_hash()[..]);
    assert_eq!(&key_v1[32..64], &request.input_snapshot_hash[..]);
    assert_eq!(&key_v1[64..], b"narrator1");
    assert_eq!(&key_v2[64..], b"narrator2");
    assert_ne!(
        key_v1, key_v2,
        "model_version is provenance: a version bump must not hit the old cache entry"
    );

    // Identity, not concrete type, drives the key: the dummy provider declaring
    // the same identity produces the same key (port-level provenance contract).
    let same_identity_static = StaticProvider {
        model_id: "dummy",
        model_version: "0",
        cloud: false,
    };
    assert_eq!(
        gen_cache_key(&same_identity_static, &request),
        gen_cache_key(&DummyAiProvider, &request)
    );
    assert_ne!(
        gen_cache_key(&DummyAiProvider, &request),
        key_v1,
        "different model identities must not collide"
    );
}

/// Covers FR-CIV-AI-001.
#[test]
fn fr_civ_ai_001_gen_request_defaults_match_configured_token_budget() {
    let request = GenRequest::from_prompt("found a religion");
    let defaults = AiConfig::default();

    assert_eq!(
        request.max_tokens, defaults.gen_token_budget,
        "GenRequest::from_prompt must not exceed the configured token budget"
    );
    assert_eq!(request.max_tokens, 600);
    assert!((request.temperature - 0.7).abs() < f32::EPSILON);
    assert!(request.json_schema.is_none());
    assert!(request.seed.is_none());

    // `from_prompt` hashes the rendered prompt into the snapshot field, and the
    // prompt hash is blake3 over the rendered prompt (cache-key inputs).
    let expected = *blake3::hash(b"found a religion").as_bytes();
    assert_eq!(request.prompt_hash(), expected);
    assert_eq!(request.input_snapshot_hash, expected);
    let other = GenRequest::from_prompt("found a different religion");
    assert_ne!(request.prompt_hash(), other.prompt_hash());

    // The request shape round-trips, so recorded provenance stays comparable.
    let encoded = serde_json::to_string(&request).expect("serialize request");
    let decoded: GenRequest = serde_json::from_str(&encoded).expect("deserialize request");
    assert_eq!(decoded, request);

    // Capabilities are a plain, copyable, round-tripping value.
    let capabilities = DummyAiProvider.capabilities();
    assert_eq!(capabilities, capabilities);
    let json = serde_json::to_string(&capabilities).expect("serialize capabilities");
    let parsed: Capabilities = serde_json::from_str(&json).expect("deserialize capabilities");
    assert_eq!(parsed, capabilities);
}

// ===========================================================================
// FR-CIV-AI-002 — LocalSlmProvider is the local in-game generator; honors the
// model path from `.env`
// ===========================================================================

/// Covers FR-CIV-AI-002.
#[test]
fn fr_civ_ai_002_local_model_env_path_is_the_required_preflight_artifact() {
    let env = EnvGuard::new();
    let scratch = ScratchDir::new("fr002-present");
    let model = scratch.write("local-slm.gguf", "not a real gguf");

    env.set("CIVAI_LOCAL_MODEL_PATH", model.to_str().expect("utf-8 model path"));
    env.set("CIVAI_NARRATOR_MODEL", "qwen2.5-1.5b-instruct");

    let config = AiConfig::from_env();
    assert_eq!(
        config.local_model_path.as_deref(),
        model.to_str(),
        "the local GGUF path must come from the .env-driven key"
    );

    let required = civ_ai::preflight::required_artifacts(&config);
    assert_eq!(required.len(), 1, "only the local GGUF is a startup requirement");
    assert_eq!(required[0].name, "qwen2.5-1.5b-instruct");
    assert_eq!(required[0].path, model.to_str().expect("utf-8 model path"));

    // Present artifact -> preflight names it; the sim may start.
    assert_eq!(
        civ_ai::preflight::preflight(&config),
        Ok(vec!["qwen2.5-1.5b-instruct".to_string()])
    );

    // An unconfigured install demands no artifact to start (nothing cloud).
    let unconfigured = AiConfig::default();
    assert!(civ_ai::preflight::required_artifacts(&unconfigured).is_empty());
    assert_eq!(civ_ai::preflight::preflight(&unconfigured), Ok(Vec::new()));
}

/// Covers FR-CIV-AI-002.
#[test]
fn fr_civ_ai_002_missing_local_artifact_fails_loud_and_named() {
    let env = EnvGuard::new();
    let scratch = ScratchDir::new("fr002-missing");
    let missing = format!("{}/absent-q4_k_m.gguf", scratch.path_str());

    env.set("CIVAI_LOCAL_MODEL_PATH", &missing);

    let config = AiConfig::from_env();
    let error = civ_ai::preflight::preflight(&config)
        .expect_err("a missing local model artifact must fail preflight, never silently disable AI");

    assert_eq!(
        error.to_string(),
        format!(
            "civ-ai preflight failed: missing model '{}' at {missing}",
            config.narrator_model
        ),
        "the failure must name the model and its path"
    );

    // Multiple missing artifacts are reported together (semicolon-separated).
    let other = scratch.path().join("absent-embed.onnx");
    let combined = civ_ai::preflight::check_artifacts(&[
        civ_ai::preflight::RequiredArtifact {
            name: "narrator".into(),
            path: missing.clone(),
        },
        civ_ai::preflight::RequiredArtifact {
            name: "embedder".into(),
            path: other.display().to_string(),
        },
    ])
    .expect_err("both artifacts are missing");
    let message = combined.to_string();
    assert!(message.contains("missing model 'narrator' at"));
    assert!(message.contains("missing model 'embedder' at"));
    assert!(message.contains("; "), "missing artifacts are listed together");
}

/// Covers FR-CIV-AI-002.
#[test]
fn fr_civ_ai_002_local_first_defaults_keep_cloud_unselected() {
    let _env = EnvGuard::new();
    let config = AiConfig::from_env();

    assert_eq!(config, AiConfig::default());
    assert!(
        !config.enable_cloud,
        "the default in-game generator is local; cloud is explicit opt-in"
    );
    assert!(
        config.narrator_model.contains("1.5b"),
        "the default narrator must stay in the <=1.5B class, got {}",
        config.narrator_model
    );
    assert_eq!(config.embed_model, "all-MiniLM-L6-v2");
    assert_eq!(config.max_concurrent_gen, 2);
    assert_eq!(config.ollama_url, "http://localhost:11434");
    assert!(config.local_model_path.is_none());

    // The dev/cloud endpoints are configured but not required at startup.
    assert!(civ_ai::preflight::required_artifacts(&config).is_empty());
}

// ===========================================================================
// FR-CIV-AI-003 — OllamaDevProvider: dev-only, OpenAI-compatible, never ships
// ===========================================================================

/// Covers FR-CIV-AI-003.
#[test]
fn fr_civ_ai_003_dev_provider_is_feature_gated_out_of_shipping_registry() {
    let manifest = crate_file("Cargo.toml");
    assert_eq!(
        manifest_value(&manifest, "default"),
        "default = []",
        "civ-ai default features must stay empty so no dev/cloud provider ships by default"
    );
    assert_eq!(manifest_value(&manifest, "dev"), "dev = [\"dep:reqwest\"]");

    let registry_source = crate_file("src/providers/mod.rs");
    assert!(
        is_feature_gated(&registry_source, "ollama_dev", "dev"),
        "OllamaDevProvider must stay behind the `dev` feature gate"
    );
}

#[cfg(feature = "dev")]
mod dev_provider {
    use super::*;

    /// Covers FR-CIV-AI-003.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fr_civ_ai_003_ollama_generate_posts_openai_chat_shape() {
        let server = loopback::spawn(vec![loopback::ok(
            r#"{"choices":[{"message":{"content":"the iron age dawns"}}]}"#,
        )]);
        let provider = civ_ai::providers::OllamaDevProvider::new(server.base_url(), "llama3.2:3b");

        let mut request = GenRequest::from_prompt("narrate the iron age");
        request.max_tokens = 128;
        request.temperature = 0.25;
        request.json_schema = Some(r#"{"type":"object"}"#.to_string());

        let output = provider.generate(&request).await.expect("generate");
        assert_eq!(output.text, "the iron age dawns");
        assert!(!output.from_cache);
        assert_eq!(
            output.output_hash,
            *blake3::hash(output.text.as_bytes()).as_bytes(),
            "provenance hash must cover the emitted text"
        );

        let captured = server.next_request();
        assert_eq!(captured.path, "/v1/chat/completions");
        let content_type = loopback::header(&captured.headers, "content-type").unwrap_or_default();
        assert!(
            content_type.contains("application/json"),
            "the chat request must be posted as JSON, got: {content_type}"
        );
        let body: serde_json::Value =
            serde_json::from_str(&captured.body).expect("request body must be JSON");
        assert_eq!(body["model"], "llama3.2:3b");
        assert_eq!(body["max_tokens"], 128);
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "narrate the iron age");
        assert_eq!(body["response_format"]["type"], "json_object");
        let temperature = body["temperature"].as_f64().expect("temperature");
        assert!((temperature - 0.25).abs() < 1e-6);

        assert_eq!(provider.model_id(), "llama3.2:3b");
        assert_eq!(provider.model_version(), "dev");
        let capabilities = provider.capabilities();
        assert!(capabilities.generate && !capabilities.embed);
        assert!(capabilities.cloud, "Ollama is a local HTTP service, classified as remote");
    }

    /// Covers FR-CIV-AI-003.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fr_civ_ai_003_ollama_endpoint_variants_normalize_to_chat_route() {
        let response = loopback::ok(r#"{"choices":[{"message":{"content":"ok"}}]}"#);
        let server = loopback::spawn(vec![response.clone(), response.clone(), response]);

        let endpoints = [
            server.base_url(),
            format!("{}/v1/", server.base_url()),
            format!("{}/v1/chat/completions", server.base_url()),
        ];
        for endpoint in endpoints {
            let provider = civ_ai::providers::OllamaDevProvider::new(endpoint.clone(), "m");
            let output = provider
                .generate(&GenRequest::from_prompt("ping"))
                .await
                .expect("generate");
            assert_eq!(output.text, "ok");
            let captured = server.next_request();
            assert_eq!(
                captured.path, "/v1/chat/completions",
                "endpoint `{endpoint}` must normalize onto the OpenAI chat route"
            );
        }

        // Without a JSON schema no response_format is forced.
        let server = loopback::spawn(vec![loopback::ok(
            r#"{"choices":[{"message":{"content":"plain"}}]}"#,
        )]);
        let provider = civ_ai::providers::OllamaDevProvider::new(server.base_url(), "m");
        provider
            .generate(&GenRequest::from_prompt("plain prompt"))
            .await
            .expect("generate");
        let body: serde_json::Value =
            serde_json::from_str(&server.next_request().body).expect("JSON body");
        assert!(
            body.get("response_format").is_none(),
            "response_format must be omitted when no schema is requested"
        );
    }

    /// Covers FR-CIV-AI-003.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fr_civ_ai_003_ollama_errors_are_typed_and_loud() {
        // HTTP 429 -> RateLimited.
        let server = loopback::spawn(vec![loopback::status(429, "Too Many Requests", "")]);
        let provider = civ_ai::providers::OllamaDevProvider::new(server.base_url(), "m");
        assert_eq!(
            provider.generate(&GenRequest::from_prompt("x")).await,
            Err(AiError::RateLimited)
        );

        // Other non-success statuses -> loud Unavailable naming the status code.
        let server = loopback::spawn(vec![loopback::status(500, "Internal Server Error", "boom")]);
        let provider = civ_ai::providers::OllamaDevProvider::new(server.base_url(), "m");
        match provider.generate(&GenRequest::from_prompt("x")).await {
            Err(AiError::Unavailable(message)) => {
                assert!(message.contains("HTTP 500"), "got: {message}");
            }
            other => panic!("expected Unavailable, got {other:?}"),
        }

        // Connection refused (no listener) -> loud Unavailable, never a panic or
        // an empty success.
        let dead = loopback::dead_address();
        let provider = civ_ai::providers::OllamaDevProvider::new(dead, "m");
        match provider.generate(&GenRequest::from_prompt("x")).await {
            Err(AiError::Unavailable(message)) => {
                assert!(message.contains("Ollama request failed"), "got: {message}");
            }
            other => panic!("expected Unavailable, got {other:?}"),
        }
    }

    /// Covers FR-CIV-AI-003.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fr_civ_ai_003_ollama_malformed_responses_are_rejected() {
        // Non-JSON body.
        let server = loopback::spawn(vec![loopback::ok("not json at all")]);
        let provider = civ_ai::providers::OllamaDevProvider::new(server.base_url(), "m");
        match provider.generate(&GenRequest::from_prompt("x")).await {
            Err(AiError::InvalidResponse(message)) => {
                assert!(message.contains("JSON decode failed"), "got: {message}");
            }
            other => panic!("expected InvalidResponse, got {other:?}"),
        }

        // JSON envelope without any choice.
        let server = loopback::spawn(vec![loopback::ok(r#"{"choices":[]}"#)]);
        let provider = civ_ai::providers::OllamaDevProvider::new(server.base_url(), "m");
        match provider.generate(&GenRequest::from_prompt("x")).await {
            Err(AiError::InvalidResponse(message)) => {
                assert!(message.contains("no content"), "got: {message}");
            }
            other => panic!("expected InvalidResponse, got {other:?}"),
        }

        // Empty content is not a successful generation.
        let server = loopback::spawn(vec![loopback::ok(
            r#"{"choices":[{"message":{"content":""}}]}"#,
        )]);
        let provider = civ_ai::providers::OllamaDevProvider::new(server.base_url(), "m");
        assert!(matches!(
            provider.generate(&GenRequest::from_prompt("x")).await,
            Err(AiError::InvalidResponse(_))
        ));
    }

    /// Covers FR-CIV-AI-003.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fr_civ_ai_003_ollama_embed_is_unsupported_and_not_advertised() {
        let provider =
            civ_ai::providers::OllamaDevProvider::new(loopback::dead_address(), "llama3.2:3b");
        let error = provider
            .embed(&EmbedRequest {
                texts: vec!["alpha".into()],
                input_snapshot_hash: [0u8; 32],
            })
            .await
            .expect_err("the dev provider is generate-only");
        assert_eq!(error, AiError::Unsupported("ollama-dev".into()));
        assert!(
            !provider.capabilities().embed,
            "capabilities must let callers route without a failed round-trip"
        );
        assert_eq!(provider.model_id(), "llama3.2:3b");
    }
}

// ===========================================================================
// FR-CIV-AI-004 — FirepassKimiProvider: cloud fallback only, loud when absent
// ===========================================================================

/// Covers FR-CIV-AI-004.
#[test]
fn fr_civ_ai_004_cloud_is_opt_in_and_not_a_startup_requirement() {
    let env = EnvGuard::new();
    let config = AiConfig::from_env();
    assert!(
        !config.enable_cloud,
        "cloud fallback is opt-in via CIVAI_ENABLE_CLOUD=1 only"
    );

    // Even with cloud credentials present, startup requires no cloud artifact.
    env.set("KIMI_API_KEY", "present-but-unused");
    env.set("FIREPASS_BASE_URL", "https://example.invalid/v1");
    assert!(
        civ_ai::preflight::required_artifacts(&config).is_empty(),
        "cloud credentials are validated at the call site, not by startup preflight"
    );
    assert_eq!(civ_ai::preflight::preflight(&config), Ok(Vec::new()));
    assert!(!AiConfig::from_env().enable_cloud);

    // Config only reports the opt-in flag; it never selects cloud implicitly.
    let mut opted_in = AiConfig::default();
    opted_in.enable_cloud = true;
    assert!(opted_in.enable_cloud && AiConfig::default().local_model_path.is_none());

    // The cloud provider stays behind its feature gate (opt-in build knob too).
    let manifest = crate_file("Cargo.toml");
    assert_eq!(manifest_value(&manifest, "cloud"), "cloud = [\"dep:civ-research\"]");
    assert!(is_feature_gated(
        &crate_file("src/providers/mod.rs"),
        "firepass_kimi",
        "cloud"
    ));
}

#[cfg(feature = "cloud")]
mod cloud_provider {
    use super::*;

    /// Covers FR-CIV-AI-004.
    #[test]
    fn fr_civ_ai_004_missing_api_key_fails_loud_at_construction() {
        let _env = EnvGuard::new();
        match civ_ai::providers::FirepassKimiProvider::from_env() {
            Err(AiError::Unavailable(message)) => {
                assert!(
                    message.contains("KIMI_API_KEY"),
                    "the failure must name the missing key, got: {message}"
                );
            }
            Ok(_) => panic!("a cloud provider must not construct without credentials"),
        }
    }

    /// Covers FR-CIV-AI-004.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fr_civ_ai_004_generate_proxies_prompt_budgets_and_schema() {
        let env = EnvGuard::new();
        let server = loopback::spawn(vec![loopback::ok(
            r#"{"choices":[{"message":{"content":"the saga of Ashur"}}]}"#,
        )]);
        env.set("KIMI_API_KEY", "test-cloud-key");
        env.set("FIREPASS_BASE_URL", &format!("{}/v1", server.base_url()));

        let provider = civ_ai::providers::FirepassKimiProvider::from_env().expect("construct");
        let mut request = GenRequest::from_prompt("write a saga");
        request.max_tokens = 256;
        request.temperature = 0.4;
        request.json_schema = Some(r#"{"type":"object"}"#.to_string());

        let output = provider.generate(&request).await.expect("generate");
        assert_eq!(output.text, "the saga of Ashur");
        assert_eq!(
            output.output_hash,
            *blake3::hash(output.text.as_bytes()).as_bytes()
        );

        let captured = server.next_request();
        assert_eq!(captured.path, "/v1/chat/completions");
        assert_eq!(
            loopback::header(&captured.headers, "authorization").as_deref(),
            Some("Bearer test-cloud-key"),
            "the wrapped client authenticates with KIMI_API_KEY"
        );
        let body: serde_json::Value = serde_json::from_str(&captured.body).expect("JSON body");
        assert_eq!(body["model"], "kimi-k2.6-turbo");
        assert_eq!(body["max_tokens"], 256);
        assert_eq!(body["response_format"]["type"], "json_object");
        let content = body["messages"][0]["content"].as_str().expect("content");
        assert!(
            content.starts_with("write a saga"),
            "the prompt must be forwarded, got: {content}"
        );
        assert!(
            content.contains("Return only JSON matching this schema:"),
            "a requested schema must be injected into the cloud prompt"
        );

        assert_eq!(provider.model_id(), "kimi-k2.6-turbo");
        assert_eq!(provider.model_version(), "cloud");
        let capabilities = provider.capabilities();
        assert!(capabilities.cloud && capabilities.generate && !capabilities.embed);
    }

    /// Covers FR-CIV-AI-004.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fr_civ_ai_004_cloud_errors_are_typed_and_loud() {
        let env = EnvGuard::new();
        env.set("KIMI_API_KEY", "test-cloud-key");

        // Unreachable endpoint -> loud Unavailable, never a silent empty result.
        env.set("FIREPASS_BASE_URL", &format!("{}/v1", loopback::dead_address()));
        let provider = civ_ai::providers::FirepassKimiProvider::from_env().expect("construct");
        match provider.generate(&GenRequest::from_prompt("x")).await {
            Err(AiError::Unavailable(message)) => {
                assert!(message.contains("Firepass/Kimi request failed"), "got: {message}");
            }
            other => panic!("expected Unavailable, got {other:?}"),
        }

        // HTTP 429 -> RateLimited.
        let server = loopback::spawn(vec![loopback::status(429, "Too Many Requests", "")]);
        env.set("FIREPASS_BASE_URL", &format!("{}/v1", server.base_url()));
        let provider = civ_ai::providers::FirepassKimiProvider::from_env().expect("construct");
        assert_eq!(
            provider.generate(&GenRequest::from_prompt("x")).await,
            Err(AiError::RateLimited)
        );

        // Malformed envelope -> InvalidResponse naming the envelope.
        let server = loopback::spawn(vec![loopback::ok(r#"{"unexpected":true}"#)]);
        env.set("FIREPASS_BASE_URL", &format!("{}/v1", server.base_url()));
        let provider = civ_ai::providers::FirepassKimiProvider::from_env().expect("construct");
        match provider.generate(&GenRequest::from_prompt("x")).await {
            Err(AiError::InvalidResponse(message)) => {
                assert!(message.contains("envelope"), "got: {message}");
            }
            other => panic!("expected InvalidResponse, got {other:?}"),
        }

        // Envelope without content -> InvalidResponse.
        let server = loopback::spawn(vec![loopback::ok(r#"{"choices":[]}"#)]);
        env.set("FIREPASS_BASE_URL", &format!("{}/v1", server.base_url()));
        let provider = civ_ai::providers::FirepassKimiProvider::from_env().expect("construct");
        match provider.generate(&GenRequest::from_prompt("x")).await {
            Err(AiError::InvalidResponse(message)) => {
                assert!(message.contains("missing chat completion content"), "got: {message}");
            }
            other => panic!("expected InvalidResponse, got {other:?}"),
        }
    }

    /// Covers FR-CIV-AI-004.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fr_civ_ai_004_embed_is_unsupported_for_cloud_provider() {
        let env = EnvGuard::new();
        env.set("KIMI_API_KEY", "test-cloud-key");
        env.set("FIREPASS_BASE_URL", &format!("{}/v1", loopback::dead_address()));
        let provider = civ_ai::providers::FirepassKimiProvider::from_env().expect("construct");

        let error = provider
            .embed(&EmbedRequest {
                texts: vec!["alpha".into()],
                input_snapshot_hash: [1u8; 32],
            })
            .await
            .expect_err("the cloud provider is generate-only");
        assert_eq!(error, AiError::Unsupported("firepass-kimi (cloud)".into()));
        assert!(!provider.capabilities().embed);
    }
}

// ===========================================================================
// FR-CIV-AI-005 — EmbedProvider: MiniLM embeddings, generation unsupported
// ===========================================================================

/// Covers FR-CIV-AI-005.
#[test]
fn fr_civ_ai_005_embed_provider_is_feature_gated_and_artifact_backed() {
    let manifest = crate_file("Cargo.toml");
    assert_eq!(manifest_value(&manifest, "embed"), "embed = [\"dep:fastembed\"]");
    assert!(
        is_feature_gated(&crate_file("src/providers/mod.rs"), "embed", "embed"),
        "EmbedProvider must stay behind the `embed` feature gate"
    );

    // The embed path never fetches weights: the loader only reads local files and
    // pulls in no HTTP/hub client.
    let source = crate_file("src/providers/embed.rs");
    for forbidden in ["hf_hub", "from_pretrained", "reqwest", "ureq", "http://", "https://"] {
        assert!(
            !source.contains(forbidden),
            "the embed provider must not contact the network (`{forbidden}` found)"
        );
    }
    assert!(
        source.contains("std::fs::read("),
        "artifact loading must read caller-owned local files"
    );
    assert!(source.contains("try_from_model_dir"));
}

/// Covers FR-CIV-AI-005.
#[tokio::test]
async fn fr_civ_ai_005_embed_requests_are_batched_and_order_sensitive() {
    // Port-level batching contract the EmbedProvider must satisfy: one vector per
    // input, in input order, with the batch shape preserved.
    let provider: Arc<dyn AiProvider> = Arc::new(DummyAiProvider);
    let request = EmbedRequest {
        texts: vec!["alpha".into(), "beta".into(), "gamma".into()],
        input_snapshot_hash: [9u8; 32],
    };
    let vectors = provider.embed(&request).await.expect("embed");
    assert_eq!(vectors.len(), request.texts.len(), "one vector per batched text");

    let reordered = EmbedRequest {
        texts: vec!["gamma".into(), "beta".into(), "alpha".into()],
        input_snapshot_hash: [9u8; 32],
    };
    let reordered_vectors = provider.embed(&reordered).await.expect("embed");
    assert_eq!(vectors[0], reordered_vectors[2], "vectors follow input order");
    assert_ne!(
        vectors[0], vectors[1],
        "distinct texts must not collapse onto one vector"
    );

    // Empty batches are legal and produce no vectors.
    let empty = provider
        .embed(&EmbedRequest {
            texts: Vec::new(),
            input_snapshot_hash: [0u8; 32],
        })
        .await
        .expect("embed");
    assert!(empty.is_empty());

    // EmbedRequest round-trips so cache keys stay comparable across processes.
    let encoded = serde_json::to_string(&request).expect("serialize");
    let decoded: EmbedRequest = serde_json::from_str(&encoded).expect("deserialize");
    assert_eq!(decoded, request);
}

#[cfg(feature = "embed")]
mod embed_provider {
    use super::*;

    /// Covers FR-CIV-AI-005.
    #[tokio::test]
    async fn fr_civ_ai_005_unloaded_provider_refuses_loudly_and_claims_no_capability() {
        let provider = civ_ai::providers::EmbedProvider::new("all-MiniLM-L6-v2");
        let capabilities = provider.capabilities();
        assert!(
            !capabilities.generate,
            "an embed provider must never advertise generation"
        );
        assert!(
            !capabilities.embed,
            "capabilities must reflect the unloaded artifact, not an optimistic yes"
        );
        assert!(!capabilities.cloud, "ONNX inference is local");
        assert_eq!(provider.model_id(), "all-MiniLM-L6-v2");
        assert_eq!(provider.model_version(), "fastembed-user-defined");

        match provider
            .embed(&EmbedRequest {
                texts: vec!["alpha".into()],
                input_snapshot_hash: [0u8; 32],
            })
            .await
        {
            Err(AiError::ModelMissing(message)) => {
                assert!(message.contains("all-MiniLM-L6-v2"), "got: {message}");
                assert!(message.contains("try_from_model_dir"), "got: {message}");
            }
            other => panic!("expected a loud ModelMissing, got {other:?}"),
        }
    }

    /// Covers FR-CIV-AI-005.
    #[tokio::test]
    async fn fr_civ_ai_005_generate_is_unsupported_for_embed_only_provider() {
        let provider = civ_ai::providers::EmbedProvider::new("all-MiniLM-L6-v2");
        let error = provider
            .generate(&GenRequest::from_prompt("write a legend"))
            .await
            .expect_err("embed-only providers must refuse generation");
        assert_eq!(error, AiError::Unsupported("embed-only".into()));
        assert!(error.to_string().contains("embed-only"));
    }

    /// Covers FR-CIV-AI-005.
    #[test]
    fn fr_civ_ai_005_model_load_names_each_missing_artifact_in_order() {
        let scratch = ScratchDir::new("fr005-artifacts");
        let dir = scratch.path();

        // Deterministic named missing-file chain: no download is ever attempted,
        // and the caller learns exactly which artifact is absent.
        let chain = [
            "tokenizer.json",
            "config.json",
            "special_tokens_map.json",
            "tokenizer_config.json",
            "model.onnx",
        ];
        for (index, missing) in chain.iter().enumerate() {
            match civ_ai::providers::EmbedProvider::try_from_model_dir(
                "all-MiniLM-L6-v2",
                dir,
                384,
            ) {
                Err(AiError::ModelMissing(message)) => {
                    assert!(
                        message.contains(missing),
                        "step {index} must name `{missing}`, got: {message}"
                    );
                    assert!(
                        message.contains("all-MiniLM-L6-v2"),
                        "the failure must name the model, got: {message}"
                    );
                    assert!(
                        message.contains(&dir.display().to_string()),
                        "the failure must name the directory, got: {message}"
                    );
                }
                other => panic!("step {index} expected ModelMissing, got {other:?}"),
            }
            // Satisfy the artifact we just learned about, with the same bytes
            // fastembed would reject later; loading must still not touch network.
            scratch.write(missing, "{}");
        }
    }

    /// Covers FR-CIV-AI-005.
    #[test]
    fn fr_civ_ai_005_zero_dimension_is_rejected_before_any_model_load() {
        let scratch = ScratchDir::new("fr005-zero-dim");
        // The directory is empty: a dimension guard must fire first, so the
        // failure is about the requested dimension and not about missing files.
        match civ_ai::providers::EmbedProvider::try_from_model_dir("m", scratch.path(), 0) {
            Err(AiError::InvalidResponse(message)) => {
                assert_eq!(message, "embedding dimension must be greater than zero");
            }
            other => panic!("expected InvalidResponse, got {other:?}"),
        }
    }

    /// Covers FR-CIV-AI-005.
    ///
    /// Requires real MiniLM artifacts: `CIVIS_EMBED_MODEL_DIR` (ONNX + tokenizer
    /// files) and, for the ONNX runtime, `ORT_DYLIB_PATH`. Skipped otherwise.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fr_civ_ai_005_env_model_returns_declared_dimension_vectors() {
        let Ok(dir) = std::env::var("CIVIS_EMBED_MODEL_DIR") else {
            eprintln!(
                "skipped fr_civ_ai_005_env_model_returns_declared_dimension_vectors: \
                 set CIVIS_EMBED_MODEL_DIR to a directory with model.onnx + tokenizer files"
            );
            return;
        };
        let dimension: usize = std::env::var("CIVIS_EMBED_DIM")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(384);

        let provider = civ_ai::providers::EmbedProvider::try_from_model_dir(
            "all-MiniLM-L6-v2",
            &dir,
            dimension,
        )
        .expect("load user-supplied embedding model");
        assert!(provider.capabilities().embed);

        let vectors = provider
            .embed(&EmbedRequest {
                texts: vec!["hello world".into(), "bonjour le monde".into()],
                input_snapshot_hash: [7u8; 32],
            })
            .await
            .expect("embed");
        assert_eq!(vectors.len(), 2);
        for vector in &vectors {
            assert_eq!(vector.len(), dimension, "MiniLM must emit {dimension}-dim vectors");
            assert!(vector.iter().all(|value| value.is_finite()));
        }
        assert_ne!(vectors[0], vectors[1], "distinct texts must embed distinctly");
    }
}

// ---------------------------------------------------------------------------
// Loopback HTTP server support (feature-gated providers only)
// ---------------------------------------------------------------------------

#[cfg(any(feature = "dev", feature = "cloud"))]
mod loopback {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::mpsc::{channel, Receiver};
    use std::thread;
    use std::time::Duration;

    /// A request as received by the loopback server.
    pub struct CapturedRequest {
        pub path: String,
        pub headers: String,
        pub body: String,
    }

    /// A canned HTTP response.
    #[derive(Clone)]
    pub struct Canned {
        status: u16,
        reason: &'static str,
        body: String,
    }

    pub fn ok(body: &str) -> Canned {
        Canned {
            status: 200,
            reason: "OK",
            body: body.to_string(),
        }
    }

    pub fn status(status: u16, reason: &'static str, body: &str) -> Canned {
        Canned {
            status,
            reason,
            body: body.to_string(),
        }
    }

    /// A bound-then-dropped address: connections are refused immediately.
    pub fn dead_address() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind probe port");
        let address = listener.local_addr().expect("probe address");
        drop(listener);
        format!("http://{address}")
    }

    /// Case-insensitive header lookup over a captured header block.
    pub fn header(headers: &str, name: &str) -> Option<String> {
        headers.lines().skip(1).find_map(|line| {
            let (key, value) = line.split_once(':')?;
            if key.trim().eq_ignore_ascii_case(name) {
                Some(value.trim().to_string())
            } else {
                None
            }
        })
    }

    pub struct Server {
        base_url: String,
        rx: Receiver<CapturedRequest>,
    }

    impl Server {
        pub fn base_url(&self) -> String {
            self.base_url.clone()
        }

        pub fn next_request(&self) -> CapturedRequest {
            self.rx
                .recv_timeout(Duration::from_secs(15))
                .expect("loopback server did not receive a request")
        }
    }

    /// Accept `responses.len()` connections on a loopback port, answering the
    /// i-th connection with the i-th canned response.
    pub fn spawn(responses: Vec<Canned>) -> Server {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback listener");
        let address = listener.local_addr().expect("loopback address");
        let (tx, rx) = channel();

        thread::spawn(move || {
            for response in responses {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let _ = stream.set_read_timeout(Some(Duration::from_secs(15)));
                let raw = read_request(&mut stream);
                let text = String::from_utf8_lossy(&raw).to_string();
                let (head, body) = text.split_once("\r\n\r\n").unwrap_or((text.as_str(), ""));
                let path = head
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or_default()
                    .to_string();
                let _ = tx.send(CapturedRequest {
                    path,
                    headers: head.to_string(),
                    body: body.to_string(),
                });
                let payload = format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response.status,
                    response.reason,
                    response.body.len(),
                    response.body
                );
                let _ = stream.write_all(payload.as_bytes());
                let _ = stream.flush();
            }
        });

        Server {
            base_url: format!("http://{address}"),
            rx,
        }
    }

    /// Read one HTTP request: headers, then `Content-Length` bytes of body.
    fn read_request(stream: &mut TcpStream) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            match stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(read) => buffer.extend_from_slice(&chunk[..read]),
                Err(_) => break,
            }
            if let Some(head_end) = find(&buffer, b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&buffer[..head_end]).to_string();
                let expected = content_length(&head).unwrap_or(0);
                if buffer.len() >= head_end + 4 + expected {
                    break;
                }
            }
        }
        buffer
    }

    fn content_length(head: &str) -> Option<usize> {
        header(head, "content-length").and_then(|value| value.parse().ok())
    }

    fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }
}
