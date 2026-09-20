//! Tests for FR-CIV-VEHICLE-002
//!
//!
//! This test file verifies FR FR-CIV-VEHICLE-002.
//! Removing a required material makes new builds fail; existing instances persist.

#[cfg(test)]
mod fr_fr_civ_vehicle_002 {
    use civ_engine::vehicle_types::*;
    use std::collections::BTreeSet;

    /// FR-CIV-VEHICLE-002 -- Removing material fails new builds.
    #[test]
    fn verify_fr_civ_vehicle_002_basic() {
        let catalog = default_archetype_catalog();
        let wagon = find_archetype(&catalog, VehicleKind::Wagon).unwrap();
        let all_traits: BTreeSet<String> = wagon.requires_traits.clone();
        let mut materials: BTreeSet<String> = wagon.requires_materials.clone();

        // With all materials: pass
        assert_eq!(
            check_build_capability(wagon, &all_traits, &materials, true),
            CapabilityGate::Pass
        );

        // Remove "livestock": fail
        materials.remove("livestock");
        let result = check_build_capability(wagon, &all_traits, &materials, true);
        match result {
            CapabilityGate::Fail { missing } => {
                assert!(missing.iter().any(|m| m.contains("livestock")));
            }
            _ => panic!("Should fail when livestock removed"),
        }
    }
}
