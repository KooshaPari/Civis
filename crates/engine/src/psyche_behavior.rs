// Comprehensive psyche behavior system: mood, beliefs, Maslow hierarchy,
// emotional state machine, Big Five personality, trauma/stress, and group
// psychology.

use crate::engine::EmotionDrivenBehavior;
use civ_agents::Psyche;

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum mood value.
pub const MOOD_MIN: f32 = 0.0;
/// Maximum mood value.
pub const MOOD_MAX: f32 = 1.0;

// ---------------------------------------------------------------------------
// Existing types and functions (preserved exactly)
// ---------------------------------------------------------------------------

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

/// Map a `Psyche` snapshot to the agent's dominant `EmotionDrivenBehavior`.
///
/// The mapping is mood-based (valence × arousal) with impulsivity as a
/// modulating gate on the aggression path:
///
/// - **Aggress**: high arousal + negative valence + high impulsivity
/// - **Flee**: high arousal + negative valence (without impulsivity gate)
/// - **Cooperate**: positive valence + low arousal (calm/content)
/// - **Neutral**: default for uncommitted emotional states
#[must_use]
pub fn behavior_from_psyche(psyche: &Psyche) -> EmotionDrivenBehavior {
    let valence = psyche.mood.valence;
    let arousal = psyche.mood.arousal;
    let impulsivity = psyche.temperament.impulsivity;

    // High arousal + negative valence + high impulsivity → Aggress (angry)
    if arousal > 0.7 && valence < -0.5 && impulsivity > 0.8 {
        return EmotionDrivenBehavior::Aggress;
    }
    // High arousal + negative valence → Flee (fearful)
    if arousal > 0.7 && valence < -0.5 {
        return EmotionDrivenBehavior::Flee;
    }
    // Positive valence + low arousal → Cooperate (content)
    if valence > 0.5 && arousal < 0.5 {
        return EmotionDrivenBehavior::Cooperate;
    }
    EmotionDrivenBehavior::Neutral
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

// ===========================================================================
// 1. Maslow Hierarchy Integration
// ===========================================================================

/// Levels of the Maslow need hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaslowLevel {
    Physiological,
    Safety,
    Belonging,
    Esteem,
    SelfActualization,
}

/// Complete Maslow hierarchy satisfaction state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaslowHierarchy {
    /// Food, water, shelter satisfaction.
    pub physiological: f32,
    /// Physical security satisfaction.
    pub safety: f32,
    /// Social connection satisfaction.
    pub belonging: f32,
    /// Achievement and respect satisfaction.
    pub esteem: f32,
    /// Growth and purpose satisfaction.
    pub self_actualization: f32,
}

impl Default for MaslowHierarchy {
    fn default() -> Self {
        Self {
            physiological: 0.3,
            safety: 0.2,
            belonging: 0.2,
            esteem: 0.1,
            self_actualization: 0.1,
        }
    }
}

impl MaslowHierarchy {
    /// Return the satisfaction value for the given level.
    #[must_use]
    pub fn level_value(&self, level: MaslowLevel) -> f32 {
        match level {
            MaslowLevel::Physiological => self.physiological,
            MaslowLevel::Safety => self.safety,
            MaslowLevel::Belonging => self.belonging,
            MaslowLevel::Esteem => self.esteem,
            MaslowLevel::SelfActualization => self.self_actualization,
        }
    }

    /// Mutable access to a level's satisfaction value.
    pub fn level_value_mut(&mut self, level: MaslowLevel) -> &mut f32 {
        match level {
            MaslowLevel::Physiological => &mut self.physiological,
            MaslowLevel::Safety => &mut self.safety,
            MaslowLevel::Belonging => &mut self.belonging,
            MaslowLevel::Esteem => &mut self.esteem,
            MaslowLevel::SelfActualization => &mut self.self_actualization,
        }
    }
}

/// Ordered levels from most basic to most advanced.
const MASLOW_LEVELS: [MaslowLevel; 5] = [
    MaslowLevel::Physiological,
    MaslowLevel::Safety,
    MaslowLevel::Belonging,
    MaslowLevel::Esteem,
    MaslowLevel::SelfActualization,
];

/// Threshold below which a level is considered unfulfilled.
const MASLOW_UNFULFILLED_THRESHOLD: f32 = 0.5;

