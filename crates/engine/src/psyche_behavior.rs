// TODO(cleanup-surgeon): stub module — `psyche_behavior` types were removed
// by an earlier lane. `engine.rs:75` still imports them. Restore the
// original or rewrite callers.

use crate::engine::EmotionDrivenBehavior;
use civ_agents::Psyche;

/// Map a `Psyche` snapshot to the agent's dominant `EmotionDrivenBehavior`.
/// Stub: returns `Neutral` until the real mapping is restored.
#[must_use]
pub fn behavior_from_psyche(_psyche: &Psyche) -> EmotionDrivenBehavior {
    EmotionDrivenBehavior::Neutral
}
