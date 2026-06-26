//! FR-CIV-PSYCHE behavior integration tests.
//!
//! These tests verify that the psyche/emotion-driven behavior API
//! (gap closure: FR-CIV-PSYCHE) correctly maps agent emotional state
//! to concrete behavioral choices.
//!
//! Test coverage:
//! 1. A fearful agent (high arousal + low valence) returns Flee.
//! 2. A content agent (high valence + low arousal) returns Cooperate.
//! 3. An angry agent (low valence + high arousal + high impulsivity) returns Aggress.
//! 4. Integration: behavior choices propagate through emergence ticks.

use civ_agents::{Mood, Psyche, Temperament};
use civ_engine::{behavior_from_psyche, EmotionDrivenBehavior, Simulation};

/// Helper: construct a test psyche with given mood and impulsivity.
fn make_test_psyche(valence: f32, arousal: f32, impulsivity: f32) -> Psyche {
    Psyche {
        drives: [0.5; 4],
        temperament: Temperament {
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

/// FR-CIV-PSYCHE §1 — fearful agent flees.
///
/// A sentient agent perceiving threat (high arousal) and misery (low valence)
/// exhibits flee behavior. This ensures the fear → flee path is wired correctly.
#[test]
fn fr_psyche_behavior_fearful_agent_flees() {
    let psyche = make_test_psyche(
        -0.8, // highly miserable
        0.9,  // highly aroused (threat perceived)
        0.5,  // normal impulsivity
    );
    let behavior = behavior_from_psyche(&psyche);
    assert_eq!(
        behavior, EmotionDrivenBehavior::Flee,
        "fearful agent (high arousal + low valence) must flee"
    );
}

/// FR-CIV-PSYCHE §2 — content agent cooperates.
///
/// A sentient agent with positive affect (high valence) and calm mood
/// (low arousal) exhibits cooperation behavior. This ensures the
/// contentment → cooperate path is wired correctly.
#[test]
fn fr_psyche_behavior_content_agent_cooperates() {
    let psyche = make_test_psyche(
        0.85, // highly content
        0.1,  // calm
        0.5,  // normal impulsivity
    );
    let behavior = behavior_from_psyche(&psyche);
    assert_eq!(
        behavior, EmotionDrivenBehavior::Cooperate,
        "content agent (high valence + low arousal) must cooperate"
    );
}

/// FR-CIV-PSYCHE §3 — angry impulsive agent aggresses.
///
/// A sentient agent with negative affect (low valence), high arousal,
/// and high impulsivity exhibits aggression behavior. This ensures the
/// anger + impulsivity → aggress path is wired correctly.
#[test]
fn fr_psyche_behavior_angry_impulsive_agent_aggresses() {
    let psyche = make_test_psyche(
        -0.7, // frustrated
        0.85, // highly aroused
        0.95, // very impulsive
    );
    let behavior = behavior_from_psyche(&psyche);
    assert_eq!(
        behavior, EmotionDrivenBehavior::Aggress,
        "angry, impulsive agent (low valence + high arousal + high impulsivity) must aggress"
    );
}

/// FR-CIV-PSYCHE §4 — balanced mood remains neutral.
///
/// An agent with neutral mood and low arousal exhibits no strong emotional
/// behavior. This ensures agents only act when there is sufficient emotional
/// signal.
#[test]
fn fr_psyche_behavior_balanced_mood_is_neutral() {
    let psyche = make_test_psyche(
        0.0,  // neutral valence
        0.05, // very calm
        0.5,  // normal impulsivity
    );
    let behavior = behavior_from_psyche(&psyche);
    assert_eq!(
        behavior, EmotionDrivenBehavior::Neutral,
        "balanced, calm agent must exhibit neutral behavior"
    );
}

/// FR-CIV-PSYCHE §5 — integration with simulation emergence tick.
///
/// After running the simulation through emergence ticks, sentient agents
/// develop psyche states (mood, temperament), and those states should map
/// to behavioral choices. This test verifies the full pipeline:
/// DNA → psyche → behavior.
#[test]
fn fr_psyche_behavior_emergence_integration() {
    let mut sim = Simulation::with_seed(42);

    // Run the sim long enough for emergence to create psyche states
    // and sentience crossings (150 ticks > 3 sample boundaries).
    for _ in 0..150 {
        sim.tick();
    }

    // Collect all agents with psyche states
    let agents_with_psyche: Vec<_> = sim
        .all_agents()
        .iter()
        .filter_map(|agent| {
            sim.agent_psyche(agent.id)
                .map(|psyche| (agent.id, psyche))
        })
        .collect();

    // At least some agents should have developed psyche states by now
    assert!(
        !agents_with_psyche.is_empty(),
        "emergence should create psyche states in some agents after 150 ticks"
    );

    // Every agent with a psyche should map to a valid behavior
    for (agent_id, psyche) in agents_with_psyche {
        let behavior = behavior_from_psyche(&psyche);
        assert!(
            !matches!(behavior, EmotionDrivenBehavior::Neutral)
                || (psyche.mood.valence.abs() < 0.3 && psyche.mood.arousal < 0.3),
            "agent {}: behavior {:?} should match emotional state (valence={:.2}, arousal={:.2})",
            agent_id,
            behavior,
            psyche.mood.valence,
            psyche.mood.arousal
        );
    }
}

/// FR-CIV-PSYCHE §6 — low-impulsivity agent with anger remains calmer.
///
/// Impulsivity modulates anger expression. An agent with low impulsivity
/// but negative valence + arousal may not reach aggression threshold.
/// This ensures temperament shapes behavioral expression.
#[test]
fn fr_psyche_behavior_low_impulsivity_dampens_anger() {
    let psyche = make_test_psyche(
        -0.6, // somewhat frustrated
        0.7,  // somewhat aroused
        0.2,  // very low impulsivity (controlled)
    );
    let behavior = behavior_from_psyche(&psyche);
    // Low impulsivity reduces anger signal; might be flee or neutral
    assert!(
        matches!(behavior, EmotionDrivenBehavior::Flee | EmotionDrivenBehavior::Neutral),
        "low-impulsivity agent should not easily aggress even with negative valence"
    );
}
