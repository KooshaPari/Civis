//! Mood system integration for the Civis Bevy godgame.
//!
//! Aggregates multiple environmental and social signals into a single mood
//! value per settlement, providing both a continuous score and discrete
//! [`MoodState`] classification that other subsystems (economy, unrest,
//! festivals) can query.

use crate::famine::FamineStage;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// MoodFactors
// ---------------------------------------------------------------------------

/// Weighted inputs that feed into the mood calculation.
///
/// All fields should be in `[0.0, 1.0]` — values outside that range are
/// clamped by the engine before use.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MoodFactors {
    /// Perceived food security (0 = starving, 1 = abundant).
    pub food_safety: f32,
    /// Quality / adequacy of shelter (0 = exposed, 1 = excellent).
    pub shelter_quality: f32,
    /// Social cohesion / connection (0 = isolated, 1 = tightly knit).
    pub social_connection: f32,
    /// Perceived external/internal threat (0 = safe, 1 = mortal danger).
    pub threat_level: f32,
    /// Happiness boost from an active festival (0 = none, 1 = peak).
    pub festival_happiness: f32,
    /// Religious / spiritual fulfillment (0 = absent, 1 = fulfilled).
    pub religious_fulfillment: f32,
}

impl Default for MoodFactors {
    fn default() -> Self {
        Self {
            food_safety: 0.5,
            shelter_quality: 0.5,
            social_connection: 0.5,
            threat_level: 0.0,
            festival_happiness: 0.0,
            religious_fulfillment: 0.0,
        }
    }
}

impl MoodFactors {
    /// Weight assigned to each factor when computing the raw mood score.
    ///
    /// Weights are:
    /// - food_safety: 0.30
    /// - shelter_quality: 0.15
    /// - social_connection: 0.20
    /// - threat_level: 0.25 (inverted — high threat *reduces* mood)
    /// - festival_happiness: 0.05
    /// - religious_fulfillment: 0.05
    const WEIGHTS: [f32; 6] = [0.30, 0.15, 0.20, 0.25, 0.05, 0.05];

    /// Build a `MoodFactors` from coarse simulation state.
    ///
    /// * `famine_stage` — current food-shortage classification.
    /// * `threat_level` — generic threat value `[0.0, 1.0]`.
    /// * `has_festival` — whether a festival is currently active.
    /// * `religion_strength` — religious fulfillment `[0.0, 1.0]`.
    #[must_use]
    pub fn from_simulation(
        famine_stage: FamineStage,
        threat_level: f32,
        has_festival: bool,
        religion_strength: f32,
    ) -> Self {
        let food_safety = match famine_stage {
            FamineStage::None => 1.0,
            FamineStage::Hungry => 0.6,
            FamineStage::Starving => 0.3,
            FamineStage::Famine => 0.1,
            FamineStage::Collapse => 0.0,
        };

        Self {
            food_safety,
            shelter_quality: 0.7, // default assumed adequate
            social_connection: 0.5,
            threat_level: threat_level.clamp(0.0, 1.0),
            festival_happiness: if has_festival { 0.8 } else { 0.0 },
            religious_fulfillment: religion_strength.clamp(0.0, 1.0),
        }
    }

