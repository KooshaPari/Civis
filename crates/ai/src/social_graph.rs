//! Kinship + contact social graph (FR-CIV-PSYCHE-910).
//!
//! A *social graph* emerges from three kinds of events recorded against pairs
//! of agents:
//!
//! 1. **Co-location** — agents observed in the same place at the same time
//!    (e.g. living in the same household, working the same field).
//! 2. **Reproduction** — a parent/child or sibling/sibling edge forms when a
//!    birth event is recorded.
//! 3. **Interaction** — anything from a greeting to a trade to a fight. The
//!    sign of the event decides whether the edge accumulates as a *bond*
//!    (positive weight) or a *grudge* (negative weight).
//!
//! Edges are stored **once per unordered pair**; every mutation flows through
//! [`SocialGraph::apply_event`] so the increment / decay rule is in one place.
//! The graph is *event-driven*: an explicit [`SocialGraph::decay`] pass ages
//! every edge toward zero, which is how "absence decays" the contact signal
//! without us having to schedule timers per pair.
//!
//! This module is **pure** (no I/O, no async, no Bevy ECS dependency) so it
//! can be embedded inside a Bevy system, a worker-pool job, or a one-off unit
//! test without dragging the rest of `civ-ai` along. It is intentionally
//! domain-agnostic — the same graph works for villagers, factions, or
//! civilizations.
//!
//! See `docs/development-guide/fr-psyche-social-graph.md` (if present) and the
//! `crates/agents/src/psyche.rs` affect-vector for the *mood* half of the
//! psyche model; this module is the *relational* half.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stable identifier for an agent inside the graph.
pub type AgentId = u64;

/// The kind of event that drives an edge's weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocialEventKind {
    /// Agents were observed in the same place at the same time. Positive sign.
    CoLocated,
    /// A birth was recorded linking two agents (parent<->child, or sibling
    /// pair). Positive sign, with a stronger default magnitude than contact.
    Kinship,
    /// A neutral social interaction (chat, trade, cooperative work). Positive
    /// sign.
    Interaction,
    /// A hostile interaction (fight, theft, betrayal). Negative sign.
    Conflict,
}

impl SocialEventKind {
    /// The sign of the weight delta this kind of event would contribute.
    ///
    /// `1.0` for cooperative kinds, `-1.0` for conflict. The magnitude is
    /// supplied separately by [`SocialGraph::apply_event`] so callers can
    /// scale it (a shouting match weighs more than a nod).
    #[must_use]
    pub fn sign(self) -> f32 {
        match self {
            SocialEventKind::CoLocated | SocialEventKind::Kinship | SocialEventKind::Interaction => {
                1.0
            }
            SocialEventKind::Conflict => -1.0,
        }
    }
}

/// A single typed event that mutates an edge.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SocialEvent {
    /// The two agents the event concerns. Order does not matter — the graph
    /// stores each unordered pair exactly once.
    pub a: AgentId,
    /// The second agent. Must differ from `a`; the graph silently no-ops a
    /// self-event rather than panic, so noisy callers don't bring the sim down.
    pub b: AgentId,
    /// What kind of event.
    pub kind: SocialEventKind,
    /// Magnitude of the weight delta. Positive number; the sign of the delta
    /// comes from [`SocialEventKind::sign`].
    pub magnitude: f32,
    /// Simulation tick when the event was recorded. Used to age edges and to
    /// trace provenance.
    pub tick: u64,
}

impl SocialEvent {
    /// Convenience constructor for the common interaction case.
    #[must_use]
    pub fn interaction(a: AgentId, b: AgentId, tick: u64) -> Self {
        Self {
            a,
            b,
            kind: SocialEventKind::Interaction,
            magnitude: 1.0,
            tick,
        }
    }

    /// Convenience constructor for a kinship (parent<->child or sibling)
    /// event with a stronger default magnitude.
    #[must_use]
    pub fn kinship(a: AgentId, b: AgentId, tick: u64) -> Self {
        Self {
            a,
            b,
            kind: SocialEventKind::Kinship,
            magnitude: 2.0,
            tick,
        }
    }

    /// Convenience constructor for a co-location event with a small magnitude.
    #[must_use]
    pub fn co_located(a: AgentId, b: AgentId, tick: u64) -> Self {
        Self {
            a,
            b,
            kind: SocialEventKind::CoLocated,
            magnitude: 0.25,
            tick,
        }
    }

