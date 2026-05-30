use std::path::PathBuf;
use std::time::Instant;

use clap::{Parser, Subcommand};
use civ_engine::{CivSaveBundle, Simulation};

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
        /// Seed used to initialize the simulation.
        #[arg(long)]
        seed: u64,

        /// Output archive path.
        #[arg(long, value_name = "PATH", default_value = "world.civsave.zst")]
        out: PathBuf,
    },
    /// Inspect a .civsave directory or archive and print metadata plus summary.
    InspectSave {
        /// Path to a .civsave directory or .civsave.zst archive.
        path: PathBuf,
    },
    /// Benchmark headless tick throughput.
    Bench {
        /// Seed used to initialize the simulation.
        #[arg(long, default_value_t = 1)]
        seed: u64,

        /// Number of ticks to execute.
        #[arg(long, default_value_t = 10_000)]
        ticks: u64,
    },
    /// Print the CLI version.
    Version,
}

fn main() {
    let cli = Cli::parse();
    let exit_code = match run(cli.command) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("{message}");
            1
        }
    };
    std::process::exit(exit_code);
}

fn run(command: Commands) -> Result<(), String> {
    match command {
        Commands::NewWorld { seed, out } => {
            let sim = Simulation::with_seed(seed);
            CivSaveBundle::save_archive(&out, &sim)
                .map_err(|err| format!("failed to write {}: {err}", out.display()))?;
            println!(
                "{}",
                serde_json::json!({
                    "command": "new-world",
                    "seed": seed,
                    "output": out,
                    "tick": sim.state.tick,
                })
            );
            Ok(())
        }
        Commands::InspectSave { path } => {
            let metadata = CivSaveBundle::read_metadata(&path)
                .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
            let sim = CivSaveBundle::load(&path)
                .map_err(|err| format!("failed to load {}: {err}", path.display()))?;
            println!(
                "{}",
                serde_json::json!({
                    "command": "inspect-save",
                    "path": path,
                    "metadata": metadata,
                    "state": {
                        "tick": sim.state.tick,
                        "population": sim.snapshot().population,
                        "energy_budget_joules": sim.state.energy_budget_joules,
                    },
                })
            );
            Ok(())
        }
        Commands::Bench { seed, ticks } => {
            let mut sim = Simulation::with_seed(seed);
            let start = Instant::now();
            for _ in 0..ticks {
                sim.tick();
            }
            let elapsed = start.elapsed();
            let per_tick_ns = elapsed.as_nanos() / u128::from(ticks.max(1));
            println!(
                "{}",
                serde_json::json!({
                    "command": "bench",
                    "seed": seed,
                    "ticks": ticks,
                    "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
                    "per_tick_ns": per_tick_ns,
                    "final_tick": sim.state.tick,
                })
            );
            Ok(())
        }
        Commands::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}

