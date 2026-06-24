use std::path::PathBuf;

use clap::{Parser, Subcommand};

use civis_cli::{
    census_to_json, command_bench, command_inspect_save, command_new_world, run_build,
    run_populated_boot, run_screenshot, run_verify, CliError, CliResult,
};
use civis_cli::{find_latest_run_log, parse_census_text};

#[derive(Parser, Debug)]
#[command(name = "civis-cli", about = "Civis world and save utility")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new world and write it to a .civsave.zst archive.
    NewWorld {
        #[arg(long)]
        seed: u64,
        #[arg(long, value_name = "PATH", default_value = "world.civsave.zst")]
        out: PathBuf,
    },
    /// Inspect a .civsave directory or archive and print metadata plus summary.
    InspectSave {
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
    /// Benchmark headless tick throughput.
    Bench {
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long, default_value_t = 10_000)]
        ticks: u64,
    },
    /// Print the CLI version.
    Version,
    /// Build standalone civ-bevy-ref target binary.
    Build {
        #[arg(long, value_name = "DIR", default_value = "E:/civis-cli-target")]
        target_dir: PathBuf,
    },
    /// Build + run Civis once to capture a screenshot.
    Screenshot {
        /// Output screenshot path.
        out: PathBuf,
    },
    /// Parse the latest run log for census metadata.
    Census,
    /// End-to-end verify loop: build, deploy, screenshot, and print summary.
    Verify {
        #[arg(long, value_name = "PATH", default_value = "screenshots/verify.png")]
        out: PathBuf,
        #[arg(long, value_name = "DIR", default_value = "E:/civis-cli-target")]
        target_dir: PathBuf,
    },
    /// Populated-evidence boot: pre-fills the engine's voxel world with a
    /// deterministic non-empty material pattern, then ticks the sim so the
    /// live `sample_emergence()` path emits one `emergence sample:`
    /// line per 50-tick boundary. Designed to prove the PR #363
    /// emergence wiring against a world with content (the default sim is
    /// empty, so the prior evidence run yielded `entropy=0 structures=0`).
    PopulatedBoot {
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long, default_value_t = 250)]
        ticks: u64,
    },
}

fn main() {
    let cli = Cli::parse();
    std::process::exit(handle(cli.command));
}

fn handle(command: Commands) -> i32 {
    match dispatch(command) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{}", err.message);
            err.exit_code
        }
    }
}

fn dispatch(command: Commands) -> CliResult<i32> {
    match command {
        Commands::NewWorld { seed, out } => {
            let json = command_new_world(seed, &out)?;
            println!("{}", json);
            Ok(0)
        }
        Commands::InspectSave { path } => {
            let json = command_inspect_save(&path)?;
            println!("{}", json);
            Ok(0)
        }
        Commands::Bench { seed, ticks } => {
            let json = command_bench(seed, ticks)?;
            println!("{}", json);
            Ok(0)
        }
        Commands::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        Commands::Build { target_dir } => run_build(&target_dir).map(|_| 0),
        Commands::Screenshot { out } => {
            let root = civis_cli::workspace_root();
            let result = run_screenshot(&out, &root)?;
            println!(
                "{}",
                serde_json::json!({ "path": result.path, "bytes": result.bytes })
            );
            Ok(0)
        }
        Commands::Census => {
            let root = civis_cli::workspace_root();
            let log_path = find_latest_run_log(&root)?;
            if let Some(path) = log_path {
                let content = std::fs::read_to_string(path)?;
                let census = parse_census_text(&content);
                println!("{}", census_to_json(&census));
                Ok(0)
            } else {
                Err(CliError::new(1, "no census logs found"))
            }
        }
        Commands::Verify { out, target_dir } => {
            let result = run_verify(&target_dir, &out)?;
            let census = result
                .census
                .as_ref()
                .map(census_to_json)
                .unwrap_or_else(|| serde_json::json!(null));
            println!(
                "{}",
                serde_json::json!({
                    "command":"verify",
                    "screenshot": { "path": result.screenshot, "bytes": result.bytes },
                    "census": census,
                })
            );
            Ok(0)
        }
        Commands::PopulatedBoot { seed, ticks } => {
            let json = run_populated_boot(seed, ticks)?;
            println!("{}", json);
            Ok(0)
        }
    }
}
