//! `asset-pipeline` — Civis asset export pipeline.
//!
//! See FR-ASSET-PIPELINE-001 (scaffold) + FR-ASSET-PIPELINE-002 (WEBM).
//!
//! ## Pipeline contract (per Civis asset brief, 2026-07-05)
//!
//! Given an input SVG, [`export_svg`] writes all required raster + icon
//! variants to the output directory:
//!
//! - PNG @ 1x, 2x, 3x  — via `resvg`
//! - ICO               — via `image` (`ico` feature only)
//! - WEBM              — via codec (TBD; see FR-ASSET-PIPELINE-002)
//!
//! ## Status
//!
//! Scaffold only — [`export_svg`] validates the crate skeleton + error path
//! without invoking external encoders. Encoders wire in once the first asset
//! (loading skeleton, FR-ASSET-PIPELINE-001 first-asset load) lands and the
//! playability-verify build (Task #2) flips to completed.

use std::path::Path;

mod error;
pub use error::ExportError;

/// Export a vector SVG source to all required raster + icon formats.
///
/// # Arguments
///
/// * `input` — path to a source SVG file
/// * `output_dir` — destination directory; PNG@1x/2x/3x, `.ico`, `.webm` are
///   written here. Must already exist.
///
/// # Errors
///
/// Returns [`ExportError`] on:
/// - missing input (`ExportError::Io`)
/// - missing output dir (`ExportError::Io`)
/// - encoder not yet wired (`ExportError::Encode`) — scaffold stub
pub fn export_svg(input: &Path, output_dir: &Path) -> Result<(), ExportError> {
    if !input.exists() {
        return Err(ExportError::Io {
            path: input.to_path_buf(),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "input svg not found",
            ),
        });
    }
    if !output_dir.exists() {
        return Err(ExportError::Io {
            path: output_dir.to_path_buf(),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "output dir not found",
            ),
        });
    }
    // Scaffold stub: confirms crate skeleton + error path. Full encoder
    // wiring (resvg → PNG, image → ICO, codec → WEBM) lands with the
    // first-asset PR (loading skeleton), gated on playability-verify #2.
    Err(ExportError::Encode(
        "scaffold-only stub; encoder wiring gated on first-asset PR per FR-ASSET-PIPELINE-001"
            .to_string(),
    ))
}
