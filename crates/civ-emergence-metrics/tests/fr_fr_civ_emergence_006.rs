//! Tests for FR-CIV-EMERGENCE-006 — structure count.
//!
//! Epic: FR-CIV-EMERGENCE
//! Status: IMPLEMENTED

#[cfg(test)]
mod fr_fr_civ_emergence_006 {
    use civ_emergence_metrics::structure::{Grid, StructureCount};
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
        // A solid 2×2×2 block is one 6-connected component.
        let data = vec![1u8; 2 * 2 * 2];
        let grid = Grid::new(2, 2, 2, &data).expect("2³ grid");
        let summary = StructureCount.evaluate(&grid, |&v| v > 0);
        assert_eq!(summary.count, 1, "solid block should be 1 component");
        assert_eq!(summary.largest, 8);
    }
}
