//! §3 Verification Gate runner for the Master DAG plan.
//!
//! The Master DAG plan (`plans/2026-06-26-civis-master-dag-v1.md` §3) specifies
//! per-node verification gates. This module turns a `Node` from a loaded plan
//! into a runnable `Gate` that:
//!
//! 1. Inspects the node's `scope` field for an explicit `civ-` crate hint.
//! 2. Falls back to a `lane`-to-crate mapping table for known lanes.
//! 3. Emits a `GateSpec` (crate, mode, why-skipped) that the binary can execute.
//!
//! The actual `cargo check` / `cargo test` subprocess invocation lives in the
//! `civ-dag-gate` binary (see `src/bin/civ-dag-gate.rs`). This module is pure
//! logic so it can be unit-tested without spawning cargo.

use std::collections::BTreeMap;

use crate::model::Node;

/// What a gate execution does to a single crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateMode {
    /// `cargo check -p <crate>`
    Check,
    /// `cargo test -p <crate>` (includes build + check + test)
    Test,
}

/// Why a node was skipped (no runnable gate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// `scope` is empty and `lane` is not in the known map.
    UnknownLane { lane: String },
    /// `scope` was an explicit `-` / "no-op" / "design-only" indicator.
    DesignOnly,
}

/// A resolved verification gate for a single plan node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    /// Run a cargo command on `crate`.
    Run { crate_name: String, mode: GateMode, why: String },
    /// Skip the node — `reason` says why.
    Skip { reason: SkipReason },
}

/// The known `lane -> crate` mapping. Lanes not listed here are considered
/// design-only and produce a `Skip::UnknownLane` unless `scope` overrides.
///
/// Keep this table small and explicit. Adding a new entry here is the
/// extension point — no new code, no new branches, no new permissions.
pub fn lane_to_crate(lane: &str) -> Option<&'static str> {
    match lane {
        // Lane A — Core sim
        "core-sim" => Some("civ_engine"),
        "core-physics" => Some("civ_physics_substrate"),
        "core-planet" => Some("civ_planet"),
        "core-voxel" => Some("civ_voxel"),
        // Lane B — Emergence
        "emergence-economy" => Some("civ_economy"),
        "emergence-needs" => Some("civ_needs"),
        "emergence-agents" => Some("civ_agents"),
        "emergence-engine" => Some("civ_engine"),
        "emergence-disasters" => Some("civ_emergence_metrics"),
        // Lane C — God-tools
        "godtools-mcp" => Some("civ_server"),
        "godtools-verbs" => Some("civ_server"),
        "godtools-holocron" => Some("civ_holocron"),
        // Lane D — Client / UX
        "client-shell" => Some("bevy_ref"),
        "client-render" => Some("bevy_ref"),
        "client-mcp" => Some("biv_z"),
        // Lane E — Meta
        "meta-dag" => Some("civ_dag"),
        "meta-trace" => Some("civ_dag"),
        _ => None,
    }
}

/// Heuristic for whether `scope` explicitly indicates a crate, irrespective of
/// `lane`. Patterns we accept: `civ-foo`, `crates/foo`, `civ_dag`, and bare
/// crate names that exist in the workspace.
pub fn extract_crate_from_scope(scope: &str) -> Option<&'static str> {
    let s = scope.trim();
    if s.is_empty() || s == "-" || s.starts_with("design") || s.starts_with("docs/") {
        return None;
    }
    // Look for a `civ-` or `crates/` token in the scope string.
    for tok in s.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_') {
        let t = tok.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
        if t.starts_with("civ-") {
            return Some(BOX_CRATE_MAP
                .iter()
                .find(|(k, _)| *k == t)
                .map(|(_, v)| *v)
                .unwrap_or(""));
        }
        if let Some(v) = BOX_CRATE_MAP.iter().find(|(k, _)| *k == t) {
            return Some(v.1);
        }
    }
    None
}

/// Const-evaluable crate-name lookup. We can't use a `match` here because the
/// keys are mixed case (civ-dag, civ_emergence_metrics) — we keep an
/// explicit slice.
const BOX_CRATE_MAP: &[(&str, &str)] = &[
    ("civ-dag", "civ_dag"),
    ("civ-engine", "civ_engine"),
    ("civ-economy", "civ_economy"),
    ("civ-needs", "civ_needs"),
    ("civ-agents", "civ_agents"),
    ("civ-planet", "civ_planet"),
    ("civ-voxel", "civ_voxel"),
    ("civ-physics-substrate", "civ_physics_substrate"),
    ("civ-emergence-metrics", "civ_emergence_metrics"),
    ("civ-server", "civ_server"),
    ("civ-laws", "civ_laws"),
    ("civ-legends", "civ_legends"),
    ("civ-holocron", "civ_holocron"),
    ("civ-protocol-3d", "civ_protocol_3d"),
    ("civ-byteport", "civ_byteport"),
    ("civ-vision-2d", "civ_vision_2d"),
    ("civ-federation", "civ_federation"),
    ("civ-ai", "civ_ai"),
    ("civ-budget", "civ_budget"),
    ("bevy-ref", "bevy_ref"),
    ("biv-z", "biv_z"),
];

