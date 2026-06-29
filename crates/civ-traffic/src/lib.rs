//! civ-traffic — Emergent + user-authored infrastructure layer for Civis.
//!
//! Two authoring channels feed ONE shared graph (the dual-authored vision):
//!
//! * **Emergent** — agents accumulate traffic weight on the edges they walk.
//!   Once an edge's accumulated traffic crosses a threshold it is *promoted*
//!   along the desire-path ladder: `None -> Trail -> Road -> Highway`. No
//!   central planner is required (WorldBox / Manor Lords style self-organising).
//! * **User** — the player freehand-places roads/trails/highways/bridges (and
//!   structures + vehicles) via `spawn_tools.rs`. User-placed segments enter the
//!   SAME [`TrafficGraph`] carrying [`InfraProvenance::UserPlaced`], so the
//!   emergent economy uses player-built roads identically to grown ones.
//!
//! Roads feed back into the life-sim: [`TrafficGraph::speed_multiplier_at`]
//! exposes a per-edge movement multiplier the pathing cost model reads, so
//! agents prefer (and move faster along) established roads — closing the
//! desire-path feedback loop (Cities-Skylines style road influence).
//!
//! Determinism: the graph is order-deterministic (BTreeMap-keyed); identical
//! event order yields an identical graph (asserted by FR-CIV-INFRA-030).
//!
//! Functional requirements: FR-CIV-INFRA-*.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;

use civ_voxel::WorldCoord;
use serde::{Deserialize, Serialize};

pub mod grid;
pub mod congestion;
pub mod lane;

pub use congestion::PathCongestion;
pub use grid::{
    CellState, GridCell, ServiceGrid, ServiceGridError, ServiceKind, SERVICE_GRID_SCHEMA_VERSION,
};

/// Marker version of this crate's public schema (replay/save guard).
pub const SCHEMA_VERSION: &str = "0.1.0";

/// Who authored a piece of infrastructure. User-placed and emergent segments
/// share every other data tag so the economy treats them identically; this only
/// lets the renderer style them differently and lets saves audit provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InfraProvenance {
    /// Grown by accumulated agent traffic (desire path).
    Emergent,
    /// Freehand-placed by the player.
    UserPlaced,
}

/// Rung on the desire-path ladder. Ordered weakest -> strongest; `as u8`
/// is the promotion rank.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RoadKind {
    /// Bare ground; no infrastructure yet.
    None,
    /// Foot-worn desire path.
    Trail,
    /// Surfaced road.
    Road,
    /// High-throughput highway.
    Highway,
    /// Road spanning water (placed, never emerges).
    Bridge,
}

pub use lane::{
    lanes_for, route_lanes, speed_for_lane, Lane, LaneClass, LaneConnection, LaneDirection,
    LaneGraph, Node, NodeKey,
};

impl RoadKind {
    /// Movement-speed multiplier the life-sim pathing cost model reads. Higher
    /// is faster (lower traversal cost). Bare ground is the `1.0` baseline.
    #[must_use]
    pub fn speed_multiplier(self) -> f32 {
        match self {
            RoadKind::None => 1.0,
            RoadKind::Trail => 1.25,
            RoadKind::Road => 1.8,
            RoadKind::Bridge => 1.8,
            RoadKind::Highway => 2.5,
        }
    }

    /// Next rung up the emergent ladder (`Bridge` is terminal — placed only).
    #[must_use]
    pub fn promoted(self) -> RoadKind {
        match self {
            RoadKind::None => RoadKind::Trail,
            RoadKind::Trail => RoadKind::Road,
            RoadKind::Road | RoadKind::Highway => RoadKind::Highway,
            RoadKind::Bridge => RoadKind::Bridge,
        }
    }
}

/// Accumulated-traffic thresholds at which an emergent edge promotes to the
/// *next* rung.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PromotionThresholds {
    /// Traffic to go None -> Trail.
    pub trail: f32,
    /// Traffic to go Trail -> Road.
    pub road: f32,
    /// Traffic to go Road -> Highway.
    pub highway: f32,
}

impl Default for PromotionThresholds {
    fn default() -> Self {
        Self {
            trail: 8.0,
            road: 32.0,
            highway: 128.0,
        }
    }
}

impl PromotionThresholds {
    /// Highest [`RoadKind`] an emergent edge with `traffic` accumulated may hold.
    #[must_use]
    pub fn kind_for(self, traffic: f32) -> RoadKind {
        if traffic >= self.highway {
            RoadKind::Highway
        } else if traffic >= self.road {
            RoadKind::Road
        } else if traffic >= self.trail {
            RoadKind::Trail
        } else {
            RoadKind::None
        }
    }
}

