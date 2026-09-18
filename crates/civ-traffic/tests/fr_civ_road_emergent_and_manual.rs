//! FR-CIV-ROAD-902 / 910 / 920 — shared infrastructure graph, emergent and
//! player-authored roads, and vehicles routing on them.
//!
//! Requirement text (docs/specs/requirements/FR-CIV-ROAD.md):
//!
//! - FR-CIV-ROAD-902: "All structures + roads SHALL carry shared data tags
//!   regardless of author (procedural vs player), via the building graph."
//!   Acceptance: "building graph tags provenance but exposes uniform query API."
//! - FR-CIV-ROAD-910: "Vehicles/transport agents SHALL emerge on the road
//!   network to move goods/people." Acceptance: "Vehicles route on emergent
//!   roads; congestion measurable; no hardcoded routes."
//! - FR-CIV-ROAD-920: "Player SHALL have manual road tools (place/curve/snap/
//!   upgrade) that inject road edges the sim then treats identically to
//!   emergent ones." Acceptance: "Road tool with curves + snapping + tier
//!   upgrade; player roads share the same graph + tags."
//!
//! These replace placeholders (`crates/civ-traffic/tests/fr_fr_civ_road_901.rs`
//! and `_902.rs`) whose entire body was
//! `assert!(!SCHEMA_VERSION.is_empty()); assert_eq!(RoadKind::None.speed_multiplier(), 1.0);`
//! — identical in both files, and asserting nothing about road emergence,
//! provenance, or vehicle routing.

use civ_traffic::{
    lane::LaneGraph, InfraProvenance, PathCongestion, RoadKind, TrafficGraph, VehicleKind,
};
use civ_voxel::WorldCoord;

/// World coord on the ground plane.
fn wc(x: i64, z: i64) -> WorldCoord {
    WorldCoord { x, y: 0, z }
}

/// An emergent road between `a` and `b`, driven purely by accumulated traffic.
///
/// Uses only `record_traffic`, i.e. the same path the life-sim takes when
/// agents walk an edge — there is no placement call, so the road is genuinely
/// self-organised rather than authored.
fn emergent_road(a: WorldCoord, b: WorldCoord, kind: RoadKind) -> TrafficGraph {
    let mut g = TrafficGraph::new();
    // Default thresholds: trail 8, road 32, highway 128.
    let weight = match kind {
        RoadKind::Trail => 10.0,
        RoadKind::Road => 40.0,
        RoadKind::Highway => 140.0,
        RoadKind::None => 0.0,
        RoadKind::Bridge => return g, // bridges never emerge
    };
    g.record_traffic(a, b, weight);
    g
}

// ---------------------------------------------------------------------------
// FR-CIV-ROAD-902 — uniform query API regardless of author
// ---------------------------------------------------------------------------

/// Covers FR-CIV-ROAD-902.
///
/// A road's *author* must not change how the graph answers queries: an emergent
/// road and a player-placed road at the same rung must report the same
/// `kind_between` and the same `speed_multiplier_at`. Provenance is carried, but
/// it is metadata, not a second query path.
#[test]
fn fr_civ_road_902_query_is_author_agnostic() {
    let (a, b) = (wc(0, 0), wc(1, 0));

    let emergent = emergent_road(a, b, RoadKind::Road);
    let mut placed = TrafficGraph::new();
    placed.place_segment(a, b, RoadKind::Road);

    assert_eq!(
        emergent.kind_between(a, b),
        RoadKind::Road,
        "emergent road should have been promoted to Road"
    );
    assert_eq!(
        placed.kind_between(a, b),
        RoadKind::Road,
        "player-placed road should read back as Road"
    );

    // Uniform query surface: identical answers.
    assert_eq!(
        emergent.kind_between(a, b),
        placed.kind_between(a, b),
        "kind_between must not depend on provenance"
    );
    assert_eq!(
        emergent.speed_multiplier_at(a, b),
        placed.speed_multiplier_at(a, b),
        "speed_multiplier_at must not depend on provenance"
    );

    // The provenance tag is still carried and still distinguishes them.
    let e_prov = emergent
        .iter_segments()
        .map(|(_, s)| s.provenance)
        .next()
        .expect("emergent segment exists");
    let p_prov = placed
        .iter_segments()
        .map(|(_, s)| s.provenance)
        .next()
        .expect("placed segment exists");
    assert_eq!(e_prov, InfraProvenance::Emergent);
    assert_eq!(p_prov, InfraProvenance::UserPlaced);
    assert_ne!(
        e_prov, p_prov,
        "provenance must distinguish the authoring channel"
    );
}

