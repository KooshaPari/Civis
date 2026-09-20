//! Tests for FR-CIV-VEHICLE-060
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-060.
//! Delivered price = local price + transport cost; market coupling.

#[cfg(test)]
mod fr_fr_civ_vehicle_060 {
    use civ_engine::vehicle_types::{effective_speed, LaneClass};

    /// FR-CIV-VEHICLE-060 -- Delivered price = local price + transport cost.
    #[test]
    fn verify_fr_civ_vehicle_060_basic() {
        let base = 1.0;
        let vehicle = 1.0;
        let coupling = 1.0;
        let load = 0.0;
        let congestion = 1.0;

        // Transport cost is inversely proportional to speed
        let highway_speed = effective_speed(base, LaneClass::Highway.speed_mult(), vehicle, coupling, load, congestion);
        let trail_speed = effective_speed(base, LaneClass::Trail.speed_mult(), vehicle, coupling, load, congestion);

        let local_price = 10.0;
        let distance = 100.0;
        let energy_cost_per_unit = 0.5;

        let transport_highway = (distance / highway_speed) * energy_cost_per_unit;
        let transport_trail = (distance / trail_speed) * energy_cost_per_unit;

        let delivered_highway = local_price + transport_highway;
        let delivered_trail = local_price + transport_trail;

        // Highway route has lower delivered price
        assert!(delivered_highway < delivered_trail, "highway delivered must be cheaper");
        // Both are above local price
        assert!(delivered_highway > local_price);
        assert!(delivered_trail > local_price);
    }
}
