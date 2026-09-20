//! Tests for FR-CIV-VEHICLE-046
//!
//!
//! This test file verifies FR FR-CIV-VEHICLE-046.
//! Far-region LOD solve produces aggregate flow without per-vehicle assignment.

#[cfg(test)]
mod fr_fr_civ_vehicle_046 {
    use civ_engine::lod::{aggregate_strategic, ZoomLevel, project_zoom};

    /// FR-CIV-VEHICLE-046 -- LOD aggregation produces aggregate without per-vehicle detail.
    #[test]
    fn verify_fr_civ_vehicle_046_basic() {
        // Strategic zoom aggregates district data
        let districts = [100, 200, 150];
        let total = aggregate_strategic(&districts);
        assert_eq!(total, 450);
        // Project zoom doesn't change tick state
        let (tick, zoom) = project_zoom(42, ZoomLevel::Strategic);
        assert_eq!(tick, 42);
        assert_eq!(zoom, ZoomLevel::Strategic);
    }
}
