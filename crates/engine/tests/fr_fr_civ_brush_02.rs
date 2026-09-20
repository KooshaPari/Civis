//! Tests for FR-CIV-BRUSH-02
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-BRUSH-02.

#[cfg(test)]
mod fr_fr_civ_brush_02 {
    use civ_engine::brush_types::BrushCluster;

    /// FR-CIV-BRUSH-02 — Nine distinct brush clusters exist in the toolbar order.
    #[test]
    fn verify_fr_civ_brush_02_basic() {
        let clusters = BrushCluster::all();
        assert_eq!(clusters.len(), 9, "must have exactly 9 clusters");
        assert_eq!(clusters[0], BrushCluster::Select);
        assert_eq!(clusters[1], BrushCluster::Material);
        assert_eq!(clusters[2], BrushCluster::Terraform);
        assert_eq!(clusters[3], BrushCluster::Life);
        assert_eq!(clusters[4], BrushCluster::Structure);
        assert_eq!(clusters[5], BrushCluster::Infrastructure);
        assert_eq!(clusters[6], BrushCluster::Disaster);
        assert_eq!(clusters[7], BrushCluster::Diplomacy);
        assert_eq!(clusters[8], BrushCluster::Policy);
    }
}
