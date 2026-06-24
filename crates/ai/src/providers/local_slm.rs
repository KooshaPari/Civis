//! `LocalSlmProvider` — in-process SLM (mistral.rs, GGUF Q4_K_M) (FR-CIV-AI-002).
//!
//! **DEFAULT in-game** generator when fully implemented. Behind the `local`
//! feature.
//!
//! ## P1 STUB
//! Full model loading (mistral.rs GGUF, llama.cpp escape hatch) lands in phase
//! P1.A2 — see `docs/design/civ-ai-crate.md` §10. This stub holds the model
//! path + id from config and advertises `generate` capability, but loudly
//! returns [`AiError::ModelMissing`] from `generate` until the loader is wired.
//! No silent "AI off" (`CLAUDE.md` failure stance).
//!
//! TODO(P1.A2): load GGUF via mistral.rs at construction; run inference here.

use crate::{AiError, AiProvider, Capabilities, EmbedRequest, GenOutput, GenRequest};

/// In-process local SLM provider (P1 stub; loader pending).
pub struct LocalSlmProvider {
    model_id: String,
    model_path: String,
}

impl LocalSlmProvider {
    /// Build from a model id + on-disk GGUF path. Preflight should have verified
    /// the path; construction itself does not load weights in P1.
    #[must_use]
    pub fn new(model_id: impl Into<String>, model_path: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            model_path: model_path.into(),
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for LocalSlmProvider {
    async fn generate(&self, _req: &GenRequest) -> Result<GenOutput, AiError> {
        // TODO(P1.A2): replace with mistral.rs GGUF inference.
        Err(AiError::ModelMissing(format!(
            "LocalSlmProvider loader not yet implemented for '{}' at {} (P1.A2)",
            self.model_id, self.model_path
        )))
    }

    async fn embed(&self, _req: &EmbedRequest) -> Result<Vec<Vec<f32>>, AiError> {
        Err(AiError::Unsupported("local-slm".into()))
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn model_version(&self) -> &str {
        "q4_k_m"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            generate: true,
            embed: false,
            cloud: false,
        }
    }
}
