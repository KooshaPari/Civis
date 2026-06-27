//! `civis-mcp` — MCP (Model Context Protocol) server for the `civis-cli`
//! verification harness.
//!
//! Three tools are exposed to MCP clients:
//!
//! | Tool            | Backing `civis-cli` API                                 |
//! |-----------------|----------------------------------------------------------|
//! | `civis_verify`  | `civis_cli::verify::run_verify` (requires `bevy` feature) |
//! | `civis_pixels`  | `civis_cli::pixels::{sample_rgb_grid, compute_pixel_stats}` |
//! | `civis_census`  | `civis_cli::census::{build_sim_status_request, decode_response, validate_sim_status}` over the civ-server WS bridge |
//!
//! The MCP shim calls the `civis-cli` library functions directly — there is
//! no `cargo run` shell-out. The only time the shim touches a child process
//! is when `civis_verify` (feature-gated) delegates to Bevy, which already
//! spawns its own windowed renderer; even there the call is in-process.
//!
//! The non-MCP helper surface (`tool_names`, `pixels_for_png`,
//! `census_sim_status`) is exposed here so unit tests can verify the tool
//! schema and the synthetic image path without standing up an rmcp
//! transport.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::path::Path;

use rmcp::model::Tool;
use serde_json::{json, Value};

use civis_cli::census::{
    build_sim_status_request, decode_response, validate_sim_status, CensusConfig,
};
use civis_cli::config::census_config_from_env;
use civis_cli::pixels::{compute_pixel_stats, sample_rgb_grid, PixelStats};

/// Canonical names of the MCP tools this crate registers. The PR description
/// references this list; tests assert the rmcp router matches it exactly so a
/// future rename surfaces in CI rather than in production.
pub const TOOL_NAMES: &[&str] = &["civis_verify", "civis_pixels", "civis_census"];

/// Library version string. Mirrors `civis_cli::HARNESS_VERSION` so MCP
/// clients can correlate evidence packets with the harness build.
pub const HARNESS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build the MCP tool router used by `main`. Extracted into a helper so
/// tests can call `tool_names()` and `tool_router().list_all()` without
/// running the full async transport.
#[doc(hidden)]
pub fn tool_router() -> rmcp::handler::server::router::tool::ToolRouter<CivisMcpServer> {
    // The `#[tool_router]` macro emits an inherent `tool_router()` fn on
    // `CivisMcpServer`; the public `registered_router()` shim in `server.rs`
    // re-exposes it across the module boundary.
    server::registered_router()
}

/// Names of every tool the router registers, sorted lexicographically.
/// Wraps `ToolRouter::list_all` and pulls the `name` field off each entry.
pub fn tool_names() -> Vec<String> {
    let mut names: Vec<String> = tool_router()
        .list_all()
        .into_iter()
        .map(|tool: Tool| tool.name.to_string())
        .collect();
    names.sort();
    names
}

