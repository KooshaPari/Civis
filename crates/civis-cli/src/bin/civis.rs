//! `civis` — main interactive CLI for the Civis simulation.
//!
//! When launched with no arguments, starts an interactive REPL with 12
//! commands for inspecting and controlling the simulation. When launched
//! with arguments, delegates to the appropriate subcommand.

use civis_cli::repl::{Repl, ReplConfig};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        // No arguments → start interactive REPL.
        let config = ReplConfig::default();
        let mut repl = Repl::new(config);

        if let Err(err) = repl.run() {
            eprintln!("civis: {err}");
            std::process::exit(1);
        }
    } else {
        // Arguments provided → print available subcommands and delegate.
        match args[0].as_str() {
            "--help" | "-h" => {
                print_main_help();
            }
            "--version" | "-V" => {
                println!("civis {}", civis_cli::HARNESS_VERSION);
            }
            "verify" => {
                eprintln!("civis: use `cargo run --bin civis-verify` for Bevy frame capture");
            }
            "pixels" => {
                eprintln!("civis: use `cargo run --bin civis-pixels` for PNG statistics");
            }
            "census" => {
                eprintln!("civis: use `cargo run --bin civis-census` for sim.status queries");
            }
            "dump" => {
                eprintln!("civis: use `cargo run --bin civis-dump` for CIVIS_DUMP validation");
            }
            "mcp" => {
                eprintln!("civis: use `cargo run --bin civis-mcp` for the MCP JSON-RPC shim");
            }
            "repl" => {
                let config = ReplConfig::default();
                let mut repl = Repl::new(config);
                if let Err(err) = repl.run() {
                    eprintln!("civis: {err}");
                    std::process::exit(1);
                }
            }
            other => {
                eprintln!("civis: unknown subcommand '{other}'. Use --help for options.");
                std::process::exit(1);
            }
        }
    }
}

fn print_main_help() {
    println!(
        "civis {} — Civis interactive CLI\n",
        civis_cli::HARNESS_VERSION
    );
    println!("Usage:");
    println!("  civis              Launch the interactive REPL");
    println!("  civis repl         Launch the interactive REPL (explicit)");
    println!("  civis --version    Show version");
    println!("  civis --help       Show this help\n");
    println!("Available binaries:");
    println!("  civis-verify       Bevy frame capture (requires `bevy` feature)");
    println!("  civis-pixels       PNG pixel statistics");
    println!("  civis-census       sim.status via WS JSON-RPC bridge");
    println!("  civis-dump         CIVIS_DUMP validation");
    println!("  civis-mcp          MCP JSON-RPC shim (stdio)");
}
