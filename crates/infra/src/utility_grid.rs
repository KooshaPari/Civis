//! Utility grid for FR-CIV-UTILITY-GRID.
//!
//! Models a directed network of **sources** (producers), **sinks**
//! (consumers) and optional **relays** (pure junctions). Each tick the grid
//! distributes available supply through edges that are constrained by
//! `capacity` (units / tick). Sinks that cannot be satisfied report a
//! `shortfall` — the canonical signal for "downstream blackout", "starving
//! population", or "dry farmland" events in the wider sim.
//!
//! This is the **pure-logic core** of the utility-grid feedback loop. It is
//! deliberately storage-agnostic and has no Bevy / DB / async dependency: it
//! can be driven from any caller that can call
//! [`UtilityGrid::distribute`] (sim scheduler, replay scrubber, scenario
//! loader, test harness). Renderers and gameplay code read the resulting
//! [`UtilityGridResult`] to decide whether to dim a district, fire a
//! notification, or trigger an emergency workload.
//!
//! ## Algorithm
//!
//! 1. The grid is built once with [`UtilityGrid::with_nodes`] then
//!    [`UtilityGrid::add_edge`] calls connecting sources -> relays -> sinks.
//! 2. Each tick the caller invokes
//!    [`UtilityGrid::distribute`] with a per-source `supply` map and a
//!    per-sink `demand` map (units requested this tick).
//! 3. [`UtilityGrid::distribute`] performs an iterative proportional push:
//!    for each round it routes flow from sources to sinks along the shortest
//!    reachable path in hops, saturating the minimum of (remaining source
//!    supply, remaining edge capacity, remaining sink demand). When a sink
//!    can no longer be reached at full demand the remainder is reported as
//!    `shortfall`.
//! 4. The result is fully deterministic for a fixed `(grid topology, supply,
//!    demand)` triple and uses sorted ordering everywhere so equality is
//!    reproducible across runs.
//!
//! See `docs/specs/requirements/FR-CIV-UTILITY-GRID.md` for the full
//! requirement.
//!
//! ## Determinism
//!
//! All iteration uses `BTreeMap` / `BTreeSet` and a deterministic
//! path-search tie-break so the same input always yields the same result.
//! This lets the acceptance test pin exact numerical expectations.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

/// Stable identifier for a node in the utility grid. Newtype so callers
/// can't accidentally pass a sink index where a source index was meant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UtilityNodeId(pub u32);

impl UtilityNodeId {
    /// Convenience constructor.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }
}

impl From<u32> for UtilityNodeId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Stable identifier for a directed edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UtilityEdgeId(pub u32);

impl UtilityEdgeId {
    /// Convenience constructor.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }
}

/// Role a node plays in the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityNodeKind {
    /// Producer — emits up to its `max_supply` per tick.
    Source,
    /// Junction — neither produces nor consumes, just routes flow.
    Relay,
    /// Consumer — accepts up to its `max_demand` per tick.
    Sink,
}

/// Per-node bookkeeping.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UtilityNode {
    /// Role of the node.
    pub kind: UtilityNodeKind,
    /// For sources: maximum units produced per tick (acts as a per-tick
    /// supply ceiling on top of the caller's `supply` argument).
    /// Ignored for other node kinds.
    pub max_supply: f32,
    /// For sinks: maximum units accepted per tick (acts as a per-tick
    /// demand ceiling on top of the caller's `demand` argument).
    /// Ignored for other node kinds.
    pub max_demand: f32,
}

impl UtilityNode {
    /// Source node with the given supply ceiling.
    #[must_use]
    pub const fn source(max_supply: f32) -> Self {
        Self {
            kind: UtilityNodeKind::Source,
            max_supply,
            max_demand: 0.0,
        }
    }

    /// Pure relay node (no production, no consumption).
    #[must_use]
    pub const fn relay() -> Self {
        Self {
            kind: UtilityNodeKind::Relay,
            max_supply: 0.0,
            max_demand: 0.0,
        }
    }

    /// Sink node with the given demand ceiling.
    #[must_use]
    pub const fn sink(max_demand: f32) -> Self {
        Self {
            kind: UtilityNodeKind::Sink,
            max_supply: 0.0,
            max_demand,
        }
    }
}

