//! Typed plan model for the master DAG.
//!
//! The [`Plan`] is the in-memory single-source-of-truth for the Civis godgame
//! roadmap. It is built once from [`crate::loader::load_from_markdown`] and
//! then handed to [`crate::wave::WaveExecutor`] for parallel dispatch.
//!
//! Every node carries an explicit `state` so the executor can compute
//! readiness, and every `Layer` carries `extendability_hooks` so the model
//! can be appended to indefinitely without rewriting earlier layers.
//!
//! ## Extension model
//!
//! Adding a new pillar / phase / lane never breaks the existing graph:
//! - New pillars add rows to [`crate::loader`]'s `§3` / `§4` tables.
//! - New lanes add rows to `§7`.
//! - New layers become a new `Layer` structure inserted at the end of
//!   `Plan::layers`, with auto-populated `extendability_hooks`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Identifier of a node inside the DAG. Opaque string kept as a typed
/// newtype so helpers can read more clearly at call-sites.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub String);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Lifecycle state of a single DAG node.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeState {
    Done,
    Merged,
    Verified,
    InFlight,
    Queued,
    Blocked,
    Stretch,
    Retrying,
    Failed,
}

impl fmt::Display for NodeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NodeState::Done => "Done",
            NodeState::Merged => "Merged",
            NodeState::Verified => "Verified",
            NodeState::InFlight => "InFlight",
            NodeState::Queued => "Queued",
            NodeState::Blocked => "Blocked",
            NodeState::Stretch => "Stretch",
            NodeState::Retrying => "Retrying",
            NodeState::Failed => "Failed",
        };
        f.write_str(s)
    }
}

/// Status state of the whole Plan. Used by the reporter to drive its
/// overall progress bar.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlanStatus {
    Loading,
    Active,
    Saturated,
    Done,
}

/// A typed estimate of how much work a node / the entire plan still has.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Eta {
    /// Total seconds we expect to spend on this node / plan.
    pub eta_seconds: u64,
    /// Seconds already spent.
    pub elapsed_seconds: u64,
    /// 0.0..=100.0 percent complete.
    pub percent: f32,
}

impl Eta {
    /// Construct a zero-progress ETA.
    pub fn zero() -> Self {
        Self {
            eta_seconds: 0,
            elapsed_seconds: 0,
            percent: 0.0,
        }
    }
}

/// A single DAG layer (e.g. P0, P1, ..., Pn).
///
/// `parallel_safe` indicates whether every node inside the layer may be
/// dispatched concurrently. `extendability_hooks` carry human-readable
/// extension recipes that the renderer surfaces to operators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub parallel_safe: bool,
    pub extendability_hooks: Vec<String>,
    pub nodes: Vec<NodeId>,
}

/// A single DAG node (a node == a concrete unit of work: a phase row, a
/// pillar epic, an open PR, a queued feature, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub layer: String,
    pub pr: Option<u32>,
    pub title: String,
    pub scope: String,
    pub state: NodeState,
    pub lane: String,
    pub deps: Vec<String>,
    pub fr_ref: Option<String>,
    pub adr_ref: Option<String>,
    pub eta: Option<Eta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<DateTime<Utc>>,
    pub summary: Option<String>,
    pub agent: Option<String>,
    pub task: Option<String>,
}

/// A parallel-safe dispatch lane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lane {
    pub id: String,
    pub purpose: String,
    pub scope: String,
    pub mutex: Option<String>,
    pub default_concurrency: usize,
    pub partition_by: Option<String>,
}

/// A single append-only tick entry. Agents self-report into the DAG
/// tick log on commit (`§8` of `CIVIS_GAME_DAG.md`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tick {
    pub id: String,
    pub merged: Vec<u32>,
    pub opened: Vec<u32>,
    pub closed_superseded: Vec<u32>,
    pub open_net: i32,
    pub notes: String,
    pub recorded_at: DateTime<Utc>,
}

/// A P3 pillar epic from `§3` of the source doc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PillarEpic {
    pub fr_id: String,
    pub name: String,
    pub lane: String,
    pub prs: Vec<u32>,
    pub adr: Option<String>,
}

/// A Pillar systems row from `§4` (QUEUED / partial pillars).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueuedPillar {
    pub fr_id: String,
    pub name: String,
    pub state: NodeState,
    pub eta_label: String,
    pub pr: Option<u32>,
    pub lane: String,
}

/// A deterministic-substrate phase from `§2`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Phase {
    pub ordinal: u8,
    pub name: String,
    pub dep: Option<String>,
    pub lane: String,
}

/// Metadata block that aggregates counts and source provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanMeta {
    pub total_nodes: usize,
    pub total_ticks: usize,
    pub last_updated: DateTime<Utc>,
    pub source_path: std::path::PathBuf,
}

