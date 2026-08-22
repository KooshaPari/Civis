//! Energy grid with supply/demand balancing and blackout simulation.
//!
//! FR-CIV-INFRA-ENERGY models a power distribution network with generators
//! (supply), consumers (demand), and transmission lines (edges with capacity).
//! Each tick the grid attempts to satisfy all demand from available supply.
//! When supply falls short or transmission capacity bottlenecks starve
//! consumers, the grid reports *power shortfalls* and optionally triggers
//! a **blackout cascade** — an expanding zone of de-energised districts.
//!
//! This is the **pure-logic core** of the energy grid system. It has no
//! Bevy / async dependency and can be driven from any caller that can call
//! [`EnergyGrid::balance`] (sim scheduler, replay scrubber, test harness).

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

/// Stable identifier for a node in the energy grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EnergyNodeId(pub u32);

impl EnergyNodeId {
    /// Convenience constructor.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }
}

impl From<u32> for EnergyNodeId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Stable identifier for a transmission line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TransmissionLineId(pub u32);

/// Role a node plays in the energy grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyNodeKind {
    /// Generator (power plant, solar field, wind farm).
    Generator,
    /// Relay node (substation, transformer junction).
    Substation,
    /// Consumer (district, factory, household cluster).
    Consumer,
}

/// An energy-grid node.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EnergyNode {
    /// Role in the grid.
    pub kind: EnergyNodeKind,
    /// For generators: maximum MW output this tick.
    pub max_output: f32,
    /// For consumers: maximum MW demand this tick.
    pub max_demand: f32,
    /// Whether the node is currently energised (false = blackout zone).
    pub energised: bool,
}

impl EnergyNode {
    /// Generator node.
    #[must_use]
    pub fn generator(max_output: f32) -> Self {
        Self {
            kind: EnergyNodeKind::Generator,
            max_output,
            max_demand: 0.0,
            energised: true,
        }
    }

    /// Substation node (pure junction).
    #[must_use]
    pub fn substation() -> Self {
        Self {
            kind: EnergyNodeKind::Substation,
            max_output: 0.0,
            max_demand: 0.0,
            energised: true,
        }
    }

    /// Consumer node.
    #[must_use]
    pub fn consumer(max_demand: f32) -> Self {
        Self {
            kind: EnergyNodeKind::Consumer,
            max_output: 0.0,
            max_demand,
            energised: true,
        }
    }
}

/// A directed transmission line between two energy nodes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransmissionLine {
    /// Tail node.
    pub from: EnergyNodeId,
    /// Head node.
    pub to: EnergyNodeId,
    /// Maximum MW this line can carry per tick.
    pub capacity: f32,
}

/// Outcome of a single [`EnergyGrid::balance`] call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BalanceResult {
    /// MW delivered to each consumer.
    pub delivered: BTreeMap<EnergyNodeId, f32>,
    /// MW each consumer could not get (shortfall).
    pub shortfall: BTreeMap<EnergyNodeId, f32>,
    /// MW that flowed along each transmission line.
    pub line_flow: BTreeMap<TransmissionLineId, f32>,
    /// Nodes that entered blackout during this tick.
    pub blackouts: Vec<EnergyNodeId>,
}

impl BalanceResult {
    /// Total shortfall across all consumers.
    #[must_use]
    pub fn total_shortfall(&self) -> f32 {
        self.shortfall.values().copied().sum()
    }

    /// Total MW delivered.
    #[must_use]
    pub fn total_delivered(&self) -> f32 {
        self.delivered.values().copied().sum()
    }

    /// `true` when every consumer received full demand.
    #[must_use]
    pub fn all_satisfied(&self) -> bool {
        self.shortfall.values().all(|v| *v <= 0.0)
    }
}

/// Configuration for blackout cascade simulation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BlackoutConfig {
    /// If a consumer's shortfall ratio exceeds this threshold, it enters
    /// blackout (0.0–1.0). Example: 0.5 means >50% unmet demand triggers
    /// blackout.
    pub threshold: f32,
    /// Blackout propagates to neighbouring consumers within this hop count.
    pub propagation_hops: u32,
}

