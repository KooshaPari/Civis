//! `OllamaDevProvider` — dev-only, OpenAI-compatible HTTP (FR-CIV-AI-003).
//!
//! Reuses the same chat-completions client shape as the cloud provider.
//! **Never** a shipping dependency — behind the `dev` feature only.
//!
//! ## P1 STUB
//! HTTP wiring lands alongside the cloud chat lift (design §3). This stub holds
//! the endpoint from config and loudly returns [`AiError::Unavailable`] until
//! wired; never silently selected in release config.
//!
//! TODO(P1.A5): wire reqwest chat-completions against the Ollama endpoint.

use crate::{AiError, AiProvider, Capabilities, EmbedRequest, GenOutput, GenRequest};

/// Dev-only Ollama provider (P1 stub; HTTP pending).
pub struct OllamaDevProvider {
    endpoint: String,
    model_id: String,
}

impl OllamaDevProvider {
    /// Build with the Ollama endpoint + model id.
    #[must_use]
    pub fn new(endpoint: impl Into<String>, model_id: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            model_id: model_id.into(),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for OllamaDevProvider {
    async fn generate(&self, _req: &GenRequest) -> Result<GenOutput, AiError> {
        // TODO(P1.A5): reqwest chat-completions against self.endpoint.
        Err(AiError::Unavailable(format!(
            "OllamaDevProvider HTTP not yet implemented (endpoint {})",
            self.endpoint
        )))
    }

    async fn embed(&self, _req: &EmbedRequest) -> Result<Vec<Vec<f32>>, AiError> {
        Err(AiError::Unsupported("ollama-dev".into()))
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn model_version(&self) -> &str {
        "dev"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            generate: true,
            embed: false,
            cloud: true,
        }
    }
}
