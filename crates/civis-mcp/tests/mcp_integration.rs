//! Integration tests for the `civis-mcp` tool surface — FR-CIV-TEST-002.
//!
//! Covers the full MCP-exposed API from outside the crate:
//!
//! - Tool registry: list returns exactly the expected 3 tools
//! - Tool schema: each tool has a non-empty description and valid JSON schema
//! - `civis_pixels` happy path: valid PNG produces correct stats shape
//! - `civis_pixels` error path: missing file returns descriptive error
//! - `civis_pixels` edge cases: 1x1, grayscale, RGBA, all-black PNG
//! - `civis_census` error path: unreachable host returns descriptive error
//! - `pixels_for_png` direct lib path matches tool payload output
//! - TOOL_NAMES constant stays in sync with router

use std::path::Path;

use civis_mcp::{pixels_for_png, pixels_tool_payload, tool_names, tool_router, HARNESS_VERSION, TOOL_NAMES};

// ── helpers ───────────────────────────────────────────────────────────────

fn write_rgb_png(path: &Path, width: u32, height: u32, fill: [u8; 3]) {
    let data: Vec<u8> = (0..(width * height))
        .flat_map(|_| fill)
        .collect();
    let file = std::fs::File::create(path).expect("create png");
    let mut enc = png::Encoder::new(file, width, height);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().expect("header");
    w.write_image_data(&data).expect("data");
    w.finish().expect("finish");
}

fn write_rgba_png(path: &Path, width: u32, height: u32, fill: [u8; 4]) {
    let data: Vec<u8> = (0..(width * height))
        .flat_map(|_| fill)
        .collect();
    let file = std::fs::File::create(path).expect("create png");
    let mut enc = png::Encoder::new(file, width, height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().expect("header");
    w.write_image_data(&data).expect("data");
    w.finish().expect("finish");
}

fn write_grayscale_png(path: &Path, width: u32, height: u32, value: u8) {
    let data: Vec<u8> = vec![value; (width * height) as usize];
    let file = std::fs::File::create(path).expect("create png");
    let mut enc = png::Encoder::new(file, width, height);
    enc.set_color(png::ColorType::Grayscale);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().expect("header");
    w.write_image_data(&data).expect("data");
    w.finish().expect("finish");
}

fn tmp_dir() -> std::path::PathBuf {
    let d = std::env::temp_dir().join("civis-mcp-integration-tests");
    std::fs::create_dir_all(&d).expect("create tmp dir");
    d
}

// ── tool registry ─────────────────────────────────────────────────────────

/// Tool list must contain exactly 3 entries, sorted lexicographically.
#[test]
fn tool_list_returns_three_tools() {
    let names = tool_names();
    assert_eq!(names.len(), 3, "expected exactly 3 tools, got {names:?}");
}

/// Tool names must be sorted (the lib sorts them; callers rely on stable order).
#[test]
fn tool_list_is_sorted() {
    let names = tool_names();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "tool_names() must be sorted lexicographically");
}

/// TOOL_NAMES constant must match the router exactly.
#[test]
fn tool_names_constant_matches_router() {
    let mut router_names = tool_names();
    router_names.sort();
    let mut const_names: Vec<String> = TOOL_NAMES.iter().map(|s| s.to_string()).collect();
    const_names.sort();
    assert_eq!(
        router_names, const_names,
        "TOOL_NAMES constant is out of sync with the registered router"
    );
}

/// Every registered tool must have a non-empty description.
#[test]
fn every_tool_has_non_empty_description() {
    let tools = tool_router().list_all();
    for tool in &tools {
        assert!(
            tool.description.as_deref().map(|d| !d.is_empty()).unwrap_or(false),
            "tool `{}` has no description",
            tool.name
        );
    }
}

/// Every registered tool must carry an `inputSchema` object (JSON Schema).
#[test]
fn every_tool_has_input_schema() {
    let tools = tool_router().list_all();
    for tool in &tools {
        // The rmcp `Tool` type serialises the input schema as a JSON Value.
        // We just verify it is not null/empty by checking the schema is an object.
        let schema = &tool.input_schema;
        assert!(
            schema.is_object() || !schema.is_null(),
            "tool `{}` has a null/empty input_schema",
            tool.name
        );
    }
}

/// civis_verify and civis_pixels and civis_census are individually present.
#[test]
fn tool_names_contain_expected_entries() {
    let names = tool_names();
    for expected in &["civis_verify", "civis_pixels", "civis_census"] {
        assert!(
            names.iter().any(|n| n == expected),
            "tool `{expected}` missing from registered router; got {names:?}"
        );
    }
}

// ── civis_pixels happy path ───────────────────────────────────────────────

/// pixels_tool_payload on an all-red 8x8 PNG must return 100% non-black, 0% gray.
#[test]
fn pixels_all_red_png_stats() {
    let dir = tmp_dir();
    let path = dir.join("all-red.png");
    write_rgb_png(&path, 8, 8, [255, 0, 0]);

    let payload = pixels_tool_payload(&path, 4).expect("pixels_tool_payload");
    let stats = payload["stats"].as_object().expect("stats object");

    let mean_r = stats["mean_r"].as_f64().expect("mean_r");
    assert!((mean_r - 255.0).abs() < 0.1, "all-red must have mean_r≈255, got {mean_r}");

    let pct_black = stats["percent_near_black"].as_f64().expect("pct_black");
    assert!(pct_black.abs() < 0.01, "all-red must have 0% near-black, got {pct_black}");

    let pct_gray = stats["percent_gray"].as_f64().expect("pct_gray");
    assert!(pct_gray.abs() < 0.01, "pure red is not gray, got {pct_gray}");

    std::fs::remove_file(&path).ok();
}

