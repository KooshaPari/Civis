//! CLI entry for the civ-dag crate.
//!
//! Reads `CIVIS_GAME_DAG.md` (or a path passed as arg[1]) and prints the
//! sponsor-format tick report to stdout.
//!
//! The renderer is delegated to `civ_dag::reporter::ReportSnapshot::render_first()`
//! so all four sponsor-format sections are emitted:
//!   1. progress line + per-epic bars (elapsed + eta)
//!   2. DAG tree (waves × nodes, ASCII branches)
//!   3. agent | task | state | summary table
//!   4. next-steps bullets

use std::path::PathBuf;
use std::sync::Arc;

use civ_dag::loader::load_from_markdown;
use civ_dag::reporter::{RecorderReporter, Reporter};
use civ_dag::wave::{NodeReport, WaveExecutor};

/// Enable UTF-8 on the Windows console so the `█ ░ ├ └ ▶` glyphs in the
/// sponsor format render correctly under `cmd.exe` / `powershell.exe`.
/// No-op on non-Windows targets.
#[cfg(windows)]
fn enable_utf8_console() {
    extern "system" {
        fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
    }
    // CP_UTF8 = 65001
    unsafe { SetConsoleOutputCP(65001); }
}

#[cfg(not(windows))]
fn enable_utf8_console() {}

fn fallback_plan(path: PathBuf) -> civ_dag::model::Plan {
    civ_dag::model::Plan {
        id: "demo".into(),
        version: "0".into(),
        layers: Vec::new(),
        lanes: Vec::new(),
        ticks: Vec::new(),
        pillars: Vec::new(),
        queued_pillars: Vec::new(),
        phases: Vec::new(),
        nodes: Vec::new(),
        metadata: civ_dag::model::PlanMeta {
            total_nodes: 0,
            total_ticks: 0,
            last_updated: chrono::Utc::now(),
            source_path: path,
        },
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_utf8_console();

    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("CIVIS_GAME_DAG.md"));

    println!("civ-dag · loading {}\n", path.display());

    let plan = match std::fs::read_to_string(&path) {
        Ok(src) => match load_from_markdown(&src, path.clone()) {
            Ok(p) => Arc::new(p),
            Err(e) => {
                eprintln!("warning: could not parse {}: {e}", path.display());
                Arc::new(fallback_plan(path.clone()))
            }
        }
        Err(e) => {
            eprintln!("warning: cannot read {}: {e}", path.display());
            Arc::new(fallback_plan(path.clone()))
        }
    };

    let reporter = Arc::new(RecorderReporter::default());
    let executor = WaveExecutor::new(Arc::clone(&plan));

    let _reports: Vec<NodeReport> = match executor.run_all().await {
        Ok(rs) => rs,
        Err(e) => {
            eprintln!("run_all halted: {e}");
            Vec::new()
        }
    };

    // Delegate to the canonical reporter renderer so we always emit the
    // full sponsor format (progress + DAG + agents + next-steps).
    let snap = reporter.snapshot(&plan);
    let rendered = snap
        .render_first()
        .unwrap_or_else(|e| format!("(render error: {e})"));
    print!("{}", rendered);
    Ok(())
}
