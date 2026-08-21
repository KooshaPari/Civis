//! `EmbedProvider` — fastembed-rs / `ort` (MiniLM, 384-dim) (FR-CIV-AI-005).
//!
//! Drives culture/meme drift (§1.4) + log triage (§3). `generate` is
//! unsupported (loud error). Behind the `embed` feature.
//!
//! Model loading is explicit and artifact-backed: the provider never downloads
//! weights. `try_from_model_dir` expects the ONNX model plus the four
//! tokenizer/config JSON files required by fastembed. The ONNX Runtime shared
//! library must be discoverable through `ORT_DYLIB_PATH` when the `embed`
//! feature is enabled.

use crate::{AiError, AiProvider, Capabilities, EmbedRequest, GenOutput, GenRequest};
use std::path::Path;

#[cfg(feature = "embed")]
use std::sync::Mutex;

/// MiniLM-compatible embedding provider.
pub struct EmbedProvider {
    model_id: String,
    dimension: usize,
    #[cfg(feature = "embed")]
    model: Option<Mutex<fastembed::TextEmbedding>>,
}

impl EmbedProvider {
    /// Build with the embedding model id.
    #[must_use]
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            dimension: 384,
            #[cfg(feature = "embed")]
            model: None,
        }
    }

    /// Load a user-supplied fastembed model from `model_dir`.
    ///
    /// The directory must contain `model.onnx`, `tokenizer.json`,
    /// `config.json`, `special_tokens_map.json`, and `tokenizer_config.json`.
    /// No network access or implicit model download occurs.
    #[cfg(feature = "embed")]
    pub fn try_from_model_dir(
        model_id: impl Into<String>,
        model_dir: impl AsRef<Path>,
        dimension: usize,
    ) -> Result<Self, AiError> {
        if dimension == 0 {
            return Err(AiError::InvalidResponse(
                "embedding dimension must be greater than zero".into(),
            ));
        }
        let model_id = model_id.into();
        let dir = model_dir.as_ref();
        let read = |name: &str| {
            std::fs::read(dir.join(name)).map_err(|err| {
                AiError::ModelMissing(format!(
                    "embedding model '{}' missing {} in {}: {}",
                    model_id,
                    name,
                    dir.display(),
                    err
                ))
            })
        };
        let files = fastembed::TokenizerFiles {
            tokenizer_file: read("tokenizer.json")?,
            config_file: read("config.json")?,
            special_tokens_map_file: read("special_tokens_map.json")?,
            tokenizer_config_file: read("tokenizer_config.json")?,
        };
        let model = fastembed::UserDefinedEmbeddingModel::new(read("model.onnx")?, files)
            .with_pooling(fastembed::Pooling::Mean);
        let options = fastembed::InitOptionsUserDefined::new();
        let model =
            fastembed::TextEmbedding::try_new_from_user_defined(model, options).map_err(|err| {
                AiError::Unavailable(format!(
                    "failed to load embedding model '{}' from {}: {}",
                    model_id,
                    dir.display(),
                    err
                ))
            })?;
        Ok(Self {
            model_id,
            dimension,
            model: Some(Mutex::new(model)),
        })
    }
}

#[async_trait::async_trait]
impl AiProvider for EmbedProvider {
    async fn generate(&self, _req: &GenRequest) -> Result<GenOutput, AiError> {
        Err(AiError::Unsupported("embed-only".into()))
    }

    async fn embed(&self, req: &EmbedRequest) -> Result<Vec<Vec<f32>>, AiError> {
        #[cfg(feature = "embed")]
        if let Some(model) = &self.model {
            let mut model = model
                .lock()
                .map_err(|_| AiError::Unavailable("embedding model mutex poisoned".into()))?;
            let vectors = model.embed(&req.texts, None).map_err(|err| {
                AiError::Unavailable(format!(
                    "embedding inference failed for '{}': {}",
                    self.model_id, err
                ))
            })?;
            if vectors.iter().any(|vector| vector.len() != self.dimension) {
                return Err(AiError::InvalidResponse(format!(
                    "embedding model '{}' returned a vector with unexpected dimension (expected {})",
                    self.model_id, self.dimension
                )));
            }
            return Ok(vectors);
        }
        Err(AiError::ModelMissing(format!(
            "embedding model '{}' is not loaded; use try_from_model_dir with the embed feature",
            self.model_id
        )))
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn model_version(&self) -> &str {
        "fastembed-user-defined"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            generate: false,
            embed: self.model.is_some(),
            cloud: false,
        }
    }
}

#[cfg(all(test, feature = "embed"))]
mod tests {
    use super::EmbedProvider;

    #[test]
    fn missing_model_artifact_fails_loudly_without_download() {
        let result = EmbedProvider::try_from_model_dir(
            "all-MiniLM-L6-v2",
            "definitely-missing-civis-embed-model",
            384,
        );
        match result {
            Err(err) => {
                assert!(err.to_string().contains("embedding model"));
                assert!(err.to_string().contains("tokenizer.json"));
            }
            Ok(_) => panic!("missing local artifacts must fail"),
        }
    }

    /// Integration test: requires `CIVIS_EMBED_MODEL_DIR` containing
    /// model.onnx + tokenizer files. Skipped in CI.
    #[test]
    #[ignore]
    fn embed_roundtrip_with_local_model() {
        let dir = std::env::var("CIVIS_EMBED_MODEL_DIR")
            .expect("set CIVIS_EMBED_MODEL_DIR to a dir with ONNX + tokenizer files");
        let provider = EmbedProvider::try_from_model_dir("test-embed", &dir, 384)
            .expect("embed model load failed");
        assert!(provider.capabilities().embed);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let vectors = rt
            .block_on(provider.embed(&super::EmbedRequest {
                texts: vec!["hello world".into()],
            }))
            .expect("embed failed");
        assert_eq!(vectors.len(), 1);
        assert_eq!(vectors[0].len(), 384, "MiniLM must produce 384-dim vectors");
    }
}
