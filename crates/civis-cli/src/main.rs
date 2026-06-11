//! Unified Civis CLI dispatch.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use serde_json::json;

mod analyzer_cli;
mod trace_cli;

/// Unified command-line entrypoint for Civis workspace tooling.
#[derive(Debug, Parser)]
#[command(name = "civis", version, about = "Unified Civis CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize Civis workspace metadata.
    Init(InitArgs),
    /// Run the analyzer.
    Analyze(analyzer_cli::AnalyzeArgs),
    /// Inspect traceability links.
    Trace(trace_cli::TraceArgs),
}

#[derive(Debug, Clone, clap::Args, PartialEq, Eq)]
struct InitArgs {
    /// Workspace directory to initialize.
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,
}

fn main() {
    civis_cli::config::load_dotenv();

    if let Err(err) = run(Cli::parse()) {
        eprintln!("civis: {err}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Init(args) => run_init(args),
        Commands::Analyze(args) => analyzer_cli::run(args),
        Commands::Trace(args) => trace_cli::run(args),
    }
}

fn run_init(args: InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    let metadata_dir = args.path.join(".civis");
    fs::create_dir_all(&metadata_dir)?;

    let manifest_path = metadata_dir.join("workspace.json");
    if !manifest_path.exists() {
        let manifest = json!({
            "schema": "civis.workspace.v1",
            "workspace": normalized_display(&args.path),
        });
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest)? + "\n",
        )?;
    }

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "command": "init",
                "metadata_dir": metadata_dir,
                "manifest": manifest_path,
                "status": "ok",
            }))?
        );
    } else {
        println!("civis init: initialized {}", metadata_dir.display());
    }

    Ok(())
}

fn normalized_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_analyze_subcommand_args() {
        let cli = Cli::try_parse_from(["civis", "analyze", "--input", "snapshot.json", "--json"])
            .expect("valid analyze args");

        match cli.command {
            Commands::Analyze(args) => {
                assert_eq!(args.input, PathBuf::from("snapshot.json"));
                assert!(args.json);
            }
            other => panic!("expected analyze command, got {other:?}"),
        }
    }

    #[test]
    fn parses_trace_subcommand_args() {
        let cli = Cli::try_parse_from([
            "civis",
            "trace",
            "--input",
            "docs/traceability",
            "--link",
            "FR-CIV-001",
        ])
        .expect("valid trace args");

        match cli.command {
            Commands::Trace(args) => {
                assert_eq!(args.input, PathBuf::from("docs/traceability"));
                assert_eq!(args.link.as_deref(), Some("FR-CIV-001"));
                assert!(!args.json);
            }
            other => panic!("expected trace command, got {other:?}"),
        }
    }
}