    /// Convenience constructor for a conflict event with a small magnitude.
    #[must_use]
    pub fn conflict(a: AgentId, b: AgentId, tick: u64) -> Self {
        Self {
            a,
            b,
            kind: SocialEventKind::Conflict,
            magnitude: 1.0,
            tick,
        }
    }
}

/// Why two agents are linked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationKind {
    /// Blood / family tie (parent, child, sibling).
    Kinship,
    /// Repeated contact + interactions (household, workmates, neighbors).
    Contact,
    /// Edge exists but wither dominated by hostile events.
    Grudge,
    /// Edge is mixed: started as contact but had at least one conflict.
    Mixed,
}

impl RelationKind {
    /// Re-derive the relation kind from the current edge weight and counts.
    #[must_use]
    pub fn classify(weight: f32, kinship_count: u32, conflict_count: u32) -> Self {
        if kinship_count > 0 {
            return RelationKind::Kinship;
        }
        if weight < 0.0 {
            return RelationKind::Grudge;
        }
        if conflict_count > 0 {
            return RelationKind::Mixed;
        }
        RelationKind::Contact
    }
}

/// A directed-into-undirected edge between two agents.
///
/// Stored once per unordered pair; both [`SocialGraph::apply_event`] and
/// [`SocialGraph::edge`] accept either ordering of the agent ids.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SocialEdge {
    /// Lower-id endpoint (canonical ordering).
    pub a: AgentId,
    /// Higher-id endpoint (canonical ordering).
    pub b: AgentId,
    /// Signed weight. Positive = bond, negative = grudge.
    pub weight: f32,
    /// Number of cooperative events recorded on this edge.
    pub positive_events: u32,
    /// Number of hostile events recorded on this edge.
    pub negative_events: u32,
    /// Number of kinship events recorded on this edge.
    pub kinship_events: u32,
    /// Most recent tick any event touched this edge.
    pub last_touched_tick: u64,
}

impl SocialEdge {
    /// The current relation kind — see [`RelationKind::classify`].
    #[must_use]
    pub fn relation(&self) -> RelationKind {
        RelationKind::classify(self.weight, self.kinship_events, self.negative_events)
    }
}

/// Outcome of [`SocialGraph::apply_event`]. Useful for tests and for callers
/// that want to log "an edge was born" or "an edge just tipped into a grudge".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApplyOutcome {
    /// No edge exists and the event was too small to create one.
    NoOp,
    /// Edge did not previously exist; it was just created.
    Created,
    /// Edge existed and its weight was updated.
    Updated {
        /// Weight *before* this event.
        prev_weight: f32,
        /// Weight *after* this event.
        new_weight: f32,
    },
}

/// The social graph itself.
///
/// Holds a flat `HashMap` keyed on the *canonical* (lower, higher) ordering of
/// an unordered pair. All mutations and queries are `O(1)` average; the
/// per-agent scans ([`SocialGraph::edges_of`], [`SocialGraph::strongest_bond`])
/// are `O(degree)` in the agent's ego-network, which is small in practice.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SocialGraph {
    edges: HashMap<(AgentId, AgentId), SocialEdge>,
    /// Cumulative counters so callers can observe graph growth without
    /// iterating every edge.
    stats: SocialGraphStats,
}

/// Cheap cumulative counters, useful for telemetry and tests.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocialGraphStats {
    /// Number of times [`SocialGraph::apply_event`] *created* a new edge.
    pub edges_created: u64,
    /// Number of times [`SocialGraph::apply_event`] *updated* an existing edge.
    pub edges_updated: u64,
    /// Number of edges pruned because their weight fell to or below
    /// [`SocialGraphConfig::prune_below`].
    pub edges_pruned: u64,
}

impl SocialGraph {
    /// Empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of edges currently stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Whether the graph has no edges.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Cumulative counters.
    #[must_use]
    pub fn stats(&self) -> SocialGraphStats {
        self.stats
    }

    /// Canonical (lower, higher) ordering for an unordered pair.
    #[must_use]
    pub fn canonical(a: AgentId, b: AgentId) -> (AgentId, AgentId) {
        if a <= b {
            (a, b)
        } else {
            (b, a)
        }
    }

