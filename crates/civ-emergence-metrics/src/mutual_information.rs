//! Coupling mutual-information metric.
//!
//! Computes I(A;B) = Σ_{a,b} p(a,b) log2 [p(a,b) / (p(a)·p(b))] in bits
//! over a [`JointHistogram`] of two categorical layers A and B, plus a
//! normalised variant I(A;B) / H(A) (clamped to [0,1]).
//!
//! See `docs/design/emergence-dashboard.md` §3.5 for design rationale.
//!
//! ## Determinism
//!
//! No `HashMap`, no parallel iteration, no platform-dependent math. The
//! loop order is deterministic (row-major over (a, b)) so two runs of the
//! same seed produce the same value.

use crate::{Histogram, shannon::ShannonEntropy};

/// Joint histogram over two categorical layers A (rows) and B (cols).
///
/// Maintain one of these per layer-pair per sample; call [`observe`] once
/// per voxel/citizen with the binned values for layers A and B.
#[derive(Debug, Clone, PartialEq)]
pub struct JointHistogram {
    bins: Vec<u64>,
    rows: usize,
    cols: usize,
}

impl JointHistogram {
    /// Create a zero-initialised joint histogram with `rows × cols` cells.
    #[must_use]
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            bins: vec![0u64; rows * cols],
            rows,
            cols,
        }
    }

    /// Record one observation of (layer-A bin `a`, layer-B bin `b`).
    ///
    /// Out-of-range observations are silently dropped so callers need
    /// not bounds-check before calling.
    pub fn observe(&mut self, a: usize, b: usize) {
        if a < self.rows && b < self.cols {
            self.bins[a * self.cols + b] =
                self.bins[a * self.cols + b].saturating_add(1);
        }
    }

    /// Total number of observations (sum of all cells).
    #[must_use]
    pub fn total(&self) -> u64 {
        self.bins.iter().sum()
    }

    /// Joint probability p(a, b). Returns 0 for out-of-range indices or
    /// when the histogram is empty.
    #[must_use]
    pub fn p_joint(&self, a: usize, b: usize) -> f32 {
        let t = self.total();
        if t == 0 || a >= self.rows || b >= self.cols {
            return 0.0;
        }
        self.bins[a * self.cols + b] as f32 / t as f32
    }

    /// Marginal histogram over layer A (sum across all B bins).
    #[must_use]
    pub fn marginal_a(&self) -> Histogram {
        let mut counts = vec![0u64; self.rows];
        for a in 0..self.rows {
            for b in 0..self.cols {
                counts[a] = counts[a].saturating_add(self.bins[a * self.cols + b]);
            }
        }
        Histogram::from_counts(counts)
    }

    /// Marginal histogram over layer B (sum across all A bins).
    #[must_use]
    pub fn marginal_b(&self) -> Histogram {
        let mut counts = vec![0u64; self.cols];
        for a in 0..self.rows {
            for b in 0..self.cols {
                counts[b] = counts[b].saturating_add(self.bins[a * self.cols + b]);
            }
        }
        Histogram::from_counts(counts)
    }

    /// Number of rows (layer-A alphabet size).
    #[must_use]
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of columns (layer-B alphabet size).
    #[must_use]
    pub fn cols(&self) -> usize {
        self.cols
    }
}

/// Mutual information I(A;B) in bits.
///
/// Returns 0.0 for an empty histogram (no observations) or when the joint
/// distribution factors as a product of its marginals (independence).
///
/// The result is clamped to `≥ 0.0` to absorb floating-point rounding
/// errors that can produce tiny negative values for near-independent
/// distributions.
#[must_use]
pub fn mutual_information_bits(joint: &JointHistogram) -> f32 {
    if joint.total() == 0 {
        return 0.0;
    }
    let ma = joint.marginal_a();
    let mb = joint.marginal_b();
    let mut mi = 0.0f32;
    for a in 0..joint.rows {
        let pa = ma.p(a);
        if pa <= 0.0 {
            continue;
        }
        for b in 0..joint.cols {
            let pb = mb.p(b);
            if pb <= 0.0 {
                continue;
            }
            let pab = joint.p_joint(a, b);
            if pab <= 0.0 {
                continue;
            }
            mi += pab * (pab / (pa * pb)).log2();
        }
    }
    mi.max(0.0)
}

/// Normalised mutual information NMI = I(A;B) / H(A), clamped to [0, 1].
///
/// When H(A) = 0 (layer A is degenerate — all observations in a single bin)
/// there is no information to share and the function returns 0.0.
///
/// A value of 1.0 means B is fully determined by A (deterministic coupling).
#[must_use]
pub fn mutual_information_normalised(joint: &JointHistogram) -> f32 {
    let h = ShannonEntropy::new().compute_bits(&joint.marginal_a());
    if h <= 0.0 {
        return 0.0;
    }
    (mutual_information_bits(joint) / h).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mi_zero_for_independent_layers() {
        // Build outer product of two marginals: p(a,b) = p(a)*p(b)
        // Use 2x2 with equal weights
        let mut joint = JointHistogram::new(2, 2);
        // Equal distribution: 25 each cell
        for _ in 0..25 {
            joint.observe(0, 0);
        }
        for _ in 0..25 {
            joint.observe(0, 1);
        }
        for _ in 0..25 {
            joint.observe(1, 0);
        }
        for _ in 0..25 {
            joint.observe(1, 1);
        }
        let mi = mutual_information_bits(&joint);
        assert!(
            mi.abs() < 1e-4,
            "MI should be ~0 for independent, got {}",
            mi
        );
    }

    #[test]
    fn mi_max_for_deterministic_coupling() {
        // Diagonal joint: b = a deterministically
        let mut joint = JointHistogram::new(4, 4);
        for _ in 0..100 {
            joint.observe(0, 0);
        }
        for _ in 0..100 {
            joint.observe(1, 1);
        }
        for _ in 0..100 {
            joint.observe(2, 2);
        }
        for _ in 0..100 {
            joint.observe(3, 3);
        }
        let mi_norm = mutual_information_normalised(&joint);
        assert!(
            (mi_norm - 1.0).abs() < 1e-4,
            "Normalised MI should be ~1.0 for deterministic, got {}",
            mi_norm
        );
    }

    #[test]
    fn mi_handles_empty_degenerate() {
        // Empty
        let empty = JointHistogram::new(3, 3);
        assert_eq!(mutual_information_bits(&empty), 0.0);
        assert!(mutual_information_bits(&empty).is_finite());

        // Single bin
        let mut single = JointHistogram::new(1, 1);
        single.observe(0, 0);
        let mi = mutual_information_bits(&single);
        assert_eq!(mi, 0.0);
        assert!(mi.is_finite());

        // Normalised also 0
        assert_eq!(mutual_information_normalised(&single), 0.0);
    }
}
