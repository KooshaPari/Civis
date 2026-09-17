//! FR-INST-002 — Institutional capture score.
//!
//! Capture score accumulates each tick based on resource concentration.
//! A high capture score indicates that a small elite has captured
//! institutional power, degrading governance quality.
//!
//! All math is integer-saturating. No floats accumulate across calls.

use serde::{Deserialize, Serialize};

/// Basis-point denominator (10_000 bp = 100 %).
const BP_DENOM: i64 = 10_000;

/// Default capture score increase per tick in basis points.
/// With default concentration (50%), this yields 5 bp/tick.
pub const DEFAULT_CAPTURE_RATE_BP: i64 = 10;

/// Maximum capture score (1.0 = 10_000 bp).
pub const MAX_CAPTURE_BP: i64 = 10_000;

/// Minimum capture score (0.0).
pub const MIN_CAPTURE_BP: i64 = 0;

/// Capture threshold for triggering events (0.75 = 7_500 bp).
pub const CAPTURE_THRESHOLD_BP: i64 = 7_500;

/// Configuration for capture score computation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureConfig {
    /// Base capture rate per tick in basis points.
    pub rate_bp: i64,
    /// Threshold at which events are emitted (basis points).
    pub threshold_bp: i64,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            rate_bp: DEFAULT_CAPTURE_RATE_BP,
            threshold_bp: CAPTURE_THRESHOLD_BP,
        }
    }
}

/// Mutable capture score state for a civilization's institutions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureScore {
    /// Current capture score in basis points [0, 10_000].
    pub value_bp: i64,
    /// Threshold at which capture-related events fire.
    pub threshold_bp: i64,
}

impl Default for CaptureScore {
    fn default() -> Self {
        Self {
            value_bp: MIN_CAPTURE_BP,
            threshold_bp: CAPTURE_THRESHOLD_BP,
        }
    }
}

impl CaptureScore {
    /// Creates a new capture score at zero with default threshold.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a capture score with a specific initial value (clamped).
    pub fn with_value(value_bp: i64) -> Self {
        Self {
            value_bp: value_bp.clamp(MIN_CAPTURE_BP, MAX_CAPTURE_BP),
            threshold_bp: CAPTURE_THRESHOLD_BP,
        }
    }

    /// Accumulate capture based on resource concentration.
    ///
    /// `concentration` is a normalized value [0, 1] representing how
    /// concentrated resources are (0 = perfectly distributed, 1 = fully
    /// concentrated in one entity).
    ///
    /// Returns the updated capture score in basis points.
    pub fn accumulate(&mut self, concentration: i64) -> i64 {
        // concentration is expected in basis points [0, 10_000]
        let clamped = concentration.clamp(MIN_CAPTURE_BP, MAX_CAPTURE_BP);
        // capture_delta = rate * concentration / 10_000
        let delta = self.rate_bp() * clamped / BP_DENOM;
        self.value_bp = (self.value_bp + delta).clamp(MIN_CAPTURE_BP, MAX_CAPTURE_BP);
        self.value_bp
    }

    /// Returns the capture rate in basis points.
    pub fn rate_bp(&self) -> i64 {
        DEFAULT_CAPTURE_RATE_BP
    }

    /// Returns the normalized capture score as a fixed-point fraction [0, 10_000].
    pub fn normalized_bp(&self) -> i64 {
        self.value_bp
    }

    /// Returns true when capture score has crossed the threshold.
    pub fn is_captured(&self) -> bool {
        self.value_bp >= self.threshold_bp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulation_increases_capture() {
        let mut cs = CaptureScore::new();
        cs.accumulate(5_000); // 50% concentration
        assert!(cs.value_bp > 0);
    }

    #[test]
    fn zero_concentration_no_capture() {
        let mut cs = CaptureScore::new();
        cs.accumulate(0);
        assert_eq!(cs.value_bp, 0);
    }

    #[test]
    fn full_concentration_max_capture_rate() {
        let mut cs = CaptureScore::new();
        // Full concentration for enough ticks to reach threshold
        for _ in 0..1001 {
            cs.accumulate(MAX_CAPTURE_BP);
        }
        assert!(cs.is_captured());
    }

    #[test]
    fn score_clamped_to_max() {
        let mut cs = CaptureScore::new();
        for _ in 0..100_000 {
            cs.accumulate(MAX_CAPTURE_BP);
        }
        assert!(cs.value_bp <= MAX_CAPTURE_BP);
    }
}
