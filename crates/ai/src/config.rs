//! `.env`-driven config (FR-CIV-AI-010). No hardcoded paths/keys.
//!
//! All `CIVAI_*` keys are read from the process environment (populate via a
//! gitignored `.env`; see committed `.env.example`). Selection + budgets only;
//! provider construction lives in [`crate::registry`].

/// Resolved AI configuration. Built from the environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiConfig {
    /// GGUF path for `LocalSlmProvider` (required if local enabled).
    pub local_model_path: Option<String>,
    /// Model id for the legends/chatter narrator role.
    pub narrator_model: String,
    /// Embedding model id.
    pub embed_model: String,
    /// Hard cap on in-flight generations (VRAM/latency budget).
    pub max_concurrent_gen: usize,
    /// Default `max_tokens` ceiling.
    pub gen_token_budget: u32,
    /// Opt-in cloud fallback.
    pub enable_cloud: bool,
    /// Dev provider endpoint (dev only).
    pub ollama_url: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            local_model_path: None,
            narrator_model: "qwen2.5-1.5b-instruct".to_string(),
            embed_model: "all-MiniLM-L6-v2".to_string(),
            max_concurrent_gen: 2,
            gen_token_budget: 600,
            enable_cloud: false,
            ollama_url: "http://localhost:11434".to_string(),
        }
    }
}

impl AiConfig {
    /// Build from the process environment, falling back to defaults per key.
    #[must_use]
    pub fn from_env() -> Self {
        let d = Self::default();
        Self {
            local_model_path: std::env::var("CIVAI_LOCAL_MODEL_PATH").ok(),
            narrator_model: env_or("CIVAI_NARRATOR_MODEL", d.narrator_model),
            embed_model: env_or("CIVAI_EMBED_MODEL", d.embed_model),
            max_concurrent_gen: std::env::var("CIVAI_MAX_CONCURRENT_GEN")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(d.max_concurrent_gen),
            gen_token_budget: std::env::var("CIVAI_GEN_TOKEN_BUDGET")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(d.gen_token_budget),
            enable_cloud: matches!(std::env::var("CIVAI_ENABLE_CLOUD").as_deref(), Ok("1")),
            ollama_url: env_or("CIVAI_OLLAMA_URL", d.ollama_url),
        }
    }
}

fn env_or(key: &str, default: String) -> String {
    std::env::var(key).unwrap_or(default)
}
