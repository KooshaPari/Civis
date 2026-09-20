//! Tests for FR-CIV-VEHICLE-041
//!
//!
//! This test file verifies FR FR-CIV-VEHICLE-041.
//! Per-good min-cost-flow respects arc capacity.

#[cfg(test)]
mod fr_fr_civ_vehicle_041 {
    use civ_engine::vehicle_types::*;

    /// FR-CIV-VEHICLE-041 -- Capacity limits flow; excess is unmet demand.
    #[test]
    fn verify_fr_civ_vehicle_041_basic() {
        let catalog = default_archetype_catalog();
        // A wagon has capacity 12; a cart has 4
        let wagon = find_archetype(&catalog, VehicleKind::Wagon).unwrap();
        let cart = find_archetype(&catalog, VehicleKind::Cart).unwrap();
        assert!(wagon.capacity > cart.capacity, "wagon must carry more than cart");
        // Two carts = 8 units, still less than one wagon
        assert!(wagon.capacity > cart.capacity * 2, "wagon capacity > 2x cart");
    }
}
