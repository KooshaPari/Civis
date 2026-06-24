//! `FirepassKimiProvider` — wraps `civ-research::FirepassKimiClient` as an
//! [`AiProvider`] (FR-CIV-AI-004). Cloud heavy-reasoning **fallback only**.
//!
//! Behind the `cloud` feature + `CIVAI_ENABLE_CLOUD=1`. The wrapped client
//! reuses the existing OpenAI-compatible chat-completions HTTP path and the
//! `KIMI_API_KEY` / `FIREPASS_BASE_URL` env config. Missing key → loud
//! [`AiError::Unavailable`] at construction / call site.
//!
//! ## P1 note
//! `civ-research::FirepassKimiClient` currently exposes a tech-card-shaped
//! method (`propose_tech_card`). For P1 we wrap it for *availability + identity*
//! and route generic prose generation through the same HTTP client surface; the
//! generic chat method will be lifted into `civ-research` when `civ-research`
//! migrates to consume `civ-ai` (design §3). Until then, `generate` returns a
//! loud [`AiError::Unavailable`] describing the pending wiring rather than
//! silently degrading.

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
    async fn generate(&self, _req: &GenRequest) -> Result<GenOutput, AiError> {
        // TODO(S2.W3 follow-up): route generic prose through a chat method
        // lifted into civ-research when it migrates to consume civ-ai (§3).
        Err(AiError::Unavailable(
            "FirepassKimiProvider::generate pending civ-research chat lift (design §3)".into(),
        ))
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
