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

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Minimum mood value.
pub const MOOD_MIN: f32 = 0.0;
/// Maximum mood value.
pub const MOOD_MAX: f32 = 1.0;

/// Detailed psyche state for a civilian.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PsycheState {
    pub mood: f32,
    pub anxiety: f32,
    pub ambition: f32,
    pub loyalty: f32,
    pub curiosity: f32,
    pub last_tick: u64,
}

impl Default for PsycheState {
    fn default() -> Self {
        Self {
            mood: 0.5,
            anxiety: 0.2,
            ambition: 0.5,
            loyalty: 0.5,
            curiosity: 0.5,
            last_tick: 0,
        }
    }
}

/// A belief held by a civilian.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub statement: String,
    pub confidence: f32,
    pub source: String,
    pub formed_tick: u64,
}

impl Default for Belief {
    fn default() -> Self {
        Self {
            statement: String::new(),
            confidence: 0.0,
            source: String::new(),
            formed_tick: 0,
        }
    }
}

/// Update mood based on food availability, safety, and social support.
#[must_use]
pub fn update_mood(
    psyche: &PsycheState,
    food_availability: f32,
    safety: f32,
    social_support: f32,
) -> PsycheState {
    let target_mood =
        (food_availability * 0.4 + safety * 0.3 + social_support * 0.3).clamp(MOOD_MIN, MOOD_MAX);
    let new_mood = (psyche.mood * 0.9 + target_mood * 0.1).clamp(MOOD_MIN, MOOD_MAX);
    PsycheState {
        mood: new_mood,
        ..psyche.clone()
    }
}

/// Form a belief from an observation at a given tick.
#[must_use]
pub fn form_belief(psyche: &PsycheState, observation: &str, tick: u64) -> Belief {
    let confidence = (psyche.mood * 0.5 + 0.3).clamp(0.0, 1.0);
    Belief {
        statement: observation.to_string(),
        confidence,
        source: "direct_observation".to_string(),
        formed_tick: tick,
    }
}

