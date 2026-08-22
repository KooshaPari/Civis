//! Road network with Dijkstra shortest-path and capacity modeling.
//!
//! FR-CIV-INFRA-ROADS models a directed graph of intersections (nodes)
//! connected by road segments (edges). Each segment carries a *travel-cost*
//! (travel time) and a *capacity* (vehicles/tick). When volume exceeds
//! capacity the segment is congested and its effective cost rises, creating
//! emergent traffic patterns. Dijkstra's algorithm finds the cheapest route
//! between any two intersections given the current congestion state.
//!
//! This is the **pure-logic core** of the road-network system. It has no
//! Bevy / async dependency and can be driven from any caller that can call
//! [`RoadNetwork::route`] (sim scheduler, replay scrubber, scenario loader,
//! test harness). Renderers and gameplay code read the resulting
//! [`RouteResult`] to decide whether to display congestion, reroute agents,
//! or trigger a gridlock event.

#![forbid(unsafe_code)]

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

use serde::{Deserialize, Serialize};

/// Lightweight ordered wrapper around `f32` for use in `BinaryHeap`. Uses
/// `total_cmp` so positive IEEE-754 floats sort correctly.
#[derive(Debug, Clone, Copy, PartialEq)]
struct OrderedF32(f32);

impl Eq for OrderedF32 {}

impl PartialOrd for OrderedF32 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedF32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

/// Stable identifier for an intersection node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IntersectionId(pub u32);

impl IntersectionId {
    /// Convenience constructor.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }
}

impl From<u32> for IntersectionId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Stable identifier for a directed road segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SegmentId(pub u32);

/// Congestion level derived from the volume-to-capacity ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CongestionLevel {
    /// Volume <= 80% of capacity.
    FreeFlow,
    /// Volume between 80% and 100% of capacity.
    Light,
    /// Volume between 100% and 150% of capacity.
    Moderate,
    /// Volume above 150% of capacity.
    Severe,
}

impl CongestionLevel {
    /// Derive congestion from a volume-to-capacity ratio.
    #[must_use]
    pub fn from_ratio(ratio: f32) -> Self {
        if ratio <= 0.8 {
            CongestionLevel::FreeFlow
        } else if ratio <= 1.0 {
            CongestionLevel::Light
        } else if ratio <= 1.5 {
            CongestionLevel::Moderate
        } else {
            CongestionLevel::Severe
        }
    }

    /// Effective-cost multiplier for Dijkstra when this congestion level
    /// applies. Free-flow is 1.0; severe is 3.0x.
    #[must_use]
    pub fn cost_multiplier(self) -> f32 {
        match self {
            CongestionLevel::FreeFlow => 1.0,
            CongestionLevel::Light => 1.3,
            CongestionLevel::Moderate => 2.0,
            CongestionLevel::Severe => 3.0,
        }
    }
}

/// An intersection node in the road network.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Intersection {
    /// World-space position (used for heuristics and rendering).
    pub position: (f32, f32),
    /// Whether the intersection is active (false = blocked / under construction).
    pub active: bool,
}

/// A directed road segment between two intersections.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    /// Tail intersection.
    pub from: IntersectionId,
    /// Head intersection.
    pub to: IntersectionId,
    /// Base travel cost (ticks or distance units).
    pub base_cost: f32,
    /// Maximum vehicles per tick the segment can handle.
    pub capacity: f32,
}

/// Outcome of [`RoadNetwork::route`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RouteResult {
    /// Ordered intersection IDs from origin to destination (inclusive).
    pub path: Vec<IntersectionId>,
    /// Ordered segment IDs along the path.
    pub segments: Vec<SegmentId>,
    /// Total effective travel cost (sum of segment costs after congestion
    /// multipliers).
    pub total_cost: f32,
    /// Per-segment congestion levels along the route.
    pub congestion: Vec<CongestionLevel>,
}

impl RouteResult {
    /// `true` when a valid path was found (path length >= 2).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.path.len() >= 2
    }
}

/// Per-segment state used internally during volume distribution.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SegmentState {
    /// Current vehicle count on the segment.
    pub volume: f32,
    /// Derived congestion level.
    pub congestion: CongestionLevel,
    /// Effective cost = base_cost * congestion_multiplier.
    pub effective_cost: f32,
}

