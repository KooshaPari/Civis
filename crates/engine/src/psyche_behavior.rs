//! FR-CIV-PSYCHE behavior gap: psyche/emotion-driven behavioral choices.
//!
//! This module bridges the psyche read-API (existing mood, temperament, beliefs)
//! to concrete behavioral decisions. A sentient agent's emotional state now
//! directly influences action choices: fear triggers flee, contentment triggers
//! cooperation, anger triggers aggression.

use civ_agents::Psyche;
use serde::{Deserialize, Serialize};

/// Emotion-driven behavior choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmotionDrivenBehavior {
    /// Agent is fearful and will attempt to flee danger.
    Flee,
    /// Agent is content and seeks cooperation.
    Cooperate,
    /// Agent is angry and will aggress if provoked.
    Aggress,
    /// Agent is neutral; no strong emotion drives action.
    Neutral,
}

/// Derive emotion-driven behavior from psyche state.
///
/// This function reads the current mood (valence and arousal) and temperament
/// to determine which behavior an agent should exhibit:
///
/// - **Fear → Flee**: High arousal + low valence (threat perceived)
/// - **Content → Cooperate**: High valence + low arousal (safe, satisfied)
/// - **Anger → Aggress**: Low valence + high arousal (frustrated, agitated)
/// - **Neutral**: Balanced mood with low arousal
///
/// # Algorithm
///
/// 1. Compute a fear signal from threat perception (arousal) and misery (negative valence).
/// 2. Compute a contentment signal from positive valence and low arousal.
/// 3. Compute an anger signal from negative valence with high arousal and impulsivity.
/// 4. Return the dominant behavior based on the strongest signal.
#[must_use]
pub fn behavior_from_psyche(psyche: &Psyche) -> EmotionDrivenBehavior {
    let Psyche {
        mood,
        temperament,
        ..
    } = psyche;

    // Fear: high arousal + misery (low/negative valence)
    let fear_signal = mood.arousal * (1.0 - mood.valence.max(0.0));

    // Contentment: positive valence + low arousal
    let contentment_signal = mood.valence.max(0.0) * (1.0 - mood.arousal);

    // Anger: negative valence + high arousal + impulsivity (quick to escalate)
    let anger_impulse = (-mood.valence).max(0.0) * mood.arousal * temperament.impulsivity;

    // Determine dominant emotion
    let max_signal = [fear_signal, contentment_signal, anger_impulse]
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);

    // Threshold: if all signals are low, behavior is neutral
    if max_signal < 0.15 {
        return EmotionDrivenBehavior::Neutral;
    }

    // Return the dominant behavior
    if fear_signal >= contentment_signal && fear_signal >= anger_impulse {
        EmotionDrivenBehavior::Flee
    } else if contentment_signal >= fear_signal && contentment_signal >= anger_impulse {
        EmotionDrivenBehavior::Cooperate
    } else if anger_impulse >= fear_signal && anger_impulse >= contentment_signal {
        EmotionDrivenBehavior::Aggress
    } else {
        EmotionDrivenBehavior::Neutral
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_agents::Mood;

    fn make_psyche(valence: f32, arousal: f32, impulsivity: f32) -> Psyche {
        Psyche {
            drives: [0.5; 4],
            temperament: civ_agents::Temperament {
                reactivity: 0.5,
                sociability: 0.5,
                risk_tol: 0.5,
                impulsivity,
            },
            mood: Mood { valence, arousal },
            beliefs: [0.5; 4],
            maturity: 0.3,
        }
    }

    #[test]
    fn fearful_agent_flees() {
        // High arousal (threat) + low valence (misery) = fear
        let psyche = make_psyche(
            -0.8, // very miserable
            0.9,  // very aroused
            0.5,  // normal impulsivity
        );
        assert_eq!(
            behavior_from_psyche(&psyche),
            EmotionDrivenBehavior::Flee,
            "fearful agent should flee"
        );
    }

    #[test]
    fn content_agent_cooperates() {
        // High valence (happy) + low arousal (calm) = contentment
        let psyche = make_psyche(
            0.85, // very happy
            0.1,  // very calm
            0.5,  // normal impulsivity
        );
        assert_eq!(
            behavior_from_psyche(&psyche),
            EmotionDrivenBehavior::Cooperate,
            "content agent should cooperate"
        );
    }

    #[test]
    fn angry_impulsive_agent_aggresses() {
        // Low valence (frustrated) + high arousal + high impulsivity = anger/aggression
        let psyche = make_psyche(
            -0.7, // frustrated
            0.85, // highly aroused
            0.95, // very impulsive
        );
        assert_eq!(
            behavior_from_psyche(&psyche),
            EmotionDrivenBehavior::Aggress,
            "angry, impulsive agent should aggress"
        );
    }

    #[test]
    fn balanced_mood_is_neutral() {
        // Balanced, calm mood = neutral behavior
        let psyche = make_psyche(
            0.0,  // neutral valence
            0.05, // very calm
            0.5,  // normal impulsivity
        );
        assert_eq!(
            behavior_from_psyche(&psyche),
            EmotionDrivenBehavior::Neutral,
            "balanced, calm agent should be neutral"
        );
    }

    #[test]
    fn slightly_negative_aroused_is_still_fear() {
        // Negative valence + high arousal = fear (even if not maximally miserable)
        let psyche = make_psyche(
            -0.5, // moderately unhappy
            0.8,  // quite aroused
            0.5,
        );
        assert_eq!(
            behavior_from_psyche(&psyche),
            EmotionDrivenBehavior::Flee,
            "agent with threat + unhappiness should flee"
        );
    }

    #[test]
    fn mildly_happy_calm_cooperates() {
        // Even mild happiness + calmness triggers cooperation
        let psyche = make_psyche(
            0.35, // somewhat happy
            0.2,  // somewhat calm
            0.5,
        );
        assert_eq!(
            behavior_from_psyche(&psyche),
            EmotionDrivenBehavior::Cooperate,
            "mildly happy, calm agent should cooperate"
        );
    }
}
