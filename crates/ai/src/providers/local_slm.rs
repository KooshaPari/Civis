//! `LocalSlmProvider` — in-process SLM (mistral.rs, GGUF Q4_K_M) (FR-CIV-AI-002).
//!
//! **DEFAULT in-game** generator behind the `local` feature.
//!
//! The provider is artifact-only: callers supply a local GGUF file and,
//! when needed, a local chat-template file. Construction loads the model once;
//! no remote model fetch is performed.

use crate::{AiError, AiProvider, Capabilities, EmbedRequest, GenOutput, GenRequest};
use mistralrs::{GgufModelBuilder, Model, RequestBuilder, TextMessageRole};
use std::path::{Path, PathBuf};

/// In-process local SLM provider loaded from a caller-owned GGUF artifact.
pub struct LocalSlmProvider {
    model_id: String,
    model_path: PathBuf,
    model: Option<std::sync::Arc<Model>>,
}

impl LocalSlmProvider {
    /// Compatibility constructor that defers loading and advertises no
    /// generation capability until [`Self::try_from_gguf`] succeeds.
    #[must_use]
    pub fn new(model_id: impl Into<String>, model_path: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            model_path: PathBuf::from(model_path.into()),
            model: None,
        }
    }

    /// Load one local GGUF file. This constructor never downloads artifacts.
    pub async fn try_from_gguf(
        model_id: impl Into<String>,
        model_dir: impl AsRef<Path>,
        gguf_filename: impl Into<String>,
        chat_template: Option<impl Into<String>>,
    ) -> Result<Self, AiError> {
        let model_id = model_id.into();
        let model_dir = model_dir.as_ref().to_path_buf();
        let gguf_filename = gguf_filename.into();
        let model_path = model_dir.join(&gguf_filename);
        if !model_path.is_file() {
            return Err(AiError::ModelMissing(model_path.display().to_string()));
        }

        let mut builder =
            GgufModelBuilder::new(model_dir.to_string_lossy().to_string(), vec![gguf_filename]);
        if let Some(chat_template) = chat_template {
            let template = chat_template.into();
            let template_path = model_dir.join(&template);
            if !template_path.is_file() && !template.trim_start().starts_with('{') {
                return Err(AiError::ModelMissing(template_path.display().to_string()));
            }
            builder = builder.with_chat_template(template);
        }

        let model = builder
            .build()
            .await
            .map_err(|err| AiError::Unavailable(format!("local GGUF load failed: {err}")))?;
        Ok(Self {
            model_id,
            model_path,
            model: Some(std::sync::Arc::new(model)),
        })
    }
}

#[async_trait::async_trait]
impl AiProvider for LocalSlmProvider {
    async fn generate(&self, req: &GenRequest) -> Result<GenOutput, AiError> {
        let model = self.model.as_deref().ok_or_else(|| {
            AiError::ModelMissing(format!(
                "local GGUF provider is not loaded: {}",
                self.model_path.display()
            ))
        })?;
        let request = RequestBuilder::new()
            .add_message(TextMessageRole::User, req.prompt.clone())
            .set_sampler_temperature(f64::from(req.temperature.max(0.0)))
            .set_sampler_max_len(req.max_tokens.max(1) as usize);
        let response = model
            .send_chat_request(request)
            .await
            .map_err(|err| AiError::Unavailable(format!("local GGUF inference failed: {err}")))?;
        let text = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .filter(|text| !text.is_empty())
            .ok_or_else(|| AiError::InvalidResponse("local GGUF returned empty text".into()))?;
        Ok(GenOutput::fresh(text))
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
            generate: self.model.is_some(),
            embed: false,
            cloud: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unloaded_compatibility_provider_does_not_claim_generation() {
        let provider = LocalSlmProvider::new("test", "missing.gguf");
        assert!(!provider.capabilities().generate);
    }

    #[tokio::test]
    async fn missing_gguf_fails_before_model_load() {
        let result = LocalSlmProvider::try_from_gguf(
            "test",
            std::env::temp_dir(),
            "missing-civis-model.gguf",
            None::<String>,
        )
        .await;
        assert!(matches!(result, Err(AiError::ModelMissing(_))));
    }
}
