//! Shannon (and normalised) entropy metric.
//!
//! See `docs/design/emergence-dashboard.md` §3.2 for the design rationale.
//!
//! The metric consumes a [`Histogram`] and returns
//! `H = -Σ p_i log2 p_i` in bits, plus an optional normalised
//! `H / log2 N` accessor. A uniform distribution gives `H = log2 N`; a
//! Dirac gives `H = 0`.
//!
//! ## Why `f32` not `f64`?
//!
//! The dashboard renders these values to the player and to the replay
//! bus; the precision needs are "trend over 4096 ticks", not "exact
//! real number". `f32` is the convention used by `civ-diffusion` and
//! `civ-economy` (see `crates/diffusion/src/lib.rs:11-19`). Replay
//! determinism is preserved by *not* changing the input order.

use crate::{Histogram, Metric};

/// Shannon-entropy metric.
///
/// ```text
/// H = -Σ p_i log2 p_i         (bits)
/// H_norm = H / log2 N         (uniform → 1.0, dirac → 0.0)
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ShannonEntropy;

impl ShannonEntropy {
    /// Construct a new instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Raw Shannon entropy in bits. Returns 0 for empty / all-zero inputs.
    #[must_use]
    pub fn compute_bits(&self, input: &Histogram) -> f32 {
        let mut h = 0.0_f32;
        for i in 0..input.len() {
            let p = input.p(i);
            if p > 0.0 {
                h -= p * p.log2();
            }
        }
        h.max(0.0)
    }

    /// Shannon entropy normalised by the maximum possible entropy for
    /// this alphabet size. Returns 0 for an empty or single-bin histogram.
    #[must_use]
    pub fn compute_normalised(&self, input: &Histogram) -> f32 {
        let n = input.len();
        if n <= 1 {
            return 0.0;
        }
        let max_h = (n as f32).log2();
        if max_h <= 0.0 {
            return 0.0;
        }
        (self.compute_bits(input) / max_h).clamp(0.0, 1.0)
    }
}

impl Metric for ShannonEntropy {
    const NAME: &'static str = "shannon_entropy";

    fn compute(&self, input: &Histogram) -> f32 {
        self.compute_bits(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-4,
            "expected {b} got {a} (|diff|={})",
            (a - b).abs()
        );
    }

    #[test]
    fn empty_histogram_is_zero() {
        let h = Histogram::default();
        assert_eq!(ShannonEntropy.compute_bits(&h), 0.0);
        assert_eq!(ShannonEntropy.compute_normalised(&h), 0.0);
    }

    #[test]
    fn single_bin_is_zero() {
        let h = Histogram::from_counts(vec![42]);
        assert_eq!(ShannonEntropy.compute_bits(&h), 0.0);
        assert_eq!(ShannonEntropy.compute_normalised(&h), 0.0);
    }

    #[test]
    fn uniform_histogram_is_log2_n() {
        let h = Histogram::uniform(4, 100);
        let bits = ShannonEntropy.compute_bits(&h);
        approx(bits, 2.0); // log2(4) = 2
        approx(ShannonEntropy.compute_normalised(&h), 1.0);
    }

    #[test]
    fn uniform_eight_bins_is_three_bits() {
        let h = Histogram::uniform(8, 7);
        let bits = ShannonEntropy.compute_bits(&h);
        approx(bits, 3.0);
        approx(ShannonEntropy.compute_normalised(&h), 1.0);
    }

    #[test]
    fn two_equal_bins_is_one_bit() {
        let h = Histogram::from_counts(vec![1, 1]);
        let bits = ShannonEntropy.compute_bits(&h);
        approx(bits, 1.0);
        approx(ShannonEntropy.compute_normalised(&h), 1.0); // log2(2) = 1
    }

    #[test]
    fn entropy_uniform_vs_peaked() {
        // Uniform 8-bin entropy = log2(8) = 3 bits; peaked (Dirac) = 0 bits.
        let uniform = Histogram::uniform(8, 100);
        let peaked = Histogram::dirac(8, 3, 100);
        let uniform_bits = ShannonEntropy.compute_bits(&uniform);
        let peaked_bits = ShannonEntropy.compute_bits(&peaked);
        approx(uniform_bits, 3.0);
        approx(peaked_bits, 0.0);
        assert!(
            uniform_bits > peaked_bits,
            "uniform entropy ({}) must exceed peaked entropy ({})",
            uniform_bits,
            peaked_bits
        );
        // Normalised: uniform → 1.0, peaked → 0.0.
        approx(ShannonEntropy.compute_normalised(&uniform), 1.0);
        approx(ShannonEntropy.compute_normalised(&peaked), 0.0);
    }

    #[test]
    fn dirac_is_zero() {
        let h = Histogram::dirac(16, 5, 1000);
        let bits = ShannonEntropy.compute_bits(&h);
        approx(bits, 0.0);
        approx(ShannonEntropy.compute_normalised(&h), 0.0);
    }

    #[test]
    fn skewed_distribution_between_dirac_and_uniform() {
        // 90/10 split on 2 bins: H = -0.9 log2 0.9 - 0.1 log2 0.1 ≈ 0.4690
        let h = Histogram::from_counts(vec![9, 1]);
        let bits = ShannonEntropy.compute_bits(&h);
        approx(bits, 0.4690);
        let norm = ShannonEntropy.compute_normalised(&h);
        approx(norm, 0.4690);
    }

    #[test]
    fn synthetic_uniform_across_many_bins() {
        // 16 bins, 1024 samples each → uniform → 4 bits.
        let h = Histogram::uniform(16, 1024);
        let bits = ShannonEntropy.compute_bits(&h);
        approx(bits, 4.0);
        approx(ShannonEntropy.compute_normalised(&h), 1.0);
    }

    #[test]
    fn metric_name_is_stable() {
        assert_eq!(ShannonEntropy::NAME, "shannon_entropy");
    }

    #[test]
    fn metric_trait_returns_bits() {
        // Same answer through the trait. We exercise the trait via a
        // generic bound rather than `&dyn Metric` because `Metric` is
        // not dyn-compatible: the per-type `NAME` associated const is
        // part of the public contract (used as the JSON key on
        // `sim.snapshot`) and we want it to be a stable, type-level
        // property rather than a vtable slot.
        fn dispatch<M: Metric>(m: &M, h: &Histogram) -> f32 {
            m.compute(h)
        }
        let h = Histogram::uniform(4, 10);
        let v = dispatch(&ShannonEntropy, &h);
        approx(v, 2.0);
    }
}