/// Directed edge between two utility nodes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UtilityEdge {
    /// Tail node (flow origin).
    pub from: UtilityNodeId,
    /// Head node (flow destination).
    pub to: UtilityNodeId,
    /// Maximum units per tick the edge can carry. Flow above this is
    /// silently clamped; downstream sinks will then shortfall.
    pub capacity: f32,
}

/// Outcome of a single [`UtilityGrid::distribute`] call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UtilityGridResult {
    /// Units delivered to each sink node (sources / relays map to `0.0`).
    pub delivered: BTreeMap<UtilityNodeId, f32>,
    /// Demand each sink node could *not* satisfy this tick (`> 0.0` means
    /// downstream blackout / starvation).
    pub shortfall: BTreeMap<UtilityNodeId, f32>,
    /// Units that flowed along each edge this tick.
    pub edge_flow: BTreeMap<UtilityEdgeId, f32>,
}

impl UtilityGridResult {
    /// Total shortfall across every sink (diagnostic / city-wide blackout
    /// event trigger). Returns `0.0` if every demand was satisfied.
    #[must_use]
    pub fn total_shortfall(&self) -> f32 {
        self.shortfall.values().copied().sum()
    }

    /// Total units delivered to all sinks (diagnostic / throughput HUD).
    #[must_use]
    pub fn total_delivered(&self) -> f32 {
        self.delivered.values().copied().sum()
    }

    /// `true` when every sink that asked for flow got its full demand.
    #[must_use]
    pub fn all_sinks_satisfied(&self) -> bool {
        self.shortfall.values().all(|v| *v <= 0.0)
    }
}

/// Directed utility graph (sources -> relays -> sinks). Pure data; the only
/// behaviour is [`UtilityGrid::distribute`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UtilityGrid {
    nodes: BTreeMap<UtilityNodeId, UtilityNode>,
    /// Outgoing edges keyed by tail node (deterministic BTreeMap ordering
    /// keeps the BFS tie-break reproducible).
    outgoing: BTreeMap<UtilityNodeId, Vec<(UtilityEdgeId, UtilityEdge)>>,
    /// Reverse lookup so we can resolve `edge_flow[edge_id]` writes.
    edge_index: BTreeMap<UtilityEdgeId, UtilityEdge>,
    next_edge_id: u32,
}

impl UtilityGrid {
    /// Empty grid.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a node. If `id` already exists, the existing entry is
    /// **replaced** (intentional — lets scenarios patch in a new role for
    /// the same node without rebuilding the graph).
    pub fn add_node(&mut self, id: UtilityNodeId, node: UtilityNode) {
        self.nodes.insert(id, node);
    }

    /// Convenience: register a source node.
    pub fn add_source(&mut self, id: UtilityNodeId, max_supply: f32) {
        self.add_node(id, UtilityNode::source(max_supply));
    }

    /// Convenience: register a relay node.
    pub fn add_relay(&mut self, id: UtilityNodeId) {
        self.add_node(id, UtilityNode::relay());
    }

    /// Convenience: register a sink node.
    pub fn add_sink(&mut self, id: UtilityNodeId, max_demand: f32) {
        self.add_node(id, UtilityNode::sink(max_demand));
    }