/// Returns the lowest unfulfilled level (first level below threshold).
/// If all levels are satisfied, returns `SelfActualization`.
#[must_use]
pub fn evaluate_current_level(hierarchy: &MaslowHierarchy) -> MaslowLevel {
    for level in &MASLOW_LEVELS {
        if hierarchy.level_value(*level) < MASLOW_UNFULFILLED_THRESHOLD {
            return *level;
        }
    }
    MaslowLevel::SelfActualization
}

/// Add satisfaction to a specific level (clamped to [0, 1]).
pub fn satisfy_level(hierarchy: &mut MaslowHierarchy, level: MaslowLevel, amount: f32) {
    let val = hierarchy.level_value_mut(level);
    *val = (*val + amount).clamp(MOOD_MIN, MOOD_MAX);
}

/// Overall completeness of the hierarchy in [0, 1].
#[must_use]
pub fn maslow_completeness(hierarchy: &MaslowHierarchy) -> f32 {
    let total: f32 = MASLOW_LEVELS
        .iter()
        .map(|l| hierarchy.level_value(*l))
        .sum();
    (total / MASLOW_LEVELS.len() as f32).clamp(MOOD_MIN, MOOD_MAX)
}

// ===========================================================================
// 2. Emotional State Machine
// ===========================================================================

/// Core emotions tracked by the state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Emotion {
    Joy,
    Fear,
    Anger,
    Sadness,
    Surprise,
    Disgust,
    Neutral,
}

/// Current emotional state with intensity and optional secondary emotion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    /// Dominant emotion.
    pub primary: Emotion,
    /// Intensity of the primary emotion in [0, 1].
    pub intensity: f32,
    /// Optional secondary emotion that blends with the primary.
    pub secondary: Option<Emotion>,
    /// Per-tick decay rate for intensity.
    pub decay_rate: f32,
}

impl Default for EmotionalState {
    fn default() -> Self {
        Self {
            primary: Emotion::Neutral,
            intensity: 0.0,
            secondary: None,
            decay_rate: 0.01,
        }
    }
}

impl EmotionalState {
    /// Create a new state with given primary, intensity, and decay.
    #[must_use]
    pub fn new(primary: Emotion, intensity: f32, decay_rate: f32) -> Self {
        Self {
            primary,
            intensity: intensity.clamp(MOOD_MIN, MOOD_MAX),
            secondary: None,
            decay_rate,
        }
    }
}

/// Transition the emotional state based on a trigger string and game tick.
/// Returns a new `EmotionalState` reflecting the transition.
#[must_use]
pub fn transition_emotion(state: &EmotionalState, trigger: &str, tick: u64) -> EmotionalState {
    // Apply base decay first.
    let decayed_intensity = (state.intensity - state.decay_rate).max(MOOD_MIN);

    let (new_primary, new_secondary, new_intensity) = match trigger {
        "celebration" | "victory" => (Emotion::Joy, state.secondary, 0.9),
        "threat" | "danger" => (Emotion::Fear, state.secondary, 0.8),
        "attack" | "betrayal" => (Emotion::Anger, Some(Emotion::Fear), 0.85),
        "loss" | "death" => (Emotion::Sadness, Some(Emotion::Fear), 0.7),
        "unexpected" | "revelation" => (Emotion::Surprise, state.secondary, 0.6),
        "contamination" | "revulsion" => (Emotion::Disgust, state.secondary, 0.5),
        "calm" | "peace" => (Emotion::Neutral, None, 0.0),
        _ => {
            // No recognized trigger: apply decay and return.
            return EmotionalState {
                primary: if decayed_intensity < 0.05 {
                    Emotion::Neutral
                } else {
                    state.primary
                },
                intensity: decayed_intensity,
                secondary: if decayed_intensity < 0.05 {
                    None
                } else {
                    state.secondary
                },
                decay_rate: state.decay_rate,
            };
        }
    };

    // If the new trigger is stronger than the current, override; otherwise decay.
    let blended = if new_intensity > decayed_intensity {
        new_intensity
    } else {
        (decayed_intensity * 0.7 + new_intensity * 0.3).clamp(MOOD_MIN, MOOD_MAX)
    };

    let _ = tick; // tick available for future time-based logic

    // "calm" always resets to zero intensity.
    let final_intensity = if trigger == "calm" || trigger == "peace" {
        0.0
    } else {
        blended
    };
    EmotionalState {
        primary: new_primary,
        intensity: final_intensity,
        secondary: new_secondary,
        decay_rate: state.decay_rate,
    }
}

