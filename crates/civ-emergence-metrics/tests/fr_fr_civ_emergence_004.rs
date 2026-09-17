//! Tests for FR-CIV-EMERGENCE-004 — mutual information.
//!
//! Epic: FR-CIV-EMERGENCE
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_emergence_004 {
    use civ_emergence_metrics::{mutual_information_bits, mutual_information_normalised, JointHistogram};

    #[test]
    fn verify_fr_civ_emergence_004_basic() {
        // Perfectly correlated: MI should be high
        let mut jh = JointHistogram::new(3, 3);
        for i in 0..100 {
            jh.observe(i % 3, i % 3);
        }
        let mi = mutual_information_bits(&jh);
        assert!(mi > 1.0, "perfect correlation should have high MI, got {mi}");
    }

    #[test]
    fn independent_variables_mi_near_zero() {
        let mut jh = JointHistogram::new(3, 3);
        // Fill uniformly: each cell gets ~11 observations
        for a in 0..3 {
            for b in 0..3 {
                for _ in 0..11 {
                    jh.observe(a, b);
                }
            }
        }
        let mi = mutual_information_bits(&jh);
        assert!(mi.abs() < 0.1, "independent should have near-zero MI, got {mi}");
    }

    #[test]
    fn normalised_mi_bounded() {
        let mut jh = JointHistogram::new(4, 4);
        for i in 0..200 {
            jh.observe(i % 4, i % 4);
        }
        let nmi = mutual_information_normalised(&jh);
        assert!(nmi >= 0.0 && nmi <= 1.0, "NMI should be in [0,1], got {nmi}");
    }
}