    /// Number of registered nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of registered edges.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edge_index.len()
    }

    /// Read-only access to the registered nodes (diagnostics / debug).
    #[must_use]
    pub fn nodes(&self) -> &BTreeMap<UtilityNodeId, UtilityNode> {
        &self.nodes
    }

    /// Add a directed edge from `from` to `to` with the given `capacity`.
    /// Auto-assigns a stable [`UtilityEdgeId`]. Returns the assigned id so
    /// callers can later read `edge_flow` for that edge.
    ///
    /// Self-loops are silently rejected (they can never contribute useful
    /// flow and would only confuse downstream metrics).
    pub fn add_edge(
        &mut self,
        from: UtilityNodeId,
        to: UtilityNodeId,
        capacity: f32,
    ) -> UtilityEdgeId {
        if from == to {
            // Skip: pick the next id but don't record an edge.
            let id = UtilityEdgeId(self.next_edge_id);
            self.next_edge_id = self.next_edge_id.saturating_add(1);
            return id;
        }
        let id = UtilityEdgeId(self.next_edge_id);
        self.next_edge_id = self.next_edge_id.saturating_add(1);
        let edge = UtilityEdge {
            from,
            to,
            capacity,
        };
        self.outgoing.entry(from).or_default().push((id, edge));
        self.edge_index.insert(id, edge);
        id
    }

    /// Push available supply from sources down to sinks. `supply` is the
    /// raw units produced at each source node this tick (clamped to the
    /// node's `max_supply`); `demand` is the raw units requested at each
    /// sink (clamped to `max_demand`). Negative entries are treated as
    /// zero.
    ///
    /// The algorithm is an iterative proportional push:
    ///
    /// 1. Initialise each sink's residual demand, each source's residual
    ///    supply, and every edge's residual capacity.
    /// 2. For up to `grid.node_count()` rounds:
    ///    * For every (source, sink) pair whose sink still has unmet
    ///      demand, run a deterministic BFS to find the **shortest** path
    ///      in hops (ties broken by sorted node ids).
    ///    * If a path exists, push `min(source_supply, edge_min_capacity,
    ///      sink_demand)` along every edge of that path, decrementing the
    ///      residuals.
    ///    * If the source is empty or no sink can reach it, retire it.
    /// 3. Anything left in `sink_demand` after the loop is `shortfall`.
    ///
    /// Complexity is `O(nodes * (nodes + edges))` which is plenty for the
    /// city-scale graphs (hundreds of nodes) the FR targets.
    #[must_use]
    pub fn distribute(
        &self,
        supply: &BTreeMap<UtilityNodeId, f32>,
        demand: &BTreeMap<UtilityNodeId, f32>,
    ) -> UtilityGridResult {
        // --- Clamp residual supply / demand to per-node ceilings.
        let mut residual_supply: BTreeMap<UtilityNodeId, f32> = BTreeMap::new();
        for (&id, node) in &self.nodes {
            if matches!(node.kind, UtilityNodeKind::Source) {
                let raw = supply.get(&id).copied().unwrap_or(0.0).max(0.0);
                residual_supply.insert(id, raw.min(node.max_supply.max(0.0)));
            }
        }

        let mut residual_demand: BTreeMap<UtilityNodeId, f32> = BTreeMap::new();
        let mut original_demand: BTreeMap<UtilityNodeId, f32> = BTreeMap::new();
        for (&id, node) in &self.nodes {
            if matches!(node.kind, UtilityNodeKind::Sink) {
                let raw = demand.get(&id).copied().unwrap_or(0.0).max(0.0);
                let clamped = raw.min(node.max_demand.max(0.0));
                original_demand.insert(id, clamped);
                residual_demand.insert(id, clamped);
            }
        }

        // --- Residual edge capacity and edge-flow accounting.
        let mut residual_edge: BTreeMap<UtilityEdgeId, f32> = self
            .edge_index
            .iter()
            .map(|(id, e)| (*id, e.capacity.max(0.0)))
            .collect();
        let mut edge_flow: BTreeMap<UtilityEdgeId, f32> =
            self.edge_index.keys().map(|id| (*id, 0.0)).collect();

        // --- Ordered source / sink lists so iteration is deterministic.
        let sources: Vec<UtilityNodeId> = residual_supply.keys().copied().collect();
        let sinks: Vec<UtilityNodeId> = residual_demand.keys().copied().collect();

        // Cap iterations so a pathological graph can't spin forever.
        let max_rounds = self.nodes.len().max(1);

        for _ in 0..max_rounds {
            let mut progress = false;

            for &src in &sources {
                // If the source has no remaining supply, nothing to do.
                let src_supply = residual_supply.get(&src).copied().unwrap_or(0.0);
                if src_supply <= 0.0 {
                    continue;
                }

                // Find the cheapest reachable sink (by hops, ties broken by
                // sink id ascending so BFS picks a stable target).
                let Some((sink, path)) = self.shortest_path_from(src, &residual_demand)
                else {
                    continue;
                };

                // Find the bottleneck along the path.
                let mut bottleneck = src_supply;
                bottleneck = bottleneck.min(
                    residual_demand
                        .get(&sink)
                        .copied()
                        .unwrap_or(0.0)
                        .max(0.0),
                );
                for edge_id in path.iter().copied() {
                    bottleneck = bottleneck.min(
                        residual_edge
                            .get(&edge_id)
                            .copied()
                            .unwrap_or(0.0)
                            .max(0.0),
                    );
                }
                if bottleneck <= 0.0 {
                    // No edge has capacity -> retire this source for the
                    // rest of the tick (any remaining edge from it is
                    // saturated and any remaining demand is unreachable).
                    residual_supply.insert(src, 0.0);
                    continue;
                }

                // Push the bottleneck along the path.
                residual_supply.insert(src, src_supply - bottleneck);
                let prev_demand = residual_demand
                    .get(&sink)
                    .copied()
                    .unwrap_or(0.0);
                residual_demand.insert(sink, (prev_demand - bottleneck).max(0.0));
                for edge_id in path.iter().copied() {
                    let prev_flow = edge_flow.get(&edge_id).copied().unwrap_or(0.0);
                    edge_flow.insert(edge_id, prev_flow + bottleneck);
                    let prev_cap = residual_edge.get(&edge_id).copied().unwrap_or(0.0);
                    residual_edge.insert(edge_id, (prev_cap - bottleneck).max(0.0));
                }
                progress = true;
            }

            // If no progress was made this round (every reachable source
            // either drained or saturated every outgoing edge) bail out
            // early — the rest of the sinks must shortfall.
            if !progress {
                break;
            }
        }

        // --- Build the result.
        let mut delivered: BTreeMap<UtilityNodeId, f32> = BTreeMap::new();
        let mut shortfall: BTreeMap<UtilityNodeId, f32> = BTreeMap::new();
        for &sink in &sinks {
            let original = original_demand.get(&sink).copied().unwrap_or(0.0);
            let remaining = residual_demand.get(&sink).copied().unwrap_or(0.0);
            let delivered_amount = (original - remaining).max(0.0);
            delivered.insert(sink, delivered_amount);
            shortfall.insert(sink, remaining);
        }
        // Make sure sources / relays appear in `delivered` so callers can
        // unconditionally read the map.
        for &id in self.nodes.keys() {
            delivered.entry(id).or_insert(0.0);
        }

        UtilityGridResult {
            delivered,
            shortfall,
            edge_flow,
        }
    }

    /// Deterministic BFS from `start`: returns the shortest hop path
    /// (ties broken by visited-node id ascending) to the lowest-id sink
    /// that still has positive `residual_demand` and is reachable.
    ///
    /// Returns `None` if `start` is not a source, has no outgoing edges,
    /// or no sink is reachable with remaining demand.
    fn shortest_path_from(
        &self,
        start: UtilityNodeId,
        residual_demand: &BTreeMap<UtilityNodeId, f32>,
    ) -> Option<(UtilityNodeId, Vec<UtilityEdgeId>)> {
        if self.outgoing.get(&start).is_none() {
            return None;
        }

        // parent[child] = (edge_id_from_parent, parent_id)
        let mut parent: BTreeMap<UtilityNodeId, (UtilityEdgeId, UtilityNodeId)> =
            BTreeMap::new();
        parent.insert(start, (UtilityEdgeId(u32::MAX), start));

        let mut frontier: VecDeque<UtilityNodeId> = VecDeque::new();
        frontier.push_back(start);

        // Sorted sinks with positive residual demand -> the BFS naturally
        // explores nodes in ascending order so the first sink we pop wins.
        let target_sinks: BTreeSet<UtilityNodeId> = residual_demand
            .iter()
            .filter_map(|(id, d)| if *d > 0.0 { Some(*id) } else { None })
            .collect();

        let mut found: Option<UtilityNodeId> = None;

        while let Some(node) = frontier.pop_front() {
            if target_sinks.contains(&node) {
                found = Some(node);
                break;
            }
            // Explore outgoing edges in deterministic (edge_id, neighbour_id)
            // ascending order so BFS tie-breaks are reproducible.
            let mut outgoing: Vec<(UtilityEdgeId, UtilityNodeId)> = self
                .outgoing
                .get(&node)
                .map(|v| v.iter().map(|(id, e)| (*id, e.to)).collect())
                .unwrap_or_default();
            outgoing.sort_by_key(|(eid, nid)| (*eid, *nid));
            for (edge_id, neighbour) in outgoing {
                if !parent.contains_key(&neighbour) {
                    parent.insert(neighbour, (edge_id, node));
                    frontier.push_back(neighbour);
                }
            }
        }

        let sink = found?;
        // Reconstruct edge-id path back to `start`. Skip the sentinel edge
        // id at `start` itself.
        let mut edges: Vec<UtilityEdgeId> = Vec::new();
        let mut cur = sink;
        while cur != start {
            let (edge_id, prev) = parent.get(&cur).copied()?;
            edges.push(edge_id);
            cur = prev;
        }
        edges.reverse();
        Some((sink, edges))
    }
}

