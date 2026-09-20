//! Tests for FR-CIV-VEHICLE-030
//!
//!
//! This test file verifies FR FR-CIV-VEHICLE-030.
//! Build charges energy+materials.

#[cfg(test)]
mod fr_fr_civ_vehicle_030 {
    use civ_engine::vehicle_types::*;

    /// FR-CIV-VEHICLE-030 -- Each archetype has a non-zero build cost.
    #[test]
    fn verify_fr_civ_vehicle_030_basic() {
        let catalog = default_archetype_catalog();
        for arch in &catalog {
            assert!(
                arch.build_cost > 0,
                "Archetype {:?} must have positive build_cost",
                arch.kind
            );
            assert!(
                arch.capacity > 0,
                "Archetype {:?} must have positive capacity",
                arch.kind
            );
        }
    }
}