/// Directed road-network graph. Pure data; the primary behaviours are
/// [`RoadNetwork::distribute_volume`] and [`RoadNetwork::route`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadNetwork {
    intersections: BTreeMap<IntersectionId, Intersection>,
    /// Outgoing segments keyed by tail intersection.
    outgoing: BTreeMap<IntersectionId, Vec<(SegmentId, Segment)>>,
    /// All segments indexed by id for O(1) lookup.
    segment_index: BTreeMap<SegmentId, Segment>,
    /// Per-segment runtime state.
    segment_state: BTreeMap<SegmentId, SegmentState>,
    next_intersection_id: u32,
    next_segment_id: u32,
}

impl Default for RoadNetwork {
    fn default() -> Self {
        Self::new()
    }
}

impl RoadNetwork {
    /// Empty network.
    #[must_use]
    pub fn new() -> Self {
        Self {
            intersections: BTreeMap::new(),
            outgoing: BTreeMap::new(),
            segment_index: BTreeMap::new(),
            segment_state: BTreeMap::new(),
            next_intersection_id: 0,
            next_segment_id: 0,
        }
    }

    /// Register a new intersection. Returns its auto-assigned id.
    pub fn add_intersection(&mut self, position: (f32, f32)) -> IntersectionId {
        let id = IntersectionId(self.next_intersection_id);
        self.next_intersection_id = self.next_intersection_id.saturating_add(1);
        self.intersections.insert(
            id,
            Intersection {
                position,
                active: true,
            },
        );
        id
    }

    /// Register an existing intersection by explicit id (for replay / save
    /// restoration). Replaces the existing entry silently.
    pub fn add_intersection_with_id(&mut self, id: IntersectionId, intersection: Intersection) {
        self.next_intersection_id = self.next_intersection_id.max(id.0 + 1);
        self.intersections.insert(id, intersection);
    }

    /// Add a directed segment from `from` to `to`. Returns its
    /// auto-assigned [`SegmentId`].
    pub fn add_segment(
        &mut self,
        from: IntersectionId,
        to: IntersectionId,
        base_cost: f32,
        capacity: f32,
    ) -> SegmentId {
        let id = SegmentId(self.next_segment_id);
        self.next_segment_id = self.next_segment_id.saturating_add(1);

        let segment = Segment {
            from,
            to,
            base_cost,
            capacity,
        };
        self.outgoing.entry(from).or_default().push((id, segment));
        self.segment_index.insert(id, segment);
        self.segment_state.insert(
            id,
            SegmentState {
                volume: 0.0,
                congestion: CongestionLevel::FreeFlow,
                effective_cost: base_cost,
            },
        );
        id
    }

    /// Number of intersections.
    #[must_use]
    pub fn intersection_count(&self) -> usize {
        self.intersections.len()
    }