#[cfg(test)]
mod tests {
    //! Acceptance tests for FR-CIV-UTILITY-GRID.
    //!
    //! Reference: FR-CIV-UTILITY-GRID ("A utility grid distributes a
    //! resource from sources to sinks; capacity limits cause shortfalls
    //! downstream.").

    use super::*;

    /// FR-CIV-UTILITY-GRID acceptance: a source feeds connected sinks and
    /// they receive their requested demand (capacity is not exceeded).
    #[test]
    fn source_feeds_connected_sinks_when_capacity_is_sufficient() {
        let mut grid = UtilityGrid::new();
        let src = UtilityNodeId::new(0);
        let sink_a = UtilityNodeId::new(1);
        let sink_b = UtilityNodeId::new(2);
        grid.add_source(src, 100.0);
        grid.add_sink(sink_a, 50.0);
        grid.add_sink(sink_b, 30.0);
        grid.add_edge(src, sink_a, 100.0);
        grid.add_edge(src, sink_b, 100.0);

        let mut supply = BTreeMap::new();
        supply.insert(src, 80.0);
        let mut demand = BTreeMap::new();
        demand.insert(sink_a, 50.0);
        demand.insert(sink_b, 30.0);

        let result = grid.distribute(&supply, &demand);

        assert_eq!(result.delivered.get(&sink_a).copied(), Some(50.0));
        assert_eq!(result.delivered.get(&sink_b).copied(), Some(30.0));
        assert_eq!(result.shortfall.get(&sink_a).copied(), Some(0.0));
        assert_eq!(result.shortfall.get(&sink_b).copied(), Some(0.0));
        assert!(result.all_sinks_satisfied());
        assert_eq!(result.total_shortfall(), 0.0);
    }

