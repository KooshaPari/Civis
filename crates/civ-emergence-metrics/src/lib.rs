//! `civ-emergence-metrics` — weak-emergence / criticality metrics for Civis.
//!
//! This crate ships the pure-math primitives that the *Emergence Dashboard*
//! (see `docs/design/emergence-dashboard.md`) consumes. It has no I/O, no
//! async runtime, and no dependency on the engine / server / voxel crates:
//! each metric takes a snapshot of a histogram or grid and returns a
//! number. That keeps it cheap, deterministic, and trivially testable in
//! isolation.
//!
//! The crate ships two metrics today:
//!
//! * [`shannon::ShannonEntropy`] — Shannon (and normalised) entropy of a
//!   categorical-state histogram. Used to detect heat-death (collapsed
//!   state distribution) and single-bin takeover.
//! * [`structure::StructureCount`] — 6-connectivity connected components on
//!   a sampled 3-D grid, with a union-find core. Used to detect structure
//!   collapse and single-cluster domination.
//! * [`branching`] — per-avalanche branching ratio `σ_a` and rolling-mean
//!   `σ̄_W` for SOC heat-death / edge-of-chaos / explosion discrimination.
//!
//! Both implement the [`Metric`] trait so the dashboard can iterate over a
//! uniform interface.
//!
//! ## Determinism
//!
//! Everything is `f32` (or `usize` for counts) and there is no `HashMap`,
//! no parallel iteration, and no platform-dependent math. Replay
//! determinism follows from the engine feeding the same input sequence;
//! this crate does not store any mutable state between calls except what
//! the caller hands it.
//!
//! ## Why no external deps?
//!
//! Each metric is tens of lines of std-only code. Adding `ndarray` or
//! `petgraph` would inflate compile time and the `cargo audit` surface
//! area for a crate that has to ship inside the binary that already
//! pulls in `hecs`, `bincode`, `zstd`, `blake3`, `sha2`, `chrono`, etc.
//!
//! ## Scope
//!
//! The remaining dashboard metrics (power-law fit, novelty rate) are scoped
//! for follow-up PRs once the wiring exists in `civ-server` and
//! `civ-protocol-3d`. Mutual information between sim layers is implemented
//! in [`mutual_information`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod branching;
pub mod criticality;
pub mod dashboard;
pub mod mutual_information;
pub mod power_law;
pub mod shannon;
pub mod sample_snapshot;
pub mod structure;

pub use mutual_information::{
    mutual_information_bits, mutual_information_normalised, JointHistogram,
};

pub use criticality::{criticality_indicator, CriticalityBands, CriticalityInputs};

/// Marker version of this crate's public schema. Bumped on breaking changes.
pub const SCHEMA_VERSION: &str = "0.5.0-power-law-alpha";

/// Common interface implemented by every metric in this crate.
///
/// The trait is intentionally minimal: a metric is a *pure function* from
/// some input to a number. The engine feeds inputs by constructing a
/// concrete metric type (e.g. [`shannon::ShannonEntropy`]) and calling
/// [`Metric::compute`].
pub trait Metric {
    /// Human-readable name. Used as the JSON key on `sim.snapshot` and
    /// as the `kind` tag on `emergence.metrics.v1` replay events.
    const NAME: &'static str;

    /// Compute the metric on `input` and return the scalar result.
    ///
    /// Implementations must be deterministic and side-effect-free.
    fn compute(&self, input: &Histogram) -> f32;
}

/// A categorical-state histogram. The dashboard constructs one of these
/// per "layer" per snapshot (voxel material, civilian faction, market
/// good, building type, …) and hands it to each metric.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Histogram {
    /// Raw counts per bin. `bins.len()` is the alphabet size for this
    /// layer; the metric does not interpret the index.
    bins: Vec<u64>,
}

impl Histogram {
    /// Build a histogram from raw counts.
    ///
    /// # Examples
    ///
    /// ```
    /// use civ_emergence_metrics::Histogram;
    /// let h = Histogram::from_counts(vec![1, 2, 3, 4]);
    /// assert_eq!(h.total(), 10);
    /// ```
    #[must_use]
    pub fn from_counts(bins: Vec<u64>) -> Self {
        Self { bins }
    }

    /// Build a uniform histogram with `bins` bins, each with `count` items.
    #[must_use]
    pub fn uniform(bins: usize, count: u64) -> Self {
        Self {
            bins: vec![count; bins],
        }
    }

    /// Build a Dirac (single-bin) histogram.
    #[must_use]
    pub fn dirac(bins: usize, hot: usize, count: u64) -> Self {
        let mut v = vec![0u64; bins];
        if bins > 0 {
            v[hot.min(bins - 1)] = count;
        }
        Self { bins: v }
    }

    /// Total number of samples (sum of all bin counts).
    #[must_use]
    pub fn total(&self) -> u64 {
        self.bins.iter().sum()
    }

    /// Number of bins (alphabet size).
    #[must_use]
    pub fn len(&self) -> usize {
        self.bins.len()
    }

    /// `true` iff the histogram has no bins.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bins.is_empty()
    }

    /// Probability of bin `i`; 0 for out-of-range indices. Returns 0
    /// when the histogram is empty.
    #[must_use]
    pub fn p(&self, i: usize) -> f32 {
        if i >= self.bins.len() {
            return 0.0;
        }
        let total = self.total();
        if total == 0 {
            return 0.0;
        }
        self.bins[i] as f32 / total as f32
    }

    /// Borrow the raw counts.
    #[must_use]
    pub fn bins(&self) -> &[u64] {
        &self.bins
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_total_and_p() {
        let h = Histogram::from_counts(vec![1, 1, 2]);
        assert_eq!(h.total(), 4);
        assert!((h.p(0) - 0.25).abs() < 1e-6);
        assert!((h.p(2) - 0.5).abs() < 1e-6);
        assert_eq!(h.p(99), 0.0);
    }

    #[test]
    fn empty_histogram_p_is_zero() {
        let h = Histogram::default();
        assert_eq!(h.p(0), 0.0);
        assert_eq!(h.total(), 0);
    }

    #[test]
    fn uniform_and_dirac() {
        let u = Histogram::uniform(4, 7);
        assert_eq!(u.total(), 28);
        for i in 0..4 {
            assert!((u.p(i) - 0.25).abs() < 1e-6);
        }
        let d = Histogram::dirac(4, 2, 9);
        assert_eq!(d.total(), 9);
        assert!((d.p(2) - 1.0).abs() < 1e-6);
        assert!((d.p(0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn dirac_clamps_hot_index() {
        let d = Histogram::dirac(2, 99, 5);
        // Out-of-range `hot` should clamp to last bin rather than panic.
        assert_eq!(d.bins(), &[0, 5]);
    }
}
