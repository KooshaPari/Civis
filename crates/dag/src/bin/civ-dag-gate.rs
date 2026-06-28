//! `civ-dag-gate` — CI-friendly runner for the Master DAG plan §3 verification gates.
//!
//! Usage:
//!     civ-dag-gate <plan.md> [--mode check|test] [--node <node_id>] [--json]
//!
//! Behavior:
//!   1. Loads the plan via `civ_dag::loader`.
//!   2. Resolves each node to a `Gate` via `civ_dag::gate::gate_for`.
//!   3. Executes the cargo command for each Run gate.
//!   4. Emits a JSON report (`--json`) or a human-readable table (default).
//!
//! Exit code is 0 if all Run gates pass, non-zero if any fail. Skip gates
//! never fail the run.

use std::collections::HashMap;
use std::io::{self, Write};
use std::process::{Command, Stdio};

use civ_dag::gate::{gate_for, summary_json, Gate, GateMode, SkipReason};
use civ_dag::loader::load_plan;
use civ_dag::model::Plan;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let parsed = match parse_args(&args) {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("civ-dag-gate: {}", msg);
            std::process::exit(2);
        }
    };

    let plan = match load_plan(&parsed.plan_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("civ-dag-gate: failed to load plan {}: {}", parsed.plan_path.display(), e);
            std::process::exit(2);
        }
    };

    let selected = select_nodes(&plan, &parsed);
    let mut results: Vec<(String, GateOutcome)> = Vec::new();
    for node in &selected {
        let gate = gate_for(node, parsed.mode);
        let outcome = match &gate {
            Gate::Run { crate_name, mode, .. } => {
                let cmd = cargo_command(*crate_name, *mode);
                run_cargo(cmd)
            }
            Gate::Skip { reason } => GateOutcome::Skip(reason_text(reason)),
        };
        results.push((node.id.clone(), outcome));
    }

    if parsed.json {
        emit_json(&results);
    } else {
        emit_table(&results);
    }

    let any_fail = results
        .iter()
        .any(|(_, o)| matches!(o, GateOutcome::Fail(_, _)));
    std::process::exit(if any_fail { 1 } else { 0 });
}

#[derive(Debug)]
struct Args {
    plan_path: std::path::PathBuf,
    mode: GateMode,
    node_filter: Option<String>,
    json: bool,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut plan_path: Option<std::path::PathBuf> = None;
    let mut mode = GateMode::Check;
    let mut node_filter: Option<String> = None;
    let mut json = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--mode" => {
                i += 1;
                mode = match args.get(i).map(String::as_str) {
                    Some("check") => GateMode::Check,
                    Some("test") => GateMode::Test,
                    other => return Err(format!("--mode expects 'check' or 'test', got {:?}", other)),
                };
            }
            "--node" => {
                i += 1;
                node_filter = args.get(i).cloned();
            }
            "--json" => json = true,
            arg if plan_path.is_none() => plan_path = Some(std::path::PathBuf::from(arg)),
            other => return Err(format!("unexpected argument: {}", other)),
        }
        i += 1;
    }
    let plan_path = plan_path.ok_or_else(|| "missing <plan.md>".to_string())?;
    Ok(Args { plan_path, mode, node_filter, json })
}

fn select_nodes<'a>(plan: &'a Plan, args: &Args) -> Vec<&'a civ_dag::model::Node> {
    plan.nodes
        .iter()
        .filter(|n| args.node_filter.as_ref().map_or(true, |f| &n.id == f))
        .collect()
}

fn cargo_command(crate_name: &str, mode: GateMode) -> Vec<String> {
    let mut cmd = vec!["cargo".to_string(), match mode {
        GateMode::Check => "check".to_string(),
        GateMode::Test => "test".to_string(),
    }];
    cmd.extend(["-p".to_string(), crate_name.to_string(), "--message-format".to_string(), "short".to_string()]);
    cmd
}

fn run_cargo(cmd: Vec<String>) -> GateOutcome {
    let mut child = match Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return GateOutcome::Fail(cmd.clone(), format!("spawn failed: {}", e)),
    };
    match child.wait_with_output() {
        Ok(out) if out.status.success() => GateOutcome::Pass(cmd.clone(), String::from_utf8_lossy(&out.stdout).into_owned()),
        Ok(out) => GateOutcome::Fail(cmd.clone(), String::from_utf8_lossy(&out.stderr).into_owned()),
        Err(e) => GateOutcome::Fail(cmd.clone(), format!("wait failed: {}", e)),
    }
}

#[derive(Debug, Clone)]
enum GateOutcome {
    Pass(Vec<String>, String),
    Fail(Vec<String>, String),
    Skip(&'static str),
}

fn reason_text(r: &SkipReason) -> &'static str {
    match r {
        SkipReason::UnknownLane { .. } => "unknown_lane",
        SkipReason::DesignOnly => "design_only",
    }
}

fn emit_json(results: &[(String, GateOutcome)]) {
    let runs: Vec<serde_json::Value> = results
        .iter()
        .map(|(id, o)| match o {
            GateOutcome::Pass(cmd, _) => serde_json::json!({"node_id": id, "result": "pass", "cmd": cmd}),
            GateOutcome::Fail(cmd, _) => serde_json::json!({"node_id": id, "result": "fail", "cmd": cmd}),
            GateOutcome::Skip(why) => serde_json::json!({"node_id": id, "result": "skip", "why": why}),
        })
        .collect();
    let n_pass = runs.iter().filter(|r| r["result"] == "pass").count();
    let n_fail = runs.iter().filter(|r| r["result"] == "fail").count();
    let n_skip = runs.iter().filter(|r| r["result"] == "skip").count();
    let report = serde_json::json!({
        "schema": "civ-dag.gate-runner/v1",
        "totals": { "pass": n_pass, "fail": n_fail, "skip": n_skip },
        "results": runs,
    });
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "{}", serde_json::to_string_pretty(&report).unwrap_or_default());
}

fn emit_table(results: &[(String, GateOutcome)]) {
    let n_pass = results.iter().filter(|(_, o)| matches!(o, GateOutcome::Pass(_, _))).count();
    let n_fail = results.iter().filter(|(_, o)| matches!(o, GateOutcome::Fail(_, _))).count();
    let n_skip = results.iter().filter(|(_, o)| matches!(o, GateOutcome::Skip(_))).count();
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "civ-dag-gate: {} nodes ({} pass, {} fail, {} skip)", results.len(), n_pass, n_fail, n_skip);
    for (id, o) in results {
        let status = match o {
            GateOutcome::Pass(_, _) => "PASS",
            GateOutcome::Fail(_, _) => "FAIL",
            GateOutcome::Skip(_) => "SKIP",
        };
        let _ = writeln!(handle, "  [{}] {}", status, id);
    }
}
