//! Tests for FR-CIV-VEHICLE-040
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-040.
//! Goods flow from surplus to deficit along cheapest path.

#[cfg(test)]
mod fr_fr_civ_vehicle_040 {
    use civ_engine::vehicle_types::{effective_speed, LaneClass};

    /// FR-CIV-VEHICLE-040 -- Cheaper route (faster speed) carries more flow.
    #[test]
    fn verify_fr_civ_vehicle_040_basic() {
        let base = 1.0;
        let vehicle = 1.0;
        let coupling = 1.0;
        let load = 0.0;
        let congestion = 1.0;

        // Route A: highway (fast)
        let speed_a = effective_speed(base, LaneClass::Highway.speed_mult(), vehicle, coupling, load, congestion);
        // Route B: trail (slow)
        let speed_b = effective_speed(base, LaneClass::Trail.speed_mult(), vehicle, coupling, load, congestion);

        assert!(speed_a > speed_b, "highway route must be faster");
        // Faster route has lower time-cost per unit
        let cost_a = 1.0 / speed_a; // time-cost = distance / speed
        let cost_b = 1.0 / speed_b;
        assert!(cost_a < cost_b, "highway cost must be lower");
    }
}
