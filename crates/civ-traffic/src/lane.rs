//! Tier-2 lane graph layered under the existing road promotion ladder.
//!
//! FR-CIV-TRAFFIC-LANE-001: promote segments into lanes by road class.
//! FR-CIV-TRAFFIC-LANE-002: connect lanes through nodes for routing.
//! FR-CIV-TRAFFIC-LANE-003: keep the scalar road-speed graph unchanged.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use civ_voxel::WorldCoord;
use serde::{Deserialize, Serialize};

use crate::{EdgeKey, RoadKind, RoadSegment, TrafficGraph};

/// Stable node identifier for the lane net.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeKey {
    /// World coordinate used as the node anchor.
    pub at: (i64, i64, i64),
}

impl From<WorldCoord> for NodeKey {
    fn from(value: WorldCoord) -> Self {
        Self {
            at: (value.x, value.y, value.z),
        }
    }
}

/// A lane-routing node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// Node location.
    pub key: NodeKey,
}

/// Lane class derived from the road class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LaneClass {
    /// Foot traffic.
    Trail,
    /// Local road traffic.
    Road,
    /// Fast highway traffic.
    Highway,
}

/// Direction of travel allowed on a lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LaneDirection {
    /// Travel from `a` toward `b`.
    AB,
    /// Travel from `b` toward `a`.
    BA,
    /// Both directions are allowed.
    Both,
}

/// One lane generated from a promoted segment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lane {
    /// Segment this lane belongs to.
    pub segment: EdgeKey,
    /// Lane index within the segment.
    pub index: usize,
    /// Lane class derived from the road rung.
    pub class: LaneClass,
    /// Allowed travel direction.
    pub direction: LaneDirection,
}

/// Lane-to-lane connection across a node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneConnection {
    /// Source lane.
    pub from: LaneRef,
    /// Destination lane.
    pub to: LaneRef,
    /// Node where the turn occurs.
    pub node: NodeKey,
}

/// Compact lane reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LaneRef {
    /// Segment the lane lives on.
    pub segment: EdgeKey,
    /// Lane index within the segment.
    pub index: usize,
}

/// Derived lane graph.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LaneGraph {
    /// Known nodes.
    pub nodes: BTreeMap<NodeKey, Node>,
    /// Lanes generated from promoted road segments.
    pub lanes: BTreeMap<LaneRef, Lane>,
    /// Lane connections keyed by source lane.
    pub connections: BTreeMap<LaneRef, Vec<LaneConnection>>,
}

impl LaneGraph {
    /// Build a lane graph from the current traffic graph.
    #[must_use]
    pub fn from_traffic(graph: &TrafficGraph) -> Self {
        let mut lane_graph = Self::default();
        let mut node_to_lanes: BTreeMap<NodeKey, Vec<LaneRef>> = BTreeMap::new();

        for (edge, segment) in graph.iter_segments() {
            let lanes = lanes_for(edge, segment);
            let a = NodeKey::from(WorldCoord {
                x: edge.a.0,
                y: edge.a.1,
                z: edge.a.2,
            });
            let b = NodeKey::from(WorldCoord {
                x: edge.b.0,
                y: edge.b.1,
                z: edge.b.2,
            });
            lane_graph.nodes.entry(a).or_insert(Node { key: a });
            lane_graph.nodes.entry(b).or_insert(Node { key: b });

            for lane in lanes {
                let lane_ref = LaneRef {
                    segment: edge,
                    index: lane.index,
                };
                lane_graph.lanes.insert(lane_ref, lane.clone());
                let (start, end) = lane_endpoints(edge, lane.direction);
                node_to_lanes.entry(start).or_default().push(lane_ref);
                node_to_lanes.entry(end).or_default().push(lane_ref);
            }
        }

        for (node, lanes) in node_to_lanes {
            for &from in &lanes {
                for &to in &lanes {
                    if from == to {
                        continue;
                    }
                    if !lane_can_connect(&lane_graph.lanes[&from], &lane_graph.lanes[&to], node) {
                        continue;
                    }
                    lane_graph
                        .connections
                        .entry(from)
                        .or_default()
                        .push(LaneConnection { from, to, node });
                }
            }
        }

        lane_graph
    }