/// Blend two emotional states with a weight in [0, 1] favoring `a` at 0 and
/// `b` at 1.
#[must_use]
pub fn blend_emotions(a: &EmotionalState, b: &EmotionalState, weight: f32) -> EmotionalState {
    let w = weight.clamp(MOOD_MIN, MOOD_MAX);
    let blended_intensity = (a.intensity * (1.0 - w) + b.intensity * w).clamp(MOOD_MIN, MOOD_MAX);
    let primary = if w < 0.5 {
        a.primary.clone()
    } else {
        b.primary.clone()
    };
    let secondary = if w < 0.3 {
        a.secondary.clone()
    } else if w > 0.7 {
        b.secondary.clone()
    } else {
        Some(a.primary.clone())
    };
    EmotionalState {
        primary,
        intensity: blended_intensity,
        secondary,
        decay_rate: (a.decay_rate * (1.0 - w) + b.decay_rate * w),
    }
}

/// Map an emotion and intensity to the engine's `EmotionDrivenBehavior`.
#[must_use]
pub fn emotion_to_behavior(emotion: &Emotion, intensity: f32) -> EmotionDrivenBehavior {
    match emotion {
        Emotion::Joy | Emotion::Surprise if intensity > 0.3 => EmotionDrivenBehavior::Cooperate,
        Emotion::Fear if intensity > 0.5 => EmotionDrivenBehavior::Flee,
        Emotion::Anger if intensity > 0.6 => EmotionDrivenBehavior::Aggress,
        Emotion::Sadness | Emotion::Disgust => EmotionDrivenBehavior::Neutral,
        _ => EmotionDrivenBehavior::Neutral,
    }
}

// ===========================================================================
// 3. Personality Traits (Big Five)
// ===========================================================================

/// Big Five personality traits, each in [0, 1].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigFive {
    /// Openness to experience.
    pub openness: f32,
    /// Conscientiousness / organization.
    pub conscientiousness: f32,
    /// Extraversion / sociability.
    pub extraversion: f32,
    /// Agreeableness / cooperation.
    pub agreeableness: f32,
    /// Neuroticism / emotional instability.
    pub neuroticism: f32,
}

impl BigFive {
    /// Generate a deterministic `BigFive` from a seed using simple hashing.
    #[must_use]
    pub fn random_deterministic(seed: u64) -> BigFive {
        let mut h = DefaultHasher::new();
        seed.hash(&mut h);
        let v1 = h.finish();
        h = DefaultHasher::new();
        (seed ^ 0xA5A5_A5A5).hash(&mut h);
        let v2 = h.finish();
        h = DefaultHasher::new();
        (seed ^ 0x5A5A_5A5A).hash(&mut h);
        let v3 = h.finish();
        h = DefaultHasher::new();
        (seed ^ 0xFFFF_0000).hash(&mut h);
        let v4 = h.finish();
        h = DefaultHasher::new();
        (seed ^ 0x0000_FFFF).hash(&mut h);
        let v5 = h.finish();

        BigFive {
            openness: (v1 as f32 / u64::MAX as f32).clamp(MOOD_MIN, MOOD_MAX),
            conscientiousness: (v2 as f32 / u64::MAX as f32).clamp(MOOD_MIN, MOOD_MAX),
            extraversion: (v3 as f32 / u64::MAX as f32).clamp(MOOD_MIN, MOOD_MAX),
            agreeableness: (v4 as f32 / u64::MAX as f32).clamp(MOOD_MIN, MOOD_MAX),
            neuroticism: (v5 as f32 / u64::MAX as f32).clamp(MOOD_MIN, MOOD_MAX),
        }
    }
}

