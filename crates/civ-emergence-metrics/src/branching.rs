//! Rolling-mean branching ratio `σ̄` for SOC criticality (charter §3.6).
//!
//! Per-avalanche ratio `σ_a = N_descendants / N_actors` with a zero-actor
//! guard; the dashboard primary scalar is the rolling mean over the last
//! `W` closed avalanches.

use serde::{Deserialize, Serialize};

/// Default rolling window (AC-002 consecutive-avalanche count).
pub const DEFAULT_BRANCHING_WINDOW: usize = 10;

/// Subcritical / heat-death threshold (`σ̄ < 0.85`, AC-005).
pub const SIGMA_SUBCRITICAL: f32 = 0.85;
/// Lower edge of the SOC target band.
pub const SIGMA_EDGE_LOW: f32 = 0.95;
/// Upper edge of the SOC target band.
pub const SIGMA_EDGE_HIGH: f32 = 0.99;
/// Supercritical threshold (`σ̄ > 1.0`, AC-002).
pub const SIGMA_SUPERCRITICAL: f32 = 1.0;

/// Named SOC regime for dashboard labelling and alarms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchingRegime {
    /// `σ̄ < 0.85` — subcritical / heat-death risk.
    HeatDeath,
    /// `[0.85, 0.95)` — warming toward criticality.
    SubcriticalTransition,
    /// `[0.95, 0.99]` — edge-of-chaos target band.
    EdgeOfChaos,
    /// `(0.99, 1.0]` — near-supercritical watch band.
    NearSupercritical,
    /// `σ̄ > 1.0` — supercritical / explosion risk.
    Supercritical,
}

impl BranchingRegime {
    /// Stable wire/dashboard label for this regime.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::HeatDeath => "Subcritical (heat-death risk)",
            Self::SubcriticalTransition => "Subcritical → critical transition",
            Self::EdgeOfChaos => "Edge of chaos (target)",
            Self::NearSupercritical => "Near-supercritical",
            Self::Supercritical => "Supercritical (explosion risk)",
        }
    }
}

/// One closed avalanche entry in the ledger.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ClosedAvalanche {
    sigma_a: f32,
    size: u64,
    close_tick: u64,
}

/// Fixed-capacity ring buffer of closed `(σ_a, s_a, close_tick)` tuples.
#[derive(Debug, Clone, PartialEq)]
pub struct BranchingLedger {
    entries: Vec<ClosedAvalanche>,
    capacity: usize,
    start: usize,
    len: usize,
    closed_total: u64,
}

impl Default for BranchingLedger {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_BRANCHING_WINDOW.max(1))
    }
}

impl BranchingLedger {
    /// Ring buffer sized for `capacity` closed avalanches (`capacity >= 1`).
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            entries: vec![
                ClosedAvalanche {
                    sigma_a: 0.0,
                    size: 0,
                    close_tick: 0,
                };
                capacity
            ],
            capacity,
            start: 0,
            len: 0,
            closed_total: 0,
        }
    }

    /// Number of closed avalanches currently retained (≤ capacity).
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// `true` when no closed avalanches are retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Total closed avalanches ever pushed (monotonic diagnostic counter).
    #[must_use]
    pub fn closed_total(&self) -> u64 {
        self.closed_total
    }

    /// Append a closed avalanche; evict the oldest entry at capacity.
    pub fn push_closed(&mut self, sigma_a: f32, size: u64, close_tick: u64) {
        let slot = if self.len < self.capacity {
            let idx = (self.start + self.len) % self.capacity;
            self.len += 1;
            idx
        } else {
            let idx = self.start;
            self.start = (self.start + 1) % self.capacity;
            idx
        };
        self.entries[slot] = ClosedAvalanche {
            sigma_a,
            size,
            close_tick,
        };
        self.closed_total = self.closed_total.saturating_add(1);
    }

    fn entry(&self, offset: usize) -> ClosedAvalanche {
        debug_assert!(offset < self.len);
        self.entries[(self.start + offset) % self.capacity]
    }
}

/// Per-avalanche branching ratio with zero-actor guard.
///
/// Returns `0.0` when `actors == 0` (caller must not push to the ledger).
#[must_use]
pub fn sigma_a(actors: u32, descendants: u32) -> f32 {
    if actors == 0 {
        return 0.0;
    }
    descendants as f32 / actors as f32
}