    /// Number of segments.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.segment_index.len()
    }

    /// Read-only access to the segment state map.
    #[must_use]
    pub fn segment_states(&self) -> &BTreeMap<SegmentId, SegmentState> {
        &self.segment_state
    }

    /// Read-only access to the intersections map.
    #[must_use]
    pub fn intersections(&self) -> &BTreeMap<IntersectionId, Intersection> {
        &self.intersections
    }

    /// Reset all segment volumes to zero and recompute congestion.
    pub fn reset_volumes(&mut self) {
        for (id, state) in &mut self.segment_state {
            state.volume = 0.0;
            state.congestion = CongestionLevel::FreeFlow;
            if let Some(seg) = self.segment_index.get(id) {
                state.effective_cost = seg.base_cost;
            }
        }
    }

    /// Set the volume on a specific segment (useful for manual placement).
    pub fn set_volume(&mut self, segment_id: SegmentId, volume: f32) {
        if let (Some(state), Some(seg)) = (
            self.segment_state.get_mut(&segment_id),
            self.segment_index.get(&segment_id),
        ) {
            state.volume = volume.max(0.0);
            let ratio = if seg.capacity > 0.0 {
                volume / seg.capacity
            } else {
                f32::INFINITY
            };
            state.congestion = CongestionLevel::from_ratio(ratio);
            state.effective_cost = seg.base_cost * state.congestion.cost_multiplier();
        }
    }

    /// Bulk-apply per-segment volumes and recompute all congestion states.
    /// Segments not present in `volumes` are reset to zero.
    pub fn distribute_volume(&mut self, volumes: &BTreeMap<SegmentId, f32>) {
        for (id, state) in &mut self.segment_state {
            let vol = volumes.get(id).copied().unwrap_or(0.0).max(0.0);
            state.volume = vol;
            if let Some(seg) = self.segment_index.get(id) {
                let ratio = if seg.capacity > 0.0 {
                    vol / seg.capacity
                } else {
                    f32::INFINITY
                };
                state.congestion = CongestionLevel::from_ratio(ratio);
                state.effective_cost = seg.base_cost * state.congestion.cost_multiplier();
            }
        }
    }

    /// Find the cheapest route from `origin` to `destination` using Dijkstra
    /// on the current effective-cost graph. Returns `None` if the destination
    /// is unreachable or either endpoint is unknown / inactive.
    #[must_use]
    pub fn route(
        &self,
        origin: IntersectionId,
        destination: IntersectionId,
    ) -> Option<RouteResult> {
        if origin == destination {
            return Some(RouteResult {
                path: vec![origin],
                segments: vec![],
                total_cost: 0.0,
                congestion: vec![],
            });
        }

        let orig = self.intersections.get(&origin)?;
        let dest = self.intersections.get(&destination)?;
        if !orig.active || !dest.active {
            return None;
        }

        let mut dist: BTreeMap<IntersectionId, f32> = BTreeMap::new();
        let mut parent: BTreeMap<IntersectionId, (SegmentId, IntersectionId)> = BTreeMap::new();
        let mut heap: BinaryHeap<Reverse<(OrderedF32, IntersectionId)>> = BinaryHeap::new();

        dist.insert(origin, 0.0);
        heap.push(Reverse((OrderedF32(0.0), origin)));

        while let Some(Reverse((OrderedF32(cost), node))) = heap.pop() {
            if node == destination {
                break;
            }
            if cost > *dist.get(&node).unwrap_or(&f32::INFINITY) {
                continue;
            }

            let mut edges: Vec<(SegmentId, Segment)> = self
                .outgoing
                .get(&node)
                .map(|v| v.iter().copied().collect())
                .unwrap_or_default();
            edges.sort_by_key(|(id, _)| *id);

            for (seg_id, seg) in edges {
                let to_node = seg.to;
                let to_intersection = self.intersections.get(&to_node);
                if to_intersection.map_or(true, |i| !i.active) {
                    continue;
                }
                let effective = self
                    .segment_state
                    .get(&seg_id)
                    .map_or(seg.base_cost, |s| s.effective_cost);
                let new_cost = cost + effective;

                if new_cost < *dist.get(&to_node).unwrap_or(&f32::INFINITY) {
                    dist.insert(to_node, new_cost);
                    parent.insert(to_node, (seg_id, node));
                    heap.push(Reverse((OrderedF32(new_cost), to_node)));
                }
            }
        }

        let mut path = vec![destination];
        let mut segments = vec![];
        let mut congestion = vec![];
        let mut cur = destination;
        while cur != origin {
            let (seg_id, prev) = parent.get(&cur)?;
            segments.push(*seg_id);
            congestion.push(
                self.segment_state
                    .get(seg_id)
                    .map_or(CongestionLevel::FreeFlow, |s| s.congestion),
            );
            cur = *prev;
            path.push(cur);
        }
        path.reverse();
        segments.reverse();
        congestion.reverse();

        let total_cost = *dist.get(&destination)?;

        Some(RouteResult {
            path,
            segments,
            total_cost,
            congestion,
        })
    }

    /// Total volume across all segments (diagnostic / HUD metric).
    #[must_use]
    pub fn total_volume(&self) -> f32 {
        self.segment_state.values().map(|s| s.volume).sum()
    }

    /// Number of segments currently in Moderate or Severe congestion.
    #[must_use]
    pub fn congested_segment_count(&self) -> usize {
        self.segment_state
            .values()
            .filter(|s| s.congestion >= CongestionLevel::Moderate)
            .count()
    }
}

// ── Road degradation ──────────────────────────────────────────────

/// Weather or environmental event that degrades road surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WeatherEvent {
    /// Heavy rain increases wear and reduces effective capacity.
    HeavyRain,
    /// Freezing temperatures cause frost damage.
    Frost,
    /// Snow accumulation blocks lanes.
    Snow,
    /// Extreme heat softens asphalt.
    Heatwave,
    /// Severe storm causing physical damage.
    SevereStorm,
}

