//! FR-CIV-ROAD cluster — real behavioural oracles for FR-CIV-ROAD-901 / 902 /
//! 910 / 920 / 921.
//!
//! Requirement text: `docs/specs/requirements/FR-CIV-ROAD.md` (lines 12-16).
//!
//! These replace the auto-generated placeholders
//! (`tests/fr_fr_civ_road_901.rs`, `_902.rs`, `_910.rs`, `_920.rs`, `_921.rs`)
//! whose whole body was
//! `assert!(!SCHEMA_VERSION.is_empty()); assert_eq!(RoadKind::None.speed_multiplier(), 1.0);`
//! — byte-identical in every one of those files, and asserting nothing about
//! emergent growth, provenance, routing, road tools or land-use lenses.
//!
//! Scope note (honest coverage boundary):
//!
//! * FR-CIV-ROAD-902 / 910 / 920 are implemented in `civ-traffic` itself, so
//!   they are tested here against the real public API.
//! * FR-CIV-ROAD-901's *structure* half (settlement build decisions reading
//!   need + local resource + labor) lives in `crates/engine` / `crates/build`
//!   (`era_gated_demand_signals`, `building_affordable_parcel_count`,
//!   `building_material_headroom_permille`), which this crate does not depend
//!   on. What *is* implemented here and is testable here is the acceptance
//!   criterion's other half: self-organisation with **no scripted build
//!   orders**, growth gated by a configurable local budget, monotonically.
//! * FR-CIV-ROAD-921's district overlay has no implementation in this
//!   repository (see the test's doc comment). What is testable here is the
//!   property the requirement turns on: a player designation biases the
//!   emergent system instead of being a hard enum that forces an outcome.

use std::collections::BTreeMap;

use civ_traffic::{
    lanes_for, speed_for_lane, EdgeKey, FlowPriorityPolicy, InfraProvenance, LaneClass,
    LaneDirection, LaneGraph, LaneVolume, NodeKey, PathCongestion, PromotionThresholds, RoadKind,
    RoadSegment, TrafficGraph, VehicleKind,
};
use civ_voxel::WorldCoord;

/// World coord on the ground plane.
fn wc(x: i64, z: i64) -> WorldCoord {
    WorldCoord { x, y: 0, z }
}

/// Default promotion budget: trail 8, road 32, highway 128.
const TRAIL: f32 = 8.0;
const ROAD: f32 = 32.0;
const HIGHWAY: f32 = 128.0;

// ---------------------------------------------------------------------------
// FR-CIV-ROAD-901 — self-organising construction, no scripted build orders
// ---------------------------------------------------------------------------

/// Covers FR-CIV-ROAD-901.
///
/// "no scripted build orders": infrastructure must appear from accumulated
/// agent use alone. Nothing is placed, no order is issued — a single traversal
/// creates the segment, a second one promotes it, and the same amount of use
/// yields the same rung anywhere on the map (decentralised, no per-site script).
#[test]
fn fr_civ_road_901_emergence_requires_no_scripted_order() {
    let (a, b) = (wc(0, 0), wc(1, 0));
    let mut g = TrafficGraph::new();
    assert_eq!(
        g.iter_segments().count(),
        0,
        "a fresh settlement starts with no infrastructure at all"
    );

    // Agents walk; no `place_segment` call, no build order.
    let first = g.record_traffic(a, b, 4.0);
    assert_eq!(first, RoadKind::None, "4.0 < trail budget {TRAIL}");
    let (_, seg) = g
        .iter_segments()
        .next()
        .expect("agent use alone must create the segment");
    assert_eq!(
        seg.provenance,
        InfraProvenance::Emergent,
        "growth is self-organised, not scripted/placed"
    );
    assert_eq!(seg.traffic, 4.0);

    // The rung is decided by *accumulated* use, not by the last step.
    assert_eq!(
        g.record_traffic(a, b, 4.0),
        RoadKind::Trail,
        "4.0 + 4.0 >= {TRAIL} promotes"
    );

    // Decentralised: identical accumulated use gives an identical rung far away.
    let far = (wc(90, -40), wc(90, -39));
    let mut elsewhere = TrafficGraph::new();
    elsewhere.record_traffic(far.0, far.1, 8.0);
    assert_eq!(
        elsewhere.kind_between(far.0, far.1),
        g.kind_between(a, b),
        "no site is special-cased: equal use, equal rung"
    );
}