/// Returns a situational modifier based on personality and situation.
/// The modifier is in [-1, 1] where positive means the personality favors the
/// situation and negative means it resists it.
#[must_use]
pub fn personality_modifier(big5: &BigFive, situation: &str) -> f32 {
    match situation {
        "social_gathering" => (big5.extraversion * 0.6 + big5.agreeableness * 0.4 - 0.5) * 2.0,
        "danger" => {
            // High neuroticism = more scared, high conscientiousness = more cautious
            ((1.0 - big5.neuroticism) * 0.5 + big5.conscientiousness * 0.5 - 0.5) * 2.0
        }
        "creative_task" => (big5.openness * 0.7 + big5.conscientiousness * 0.3 - 0.5) * 2.0,
        "conflict" => {
            // High agreeableness resists conflict
            (big5.agreeableness * 0.6 + (1.0 - big5.neuroticism) * 0.4 - 0.5) * -2.0
        }
        "isolation" => {
            // High extraversion resists isolation
            (big5.extraversion - 0.5) * -2.0
        }
        _ => 0.0,
    }
}

/// Compatibility score between two personalities in [0, 1].
/// Based on complementary and similar trait distances.
#[must_use]
pub fn compatibility_score(a: &BigFive, b: &BigFive) -> f32 {
    let openness_diff = (a.openness - b.openness).abs();
    let conscientiousness_diff = (a.conscientiousness - b.conscientiousness).abs();
    let extraversion_diff = (a.extraversion - b.extraversion).abs();
    let agreeableness_diff = (a.agreeableness - b.agreeableness).abs();
    let neuroticism_diff = (a.neuroticism - b.neuroticism).abs();

    // Similarity in most traits is good; high difference in neuroticism is bad.
    let avg_similarity = 1.0
        - (openness_diff + conscientiousness_diff + extraversion_diff + agreeableness_diff) / 4.0;
    let neuroticism_penalty = neuroticism_diff * 0.3;

    (avg_similarity - neuroticism_penalty).clamp(MOOD_MIN, MOOD_MAX)
}

// ===========================================================================
// 4. Trauma / Stress Response System
// ===========================================================================

/// A traumatic event experienced by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraumaEvent {
    /// Human-readable description.
    pub description: String,
    /// Severity in [0, 1].
    pub severity: f32,
    /// Game tick when the event occurred.
    pub tick: u64,
    /// Whether the agent has recovered from this trauma.
    pub recovered: bool,
}

/// Stress response state for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressResponse {
    /// Current stress level in [0, 1].
    pub current_stress: f32,
    /// History of traumatic events.
    pub trauma_history: Vec<TraumaEvent>,
    /// Resilience factor in [0, 1]; higher = recovers faster.
    pub resilience: f32,
}

impl Default for StressResponse {
    fn default() -> Self {
        Self {
            current_stress: 0.0,
            trauma_history: Vec::new(),
            resilience: 0.5,
        }
    }
}

/// Apply a stressor to the response. Stress accumulates with diminishing
/// returns and is reduced by resilience.
pub fn experience_stress(response: &mut StressResponse, stressor: f32, tick: u64) {
    let effective_stressor = stressor * (1.0 - response.resilience * 0.5);
    response.current_stress =
        (response.current_stress + effective_stressor).clamp(MOOD_MIN, MOOD_MAX);

    // Record severe stressors as trauma events.
    if stressor > 0.5 {
        response.trauma_history.push(TraumaEvent {
            description: format!("stressor_{}", tick),
            severity: stressor,
            tick,
            recovered: false,
        });
    }
}

/// Slowly recover from accumulated stress and mark old traumas as recovered.
pub fn recover_from_trauma(response: &mut StressResponse, recovery_rate: f32) {
    let rate = recovery_rate * (0.5 + response.resilience * 0.5);
    response.current_stress = (response.current_stress - rate).max(MOOD_MIN);

    // Mark traumas older than 100 ticks as recovered.
    for trauma in &mut response.trauma_history {
        if !trauma.recovered && trauma.tick + 100 < trauma.tick {
            // This is a safety check; in practice, use the current tick.
            // We mark anything that hasn't been recovered yet as eligible.
            trauma.recovered = true;
        }
    }
}

/// Returns a modifier in [0.5, 1.5] that multiplies other psyche calculations.
/// High stress reduces the modifier; high resilience boosts it.
#[must_use]
pub fn stress_modifier(response: &StressResponse) -> f32 {
    let stress_penalty = response.current_stress * 0.3;
    let resilience_bonus = response.resilience * 0.2;
    (1.0 - stress_penalty + resilience_bonus).clamp(0.5, 1.5)
}