/// Undirected edge key between two world cells. Endpoints are stored in a
/// canonical (sorted) order so `(a,b)` and `(b,a)` map to the same segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EdgeKey {
    /// Lower-sorted endpoint.
    pub a: (i64, i64, i64),
    /// Higher-sorted endpoint.
    pub b: (i64, i64, i64),
}

impl EdgeKey {
    /// Build a canonical undirected edge key from two world coords.
    #[must_use]
    pub fn new(from: WorldCoord, to: WorldCoord) -> Self {
        let p = (from.x, from.y, from.z);
        let q = (to.x, to.y, to.z);
        if p <= q {
            Self { a: p, b: q }
        } else {
            Self { a: q, b: p }
        }
    }
}

/// One infrastructure segment in the shared graph.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RoadSegment {
    /// Current rung on the desire-path ladder.
    pub kind: RoadKind,
    /// Accumulated traffic weight (drives emergent promotion).
    pub traffic: f32,
    /// Authoring channel.
    pub provenance: InfraProvenance,
}

/// Tech / resource unlock tier for mechanical movement abstractions. Vehicles
/// accelerate movement + trade once their tier is unlocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VehicleKind {
    /// Hand cart — earliest unlock.
    Cart,
    /// Draft wagon — needs road + livestock tier.
    Wagon,
}

impl VehicleKind {
    /// Multiplier applied on top of the road multiplier when a unit uses this
    /// vehicle (carts/wagons accelerate movement + trade throughput).
    #[must_use]
    pub fn speed_multiplier(self) -> f32 {
        match self {
            VehicleKind::Cart => 1.3,
            VehicleKind::Wagon => 1.7,
        }
    }

    /// Minimum tech era at which this vehicle abstraction unlocks.
    #[must_use]
    pub fn unlock_era(self) -> u16 {
        match self {
            VehicleKind::Cart => 1,
            VehicleKind::Wagon => 2,
        }
    }
}

/// A placed vehicle instance (user- or sim-authored). Seats on a road cell.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vehicle {
    /// Vehicle archetype.
    pub kind: VehicleKind,
    /// World cell the vehicle currently occupies.
    pub at: (i64, i64, i64),
    /// Authoring channel.
    pub provenance: InfraProvenance,
}

/// Shared, dual-authored infrastructure graph. Both the emergent traffic system
/// and the freehand placement tools mutate this single structure.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TrafficGraph {
    /// Segments keyed by canonical undirected edge.
    pub segments: BTreeMap<EdgeKey, RoadSegment>,
    /// Placed vehicles (deterministic order).
    pub vehicles: Vec<Vehicle>,
    /// Promotion thresholds for emergent growth.
    pub thresholds: PromotionThresholds,
}

impl TrafficGraph {
    /// Empty graph with default thresholds.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Accumulate `weight` of traffic on the edge between `from` and `to`
    /// (an agent walked it), then promote the segment if it crossed a
    /// threshold. Creates the segment on first traversal. Returns the resulting
    /// [`RoadKind`]. User-placed segments keep their placed rank and provenance
    /// but still tally traffic (so the renderer can show wear/use).
    pub fn record_traffic(&mut self, from: WorldCoord, to: WorldCoord, weight: f32) -> RoadKind {
        if from == to || weight <= 0.0 {
            return self.kind_between(from, to);
        }
        let key = EdgeKey::new(from, to);
        let thresholds = self.thresholds;
        let seg = self.segments.entry(key).or_insert(RoadSegment {
            kind: RoadKind::None,
            traffic: 0.0,
            provenance: InfraProvenance::Emergent,
        });
        seg.traffic += weight;
        // Emergent segments climb the ladder by accumulated traffic. User-placed
        // segments never downgrade below what the player drew, but DO upgrade if
        // heavy use would have grown an even higher rung.
        if seg.provenance == InfraProvenance::Emergent {
            seg.kind = thresholds.kind_for(seg.traffic);
        } else {
            let grown = thresholds.kind_for(seg.traffic);
            if grown > seg.kind && grown != RoadKind::Bridge {
                seg.kind = grown;
            }
        }
        seg.kind
    }