/// Covers FR-CIV-ROAD-901.
///
/// The promotion tier must be a pure function of accumulated traffic: delivering
/// the same total weight in one lump or in many small trips must produce the
/// *identical* graph. A scripted/heuristic build order would break this.
#[test]
fn fr_civ_road_901_promotion_is_pure_in_accumulated_traffic() {
    let (a, b) = (wc(0, 0), wc(1, 0));

    let mut lump = TrafficGraph::new();
    lump.record_traffic(a, b, 40.0);

    let mut chunked = TrafficGraph::new();
    for step in [7.5_f32, 7.5, 25.0] {
        chunked.record_traffic(a, b, step);
    }

    assert_eq!(
        chunked.kind_between(a, b),
        lump.kind_between(a, b),
        "chunked and lump-sum use of the same total must reach the same rung"
    );
    assert_eq!(
        chunked, lump,
        "the whole graph (kind + accumulated traffic + provenance) is a pure \
         function of total use, so the same use is replay-identical"
    );

    // Climbing is monotone: use never demotes an emergent edge.
    let mut g = TrafficGraph::new();
    let mut seen = vec![RoadKind::None];
    for step in [4.0_f32, 4.0, 24.0, 100.0] {
        seen.push(g.record_traffic(a, b, step));
    }
    assert!(
        seen.windows(2).all(|w| w[1] >= w[0]),
        "the ladder must never go backwards under accumulating use: {seen:?}"
    );
    assert_eq!(*seen.last().expect("steps"), RoadKind::Highway);
}

/// Covers FR-CIV-ROAD-901.
///
/// "Build decisions read need+resource+labor": growth is gated by a configurable
/// local budget (the resource/labor analogue here), not by a fixed script. The
/// same use that grows a Road in a rich region only raises a Trail in a region
/// whose budget is scarcer — and the gate is read from thresholds, never
/// hardcoded per site.
#[test]
fn fr_civ_road_901_growth_is_budget_gated_not_scripted() {
    let (a, b) = (wc(0, 0), wc(1, 0));

    let mut rich = TrafficGraph::new();
    rich.record_traffic(a, b, 40.0);
    assert_eq!(rich.kind_between(a, b), RoadKind::Road);

    let mut scarce = TrafficGraph::new();
    scarce.thresholds = PromotionThresholds {
        trail: 20.0,
        road: 60.0,
        highway: 200.0,
    };
    scarce.record_traffic(a, b, 40.0);
    assert_eq!(
        scarce.kind_between(a, b),
        RoadKind::Trail,
        "the same use under a scarcer budget must yield a weaker rung"
    );

    // The gate is also symmetric: richer budgets promote no later than poorer
    // ones for the same use (monotone in budget).
    let kind_under = |road: f32| {
        let mut g = TrafficGraph::new();
        g.thresholds = PromotionThresholds {
            trail: TRAIL,
            road,
            highway: HIGHWAY,
        };
        g.record_traffic(a, b, 40.0);
        g.kind_between(a, b)
    };
    assert_eq!(kind_under(ROAD), RoadKind::Road);
    assert_eq!(kind_under(64.0), RoadKind::Trail);
}

