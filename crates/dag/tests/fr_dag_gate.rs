//! Tests for the §3 verification gate runner.
//!
//! These are pure-logic tests — they exercise `gate::gate_for` and the JSON
//! summary without spawning cargo. The actual subprocess execution lives
//! in `src/bin/civ-dag-gate.rs` and is covered by the workflow test in
//! `.github/workflows/dag-gate.yml`.

use civ_dag::gate::{
    extract_crate_from_scope, gate_for, gates_for_all, lane_to_crate, summary_json, Gate, GateMode, SkipReason,
};
use civ_dag::model::Node;

fn node(id: &str, lane: &str, scope: &str) -> Node {
    Node {
        id: id.to_string(),
        title: format!("Node {}", id),
        lane: lane.to_string(),
        scope: scope.to_string(),
        deps: vec![],
        fr_ref: None,
        node_kind: civ_dag::model::NodeKind::Code,
        body: String::new(),
        meta: Default::default(),
    }
}

#[test]
fn lane_to_crate_resolves_known_lanes() {
    assert_eq!(lane_to_crate("core-sim"), Some("civ_engine"));
    assert_eq!(lane_to_crate("emergence-economy"), Some("civ_economy"));
    assert_eq!(lane_to_crate("godtools-holocron"), Some("civ_holocron"));
    assert_eq!(lane_to_crate("client-shell"), Some("bevy_ref"));
    assert_eq!(lane_to_crate("meta-dag"), Some("civ_dag"));
}

#[test]
fn lane_to_crate_returns_none_for_unknown() {
    assert_eq!(lane_to_crate("not-a-real-lane"), None);
    assert_eq!(lane_to_crate(""), None);
}

#[test]
fn scope_override_beats_lane_default() {
    let n = node("N1", "core-sim", "crates/civ_holocron");
    match gate_for(&n, GateMode::Check) {
        Gate::Run { crate_name, why, .. } => {
            assert_eq!(crate_name, "civ_holocron");
            assert!(why.contains("scope"));
        }
        Gate::Skip { .. } => panic!("expected Run for explicit scope"),
    }
}

#[test]
fn empty_scope_falls_back_to_lane_default() {
    let n = node("N1", "emergence-economy", "");
    match gate_for(&n, GateMode::Test) {
        Gate::Run { crate_name, mode, .. } => {
            assert_eq!(crate_name, "civ_economy");
            assert_eq!(mode, GateMode::Test);
        }
        Gate::Skip { .. } => panic!("expected Run via lane default"),
    }
}

#[test]
fn dash_scope_with_unknown_lane_skips() {
    let n = node("N1", "design-only", "-");
    match gate_for(&n, GateMode::Check) {
        Gate::Skip { reason: SkipReason::UnknownLane { lane } } => {
            assert_eq!(lane, "design-only");
        }
        other => panic!("expected UnknownLane skip, got {:?}", other),
    }
}

#[test]
fn docs_design_scope_skips_as_design_only() {
    let n = node("N1", "core-sim", "docs/design/RELIGION_EMERGENCE.md");
    match gate_for(&n, GateMode::Check) {
        Gate::Skip { reason: SkipReason::DesignOnly } => {}
        other => panic!("expected DesignOnly skip, got {:?}", other),
    }
}

#[test]
fn extract_crate_from_scope_handles_multiple_tokens() {
    assert_eq!(extract_crate_from_scope("crates/civ_dag + clients/bevy-ref"), Some("civ_dag"));
    assert_eq!(extract_crate_from_scope("touches civ-emergence-metrics and civ-server"), Some("civ_emergence_metrics"));
    assert_eq!(extract_crate_from_scope(""), None);
    assert_eq!(extract_crate_from_scope("-"), None);
    assert_eq!(extract_crate_from_scope("design-only"), None);
    assert_eq!(extract_crate_from_scope("docs/adr/039.md"), None);
}

#[test]
fn gates_for_all_resolves_every_node_in_plan_order() {
    let ns = vec![
        node("A", "core-sim", ""),
        node("B", "emergence-economy", "civ-needs"),
        node("C", "not-a-real-lane", ""),
    ];
    let g = gates_for_all(ns.iter(), GateMode::Check);
    assert_eq!(g.len(), 3);
    assert_eq!(g[0].0, "A");
    assert!(matches!(g[0].1, Gate::Run { ref crate_name, .. } if crate_name == "civ_engine"));
    assert_eq!(g[1].0, "B");
    assert!(matches!(g[1].1, Gate::Run { ref crate_name, .. } if crate_name == "civ_needs"));
    assert_eq!(g[2].0, "C");
    assert!(matches!(g[2].1, Gate::Skip { .. }));
}

#[test]
fn summary_json_emits_stable_schema_with_totals() {
    let ns = vec![
        node("A", "core-sim", ""),
        node("B", "unknown", ""),
    ];
    let g = gates_for_all(ns.iter(), GateMode::Test);
    let s = summary_json(&g);
    assert_eq!(s["schema"], "civ-dag.gate-summary/v1");
    assert_eq!(s["totals"]["run"], 1);
    assert_eq!(s["totals"]["skip"], 1);
    assert_eq!(s["gates"][0]["action"], "run");
    assert_eq!(s["gates"][0]["mode"], "test");
    assert_eq!(s["gates"][1]["action"], "skip");
    assert_eq!(s["gates"][1]["reason"], "unknown_lane");
}
