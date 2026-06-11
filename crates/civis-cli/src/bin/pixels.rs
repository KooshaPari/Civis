use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use civis_cli::pixels::{compute_pixel_stats, sample_rgb_grid};
use serde_json::json;

#[derive(Debug, Parser)]
#[command(name = "civis-pixels", about = "Compute PNG pixel statistics as JSON")]
struct Args {
    /// PNG file to sample.
    input: PathBuf,

    /// Sampling grid size per axis.
    #[arg(long, default_value_t = 16)]
    grid: usize,

    /// Near-black threshold (0-255).
    #[arg(long, default_value_t = 8)]
    near_black_threshold: u8,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let file = File::open(&args.input)?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let output = reader.next_frame(&mut buf)?;

    let bytes = &buf[..output.buffer_size()];
    let data = match output.color_type {
        png::ColorType::Rgb => bytes.to_vec(),
        png::ColorType::Rgba => bytes
            .chunks_exact(4)
            .flat_map(|px| [px[0], px[1], px[2]])
            .collect(),
        png::ColorType::Grayscale => bytes.iter().flat_map(|&v| [v, v, v]).collect(),
        png::ColorType::GrayscaleAlpha => bytes
            .chunks_exact(2)
            .flat_map(|px| [px[0], px[0], px[0]])
            .collect(),
        png::ColorType::Indexed => {
            return Err("indexed PNGs are not supported; convert to RGB first".into())
        }
    };

    let samples = sample_rgb_grid(output.width as usize, output.height as usize, args.grid, &data);
    let mut stats = compute_pixel_stats(&samples);
    if args.near_black_threshold != 8 && !samples.is_empty() {
        let near_black = samples
            .iter()
            .filter(|s| s.is_near_black(args.near_black_threshold))
            .count();
        stats.percent_near_black = (near_black as f32 / samples.len() as f32) * 100.0;
    }

    let payload = json!({
        "input": args.input,
        "grid": args.grid,
        "near_black_threshold": args.near_black_threshold,
        "stats": stats,
    });
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}
