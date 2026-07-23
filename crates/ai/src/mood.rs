//! Agent mood as a measured function of contributing factors (FR-CIV-PSYCHE-901).
//!
//! Mood is **not** a fixed table or a per-tick assignment: it is recomputed
//! every Hot tick from a small set of inspectable named
//! [`MoodFactor`]s — need satisfaction, recent memory, environment, and social
//! events — and exposed via [`MoodState::factors`] so callers (the inspector,
//! dashboards, downstream psyche code) can read the contributors just like any
//! other measured sim quantity (see FR-CIV-INSPECT-901).
//!
//! The design follows the same shape as [`crate::goal`]: pure logic, no I/O,
//! no async, no Bevy ECS dependency, so the same code path runs inside a Bevy
//! system, a worker-pool job, a replay re-simulation, or a one-off unit test.
//!
//! Determinism: identical [`MoodInputs`] → identical recomputed valence and
//! identical factor breakdown. There is no RNG in the model.
//!
//! ## Semantics
//!
//! The valence lives in `[-1.0, 1.0]`:
//!
//! | Range          | Label      |
//! |----------------|------------|
//! | `(0.5, 1.0]`   | Elated     |
//! | `(0.1, 0.5]`   | Content    |
//! | `(-0.1, 0.1]` | Neutral    |
//! | `(-0.5, -0.1]` | Displeased |
//! | `[-1.0, -0.5]` | Miserable  |
//!
//! The labels are convenience buckets for tests and dashboards; the valence
//! itself is the source of truth.

#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

use crate::goal::Need;

/// The four canonical contributors to mood. Every variant maps to a
/// [`MoodFactor`] value stored in [`MoodState::factors`] so the breakdown is
/// always enumerable, never hidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoodFactorKind {
    /// Mean need satisfaction across the agent's [`Need`] vector (satisfied
    /// needs → positive contribution).
    NeedSatisfaction,
    /// Recent memory: the mean valence of memory events within a lookback
    /// window (positive memories → positive contribution).
    RecentMemory,
    /// Environment: a single bounded signal in `[-1.0, 1.0]` summarizing the
    /// surroundings (weather, shelter quality, ambient danger, …).
    Environment,
    /// Social events: the mean valence of recent social events (positive
    /// interactions → positive contribution).
    SocialEvents,
}

impl MoodFactorKind {
    /// All factor kinds, in stable inspection order. Use this to enumerate the
    /// factors without depending on the [`MoodState::factors`] insertion order.
    #[must_use]
    pub const fn all() -> [MoodFactorKind; 4] {
        [
            MoodFactorKind::NeedSatisfaction,
            MoodFactorKind::RecentMemory,
            MoodFactorKind::Environment,
            MoodFactorKind::SocialEvents,
        ]
    }
}

/// A single named, bounded contribution to the agent's mood. Stored in
/// [`MoodState::factors`] and surfaced by inspectors.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MoodFactor {
    /// Which contributor this entry represents.
    pub kind: MoodFactorKind,
    /// Bounded contribution in `[-1.0, 1.0]`. The weighted sum of all factors
    /// (see [`DEFAULT_WEIGHTS`]) produces the raw mood; the raw mood is then
    /// clamped into `[-1.0, 1.0]` to yield the agent's [`MoodState::valence`].
    pub value: f32,
}

impl MoodFactor {
    /// Convenience constructor that clamps `value` into `[-1.0, 1.0]` so a
    /// misconfigured upstream cannot poison the mood model.
    #[must_use]
    pub fn new(kind: MoodFactorKind, value: f32) -> Self {
        Self {
            kind,
            value: value.clamp(-1.0, 1.0),
        }
    }
}