/// Top-level container. Out-of-order field initializers would harm
/// deterministic JSON snapshots, so `Serialize`/`Deserialize` are derived.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub version: String,
    pub layers: Vec<Layer>,
    pub lanes: Vec<Lane>,
    pub ticks: Vec<Tick>,
    pub pillars: Vec<PillarEpic>,
    pub queued_pillars: Vec<QueuedPillar>,
    pub phases: Vec<Phase>,
    pub nodes: Vec<Node>,
    pub metadata: PlanMeta,
}

/// Errors raised by Plan-modelling helpers.
///
/// The loader wraps these into `anyhow::Error`; the rust API exposed to
/// callers inside the crate uses this typed enum directly.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DagError {
    #[error("invalid node id: {0}")]
    InvalidNodeId(String),
    #[error("cycle detected at node {0}")]
    CycleDetected(String),
    #[error("missing layer: {0}")]
    MissingLayer(String),
    #[error("duplicate node id: {0}")]
    DuplicateNodeId(String),
    #[error("node not found: {0}")]
    NodeNotFound(String),
}

impl Plan {
    /// Total node count.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Aggregate ETA across all known nodes.
    pub fn progress(&self) -> Eta {
        let mut eta_seconds: u64 = 0;
        let mut elapsed_seconds: u64 = 0;
        let mut done: u64 = 0;
        for n in &self.nodes {
            let total = estimated_total_for(n);
            eta_seconds = eta_seconds.saturating_add(total.0);
            elapsed_seconds = elapsed_seconds.saturating_add(total.1);
            if n.state == NodeState::Done {
                done = done.saturating_add(1);
            }
        }
        let percent = if eta_seconds == 0 {
            0.0
        } else {
            (elapsed_seconds as f32 / eta_seconds as f32) * 100.0
        };
        let _ = done; // reserved for future per-state-progress accounting
        Eta {
            eta_seconds,
            elapsed_seconds,
            percent,
        }
    }

    /// Filter nodes by lifecycle state.
    pub fn nodes_by_state(&self, state: NodeState) -> Vec<&Node> {
        self.nodes.iter().filter(|n| n.state == state).collect()
    }

    /// Nodes that are queued and whose deps have all reached `Done`.
    pub fn ready_nodes(&self) -> Vec<&Node> {
        let done_ids: std::collections::HashSet<&str> = self
            .nodes
            .iter()
            .filter(|n| n.state == NodeState::Done)
            .map(|n| n.id.as_str())
            .collect();
        self.nodes
            .iter()
            .filter(|n| n.state == NodeState::Queued)
            .filter(|n| n.deps.iter().all(|d| done_ids.contains(d.as_str())))
            .collect()
    }

    /// Compute the batched wave plan for parallel dispatch.
    ///
    /// Kahn-style topological waves: nodes with no deps form wave 0,
    /// nodes whose deps are all in earlier waves form later waves.
    /// The wave id is 0-indexed.
    pub fn waves(&self) -> Vec<Vec<NodeId>> {
        let mut waves: Vec<Vec<NodeId>> = Vec::new();
        if self.nodes.is_empty() {
            return waves;
        }

        // id -> node index for cheap lookup
        let mut placed: std::collections::HashMap<String, usize> =
            std::collections::HashMap::with_capacity(self.nodes.len());

        // remaining-deps: how many deps are still un-placed
        let mut remaining: Vec<usize> = vec![0; self.nodes.len()];
        for (i, n) in self.nodes.iter().enumerate() {
            remaining[i] = n.deps.len();
        }

        // for each dep un-place, bump the dependent node's remaining
        // (we just initialised above; the actual dependency-array is
        // also carried inside `deps` directly).

        // mutating vector of "still-un-emitted" indices
        let mut pending: Vec<usize> = (0..self.nodes.len()).collect();

        while !pending.is_empty() {
            let mut emit_ids: Vec<NodeId> = Vec::new();
            let mut next_pending: Vec<usize> = Vec::with_capacity(pending.len());
            for &i in &pending {
                let n = &self.nodes[i];
                let ready = n
                    .deps
                    .iter()
                    .all(|d| placed.contains_key(d.as_str()));
                if ready {
                    emit_ids.push(NodeId(n.id.clone()));
                } else {
                    next_pending.push(i);
                }
            }
            if emit_ids.is_empty() {
                // Cycle: give up so we never loop forever. The first
                // unresolved node is reported via DagError elsewhere.
                break;
            }
            // Commit pass: deferred insert so siblings in this same wave
            // do not see each other until after all are decided.
            let wave_idx = waves.len();
            // Owned-string keys so the HashMap owns its memory and we
            // don't have to leak strings via Box::leak.
            for id in emit_ids.iter().map(|n| n.0.clone()) {
                placed.insert(id, wave_idx);
            }
            waves.push(emit_ids);
            pending = next_pending;
        }
        waves
    }
    /// Mutable accessor for in-place extension.
    pub fn layer_mut(&mut self, id: &str) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    /// Look up a single node by id.
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }
}