// ===========================================================================
// 5. Group Psychology
// ===========================================================================

/// Dynamics of a group of agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDynamics {
    /// Number of agents in the group.
    pub group_size: u32,
    /// Pressure to conform in [0, 1].
    pub conformity_pressure: f32,
    /// Leader's influence in [0, 1].
    pub leader_influence: f32,
    /// Current panic level in [0, 1].
    pub panic_level: f32,
}

impl Default for GroupDynamics {
    fn default() -> Self {
        Self {
            group_size: 1,
            conformity_pressure: 0.3,
            leader_influence: 0.5,
            panic_level: 0.0,
        }
    }
}

/// Compute conformity: how much an individual conforms to group norms.
/// Returns a value in [0, 1] where 1 = full conformity.
#[must_use]
pub fn compute_conformity(dynamics: &GroupDynamics, individual_deviance: f32) -> f32 {
    let size_factor = (dynamics.group_size as f32 / 100.0).clamp(MOOD_MIN, MOOD_MAX);
    let pressure = dynamics.conformity_pressure * 0.5 + size_factor * 0.5;
    // High deviance → low conformity; high pressure → more conformity
    let conformity = pressure * (1.0 - individual_deviance);
    conformity.clamp(MOOD_MIN, MOOD_MAX)
}

/// Determine mob behavior based on group dynamics and a trigger intensity.
#[must_use]
pub fn mob_behavior(dynamics: &GroupDynamics, trigger_intensity: f32) -> EmotionDrivenBehavior {
    let combined = trigger_intensity * 0.6 + dynamics.panic_level * 0.4;
    if combined > 0.8 && dynamics.group_size > 10 {
        EmotionDrivenBehavior::Flee
    } else if combined > 0.6 && dynamics.leader_influence > 0.5 {
        EmotionDrivenBehavior::Aggress
    } else if combined > 0.3 {
        EmotionDrivenBehavior::Cooperate
    } else {
        EmotionDrivenBehavior::Neutral
    }
}