/// Covers FR-CIV-ROAD-901.
///
/// The emergent ladder is bounded by what agents can build up: it tops out at
/// `Highway`. A `Bridge` is a placed structure only — no amount of traffic may
/// fabricate one, and the top rung is terminal rather than wrapping around.
#[test]
fn fr_civ_road_901_emergent_ladder_caps_at_highway() {
    let thresholds = PromotionThresholds::default();
    for traffic in [
        0.0_f32,
        7.9,
        8.0,
        31.9,
        32.0,
        127.9,
        128.0,
        10_000.0,
        f32::MAX,
    ] {
        assert_ne!(
            thresholds.kind_for(traffic),
            RoadKind::Bridge,
            "traffic {traffic} must never fabricate a bridge"
        );
    }
    assert_eq!(thresholds.kind_for(f32::MAX), RoadKind::Highway);

    // Rung boundaries are exact and inclusive at the lower edge.
    assert_eq!(thresholds.kind_for(TRAIL - 0.1), RoadKind::None);
    assert_eq!(thresholds.kind_for(TRAIL), RoadKind::Trail);
    assert_eq!(thresholds.kind_for(ROAD), RoadKind::Road);
    assert_eq!(thresholds.kind_for(HIGHWAY), RoadKind::Highway);

    // Terminal rungs: promotion cannot leave Highway/Bridge.
    assert_eq!(RoadKind::Highway.promoted(), RoadKind::Highway);
    assert_eq!(RoadKind::Bridge.promoted(), RoadKind::Bridge);
    assert_eq!(RoadKind::None.promoted(), RoadKind::Trail);
    assert_eq!(RoadKind::Trail.promoted(), RoadKind::Road);
    assert_eq!(RoadKind::Road.promoted(), RoadKind::Highway);

    // ...and a real graph confirms it end to end.
    let mut g = TrafficGraph::new();
    let kind = g.record_traffic(wc(0, 0), wc(1, 0), 1_000_000.0);
    assert_eq!(kind, RoadKind::Highway);
}

// ---------------------------------------------------------------------------
// FR-CIV-ROAD-902 — shared data tags, author-agnostic query API
// ---------------------------------------------------------------------------

/// Covers FR-CIV-ROAD-902.
///
/// "tags provenance but exposes uniform query API": provenance is metadata, not
/// a second graph. An emergent road and a player-placed road at the same rung
/// must answer every query identically and live under the same canonical
/// undirected edge key, while still carrying the tag that distinguishes them.
#[test]
fn fr_civ_road_902_shared_tags_expose_one_query_api() {
    let (a, b) = (wc(0, 0), wc(1, 0));

    let mut emergent = TrafficGraph::new();
    emergent.record_traffic(a, b, 40.0);

    let mut placed = TrafficGraph::new();
    placed.place_segment(a, b, RoadKind::Road);

    // Uniform reads.
    assert_eq!(
        emergent.kind_between(a, b),
        placed.kind_between(a, b),
        "kind_between must not depend on the author"
    );
    assert_eq!(
        emergent.speed_multiplier_at(a, b),
        placed.speed_multiplier_at(a, b),
        "the life-sim cost model must read both authors identically"
    );
    assert_eq!(emergent.count_at_least(RoadKind::Road), 1);
    assert_eq!(placed.count_at_least(RoadKind::Road), 1);
    assert_eq!(emergent.count_at_least(RoadKind::Highway), 0);
    assert_eq!(placed.count_at_least(RoadKind::Highway), 0);

    // Same canonical key, in both endpoint orders: one shared graph.
    let e_keys: Vec<EdgeKey> = emergent.iter_segments().map(|(k, _)| k).collect();
    let p_keys: Vec<EdgeKey> = placed.iter_segments().map(|(k, _)| k).collect();
    assert_eq!(e_keys, vec![EdgeKey::new(a, b)]);
    assert_eq!(e_keys, p_keys, "both authors key into the same edge space");
    assert_eq!(EdgeKey::new(b, a), e_keys[0], "edges are undirected");

    // The tag itself is still carried, so the renderer can style them apart.
    let e_tag = emergent
        .iter_segments()
        .next()
        .expect("emergent")
        .1
        .provenance;
    let p_tag = placed.iter_segments().next().expect("placed").1.provenance;
    assert_eq!(e_tag, InfraProvenance::Emergent);
    assert_eq!(p_tag, InfraProvenance::UserPlaced);
    assert_ne!(e_tag, p_tag);
}

