//! Tests for FR-CIV-EMERGENCE-006 — structure count.
//!
//! Epic: FR-CIV-EMERGENCE
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_emergence_006 {
    use civ_emergence_metrics::structure::StructureCount;
    use civ_emergence_metrics::{Histogram, Metric};

    #[test]
    fn verify_fr_civ_emergence_006_basic() {
        // Single connected component of active cells
        let sc = StructureCount::new();
        // Empty histogram → 0 structures
        let h = Histogram::from_counts(vec![0, 0, 0]);
        let count = sc.compute(&h);
        assert_eq!(count, 0.0);
    }

    #[test]
    fn all_active_single_component() {
        let sc = StructureCount::new();
        let h = Histogram::from_counts(vec![1, 1, 1, 1]);
        let count = sc.compute(&h);
        assert!(count >= 1.0, "should find at least 1 structure");
    }
}