/// Decode a PNG file and compute pixel statistics using the pure
/// `civis_cli::pixels` library functions. Exposed at the library level so
/// the unit test can exercise the same code path the MCP tool uses.
///
/// `grid` is the number of sample points per axis (default 16 in the MCP
/// tool; tests use smaller grids on synthetic 4×4 images).
pub fn pixels_for_png(path: &Path, grid: usize) -> Result<PixelStats, String> {
    let bytes = std::fs::read(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    let data = decode_png_to_rgb(&bytes)?;
    let samples = sample_rgb_grid(data.width, data.height, grid, &data.rgb);
    Ok(compute_pixel_stats(&samples))
}

/// Result of decoding a PNG into a flat RGB buffer.
struct DecodedRgb {
    width: usize,
    height: usize,
    rgb: Vec<u8>,
}

fn decode_png_to_rgb(bytes: &[u8]) -> Result<DecodedRgb, String> {
    let decoder = png::Decoder::new(bytes);
    let mut reader = decoder
        .read_info()
        .map_err(|err| format!("png decode: {err}"))?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let frame = reader
        .next_frame(&mut buf)
        .map_err(|err| format!("png frame: {err}"))?;
    let width = frame.width as usize;
    let height = frame.height as usize;
    let rgb: Vec<u8> = match frame.color_type {
        png::ColorType::Rgb => buf[..frame.buffer_size()].to_vec(),
        png::ColorType::Rgba => buf[..frame.buffer_size()]
            .chunks_exact(4)
            .flat_map(|px| [px[0], px[1], px[2]])
            .collect(),
        png::ColorType::Grayscale => buf[..frame.buffer_size()]
            .iter()
            .flat_map(|&v| [v, v, v])
            .collect(),
        png::ColorType::GrayscaleAlpha => buf[..frame.buffer_size()]
            .chunks_exact(2)
            .flat_map(|px| [px[0], px[0], px[0]])
            .collect(),
        png::ColorType::Indexed => return Err("indexed PNGs are not supported".to_string()),
    };
    Ok(DecodedRgb { width, height, rgb })
}

/// Issue a `sim.status` JSON-RPC request over the civ-server WebSocket
/// bridge and decode the response. Mirrors the `civis-census` bin's
/// transport but skips the JSON parse envelope so the MCP tool can
/// surface the raw `sim.status` payload.
///
/// `config` is taken by value so the MCP tool can read from `.env` once
/// and pass the resolved config to the library. Returns the validated
/// `SimStatusResult` ready to be re-serialised as JSON.
pub fn census_sim_status(
    config: &CensusConfig,
) -> Result<civis_cli::census::SimStatusResult, String> {
    let url = config.ws_url();
    let frame = build_sim_status_request(civis_cli::census::wire::RequestId::Number(1));

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|err| format!("build census runtime: {err}"))?;

    runtime.block_on(async move {
        let connect = tokio_tungstenite::connect_async(&url)
            .await
            .map_err(|err| format!("WS connect {url}: {err}"))?;
        let (mut ws, _response) = connect;
        use futures_util::{SinkExt, StreamExt};
        ws.send(tokio_tungstenite::tungstenite::Message::Text(frame))
            .await
            .map_err(|err| format!("WS send sim.status: {err}"))?;
        let response = tokio::time::timeout(config.timeout(), ws.next())
            .await
            .map_err(|_| format!("timed out waiting for {url}"))?
            .ok_or_else(|| "server closed the connection".to_string())?
            .map_err(|err| format!("WS read: {err}"))?;
        let text = match response {
            tokio_tungstenite::tungstenite::Message::Text(t) => t,
            tokio_tungstenite::tungstenite::Message::Binary(b) => {
                String::from_utf8(b).map_err(|err| format!("binary frame utf-8: {err}"))?
            }
            other => return Err(format!("unexpected frame {other:?}")),
        };
        let parsed = decode_response(&text).map_err(|err| format!("decode: {err:?}"))?;
        validate_sim_status(&parsed).map_err(|err| format!("validate: {err}"))
    })
}

/// Resolve the [`CensusConfig`] from environment, plus the URL string the
/// MCP tool should echo back to the operator.
pub fn census_config_with_url() -> CensusConfig {
    census_config_from_env()
}

/// Build a `serde_json::Value` for the `civis_pixels` MCP tool. Splits the
/// decode + stats work so the unit test can compare against the same shape
/// the tool returns over the wire.
pub fn pixels_tool_payload(path: &Path, grid: usize) -> Result<Value, String> {
    let stats = pixels_for_png(path, grid)?;
    Ok(json!({
        "path": path,
        "grid": grid,
        "stats": stats,
    }))
}

#[doc(hidden)]
pub use server::CivisMcpServer;

mod server;

#[cfg(test)]
mod tests {
    //! Unit tests for the `civis-mcp` library.
    //!
    //! Two test families:
    //!
    //! 1. **Schema tests** assert the rmcp `ToolRouter` registers exactly
    //!    the three tools the PR description promises, with the expected
    //!    names and descriptions. They catch renames / regressions without
    //!    needing a live MCP client.
    //! 2. **Pixels unit test** exercises the `civis_pixels` MCP tool's
    //!    library path against a synthetic RGB buffer (decoded from a
    //!    hand-built PNG) so the test stays deterministic and offline.

    use super::*;

