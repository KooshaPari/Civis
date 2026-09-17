//! FR-CLIM-005 — Tipping-point cascade modelling.
//!
//! Above a critical temperature, the climate system can undergo irreversible
//! state changes (tipping points). Each tipping point has a trigger threshold
//! and amplifying feedback effects. Once triggered, a cascade can activate
//! dependent tipping points.
//!
//! Modelled tipping points:
//! - **Ice-albedo**: Arctic sea ice loss reduces reflectivity, amplifying warming.
//! - **Permafrost**: Thaw releases methane, adding to greenhouse forcing.
//! - **Amazon dieback**: Forest loss reduces carbon sink, raising CO₂.

use serde::{Deserialize, Serialize};

/// A named tipping point in the climate system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TippingPoint {
    /// Arctic ice-albedo feedback: ice loss → lower albedo → more warming.
    IceAlbedo,
    /// Permafrost thaw releases trapped methane.
    Permafrost,
    /// Amazon rainforest dieback reduces carbon sink.
    AmazonDieback,
}

/// Configuration for a single tipping point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TippingPointConfig {
    /// The tipping point identity.
    pub point: TippingPoint,
    /// Temperature anomaly (°C) that triggers this tipping point.
    pub trigger_anomaly_c: f64,
    /// Amplifying feedback: additional °C per tick once triggered (decays).
    pub feedback_amplification_c: f64,
    /// Additional CO₂ ppm equivalent released per tick while active.
    pub co2_release_ppm: f64,
    /// Tick at which this tipping point becomes active (set by trigger).
    pub activated_at_tick: Option<u64>,
    /// Whether this tipping point has been triggered.
    pub triggered: bool,
}

impl TippingPointConfig {
    pub fn ice_albedo() -> Self {
        Self {
            point: TippingPoint::IceAlbedo,
            trigger_anomaly_c: 2.5,
            feedback_amplification_c: 0.15,
            co2_release_ppm: 0.0,
            activated_at_tick: None,
            triggered: false,
        }
    }

    pub fn permafrost() -> Self {
        Self {
            point: TippingPoint::Permafrost,
            trigger_anomaly_c: 3.0,
            feedback_amplification_c: 0.05,
            co2_release_ppm: 0.5,
            activated_at_tick: None,
            triggered: false,
        }
    }

    pub fn amazon_dieback() -> Self {
        Self {
            point: TippingPoint::AmazonDieback,
            trigger_anomaly_c: 3.5,
            feedback_amplification_c: 0.08,
            co2_release_ppm: 0.3,
            activated_at_tick: None,
            triggered: false,
        }
    }
}

/// Result of a cascade evaluation for one tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CascadeResult {
    /// Total additional temperature amplification from all active tipping points (°C).
    pub amplification_c: f64,
    /// Total additional CO₂ released from all active tipping points (ppm).
    pub co2_release_ppm: f64,
    /// List of tipping points that were newly triggered this tick.
    pub newly_triggered: Vec<TippingPoint>,
}

/// Manages the full set of tipping points and cascade logic.
#[derive(Debug, Clone)]
pub struct CascadeTracker {
    /// The tipping point configurations.
    pub tipping_points: Vec<TippingPointConfig>,
}

impl CascadeTracker {
    /// Create with default tipping points (ice-albedo, permafrost, amazon).
    pub fn new() -> Self {
        Self {
            tipping_points: vec![
                TippingPointConfig::ice_albedo(),
                TippingPointConfig::permafrost(),
                TippingPointConfig::amazon_dieback(),
            ],
        }
    }

    /// Evaluate all tipping points against the current temperature anomaly.
    ///
    /// Newly triggered points are recorded. All triggered points contribute
    /// their amplification and CO₂ release.
    pub fn evaluate(&mut self, anomaly_c: f64, tick: u64) -> CascadeResult {
        let mut amplification_c = 0.0;
        let mut co2_release = 0.0;
        let mut newly_triggered = Vec::new();

        for tp in &mut self.tipping_points {
            if !tp.triggered && anomaly_c >= tp.trigger_anomaly_c {
                tp.triggered = true;
                tp.activated_at_tick = Some(tick);
                newly_triggered.push(tp.point);
            }
            if tp.triggered {
                amplification_c += tp.feedback_amplification_c;
                co2_release += tp.co2_release_ppm;
            }
        }

        CascadeResult {
            amplification_c,
            co2_release_ppm: co2_release,
            newly_triggered,
        }
    }

    /// Whether a specific tipping point has been triggered.
    pub fn is_triggered(&self, point: TippingPoint) -> bool {
        self.tipping_points
            .iter()
            .any(|tp| tp.point == point && tp.triggered)
    }

    /// Number of triggered tipping points.
    pub fn triggered_count(&self) -> usize {
        self.tipping_points.iter().filter(|tp| tp.triggered).count()
    }
}

impl Default for CascadeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_tipping_below_thresholds() {
        let mut tracker = CascadeTracker::new();
        let result = tracker.evaluate(1.0, 10);
        assert!(result.newly_triggered.is_empty());
        assert_eq!(result.amplification_c, 0.0);
        assert_eq!(result.co2_release_ppm, 0.0);
    }

    #[test]
    fn ice_albedo_triggers_at_2_5() {
        let mut tracker = CascadeTracker::new();
        let result = tracker.evaluate(2.5, 10);
        assert!(result.newly_triggered.contains(&TippingPoint::IceAlbedo));
        assert!(tracker.is_triggered(TippingPoint::IceAlbedo));
    }

    #[test]
    fn cascade_all_three_at_high_anomaly() {
        let mut tracker = CascadeTracker::new();
        let result = tracker.evaluate(5.0, 100);
        assert_eq!(result.newly_triggered.len(), 3);
        assert_eq!(tracker.triggered_count(), 3);
    }

    #[test]
    fn triggered_points_do_not_re_trigger() {
        let mut tracker = CascadeTracker::new();
        tracker.evaluate(3.0, 10);
        let result = tracker.evaluate(3.0, 11);
        assert!(
            result.newly_triggered.is_empty(),
            "already triggered points should not re-trigger"
        );
    }

    #[test]
    fn amplification_accumulates() {
        let mut tracker = CascadeTracker::new();
        let r1 = tracker.evaluate(2.5, 10);
        let r2 = tracker.evaluate(3.5, 11);
        // r2 should have more amplification because amazon dieback activated
        assert!(r2.amplification_c > r1.amplification_c);
    }

    #[test]
    fn sequential_triggering() {
        let mut tracker = CascadeTracker::new();
        let r1 = tracker.evaluate(2.0, 10);
        assert!(r1.newly_triggered.is_empty());
        let r2 = tracker.evaluate(2.6, 11);
        assert_eq!(r2.newly_triggered.len(), 1); // ice-albedo
        let r3 = tracker.evaluate(3.1, 12);
        assert_eq!(r3.newly_triggered.len(), 1); // permafrost
        assert_eq!(tracker.triggered_count(), 2);
    }
}
