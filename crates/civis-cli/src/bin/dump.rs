//! Validate or diff CIVIS_DUMP render-frame JSON against policy/baseline.

use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use civis_cli::config::{dump_policy_name_from_env, load_dotenv};
use civis_cli::dump::{
    compare_to_baseline, extract_dump_from_markers, parse_dump_json, validate_dump, DumpError,
    DumpPolicy, DumpTolerance,
};
use clap::{Parser, Subcommand};
use serde_json::json;

#[derive(Debug, Parser)]
#[command(
    name = "civis-dump",
    about = "CIVIS_DUMP render-frame regression — validate scene+sim JSON"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run invariant gates (`headless` or `headful` policy).
    Check {
        /// Dump JSON file (or `-` for stdin).
        input: PathBuf,
        /// Policy preset: `headless` (default) or `headful`.
        #[arg(long, default_value = "headless")]
        policy: String,
        /// When set, input is stdout text with CIVIS_DUMP markers.
        #[arg(long)]
        from_markers: bool,
    },
    /// Diff actual dump against a committed baseline fixture.
    Diff {
        /// Baseline JSON path.
        baseline: PathBuf,
        /// Actual dump JSON path (or `-` for stdin).
        actual: PathBuf,
        /// Absolute tolerance for resource floats.
        #[arg(long, default_value_t = 1.0)]
        resource_tol: f64,
        /// Absolute tolerance for placement/bounds floats.
        #[arg(long, default_value_t = 0.5)]
        placement_tol: f64,
        /// Allowed integer count delta.
        #[arg(long, default_value_t = 0)]
        count_delta: u64,
        /// When set, actual input uses CIVIS_DUMP stdout markers.
        #[arg(long)]
        from_markers: bool,
    },
    /// Extract JSON from marker-wrapped stdout to plain JSON on stdout.
    Extract {
        /// Input file (default stdin).
        #[arg(default_value = "-")]
        input: PathBuf,
    },
}

fn main() -> ExitCode {
    load_dotenv();
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("civis-dump: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), DumpError> {
    match cli.command {
        Command::Check {
            input,
            policy,
            from_markers,
        } => {
            let dump = read_dump(&input, from_markers)?;
            let policy = resolve_policy(&policy)?;
            let report = validate_dump(&dump, &policy).with_policy(&policy_name(&policy));
            emit_report(&report)
        }
        Command::Diff {
            baseline,
            actual,
            resource_tol,
            placement_tol,
            count_delta,
            from_markers,
        } => {
            let baseline_dump = read_dump_file(&baseline)?;
            let actual_dump = read_dump(&actual, from_markers)?;
            let tolerance = DumpTolerance {
                resource_abs: resource_tol,
                placement_abs: placement_tol,
                count_delta,
            };
            let label = baseline.to_string_lossy().into_owned();
            let report = compare_to_baseline(&actual_dump, &baseline_dump, &tolerance)
                .with_baseline(label);
            emit_report(&report)
        }
        Command::Extract { input } => {
            let text = read_text(&input)?;
            let dump = extract_dump_from_markers(&text)?;
            let json = serde_json::to_string_pretty(&dump).map_err(DumpError::Decode)?;
            println!("{json}");
            Ok(())
        }
    }
}

fn resolve_policy(name: &str) -> Result<DumpPolicy, DumpError> {
    if let Some(env_name) = dump_policy_name_from_env() {
        return DumpPolicy::from_name(&env_name);
    }
    DumpPolicy::from_name(name)
}

fn policy_name(policy: &DumpPolicy) -> String {
    if policy.require_animation_when_actors {
        "headful".to_string()
    } else {
        "headless".to_string()
    }
}

fn read_dump(path: &PathBuf, from_markers: bool) -> Result<civis_cli::dump::SceneDump, DumpError> {
    let text = read_text(path)?;
    if from_markers {
        extract_dump_from_markers(&text)
    } else {
        parse_dump_json(&text)
    }
}

fn read_dump_file(path: &PathBuf) -> Result<civis_cli::dump::SceneDump, DumpError> {
    let text = read_text(path)?;
    parse_dump_json(&text)
}

fn read_text(path: &PathBuf) -> Result<String, DumpError> {
    if path.as_os_str() == "-" {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|source| DumpError::Io {
                path: PathBuf::from("-"),
                source,
            })?;
        return Ok(buf);
    }
    std::fs::read_to_string(path).map_err(|source| DumpError::Io {
        path: path.clone(),
        source,
    })
}

fn emit_report(report: &civis_cli::dump::DumpReport) -> Result<(), DumpError> {
    let payload = json!({
        "passed": report.passed,
        "violations": report.violations,
        "policy": report.policy,
        "baseline": report.baseline,
        "harness_version": civis_cli::HARNESS_VERSION,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&payload).map_err(DumpError::Decode)?
    );
    if report.passed {
        Ok(())
    } else {
        Err(DumpError::GatesFailed {
            count: report.violations.len(),
        })
    }
}
