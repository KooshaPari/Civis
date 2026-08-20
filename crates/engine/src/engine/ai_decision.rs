//! AI decision-making logic: utility scoring, goal evaluation, mood
//! influence on decisions. Extracted from `engine.rs` in decomposition
//! pass 2 (Civis Engine Decomposition).

use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::engine::Simulation;
use crate::psyche_behavior::behavior_from_psyche;

// ---------------------------------------------------------------------------
// Types (moved from engine.rs)
// ---------------------------------------------------------------------------

/// Agent action choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentAction {
    Flee,
    Socialize,
    Work,
}

/// Emotion-driven behavior selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmotionDrivenBehavior {
    Flee,
    Cooperate,
    Aggress,
    Neutral,
}

/// Per-agent behavior selected from current psyche state (FR-CIV-PSYCHE).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PsycheDrivenBehavior {
    pub emotion: EmotionDrivenBehavior,
    pub action: AgentAction,
    pub tick: u64,
}

/// Map an [`EmotionDrivenBehavior`] to an [`AgentAction`].
#[must_use]
pub fn action_from_emotion_behavior(behavior: EmotionDrivenBehavior) -> AgentAction {
    match behavior {
        EmotionDrivenBehavior::Flee => AgentAction::Flee,
        EmotionDrivenBehavior::Cooperate => AgentAction::Socialize,
        EmotionDrivenBehavior::Aggress => AgentAction::Work,
        EmotionDrivenBehavior::Neutral => AgentAction::Work,
    }
}

// ---------------------------------------------------------------------------
// Simulation methods
// ---------------------------------------------------------------------------

impl Simulation {
    /// Per-faction isolation pressure — sum of social-tension terms
    /// (FR-CIV-PSYCHE-911).
    ///
    /// Stub: returns 0.0 until `last_tick_cluster_payoffs` schema is
    /// finalized.
    pub fn faction_isolation_pressure(&self, _faction: u32) -> f32 {
        0.0
    }

    /// Compute faction decisions for the current tick (FR-CIV-FACTION).
    ///
    /// Clears the per-faction intent sets, evaluates each faction, and
    /// applies the resulting intents.
    pub(crate) fn phase_faction_decisions(&mut self) {
        self.state
            .last_tick_faction_unrest_response_intents
            .clear();
        self.state.last_tick_faction_hostility_intents.clear();
        self.state
            .last_tick_faction_trade_open_intents
            .clear();
        for (faction_id, decision) in crate::faction_decisions::compute_faction_decisions(self) {
            match decision {
                crate::faction_decisions::FactionDecision::RaiseUnrestResponse => {
                    self.state
                        .last_tick_faction_unrest_response_intents
                        .insert(faction_id);
                }
                crate::faction_decisions::FactionDecision::FlagHostility => {
                    self.state
                        .last_tick_faction_hostility_intents
                        .insert(faction_id);
                }
                crate::faction_decisions::FactionDecision::FlagTradeOpen => {
                    self.state
                        .last_tick_faction_trade_open_intents
                        .insert(faction_id);
                }
                crate::faction_decisions::FactionDecision::Maintain => {}
            }
        }
        crate::faction_decisions::apply_faction_decision_intents(self);
    }

    /// Per-agent psyche-driven behavior evaluation (FR-CIV-PSYCHE).
    ///
    /// Queries every entity with a `Psyche` component, derives an
    /// `EmotionDrivenBehavior` via `behavior_from_psyche`, and writes
    /// the result as a `PsycheDrivenBehavior` component.
    pub(crate) fn phase_psyche_behavior(&mut self) {
        let tick = self.state.tick;
        let behaviors: Vec<(Entity, PsycheDrivenBehavior)> = self
            .world
            .query::<&civ_agents::Psyche>()
            .iter()
            .map(|(entity, psyche)| {
                let emotion = behavior_from_psyche(psyche);
                (
                    entity,
                    PsycheDrivenBehavior {
                        emotion,
                        action: action_from_emotion_behavior(emotion),
                        tick,
                    },
                )
            })
            .collect();

        for (entity, behavior) in behaviors {
            // insert overwrites an existing component, covering both update and
            // first-insert without a conflicting mutable get borrow (E0502).
            let _ = self.world.insert(entity, (behavior,));
        }
    }
}
