//! Analyzer subcommand dispatch for the unified Civis binary.

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

/// Arguments for civis analyze.
#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct AnalyzeArgs {
    /// Workspace or snapshot path to analyze.
    #[arg(long, default_value = ".")]
    pub input: PathBuf,

    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

/// Run the analyzer dispatch.
pub fn run(args: AnalyzeArgs) -> Result<(), Box<dyn std::error::Error>> {
    let payload = json!({
        "command": "analyze",
        "input": args.input,
        "status": "dispatched",
    });

    if args.json {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        println!(
            "civis analyze: dispatched analyzer for {}",
            args.input.display()
        );
    }

    Ok(())
}