    /// Compute a raw mood score from the weighted factor average.
    ///
    /// The threat level is *inverted* (subtracted) so that high threat
    /// decreases mood.  Returns a value in `[0.0, 1.0]`.
    #[must_use]
    pub fn weighted_average(&self) -> f32 {
        let raw = self.food_safety * Self::WEIGHTS[0]
            + self.shelter_quality * Self::WEIGHTS[1]
            + self.social_connection * Self::WEIGHTS[2]
            + (1.0 - self.threat_level) * Self::WEIGHTS[3]
            + self.festival_happiness * Self::WEIGHTS[4]
            + self.religious_fulfillment * Self::WEIGHTS[5];

        raw.clamp(0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// MoodState
// ---------------------------------------------------------------------------

/// Discrete mood classification derived from the continuous mood score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MoodState {
    /// mood > 0.8
    Euphoric,
    /// mood > 0.6
    Happy,
    /// mood > 0.4
    Content,
    /// mood > 0.2
    Uneasy,
    /// mood <= 0.2
    Miserable,
}

impl MoodState {
    /// Classify a raw mood value in `[0.0, 1.0]` into a discrete state.
    #[must_use]
    pub fn from_score(score: f32) -> Self {
        if score > 0.8 {
            Self::Euphoric
        } else if score > 0.6 {
            Self::Happy
        } else if score > 0.4 {
            Self::Content
        } else if score > 0.2 {
            Self::Uneasy
        } else {
            Self::Miserable
        }
    }
}

// ---------------------------------------------------------------------------
// MoodEngine
// ---------------------------------------------------------------------------

/// Tick-based mood engine that smooths factor changes through inertia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodEngine {
    /// Current smoothed mood value `[0.0, 1.0]`.
    pub current_mood: f32,
    /// Rolling history of recent mood scores (most recent last).
    pub mood_history: Vec<f32>,
    /// How slowly the mood reacts to factor changes.
    ///
    /// `0.0` = instant snap to new value, `1.0` = never changes from initial.
    /// Typical value: `0.85`.
    pub mood_inertia: f32,
}

impl Default for MoodEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MoodEngine {
    /// Create a new engine with a neutral starting mood.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_mood: 0.5,
            mood_history: Vec::new(),
            mood_inertia: 0.85,
        }
    }

    /// Advance the mood by one tick.
    ///
    /// * `factors` — current environmental/social signals.
    /// * `dt` — time elapsed since last update (in ticks, typically 1.0).
    ///
    /// Returns the resulting [`MoodState`].
    pub fn update_from_factors(&mut self, factors: &MoodFactors, dt: f32) -> MoodState {
        let target = factors.weighted_average();

        // Exponential smoothing: new = lerp(current, target, (1 - inertia)^dt)
        let alpha = (1.0 - self.mood_inertia).powf(dt);
        self.current_mood = self.current_mood + alpha * (target - self.current_mood);
        self.current_mood = self.current_mood.clamp(0.0, 1.0);

        self.record_history();
        self.current_state()
    }

    /// Current discrete mood state.
    #[must_use]
    pub fn current_state(&self) -> MoodState {
        MoodState::from_score(self.current_mood)
    }

    /// A modifier value in `[-0.3, +0.3]` that other systems can add to
    /// their calculations.
    ///
    /// - mood 0.0 → modifier -0.3
    /// - mood 0.5 → modifier 0.0
    /// - mood 1.0 → modifier +0.3
    #[must_use]
    pub fn mood_modifier(&self) -> f32 {
        ((self.current_mood - 0.5) * 0.6).clamp(-0.3, 0.3)
    }

    /// Push the current mood score onto the history buffer.
    ///
    /// Keeps at most 100 entries; the oldest entry is dropped when full.
    pub fn record_history(&mut self) {
        const MAX_HISTORY: usize = 100;
        if self.mood_history.len() >= MAX_HISTORY {
            self.mood_history.remove(0);
        }
        self.mood_history.push(self.current_mood);
    }

    /// Average mood over the last `ticks` recorded values.
    ///
    /// If fewer than `ticks` values exist, uses the entire history.
    #[must_use]
    pub fn average_mood(&self, ticks: u32) -> f32 {
        if self.mood_history.is_empty() {
            return self.current_mood;
        }
        let n = ticks as usize;
        let window = if n >= self.mood_history.len() {
            &self.mood_history[..]
        } else {
            &self.mood_history[self.mood_history.len() - n..]
        };
        let sum: f32 = window.iter().sum();
        sum / window.len() as f32
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- MoodFactors -----------------------------------------------------

    #[test]
    fn factors_weighted_average_all_ones() {
        let f = MoodFactors {
            food_safety: 1.0,
            shelter_quality: 1.0,
            social_connection: 1.0,
            threat_level: 0.0,
            festival_happiness: 1.0,
            religious_fulfillment: 1.0,
        };
        let avg = f.weighted_average();
        assert!(
            (avg - 1.0).abs() < f32::EPSILON,
            "all-ones factors should yield 1.0, got {avg}"
        );
    }

    #[test]
    fn factors_weighted_average_all_zeros() {
        let f = MoodFactors {
            food_safety: 0.0,
            shelter_quality: 0.0,
            social_connection: 0.0,
            threat_level: 1.0,
            festival_happiness: 0.0,
            religious_fulfillment: 0.0,
        };
        let avg = f.weighted_average();
        assert!(
            avg.abs() < f32::EPSILON,
            "all-zeros factors should yield 0.0, got {avg}"
        );
    }

    #[test]
    fn from_simulation_none_famine_high_food() {
        let f = MoodFactors::from_simulation(FamineStage::None, 0.0, false, 0.0);
        assert_eq!(f.food_safety, 1.0);
        assert_eq!(f.threat_level, 0.0);
        assert_eq!(f.festival_happiness, 0.0);
    }

    #[test]
    fn from_simulation_collapse_low_food() {
        let f = MoodFactors::from_simulation(FamineStage::Collapse, 1.0, true, 0.5);
        assert_eq!(f.food_safety, 0.0);
        assert_eq!(f.threat_level, 1.0);
        assert_eq!(f.festival_happiness, 0.8);
        assert_eq!(f.religious_fulfillment, 0.5);
    }

    // ---- MoodState -------------------------------------------------------

    #[test]
    fn state_euphoric_above_08() {
        assert_eq!(MoodState::from_score(0.85), MoodState::Euphoric);
        assert_eq!(MoodState::from_score(1.0), MoodState::Euphoric);
    }

    #[test]
    fn state_happy_above_06() {
        assert_eq!(MoodState::from_score(0.7), MoodState::Happy);
    }

    #[test]
    fn state_content_above_04() {
        assert_eq!(MoodState::from_score(0.5), MoodState::Content);
    }

    #[test]
    fn state_uneasy_above_02() {
        assert_eq!(MoodState::from_score(0.3), MoodState::Uneasy);
    }

    #[test]
    fn state_miserable_at_or_below_02() {
        assert_eq!(MoodState::from_score(0.0), MoodState::Miserable);
        assert_eq!(MoodState::from_score(0.2), MoodState::Miserable);
    }

    // ---- MoodEngine ------------------------------------------------------

    #[test]
    fn engine_starts_at_neutral() {
        let e = MoodEngine::new();
        assert_eq!(e.current_mood, 0.5);
        assert_eq!(e.current_state(), MoodState::Content);
    }

    #[test]
    fn engine_inertia_delays_change() {
        let mut e = MoodEngine::new();
        e.mood_inertia = 0.9; // high inertia — slow change
        let f = MoodFactors {
            food_safety: 1.0,
            shelter_quality: 1.0,
            social_connection: 1.0,
            threat_level: 0.0,
            festival_happiness: 1.0,
            religious_fulfillment: 1.0,
        };

        // After one tick with high inertia the mood should barely move.
        e.update_from_factors(&f, 1.0);
        assert!(
            e.current_mood <= 0.56,
            "inertia should prevent jump; got {}",
            e.current_mood
        );
    }

    #[test]
    fn engine_converges_over_ticks() {
        let mut e = MoodEngine::new();
        e.mood_inertia = 0.85;
        let f = MoodFactors {
            food_safety: 1.0,
            shelter_quality: 1.0,
            social_connection: 1.0,
            threat_level: 0.0,
            festival_happiness: 0.0,
            religious_fulfillment: 0.0,
        };

        for _ in 0..50 {
            e.update_from_factors(&f, 1.0);
        }
        // After 50 ticks it should be close to the target.
        let target = f.weighted_average();
        assert!(
            (e.current_mood - target).abs() < 0.05,
            "should converge toward {target}, got {}",
            e.current_mood
        );
    }

    #[test]
    fn mood_modifier_range() {
        let mut e = MoodEngine::new();

        // Extreme low
        e.current_mood = 0.0;
        assert_eq!(e.mood_modifier(), -0.3);

        // Mid
        e.current_mood = 0.5;
        assert_eq!(e.mood_modifier(), 0.0);

        // Extreme high
        e.current_mood = 1.0;
        assert_eq!(e.mood_modifier(), 0.3);
    }

    #[test]
    fn average_mood_uses_window() {
        let mut e = MoodEngine::new();
        // Manually build history
        e.mood_history = vec![0.2, 0.4, 0.6, 0.8, 1.0];

        // Last 3 ticks
        let avg = e.average_mood(3);
        assert!((avg - 0.8).abs() < f32::EPSILON, "expected 0.8, got {avg}");

        // More ticks than history → uses all
        let avg_all = e.average_mood(100);
        let expected = (0.2 + 0.4 + 0.6 + 0.8 + 1.0) / 5.0;
        assert!(
            (avg_all - expected).abs() < f32::EPSILON,
            "expected {expected}, got {avg_all}"
        );
    }

    #[test]
    fn average_mood_empty_history_returns_current() {
        let e = MoodEngine::new();
        // Empty history, should return current_mood (0.5).
        assert_eq!(e.average_mood(10), 0.5);
    }

    #[test]
    fn history_capped_at_100() {
        let mut e = MoodEngine::new();
        e.mood_inertia = 0.0; // instant response for easy testing
        let f = MoodFactors::default();

        for _ in 0..150 {
            e.update_from_factors(&f, 1.0);
        }
        assert!(
            e.mood_history.len() <= 100,
            "history should be capped at 100, got {}",
            e.mood_history.len()
        );
    }
}
