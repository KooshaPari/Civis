//! Tests for FR-CIV-BRUSH-04
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-BRUSH-04.

#[cfg(test)]
mod fr_fr_civ_brush_04 {
    use civ_engine::brush_types::{ActionKind, BrushCluster};

    /// FR-CIV-BRUSH-04 -- ActionKind::cluster() routes to correct cluster.
    #[test]
    fn verify_fr_civ_brush_04_basic() {
        assert_eq!(ActionKind::MaterialReplace.cluster(), BrushCluster::Material);
        assert_eq!(ActionKind::TerraformRaise.cluster(), BrushCluster::Terraform);
        assert_eq!(ActionKind::LifeBless.cluster(), BrushCluster::Life);
        assert_eq!(ActionKind::Select.cluster(), BrushCluster::Select);
        assert_eq!(ActionKind::PolicyTax.cluster(), BrushCluster::Policy);
    }
}
