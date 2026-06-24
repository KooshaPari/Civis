//! `DummyAiProvider` — deterministic, test-only (FR-CIV-AI-006).
//!
//! Mirrors `civ-research::DummyLlmClient`: stable output for the same input,
//! no weights required. Used by all feature unit tests. Supports both
//! `generate` and `embed` so consumers can exercise either path.

use crate::{AiError, AiProvider, Capabilities, EmbedRequest, GenOutput, GenRequest};

/// Deterministic provider for tests. Output is a pure function of the input.
#[derive(Debug, Default, Clone)]
pub struct DummyAiProvider;

impl DummyAiProvider {
    /// FNV-1a over the prompt + snapshot — stable across runs.
    fn hash_input(prompt: &str, snapshot: &[u8]) -> u64 {
        let mut state: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in prompt.as_bytes().iter().chain(snapshot.iter()) {
            state ^= u64::from(*byte);
            state = state.wrapping_mul(0x0100_0000_01b3);
        }
        state
    }

    fn derived_text(req: &GenRequest) -> String {
        let h = Self::hash_input(&req.prompt, &req.input_snapshot_hash);
        format!("dummy-generation-{h:016x}")
    }

    /// Synchronous generate for the sim hot path (engine tick, tests).
    #[must_use]
    pub fn generate_sync(&self, req: &GenRequest) -> GenOutput {
        GenOutput::fresh(Self::derived_text(req))
    }

    /// Deterministic 8-dim unit-ish vector per text (fixed seed by content).
    fn derived_vector(text: &str) -> Vec<f32> {
        let h = Self::hash_input(text, &[]);
        (0..8)
            .map(|i| {
                let bits = h.rotate_left(i * 8) & 0xff;
                (bits as f32) / 255.0
            })
            .collect()
    }
}

#[async_trait::async_trait]
impl AiProvider for DummyAiProvider {
    async fn generate(&self, req: &GenRequest) -> Result<GenOutput, AiError> {
        Ok(GenOutput::fresh(Self::derived_text(req)))
    }

    async fn embed(&self, req: &EmbedRequest) -> Result<Vec<Vec<f32>>, AiError> {
        Ok(req.texts.iter().map(|t| Self::derived_vector(t)).collect())
    }

    fn model_id(&self) -> &str {
        "dummy"
    }

    fn model_version(&self) -> &str {
        "0"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            generate: true,
            embed: true,
            cloud: false,
        }
    }
}
