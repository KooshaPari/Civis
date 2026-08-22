//! Water system with aqueduct network and contamination events.
//!
//! FR-CIV-INFRA-WATER models a potable-water distribution network with
//! reservoirs (supply sources), treatment plants, pipe segments (edges with
//! flow capacity), and delivery nodes (consumers). Each tick the system
//! routes water through aqueduct segments, tracks chemical/biological
//! contamination that propagates downstream, and reports delivery shortfalls
//! and contaminated zones.
//!
//! This is the **pure-logic core** of the water system. It has no Bevy /
//! async dependency and can be driven from any caller that can call
//! [`WaterSystem::distribute`] or [`WaterSystem::apply_contamination`].

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

/// Stable identifier for a node in the water network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WaterNodeId(pub u32);

impl WaterNodeId {
    /// Convenience constructor.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }
}

impl From<u32> for WaterNodeId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Stable identifier for a pipe segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PipeId(pub u32);

/// Role a node plays in the water network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaterNodeKind {
    /// Reservoir (natural or artificial water source).
    Reservoir,
    /// Treatment plant (can purify contaminated inflow).
    TreatmentPlant,
    /// Pure relay (junction / valve).
    Relay,
    /// Consumer (household, farm, market).
    Consumer,
}

/// Contamination type present in a pipe or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Contaminant {
    /// Biological contamination (bacteria, parasites).
    Biological,
    /// Chemical contamination (heavy metals, industrial runoff).
    Chemical,
    /// Sediment / particulate (reduces flow, clogs pipes).
    Sediment,
}

/// A water-network node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaterNode {
    /// Role in the network.
    pub kind: WaterNodeKind,
    /// For reservoirs: maximum flow output per tick.
    pub max_output: f32,
    /// For consumers: maximum flow demand per tick.
    pub max_demand: f32,
    /// Contaminants currently present at this node.
    pub contaminants: BTreeSet<Contaminant>,
}

impl WaterNode {
    /// Reservoir node.
    #[must_use]
    pub fn reservoir(max_output: f32) -> Self {
        Self {
            kind: WaterNodeKind::Reservoir,
            max_output,
            max_demand: 0.0,
            contaminants: BTreeSet::new(),
        }
    }

    /// Treatment plant node.
    #[must_use]
    pub fn treatment_plant() -> Self {
        Self {
            kind: WaterNodeKind::TreatmentPlant,
            max_output: 0.0,
            max_demand: 0.0,
            contaminants: BTreeSet::new(),
        }
    }

    /// Relay node (pure junction).
    #[must_use]
    pub fn relay() -> Self {
        Self {
            kind: WaterNodeKind::Relay,
            max_output: 0.0,
            max_demand: 0.0,
            contaminants: BTreeSet::new(),
        }
    }

    /// Consumer node.
    #[must_use]
    pub fn consumer(max_demand: f32) -> Self {
        Self {
            kind: WaterNodeKind::Consumer,
            max_output: 0.0,
            max_demand,
            contaminants: BTreeSet::new(),
        }
    }
}

/// A directed pipe segment between two water nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pipe {
    /// Tail node.
    pub from: WaterNodeId,
    /// Head node.
    pub to: WaterNodeId,
    /// Maximum flow (units/tick) the pipe can carry.
    pub capacity: f32,
    /// Contaminants present in this pipe segment.
    pub contaminants: BTreeSet<Contaminant>,
}

/// Outcome of a single [`WaterSystem::distribute`] call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WaterResult {
    /// Flow delivered to each consumer.
    pub delivered: BTreeMap<WaterNodeId, f32>,
    /// Unmet demand per consumer.
    pub shortfall: BTreeMap<WaterNodeId, f32>,
    /// Flow along each pipe.
    pub pipe_flow: BTreeMap<PipeId, f32>,
    /// Nodes that received contaminated water this tick.
    pub contaminated_nodes: Vec<WaterNodeId>,
}

impl WaterResult {
    /// Total shortfall across consumers.
    #[must_use]
    pub fn total_shortfall(&self) -> f32 {
        self.shortfall.values().copied().sum()
    }

    /// Total flow delivered.
    #[must_use]
    pub fn total_delivered(&self) -> f32 {
        self.delivered.values().copied().sum()
    }

    /// `true` when every consumer is fully satisfied and uncontaminated.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.shortfall.values().all(|v| *v <= 0.0) && self.contaminated_nodes.is_empty()
    }
}