/// Covers FR-CIV-ROAD-902.
///
/// Both authoring channels must land in ONE shared graph, keyed by the same
/// canonical undirected edge, so downstream consumers read a single network.
#[test]
fn fr_civ_road_902_both_authors_share_one_canonical_graph() {
    let (a, b) = (wc(0, 0), wc(3, 0));

    // Emergent edge, then a player action on the same edge.
    let mut g = TrafficGraph::new();
    g.record_traffic(a, b, 40.0);
    assert_eq!(g.iter_segments().count(), 1);

    g.place_segment(b, a, RoadKind::Highway); // reversed endpoints on purpose
    assert_eq!(
        g.iter_segments().count(),
        1,
        "(a,b) and (b,a) must map to the same canonical edge, not two segments"
    );
    assert_eq!(
        g.kind_between(a, b),
        RoadKind::Highway,
        "the stronger placement must be visible from either endpoint order"
    );
    assert_eq!(g.kind_between(b, a), RoadKind::Highway);
}

// ---------------------------------------------------------------------------
// FR-CIV-ROAD-920 — manual road tools inject edges the sim treats identically
// ---------------------------------------------------------------------------

/// Covers FR-CIV-ROAD-920.
///
/// "Road tool with curves": a drag-to-draw polyline must create one segment per
/// consecutive pair, all queryable through the same API, and all tagged as
/// player-placed.
#[test]
fn fr_civ_road_920_place_path_creates_a_connected_polyline() {
    let mut g = TrafficGraph::new();
    let points = [wc(0, 0), wc(1, 0), wc(2, 0), wc(2, 1)];

    g.place_path(&points, RoadKind::Road);

    assert_eq!(
        g.iter_segments().count(),
        points.len() - 1,
        "a 4-point path must create 3 segments"
    );
    for pair in points.windows(2) {
        assert_eq!(
            g.kind_between(pair[0], pair[1]),
            RoadKind::Road,
            "every consecutive pair in the drawn path must be a Road segment"
        );
    }
    for (_, seg) in g.iter_segments() {
        assert_eq!(
            seg.provenance,
            InfraProvenance::UserPlaced,
            "drawn roads are player-authored"
        );
    }

    // Fewer than two points is a documented no-op.
    let before = g.iter_segments().count();
    g.place_path(&[wc(9, 9)], RoadKind::Highway);
    assert_eq!(
        g.iter_segments().count(),
        before,
        "a single-point path must not create a segment"
    );
}

/// Covers FR-CIV-ROAD-920.
///
/// "Tier upgrade": a later, stronger placement raises the rung; a weaker one
/// must never downgrade a road the player already drew.
#[test]
fn fr_civ_road_920_placement_upgrades_but_never_downgrades() {
    let (a, b) = (wc(0, 0), wc(1, 0));
    let mut g = TrafficGraph::new();

    g.place_segment(a, b, RoadKind::Trail);
    assert_eq!(g.kind_between(a, b), RoadKind::Trail);

    g.place_segment(a, b, RoadKind::Highway);
    assert_eq!(
        g.kind_between(a, b),
        RoadKind::Highway,
        "a stronger placement must upgrade the rung"
    );

    g.place_segment(a, b, RoadKind::Trail);
    assert_eq!(
        g.kind_between(a, b),
        RoadKind::Highway,
        "a weaker placement must not downgrade an existing road"
    );

    // Cross-check the ordering the upgrade rule relies on.
    assert!(RoadKind::Highway > RoadKind::Road);
    assert!(RoadKind::Road > RoadKind::Trail);
    assert!(RoadKind::Trail > RoadKind::None);
}

