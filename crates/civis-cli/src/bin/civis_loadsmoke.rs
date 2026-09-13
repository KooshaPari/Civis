//! `civis-loadsmoke` — native load-and-continue smoke harness.
//!
//! Loads a `.civsave.zst` (or `.civsave/` directory) via `CivSaveBundle`,
//! ticks the engine N times offline, optionally re-saves, and prints a
//! JSON receipt to stdout. Designed for CI gates on "can the engine
//! still tick after a load".
//!
//! Usage:
//!   civis-loadsmoke --input <path> [--ticks N] [--save-out <path>]
//!
//! Exit codes:
//!   0 → receipt.passed == true
//!   1 → receipt.passed == false (load / save-out / IO error)
//!   2 → argument parse error

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use civis_cli::config::load_dotenv;
use civis_cli::loadsmoke::{run_loadsmoke, LoadSmokeOptions};

#[derive(Debug, Parser)]
#[command(
    name = "civis-loadsmoke",
    about = "Load a .civsave.zst, tick N times, emit a structured JSON receipt"
)]
struct Args {
    /// Path to a `.civsave.zst` archive or `.civsave/` directory.
    #[arg(long)]
    input: PathBuf,

    /// Number of ticks to advance after load. Default 3.
    #[arg(long, default_value_t = 3)]
    ticks: u32,

    /// Optional output archive path. If set, the post-load simulation
    /// is written here as a fresh `.civsave.zst`.
    #[arg(long)]
    save_out: Option<PathBuf>,
}

fn main() -> ExitCode {
    load_dotenv();

    let args = Args::parse();

    if !args.input.exists() {
        eprintln!(
            "civis-loadsmoke: input path does not exist: {}",
            args.input.display()
        );
        return ExitCode::from(2);
    }

    let options = LoadSmokeOptions {
        input_path: args.input,
        advanced_ticks: args.ticks,
        save_out: args.save_out,
        harness_version: civis_cli::HARNESS_VERSION.to_string(),
    };

    let receipt = run_loadsmoke(&options);

    let serialised = match serde_json::to_string_pretty(&receipt) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("civis-loadsmoke: failed to serialise receipt: {err}");
            return ExitCode::from(2);
        }
    };
    println!("{serialised}");

    if receipt.passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
