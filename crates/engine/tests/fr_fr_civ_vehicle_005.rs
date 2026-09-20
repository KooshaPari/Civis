//! Tests for FR-CIV-VEHICLE-005
//!
//!
//! This test file verifies FR FR-CIV-VEHICLE-005.
//! era_hint is never read by routing/build logic.

#[cfg(test)]
mod fr_fr_civ_vehicle_005 {
    use civ_engine::vehicle_types::*;
    use std::collections::BTreeSet;

    /// FR-CIV-VEHICLE-005 -- era_hint does not affect capability gate.
    #[test]
    fn verify_fr_civ_vehicle_005_basic() {
        let catalog = default_archetype_catalog();
        let cart = find_archetype(&catalog, VehicleKind::Cart).unwrap();
        let traits: BTreeSet<String> = cart.requires_traits.clone();
        let materials: BTreeSet<String> = cart.requires_materials.clone();

        // Save original era_hint and change it
        let original_hint = cart.era_hint;
        let mut modified = cart.clone();
        modified.era_hint = 999; // absurd era hint

        // Capability gate should be identical regardless of era_hint
        let result_orig = check_build_capability(cart, &traits, &materials, true);
        let result_mod = check_build_capability(&modified, &traits, &materials, true);
        assert_eq!(result_orig, result_mod);
    }
}