    /// Route across lanes from one node to another.
    #[must_use]
    pub fn route_lanes(&self, from_node: NodeKey, to_node: NodeKey) -> Vec<LaneConnection> {
        let starts = self.lanes_starting_at(from_node);
        let mut goal = BTreeSet::new();
        for lane in self.lanes_ending_at(to_node) {
            goal.insert(lane);
        }
        self.shortest_lane_route(&starts, &goal).unwrap_or_default()
    }

    fn lanes_starting_at(&self, node: NodeKey) -> Vec<LaneRef> {
        self.lanes
            .iter()
            .filter_map(|(lane_ref, lane)| {
                if lane_starts_at(lane, node) {
                    Some(*lane_ref)
                } else {
                    None
                }
            })
            .collect()
    }

    fn lanes_ending_at(&self, node: NodeKey) -> Vec<LaneRef> {
        self.lanes
            .iter()
            .filter_map(|(lane_ref, lane)| {
                if lane_ends_at(lane, node) {
                    Some(*lane_ref)
                } else {
                    None
                }
            })
            .collect()
    }

    fn shortest_lane_route(
        &self,
        starts: &[LaneRef],
        goals: &BTreeSet<LaneRef>,
    ) -> Option<Vec<LaneConnection>> {
        let mut queue = VecDeque::new();
        let mut prev: BTreeMap<LaneRef, (LaneRef, LaneConnection)> = BTreeMap::new();
        let mut seen = BTreeSet::new();

        for &start in starts {
            queue.push_back(start);
            seen.insert(start);
        }

        while let Some(current) = queue.pop_front() {
            if goals.contains(&current) {
                return Some(reconstruct_path(current, &prev));
            }
            for conn in self.connections.get(&current).into_iter().flatten() {
                if seen.insert(conn.to) {
                    prev.insert(conn.to, (current, conn.clone()));
                    queue.push_back(conn.to);
                }
            }
        }
        None
    }
}

/// Generate lanes for one promoted segment.
#[must_use]
pub fn lanes_for(segment: EdgeKey, road: &RoadSegment) -> Vec<Lane> {
    let (class, count) = match road.kind {
        RoadKind::None => return Vec::new(),
        RoadKind::Trail => (LaneClass::Trail, 1),
        RoadKind::Road => (LaneClass::Road, 2),
        RoadKind::Highway | RoadKind::Bridge => (LaneClass::Highway, 3),
    };

    let directions = match count {
        1 => vec![LaneDirection::Both],
        2 => vec![LaneDirection::AB, LaneDirection::BA],
        3 => vec![LaneDirection::AB, LaneDirection::BA, LaneDirection::AB],
        _ => unreachable!(),
    };

    directions
        .into_iter()
        .enumerate()
        .map(|(index, direction)| Lane {
            segment,
            index,
            class,
            direction,
        })
        .collect()
}

/// Derive a speed multiplier for a lane without disturbing the scalar graph.
#[must_use]
pub fn speed_for_lane(lane: &Lane) -> f32 {
    match lane.class {
        LaneClass::Trail => RoadKind::Trail.speed_multiplier(),
        LaneClass::Road => RoadKind::Road.speed_multiplier(),
        LaneClass::Highway => RoadKind::Highway.speed_multiplier(),
    }
}

/// Route through the lane graph between two nodes.
#[must_use]
pub fn route_lanes(graph: &LaneGraph, from_node: NodeKey, to_node: NodeKey) -> Vec<LaneConnection> {
    graph.route_lanes(from_node, to_node)
}

fn reconstruct_path(
    mut current: LaneRef,
    prev: &BTreeMap<LaneRef, (LaneRef, LaneConnection)>,
) -> Vec<LaneConnection> {
    let mut path = Vec::new();
    while let Some((parent, conn)) = prev.get(&current) {
        path.push(conn.clone());
        current = *parent;
    }
    path.reverse();
    path
}

fn lane_endpoints(edge: EdgeKey, direction: LaneDirection) -> (NodeKey, NodeKey) {
    let a = NodeKey { at: edge.a };
    let b = NodeKey { at: edge.b };
    match direction {
        LaneDirection::AB => (a, b),
        LaneDirection::BA => (b, a),
        LaneDirection::Both => (a, b),
    }
}

