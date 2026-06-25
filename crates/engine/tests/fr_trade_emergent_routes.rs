//! FR-TRADE — emergent settlement trade routes from price gaps + connectivity.

use civ_engine::Simulation;
use glam::IVec3;

/// FR-TRADE — simulation tick discovers a route between surplus and deficit
/// settlements when a profitable price gap and viable path exist; snapshot
/// surfaces the active route and food flows along it.
#[test]
fn fr_trade_route_forms_between_surplus_and_deficit_settlement() {
    let mut sim = Simulation::with_seed(42);
    sim.set_settlement_population(1, 100);
    sim.set_settlement_population(2, 100);
    sim.set_settlement_food_stocked(1, 5_000);
    sim.set_settlement_food_stocked(2, 5);
    sim.set_settlement_position(1, IVec3::ZERO);
    sim.set_settlement_position(2, IVec3::new(8, 0, 0));

    let importer_before = sim
        .snapshot()
        .settlement_trade_routes
        .len();
    assert_eq!(importer_before, 0);

    sim.tick();

    let snapshot = sim.snapshot();
    let routes = &snapshot.settlement_trade_routes;
    assert!(
        !routes.is_empty(),
        "expected emergent settlement trade route on snapshot"
    );
    let route = routes
        .iter()
        .find(|r| r.origin == 1 && r.destination == 2)
        .expect("route 1→2 must emerge from surplus/deficit price gap");
    assert_eq!(route.good, "food");
    assert!(route.flow > 0);
    assert!(route.margin_cents > 0);
}