    /// Freehand-place (or upgrade) a segment between two cells with an explicit
    /// [`RoadKind`]. Tagged [`InfraProvenance::UserPlaced`]. A later, stronger
    /// placement upgrades; a weaker one never downgrades an existing road.
    pub fn place_segment(&mut self, from: WorldCoord, to: WorldCoord, kind: RoadKind) {
        if from == to {
            return;
        }
        let key = EdgeKey::new(from, to);
        self.segments
            .entry(key)
            .and_modify(|seg| {
                if kind > seg.kind {
                    seg.kind = kind;
                }
                seg.provenance = InfraProvenance::UserPlaced;
            })
            .or_insert(RoadSegment {
                kind,
                traffic: 0.0,
                provenance: InfraProvenance::UserPlaced,
            });
    }

    /// Place a connected polyline of segments (drag-to-draw). Consecutive points
    /// are joined as undirected edges. Fewer than two points is a no-op.
    pub fn place_path(&mut self, points: &[WorldCoord], kind: RoadKind) {
        for window in points.windows(2) {
            self.place_segment(window[0], window[1], kind);
        }
    }

    /// Current [`RoadKind`] between two cells, or [`RoadKind::None`] if no
    /// segment exists.
    #[must_use]
    pub fn kind_between(&self, from: WorldCoord, to: WorldCoord) -> RoadKind {
        if from == to {
            return RoadKind::None;
        }
        self.segments
            .get(&EdgeKey::new(from, to))
            .map_or(RoadKind::None, |seg| seg.kind)
    }

    /// Movement-speed multiplier the life-sim pathing cost model reads for the
    /// edge between `from` and `to`. `1.0` (baseline) when no road exists, so an
    /// agent always has a defined cost. This is the shared road layer the
    /// Life-Sim Lead's engine reads to make road travel cheaper/faster.
    #[must_use]
    pub fn speed_multiplier_at(&self, from: WorldCoord, to: WorldCoord) -> f32 {
        self.kind_between(from, to).speed_multiplier()
    }

    /// Place a vehicle if its tier is unlocked at `era`. Returns `true` when the
    /// vehicle was added (tech/resource threshold met), `false` otherwise.
    pub fn place_vehicle(
        &mut self,
        kind: VehicleKind,
        at: WorldCoord,
        era: u16,
        provenance: InfraProvenance,
    ) -> bool {
        if era < kind.unlock_era() {
            return false;
        }
        self.vehicles.push(Vehicle {
            kind,
            at: (at.x, at.y, at.z),
            provenance,
        });
        true
    }

    /// Number of segments at or above a given rung (renderer LOD / stats).
    #[must_use]
    pub fn count_at_least(&self, min: RoadKind) -> usize {
        self.segments.values().filter(|s| s.kind >= min).count()
    }