/// Covers FR-CIV-ROAD-920.
///
/// "the sim then treats identically": accumulating agent traffic on a
/// player-drawn road must go through the same promotion logic as an emergent
/// one, so heavy use can grow a player road to a higher rung.
#[test]
fn fr_civ_road_920_player_road_still_grows_under_traffic() {
    let (a, b) = (wc(0, 0), wc(1, 0));
    let mut g = TrafficGraph::new();

    g.place_segment(a, b, RoadKind::Trail);
    assert_eq!(g.kind_between(a, b), RoadKind::Trail);

    // Heavy use: enough to cross the highway threshold (128).
    let kind = g.record_traffic(a, b, 140.0);

    assert_eq!(
        kind,
        RoadKind::Highway,
        "a player-drawn trail under heavy traffic must grow to Highway"
    );
    assert_eq!(g.kind_between(a, b), RoadKind::Highway);

    // And it must still be recorded as player-authored.
    let prov = g.iter_segments().next().expect("segment").1.provenance;
    assert_eq!(
        prov,
        InfraProvenance::UserPlaced,
        "growth must not rewrite the authoring channel"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-ROAD-910 — vehicles on the road network
// ---------------------------------------------------------------------------

/// Covers FR-CIV-ROAD-910.
///
/// "Vehicles route on emergent roads": a lane graph derived from purely
/// emergent segments must produce a real multi-hop route. There are no
/// hardcoded routes — the lane net is generated from the grown roads, and the
/// route is expressed as lane-to-lane connections through the middle node.
#[test]
fn fr_civ_road_910_vehicles_route_on_emergent_roads() {
    let (a, b, c) = (wc(0, 0), wc(1, 0), wc(2, 0));
    let mut g = TrafficGraph::new();
    // Repeated agent traversals promote both edges, with no placement calls.
    g.record_traffic(a, b, 40.0);
    g.record_traffic(b, c, 40.0);
    assert_eq!(g.kind_between(a, b), RoadKind::Road);
    assert_eq!(g.kind_between(b, c), RoadKind::Road);

    let lanes = LaneGraph::from_traffic(&g);
    let route = lanes.route_lanes(a.into(), c.into());

    assert!(
        !route.is_empty(),
        "routing across two promoted emergent roads must yield at least one \
         lane connection at the middle node"
    );
    // Every hop in the route must stay anchored on a node of the grown net.
    for conn in &route {
        assert!(
            lanes.nodes.contains_key(&conn.node),
            "route hop must be anchored on a real lane-net node"
        );
    }
}

/// Covers FR-CIV-ROAD-910.
///
/// An unpromoted edge has no lanes, so a chain below the trail threshold cannot
/// be routed: the lane net really is derived from road rung, not hardcoded.
#[test]
fn fr_civ_road_910_lane_net_tracks_promotion_not_placement() {
    let (a, b, c) = (wc(0, 0), wc(1, 0), wc(2, 0));

    let mut bare = TrafficGraph::new();
    bare.record_traffic(a, b, 1.0); // below trail threshold (8)
    bare.record_traffic(b, c, 1.0);
    assert_eq!(bare.kind_between(a, b), RoadKind::None);
    assert!(
        LaneGraph::from_traffic(&bare)
            .route_lanes(a.into(), c.into())
            .is_empty(),
        "edges below the trail threshold must not be routable"
    );

    // Same chain, now over the threshold.
    let mut grown = TrafficGraph::new();
    grown.record_traffic(a, b, 140.0);
    grown.record_traffic(b, c, 140.0);
    assert_eq!(grown.kind_between(a, b), RoadKind::Highway);
    assert!(
        !LaneGraph::from_traffic(&grown)
            .route_lanes(a.into(), c.into())
            .is_empty(),
        "once promoted to Highway the same chain must be routable"
    );
}

/// Covers FR-CIV-ROAD-910.
///
/// "congestion measurable": concurrent users on an edge raise its traversal
/// cost, and it relaxes again when they leave.
#[test]
fn fr_civ_road_910_congestion_is_measurable() {
    let mut c = PathCongestion::new(10.0, 2.5);
    let idle = c.cost();

    c.enter();
    let one = c.cost();
    c.enter();
    let two = c.cost();

    assert!(one > idle, "one user must cost more than idling");
    assert!(two > one, "two users must cost more than one");

    c.leave();
    assert!(
        c.cost() < two,
        "cost must relax when a user leaves the edge"
    );
}

/// Covers FR-CIV-ROAD-910.
///
/// Vehicles are era-gated: a vehicle archetype cannot be placed before its
/// unlock era, and is recorded once the era is met.
#[test]
fn fr_civ_road_910_vehicle_placement_respects_unlock_era() {
    let mut g = TrafficGraph::new();
    let at = wc(0, 0);

    // Cart unlocks at era 1, Wagon at era 2.
    assert!(
        !g.place_vehicle(VehicleKind::Cart, at, 0, InfraProvenance::Emergent),
        "Cart must not be placeable before its unlock era"
    );
    assert!(
        g.place_vehicle(VehicleKind::Cart, at, 1, InfraProvenance::Emergent),
        "Cart must be placeable at its unlock era"
    );
    assert!(
        !g.place_vehicle(VehicleKind::Wagon, at, 1, InfraProvenance::Emergent),
        "Wagon must still be locked at era 1"
    );
    assert!(
        g.place_vehicle(VehicleKind::Wagon, at, 2, InfraProvenance::UserPlaced),
        "Wagon must be placeable at its unlock era"
    );

    assert_eq!(g.vehicles.len(), 2, "two vehicles should have been recorded");
    assert_eq!(g.vehicles[1].kind, VehicleKind::Wagon);
    assert_eq!(g.vehicles[1].provenance, InfraProvenance::UserPlaced);
}
