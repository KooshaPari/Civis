//! External integration tests for crates/civ-traffic coverage gaps.
//!
//! Covers: transmit (outage propagation), outage_count, place_vehicle (era gate).
use civ_traffic::{InfraProvenance, ServiceGrid, ServiceKind, TrafficGraph, VehicleKind};
use civ_voxel::WorldCoord;

fn wc(x: i64, z: i64) -> WorldCoord {
    WorldCoord { x, y: 0, z }
}

#[test]
fn transmit_propagates_outage_and_outage_count_agrees() {
    let mut g = ServiceGrid::new();
    let a = wc(0, 0);
    let b = wc(1, 0);
    let c = wc(2, 0);
    g.place_source(a, ServiceKind::Power).unwrap();
    g.place_source(b, ServiceKind::Power).unwrap();
    g.place_source(c, ServiceKind::Power).unwrap();
    g.connect_bidirectional(a, b).connect_bidirectional(b, c);

    assert_eq!(g.outage_count(), 0, "no outage before transmit");
    let flipped = g.transmit(a);
    assert_eq!(flipped, 3, "all 3 cells should flip to Outage");
    assert_eq!(g.outage_count(), 3);
}

#[test]
fn transmit_stops_at_disconnected_component() {
    let mut g = ServiceGrid::new();
    let a = wc(0, 0);
    let b = wc(1, 0);
    let isolated = wc(9, 9);
    g.place_source(a, ServiceKind::Water).unwrap();
    g.place_source(b, ServiceKind::Water).unwrap();
    g.place_source(isolated, ServiceKind::Water).unwrap();
    g.connect_bidirectional(a, b);

    g.transmit(a);
    assert_eq!(g.outage_count(), 2, "only a+b should be in outage; isolated survives");
}

#[test]
fn place_vehicle_cart_unlocks_at_era_1() {
    let mut tg = TrafficGraph::new();
    let at = wc(0, 0);
    // Cart.unlock_era() == 1; era 0 < 1 => rejected
    let rejected = tg.place_vehicle(VehicleKind::Cart, at, 0, InfraProvenance::Emergent);
    assert!(!rejected, "Cart should not be placeable at era 0");
    // era 1 >= 1 => accepted
    let added = tg.place_vehicle(VehicleKind::Cart, at, 1, InfraProvenance::Emergent);
    assert!(added, "Cart should be placeable at era 1");
}

#[test]
fn place_vehicle_wagon_locked_until_era_2() {
    let mut tg = TrafficGraph::new();
    let at = wc(5, 5);
    let rejected = tg.place_vehicle(VehicleKind::Wagon, at, 1, InfraProvenance::Emergent);
    assert!(!rejected, "Wagon should not be placeable at era 1");
    let added = tg.place_vehicle(VehicleKind::Wagon, at, 2, InfraProvenance::Emergent);
    assert!(added, "Wagon should be placeable at era 2");
}