/// Check consistency between two beliefs using Jaccard word similarity.
#[must_use]
pub fn check_belief_consistency(a: &Belief, b: &Belief) -> f32 {
    let words_a: std::collections::HashSet<&str> = a.statement.split_whitespace().collect();
    let words_b: std::collections::HashSet<&str> = b.statement.split_whitespace().collect();
    if words_a.is_empty() && words_b.is_empty() {
        return 1.0;
    }
    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Advance psyche state by one tick: decay anxiety, adjust loyalty.
#[must_use]
pub fn tick_psychology(psyche: &PsycheState, tick: u64) -> PsycheState {
    let new_anxiety = (psyche.anxiety - 0.005).max(0.0);
    let loyalty_drift = if psyche.mood > 0.7 {
        0.005
    } else if psyche.mood < 0.3 {
        -0.005
    } else {
        0.0
    };
    let new_loyalty = (psyche.loyalty + loyalty_drift).clamp(MOOD_MIN, MOOD_MAX);
    PsycheState {
        anxiety: new_anxiety,
        loyalty: new_loyalty,
        last_tick: tick,
        ..psyche.clone()
    }
}

/// Create a batch of civilian psyches with deterministic seeds.
#[must_use]
pub fn create_civilian_psyches(count: u32, base_seed: u64) -> HashMap<u64, PsycheState> {
    let mut psyches = HashMap::with_capacity(count as usize);
    for i in 0..count {
        let mut hasher = DefaultHasher::new();
        base_seed.hash(&mut hasher);
        i.hash(&mut hasher);
        let hash = hasher.finish();
        let mood = (hash as f32 / u64::MAX as f32 * 0.8 + 0.1).clamp(MOOD_MIN, MOOD_MAX);
        let anxiety = ((hash >> 8) as f32 / u64::MAX as f32 * 0.4).clamp(MOOD_MIN, MOOD_MAX);
        let ambition = ((hash >> 16) as f32 / u64::MAX as f32).clamp(MOOD_MIN, MOOD_MAX);
        let loyalty = ((hash >> 24) as f32 / u64::MAX as f32).clamp(MOOD_MIN, MOOD_MAX);
        let curiosity = ((hash >> 32) as f32 / u64::MAX as f32).clamp(MOOD_MIN, MOOD_MAX);
        psyches.insert(
            i as u64,
            PsycheState {
                mood,
                anxiety,
                ambition,
                loyalty,
                curiosity,
                last_tick: 0,
            },
        );
    }
    psyches
}

#[cfg(test)]
mod psyche_extended_tests {
    use super::*;

    #[test]
    fn psyche_state_default_values() {
        let p = PsycheState::default();
        assert_eq!(p.mood, 0.5);
        assert_eq!(p.anxiety, 0.2);
        assert_eq!(p.ambition, 0.5);
        assert_eq!(p.loyalty, 0.5);
        assert_eq!(p.curiosity, 0.5);
    }

    #[test]
    fn update_mood_improves_with_resources() {
        let p = PsycheState::default();
        let updated = update_mood(&p, 1.0, 1.0, 1.0);
        assert!(updated.mood > p.mood);
    }

    #[test]
    fn update_mood_decreases_without_resources() {
        let p = PsycheState {
            mood: 0.8,
            ..PsycheState::default()
        };
        let updated = update_mood(&p, 0.0, 0.0, 0.0);
        assert!(updated.mood < p.mood);
    }

    #[test]
    fn update_mood_clamped() {
        let p = PsycheState {
            mood: 0.99,
            ..PsycheState::default()
        };
        let updated = update_mood(&p, 1.0, 1.0, 1.0);
        assert!(updated.mood <= MOOD_MAX);
    }

    #[test]
    fn form_belief_basic() {
        let p = PsycheState::default();
        let belief = form_belief(&p, "the sky is blue", 10);
        assert_eq!(belief.statement, "the sky is blue");
        assert_eq!(belief.formed_tick, 10);
        assert!(belief.confidence > 0.0);
    }

    #[test]
    fn check_belief_consistency_identical() {
        let a = Belief {
            statement: "hello world".into(),
            ..Belief::default()
        };
        assert!((check_belief_consistency(&a, &a) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn check_belief_consistency_disjoint() {
        let a = Belief {
            statement: "foo bar".into(),
            ..Belief::default()
        };
        let b = Belief {
            statement: "baz qux".into(),
            ..Belief::default()
        };
        assert!((check_belief_consistency(&a, &b) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn check_belief_consistency_partial() {
        let a = Belief {
            statement: "hello world".into(),
            ..Belief::default()
        };
        let b = Belief {
            statement: "hello there".into(),
            ..Belief::default()
        };
        let score = check_belief_consistency(&a, &b);
        assert!(score > 0.0 && score < 1.0);
    }

    #[test]
    fn check_belief_consistency_empty() {
        let a = Belief::default();
        let b = Belief::default();
        assert!((check_belief_consistency(&a, &b) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_psychology_decays_anxiety() {
        let p = PsycheState {
            anxiety: 0.5,
            ..PsycheState::default()
        };
        let ticked = tick_psychology(&p, 10);
        assert!(ticked.anxiety < p.anxiety);
    }

    #[test]
    fn tick_psychology_loyalty_boosts_on_high_mood() {
        let p = PsycheState {
            mood: 0.9,
            loyalty: 0.5,
            ..PsycheState::default()
        };
        let ticked = tick_psychology(&p, 10);
        assert!(ticked.loyalty > p.loyalty);
    }

    #[test]
    fn tick_psychology_loyalty_drops_on_low_mood() {
        let p = PsycheState {
            mood: 0.1,
            loyalty: 0.5,
            ..PsycheState::default()
        };
        let ticked = tick_psychology(&p, 10);
        assert!(ticked.loyalty < p.loyalty);
    }

    #[test]
    fn create_civilian_psyches_count() {
        let psyches = create_civilian_psyches(100, 42);
        assert_eq!(psyches.len(), 100);
    }

    #[test]
    fn create_civilian_psyches_deterministic() {
        let a = create_civilian_psyches(10, 42);
        let b = create_civilian_psyches(10, 42);
        for key in a.keys() {
            assert_eq!(a[key].mood, b[key].mood);
        }
    }
}
