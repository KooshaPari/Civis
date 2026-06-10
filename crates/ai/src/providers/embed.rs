//! `EmbedProvider` — fastembed-rs / `ort` (MiniLM, 384-dim) (FR-CIV-AI-005).
//!
//! Drives culture/meme drift (§1.4) + log triage (§3). `generate` is
//! unsupported (loud error). Behind the `embed` feature.
//!
//! ## P1 STUB
//! Real fastembed-rs/ort model loading lands in phase P1.A4 — see
//! `docs/design/civ-ai-crate.md` §10. This stub advertises `embed` capability
//! and loudly returns [`AiError::ModelMissing`] until the backend is wired.
//!
//! TODO(P1.A4): load MiniLM via fastembed-rs/ort; return 384-dim batches.

use crate::{AiError, AiProvider, Capabilities, EmbedRequest, GenOutput, GenRequest};

/// MiniLM embedding provider (P1 stub; backend pending).
pub struct EmbedProvider {
    model_id: String,
}

impl EmbedProvider {
    /// Build with the embedding model id.
    #[must_use]
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for EmbedProvider {
    async fn generate(&self, _req: &GenRequest) -> Result<GenOutput, AiError> {
        Err(AiError::Unsupported("embed-only".into()))
    }

    async fn embed(&self, _req: &EmbedRequest) -> Result<Vec<Vec<f32>>, AiError> {
        // TODO(P1.A4): replace with fastembed-rs/ort MiniLM inference.
        Err(AiError::ModelMissing(format!(
            "EmbedProvider backend not yet implemented for '{}' (P1.A4)",
            self.model_id
        )))
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn model_version(&self) -> &str {
        "384d"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            generate: false,
            embed: true,
            cloud: false,
        }
    }
}