/// Water distribution system with aqueduct routing and contamination
/// tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterSystem {
    nodes: BTreeMap<WaterNodeId, WaterNode>,
    outgoing: BTreeMap<WaterNodeId, Vec<(PipeId, Pipe)>>,
    pipe_index: BTreeMap<PipeId, Pipe>,
    next_node_id: u32,
    next_pipe_id: u32,
}

impl Default for WaterSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl WaterSystem {
    /// Empty system.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            outgoing: BTreeMap::new(),
            pipe_index: BTreeMap::new(),
            next_node_id: 0,
            next_pipe_id: 0,
        }
    }

    /// Register a node.
    pub fn add_node(&mut self, id: WaterNodeId, node: WaterNode) {
        self.next_node_id = self.next_node_id.max(id.0 + 1);
        self.nodes.insert(id, node);
    }

    /// Convenience: register a reservoir.
    pub fn add_reservoir(&mut self, id: WaterNodeId, max_output: f32) {
        self.add_node(id, WaterNode::reservoir(max_output));
    }

    /// Convenience: register a treatment plant.
    pub fn add_treatment_plant(&mut self, id: WaterNodeId) {
        self.add_node(id, WaterNode::treatment_plant());
    }

    /// Convenience: register a relay.
    pub fn add_relay(&mut self, id: WaterNodeId) {
        self.add_node(id, WaterNode::relay());
    }

    /// Convenience: register a consumer.
    pub fn add_consumer(&mut self, id: WaterNodeId, max_demand: f32) {
        self.add_node(id, WaterNode::consumer(max_demand));
    }

    /// Add a directed pipe. Returns its id.
    pub fn add_pipe(&mut self, from: WaterNodeId, to: WaterNodeId, capacity: f32) -> PipeId {
        let id = PipeId(self.next_pipe_id);
        self.next_pipe_id = self.next_pipe_id.saturating_add(1);
        let pipe = Pipe {
            from,
            to,
            capacity,
            contaminants: BTreeSet::new(),
        };
        self.pipe_index.insert(id, pipe.clone());
        self.outgoing.entry(from).or_default().push((id, pipe));
        id
    }

    /// Number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of pipes.
    #[must_use]
    pub fn pipe_count(&self) -> usize {
        self.pipe_index.len()
    }

    /// Read-only access to nodes.
    #[must_use]
    pub fn nodes(&self) -> &BTreeMap<WaterNodeId, WaterNode> {
        &self.nodes
    }

    /// Read-only access to pipes.
    #[must_use]
    pub fn pipes(&self) -> &BTreeMap<PipeId, Pipe> {
        &self.pipe_index
    }

    /// Introduce a contaminant at a specific node. The contamination will
    /// propagate downstream on the next [`Self::distribute`] call.
    pub fn introduce_contaminant(&mut self, node_id: WaterNodeId, contaminant: Contaminant) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.contaminants.insert(contaminant);
        }
    }

    /// Introduce a contaminant into a specific pipe segment.
    pub fn contaminate_pipe(&mut self, pipe_id: PipeId, contaminant: Contaminant) {
        if let Some(pipe) = self.pipe_index.get_mut(&pipe_id) {
            pipe.contaminants.insert(contaminant);
        }
        // Update in outgoing map too.
        if let Some(pipe) = self.pipe_index.get(&pipe_id) {
            let from = pipe.from;
            if let Some(edges) = self.outgoing.get_mut(&from) {
                for (id, p) in edges.iter_mut() {
                    if *id == pipe_id {
                        p.contaminants.insert(contaminant);
                    }
                }
            }
        }
    }

    /// Clear all contaminants from a node (treatment / flushing).
    pub fn purify_node(&mut self, node_id: WaterNodeId) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.contaminants.clear();
        }
    }

    /// Distribute water from reservoirs to consumers through the pipe
    /// network. After flow distribution, propagate contamination downstream
    /// through BFS, with treatment plants stripping contaminants.
    #[must_use]
    pub fn distribute(
        &mut self,
        supply: &BTreeMap<WaterNodeId, f32>,
        demand: &BTreeMap<WaterNodeId, f32>,
    ) -> WaterResult {
        // Residual supply from reservoirs.
        let mut residual_supply: BTreeMap<WaterNodeId, f32> = BTreeMap::new();
        for (&id, node) in &self.nodes {
            if matches!(node.kind, WaterNodeKind::Reservoir) {
                let raw = supply.get(&id).copied().unwrap_or(0.0).max(0.0);
                residual_supply.insert(id, raw.min(node.max_output.max(0.0)));
            }
        }

        // Residual demand from consumers.
        let mut residual_demand: BTreeMap<WaterNodeId, f32> = BTreeMap::new();
        let mut original_demand: BTreeMap<WaterNodeId, f32> = BTreeMap::new();
        for (&id, node) in &self.nodes {
            if matches!(node.kind, WaterNodeKind::Consumer) {
                let raw = demand.get(&id).copied().unwrap_or(0.0).max(0.0);
                let clamped = raw.min(node.max_demand.max(0.0));
                original_demand.insert(id, clamped);
                residual_demand.insert(id, clamped);
            }
        }

        // Residual pipe capacity.
        let mut residual_pipe: BTreeMap<PipeId, f32> = self
            .pipe_index
            .iter()
            .map(|(id, p)| (*id, p.capacity.max(0.0)))
            .collect();
        let mut pipe_flow: BTreeMap<PipeId, f32> =
            self.pipe_index.keys().map(|id| (*id, 0.0)).collect();

        let sources: Vec<WaterNodeId> = residual_supply.keys().copied().collect();

        let max_rounds = self.nodes.len().max(1);
        for _ in 0..max_rounds {
            let mut progress = false;

            for &src in &sources {
                let src_supply = residual_supply.get(&src).copied().unwrap_or(0.0);
                if src_supply <= 0.0 {
                    continue;
                }

                let Some((consumer, path)) =
                    self.shortest_path_to_consumer(src, &residual_demand)
                else {
                    continue;
                };

                let mut bottleneck = src_supply;
                bottleneck = bottleneck.min(
                    residual_demand
                        .get(&consumer)
                        .copied()
                        .unwrap_or(0.0)
                        .max(0.0),
                );
                for &pipe_id in &path {
                    bottleneck = bottleneck
                        .min(residual_pipe.get(&pipe_id).copied().unwrap_or(0.0).max(0.0));
                }
                if bottleneck <= 0.0 {
                    residual_supply.insert(src, 0.0);
                    continue;
                }

                residual_supply.insert(src, src_supply - bottleneck);
                let prev_demand = residual_demand.get(&consumer).copied().unwrap_or(0.0);
                residual_demand.insert(consumer, (prev_demand - bottleneck).max(0.0));
                for &pipe_id in &path {
                    let prev_flow = pipe_flow.get(&pipe_id).copied().unwrap_or(0.0);
                    pipe_flow.insert(pipe_id, prev_flow + bottleneck);
                    let prev_cap = residual_pipe.get(&pipe_id).copied().unwrap_or(0.0);
                    residual_pipe.insert(pipe_id, (prev_cap - bottleneck).max(0.0));
                }
                progress = true;
            }

            if !progress {
                break;
            }
        }

        // Build delivered / shortfall.
        let mut delivered: BTreeMap<WaterNodeId, f32> = BTreeMap::new();
        let mut shortfall: BTreeMap<WaterNodeId, f32> = BTreeMap::new();
        for (&id, &orig) in &original_demand {
            let remain = residual_demand.get(&id).copied().unwrap_or(0.0);
            delivered.insert(id, (orig - remain).max(0.0));
            shortfall.insert(id, remain);
        }
        for &id in self.nodes.keys() {
            delivered.entry(id).or_insert(0.0);
        }

        // Propagate contamination downstream.
        let contaminated_nodes = self.propagate_contamination();

        WaterResult {
            delivered,
            shortfall,
            pipe_flow,
            contaminated_nodes,
        }
    }

    /// BFS propagation of contaminants from sources through pipes.
    /// Treatment plants strip all contaminants at their output.
    fn propagate_contamination(&mut self) -> Vec<WaterNodeId> {
        // Collect all nodes that have contaminants initially or inherit
        // them from upstream pipes.
        let mut contaminated: BTreeSet<WaterNodeId> = BTreeSet::new();
        let mut queue: VecDeque<WaterNodeId> = VecDeque::new();

        // Seed from nodes with existing contaminants.
        for (&id, node) in &self.nodes {
            if !node.contaminants.is_empty() {
                contaminated.insert(id);
                queue.push_back(id);
            }
        }

        // BFS downstream.
        while let Some(node) = queue.pop_front() {
            let node_contaminants: BTreeSet<Contaminant> = self
                .nodes
                .get(&node)
                .map(|n| n.contaminants.clone())
                .unwrap_or_default();

            if let Some(edges) = self.outgoing.get(&node) {
                for (_pipe_id, pipe) in edges.iter() {
                    // Combine node contaminants with pipe's own contaminants.
                    let mut combined: BTreeSet<Contaminant> = node_contaminants.clone();
                    combined.extend(pipe.contaminants.iter());

                    // Treatment plants strip everything at their input.
                    let target = self.nodes.get(&pipe.to);
                    let stripped = if target
                        .map_or(false, |n| matches!(n.kind, WaterNodeKind::TreatmentPlant))
                    {
                        BTreeSet::new()
                    } else {
                        combined
                    };

                    if !stripped.is_empty() {
                        if let Some(target) = self.nodes.get_mut(&pipe.to) {
                            target.contaminants.extend(stripped.iter());
                        }
                        if contaminated.insert(pipe.to) {
                            queue.push_back(pipe.to);
                        }
                    }
                }
            }
        }

        let mut result: Vec<WaterNodeId> = contaminated.into_iter().collect();
        result.sort_by_key(|id| id.0);
        result
    }

    /// BFS shortest path from `start` to any consumer with positive
    /// residual demand.
    fn shortest_path_to_consumer(
        &self,
        start: WaterNodeId,
        residual_demand: &BTreeMap<WaterNodeId, f32>,
    ) -> Option<(WaterNodeId, Vec<PipeId>)> {
        self.outgoing.get(&start)?;

        let mut parent: BTreeMap<WaterNodeId, (PipeId, WaterNodeId)> = BTreeMap::new();
        parent.insert(start, (PipeId(u32::MAX), start));
        let mut frontier: VecDeque<WaterNodeId> = VecDeque::new();
        frontier.push_back(start);

        let targets: BTreeSet<WaterNodeId> = residual_demand
            .iter()
            .filter_map(|(id, d)| if *d > 0.0 { Some(*id) } else { None })
            .collect();

        let mut found: Option<WaterNodeId> = None;
        while let Some(node) = frontier.pop_front() {
            if targets.contains(&node) {
                found = Some(node);
                break;
            }
            let mut edges: Vec<(PipeId, WaterNodeId)> = self
                .outgoing
                .get(&node)
                .map(|v| v.iter().map(|(id, p)| (*id, p.to)).collect())
                .unwrap_or_default();
            edges.sort_by_key(|(pid, nid)| (*pid, *nid));
            for (pipe_id, neighbour) in edges {
                if let std::collections::btree_map::Entry::Vacant(entry) = parent.entry(neighbour) {
                    entry.insert((pipe_id, node));
                    frontier.push_back(neighbour);
                }
            }
        }

        let consumer = found?;
        let mut pipes: Vec<PipeId> = Vec::new();
        let mut cur = consumer;
        while cur != start {
            let (pipe_id, prev) = parent.get(&cur).copied()?;
            pipes.push(pipe_id);
            cur = prev;
        }
        pipes.reverse();
        Some((consumer, pipes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_simple_water_system() -> WaterSystem {
        // reservoir(0) --pipe1(20)--> plant(1) --pipe2(15)--> consumer(2)
        let mut sys = WaterSystem::new();
        sys.add_reservoir(WaterNodeId(0), 50.0);
        sys.add_treatment_plant(WaterNodeId(1));
        sys.add_consumer(WaterNodeId(2), 30.0);
        sys.add_pipe(WaterNodeId(0), WaterNodeId(1), 20.0);
        sys.add_pipe(WaterNodeId(1), WaterNodeId(2), 15.0);
        sys
    }

    #[test]
    fn supply_satisfies_demand_within_capacity() {
        let mut sys = build_simple_water_system();
        let mut supply = BTreeMap::new();
        supply.insert(WaterNodeId(0), 30.0);
        let mut demand = BTreeMap::new();
        demand.insert(WaterNodeId(2), 15.0);

        let result = sys.distribute(&supply, &demand);
        assert_eq!(result.delivered.get(&WaterNodeId(2)).copied(), Some(15.0));
        assert_eq!(result.shortfall.get(&WaterNodeId(2)).copied(), Some(0.0));
        assert!(result.is_healthy());
    }

    #[test]
    fn pipe_capacity_bottleneck_causes_shortfall() {
        let mut sys = build_simple_water_system();
        let mut supply = BTreeMap::new();
        supply.insert(WaterNodeId(0), 50.0);
        let mut demand = BTreeMap::new();
        demand.insert(WaterNodeId(2), 30.0);

        let result = sys.distribute(&supply, &demand);
        // Pipe2 capacity is 15.
        assert_eq!(result.delivered.get(&WaterNodeId(2)).copied(), Some(15.0));
        assert_eq!(result.shortfall.get(&WaterNodeId(2)).copied(), Some(15.0));
        assert!(!result.is_healthy());
    }

    #[test]
    fn contamination_propagates_downstream() {
        let mut sys = build_simple_water_system();
        sys.introduce_contaminant(WaterNodeId(0), Contaminant::Chemical);

        let mut supply = BTreeMap::new();
        supply.insert(WaterNodeId(0), 15.0);
        let mut demand = BTreeMap::new();
        demand.insert(WaterNodeId(2), 15.0);

        let result = sys.distribute(&supply, &demand);
        assert!(
            result.contaminated_nodes.contains(&WaterNodeId(2)),
            "consumer should be contaminated"
        );
        assert!(!result.is_healthy());
    }

    #[test]
    fn treatment_plant_strips_contamination() {
        let mut sys = build_simple_water_system();
        // Contaminate pipe from reservoir to plant (not the plant itself).
        let pipes: Vec<PipeId> = sys.pipe_index.keys().copied().collect();
        sys.contaminate_pipe(pipes[0], Contaminant::Biological);

        let mut supply = BTreeMap::new();
        supply.insert(WaterNodeId(0), 15.0);
        let mut demand = BTreeMap::new();
        demand.insert(WaterNodeId(2), 15.0);

        let result = sys.distribute(&supply, &demand);
        // Treatment plant (node 1) should strip biological contamination,
        // so consumer (node 2) should NOT be contaminated.
        assert!(
            !result.contaminated_nodes.contains(&WaterNodeId(2)),
            "treatment plant should protect downstream consumer"
        );
        assert!(result.is_healthy());
    }

    #[test]
    fn purify_node_clears_contaminants() {
        let mut sys = build_simple_water_system();
        sys.introduce_contaminant(WaterNodeId(0), Contaminant::Sediment);
        assert!(!sys
            .nodes()
            .get(&WaterNodeId(0))
            .unwrap()
            .contaminants
            .is_empty());

        sys.purify_node(WaterNodeId(0));
        assert!(sys
            .nodes()
            .get(&WaterNodeId(0))
            .unwrap()
            .contaminants
            .is_empty());
    }

    #[test]
    fn multiple_reservoirs_feed_multiple_consumers() {
        let mut sys = WaterSystem::new();
        sys.add_reservoir(WaterNodeId(0), 40.0);
        sys.add_reservoir(WaterNodeId(1), 40.0);
        sys.add_consumer(WaterNodeId(2), 25.0);
        sys.add_consumer(WaterNodeId(3), 25.0);
        sys.add_pipe(WaterNodeId(0), WaterNodeId(2), 40.0);
        sys.add_pipe(WaterNodeId(1), WaterNodeId(3), 40.0);

        let mut supply = BTreeMap::new();
        supply.insert(WaterNodeId(0), 40.0);
        supply.insert(WaterNodeId(1), 40.0);
        let mut demand = BTreeMap::new();
        demand.insert(WaterNodeId(2), 25.0);
        demand.insert(WaterNodeId(3), 25.0);

        let result = sys.distribute(&supply, &demand);
        assert!(result.is_healthy());
        assert_eq!(result.delivered.get(&WaterNodeId(2)).copied(), Some(25.0));
        assert_eq!(result.delivered.get(&WaterNodeId(3)).copied(), Some(25.0));
    }

    #[test]
    fn contamination_from_pipe_also_propagates() {
        let mut sys = WaterSystem::new();
        sys.add_reservoir(WaterNodeId(0), 50.0);
        sys.add_consumer(WaterNodeId(1), 20.0);
        let pipe = sys.add_pipe(WaterNodeId(0), WaterNodeId(1), 50.0);
        sys.contaminate_pipe(pipe, Contaminant::Chemical);

        let mut supply = BTreeMap::new();
        supply.insert(WaterNodeId(0), 20.0);
        let mut demand = BTreeMap::new();
        demand.insert(WaterNodeId(1), 20.0);

        let result = sys.distribute(&supply, &demand);
        assert!(
            result.contaminated_nodes.contains(&WaterNodeId(1)),
            "consumer should be contaminated via pipe"
        );
    }
}