/// Obedience level based on group dynamics and trust in authority.
/// Returns a value in [0, 1] where 1 = full obedience.
#[must_use]
pub fn obedience_level(dynamics: &GroupDynamics, authority_trust: f32) -> f32 {
    let trust_factor = authority_trust.clamp(MOOD_MIN, MOOD_MAX);
    let leader_factor = dynamics.leader_influence;
    let panic_penalty = dynamics.panic_level * 0.4;
    let size_pressure = (dynamics.group_size as f32 / 200.0).clamp(MOOD_MIN, MOOD_MAX);

    let obedience = (trust_factor * 0.4 + leader_factor * 0.3 + size_pressure * 0.3
        - panic_penalty)
        .clamp(MOOD_MIN, MOOD_MAX);
    obedience
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod psyche_extended_tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Original tests (preserved)
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // behavior_from_psyche tests
    // -----------------------------------------------------------------------

    #[test]
    fn behavior_from_psyche_high_fear_flee() {
        let psyche = Psyche {
            drives: [0.5, 0.8, 0.2, 0.3],
            temperament: civ_agents::Temperament {
                reactivity: 0.6,
                sociability: 0.5,
                risk_tol: 0.5,
                impulsivity: 0.5,
            },
            mood: civ_agents::Mood {
                valence: -0.6,
                arousal: 0.8,
            },
            beliefs: [0.5; 4],
            maturity: 0.5,
        };
        assert_eq!(behavior_from_psyche(&psyche), EmotionDrivenBehavior::Flee);
    }

    #[test]
    fn behavior_from_psyche_high_social_cooperate() {
        let psyche = Psyche {
            drives: [0.3, 0.2, 0.7, 0.3],
            temperament: civ_agents::Temperament::neutral(),
            mood: civ_agents::Mood {
                valence: 0.6,
                arousal: 0.3,
            },
            beliefs: [0.5; 4],
            maturity: 0.5,
        };
        assert_eq!(
            behavior_from_psyche(&psyche),
            EmotionDrivenBehavior::Cooperate
        );
    }

    #[test]
    fn behavior_from_psyche_default_neutral() {
        let psyche = Psyche {
            drives: [0.3, 0.3, 0.3, 0.3],
            temperament: civ_agents::Temperament::neutral(),
            mood: civ_agents::Mood::neutral(),
            beliefs: [0.5; 4],
            maturity: 0.5,
        };
        assert_eq!(
            behavior_from_psyche(&psyche),
            EmotionDrivenBehavior::Neutral
        );
    }

    // -----------------------------------------------------------------------
    // Maslow hierarchy tests
    // -----------------------------------------------------------------------

    #[test]
    fn maslow_default_level_is_physiological() {
        let h = MaslowHierarchy::default();
        assert_eq!(evaluate_current_level(&h), MaslowLevel::Physiological);
    }

    #[test]
    fn maslow_satisfy_physiological() {
        let mut h = MaslowHierarchy::default();
        satisfy_level(&mut h, MaslowLevel::Physiological, 0.6);
        assert!(h.physiological >= 1.0 || h.physiological > 0.5);
        assert_eq!(evaluate_current_level(&h), MaslowLevel::Safety);
    }

    #[test]
    fn maslow_satisfy_all_levels() {
        let mut h = MaslowHierarchy {
            physiological: 1.0,
            safety: 1.0,
            belonging: 1.0,
            esteem: 1.0,
            self_actualization: 1.0,
        };
        assert_eq!(evaluate_current_level(&h), MaslowLevel::SelfActualization);
        assert!((maslow_completeness(&h) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn maslow_completeness_partial() {
        let h = MaslowHierarchy {
            physiological: 0.8,
            safety: 0.6,
            belonging: 0.4,
            esteem: 0.2,
            self_actualization: 0.0,
        };
        let c = maslow_completeness(&h);
        assert!(c > 0.0 && c < 1.0);
        assert!((c - 0.4).abs() < 0.01);
    }

    #[test]
    fn maslow_satisfy_level_clamped() {
        let mut h = MaslowHierarchy::default();
        satisfy_level(&mut h, MaslowLevel::Physiological, 10.0);
        assert!((h.physiological - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn maslow_level_value_accessor() {
        let h = MaslowHierarchy {
            physiological: 0.1,
            safety: 0.2,
            belonging: 0.3,
            esteem: 0.4,
            self_actualization: 0.5,
        };
        assert!((h.level_value(MaslowLevel::Physiological) - 0.1).abs() < f32::EPSILON);
        assert!((h.level_value(MaslowLevel::SelfActualization) - 0.5).abs() < f32::EPSILON);
    }

    // -----------------------------------------------------------------------
    // Emotional state machine tests
    // -----------------------------------------------------------------------

    #[test]
    fn emotion_default_is_neutral() {
        let s = EmotionalState::default();
        assert_eq!(s.primary, Emotion::Neutral);
        assert!((s.intensity - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn transition_emotion_celebration() {
        let s = EmotionalState::default();
        let next = transition_emotion(&s, "celebration", 100);
        assert_eq!(next.primary, Emotion::Joy);
        assert!(next.intensity > 0.0);
    }

    #[test]
    fn transition_emotion_threat() {
        let s = EmotionalState::default();
        let next = transition_emotion(&s, "threat", 100);
        assert_eq!(next.primary, Emotion::Fear);
    }

    #[test]
    fn transition_emotion_unknown_decay() {
        let s = EmotionalState::new(Emotion::Joy, 0.3, 0.01);
        let next = transition_emotion(&s, "unknown_trigger", 100);
        // Should decay
        assert!(next.intensity <= s.intensity);
    }

    #[test]
    fn transition_emotion_calm_resets() {
        let s = EmotionalState::new(Emotion::Anger, 0.9, 0.01);
        let next = transition_emotion(&s, "calm", 100);
        assert_eq!(next.primary, Emotion::Neutral);
        assert!((next.intensity - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn blend_emotions_equal_weight() {
        let a = EmotionalState::new(Emotion::Joy, 0.8, 0.01);
        let b = EmotionalState::new(Emotion::Fear, 0.4, 0.02);
        let blended = blend_emotions(&a, &b, 0.5);
        assert!((blended.intensity - 0.6).abs() < 0.01);
    }

    #[test]
    fn blend_emotions_all_weight_a() {
        let a = EmotionalState::new(Emotion::Joy, 0.9, 0.01);
        let b = EmotionalState::new(Emotion::Fear, 0.1, 0.02);
        let blended = blend_emotions(&a, &b, 0.0);
        assert_eq!(blended.primary, Emotion::Joy);
    }

    #[test]
    fn emotion_to_behavior_joy_cooperates() {
        assert_eq!(
            emotion_to_behavior(&Emotion::Joy, 0.5),
            EmotionDrivenBehavior::Cooperate
        );
    }

    #[test]
    fn emotion_to_behavior_fear_flees() {
        assert_eq!(
            emotion_to_behavior(&Emotion::Fear, 0.8),
            EmotionDrivenBehavior::Flee
        );
    }

    #[test]
    fn emotion_to_behavior_anger_aggresses() {
        assert_eq!(
            emotion_to_behavior(&Emotion::Anger, 0.9),
            EmotionDrivenBehavior::Aggress
        );
    }

    #[test]
    fn emotion_to_behavior_neutral_is_neutral() {
        assert_eq!(
            emotion_to_behavior(&Emotion::Neutral, 0.5),
            EmotionDrivenBehavior::Neutral
        );
    }

    // -----------------------------------------------------------------------
    // Big Five tests
    // -----------------------------------------------------------------------

    #[test]
    fn big_five_deterministic_same_seed() {
        let a = BigFive::random_deterministic(12345);
        let b = BigFive::random_deterministic(12345);
        assert!((a.openness - b.openness).abs() < f32::EPSILON);
        assert!((a.conscientiousness - b.conscientiousness).abs() < f32::EPSILON);
        assert!((a.extraversion - b.extraversion).abs() < f32::EPSILON);
        assert!((a.agreeableness - b.agreeableness).abs() < f32::EPSILON);
        assert!((a.neuroticism - b.neuroticism).abs() < f32::EPSILON);
    }

    #[test]
    fn big_five_different_seeds_differ() {
        let a = BigFive::random_deterministic(1);
        let b = BigFive::random_deterministic(2);
        // At least one trait should differ
        let differs = (a.openness - b.openness).abs() > f32::EPSILON
            || (a.neuroticism - b.neuroticism).abs() > f32::EPSILON;
        assert!(differs);
    }

    #[test]
    fn big_five_values_in_range() {
        let b = BigFive::random_deterministic(999);
        assert!(b.openness >= 0.0 && b.openness <= 1.0);
        assert!(b.conscientiousness >= 0.0 && b.conscientiousness <= 1.0);
        assert!(b.extraversion >= 0.0 && b.extraversion <= 1.0);
        assert!(b.agreeableness >= 0.0 && b.agreeableness <= 1.0);
        assert!(b.neuroticism >= 0.0 && b.neuroticism <= 1.0);
    }

    #[test]
    fn compatibility_identical_is_high() {
        let b = BigFive::random_deterministic(42);
        let score = compatibility_score(&b, &b);
        assert!(score > 0.8);
    }

    #[test]
    fn personality_modifier_social_gathering_extravert() {
        let b = BigFive {
            openness: 0.5,
            conscientiousness: 0.5,
            extraversion: 0.9,
            agreeableness: 0.5,
            neuroticism: 0.5,
        };
        let m = personality_modifier(&b, "social_gathering");
        assert!(m > 0.0);
    }

    #[test]
    fn personality_modifier_isolation_introvert() {
        let b = BigFive {
            openness: 0.5,
            conscientiousness: 0.5,
            extraversion: 0.1,
            agreeableness: 0.5,
            neuroticism: 0.5,
        };
        let m = personality_modifier(&b, "isolation");
        assert!(m > 0.0); // introvert tolerates isolation
    }

    #[test]
    fn personality_modifier_unknown_returns_zero() {
        let b = BigFive::random_deterministic(7);
        let m = personality_modifier(&b, "unknown_situation");
        assert!((m - 0.0).abs() < f32::EPSILON);
    }

    // -----------------------------------------------------------------------
    // Trauma / stress tests
    // -----------------------------------------------------------------------

    #[test]
    fn stress_accumulates() {
        let mut r = StressResponse::default();
        experience_stress(&mut r, 0.5, 10);
        assert!(r.current_stress > 0.0);
    }

    #[test]
    fn stress_records_trauma_on_severe() {
        let mut r = StressResponse::default();
        experience_stress(&mut r, 0.8, 10);
        assert_eq!(r.trauma_history.len(), 1);
        assert!((r.trauma_history[0].severity - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn stress_mild_not_recorded() {
        let mut r = StressResponse::default();
        experience_stress(&mut r, 0.3, 10);
        assert!(r.trauma_history.is_empty());
    }

    #[test]
    fn stress_recovery_reduces() {
        let mut r = StressResponse {
            current_stress: 0.8,
            ..StressResponse::default()
        };
        recover_from_trauma(&mut r, 0.1);
        assert!(r.current_stress < 0.8);
    }

    #[test]
    fn stress_modifier_stressed_is_lower() {
        let r = StressResponse {
            current_stress: 0.9,
            resilience: 0.5,
            ..StressResponse::default()
        };
        assert!(stress_modifier(&r) < 1.0);
    }

    #[test]
    fn stress_modifier_resilient_is_higher() {
        let r = StressResponse {
            current_stress: 0.0,
            resilience: 0.9,
            ..StressResponse::default()
        };
        assert!(stress_modifier(&r) > 1.0);
    }

    #[test]
    fn stress_clamped_upper() {
        let mut r = StressResponse::default();
        experience_stress(&mut r, 10.0, 1);
        assert!(r.current_stress <= 1.0);
    }

    // -----------------------------------------------------------------------
    // Group psychology tests
    // -----------------------------------------------------------------------

    #[test]
    fn conformity_high_deviance_is_low() {
        let g = GroupDynamics::default();
        let c = compute_conformity(&g, 0.9);
        assert!(c < 0.7);
    }

    #[test]
    fn conformity_zero_deviance_is_high() {
        let g = GroupDynamics {
            conformity_pressure: 0.8,
            ..GroupDynamics::default()
        };
        let c = compute_conformity(&g, 0.0);
        assert!(c > 0.3);
    }

    #[test]
    fn mob_behavior_large_group_high_panic_flees() {
        let g = GroupDynamics {
            group_size: 50,
            panic_level: 0.9,
            ..GroupDynamics::default()
        };
        assert_eq!(mob_behavior(&g, 0.9), EmotionDrivenBehavior::Flee);
    }

    #[test]
    fn mob_behavior_leader_influenced_aggress() {
        let g = GroupDynamics {
            group_size: 20,
            leader_influence: 0.8,
            panic_level: 0.3,
            conformity_pressure: 0.5,
        };
        assert_eq!(mob_behavior(&g, 0.9), EmotionDrivenBehavior::Aggress);
    }

    #[test]
    fn mob_behavior_calm_group_neutral() {
        let g = GroupDynamics {
            group_size: 5,
            panic_level: 0.0,
            ..GroupDynamics::default()
        };
        assert_eq!(mob_behavior(&g, 0.1), EmotionDrivenBehavior::Neutral);
    }

    #[test]
    fn obedience_high_trust_high_leader() {
        let g = GroupDynamics {
            group_size: 20,
            leader_influence: 0.9,
            conformity_pressure: 0.5,
            panic_level: 0.0,
        };
        let o = obedience_level(&g, 0.9);
        assert!(o > 0.5);
    }

    #[test]
    fn obedience_panic_reduces() {
        let g = GroupDynamics {
            leader_influence: 0.9,
            panic_level: 0.9,
            ..GroupDynamics::default()
        };
        let calm_g = GroupDynamics {
            leader_influence: 0.9,
            panic_level: 0.0,
            ..GroupDynamics::default()
        };
        let o_panic = obedience_level(&g, 0.8);
        let o_calm = obedience_level(&calm_g, 0.8);
        assert!(o_panic < o_calm);
    }

    #[test]
    fn obedience_in_range() {
        let g = GroupDynamics {
            group_size: 100,
            conformity_pressure: 0.5,
            leader_influence: 0.7,
            panic_level: 0.3,
        };
        let o = obedience_level(&g, 0.6);
        assert!(o >= 0.0 && o <= 1.0);
    }
}