/// Rolling mean `σ̄_W` over the last `window` closed avalanches.
///
/// Returns `0.0` when the ledger is empty.
#[must_use]
pub fn rolling_mean_sigma(ledger: &BranchingLedger, window: usize) -> f32 {
    if ledger.is_empty() || window == 0 {
        return 0.0;
    }
    let take = window.min(ledger.len());
    let start = ledger.len().saturating_sub(take);
    let mut sum = 0.0_f32;
    for offset in start..ledger.len() {
        sum += ledger.entry(offset).sigma_a;
    }
    sum / take as f32
}

/// Normalised edge-of-chaos score in `[0, 1]` (clamped above the band).
#[must_use]
pub fn sigma_score(sigma_bar: f32) -> f32 {
    let span = SIGMA_EDGE_HIGH - SIGMA_SUBCRITICAL;
    if span <= 0.0 {
        return 0.0;
    }
    let raw = (sigma_bar - SIGMA_SUBCRITICAL) / span;
    raw.clamp(0.0, 1.0)
}

/// Classify `σ̄_W` into the charter regime bands.
#[must_use]
pub fn classify_regime(sigma_bar: f32) -> BranchingRegime {
    if sigma_bar < SIGMA_SUBCRITICAL {
        BranchingRegime::HeatDeath
    } else if sigma_bar < SIGMA_EDGE_LOW {
        BranchingRegime::SubcriticalTransition
    } else if sigma_bar <= SIGMA_EDGE_HIGH {
        BranchingRegime::EdgeOfChaos
    } else if sigma_bar <= SIGMA_SUPERCRITICAL {
        BranchingRegime::NearSupercritical
    } else {
        BranchingRegime::Supercritical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sigma_a_ratio_and_zero_actor_guard() {
        assert!((sigma_a(10, 9) - 0.9).abs() < 1e-6);
        assert_eq!(sigma_a(0, 5), 0.0);
    }

    #[test]
    fn rolling_mean_sigma_window_four() {
        let mut ledger = BranchingLedger::with_capacity(8);
        for sigma in [0.8, 0.9, 1.1, 0.95] {
            ledger.push_closed(sigma, 1, 0);
        }
        let mean = rolling_mean_sigma(&ledger, 4);
        assert!((mean - 0.9375).abs() < 1e-6, "expected 0.9375, got {mean}");
    }

    #[test]
    fn sigma_score_endpoints_and_clamp() {
        assert!((sigma_score(SIGMA_SUBCRITICAL) - 0.0).abs() < 1e-6);
        assert!((sigma_score(SIGMA_EDGE_HIGH) - 1.0).abs() < 1e-6);
        assert!((sigma_score(1.1) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn rolling_mean_is_deterministic_for_same_push_sequence() {
        let mut a = BranchingLedger::with_capacity(4);
        let mut b = BranchingLedger::with_capacity(4);
        for (sigma, size, tick) in [(0.7, 3, 1), (1.2, 8, 2), (0.95, 5, 3)] {
            a.push_closed(sigma, size, tick);
            b.push_closed(sigma, size, tick);
        }
        assert_eq!(rolling_mean_sigma(&a, 3), rolling_mean_sigma(&b, 3));
    }

    /// Known closed-avalanche stream → expected `σ̄` and regime classification.
    #[test]
    fn known_stream_yields_expected_sigma_bar_and_regime() {
        let stream = [(10, 8), (10, 9), (10, 10), (10, 11)];
        let mut ledger = BranchingLedger::with_capacity(stream.len());
        for (actors, descendants) in stream {
            let sigma = sigma_a(actors, descendants);
            ledger.push_closed(sigma, u64::from(descendants), 0);
        }
        let sigma_bar = rolling_mean_sigma(&ledger, stream.len());
        assert!(
            (sigma_bar - 0.95).abs() < 1e-6,
            "expected σ̄=0.95, got {sigma_bar}"
        );
        assert_eq!(classify_regime(sigma_bar), BranchingRegime::EdgeOfChaos);

        let heat_death_bar = rolling_mean_sigma(
            &{
                let mut l = BranchingLedger::with_capacity(2);
                l.push_closed(0.8, 4, 1);
                l.push_closed(0.7, 3, 2);
                l
            },
            2,
        );
        assert_eq!(classify_regime(heat_death_bar), BranchingRegime::HeatDeath);

        let explosion_bar = rolling_mean_sigma(
            &{
                let mut l = BranchingLedger::with_capacity(2);
                l.push_closed(1.1, 12, 3);
                l.push_closed(1.2, 15, 4);
                l
            },
            2,
        );
        assert_eq!(
            classify_regime(explosion_bar),
            BranchingRegime::Supercritical
        );
    }
}