impl Default for BlackoutConfig {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            propagation_hops: 2,
        }
    }
}

/// Directed energy-grid graph with supply/demand balancing and blackout
/// simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyGrid {
    nodes: BTreeMap<EnergyNodeId, EnergyNode>,
    outgoing: BTreeMap<EnergyNodeId, Vec<(TransmissionLineId, TransmissionLine)>>,
    line_index: BTreeMap<TransmissionLineId, TransmissionLine>,
    blackout_config: BlackoutConfig,
    next_node_id: u32,
    next_line_id: u32,
}

impl Default for EnergyGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl EnergyGrid {
    /// Empty grid.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            outgoing: BTreeMap::new(),
            line_index: BTreeMap::new(),
            blackout_config: BlackoutConfig::default(),
            next_node_id: 0,
            next_line_id: 0,
        }
    }

    /// Set the blackout cascade configuration.
    pub fn set_blackout_config(&mut self, config: BlackoutConfig) {
        self.blackout_config = config;
    }

    /// Register a node.
    pub fn add_node(&mut self, id: EnergyNodeId, node: EnergyNode) {
        self.next_node_id = self.next_node_id.max(id.0 + 1);
        self.nodes.insert(id, node);
    }

    /// Convenience: register a generator.
    pub fn add_generator(&mut self, id: EnergyNodeId, max_output: f32) {
        self.add_node(id, EnergyNode::generator(max_output));
    }

    /// Convenience: register a substation.
    pub fn add_substation(&mut self, id: EnergyNodeId) {
        self.add_node(id, EnergyNode::substation());
    }

    /// Convenience: register a consumer.
    pub fn add_consumer(&mut self, id: EnergyNodeId, max_demand: f32) {
        self.add_node(id, EnergyNode::consumer(max_demand));
    }

    /// Add a directed transmission line. Returns its id.
    pub fn add_line(
        &mut self,
        from: EnergyNodeId,
        to: EnergyNodeId,
        capacity: f32,
    ) -> TransmissionLineId {
        let id = TransmissionLineId(self.next_line_id);
        self.next_line_id = self.next_line_id.saturating_add(1);
        let line = TransmissionLine { from, to, capacity };
        self.outgoing.entry(from).or_default().push((id, line));
        self.line_index.insert(id, line);
        id
    }

    /// Number of nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of lines.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_index.len()
    }

    /// Read-only access to nodes.
    #[must_use]
    pub fn nodes(&self) -> &BTreeMap<EnergyNodeId, EnergyNode> {
        &self.nodes
    }

    /// Read-only access to lines.
    #[must_use]
    pub fn lines(&self) -> &BTreeMap<TransmissionLineId, TransmissionLine> {
        &self.line_index
    }

    /// De-energise a specific node (simulate equipment failure).
    pub fn blackout_node(&mut self, id: EnergyNodeId) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.energised = false;
        }
    }

    /// Re-energise a node (repair).
    pub fn restore_node(&mut self, id: EnergyNodeId) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.energised = true;
        }
    }

    /// Distribute power from generators to consumers through the
    /// transmission network. Performs an iterative proportional push along
    /// shortest paths (BFS hops), saturating the minimum of source supply,
    /// line capacity, and sink demand.
    ///
    /// After balancing, any consumer with shortfall exceeding
    /// `blackout_config.threshold` triggers a cascade that de-energises
    /// nearby consumers within `propagation_hops` hops.
    #[must_use]
    pub fn balance(
        &mut self,
        supply: &BTreeMap<EnergyNodeId, f32>,
        demand: &BTreeMap<EnergyNodeId, f32>,
    ) -> BalanceResult {
        // Initialise residual supply per generator.
        let mut residual_supply: BTreeMap<EnergyNodeId, f32> = BTreeMap::new();
        for (&id, node) in &self.nodes {
            if matches!(node.kind, EnergyNodeKind::Generator) && node.energised {
                let raw = supply.get(&id).copied().unwrap_or(0.0).max(0.0);
                residual_supply.insert(id, raw.min(node.max_output.max(0.0)));
            }
        }

        // Initialise residual demand per consumer.
        let mut residual_demand: BTreeMap<EnergyNodeId, f32> = BTreeMap::new();
        let mut original_demand: BTreeMap<EnergyNodeId, f32> = BTreeMap::new();
        for (&id, node) in &self.nodes {
            if matches!(node.kind, EnergyNodeKind::Consumer) && node.energised {
                let raw = demand.get(&id).copied().unwrap_or(0.0).max(0.0);
                let clamped = raw.min(node.max_demand.max(0.0));
                original_demand.insert(id, clamped);
                residual_demand.insert(id, clamped);
            }
        }

        // Residual line capacity.
        let mut residual_line: BTreeMap<TransmissionLineId, f32> = self
            .line_index
            .iter()
            .map(|(id, l)| (*id, l.capacity.max(0.0)))
            .collect();
        let mut line_flow: BTreeMap<TransmissionLineId, f32> =
            self.line_index.keys().map(|id| (*id, 0.0)).collect();

        let sources: Vec<EnergyNodeId> = residual_supply.keys().copied().collect();
        let consumers: Vec<EnergyNodeId> = residual_demand.keys().copied().collect();

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
                for &line_id in &path {
                    bottleneck = bottleneck
                        .min(residual_line.get(&line_id).copied().unwrap_or(0.0).max(0.0));
                }
                if bottleneck <= 0.0 {
                    residual_supply.insert(src, 0.0);
                    continue;
                }

                residual_supply.insert(src, src_supply - bottleneck);
                let prev_demand = residual_demand.get(&consumer).copied().unwrap_or(0.0);
                residual_demand.insert(consumer, (prev_demand - bottleneck).max(0.0));
                for &line_id in &path {
                    let prev_flow = line_flow.get(&line_id).copied().unwrap_or(0.0);
                    line_flow.insert(line_id, prev_flow + bottleneck);
                    let prev_cap = residual_line.get(&line_id).copied().unwrap_or(0.0);
                    residual_line.insert(line_id, (prev_cap - bottleneck).max(0.0));
                }
                progress = true;
            }

            if !progress {
                break;
            }
        }

        // Build delivered / shortfall.
        let mut delivered: BTreeMap<EnergyNodeId, f32> = BTreeMap::new();
        let mut shortfall: BTreeMap<EnergyNodeId, f32> = BTreeMap::new();
        for &c in &consumers {
            let orig = original_demand.get(&c).copied().unwrap_or(0.0);
            let remain = residual_demand.get(&c).copied().unwrap_or(0.0);
            delivered.insert(c, (orig - remain).max(0.0));
            shortfall.insert(c, remain);
        }
        for &id in self.nodes.keys() {
            delivered.entry(id).or_insert(0.0);
        }

        // Blackout cascade.
        let blackouts = self.simulate_blackout_cascade(&shortfall);

        BalanceResult {
            delivered,
            shortfall,
            line_flow,
            blackouts,
        }
    }

    /// Run blackout cascade: consumers whose shortfall ratio exceeds the
    /// threshold are de-energised, then the blackout propagates to
    /// neighbouring consumers within the hop limit.
    fn simulate_blackout_cascade(
        &mut self,
        shortfall: &BTreeMap<EnergyNodeId, f32>,
    ) -> Vec<EnergyNodeId> {
        let mut blackouts: Vec<EnergyNodeId> = Vec::new();

        // Find initial blackout triggers.
        let mut queue: VecDeque<(EnergyNodeId, u32)> = VecDeque::new();
        let mut visited: BTreeSet<EnergyNodeId> = BTreeSet::new();

        for (&id, &short) in shortfall {
            let orig = self.nodes.get(&id).and_then(|n| {
                if matches!(n.kind, EnergyNodeKind::Consumer) {
                    Some(n.max_demand)
                } else {
                    None
                }
            });
            if let Some(max_d) = orig {
                if max_d > 0.0 && (short / max_d) >= self.blackout_config.threshold {
                    queue.push_back((id, 0));
                    visited.insert(id);
                }
            }
        }

        // BFS propagation.
        while let Some((node, depth)) = queue.pop_front() {
            if let Some(n) = self.nodes.get_mut(&node) {
                n.energised = false;
            }
            blackouts.push(node);

            if depth < self.blackout_config.propagation_hops {
                if let Some(edges) = self.outgoing.get(&node) {
                    for &(_, line) in edges {
                        if !visited.contains(&line.to) {
                            if let Some(target) = self.nodes.get(&line.to) {
                                if matches!(target.kind, EnergyNodeKind::Consumer) {
                                    visited.insert(line.to);
                                    queue.push_back((line.to, depth + 1));
                                }
                            }
                        }
                    }
                }
            }
        }

        blackouts.sort_by_key(|id| id.0);
        blackouts
    }

    /// BFS shortest path (fewest hops) from `start` to any consumer with
    /// positive residual demand.
    fn shortest_path_to_consumer(
        &self,
        start: EnergyNodeId,
        residual_demand: &BTreeMap<EnergyNodeId, f32>,
    ) -> Option<(EnergyNodeId, Vec<TransmissionLineId>)> {
        self.outgoing.get(&start)?;

        let mut parent: BTreeMap<EnergyNodeId, (TransmissionLineId, EnergyNodeId)> =
            BTreeMap::new();
        parent.insert(start, (TransmissionLineId(u32::MAX), start));
        let mut frontier: VecDeque<EnergyNodeId> = VecDeque::new();
        frontier.push_back(start);

        let targets: BTreeSet<EnergyNodeId> = residual_demand
            .iter()
            .filter_map(|(id, d)| if *d > 0.0 { Some(*id) } else { None })
            .collect();

        let mut found: Option<EnergyNodeId> = None;
        while let Some(node) = frontier.pop_front() {
            if targets.contains(&node) {
                found = Some(node);
                break;
            }
            let mut edges: Vec<(TransmissionLineId, EnergyNodeId)> = self
                .outgoing
                .get(&node)
                .map(|v| v.iter().map(|(id, l)| (*id, l.to)).collect())
                .unwrap_or_default();
            edges.sort_by_key(|(eid, nid)| (*eid, *nid));
            for (line_id, neighbour) in edges {
                let node_e = self.nodes.get(&neighbour);
                if node_e.map_or(true, |n| !n.energised) {
                    continue;
                }
                if let std::collections::btree_map::Entry::Vacant(entry) = parent.entry(neighbour) {
                    entry.insert((line_id, node));
                    frontier.push_back(neighbour);
                }
            }
        }

        let consumer = found?;
        let mut lines: Vec<TransmissionLineId> = Vec::new();
        let mut cur = consumer;
        while cur != start {
            let (line_id, prev) = parent.get(&cur).copied()?;
            lines.push(line_id);
            cur = prev;
        }
        lines.reverse();
        Some((consumer, lines))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_simple_grid() -> EnergyGrid {
        // gen(0) -- line1(20) --> substation(1) -- line2(15) --> consumer(2)
        let mut grid = EnergyGrid::new();
        grid.add_generator(EnergyNodeId(0), 50.0);
        grid.add_substation(EnergyNodeId(1));
        grid.add_consumer(EnergyNodeId(2), 30.0);
        grid.add_line(EnergyNodeId(0), EnergyNodeId(1), 20.0);
        grid.add_line(EnergyNodeId(1), EnergyNodeId(2), 15.0);
        grid
    }

    #[test]
    fn supply_satisfies_demand_when_capacity_sufficient() {
        let mut grid = build_simple_grid();
        let mut supply = BTreeMap::new();
        supply.insert(EnergyNodeId(0), 30.0);
        let mut demand = BTreeMap::new();
        demand.insert(EnergyNodeId(2), 15.0);

        let result = grid.balance(&supply, &demand);
        assert_eq!(result.delivered.get(&EnergyNodeId(2)).copied(), Some(15.0));
        assert_eq!(result.shortfall.get(&EnergyNodeId(2)).copied(), Some(0.0));
        assert!(result.all_satisfied());
    }

    #[test]
    fn capacity_bottleneck_causes_shortfall() {
        let mut grid = build_simple_grid();
        let mut supply = BTreeMap::new();
        supply.insert(EnergyNodeId(0), 50.0);
        let mut demand = BTreeMap::new();
        demand.insert(EnergyNodeId(2), 30.0);

        let result = grid.balance(&supply, &demand);
        // Line2 capacity is 15, so at most 15 MW gets through.
        assert_eq!(result.delivered.get(&EnergyNodeId(2)).copied(), Some(15.0));
        assert_eq!(result.shortfall.get(&EnergyNodeId(2)).copied(), Some(15.0));
        assert!(!result.all_satisfied());
    }

    #[test]
    fn blackout_triggers_when_shortfall_exceeds_threshold() {
        let mut grid = build_simple_grid();
        let mut supply = BTreeMap::new();
        supply.insert(EnergyNodeId(0), 50.0);
        let mut demand = BTreeMap::new();
        demand.insert(EnergyNodeId(2), 30.0);

        let result = grid.balance(&supply, &demand);
        // Shortfall = 15 out of 30 demand = 50%, at default threshold 0.5.
        assert!(
            result.blackouts.contains(&EnergyNodeId(2)),
            "consumer should enter blackout"
        );
    }

    #[test]
    fn blackout_does_not_trigger_when_shortfall_below_threshold() {
        let mut grid = build_simple_grid();
        let mut supply = BTreeMap::new();
        supply.insert(EnergyNodeId(0), 50.0);
        let mut demand = BTreeMap::new();
        demand.insert(EnergyNodeId(2), 10.0); // demand 10, capacity 15 -> no shortfall

        let result = grid.balance(&supply, &demand);
        assert!(result.blackouts.is_empty());
    }

    #[test]
    fn de_energised_generator_produces_no_power() {
        let mut grid = build_simple_grid();
        grid.blackout_node(EnergyNodeId(0));
        let mut supply = BTreeMap::new();
        supply.insert(EnergyNodeId(0), 50.0);
        let mut demand = BTreeMap::new();
        demand.insert(EnergyNodeId(2), 10.0);

        let result = grid.balance(&supply, &demand);
        assert_eq!(result.delivered.get(&EnergyNodeId(2)).copied(), Some(0.0));
        assert_eq!(result.shortfall.get(&EnergyNodeId(2)).copied(), Some(10.0));
    }

    #[test]
    fn restore_node_resumes_supply() {
        let mut grid = build_simple_grid();
        grid.blackout_node(EnergyNodeId(0));
        grid.restore_node(EnergyNodeId(0));
        let mut supply = BTreeMap::new();
        supply.insert(EnergyNodeId(0), 50.0);
        let mut demand = BTreeMap::new();
        demand.insert(EnergyNodeId(2), 10.0);

        let result = grid.balance(&supply, &demand);
        assert!(result.all_satisfied());
    }

    #[test]
    fn multiple_generators_distribute_to_multiple_consumers() {
        let mut grid = EnergyGrid::new();
        grid.add_generator(EnergyNodeId(0), 40.0);
        grid.add_generator(EnergyNodeId(1), 40.0);
        grid.add_consumer(EnergyNodeId(2), 25.0);
        grid.add_consumer(EnergyNodeId(3), 25.0);
        grid.add_line(EnergyNodeId(0), EnergyNodeId(2), 40.0);
        grid.add_line(EnergyNodeId(1), EnergyNodeId(3), 40.0);

        let mut supply = BTreeMap::new();
        supply.insert(EnergyNodeId(0), 40.0);
        supply.insert(EnergyNodeId(1), 40.0);
        let mut demand = BTreeMap::new();
        demand.insert(EnergyNodeId(2), 25.0);
        demand.insert(EnergyNodeId(3), 25.0);

        let result = grid.balance(&supply, &demand);
        assert!(result.all_satisfied());
        assert_eq!(result.delivered.get(&EnergyNodeId(2)).copied(), Some(25.0));
        assert_eq!(result.delivered.get(&EnergyNodeId(3)).copied(), Some(25.0));
    }
}