    /// Iterate every segment with its endpoints, in deterministic edge order,
    /// so the renderer can draw the grown + placed network.
    pub fn iter_segments(&self) -> impl Iterator<Item = (EdgeKey, &RoadSegment)> {
        self.segments.iter().map(|(k, v)| (*k, v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wc(x: i64, z: i64) -> WorldCoord {
        WorldCoord { x, y: 0, z }
    }

    /// FR-CIV-INFRA-001 — graph round-trips through RON without loss.
    #[test]
    fn graph_round_trips_through_ron() {
        let mut g = TrafficGraph::new();
        g.place_segment(wc(0, 0), wc(1, 0), RoadKind::Road);
        g.record_traffic(wc(0, 0), wc(0, 1), 10.0);
        g.place_vehicle(VehicleKind::Cart, wc(0, 0), 1, InfraProvenance::UserPlaced);
        let encoded = ron::to_string(&g).expect("serialize");
        let decoded: TrafficGraph = ron::from_str(&encoded).expect("deserialize");
        assert_eq!(g, decoded);
    }

    /// FR-CIV-INFRA-010 — accumulated traffic promotes an edge up the ladder.
    #[test]
    fn traffic_promotes_desire_path_ladder() {
        let mut g = TrafficGraph::new();
        let (a, b) = (wc(0, 0), wc(1, 0));
        assert_eq!(g.record_traffic(a, b, 4.0), RoadKind::None);
        assert_eq!(g.record_traffic(a, b, 5.0), RoadKind::Trail); // 9 >= 8
        assert_eq!(g.record_traffic(a, b, 24.0), RoadKind::Road); // 33 >= 32
        assert_eq!(g.record_traffic(a, b, 100.0), RoadKind::Highway); // 133 >= 128
    }

    /// FR-CIV-INFRA-011 — edges are undirected (a,b == b,a).
    #[test]
    fn edges_are_undirected() {
        let mut g = TrafficGraph::new();
        g.record_traffic(wc(2, 3), wc(2, 4), 10.0);
        assert_eq!(g.kind_between(wc(2, 4), wc(2, 3)), RoadKind::Trail);
        assert_eq!(g.segments.len(), 1);
    }

    /// FR-CIV-INFRA-020 — roads make travel faster for the life-sim cost model.
    #[test]
    fn roads_speed_up_pathing() {
        let mut g = TrafficGraph::new();
        let (a, b) = (wc(0, 0), wc(1, 0));
        assert_eq!(g.speed_multiplier_at(a, b), 1.0); // bare ground baseline
        g.place_segment(a, b, RoadKind::Highway);
        assert!(g.speed_multiplier_at(a, b) > 2.0);
    }

    /// FR-CIV-INFRA-021 — user-placed roads share data tags with emergent ones
    /// (same graph, same multiplier path), only provenance differs.
    #[test]
    fn user_and_emergent_share_one_graph() {
        let mut g = TrafficGraph::new();
        g.place_segment(wc(0, 0), wc(1, 0), RoadKind::Road);
        g.record_traffic(wc(5, 0), wc(6, 0), 40.0);
        let placed = g.segments[&EdgeKey::new(wc(0, 0), wc(1, 0))];
        let grown = g.segments[&EdgeKey::new(wc(5, 0), wc(6, 0))];
        assert_eq!(placed.provenance, InfraProvenance::UserPlaced);
        assert_eq!(grown.provenance, InfraProvenance::Emergent);
        // Both are real Roads the economy uses identically.
        assert_eq!(placed.kind, RoadKind::Road);
        assert_eq!(grown.kind, RoadKind::Road);
    }

    /// FR-CIV-INFRA-022 — user-placed road never downgrades but heavy use can
    /// upgrade it.
    #[test]
    fn user_road_upgrades_under_heavy_use_never_downgrades() {
        let mut g = TrafficGraph::new();
        let (a, b) = (wc(0, 0), wc(1, 0));
        g.place_segment(a, b, RoadKind::Road);
        g.record_traffic(a, b, 1.0); // tiny traffic must not demote to None/Trail
        assert_eq!(g.kind_between(a, b), RoadKind::Road);
        g.record_traffic(a, b, 200.0); // heavy use -> Highway
        assert_eq!(g.kind_between(a, b), RoadKind::Highway);
    }

    /// FR-CIV-INFRA-030 — identical event order yields identical graph.
    #[test]
    fn emergent_growth_is_deterministic() {
        let build = || {
            let mut g = TrafficGraph::new();
            for i in 0..50i64 {
                g.record_traffic(wc(i % 4, 0), wc((i + 1) % 4, 0), 3.0);
            }
            g
        };
        assert_eq!(build(), build());
    }

    /// FR-CIV-INFRA-040 — vehicles unlock only at/after their tech era.
    #[test]
    fn vehicles_gate_on_tech_era() {
        let mut g = TrafficGraph::new();
        assert!(!g.place_vehicle(VehicleKind::Wagon, wc(0, 0), 1, InfraProvenance::Emergent));
        assert!(g.place_vehicle(VehicleKind::Wagon, wc(0, 0), 2, InfraProvenance::Emergent));
        assert!(g.place_vehicle(VehicleKind::Cart, wc(0, 0), 1, InfraProvenance::UserPlaced));
        assert_eq!(g.vehicles.len(), 2);
    }

    /// FR-CIV-INFRA-050 — drag-to-draw lays a connected polyline of segments.
    #[test]
    fn place_path_lays_connected_segments() {
        let mut g = TrafficGraph::new();
        let pts = [wc(0, 0), wc(1, 0), wc(2, 0), wc(3, 0)];
        g.place_path(&pts, RoadKind::Trail);
        assert_eq!(g.segments.len(), 3);
        assert_eq!(g.count_at_least(RoadKind::Trail), 3);
    }

    /// FR-CIV-INFRA-060 — bridges are placed (terminal kind), priced like roads.
    #[test]
    fn bridges_place_and_price_like_roads() {
        let mut g = TrafficGraph::new();
        g.place_segment(wc(0, 0), wc(0, 5), RoadKind::Bridge);
        assert_eq!(g.kind_between(wc(0, 0), wc(0, 5)), RoadKind::Bridge);
        assert!(
            (RoadKind::Bridge.speed_multiplier() - RoadKind::Road.speed_multiplier()).abs() < 1e-6
        );
    }
}