    /// Apply a single [`SocialEvent`] to the graph.
    ///
    /// Rules:
    /// - Self-events (`a == b`) are silently ignored.
    /// - The magnitude is multiplied by [`SocialEventKind::sign`].
    /// - If the resulting delta is non-positive *and* no edge exists, the
    ///   event is dropped: a single unprovoked conflict doesn't make two
    ///   strangers into grudging strangers. (A positive delta *does* create a
    ///   new edge, even with magnitude `0.25`.)
    /// - Otherwise the edge is created or updated in place.
    pub fn apply_event(&mut self, ev: SocialEvent, cfg: &SocialGraphConfig) -> ApplyOutcome {
        if ev.a == ev.b {
            return ApplyOutcome::NoOp;
        }
        let key = Self::canonical(ev.a, ev.b);
        let signed = ev.magnitude.max(0.0) * ev.kind.sign();

        // No edge yet.
        let Some(edge) = self.edges.get_mut(&key) else {
            if signed <= 0.0 {
                return ApplyOutcome::NoOp;
            }
            // Birth the edge with a small positive seed so future conflicts
            // can flip it into a grudge.
            self.edges.insert(
                key,
                SocialEdge {
                    a: key.0,
                    b: key.1,
                    weight: signed,
                    positive_events: 1,
                    negative_events: 0,
                    kinship_events: u32::from(ev.kind == SocialEventKind::Kinship),
                    last_touched_tick: ev.tick,
                },
            );
            self.stats.edges_created += 1;
            return ApplyOutcome::Created;
        };

        // Edge exists — update in place.
        let prev_weight = edge.weight;
        edge.weight += signed;
        edge.last_touched_tick = ev.tick;
        if signed >= 0.0 {
            edge.positive_events = edge.positive_events.saturating_add(1);
        } else {
            edge.negative_events = edge.negative_events.saturating_add(1);
        }
        if ev.kind == SocialEventKind::Kinship {
            edge.kinship_events = edge.kinship_events.saturating_add(1);
        }
        self.stats.edges_updated += 1;
        let new_weight = edge.weight;

        // Maybe prune if the weight has fallen below the noise floor.
        if edge.weight.abs() < cfg.prune_below {
            self.edges.remove(&key);
            self.stats.edges_pruned += 1;
        }

        ApplyOutcome::Updated {
            prev_weight,
            new_weight,
        }
    }

    /// Look up the edge between two agents, if any.
    #[must_use]
    pub fn edge(&self, a: AgentId, b: AgentId) -> Option<&SocialEdge> {
        self.edges.get(&Self::canonical(a, b))
    }

    /// All edges incident to `agent`, in canonical-pair form.
    #[must_use]
    pub fn edges_of(&self, agent: AgentId) -> Vec<SocialEdge> {
        self.edges
            .values()
            .filter(|e| e.a == agent || e.b == agent)
            .copied()
            .collect()
    }

    /// All edges of `agent` paired with the *other* endpoint id.
    #[must_use]
    pub fn neighbors_of(&self, agent: AgentId) -> Vec<(AgentId, SocialEdge)> {
        self.edges
            .values()
            .filter_map(|e| {
                if e.a == agent {
                    Some((e.b, *e))
                } else if e.b == agent {
                    Some((e.a, *e))
                } else {
                    None
                }
            })
            .collect()
    }

    /// The agent's strongest bond — the edge with the highest weight among
    /// `agent`'s incident edges, ties broken by the most recent
    /// `last_touched_tick` then by lower other-id for determinism.
    ///
    /// Returns `None` if the agent has no incident edges.
    #[must_use]
    pub fn strongest_bond(&self, agent: AgentId) -> Option<(AgentId, SocialEdge)> {
        self.neighbors_of(agent)
            .into_iter()
            .max_by(|(other_a, ea), (other_b, eb)| {
                ea.weight
                    .partial_cmp(&eb.weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| eb.last_touched_tick.cmp(&ea.last_touched_tick))
                    .then_with(|| other_a.cmp(other_b))
            })
    }

    /// The agent's strongest grudge — same tie-break as
    /// [`SocialGraph::strongest_bond`] but minimizing weight.
    #[must_use]
    pub fn strongest_grudge(&self, agent: AgentId) -> Option<(AgentId, SocialEdge)> {
        self.neighbors_of(agent)
            .into_iter()
            .min_by(|(other_a, ea), (other_b, eb)| {
                eb.weight
                    .partial_cmp(&ea.weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| eb.last_touched_tick.cmp(&ea.last_touched_tick))
                    .then_with(|| other_a.cmp(other_b))
            })
    }

