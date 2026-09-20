//! Tests for FR-CIV-VEHICLE-001
//!
//!
//! This test file verifies FR FR-CIV-VEHICLE-001.
//! Vehicle archetype is buildable iff all three gates pass.

#[cfg(test)]
mod fr_fr_civ_vehicle_001 {
    use civ_engine::vehicle_types::*;
    use std::collections::BTreeSet;

    /// FR-CIV-VEHICLE-001 -- Capability gate: all three conditions must pass.
    #[test]
    fn verify_fr_civ_vehicle_001_basic() {
        let catalog = default_archetype_catalog();
        let cart = find_archetype(&catalog, VehicleKind::Cart).unwrap();
        let traits: BTreeSet<String> = BTreeSet::from(["wheel".into()]);
        let materials: BTreeSet<String> = BTreeSet::from(["wood".into()]);

        // All three gates pass
        assert_eq!(
            check_build_capability(cart, &traits, &materials, true),
            CapabilityGate::Pass
        );

        // Missing trait
        let no_traits = BTreeSet::new();
        let result = check_build_capability(cart, &no_traits, &materials, true);
        assert!(matches!(result, CapabilityGate::Fail { .. }));

        // Missing material
        let no_mats = BTreeSet::new();
        let result = check_build_capability(cart, &traits, &no_mats, true);
        assert!(matches!(result, CapabilityGate::Fail { .. }));

        // Medium not available
        let result = check_build_capability(cart, &traits, &materials, false);
        assert!(matches!(result, CapabilityGate::Fail { .. }));
    }
}