/// Per-tick inputs the mood model needs. Callers fill these from their own
/// sources (e.g. `civ-genetics` for temperament, the social graph for events);
/// this module deliberately knows nothing about those systems so it can be
/// embedded anywhere.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MoodInputs {
    /// The agent's current need vector. Need satisfaction contributes as
    /// `1.0 − mean(urgency)`, i.e. fully satisfied → `+1.0`, every need
    /// critical → `−1.0`.
    pub needs: Vec<Need>,
    /// Recent memory events, each carrying a bounded valence (`[-1, 1]`) and
    /// a tick. Only the most recent `memory_window` events are considered.
    pub memory_events: Vec<MemoryEvent>,
    /// Environment summary in `[-1.0, 1.0]`. `+1.0` = idyllic; `−1.0` =
    /// hostile; `0.0` = neutral baseline.
    pub environment: f32,
    /// Social events, each carrying a bounded valence (`[-1, 1]`) and a tick.
    /// Only the most recent `social_window` events are considered.
    pub social_events: Vec<SocialMoodEvent>,
    /// Lookback window (in ticks) for [`MoodInputs::memory_events`]. Defaults
    /// to [`DEFAULT_MEMORY_WINDOW`] when `0`.
    pub memory_window: u64,
    /// Lookback window (in ticks) for [`MoodInputs::social_events`]. Defaults
    /// to [`DEFAULT_SOCIAL_WINDOW`] when `0`.
    pub social_window: u64,
    /// Current sim tick. Only used so the model can keep the factor breakdown
    /// traceable; does not affect arithmetic.
    pub current_tick: u64,
}

/// A single recent-memory item the agent will fold into mood.
///
/// Additive: callers (e.g. `civ-genetics` or a chronicle writer) populate
/// these from whatever memory subsystem they already have; this struct is the
/// narrow contract the mood model needs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MemoryEvent {
    /// Tick when the memory was recorded.
    pub tick: u64,
    /// Bounded valence in `[-1.0, 1.0]`. `+1.0` = wonderful memory,
    /// `−1.0` = traumatic memory.
    pub valence: f32,
}

impl MemoryEvent {
    /// Construct a memory event, clamping `valence` into `[-1, 1]`.
    #[must_use]
    pub fn new(tick: u64, valence: f32) -> Self {
        Self {
            tick,
            valence: valence.clamp(-1.0, 1.0),
        }
    }
}

/// A single social event the agent will fold into mood. Mirrors
/// [`MemoryEvent`] but lives on the social side of the psyche so callers can
/// keep the two streams separate if they want to.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SocialMoodEvent {
    /// Tick when the event was recorded.
    pub tick: u64,
    /// Bounded valence in `[-1.0, 1.0]`. `+1.0` = warmly received,
    /// `−1.0` = harshly rejected.
    pub valence: f32,
}

impl SocialMoodEvent {
    /// Construct a social event, clamping `valence` into `[-1, 1]`.
    #[must_use]
    pub fn new(tick: u64, valence: f32) -> Self {
        Self {
            tick,
            valence: valence.clamp(-1.0, 1.0),
        }
    }
}

/// Default lookback window for memory events (ticks).
pub const DEFAULT_MEMORY_WINDOW: u64 = 200;
/// Default lookback window for social events (ticks).
pub const DEFAULT_SOCIAL_WINDOW: u64 = 100;

/// Per-factor weights used to combine the four [`MoodFactor`]s into the raw
/// mood. Sum to `1.0` so the raw mood stays in `[-1.0, 1.0]` when every
/// factor stays in `[-1.0, 1.0]`.
///
/// Tuned so need satisfaction is the largest single contributor (an agent
/// whose drives are met feels good even when other factors are noisy) but
/// no factor dominates the picture.
pub const DEFAULT_WEIGHTS: MoodWeights = MoodWeights {
    need_satisfaction: 0.45,
    recent_memory: 0.25,
    environment: 0.15,
    social_events: 0.15,
};

