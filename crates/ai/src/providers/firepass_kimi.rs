//! `FirepassKimiProvider` — wraps `civ-research::FirepassKimiClient` as an
//! [`AiProvider`] (FR-CIV-AI-004). Cloud heavy-reasoning **fallback only**.
//!
//! Behind the `cloud` feature + `CIVAI_ENABLE_CLOUD=1`. The wrapped client
//! reuses the existing OpenAI-compatible chat-completions HTTP path and the
//! `KIMI_API_KEY` / `FIREPASS_BASE_URL` env config. Missing key → loud
//! [`AiError::Unavailable`] at construction / call site.
//!
//! Generic generation reuses the existing authenticated chat-completions path.

use crate::{AiError, AiProvider, Capabilities, EmbedRequest, GenOutput, GenRequest};

/// Cloud provider wrapping the existing Firepass/Kimi client.
pub struct FirepassKimiProvider {
    inner: civ_research::firepass::FirepassKimiClient,
}

impl FirepassKimiProvider {
    /// Build from env (`KIMI_API_KEY` / `FIREPASS_BASE_URL`).
    ///
    /// # Errors
    /// Returns [`AiError::Unavailable`] (loud) when `KIMI_API_KEY` is missing.
    pub fn from_env() -> Result<Self, AiError> {
        let inner = civ_research::firepass::FirepassKimiClient::from_env()
            .map_err(|_| AiError::Unavailable("KIMI_API_KEY missing or invalid".into()))?;
        Ok(Self { inner })
    }

    /// Access the wrapped client (for tech-card-shaped calls in `civ-research`).
    #[must_use]
    pub fn inner(&self) -> &civ_research::firepass::FirepassKimiClient {
        &self.inner
    }
}

#[async_trait::async_trait]
impl AiProvider for FirepassKimiProvider {
    async fn generate(&self, req: &GenRequest) -> Result<GenOutput, AiError> {
        let text = self
            .inner
            .generate(
                &req.prompt,
                req.max_tokens,
                req.temperature,
                req.json_schema.as_deref(),
            )
            .await
            .map_err(|error| match error {
                civ_research::LlmError::RateLimited => AiError::RateLimited,
                civ_research::LlmError::InvalidResponse(message) => {
                    AiError::InvalidResponse(message)
                }
                civ_research::LlmError::NetworkUnavailable => {
                    AiError::Unavailable("Firepass/Kimi request failed".into())
                }
            })?;
        Ok(GenOutput::fresh(text))
    }

    async fn embed(&self, _req: &EmbedRequest) -> Result<Vec<Vec<f32>>, AiError> {
        Err(AiError::Unsupported("firepass-kimi (cloud)".into()))
    }

    fn model_id(&self) -> &str {
        "kimi-k2.6-turbo"
    }

    fn model_version(&self) -> &str {
        "cloud"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            generate: true,
            embed: false,
            cloud: true,
        }
    }
}