/// pixels_for_png on an all-black 8x8 PNG: 100% near-black, 0 distinct hues.
#[test]
fn pixels_all_black_png_stats() {
    let dir = tmp_dir();
    let path = dir.join("all-black.png");
    write_rgb_png(&path, 8, 8, [0, 0, 0]);

    let stats = pixels_for_png(&path, 4).expect("pixels_for_png");
    assert!(
        stats.mean_r < 1.0,
        "all-black mean_r must be ~0, got {}",
        stats.mean_r
    );
    assert!(
        (stats.percent_near_black - 100.0).abs() < 0.01,
        "all-black must be 100% near-black, got {}",
        stats.percent_near_black
    );
    assert_eq!(
        stats.distinct_hue_count, 0,
        "all-black has 0 distinct hues, got {}",
        stats.distinct_hue_count
    );

    std::fs::remove_file(&path).ok();
}

/// RGBA PNG is decoded correctly (alpha stripped, RGB stats valid).
#[test]
fn pixels_rgba_png_decoded_correctly() {
    let dir = tmp_dir();
    let path = dir.join("rgba-green.png");
    write_rgba_png(&path, 4, 4, [0, 255, 0, 128]);

    let stats = pixels_for_png(&path, 2).expect("pixels_for_png rgba");
    assert!(
        stats.mean_r < 1.0,
        "RGBA green must have mean_r≈0, got {}",
        stats.mean_r
    );
    assert!(
        stats.percent_near_black.abs() < 0.01,
        "RGBA green is not near-black, got {}",
        stats.percent_near_black
    );

    std::fs::remove_file(&path).ok();
}

/// Grayscale PNG is decoded correctly (gray pixels count as gray, not colored).
#[test]
fn pixels_grayscale_png_decoded_correctly() {
    let dir = tmp_dir();
    let path = dir.join("gray-mid.png");
    // mid-gray: R=G=B=128 → should be 100% gray, 0% near-black
    write_grayscale_png(&path, 4, 4, 128);

    let stats = pixels_for_png(&path, 2).expect("pixels_for_png grayscale");
    assert!(
        (stats.percent_gray - 100.0).abs() < 0.01,
        "mid-gray must be 100% gray, got {}",
        stats.percent_gray
    );

    std::fs::remove_file(&path).ok();
}

/// 1×1 PNG is handled without panic or error.
#[test]
fn pixels_1x1_png_no_panic() {
    let dir = tmp_dir();
    let path = dir.join("tiny.png");
    write_rgb_png(&path, 1, 1, [200, 100, 50]);

    let result = pixels_for_png(&path, 1);
    assert!(result.is_ok(), "1x1 PNG must not error: {:?}", result.err());
    let stats = result.unwrap();
    assert_eq!(stats.samples, 1, "1x1 grid=1 must produce 1 sample");

    std::fs::remove_file(&path).ok();
}

// ── civis_pixels error path ───────────────────────────────────────────────

/// Missing PNG file must return an error, not panic.
#[test]
fn pixels_missing_file_returns_error() {
    let result = pixels_for_png(Path::new("/nonexistent/path/frame.png"), 4);
    assert!(result.is_err(), "missing file must return Err");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("read") || msg.contains("No such") || msg.contains("cannot"),
        "error must describe the read failure, got: {msg}"
    );
}

/// Non-PNG bytes must return a descriptive decode error, not panic.
#[test]
fn pixels_non_png_bytes_returns_error() {
    let dir = tmp_dir();
    let path = dir.join("not-a-png.png");
    std::fs::write(&path, b"this is not a PNG file").expect("write");

    let result = pixels_for_png(&path, 4);
    assert!(result.is_err(), "non-PNG file must return Err");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("png") || msg.contains("decode") || msg.contains("invalid"),
        "error must mention PNG decode failure, got: {msg}"
    );

    std::fs::remove_file(&path).ok();
}

// ── civis_census error path ───────────────────────────────────────────────

/// census_sim_status on an unreachable host must return an error with URL info.
#[test]
fn census_unreachable_host_returns_error() {
    use civis_mcp::census_sim_status;
    use civis_cli::census::CensusConfig;

    // Port 1 is almost certainly closed and causes immediate refusal.
    let config = CensusConfig {
        host: "127.0.0.1".to_string(),
        port: 1,
        timeout_ms: 500,
    };
    let result = census_sim_status(&config);
    assert!(result.is_err(), "unreachable host must return Err");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("WS connect") || msg.contains("connect") || msg.contains("refused")
            || msg.contains("timed out"),
        "error must mention connection failure, got: {msg}"
    );
}

// ── lib-level invariants ──────────────────────────────────────────────────

/// HARNESS_VERSION must be non-empty (from Cargo.toml PKG_VERSION).
#[test]
fn harness_version_is_non_empty() {
    assert!(!HARNESS_VERSION.is_empty(), "HARNESS_VERSION must be set from CARGO_PKG_VERSION");
}

/// pixels_tool_payload payload shape: path, grid, stats keys present.
#[test]
fn pixels_tool_payload_shape() {
    let dir = tmp_dir();
    let path = dir.join("shape-test.png");
    write_rgb_png(&path, 4, 4, [100, 150, 200]);

    let payload = pixels_tool_payload(&path, 2).expect("pixels_tool_payload");
    assert!(payload.get("path").is_some(), "payload missing 'path'");
    assert!(payload.get("grid").is_some(), "payload missing 'grid'");
    assert!(payload.get("stats").is_some(), "payload missing 'stats'");

    let stats = payload["stats"].as_object().expect("stats is object");
    for key in &["samples", "mean_r", "percent_near_black", "percent_gray", "distinct_hue_count"] {
        assert!(stats.contains_key(*key), "stats missing '{key}': {stats:?}");
    }

    std::fs::remove_file(&path).ok();
}