/// Per-factor weights. Sum does not strictly have to equal `1.0`; the final
/// valence is clamped into `[-1, 1]` regardless. [`DEFAULT_WEIGHTS`] is the
/// recommended starting point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MoodWeights {
    /// Weight applied to [`MoodFactorKind::NeedSatisfaction`].
    pub need_satisfaction: f32,
    /// Weight applied to [`MoodFactorKind::RecentMemory`].
    pub recent_memory: f32,
    /// Weight applied to [`MoodFactorKind::Environment`].
    pub environment: f32,
    /// Weight applied to [`MoodFactorKind::SocialEvents`].
    pub social_events: f32,
}

impl Default for MoodWeights {
    fn default() -> Self {
        DEFAULT_WEIGHTS
    }
}

/// The agent's current mood: a recomputed valence plus the full breakdown of
/// contributing factors. Designed to be cheap to compute and inspectable on
/// every read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoodState {
    /// Bounded mood valence in `[-1.0, 1.0]`. `+1.0` = elated,
    /// `−1.0` = miserable.
    pub valence: f32,
    /// Per-factor breakdown. Always contains exactly one entry per
    /// [`MoodFactorKind`] in the order returned by [`MoodFactorKind::all`],
    /// so enumerating [`MoodState::factors`] yields every contributor.
    pub factors: Vec<MoodFactor>,
    /// Tick this mood was last computed. Useful for staleness checks.
    pub computed_at_tick: u64,
}

impl MoodState {
    /// Neutral baseline: valence `0.0`, every factor at `0.0`, computed at
    /// tick `0`.
    #[must_use]
    pub fn neutral() -> Self {
        Self {
            valence: 0.0,
            factors: MoodFactorKind::all()
                .iter()
                .map(|&k| MoodFactor::new(k, 0.0))
                .collect(),
            computed_at_tick: 0,
        }
    }

    /// Look up a single factor by kind. Returns `None` if the breakdown does
    /// not contain an entry for `kind` (should not happen for [`recompute`]
    /// output, but the option keeps callers honest).
    #[must_use]
    pub fn factor(&self, kind: MoodFactorKind) -> Option<MoodFactor> {
        self.factors.iter().find(|f| f.kind == kind).copied()
    }

    /// Convenience: the bounded [`MoodFactor`] for need satisfaction.
    #[must_use]
    pub fn need_satisfaction(&self) -> f32 {
        self.factor(MoodFactorKind::NeedSatisfaction)
            .map_or(0.0, |f| f.value)
    }

    /// Convenience: the bounded [`MoodFactor`] for recent memory.
    #[must_use]
    pub fn recent_memory(&self) -> f32 {
        self.factor(MoodFactorKind::RecentMemory)
            .map_or(0.0, |f| f.value)
    }

    /// Convenience: the bounded [`MoodFactor`] for environment.
    #[must_use]
    pub fn environment(&self) -> f32 {
        self.factor(MoodFactorKind::Environment)
            .map_or(0.0, |f| f.value)
    }

    /// Convenience: the bounded [`MoodFactor`] for social events.
    #[must_use]
    pub fn social_events(&self) -> f32 {
        self.factor(MoodFactorKind::SocialEvents)
            .map_or(0.0, |f| f.value)
    }

    /// Recompute the mood for the given [`MoodInputs`] and weights, returning
    /// the updated [`MoodState`]. Intended to be called every Hot tick.
    ///
    /// The algorithm:
    ///
    /// 1. For each [`MoodFactorKind`], compute a bounded contribution from
    ///    the corresponding slice of `inputs`.
    /// 2. Combine the four contributions using `weights`.
    /// 3. Clamp the weighted sum into `[-1.0, 1.0]` to produce `valence`.
    /// 4. Return a fresh [`MoodState`] whose `factors` enumerate every
    ///    contributor (inspectability is the point — see FR-CIV-INSPECT-901).
    #[must_use]
    pub fn recompute(inputs: &MoodInputs, weights: &MoodWeights) -> Self {
        let factors = vec![
            MoodFactor::new(
                MoodFactorKind::NeedSatisfaction,
                need_satisfaction_factor(&inputs.needs),
            ),
            MoodFactor::new(MoodFactorKind::RecentMemory, memory_factor(inputs)),
            MoodFactor::new(MoodFactorKind::Environment, inputs.environment),
            MoodFactor::new(MoodFactorKind::SocialEvents, social_factor(inputs)),
        ];

        let raw = factors
            .iter()
            .map(|f| match f.kind {
                MoodFactorKind::NeedSatisfaction => weights.need_satisfaction * f.value,
                MoodFactorKind::RecentMemory => weights.recent_memory * f.value,
                MoodFactorKind::Environment => weights.environment * f.value,
                MoodFactorKind::SocialEvents => weights.social_events * f.value,
            })
            .sum::<f32>();

        Self {
            valence: raw.clamp(-1.0, 1.0),
            factors,
            computed_at_tick: inputs.current_tick,
        }
    }

