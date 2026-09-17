//! Tests for FR-CIV-EMERGENCE-010 — Shannon entropy.
//!
//! Epic: FR-CIV-EMERGENCE
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_emergence_010 {
    use civ_emergence_metrics::shannon::ShannonEntropy;
    use civ_emergence_metrics::{Histogram, Metric};

    #[test]
    fn verify_fr_civ_emergence_010_basic() {
        let se = ShannonEntropy::new();
        // Uniform distribution → max entropy
        let h = Histogram::uniform(4, 25);
        let e = se.compute(&h);
        assert!(e > 1.9, "4-bin uniform should have entropy ~2.0, got {e}");
    }

    #[test]
    fn dirac_distribution_zero_entropy() {
        let se = ShannonEntropy::new();
        let h = Histogram::dirac(100, 0, 1);
        let e = se.compute(&h);
        assert_eq!(e, 0.0, "single-bin should have zero entropy");
    }

    #[test]
    fn empty_histogram_zero_entropy() {
        let se = ShannonEntropy::new();
        let h = Histogram::from_counts(vec![]);
        let e = se.compute(&h);
        assert_eq!(e, 0.0);
    }
}
