//! `svg_export` — CLI binary for `asset_pipeline::export_svg`.
//!
//! Gated behind `--features cli` (see Cargo.toml `required-features`).
//! Invoke:
//!
//! ```text
//! cargo run -p asset-pipeline --features cli --bin svg_export -- \
//!     --input path/to/source.svg \
//!     --output-dir path/to/out
//! ```

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "svg_export",
    about = "Export an SVG to PNG@1x/2x/3x + ICO + (WEBM, FR-ASSET-PIPELINE-002) variants",
    long_about = "Civis asset-pipeline CLI. Reads an SVG and writes raster + icon variants."
)]
struct Args {
    /// Path to the source SVG file.
    #[arg(short = 'i', long = "input")]
    input: PathBuf,

    /// Output directory for raster + icon variants. Must already exist.
    #[arg(short = 'o', long = "output-dir")]
    output_dir: PathBuf,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match asset_pipeline::export_svg(&args.input, &args.output_dir) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("svg_export: error: {}", e);
            ExitCode::FAILURE
        }
    }
}