    /// Human-readable bucket for dashboards / tests. See the module docs for
    /// the cut points.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self.valence {
            v if v > 0.5 => "Elated",
            v if v > 0.1 => "Content",
            v if v >= -0.1 => "Neutral",
            v if v >= -0.5 => "Displeased",
            _ => "Miserable",
        }
    }
}

impl Default for MoodState {
    fn default() -> Self {
        Self::neutral()
    }
}

// -----------------------------------------------------------------------------
// Pure helpers
// -----------------------------------------------------------------------------

/// Need satisfaction factor: `1.0 − mean(urgency)`. Empty needs → `0.0`
/// (neutral: no signal, no penalty).
fn need_satisfaction_factor(needs: &[Need]) -> f32 {
    if needs.is_empty() {
        return 0.0;
    }
    let mean_urgency: f32 = needs.iter().map(|n| n.urgency()).sum::<f32>() / needs.len() as f32;
    (1.0 - 2.0 * mean_urgency).clamp(-1.0, 1.0)
}

/// Mean valence of memory events within the lookback window. Empty (or
/// out-of-window) memory → `0.0`.
fn memory_factor(inputs: &MoodInputs) -> f32 {
    let window = effective_window(inputs.memory_window, DEFAULT_MEMORY_WINDOW);
    let cutoff = inputs.current_tick.saturating_sub(window);
    let recent: Vec<f32> = inputs
        .memory_events
        .iter()
        .filter(|m| m.tick >= cutoff)
        .map(|m| m.valence)
        .collect();
    if recent.is_empty() {
        0.0
    } else {
        let mean = recent.iter().sum::<f32>() / recent.len() as f32;
        mean.clamp(-1.0, 1.0)
    }
}

/// Mean valence of social events within the lookback window. Empty (or
/// out-of-window) events → `0.0`.
fn social_factor(inputs: &MoodInputs) -> f32 {
    let window = effective_window(inputs.social_window, DEFAULT_SOCIAL_WINDOW);
    let cutoff = inputs.current_tick.saturating_sub(window);
    let recent: Vec<f32> = inputs
        .social_events
        .iter()
        .filter(|e| e.tick >= cutoff)
        .map(|e| e.valence)
        .collect();
    if recent.is_empty() {
        0.0
    } else {
        let mean = recent.iter().sum::<f32>() / recent.len() as f32;
        mean.clamp(-1.0, 1.0)
    }
}

