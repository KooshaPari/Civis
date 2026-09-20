//! Tests for FR-CIV-VEHICLE-003
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-003.
//! Catalog is additive/forward-only: new archetypes slot in by adding rows.

#[cfg(test)]
mod fr_fr_civ_vehicle_003 {
    use civ_engine::vehicle_types::*;

    /// FR-CIV-VEHICLE-003 -- Catalog is additive: all existing kinds present.
    #[test]
    fn verify_fr_civ_vehicle_003_basic() {
        let catalog = default_archetype_catalog();
        // All existing kinds are in the catalog
        let kinds: Vec<VehicleKind> = catalog.iter().map(|a| a.kind).collect();
        assert!(kinds.contains(&VehicleKind::Cart));
        assert!(kinds.contains(&VehicleKind::Wagon));
        assert!(kinds.contains(&VehicleKind::SailingShip));
        assert!(kinds.contains(&VehicleKind::Truck));
        assert!(kinds.contains(&VehicleKind::Aircraft));
        assert!(kinds.contains(&VehicleKind::NearFuture));
        // Count is >= 13 (the baseline catalog)
        assert!(catalog.len() >= 13);
    }
}