    /// FR-CIV-UTILITY-GRID acceptance: over-demand past the edge capacity
    /// causes downstream shortfall. The source can produce 100 but the
    /// only outgoing edge can carry 40 -> sink_a gets 40, sink_b gets 0,
    /// both short by their original demand.
    #[test]
    fn over_demand_past_edge_capacity_creates_downstream_shortfall() {
        let mut grid = UtilityGrid::new();
        let src = UtilityNodeId::new(0);
        let sink_a = UtilityNodeId::new(1);
        let sink_b = UtilityNodeId::new(2);
        grid.add_source(src, 100.0);
        grid.add_sink(sink_a, 60.0);
        grid.add_sink(sink_b, 50.0);
        // Bottleneck: only 40 units/tick can leave the source, but the
        // sinks together want 110.
        grid.add_edge(src, sink_a, 40.0);
        grid.add_edge(src, sink_b, 40.0);

        let mut supply = BTreeMap::new();
        supply.insert(src, 100.0);
        let mut demand = BTreeMap::new();
        demand.insert(sink_a, 60.0);
        demand.insert(sink_b, 50.0);

        let result = grid.distribute(&supply, &demand);

        // Edge caps dominate: each edge can only carry 40.
        let delivered_a = result.delivered.get(&sink_a).copied().unwrap_or(0.0);
        let delivered_b = result.delivered.get(&sink_b).copied().unwrap_or(0.0);
        assert!(
            delivered_a <= 40.0 + f32::EPSILON,
            "sink_a must respect edge capacity (got {delivered_a})"
        );
        assert!(
            delivered_b <= 40.0 + f32::EPSILON,
            "sink_b must respect edge capacity (got {delivered_b})"
        );

        // The downstream shortfall is the canonical FR signal.
        let shortfall_a = result.shortfall.get(&sink_a).copied().unwrap_or(0.0);
        let shortfall_b = result.shortfall.get(&sink_b).copied().unwrap_or(0.0);
        assert!(
            shortfall_a > 0.0,
            "sink_a must be in shortfall when over-demand hits capacity"
        );
        assert!(
            shortfall_b > 0.0,
            "sink_b must be in shortfall when over-demand hits capacity"
        );

        // Conservation: every unit the source pushed reached some sink
        // (no leaks into relays here) and shortfalls add up to the
        // unmet demand.
        let total_unmet = shortfall_a + shortfall_b;
        let total_requested = 60.0 + 50.0;
        let total_delivered = delivered_a + delivered_b;
        assert!(
            (total_delivered + total_unmet - total_requested).abs() < 1e-3,
            "delivered ({total_delivered}) + shortfall ({total_unmet}) must equal requested ({total_requested})"
        );

        assert!(!result.all_sinks_satisfied());
        assert!(result.total_shortfall() > 0.0);
    }

