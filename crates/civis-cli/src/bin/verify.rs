use std::path::PathBuf;

use civis_cli::config::{load_dotenv, verify_output_dir_from_env, verify_settle_frames_from_env};
use civis_cli::verify::{run_verify, VerifyOptions, VerifyResult};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "civis-verify",
    about = "Launch a windowed Bevy 0.18 client, wait N ticks, capture a frame to PNG"
)]
struct Args {
    /// Override the output PNG path (defaults to $CIV_VERIFY_OUT_DIR/frame-0.png).
    #[arg(long)]
    out: Option<PathBuf>,

    /// Number of frames to let the renderer settle before the screenshot.
    #[arg(long)]
    settle_frames: Option<u32>,

    /// Window width in logical pixels.
    #[arg(long, default_value_t = 640)]
    width: u32,

    /// Window height in logical pixels.
    #[arg(long, default_value_t = 360)]
    height: u32,
}

fn main() {
    load_dotenv();
    pheno_tracing::init();

    let args = Args::parse();
    let out_dir = verify_output_dir_from_env();
    let output_path = args.out.unwrap_or_else(|| out_dir.join("frame-0.png"));

    let options = VerifyOptions {
        output_path,
        settle_frames: args
            .settle_frames
            .unwrap_or_else(verify_settle_frames_from_env),
        width: args.width,
        height: args.height,
    };

    match run_verify(options) {
        Ok(result) => print_result(&result),
        Err(err) => {
            eprintln!("civis-verify failed: {err}");
            std::process::exit(1);
        }
    }
}

fn print_result(result: &VerifyResult) {
    match serde_json::to_string_pretty(result) {
        Ok(text) => println!("{text}"),
        Err(err) => {
            eprintln!("civis-verify: failed to serialise result: {err}");
            std::process::exit(2);
        }
    }
}