    /// Age every edge by multiplying its weight by `factor` (a value in
    /// `(0.0, 1.0]`), then pruning any that fell below
    /// [`SocialGraphConfig::prune_below`].
    ///
    /// This is the "absence decays it" half of the acceptance test: edges
    /// that don't receive fresh events lose weight over time, and edges that
    /// go stale enough are removed entirely.
    ///
    /// Returns the number of edges pruned during this decay pass.
    pub fn decay(&mut self, cfg: &SocialGraphConfig, factor: f32) -> usize {
        let factor = factor.clamp(0.0, 1.0);
        let mut pruned = 0usize;
        self.edges.retain(|_, e| {
            e.weight *= factor;
            if e.weight.abs() < cfg.prune_below {
                pruned += 1;
                false
            } else {
                true
            }
        });
        self.stats.edges_pruned = self.stats.edges_pruned.saturating_add(pruned as u64);
        pruned
    }
}

/// Knobs the caller can tune per graph (or per simulation).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SocialGraphConfig {
    /// Edges whose `|weight|` falls below this value are pruned on decay and
    /// on update. Defaults to a tiny positive number so a single `0.25`
    /// co-location event survives one decay but eventually goes away.
    pub prune_below: f32,
}

impl Default for SocialGraphConfig {
    fn default() -> Self {
        Self { prune_below: 0.05 }
    }
}

impl SocialGraphConfig {
    /// "Default" config: edges whose absolute weight falls below `0.05` are
    /// pruned.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a config with a custom prune threshold.
    #[must_use]
    pub fn with_prune_below(prune_below: f32) -> Self {
        Self { prune_below }
    }
}

#[cfg(test)]
mod tests {
    //! Acceptance tests for FR-CIV-PSYCHE-910.
    //!
    //! Two things must hold (per the FR acceptance test):
    //!   1. An interaction increments an edge's weight.
    //!   2. Absence decays the weight (and eventually prunes the edge).
    //!   3. The graph can answer "what is agent X's strongest bond?".

    use super::*;

    fn cfg() -> SocialGraphConfig {
        // Tight threshold so the tests can demonstrate pruning without
        // waiting through a hundred decay passes.
        SocialGraphConfig::with_prune_below(0.05)
    }

    #[test]
    fn fr_civ_psyche_910_acceptance_interaction_increments_weight() {
        // Acceptance criterion (1): an interaction increments the edge weight.
        let mut g = SocialGraph::new();
        let outcome = g.apply_event(SocialEvent::interaction(1, 2, 1), &cfg());
        assert_eq!(outcome, ApplyOutcome::Created);
        let before = g.edge(1, 2).expect("edge should exist").weight;
        assert!(before > 0.0, "first interaction must produce a positive weight");

        // A second interaction updates (does not recreate) the edge.
        let outcome = g.apply_event(SocialEvent::interaction(2, 1, 2), &cfg());
        match outcome {
            ApplyOutcome::Updated { prev_weight, new_weight } => {
                assert!(new_weight > prev_weight, "weight must strictly increase");
            }
            other => panic!("expected Updated, got {other:?}"),
        }

        // Ordering of (a, b) doesn't matter — the graph canonicalizes.
        let e12 = g.edge(1, 2).unwrap();
        let e21 = g.edge(2, 1).unwrap();
        assert_eq!(e12.weight, e21.weight);
        assert_eq!(e12.positive_events, 2);
    }

    #[test]
    fn fr_civ_psyche_910_acceptance_absence_decays_weight() {
        // Acceptance criterion (2): absence decays the edge weight.
        let mut g = SocialGraph::new();
        g.apply_event(SocialEvent::interaction(1, 2, 1), &cfg());
        let start = g.edge(1, 2).unwrap().weight;

        // Half-life the weight a few times with no fresh events.
        for _ in 0..3 {
            g.decay(&cfg(), 0.5);
        }

        let after = g.edge(1, 2).map(|e| e.weight);
        match after {
            Some(w) => assert!(w < start, "decay must shrink the weight"),
            None => {} // Also acceptable: pruned entirely.
        }

        // Keep decaying until the edge is gone.
        for _ in 0..20 {
            g.decay(&cfg(), 0.5);
        }
        assert!(
            g.edge(1, 2).is_none(),
            "an edge that nobody interacts with for long enough must be pruned"
        );
    }