    /// Multi-hop network: source -> relay -> sink. The relay is a pure
    /// junction, so it neither supplies nor consumes; flow reaching the
    /// sink equals what the edge to the sink allows.
    #[test]
    fn relays_route_flow_without_consuming_it() {
        let mut grid = UtilityGrid::new();
        let src = UtilityNodeId::new(0);
        let relay = UtilityNodeId::new(1);
        let sink = UtilityNodeId::new(2);
        grid.add_source(src, 100.0);
        grid.add_relay(relay);
        grid.add_sink(sink, 80.0);
        grid.add_edge(src, relay, 100.0);
        // Edge from relay to sink is the bottleneck at 25.
        grid.add_edge(relay, sink, 25.0);

        let mut supply = BTreeMap::new();
        supply.insert(src, 100.0);
        let mut demand = BTreeMap::new();
        demand.insert(sink, 80.0);

        let result = grid.distribute(&supply, &demand);

        assert_eq!(
            result.delivered.get(&sink).copied(),
            Some(25.0),
            "sink must receive only as much as the bottleneck edge allows"
        );
        assert_eq!(
            result.shortfall.get(&sink).copied(),
            Some(55.0),
            "remaining demand is downstream shortfall"
        );
        // Relay itself never reports a delivery / shortfall.
        assert_eq!(result.delivered.get(&relay).copied(), Some(0.0));
    }

    /// No path -> full shortfall, no spurious flow.
    #[test]
    fn disconnected_sink_reports_full_shortfall() {
        let mut grid = UtilityGrid::new();
        let src = UtilityNodeId::new(0);
        let connected_sink = UtilityNodeId::new(1);
        let isolated_sink = UtilityNodeId::new(2);
        grid.add_source(src, 50.0);
        grid.add_sink(connected_sink, 30.0);
        grid.add_sink(isolated_sink, 20.0);
        // Only one sink is wired up.
        grid.add_edge(src, connected_sink, 50.0);

        let mut supply = BTreeMap::new();
        supply.insert(src, 50.0);
        let mut demand = BTreeMap::new();
        demand.insert(connected_sink, 30.0);
        demand.insert(isolated_sink, 20.0);

        let result = grid.distribute(&supply, &demand);
        assert_eq!(result.delivered.get(&connected_sink).copied(), Some(30.0));
        assert_eq!(result.shortfall.get(&connected_sink).copied(), Some(0.0));
        assert_eq!(result.delivered.get(&isolated_sink).copied(), Some(0.0));
        assert_eq!(result.shortfall.get(&isolated_sink).copied(), Some(20.0));
    }

