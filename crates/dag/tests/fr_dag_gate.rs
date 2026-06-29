//! Tests for the DAG §3 verification gate module (`crates/dag/src/gate.rs`).
//!
//! These tests cover the pure gate logic — the binary shell-out to
//! `cargo check` / `cargo test` is exercised by the integration suite, not
//! here. The unit tests assert the contract: which crates get gated for a
//! given node, what the JSON shape looks like, and how the design-only
//! lanes are skipped.

use chrono::Utc;
use civ_dag::gate::{
    extract_crate_from_scope, gates_for_all, lane_to_crate, reason_text, Gate, GateMode,
    SkipReason,
};
use civ_dag::model::{Node, NodeState, Plan, PlanMeta};

fn make_node(id: &str, lane: &str, scope: &str) -> Node {
    Node {
        id: id.to_string(),
        layer: "P0".into(),
        pr: None,
        title: format!("node-{id}"),
        scope: scope.to_string(),
        state: NodeState::Queued,
        lane: lane.to_string(),
        deps: vec![],
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

fn make_meta() -> PlanMeta {
    PlanMeta {
        total_nodes: 0,
        total_ticks: 0,
        last_updated: Utc::now(),
        source_path: std::path::PathBuf::from("plan.md"),
    }
}

fn make_plan(nodes: Vec<Node>) -> Plan {
    Plan {
        id: "test-plan".into(),
        version: "v1".into(),
        layers: vec![],
        lanes: vec![],
        ticks: vec![],
        pillars: vec![],
        queued_pillars: vec![],
        phases: vec![],
        nodes,
        metadata: make_meta(),
    }
}

#[test]
fn lane_to_crate_maps_known_lanes() {
    // The §3 contract: every node has a lane, every lane maps to one crate.
    // Unknown lanes must fall through to a Skip with UnknownLane so the
    // gate runner can emit a clean JSON entry instead of crashing.
    assert_eq!(lane_to_crate("core-sim"), Some("civ-engine".to_string()));
    assert_eq!(lane_to_crate("emergence"), Some("civ-engine".to_string()));
    assert_eq!(lane_to_crate("client"), Some("civ-bevy-ref".to_string()));
    assert_eq!(lane_to_crate("unknown-lane-xyz"), None);
}

#[test]
fn extract_crate_from_scope_picks_up_explicit_civ_prefix() {
    // A node's `scope` may name a crate directly even when its lane is
    // generic. The §3 contract: the explicit prefix wins.
    assert_eq!(
        extract_crate_from_scope("civ-protocol-3d: lifecycle hooks"),
        Some("civ-protocol-3d")
    );
    assert_eq!(
        extract_crate_from_scope("crates/voxel — fluid CA + boundary"),
        Some("crates/voxel")
    );
    assert_eq!(extract_crate_from_scope("docs only — design doc"), None);
}

#[test]
fn gates_for_all_returns_run_when_lane_resolves() {
    let node = make_node("n1", "core-sim", "core engine sim");
    let results = gates_for_all([&node], GateMode::Test);
    assert_eq!(results.len(), 1);
    let (id, gate) = &results[0];
    assert_eq!(id, "n1");
    match gate {
        Gate::Run { crate_name, .. } => assert_eq!(*crate_name, "civ-engine"),
        Gate::Skip { .. } => panic!("expected Run for core-sim lane"),
    }
}

#[test]
fn gates_for_all_returns_skip_when_lane_is_unknown() {
    let node = make_node("n2", "unknown-lane-xyz", "anything");
    let results = gates_for_all([&node], GateMode::Test);
    assert_eq!(results.len(), 1);
    let (id, gate) = &results[0];
    assert_eq!(id, "n2");
    match gate {
        Gate::Skip { reason: SkipReason::UnknownLane { lane } } => {
            assert_eq!(lane, "unknown-lane-xyz");
        }
        other => panic!("expected UnknownLane skip, got {other:?}"),
    }
}

#[test]
fn gates_for_all_runs_all_nodes_in_a_plan() {
    let nodes = vec![
        make_node("a", "core-sim", "engine"),
        make_node("b", "client", "ui"),
        make_node("c", "design-only", "docs"),
    ];
    let results = gates_for_all(nodes.iter(), GateMode::Check);
    assert_eq!(results.len(), 3);
    assert!(matches!(results[0].1, Gate::Run { .. }));
    assert!(matches!(results[1].1, Gate::Run { .. }));
    assert!(matches!(results[2].1, Gate::Skip { .. }));
}

#[test]
fn reason_text_returns_static_str_for_each_variant() {
    assert_eq!(reason_text(&SkipReason::UnknownLane { lane: "x".into() }), "unknown lane");
    assert_eq!(reason_text(&SkipReason::DesignOnly), "design-only lane");
}

#[test]
fn summary_json_emits_per_node_entries() {
    use civ_dag::gate::summary_json;
    let nodes = vec![make_node("p1", "core-sim", "x"), make_node("p2", "design-only", "y")];
    let results = gates_for_all(nodes.iter(), GateMode::Test);
    let v = summary_json(&results);
    let arr = v.as_array().expect("summary_json returns an array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["node_id"], "p1");
    assert_eq!(arr[1]["node_id"], "p2");
    assert_eq!(arr[0]["action"], "run");
    assert_eq!(arr[1]["action"], "skip");
}

#[test]
fn plan_constructs_with_metadata() {
    let nodes = vec![make_node("only", "core-sim", "x")];
    let plan = make_plan(nodes);
    assert_eq!(plan.id, "test-plan");
    assert_eq!(plan.version, "v1");
    assert_eq!(plan.nodes.len(), 1);
    assert_eq!(plan.nodes[0].id, "only");
    assert_eq!(plan.metadata.total_nodes, 0);
}