    #[test]
    fn fr_civ_psyche_910_acceptance_query_returns_strongest_bond() {
        // Acceptance criterion (3): query returns the agent's strongest bond.
        let mut g = SocialGraph::new();
        // Three contacts: 1-2, 1-3, 1-4. Make 1-3 the strongest.
        g.apply_event(SocialEvent::interaction(1, 2, 1), &cfg());
        g.apply_event(SocialEvent::interaction(1, 3, 1), &cfg());
        g.apply_event(SocialEvent::interaction(1, 3, 2), &cfg());
        g.apply_event(SocialEvent::interaction(1, 3, 3), &cfg());
        g.apply_event(SocialEvent::interaction(1, 4, 1), &cfg());
        // 1-2 gets a conflict so its weight drops below 1-3.
        g.apply_event(SocialEvent::conflict(1, 2, 4), &cfg());

        let (other, edge) = g
            .strongest_bond(1)
            .expect("agent 1 must have at least one bond");
        assert_eq!(other, 3, "agent 3 must be the strongest bond of agent 1");
        assert!(edge.weight > 0.0);

        // A grudge query on the same ego returns 2 (the negative edge).
        let (grudge_other, grudge_edge) = g
            .strongest_grudge(1)
            .expect("agent 1 must have at least one grudge");
        assert_eq!(grudge_other, 2);
        assert!(grudge_edge.weight < 0.0);
    }

    #[test]
    fn self_event_is_a_no_op() {
        // Self-loops must never create an edge.
        let mut g = SocialGraph::new();
        let outcome = g.apply_event(SocialEvent::interaction(7, 7, 1), &cfg());
        assert_eq!(outcome, ApplyOutcome::NoOp);
        assert!(g.is_empty());
    }

    #[test]
    fn kinship_creates_strong_bond_and_survives_decay() {
        // Kinship is heavier than contact and outlasts repeated decays.
        let mut g = SocialGraph::new();
        g.apply_event(SocialEvent::kinship(10, 11, 1), &cfg());
        let start = g.edge(10, 11).unwrap().weight;
        for _ in 0..5 {
            g.decay(&cfg(), 0.9);
        }
        let end = g.edge(10, 11).expect("kinship edge must survive light decay").weight;
        assert!(end > 0.0);
        assert!(end < start);
        assert_eq!(
            g.edge(10, 11).unwrap().relation(),
            RelationKind::Kinship,
            "kinship events must classify the edge as Kinship"
        );
    }

    #[test]
    fn repeated_conflict_turns_bond_into_grudge() {
        // Bonds can flip sign when hostile events outweigh cooperative ones.
        let mut g = SocialGraph::new();
        g.apply_event(SocialEvent::interaction(1, 2, 1), &cfg());
        for t in 2..6 {
            g.apply_event(SocialEvent::conflict(1, 2, t), &cfg());
        }
        let edge = g.edge(1, 2).expect("edge must still exist");
        assert!(edge.weight < 0.0, "weight must have flipped negative");
        assert_eq!(
            edge.relation(),
            RelationKind::Grudge,
            "negative weight with no kinship must classify as Grudge"
        );
    }

    #[test]
    fn strongest_bond_is_none_for_isolated_agent() {
        let g = SocialGraph::new();
        assert!(g.strongest_bond(42).is_none());
        assert!(g.strongest_grudge(42).is_none());
        assert!(g.edges_of(42).is_empty());
    }

    #[test]
    fn stats_track_creates_updates_and_prunes() {
        let mut g = SocialGraph::new();
        g.apply_event(SocialEvent::interaction(1, 2, 1), &cfg());
        g.apply_event(SocialEvent::interaction(1, 2, 2), &cfg());
        g.apply_event(SocialEvent::interaction(3, 4, 3), &cfg());

        let s = g.stats();
        assert_eq!(s.edges_created, 2);
        assert_eq!(s.edges_updated, 1);

        // Decay past the prune threshold.
        for _ in 0..10 {
            g.decay(&cfg(), 0.1);
        }
        assert!(g.is_empty(), "everything should have been pruned");
        assert!(g.stats().edges_pruned >= 2);
    }

    #[test]
    fn canonical_pair_is_order_independent() {
        assert_eq!(SocialGraph::canonical(5, 9), (5, 9));
        assert_eq!(SocialGraph::canonical(9, 5), (5, 9));
        assert_eq!(SocialGraph::canonical(0, 0), (0, 0));
    }
}