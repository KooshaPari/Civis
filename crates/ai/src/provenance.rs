//! Provenance: `AiEvent` (generalizes `civ-research::LlmEvent`) + `ReplayMode`.
//!
//! The composite **cache key is reused verbatim** from `LlmEvent::cache_key`:
//! `prompt_hash ‖ input_snapshot_hash ‖ model_id ‖ model_version`
//! (FR-CIV-AI-007). `ReplayMode` + the replay-advance rule carry over unchanged
//! from ADR-006.

use crate::cache::AiCache;
use serde::{Deserialize, Serialize};

/// Compose the blake3-derived composite cache key shared by `AiEvent` and the
/// cache layer. Kept identical to `civ-research::LlmEvent::cache_key`.
#[must_use]
pub fn compose_cache_key(
    prompt_hash: &[u8; 32],
    input_snapshot_hash: &[u8; 32],
    model_id: &str,
    model_version: &str,
) -> Vec<u8> {
    let mut key = Vec::with_capacity(64 + model_id.len() + model_version.len());
    key.extend_from_slice(prompt_hash);
    key.extend_from_slice(input_snapshot_hash);
    key.extend_from_slice(model_id.as_bytes());
    key.extend_from_slice(model_version.as_bytes());
    key
}

/// Per-save progression mode (ADR-006). Cosmetic flavor need not be
/// replay-gated; any advisory/sim-affecting AI records an [`AiEvent`] and
/// honors these modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReplayMode {
    /// Replay refuses any [`AiEvent`].
    Canonical,
    /// Replay requires cache hits.
    Hybrid,
    /// Replay requires cache hits; AI may propose alt-physics/biology.
    Free,
}

/// Hash-keyed AI output recorded in the event log (generalizes `LlmEvent`).
///
/// Generic over the output payload `O` so a tech-card consumer can use
/// `AiEvent<TechCard>` while flavor uses `AiEvent<String>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiEvent<O> {
    /// RNG seed supplied to the model call (provenance only).
    pub seed: u64,
    /// blake3 of prompt template + variables.
    pub prompt_hash: [u8; 32],
    /// Provider model identifier.
    pub model_id: String,
    /// Provider model version.
    pub model_version: String,
    /// blake3 of the snapshot region the call observed.
    pub input_snapshot_hash: [u8; 32],
    /// blake3 of serialized output.
    pub output_hash: [u8; 32],
    /// Output emitted by the call.
    pub output: O,
    /// Simulation tick when the event was recorded.
    pub tick: u64,
}

impl<O> AiEvent<O> {
    /// Composite cache key — identical layout to `LlmEvent::cache_key`.
    #[must_use]
    pub fn cache_key(&self) -> Vec<u8> {
        compose_cache_key(
            &self.prompt_hash,
            &self.input_snapshot_hash,
            &self.model_id,
            &self.model_version,
        )
    }
}

/// Why replay refused to advance on an AI event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayRefusal {
    /// Canonical mode encountered an `AiEvent` in the log.
    CanonicalAiEvent,
    /// Hybrid/Free replay could not resolve the event from cache.
    HybridCacheMiss,
}

/// Outcome of attempting to apply an `AiEvent` during replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayAdvanceOutcome {
    /// Cache hit (Hybrid/Free) or live run — event may be applied.
    Advanced,
    /// Replay must halt until the log/cache is repaired.
    Refused(ReplayRefusal),
}

/// Apply ADR-006 replay rules for a single [`AiEvent`].
///
/// During live play (`is_replay == false`) all modes advance. During replay,
/// Canonical refuses every AI event; Hybrid/Free require a cache hit.
#[must_use]
pub fn replay_advance_ai_event<O, V>(
    mode: ReplayMode,
    cache: &AiCache<V>,
    event: &AiEvent<O>,
    is_replay: bool,
) -> ReplayAdvanceOutcome {
    if !is_replay {
        return ReplayAdvanceOutcome::Advanced;
    }
    match mode {
        ReplayMode::Canonical => ReplayAdvanceOutcome::Refused(ReplayRefusal::CanonicalAiEvent),
        ReplayMode::Hybrid | ReplayMode::Free => {
            if cache.contains_key(&event.cache_key()) {
                ReplayAdvanceOutcome::Advanced
            } else {
                ReplayAdvanceOutcome::Refused(ReplayRefusal::HybridCacheMiss)
            }
        }
    }
}