/// Resolve a `Node` to a `Gate`. The `mode` is the action the runner wants
/// to take — `Check` for fast lanes, `Test` for evidence-emitting nodes.
pub fn gate_for(node: &Node, mode: GateMode) -> Gate {
    // 1. Explicit scope override.
    if let Some(crate_name) = extract_crate_from_scope(&node.scope) {
        return Gate::Run {
            crate_name: crate_name.to_string(),
            mode,
            why: format!("scope='{}' contains '{}'", node.scope, crate_name),
        };
    }
    // 2. Lane-based default.
    if let Some(crate_name) = lane_to_crate(&node.lane) {
        return Gate::Run {
            crate_name: crate_name.to_string(),
            mode,
            why: format!("lane='{}' maps to '{}'", node.lane, crate_name),
        };
    }
    // 3. Lane is unknown and scope is empty — skip with a clear reason.
    if node.scope.trim().is_empty() || node.scope.trim() == "-" {
        return Gate::Skip {
            reason: SkipReason::UnknownLane { lane: node.lane.clone() },
        };
    }
    // 4. Scope exists but is design-only (e.g. "docs/design/FOO.md").
    Gate::Skip {
        reason: SkipReason::DesignOnly,
    }
}

/// Convenience: run `gate_for` over every node in a plan and collect the
/// resolved gates in plan order. Returns `(node_id, Gate)` pairs so the
/// reporter can format node_id alongside the result.
pub fn gates_for_all<'a>(nodes: impl IntoIterator<Item = &'a Node>, mode: GateMode) -> Vec<(String, Gate)> {
    nodes
        .into_iter()
        .map(|n| (n.id.clone(), gate_for(n, mode)))
        .collect()
}

/// Emit a JSON-shaped summary of the resolved gates for a plan. Stable
/// schema so the CI workflow (`.github/workflows/dag-gate.yml`) can diff
/// across runs.
pub fn summary_json(node_gates: &[(String, Gate)]) -> serde_json::Value {
    let runs: Vec<_> = node_gates
        .iter()
        .filter_map(|(id, g)| match g {
            Gate::Run { crate_name, mode, why } => Some(serde_json::json!({
                "node_id": id,
                "action": "run",
                "crate": crate_name,
                "mode": match mode {
                    GateMode::Check => "check",
                    GateMode::Test => "test",
                },
                "why": why,
            })),
            Gate::Skip { reason } => Some(serde_json::json!({
                "node_id": id,
                "action": "skip",
                "reason": reason_text(reason),
            })),
        })
        .collect();
    let n_runs = runs.iter().filter(|r| r["action"] == "run").count();
    let n_skips = runs.iter().filter(|r| r["action"] == "skip").count();
    serde_json::json!({
        "schema": "civ-dag.gate-summary/v1",
        "totals": {
            "run": n_runs,
            "skip": n_skips,
        },
        "gates": runs,
    })
}

fn reason_text(r: &SkipReason) -> &'static str {
    match r {
        SkipReason::UnknownLane { .. } => "unknown_lane",
        SkipReason::DesignOnly => "design_only",
    }
}

/// Build a stable, test-friendly `BTreeMap` of lane -> crate overrides. Used
/// by callers that want a non-static lookup (e.g. JSON-driven config).
pub fn lane_map() -> BTreeMap<&'static str, &'static str> {
    let entries: [(&'static str, &'static str); 22] = [
        ("core-sim", "civ_engine"),
        ("core-physics", "civ_physics_substrate"),
        ("core-planet", "civ_planet"),
        ("core-voxel", "civ_voxel"),
        ("emergence-economy", "civ_economy"),
        ("emergence-needs", "civ_needs"),
        ("emergence-agents", "civ_agents"),
        ("emergence-engine", "civ_engine"),
        ("emergence-disasters", "civ_emergence_metrics"),
        ("godtools-mcp", "civ_server"),
        ("godtools-verbs", "civ_server"),
        ("godtools-holocron", "civ_holocron"),
        ("client-shell", "bevy_ref"),
        ("client-render", "bevy_ref"),
        ("client-mcp", "biv_z"),
        ("meta-dag", "civ_dag"),
        ("meta-trace", "civ_dag"),
        ("core-build", "civ_build"),
        ("core-needs", "civ_needs"),
        ("emergence-mood", "civ_emergence_metrics"),
        ("godtools-substrate", "civ_holocron"),
        ("meta-ci", "civ_dag"),
    ];
    entries.into_iter().collect()
}
