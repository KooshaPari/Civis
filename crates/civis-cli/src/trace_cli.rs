//! Trace-link inspector subcommand dispatch for the unified Civis binary.

use std::path::PathBuf;

use clap::Args;
use serde_json::json;

/// Arguments for civis trace.
#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct TraceArgs {
    /// Traceability document, replay, or link source to inspect.
    #[arg(long, default_value = "docs/traceability")]
    pub input: PathBuf,

    /// Optional link or requirement identifier to inspect.
    #[arg(long)]
    pub link: Option<String>,

    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

/// Run the trace-link inspector dispatch.
pub fn run(args: TraceArgs) -> Result<(), Box<dyn std::error::Error>> {
    let payload = json!({
        "command": "trace",
        "input": args.input,
        "link": args.link,
        "status": "dispatched",
    });

    if args.json {
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else if let Some(link) = args.link {
        println!(
            "civis trace: dispatched trace-link inspector for {} in {}",
            link,
            args.input.display()
        );
    } else {
        println!(
            "civis trace: dispatched trace-link inspector for {}",
            args.input.display()
        );
    }

    Ok(())
}