/// A `0` window falls back to the default so callers can leave the field
/// alone and still get a sensible behaviour.
fn effective_window(requested: u64, default: u64) -> u64 {
    if requested == 0 {
        default
    } else {
        requested
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Convenience builder for tests that want all factors enumerated.
    fn factor_value(state: &MoodState, kind: MoodFactorKind) -> f32 {
        state.factor(kind).map_or(f32::NAN, |f| f.value)
    }

    /// FR-CIV-PSYCHE-901 — mood rises with need satisfaction. Hungry agent
    /// becomes content as every need approaches satisfied.
    #[test]
    fn mood_rises_with_need_satisfaction() {
        let weights = MoodWeights::default();
        let env = MoodInputs {
            current_tick: 100,
            ..MoodInputs::default()
        };

        let hungry = MoodInputs {
            needs: vec![Need::Hunger(1.0), Need::Rest(1.0), Need::Safety(1.0)],
            ..env.clone()
        };
        let content = MoodInputs {
            needs: vec![Need::Hunger(0.0), Need::Rest(0.0), Need::Safety(0.0)],
            ..env
        };

        let m_hungry = MoodState::recompute(&hungry, &weights);
        let m_content = MoodState::recompute(&content, &weights);

        assert!(
            m_content.valence > m_hungry.valence,
            "content agent ({}) should be happier than starving agent ({})",
            m_content.valence,
            m_hungry.valence
        );
        assert!(
            m_hungry.valence < 0.0,
            "every need critical should pull mood below zero (got {})",
            m_hungry.valence
        );
        assert!(
            m_content.valence > 0.0,
            "every need satisfied should push mood above zero (got {})",
            m_content.valence
        );
        assert!(
            factor_value(&m_content, MoodFactorKind::NeedSatisfaction) > 0.9,
            "all-satisfied needs should give ~+1.0 need factor"
        );
    }

    /// FR-CIV-PSYCHE-901 — mood falls with negative events. A burst of
    /// strongly-negative memory + social events must drag mood below zero,
    /// while a burst of positive events lifts it above zero.
    #[test]
    fn mood_falls_with_negative_events_and_rises_with_positive() {
        let weights = MoodWeights::default();
        let base = MoodInputs {
            needs: vec![Need::Hunger(0.5)], // neutral need factor
            current_tick: 500,
            ..MoodInputs::default()
        };

        let negative = MoodInputs {
            memory_events: (0..5).map(|i| MemoryEvent::new(500 - i, -0.9)).collect(),
            social_events: (0..5)
                .map(|i| SocialMoodEvent::new(500 - i, -0.9))
                .collect(),
            ..base.clone()
        };
        let positive = MoodInputs {
            memory_events: (0..5).map(|i| MemoryEvent::new(500 - i, 0.9)).collect(),
            social_events: (0..5).map(|i| SocialMoodEvent::new(500 - i, 0.9)).collect(),
            ..base
        };

        let m_neg = MoodState::recompute(&negative, &weights);
        let m_pos = MoodState::recompute(&positive, &weights);

        assert!(
            m_neg.valence < 0.0,
            "negative events should pull mood below zero (got {})",
            m_neg.valence
        );
        assert!(
            m_pos.valence > 0.0,
            "positive events should lift mood above zero (got {})",
            m_pos.valence
        );
        assert!(
            m_neg.valence < m_pos.valence,
            "negative-event mood ({}) should be below positive-event mood ({})",
            m_neg.valence,
            m_pos.valence
        );
    }

    /// FR-CIV-PSYCHE-901 — the breakdown is enumerable: every factor kind is
    /// present, in stable order, with a bounded value.
    #[test]
    fn factors_are_enumerable_for_inspection() {
        let inputs = MoodInputs {
            needs: vec![Need::Hunger(0.2), Need::Rest(0.3)],
            memory_events: vec![MemoryEvent::new(10, 0.4)],
            environment: 0.1,
            social_events: vec![SocialMoodEvent::new(10, 0.5)],
            current_tick: 20,
            ..MoodInputs::default()
        };
        let state = MoodState::recompute(&inputs, &MoodWeights::default());

        let kinds: Vec<MoodFactorKind> = state.factors.iter().map(|f| f.kind).collect();
        assert_eq!(kinds, MoodFactorKind::all().to_vec());

        for factor in &state.factors {
            assert!(
                (-1.0..=1.0).contains(&factor.value),
                "factor {:?} value {} must be in [-1, 1]",
                factor.kind,
                factor.value
            );
        }

        // Convenience accessors must agree with the breakdown.
        assert!(
            (state.need_satisfaction() - factor_value(&state, MoodFactorKind::NeedSatisfaction))
                .abs()
                < f32::EPSILON
        );
        assert!(
            (state.recent_memory() - factor_value(&state, MoodFactorKind::RecentMemory)).abs()
                < f32::EPSILON
        );
        assert!(
            (state.environment() - factor_value(&state, MoodFactorKind::Environment)).abs()
                < f32::EPSILON
        );
        assert!(
            (state.social_events() - factor_value(&state, MoodFactorKind::SocialEvents)).abs()
                < f32::EPSILON
        );

        // The breakdown must be visible without recomputing (inspectability).
        let _ = state.valence; // explicit field access
        let _ = state.factors; // explicit field access
        assert!(
            !state.factors.is_empty(),
            "factor breakdown must be non-empty"
        );
    }

    /// FR-CIV-PSYCHE-901 — recompute is a pure function of inputs: identical
    /// inputs yield identical valence and identical factor breakdown.
    #[test]
    fn recompute_is_deterministic_for_same_inputs() {
        let inputs = MoodInputs {
            needs: vec![Need::Hunger(0.4), Need::Safety(0.6)],
            memory_events: vec![MemoryEvent::new(50, 0.2), MemoryEvent::new(40, -0.1)],
            environment: 0.0,
            social_events: vec![SocialMoodEvent::new(45, 0.3)],
            current_tick: 60,
            ..MoodInputs::default()
        };

        let a = MoodState::recompute(&inputs, &MoodWeights::default());
        let b = MoodState::recompute(&inputs, &MoodWeights::default());

        assert_eq!(a, b, "identical inputs must produce identical mood states");
    }

    /// FR-CIV-PSYCHE-901 — out-of-window events do not influence mood. Old
    /// positive memories should not lift an otherwise-empty mood.
    #[test]
    fn lookback_window_ignores_old_events() {
        let inputs = MoodInputs {
            needs: vec![],
            memory_events: vec![MemoryEvent::new(0, 1.0)], // ancient
            environment: 0.0,
            social_events: vec![SocialMoodEvent::new(0, 1.0)], // ancient
            current_tick: 10_000,
            memory_window: DEFAULT_MEMORY_WINDOW,
            social_window: DEFAULT_SOCIAL_WINDOW,
        };
        let state = MoodState::recompute(&inputs, &MoodWeights::default());

        assert_eq!(
            factor_value(&state, MoodFactorKind::RecentMemory),
            0.0,
            "old memory events must not influence mood"
        );
        assert_eq!(
            factor_value(&state, MoodFactorKind::SocialEvents),
            0.0,
            "old social events must not influence mood"
        );
    }

    /// FR-CIV-PSYCHE-901 — `factor` returns `None` for an unknown kind rather
    /// than panicking. Also: `MoodState::neutral` already exposes every kind.
    #[test]
    fn factor_lookup_is_safe_on_neutral_state() {
        let state = MoodState::neutral();
        for kind in MoodFactorKind::all() {
            let f = state
                .factor(kind)
                .expect("neutral state must enumerate every kind");
            assert_eq!(f.value, 0.0);
        }
        assert_eq!(state.valence, 0.0);
        assert_eq!(state.label(), "Neutral");
    }

    /// FR-CIV-PSYCHE-901 — valence is clamped into `[-1, 1]` even when the
    /// weighted sum would exceed the range.
    #[test]
    fn valence_is_clamped_to_unit_interval() {
        let inputs = MoodInputs {
            needs: vec![Need::Hunger(0.0)],
            memory_events: vec![MemoryEvent::new(10, 1.0)],
            environment: 1.0,
            social_events: vec![SocialMoodEvent::new(10, 1.0)],
            current_tick: 10,
            ..MoodInputs::default()
        };
        let state = MoodState::recompute(&inputs, &MoodWeights::default());
        assert!(
            (-1.0..=1.0).contains(&state.valence),
            "valence must stay in [-1, 1] (got {})",
            state.valence
        );
    }
}
