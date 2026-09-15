//! # CLI Branding Integration
//!
//! Shows how to embed the Civis ASCII art logo into CLI output
//! using `include_str!()` for compile-time embedding.
//!
//! This is a **documentation snippet** — not compiled into any binary.
//! Copy the relevant functions into `civis-cli/src/bin/civis.rs` as needed.

// ------------------------------------------------------------------
// 1. Embed the logos at compile time
// ------------------------------------------------------------------

/// Full ASCII art logo (from assets/branding/civis-logo.txt).
const LOGO: &str = include_str!("../../assets/branding/civis-logo.txt");

/// Compact single-line logo (from assets/branding/civis-logo-small.txt).
const LOGO_SMALL: &str = include_str!("../../assets/branding/civis-logo-small.txt");

// ------------------------------------------------------------------
// 2. Print on --help
// ------------------------------------------------------------------

/// Print the full logo followed by help text.
fn print_main_help() {
    println!("{LOGO}");
    println!("  civis {} — Civis interactive CLI", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Usage:");
    println!("  civis              Launch the interactive REPL");
    println!("  civis repl         Launch the interactive REPL (explicit)");
    println!("  civis --version    Show version");
    println!("  civis --help       Show this help");
    println!();
    println!("Available binaries:");
    println!("  civis-verify       Bevy frame capture (requires `bevy` feature)");
    println!("  civis-pixels       PNG pixel statistics");
    println!("  civis-census       sim.status via WS JSON-RPC bridge");
    println!("  civis-dump         CIVIS_DUMP validation");
    println!("  civis-mcp          MCP JSON-RPC shim (stdio)");
}

// ------------------------------------------------------------------
// 3. Print on --version
// ------------------------------------------------------------------

/// Print version with the compact logo.
fn print_version() {
    println!(
        "{} civis {}",
        LOGO_SMALL.trim(),
        env!("CARGO_PKG_VERSION")
    );
}

// ------------------------------------------------------------------
// 4. Use in the REPL welcome banner
// ------------------------------------------------------------------

/// Print a welcome banner when the interactive REPL starts.
fn print_repl_banner() {
    println!("{LOGO}");
    println!("  Type 'help' for commands, 'exit' to quit.");
    println!();
}

// ------------------------------------------------------------------
// 5. Full integration example (replaces the match block in main)
// ------------------------------------------------------------------

/*
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_repl_banner();
        let config = ReplConfig::default();
        let mut repl = Repl::new(config);
        if let Err(err) = repl.run() {
            eprintln!("civis: {err}");
            std::process::exit(1);
        }
    } else {
        match args[0].as_str() {
            "--help" | "-h" => print_main_help(),
            "--version" | "-V" => print_version(),
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
                print_repl_banner();
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
*/