impl WeatherEvent {
    /// Capacity reduction factor (0.0–1.0) applied by this event.
    #[must_use]
    pub fn capacity_factor(self) -> f32 {
        match self {
            WeatherEvent::HeavyRain => 0.85,
            WeatherEvent::Frost => 0.70,
            WeatherEvent::Snow => 0.50,
            WeatherEvent::Heatwave => 0.80,
            WeatherEvent::SevereStorm => 0.40,
        }
    }

    /// Base cost multiplier caused by this event.
    #[must_use]
    pub fn cost_multiplier(self) -> f32 {
        match self {
            WeatherEvent::HeavyRain => 1.2,
            WeatherEvent::Frost => 1.5,
            WeatherEvent::Snow => 2.0,
            WeatherEvent::Heatwave => 1.1,
            WeatherEvent::SevereStorm => 2.5,
        }
    }
}

/// Severity of road surface degradation (0.0 = pristine, 1.0 = impassable).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DegradationLevel(pub f32);

impl DegradationLevel {
    /// Create a new degradation level, clamped to [0.0, 1.0].
    #[must_use]
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Whether the road is impassable.
    #[must_use]
    pub fn is_impassable(self) -> bool {
        self.0 >= 0.95
    }

    /// Effective capacity factor after degradation (1.0 = pristine, 0.0 = none).
    #[must_use]
    pub fn capacity_factor(self) -> f32 {
        (1.0 - self.0).max(0.0)
    }

    /// Cost multiplier from degradation alone.
    #[must_use]
    pub fn cost_multiplier(self) -> f32 {
        1.0 + self.0 * 4.0 // up to 5x at max degradation
    }
}

/// Result of applying degradation to the road network.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DegradationReport {
    /// Per-segment degradation levels after the event.
    pub segment_degradation: BTreeMap<SegmentId, DegradationLevel>,
    /// Segments that became impassable.
    pub impassable_segments: Vec<SegmentId>,
    /// Total cost increase across all segments (estimated).
    pub total_extra_cost: f32,
}

impl RoadNetwork {
    /// Apply a weather event to all segments, adjusting effective capacity
    /// and cost. Returns a report of the degradation applied.
    pub fn apply_weather_event(&mut self, event: WeatherEvent) -> DegradationReport {
        let mut report = DegradationReport::default();
        let event_factor = event.capacity_factor();
        let event_cost = event.cost_multiplier();

        for (&seg_id, segment) in &self.segment_index {
            let effective_capacity = segment.capacity * event_factor;
            let degradation = DegradationLevel::new(1.0 - event_factor);
            report.segment_degradation.insert(seg_id, degradation);

            if let Some(state) = self.segment_state.get_mut(&seg_id) {
                let ratio = if effective_capacity > 0.0 {
                    state.volume / effective_capacity
                } else {
                    f32::INFINITY
                };
                state.congestion = CongestionLevel::from_ratio(ratio);
                state.effective_cost =
                    segment.base_cost * state.congestion.cost_multiplier() * event_cost;
                report.total_extra_cost += state.effective_cost - segment.base_cost;

                if degradation.is_impassable() {
                    report.impassable_segments.push(seg_id);
                }
            }
        }
        report
    }