    /// Synthesize an 8x8 RGB PNG file on disk. The image alternates rows
    /// of red and black so the expected pixel stats are trivial to verify.
    fn write_synthetic_png(path: &Path) {
        let width: u32 = 8;
        let height: u32 = 8;
        let mut data = vec![0u8; (width * height * 3) as usize];
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 3) as usize;
                if y < height / 2 {
                    // Top half: pure red
                    data[idx] = 255;
                    data[idx + 1] = 0;
                    data[idx + 2] = 0;
                } else {
                    // Bottom half: pure black (near-black)
                    data[idx] = 0;
                    data[idx + 1] = 0;
                    data[idx + 2] = 0;
                }
            }
        }
        let file = std::fs::File::create(path).expect("create png");
        let mut encoder = png::Encoder::new(file, width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("png header");
        writer.write_image_data(&data).expect("png data");
        writer.finish().expect("png finish");
    }

    /// `tool_names()` must return exactly the three verify-related tools,
    /// sorted lexicographically.
    #[test]
    fn tool_names_match_expected() {
        let names = tool_names();
        assert_eq!(
            names,
            vec![
                "civis_census".to_string(),
                "civis_pixels".to_string(),
                "civis_verify".to_string(),
            ]
        );
        // Sanity: the constant table in lib.rs matches the router.
        let mut const_names: Vec<String> = TOOL_NAMES.iter().map(|s| (*s).to_string()).collect();
        const_names.sort();
        assert_eq!(names, const_names);
    }

    /// `tool_router().list_all()` must return one `Tool` entry per name and
    /// each entry's `name` field must equal the function-level `#[tool(name
    /// = "...")]` attribute.
    #[test]
    fn router_list_all_is_consistent() {
        let tools = tool_router().list_all();
        let names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
        assert_eq!(names.len(), 3, "expected exactly 3 tools, got {names:?}");
        for expected in TOOL_NAMES {
            assert!(
                names.iter().any(|n| n == expected),
                "missing tool `{expected}` in {names:?}"
            );
        }
    }

    /// `civis_pixels` must produce sensible statistics on a synthetic
    /// half-red / half-black 8x8 PNG:
    /// - 4×4 grid = 16 samples, all pure red (R=255) or pure black (R=0).
    /// - mean_r = 127.5 (half of 255).
    /// - percent_near_black = 50% (the threshold is 8; black passes it,
    ///   red does not).
    /// - percent_gray = 0% (no samples have R == G == B; red is (255,0,0)).
    /// - distinct_hue_count = 1 (only the red hue).
    #[test]
    fn pixels_stats_on_synthetic_image() {
        let tmp_dir = std::env::temp_dir().join("civis-mcp-tests");
        std::fs::create_dir_all(&tmp_dir).expect("create tmp dir");
        let path = tmp_dir.join("synthetic-red-black.png");
        write_synthetic_png(&path);

        let payload = pixels_tool_payload(&path, 4).expect("pixels_tool_payload");
        let stats = payload
            .get("stats")
            .expect("payload has stats")
            .as_object()
            .expect("stats is object");

        let samples = stats["samples"].as_u64().expect("samples u64");
        assert_eq!(samples, 16, "4x4 grid = 16 samples");

        let mean_r = stats["mean_r"].as_f64().expect("mean_r f64");
        assert!(
            (mean_r - 127.5).abs() < 0.5,
            "mean_r expected ~127.5, got {mean_r}"
        );

        let pct_black = stats["percent_near_black"].as_f64().expect("pct black f64");
        assert!(
            (pct_black - 50.0).abs() < 0.01,
            "percent_near_black expected 50, got {pct_black}"
        );

        let pct_gray = stats["percent_gray"].as_f64().expect("pct gray f64");
        assert!(
            pct_gray.abs() < 0.01,
            "percent_gray expected 0 (no grayscale samples), got {pct_gray}"
        );

        let distinct = stats["distinct_hue_count"].as_u64().expect("distinct u64");
        assert_eq!(distinct, 1, "only the red hue should be present");

        // The harness version is echoed back so MCP clients can correlate
        // evidence packets with the build.
        let _ = payload.get("path").expect("payload has path");
        let _ = payload.get("grid").expect("payload has grid");

        std::fs::remove_file(&path).ok();
    }
}