/// Covers FR-CIV-ROAD-902.
///
/// Shared tags are only "shared" if they survive persistence: a save must keep
/// both provenance tags and the uniform query answers, otherwise replays would
/// silently lose the authoring channel.
#[test]
fn fr_civ_road_902_tags_and_queries_survive_save_load() {
    let mut g = TrafficGraph::new();
    g.record_traffic(wc(0, 0), wc(1, 0), 140.0); // emergent highway
    g.place_segment(wc(1, 0), wc(2, 0), RoadKind::Road); // player road

    let encoded = ron::to_string(&g).expect("serialize shared graph");
    let back: TrafficGraph = ron::from_str(&encoded).expect("deserialize shared graph");

    assert_eq!(back, g, "round-trip must be lossless");
    assert_eq!(
        back.speed_multiplier_at(wc(0, 0), wc(1, 0)),
        RoadKind::Highway.speed_multiplier()
    );
    assert_eq!(back.kind_between(wc(1, 0), wc(2, 0)), RoadKind::Road);

    let tags: Vec<(RoadKind, InfraProvenance)> = back
        .iter_segments()
        .map(|(_, s)| (s.kind, s.provenance))
        .collect();
    assert!(
        tags.contains(&(RoadKind::Highway, InfraProvenance::Emergent)),
        "emergent tag must survive save/load: {tags:?}"
    );
    assert!(
        tags.contains(&(RoadKind::Road, InfraProvenance::UserPlaced)),
        "player tag must survive save/load: {tags:?}"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-ROAD-910 — vehicles on the road network, flow + congestion
// ---------------------------------------------------------------------------

/// Covers FR-CIV-ROAD-910.
///
/// "Vehicles route on emergent roads; no hardcoded routes": the lane net is a
/// pure function of the grown rung. Bare ground yields zero lanes, and each rung
/// yields its own deterministic lane count/class/directions — nothing is
/// pre-baked.
#[test]
fn fr_civ_road_910_lane_net_is_derived_from_the_grown_rung() {
    let (a, b) = (wc(0, 0), wc(1, 0));
    let edge = EdgeKey::new(a, b);

    // Below the trail budget there is no net at all.
    let mut bare = TrafficGraph::new();
    bare.record_traffic(a, b, TRAIL - 0.1);
    assert_eq!(bare.kind_between(a, b), RoadKind::None);
    assert_eq!(
        LaneGraph::from_traffic(&bare).lanes.len(),
        0,
        "unpromoted ground must not be routable"
    );

    // Each rung produces exactly its own lane shape.
    for (weight, class, count) in [
        (TRAIL, LaneClass::Trail, 1_usize),
        (ROAD, LaneClass::Road, 2),
        (HIGHWAY, LaneClass::Highway, 3),
    ] {
        let mut g = TrafficGraph::new();
        g.record_traffic(a, b, weight);
        let seg = g.iter_segments().next().expect("segment").1;
        assert_eq!(seg.kind.speed_multiplier(), speed_for_lane_class(class));

        let lanes = lanes_for(edge, seg);
        assert_eq!(lanes.len(), count, "lane count must follow the rung");
        assert!(lanes.iter().all(|l| l.class == class));
        assert!(lanes.iter().all(|l| l.segment == edge));
        assert!(lanes
            .iter()
            .all(|l| speed_for_lane(l) == seg.kind.speed_multiplier()));
        assert_eq!(
            lanes.iter().map(|l| l.index).collect::<Vec<_>>(),
            (0..count).collect::<Vec<_>>(),
            "lane indices must be dense and ordered"
        );

        let lane_graph = LaneGraph::from_traffic(&g);
        assert_eq!(lane_graph.lanes.len(), count);
        assert_eq!(lane_graph.nodes.len(), 2, "both endpoints are lane nodes");
        assert!(lane_graph.nodes.contains_key(&NodeKey::from(a)));
        assert!(lane_graph.nodes.contains_key(&NodeKey::from(b)));
    }

    // Directions are generated, not scripted per site.
    let dirs = |kind: RoadKind| {
        let seg = RoadSegment {
            kind,
            traffic: 0.0,
            provenance: InfraProvenance::Emergent,
        };
        lanes_for(edge, &seg)
            .into_iter()
            .map(|l| l.direction)
            .collect::<Vec<_>>()
    };
    assert_eq!(dirs(RoadKind::None), Vec::new());
    assert_eq!(dirs(RoadKind::Trail), vec![LaneDirection::Both]);
    assert_eq!(
        dirs(RoadKind::Road),
        vec![LaneDirection::AB, LaneDirection::BA]
    );
    assert_eq!(
        dirs(RoadKind::Highway),
        vec![LaneDirection::AB, LaneDirection::BA, LaneDirection::AB]
    );
}

/// Lane class -> the scalar road speed it must mirror.
fn speed_for_lane_class(class: LaneClass) -> f32 {
    match class {
        LaneClass::Trail => RoadKind::Trail.speed_multiplier(),
        LaneClass::Road => RoadKind::Road.speed_multiplier(),
        LaneClass::Highway => RoadKind::Highway.speed_multiplier(),
    }
}

/// Covers FR-CIV-ROAD-910.
///
/// A vehicle route must be discovered across junctions of *emergent* roads, and
/// must be empty when the roads do not exist. The exact hop sequence is asserted
/// so a regression in junction connectivity cannot pass silently.
#[test]
fn fr_civ_road_910_route_crosses_an_emergent_junction() {
    // 4 cells, 3 edges, all grown purely by agent use (no placement calls).
    let pts = [wc(0, 0), wc(1, 0), wc(2, 0), wc(3, 0)];
    let mut g = TrafficGraph::new();
    for pair in pts.windows(2) {
        g.record_traffic(pair[0], pair[1], 140.0);
    }
    assert_eq!(g.count_at_least(RoadKind::Highway), 3);

    let lanes = LaneGraph::from_traffic(&g);
    assert_eq!(lanes.nodes.len(), 4);
    assert_eq!(lanes.lanes.len(), 9, "3 highway edges x 3 lanes");

    let route = lanes.route_lanes(NodeKey::from(pts[0]), NodeKey::from(pts[3]));
    assert_eq!(route.len(), 2, "the path must cross both junctions");
    assert_eq!(route[0].from.segment, EdgeKey::new(pts[0], pts[1]));
    assert_eq!(route[0].to.segment, EdgeKey::new(pts[1], pts[2]));
    assert_eq!(route[0].node, NodeKey::from(pts[1]));
    assert_eq!(route[1].from.segment, EdgeKey::new(pts[1], pts[2]));
    assert_eq!(route[1].to.segment, EdgeKey::new(pts[2], pts[3]));
    assert_eq!(route[1].node, NodeKey::from(pts[2]));
    assert!(
        route.iter().all(|c| lanes.nodes.contains_key(&c.node)),
        "every turn must be anchored on a real lane node"
    );

    // Reverse direction is routable too (roads are not one-way by fiat).
    let back = lanes.route_lanes(NodeKey::from(pts[3]), NodeKey::from(pts[0]));
    assert_eq!(back.len(), 2);
    assert_eq!(back[0].from.segment, EdgeKey::new(pts[2], pts[3]));
    assert_eq!(back[1].to.segment, EdgeKey::new(pts[0], pts[1]));

    // With no grown road between them there is nothing to route on.
    let disconnected = LaneGraph::from_traffic(&TrafficGraph::new());
    assert!(disconnected
        .route_lanes(NodeKey::from(pts[0]), NodeKey::from(pts[3]))
        .is_empty());
}

/// Covers FR-CIV-ROAD-910.
///
/// "congestion measurable": the edge cost is exactly `base + users * penalty`,
/// it rises per entrant, relaxes per leaver, and saturates at zero users instead
/// of wrapping.
#[test]
fn fr_civ_road_910_congestion_is_measurable_and_saturates() {
    let mut c = PathCongestion::new(10.0, 2.5);
    assert_eq!(c.cost(), 10.0, "idle cost is the baseline");
    assert_eq!(c.enter(), 1);
    assert_eq!(c.cost(), 12.5);
    assert_eq!(c.enter(), 2);
    assert_eq!(c.cost(), 15.0, "10.0 + 2 users x 2.5");
    assert_eq!(c.leave(), 1);
    assert_eq!(c.cost(), 12.5, "cost relaxes when a user leaves");
    assert_eq!(c.leave(), 0);
    assert_eq!(c.leave(), 0, "user count saturates at 0, never wraps");
    assert_eq!(c.cost(), 10.0, "fully drained edge returns to baseline");
}

/// Covers FR-CIV-ROAD-910.
///
/// Flow is scheduled from *measured* lane volume (no hardcoded lane assignment):
/// the busiest lane goes green, blocked lanes accrue wait, and the starvation
/// guard overrides volume once a lane has waited too long.
#[test]
fn fr_civ_road_910_flow_priority_schedules_by_measured_volume() {
    let policy = FlowPriorityPolicy::default();
    let lanes = [
        LaneVolume { lane: 4, volume: 1 },
        LaneVolume {
            lane: 9,
            volume: 12,
        },
    ];

    let first = policy.schedule(&lanes, &BTreeMap::new());
    assert_eq!(first.green, vec![9], "highest measured volume goes green");
    assert_eq!(first.phase_ticks, 1);
    assert_eq!(first.waits_after[&9], 0, "the green lane drains its queue");
    assert_eq!(first.waits_after[&4], 1, "blocked lane accrues wait");

    // Starvation guard: 64 ticks of waiting force-promotes the quiet lane.
    let mut starved = BTreeMap::new();
    starved.insert(4_u32, 64_u64);
    let second = policy.schedule(&lanes, &starved);
    assert_eq!(
        second.green,
        vec![4],
        "a starved lane outranks a busier one, or it would never move"
    );

    // Empty intersection is a no-op, not a panic.
    let empty = policy.schedule(&[], &BTreeMap::new());
    assert!(empty.green.is_empty());
    assert_eq!(empty.phase_ticks, 0);
}

/// Covers FR-CIV-ROAD-910.
///
/// Vehicles are era-gated and recorded deterministically; rejected placements
/// leave no trace. Vehicle multipliers stack on top of the road multiplier so a
/// wagon outruns a cart on the same road.
#[test]
fn fr_civ_road_910_vehicles_unlock_by_era_and_scale_speed() {
    assert_eq!(VehicleKind::Cart.unlock_era(), 1);
    assert_eq!(VehicleKind::Wagon.unlock_era(), 2);
    assert!(VehicleKind::Wagon.speed_multiplier() > VehicleKind::Cart.speed_multiplier());
    for kind in [VehicleKind::Cart, VehicleKind::Wagon] {
        assert!(
            kind.speed_multiplier() > 1.0,
            "{kind:?} must accelerate movement on top of the road multiplier"
        );
    }

    let at = wc(0, 0);
    let mut g = TrafficGraph::new();
    assert!(!g.place_vehicle(VehicleKind::Wagon, at, 1, InfraProvenance::UserPlaced));
    assert!(!g.place_vehicle(VehicleKind::Wagon, at, 0, InfraProvenance::Emergent));
    assert!(g.place_vehicle(VehicleKind::Cart, at, 1, InfraProvenance::Emergent));
    assert!(g.place_vehicle(VehicleKind::Wagon, wc(2, 0), 2, InfraProvenance::UserPlaced));

    assert_eq!(
        g.vehicles.len(),
        2,
        "rejected placements must not be recorded"
    );
    assert_eq!(g.vehicles[0].kind, VehicleKind::Cart);
    assert_eq!(g.vehicles[0].at, (0, 0, 0));
    assert_eq!(g.vehicles[0].provenance, InfraProvenance::Emergent);
    assert_eq!(g.vehicles[1].kind, VehicleKind::Wagon);
    assert_eq!(g.vehicles[1].at, (2, 0, 0));
    assert_eq!(g.vehicles[1].provenance, InfraProvenance::UserPlaced);
}

// ---------------------------------------------------------------------------
// FR-CIV-ROAD-920 — manual road tools: place / curve / snap / upgrade
// ---------------------------------------------------------------------------

/// Covers FR-CIV-ROAD-920.
///
/// "Road tool with curves + tier upgrade": a drag-to-draw polyline (the curve)
/// creates one segment per consecutive pair, the upgrade tool raises the rung,
/// and a weaker re-drag never downgrades what was already drawn. Snapping is
/// the canonical undirected edge key, so re-drawing a reversed pair edits the
/// same segment instead of stacking a duplicate.
#[test]
fn fr_civ_road_920_place_path_draws_a_polyline_and_upgrades() {
    let pts = [wc(0, 0), wc(1, 0), wc(2, 0), wc(2, 1)];
    let mut g = TrafficGraph::new();

    g.place_path(&pts, RoadKind::Trail);
    assert_eq!(g.iter_segments().count(), 3, "4 points -> 3 segments");
    for pair in pts.windows(2) {
        assert_eq!(g.kind_between(pair[0], pair[1]), RoadKind::Trail);
    }
    assert!(g
        .iter_segments()
        .all(|(_, s)| s.provenance == InfraProvenance::UserPlaced));

    // Upgrade tool: re-drag the same corridor at a higher tier.
    g.place_path(&pts, RoadKind::Highway);
    assert_eq!(g.count_at_least(RoadKind::Highway), 3);
    assert_eq!(
        g.iter_segments().count(),
        3,
        "an upgrade must not add edges"
    );

    // Downgrade attempt: the tool is monotone per edge.
    g.place_path(&pts, RoadKind::Trail);
    assert_eq!(
        g.count_at_least(RoadKind::Highway),
        3,
        "a weaker re-drag must never downgrade an existing road"
    );

    // Snapping: drawing the segment backwards targets the same canonical edge.
    let before = g.iter_segments().count();
    g.place_segment(pts[1], pts[0], RoadKind::Highway);
    assert_eq!(
        g.iter_segments().count(),
        before,
        "reversed endpoints must snap onto the existing segment"
    );

    // Fewer than two points is a documented no-op.
    g.place_path(&[wc(9, 9)], RoadKind::Highway);
    assert_eq!(g.iter_segments().count(), before);
    g.place_path(&[], RoadKind::Highway);
    assert_eq!(g.iter_segments().count(), before);
}

/// Covers FR-CIV-ROAD-920.
///
/// "the sim then treats identically": a player road and a grown road must be
/// read the same way, and heavy use of a player road must grow it up the ladder
/// (same promotion logic as emergent) while keeping its authoring tag. A placed
/// bridge is terminal — traffic cannot rewrite a structure the player built.
#[test]
fn fr_civ_road_920_player_edges_join_the_same_graph() {
    let (a, b) = (wc(0, 0), wc(1, 0));

    let mut g = TrafficGraph::new();
    g.place_segment(a, b, RoadKind::Trail);
    assert_eq!(g.record_traffic(a, b, 140.0), RoadKind::Highway);
    let seg = g.iter_segments().next().expect("segment").1;
    assert_eq!(seg.traffic, 140.0, "traffic is tallied on player roads too");
    assert_eq!(
        seg.provenance,
        InfraProvenance::UserPlaced,
        "growth must not rewrite the authoring channel"
    );

    let mut emergent = TrafficGraph::new();
    emergent.record_traffic(a, b, 140.0);
    assert_eq!(
        emergent.speed_multiplier_at(a, b),
        g.speed_multiplier_at(a, b),
        "the sim must read a player road exactly like a grown one"
    );
    assert_eq!(
        emergent.count_at_least(RoadKind::Road),
        g.count_at_least(RoadKind::Road)
    );

    // A placed bridge is terminal: no amount of use downgrades it.
    let mut bridge = TrafficGraph::new();
    bridge.place_segment(a, b, RoadKind::Bridge);
    assert_eq!(bridge.kind_between(a, b), RoadKind::Bridge);
    assert_eq!(
        bridge.record_traffic(a, b, 1_000_000.0),
        RoadKind::Bridge,
        "a bridge is a placed structure; traffic must not rewrite it"
    );
    assert_eq!(
        bridge.speed_multiplier_at(a, b),
        RoadKind::Road.speed_multiplier(),
        "a bridge is a road-grade span"
    );

    // ...and a bridge never appears on its own (ladder ordering is explicit).
    assert!(RoadKind::Bridge > RoadKind::Highway);
    assert!(RoadKind::Highway > RoadKind::Road);
    assert!(RoadKind::Road > RoadKind::Trail);
    assert!(RoadKind::Trail > RoadKind::None);
}

/// Covers FR-CIV-ROAD-920.
///
/// Tool boundaries: a zero-length placement, a self-edge traversal and any
/// non-positive traffic weight must be rejected outright rather than creating
/// phantom infrastructure or NaN state.
#[test]
fn fr_civ_road_920_zero_length_and_non_positive_traffic_are_rejected() {
    let (a, b) = (wc(0, 0), wc(1, 0));
    let mut g = TrafficGraph::new();

    g.place_segment(a, a, RoadKind::Highway);
    assert_eq!(g.iter_segments().count(), 0, "zero-length road rejected");
    assert_eq!(g.record_traffic(a, a, 140.0), RoadKind::None);
    assert_eq!(g.record_traffic(a, b, 0.0), RoadKind::None);
    assert_eq!(g.record_traffic(a, b, -5.0), RoadKind::None);
    assert_eq!(
        g.iter_segments().count(),
        0,
        "self-edges and non-positive weights must not create infrastructure"
    );
    assert_eq!(g.count_at_least(RoadKind::Trail), 0);

    // A valid placement afterwards still works normally.
    g.place_segment(a, b, RoadKind::Road);
    assert_eq!(
        g.speed_multiplier_at(a, b),
        RoadKind::Road.speed_multiplier()
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-ROAD-921 — designation as a lens/hint, not a hard zoning enum
// ---------------------------------------------------------------------------

/// Covers FR-CIV-ROAD-921.
///
/// Coverage boundary: the district/zoning overlay itself has **no
/// implementation in this repository** — a case-insensitive search for
/// `district`/`zoning`/`designate`/`lens` finds only `crates/economy/src/
/// district.rs` (FR-ECON-008 energy collapse, unrelated) and doc text. The
/// property the requirement turns on is testable here, though: a player
/// designation is a *lens that biases* the emergent system rather than an enum
/// that forces an outcome. A designated edge keeps its drawn rung while idle,
/// emergent use may still override it, and the designation stays local.
#[test]
fn fr_civ_road_921_designation_biases_but_never_forces() {
    let (a, b, c) = (wc(0, 0), wc(1, 0), wc(2, 0));
    let mut g = TrafficGraph::new();

    // The player designates one edge only.
    g.place_segment(a, b, RoadKind::Trail);
    assert_eq!(g.kind_between(a, b), RoadKind::Trail);
    assert_eq!(
        g.kind_between(b, c),
        RoadKind::None,
        "a designation is a named region overlay, not a global rule: it must \
         not spill onto undesignated land"
    );

    // The designation biases but does not pin: heavier use overrides it.
    assert_eq!(
        g.record_traffic(a, b, 140.0),
        RoadKind::Highway,
        "the lens must not freeze the type it hinted at"
    );
    assert_eq!(
        g.kind_between(b, c),
        RoadKind::None,
        "the neighbouring edge that saw no use stays bare"
    );
}

/// Covers FR-CIV-ROAD-921.
///
/// "influences agent preference weights, does not force building types":
/// designation alone must not manufacture growth, and identical use must give
/// the identical rung whether or not the land was declared. (See the boundary
/// note above: the overlay itself is unimplemented; this asserts the
/// non-forcing property on the implemented designation surface.)
#[test]
fn fr_civ_road_921_designation_does_not_force_growth() {
    let pts = [wc(0, 0), wc(1, 0), wc(2, 0)];

    let mut designated = TrafficGraph::new();
    designated.place_path(&pts, RoadKind::Trail);
    assert_eq!(designated.iter_segments().count(), 2);
    for (_, seg) in designated.iter_segments() {
        assert_eq!(seg.kind, RoadKind::Trail);
        assert_eq!(
            seg.traffic, 0.0,
            "declaring a region must not fabricate the use it is meant to hint at"
        );
    }

    // Identical use on declared vs undeclared land -> identical rung.
    let mut undeclared = TrafficGraph::new();
    undeclared.record_traffic(pts[0], pts[1], 40.0);
    designated.record_traffic(pts[0], pts[1], 40.0);
    assert_eq!(
        designated.kind_between(pts[0], pts[1]),
        undeclared.kind_between(pts[0], pts[1]),
        "declaration must not change the outcome for equal use"
    );
    assert_eq!(designated.kind_between(pts[0], pts[1]), RoadKind::Road);

    // And the declared corridor's untouched neighbour is unaffected: the hint
    // biases the region it names, it never forces a land-use plan.
    assert_eq!(designated.kind_between(pts[1], pts[2]), RoadKind::Trail);
    assert_eq!(designated.count_at_least(RoadKind::Highway), 0);
    assert_eq!(undeclared.count_at_least(RoadKind::Trail), 1);
}