    /// Apply cumulative wear to a specific segment. The `wear` value
    /// (0.0–1.0) is added to the existing degradation state. Returns the
    /// new degradation level, or `None` if the segment is unknown.
    pub fn apply_wear(&mut self, segment_id: SegmentId, wear: f32) -> Option<DegradationLevel> {
        let segment = self.segment_index.get(&segment_id)?;
        let current_degradation = self
            .segment_state
            .get(&segment_id)
            .map(|s| {
                // Infer degradation from effective_cost vs base_cost
                if segment.base_cost > 0.0 {
                    (s.effective_cost / segment.base_cost - 1.0) / 4.0
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);

        let new_level = DegradationLevel::new(current_degradation + wear);
        let factor = new_level.capacity_factor();
        let cost_mult = new_level.cost_multiplier();

        if let Some(state) = self.segment_state.get_mut(&segment_id) {
            let effective_capacity = segment.capacity * factor;
            let ratio = if effective_capacity > 0.0 {
                state.volume / effective_capacity
            } else {
                f32::INFINITY
            };
            state.congestion = CongestionLevel::from_ratio(ratio);
            state.effective_cost = segment.base_cost * cost_mult;
        }
        Some(new_level)
    }

    /// Repair a segment back to pristine condition.
    pub fn repair_segment(&mut self, segment_id: SegmentId) {
        if let (Some(state), Some(seg)) = (
            self.segment_state.get_mut(&segment_id),
            self.segment_index.get(&segment_id),
        ) {
            let ratio = if seg.capacity > 0.0 {
                state.volume / seg.capacity
            } else {
                f32::INFINITY
            };
            state.congestion = CongestionLevel::from_ratio(ratio);
            state.effective_cost = seg.base_cost * state.congestion.cost_multiplier();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_triangle_network() -> RoadNetwork {
        let mut net = RoadNetwork::new();
        let _ = net.add_intersection_with_id(
            IntersectionId(0),
            Intersection {
                position: (0.0, 0.0),
                active: true,
            },
        );
        let _ = net.add_intersection_with_id(
            IntersectionId(1),
            Intersection {
                position: (1.0, 0.0),
                active: true,
            },
        );
        let _ = net.add_intersection_with_id(
            IntersectionId(2),
            Intersection {
                position: (1.0, 1.0),
                active: true,
            },
        );
        // 0 -> 1: cost 4, cap 10
        net.add_segment(IntersectionId(0), IntersectionId(1), 4.0, 10.0);
        // 1 -> 2: cost 3, cap 10
        net.add_segment(IntersectionId(1), IntersectionId(2), 3.0, 10.0);
        // 0 -> 2: cost 10, cap 10
        net.add_segment(IntersectionId(0), IntersectionId(2), 10.0, 10.0);
        net
    }

    #[test]
    fn dijkstra_finds_shortest_cost_path() {
        let net = build_triangle_network();
        let result = net
            .route(IntersectionId(0), IntersectionId(2))
            .expect("route should exist");
        assert!(result.is_valid());
        assert_eq!(
            result.path,
            vec![IntersectionId(0), IntersectionId(1), IntersectionId(2)]
        );
        assert!((result.total_cost - 7.0).abs() < f32::EPSILON);
        assert_eq!(result.segments.len(), 2);
    }

    #[test]
    fn routing_to_same_intersection_returns_identity() {
        let net = build_triangle_network();
        let result = net
            .route(IntersectionId(0), IntersectionId(0))
            .expect("identity route ok");
        assert_eq!(result.path, vec![IntersectionId(0)]);
        assert!((result.total_cost - 0.0).abs() < f32::EPSILON);
        assert!(result.segments.is_empty());
    }

    #[test]
    fn unreachable_destination_returns_none() {
        let mut net = RoadNetwork::new();
        let _ = net.add_intersection_with_id(
            IntersectionId(0),
            Intersection {
                position: (0.0, 0.0),
                active: true,
            },
        );
        let _ = net.add_intersection_with_id(
            IntersectionId(1),
            Intersection {
                position: (5.0, 5.0),
                active: true,
            },
        );
        assert!(net.route(IntersectionId(0), IntersectionId(1)).is_none());
    }

    #[test]
    fn inactive_intersection_blocks_routing() {
        let mut net = build_triangle_network();
        let i1 = net.intersections.get_mut(&IntersectionId(1)).unwrap();
        i1.active = false;
        let result = net
            .route(IntersectionId(0), IntersectionId(2))
            .expect("direct route ok");
        assert_eq!(result.path, vec![IntersectionId(0), IntersectionId(2)]);
        assert!((result.total_cost - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn congestion_reroutes_away_from_saturated_segment() {
        let mut net = build_triangle_network();
        let segs: Vec<SegmentId> = net.segment_index.keys().copied().collect();
        // segs[0] = 0 -> 1 (cost 4, cap 10) -> set 3x over cap
        net.set_volume(segs[0], 30.0);

        let result = net
            .route(IntersectionId(0), IntersectionId(2))
            .expect("route ok");
        assert_eq!(
            result.path,
            vec![IntersectionId(0), IntersectionId(2)],
            "should reroute around congested segment"
        );
        assert!((result.total_cost - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn distribute_volume_updates_all_segment_states() {
        let mut net = build_triangle_network();
        let segs: Vec<SegmentId> = net.segment_index.keys().copied().collect();
        let mut volumes = BTreeMap::new();
        volumes.insert(segs[0], 5.0); // 5/10 = 0.5 -> FreeFlow
        volumes.insert(segs[1], 9.0); // 9/10 = 0.9 -> Light
        volumes.insert(segs[2], 16.0); // 16/10 = 1.6 -> Severe
        net.distribute_volume(&volumes);

        let states = net.segment_states();
        assert_eq!(
            states.get(&segs[0]).unwrap().congestion,
            CongestionLevel::FreeFlow
        );
        assert_eq!(
            states.get(&segs[1]).unwrap().congestion,
            CongestionLevel::Light
        );
        assert_eq!(
            states.get(&segs[2]).unwrap().congestion,
            CongestionLevel::Severe
        );
    }

    #[test]
    fn congestion_level_from_ratio_boundaries() {
        assert_eq!(CongestionLevel::from_ratio(0.0), CongestionLevel::FreeFlow);
        assert_eq!(CongestionLevel::from_ratio(0.8), CongestionLevel::FreeFlow);
        assert_eq!(CongestionLevel::from_ratio(0.81), CongestionLevel::Light);
        assert_eq!(CongestionLevel::from_ratio(1.0), CongestionLevel::Light);
        assert_eq!(CongestionLevel::from_ratio(1.01), CongestionLevel::Moderate);
        assert_eq!(CongestionLevel::from_ratio(1.5), CongestionLevel::Moderate);
        assert_eq!(CongestionLevel::from_ratio(1.51), CongestionLevel::Severe);
    }

    #[test]
    fn total_volume_and_congested_count_aggregate_correctly() {
        let mut net = build_triangle_network();
        assert!((net.total_volume()).abs() < f32::EPSILON);
        assert_eq!(net.congested_segment_count(), 0);

        let segs: Vec<SegmentId> = net.segment_index.keys().copied().collect();
        net.set_volume(segs[0], 12.0); // Moderate
        net.set_volume(segs[1], 20.0); // Severe

        assert!((net.total_volume() - 32.0).abs() < f32::EPSILON);
        assert_eq!(net.congested_segment_count(), 2);
    }

    // ── Degradation tests ──────────────────────────────────────────

    #[test]
    fn weather_event_capacity_factor() {
        assert_eq!(WeatherEvent::HeavyRain.capacity_factor(), 0.85);
        assert_eq!(WeatherEvent::Snow.capacity_factor(), 0.50);
        assert_eq!(WeatherEvent::SevereStorm.capacity_factor(), 0.40);
    }

    #[test]
    fn weather_event_cost_multiplier() {
        assert_eq!(WeatherEvent::Heatwave.cost_multiplier(), 1.1);
        assert_eq!(WeatherEvent::SevereStorm.cost_multiplier(), 2.5);
    }

    #[test]
    fn degradation_level_clamping() {
        let d = DegradationLevel::new(1.5);
        assert_eq!(d.0, 1.0);
        let d2 = DegradationLevel::new(-0.3);
        assert_eq!(d2.0, 0.0);
    }

    #[test]
    fn degradation_impassable_at_095() {
        assert!(!DegradationLevel::new(0.9).is_impassable());
        assert!(DegradationLevel::new(0.95).is_impassable());
        assert!(DegradationLevel::new(1.0).is_impassable());
    }

    #[test]
    fn apply_weather_event_increases_cost() {
        let mut net = build_triangle_network();
        let segs: Vec<SegmentId> = net.segment_index.keys().copied().collect();
        net.set_volume(segs[0], 5.0);

        let report = net.apply_weather_event(WeatherEvent::Snow);
        assert_eq!(report.segment_degradation.len(), 3);
        assert!(report.total_extra_cost > 0.0);
    }

    #[test]
    fn apply_wear_increases_degradation() {
        let mut net = build_triangle_network();
        let segs: Vec<SegmentId> = net.segment_index.keys().copied().collect();
        let level = net.apply_wear(segs[0], 0.3);
        assert!(level.is_some());
        assert!(level.unwrap().0 > 0.0);
    }

    #[test]
    fn repair_segment_restores_cost() {
        let mut net = build_triangle_network();
        let segs: Vec<SegmentId> = net.segment_index.keys().copied().collect();
        let original_cost = net.segment_index.get(&segs[0]).unwrap().base_cost;
        net.apply_wear(segs[0], 0.8);
        net.repair_segment(segs[0]);
        let state = net.segment_state.get(&segs[0]).unwrap();
        assert!(state.effective_cost <= original_cost * 1.1);
    }
}
