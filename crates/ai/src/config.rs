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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serializes tests that mutate process-global `CIVAI_*` env vars so they do
    /// not race under cargo's parallel test runner (which shares one process per
    /// test binary).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// FR-CIV-AI-010 — `AiConfig` defaults are stable and non-empty where required.
    #[test]
    fn defaults_are_stable() {
        let d = AiConfig::default();
        assert_eq!(d.narrator_model, "qwen2.5-1.5b-instruct");
        assert_eq!(d.embed_model, "all-MiniLM-L6-v2");
        assert_eq!(d.max_concurrent_gen, 2);
        assert_eq!(d.gen_token_budget, 600);
        assert!(!d.enable_cloud);
        assert_eq!(d.ollama_url, "http://localhost:11434");
        assert_eq!(d.local_model_path, None);
    }

    /// FR-CIV-AI-010 — `from_env` table: each env variable overrides its default.
    #[test]
    fn from_env_honors_each_key() {
        let _env = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let keys = [
            "CIVAI_LOCAL_MODEL_PATH",
            "CIVAI_NARRATOR_MODEL",
            "CIVAI_EMBED_MODEL",
            "CIVAI_MAX_CONCURRENT_GEN",
            "CIVAI_GEN_TOKEN_BUDGET",
            "CIVAI_ENABLE_CLOUD",
            "CIVAI_OLLAMA_URL",
        ];
        let saved: Vec<(&str, Option<String>)> =
            keys.iter().map(|k| (*k, std::env::var(k).ok())).collect();
        for k in &keys {
            std::env::remove_var(k);
        }

        // All missing -> defaults.
        let defaults = AiConfig::default();
        assert_eq!(AiConfig::from_env(), defaults);

        // Override each key in turn.
        std::env::set_var("CIVAI_NARRATOR_MODEL", "custom-narrator");
        assert_eq!(AiConfig::from_env().narrator_model, "custom-narrator");
        std::env::remove_var("CIVAI_NARRATOR_MODEL");

        std::env::set_var("CIVAI_EMBED_MODEL", "custom-embed");
        assert_eq!(AiConfig::from_env().embed_model, "custom-embed");
        std::env::remove_var("CIVAI_EMBED_MODEL");

        std::env::set_var("CIVAI_MAX_CONCURRENT_GEN", "7");
        assert_eq!(AiConfig::from_env().max_concurrent_gen, 7);
        std::env::remove_var("CIVAI_MAX_CONCURRENT_GEN");

        std::env::set_var("CIVAI_GEN_TOKEN_BUDGET", "1024");
        assert_eq!(AiConfig::from_env().gen_token_budget, 1024);
        std::env::remove_var("CIVAI_GEN_TOKEN_BUDGET");

        std::env::set_var("CIVAI_ENABLE_CLOUD", "1");
        assert!(AiConfig::from_env().enable_cloud);
        std::env::remove_var("CIVAI_ENABLE_CLOUD");

        std::env::set_var("CIVAI_OLLAMA_URL", "http://ollama.local:11434");
        assert_eq!(AiConfig::from_env().ollama_url, "http://ollama.local:11434");
        std::env::remove_var("CIVAI_OLLAMA_URL");

        std::env::set_var("CIVAI_LOCAL_MODEL_PATH", "/models/model.gguf");
        assert_eq!(
            AiConfig::from_env().local_model_path,
            Some("/models/model.gguf".to_string())
        );
        std::env::remove_var("CIVAI_LOCAL_MODEL_PATH");

        // Restore.
        for (k, v) in &saved {
            match v {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
    }

    /// FR-CIV-AI-010 — malformed integer env vars fall back to defaults.
    #[test]
    fn from_env_ignores_malformed_integers() {
        let _env = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let backup_max = std::env::var("CIVAI_MAX_CONCURRENT_GEN").ok();
        let backup_budget = std::env::var("CIVAI_GEN_TOKEN_BUDGET").ok();

        std::env::set_var("CIVAI_MAX_CONCURRENT_GEN", "not_a_number");
        std::env::set_var("CIVAI_GEN_TOKEN_BUDGET", "also_bad");
        let cfg = AiConfig::from_env();
        assert_eq!(cfg.max_concurrent_gen, 2);
        assert_eq!(cfg.gen_token_budget, 600);

        match backup_max {
            Some(v) => std::env::set_var("CIVAI_MAX_CONCURRENT_GEN", v),
            None => std::env::remove_var("CIVAI_MAX_CONCURRENT_GEN"),
        }
        match backup_budget {
            Some(v) => std::env::set_var("CIVAI_GEN_TOKEN_BUDGET", v),
            None => std::env::remove_var("CIVAI_GEN_TOKEN_BUDGET"),
        }
    }
}