fn lane_can_connect(from: &Lane, to: &Lane, node: NodeKey) -> bool {
    let from_ok = lane_ends_at(from, node);
    let to_ok = lane_starts_at(to, node);
    from_ok && to_ok && from.segment != to.segment
}

fn lane_starts_at(lane: &Lane, node: NodeKey) -> bool {
    let a = NodeKey { at: lane.segment.a };
    let b = NodeKey { at: lane.segment.b };
    (matches!(lane.direction, LaneDirection::AB | LaneDirection::Both) && a == node)
        || (matches!(lane.direction, LaneDirection::BA | LaneDirection::Both) && b == node)
}

fn lane_ends_at(lane: &Lane, node: NodeKey) -> bool {
    let a = NodeKey { at: lane.segment.a };
    let b = NodeKey { at: lane.segment.b };
    (matches!(lane.direction, LaneDirection::AB | LaneDirection::Both) && b == node)
        || (matches!(lane.direction, LaneDirection::BA | LaneDirection::Both) && a == node)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wc(x: i64, z: i64) -> WorldCoord {
        WorldCoord { x, y: 0, z }
    }

    fn edge(a: WorldCoord, b: WorldCoord) -> EdgeKey {
        EdgeKey::new(a, b)
    }

    /// FR-CIV-TRAFFIC-LANE-001 — lane counts follow the promotion ladder.
    #[test]
    fn lanes_follow_road_kind_ladder() {
        let trail = RoadSegment {
            kind: RoadKind::Trail,
            traffic: 0.0,
            provenance: crate::InfraProvenance::Emergent,
        };
        let road = RoadSegment {
            kind: RoadKind::Road,
            traffic: 0.0,
            provenance: crate::InfraProvenance::Emergent,
        };
        let highway = RoadSegment {
            kind: RoadKind::Highway,
            traffic: 0.0,
            provenance: crate::InfraProvenance::Emergent,
        };
        assert_eq!(lanes_for(edge(wc(0, 0), wc(1, 0)), &trail).len(), 1);
        assert_eq!(lanes_for(edge(wc(0, 0), wc(1, 0)), &road).len(), 2);
        assert_eq!(lanes_for(edge(wc(0, 0), wc(1, 0)), &highway).len(), 3);
    }

    /// FR-CIV-TRAFFIC-LANE-002 — nodes connect lanes across shared junctions.
    #[test]
    fn route_lanes_crosses_shared_node() {
        let mut g = TrafficGraph::new();
        g.place_segment(wc(0, 0), wc(1, 0), RoadKind::Road);
        g.place_segment(wc(1, 0), wc(2, 0), RoadKind::Road);
        let lanes = LaneGraph::from_traffic(&g);
        let path = lanes.route_lanes(NodeKey::from(wc(0, 0)), NodeKey::from(wc(2, 0)));
        assert!(!path.is_empty());
        assert_eq!(path.first().unwrap().from.segment, edge(wc(0, 0), wc(1, 0)));
        assert_eq!(path.last().unwrap().to.segment, edge(wc(1, 0), wc(2, 0)));
    }

    /// FR-CIV-TRAFFIC-LANE-003 — lane speed mirrors the existing scalar layer.
    #[test]
    fn lane_speed_matches_road_speed() {
        let lane = Lane {
            segment: edge(wc(0, 0), wc(1, 0)),
            index: 0,
            class: LaneClass::Highway,
            direction: LaneDirection::AB,
        };
        assert_eq!(speed_for_lane(&lane), RoadKind::Highway.speed_multiplier());
    }

    /// FR-CIV-TRAFFIC-LANE-004 — the legacy scalar speed graph still works.
    #[test]
    fn scalar_speed_graph_stays_intact() {
        let mut g = TrafficGraph::new();
        let a = wc(0, 0);
        let b = wc(1, 0);
        assert_eq!(g.speed_multiplier_at(a, b), 1.0);
        g.place_segment(a, b, RoadKind::Road);
        assert_eq!(
            g.speed_multiplier_at(a, b),
            RoadKind::Road.speed_multiplier()
        );
    }
}
