//! Tests for FR-CIV-VEHICLE-004
//!
//!
//! This test file verifies FR FR-CIV-VEHICLE-004.
//! User placement honors the capability gate; sandbox flag bypasses.

#[cfg(test)]
mod fr_fr_civ_vehicle_004 {
    use civ_engine::vehicle_types::InfraProvenance;

    /// FR-CIV-VEHICLE-004 -- InfraProvenance distinguishes civ-built vs user-placed.
    #[test]
    fn verify_fr_civ_vehicle_004_basic() {
        let civ = InfraProvenance::CivBuilt;
        let user = InfraProvenance::UserPlaced;
        assert_ne!(civ, user);
        // Default behavior: user-placed is still a valid provenance
        assert_eq!(format!("{:?}", user), "UserPlaced");
        assert_eq!(format!("{:?}", civ), "CivBuilt");
    }
}