    /// Determinism: identical inputs yield byte-identical results across
    /// repeated calls (the FR spec demands this for replay equality).
    #[test]
    fn distribution_is_deterministic() {
        let build = || {
            let mut grid = UtilityGrid::new();
            let src1 = UtilityNodeId::new(0);
            let src2 = UtilityNodeId::new(1);
            let relay = UtilityNodeId::new(2);
            let sink_a = UtilityNodeId::new(3);
            let sink_b = UtilityNodeId::new(4);
            grid.add_source(src1, 40.0);
            grid.add_source(src2, 40.0);
            grid.add_relay(relay);
            grid.add_sink(sink_a, 35.0);
            grid.add_sink(sink_b, 35.0);
            grid.add_edge(src1, relay, 30.0);
            grid.add_edge(src2, relay, 30.0);
            grid.add_edge(relay, sink_a, 25.0);
            grid.add_edge(relay, sink_b, 25.0);

            let mut supply = BTreeMap::new();
            supply.insert(src1, 40.0);
            supply.insert(src2, 40.0);
            let mut demand = BTreeMap::new();
            demand.insert(sink_a, 35.0);
            demand.insert(sink_b, 35.0);
            grid.distribute(&supply, &demand)
        };
        assert_eq!(build(), build());
    }

    /// Supply clamp: the caller asks for 200 units but the source's
    /// `max_supply` is 50 -> the grid treats the source as having 50 and
    /// downstream sinks shortfall accordingly.
    #[test]
    fn supply_above_max_supply_is_clamped() {
        let mut grid = UtilityGrid::new();
        let src = UtilityNodeId::new(0);
        let sink = UtilityNodeId::new(1);
        grid.add_source(src, 50.0); // <-- hard cap
        grid.add_sink(sink, 100.0);
        grid.add_edge(src, sink, 100.0);

        let mut supply = BTreeMap::new();
        supply.insert(src, 200.0); // caller asks for too much
        let mut demand = BTreeMap::new();
        demand.insert(sink, 100.0);

        let result = grid.distribute(&supply, &demand);
        assert_eq!(result.delivered.get(&sink).copied(), Some(50.0));
        assert_eq!(result.shortfall.get(&sink).copied(), Some(50.0));
    }

    /// Demand clamp: caller asks for 200 units but the sink's
    /// `max_demand` is 40 -> the grid never tries to deliver more than 40.
    #[test]
    fn demand_above_max_demand_is_clamped() {
        let mut grid = UtilityGrid::new();
        let src = UtilityNodeId::new(0);
        let sink = UtilityNodeId::new(1);
        grid.add_source(src, 100.0);
        grid.add_sink(sink, 40.0); // <-- hard cap
        grid.add_edge(src, sink, 100.0);

        let mut supply = BTreeMap::new();
        supply.insert(src, 100.0);
        let mut demand = BTreeMap::new();
        demand.insert(sink, 200.0); // caller asks for too much

        let result = grid.distribute(&supply, &demand);
        // Delivered must NOT exceed max_demand.
        assert!(result.delivered.get(&sink).copied().unwrap_or(0.0) <= 40.0);
        // Sink is fully satisfied within its cap.
        assert_eq!(result.delivered.get(&sink).copied(), Some(40.0));
        // No shortfall because the cap absorbed the over-demand.
        assert_eq!(result.shortfall.get(&sink).copied(), Some(0.0));
    }

    /// Edge accounting: every unit pushed through the network is recorded
    /// on the edges it travelled, up to their capacities.
    #[test]
    fn edge_flow_records_throughput_up_to_capacity() {
        let mut grid = UtilityGrid::new();
        let src = UtilityNodeId::new(0);
        let relay = UtilityNodeId::new(1);
        let sink = UtilityNodeId::new(2);
        grid.add_source(src, 50.0);
        grid.add_relay(relay);
        grid.add_sink(sink, 30.0);
        let e1 = grid.add_edge(src, relay, 50.0);
        let e2 = grid.add_edge(relay, sink, 30.0);

        let mut supply = BTreeMap::new();
        supply.insert(src, 50.0);
        let mut demand = BTreeMap::new();
        demand.insert(sink, 30.0);

        let result = grid.distribute(&supply, &demand);
        assert_eq!(result.edge_flow.get(&e1).copied(), Some(30.0));
        assert_eq!(result.edge_flow.get(&e2).copied(), Some(30.0));
    }
}