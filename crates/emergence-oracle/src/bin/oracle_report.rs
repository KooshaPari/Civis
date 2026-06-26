//! oracle_report — FR-validation CI report for emergence-oracle.
//!
//! Runs a Simulation for 300 ticks then executes every registered FeatureOracle,
//! printing one line per verdict and a summary pass/fail count.
//!
//! Exit codes
//! ----------
//! 0 — passed_count >= BASELINE  (gate green)
//! 1 — passed_count <  BASELINE  (gate red — emergence regression detected)

use std::process;

use civ_engine::Simulation;
use emergence_oracle::OracleRegistry;

/// The number of FR contracts known to be passing.
/// Raise this constant as additional oracles are validated and must never regress.
/// Do NOT lower it — that would silently weaken the gate.
const BASELINE: usize = 8;

fn main() {
    let mut sim = Simulation::new();

    // Advance the simulation to give oracles meaningful data to inspect.
    for _ in 0..300 {
        sim.tick();
    }

    let registry = OracleRegistry::with_defaults();
    let verdicts = registry.run_all(&sim);
    let total = verdicts.len();

    for v in &verdicts {
        let status = if v.passed { "PASS" } else { "FAIL" };
        println!(
            "{}: {} measured={:.4} threshold={:.4} — {}",
            v.fr_id, status, v.measured, v.threshold, v.detail
        );
    }

    let passed = verdicts.iter().filter(|v| v.passed).count();
    println!();
    println!(
        "ORACLE: {passed}/{total} contracts passed (baseline={BASELINE})"
    );

    if passed < BASELINE {
        eprintln!(
            "ORACLE: GATE RED — only {passed} contracts pass; baseline requires {BASELINE}. \
             Emergence regression detected."
        );
        process::exit(1);
    }

    println!("ORACLE: GATE GREEN — baseline satisfied.");
}