impl Layer {
    /// Add a node to this layer and remember its id in `nodes`.
    ///
    /// Recomputes the layer-level ETA aggregate implicitly via the
    /// node's own `eta` field; call sites that need the new node to
    /// appear in `progress()` should also push the node onto
    /// `Plan::nodes` directly.
    pub fn add_node(&mut self, node: Node) -> Result<NodeId, DagError> {
        if node.id.trim().is_empty() {
            return Err(DagError::InvalidNodeId(node.id));
        }
        let id = NodeId::from(node.id.clone());
        self.nodes.push(id.clone());
        Ok(id)
    }
}

fn estimated_total_for(n: &Node) -> (u64, u64) {
    if let Some(eta) = &n.eta {
        (eta.eta_seconds, eta.elapsed_seconds)
    } else {
        match n.state {
            NodeState::Done => (1, 1),
            NodeState::Queued | NodeState::Stretch => (60, 0),
            _ => (30, 10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_node(id: &str, state: NodeState, deps: Vec<&str>) -> Node {
        Node {
            id: id.to_string(),
            layer: "P0".into(),
            pr: None,
            title: format!("title-{id}"),
            scope: "scope".into(),
            state,
            lane: "core".into(),
            deps: deps.into_iter().map(String::from).collect(),
            fr_ref: None,
            adr_ref: None,
            eta: None,
            started_at: None,
            finished_at: None,
            summary: None,
            agent: None,
            task: None,
        }
    }

    fn sample_plan() -> Plan {
        let nodes = vec![
            sample_node("a", NodeState::Done, vec![]),
            sample_node("b", NodeState::Done, vec!["a"]),
            sample_node("c", NodeState::Done, vec!["a"]),
            sample_node("d", NodeState::Queued, vec!["b", "c"]),
        ];
        Plan {
            id: "test".into(),
            version: "v0".into(),
            layers: vec![],
            lanes: vec![],
            ticks: vec![],
            pillars: vec![],
            queued_pillars: vec![],
            phases: vec![],
            nodes,
            metadata: PlanMeta {
                total_nodes: 4,
                total_ticks: 0,
                last_updated: Utc::now(),
                source_path: std::path::PathBuf::from("test.md"),
            },
        }
    }

    #[test]
    fn nodes_by_state_filters() {
        let p = sample_plan();
        let done = p.nodes_by_state(NodeState::Done);
        assert_eq!(done.len(), 3);
        let queued = p.nodes_by_state(NodeState::Queued);
        assert_eq!(queued.len(), 1);
    }

    #[test]
    fn ready_nodes_only_emits_when_deps_done() {
        let p = sample_plan();
        let ready = p.ready_nodes();
        let ids: Vec<_> = ready.iter().map(|n| n.id.as_str()).collect();
        assert_eq!(ids, vec!["d"]);
    }

    #[test]
    fn waves_partition_by_depth() {
        let p = sample_plan();
        let waves = p.waves();
        assert_eq!(waves.len(), 3, "a, b/c, d");
        assert_eq!(waves[0].len(), 1);
        assert!(waves[0].iter().any(|n| n.0 == "a"));
        assert!(waves[2].iter().any(|n| n.0 == "d"));
    }

    #[test]
    fn layer_add_node_tracks_id() {
        let mut l = Layer {
            id: "P0".into(),
            name: "Foundation".into(),
            scope: "core".into(),
            parallel_safe: true,
            extendability_hooks: vec![],
            nodes: vec![],
        };
        let id = l
            .add_node(sample_node("z", NodeState::Queued, vec![]))
            .expect("ok");
        assert_eq!(id.0, "z");
        assert_eq!(l.nodes.len(), 1);
    }

    #[test]
    fn layer_add_node_rejects_blank() {
        let mut l = Layer {
            id: "PX".into(),
            name: "x".into(),
            scope: "x".into(),
            parallel_safe: true,
            extendability_hooks: vec![],
            nodes: vec![],
        };
        let mut bad = sample_node("", NodeState::Queued, vec![]);
        bad.id = " ".into();
        let err = l.add_node(bad).unwrap_err();
        assert!(matches!(err, DagError::InvalidNodeId(_)));
    }

    #[test]
    fn progress_aggregates_etas() {
        let mut n1 = sample_node("a", NodeState::Done, vec![]);
        n1.eta = Some(Eta {
            eta_seconds: 100,
            elapsed_seconds: 100,
            percent: 100.0,
        });
        let mut n2 = sample_node("b", NodeState::Queued, vec![]);
        n2.eta = Some(Eta {
            eta_seconds: 50,
            elapsed_seconds: 0,
            percent: 0.0,
        });
        let mut p = sample_plan();
        p.nodes = vec![n1, n2];
        let eta = p.progress();
        assert_eq!(eta.eta_seconds, 150);
        assert_eq!(eta.elapsed_seconds, 100);
    }
}
