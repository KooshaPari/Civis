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
pub mod manifest;
mod validate;
pub use error::ExportError;
pub use validate::{validate_svg_template, TemplateRule};

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
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "input svg not found"),
        });
    }
    if !output_dir.exists() {
        return Err(ExportError::Io {
            path: output_dir.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "output dir not found"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;
    use std::path::PathBuf;

    // FR-ASSET-PIPELINE-002 — WEBM is the last un-wired variant in the
    // export contract (PNG@1x/2x/3x + ICO + WEBM). While the crate is a
    // scaffold, `export_svg` must fail with `ExportError::Encode` and the
    // error must carry the WEBM/gating context rather than a parse or io
    // error, for both a present input and a valid output dir.
    #[test]
    fn export_svg_scaffold_reports_encode_error_for_valid_paths() {
        let dir = std::env::temp_dir().join(format!(
            "asset-pipeline-002-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create output dir");
        let input = dir.join("source.svg");
        std::fs::write(&input, "<svg viewBox=\"0 0 1 1\"/>").expect("write input svg");

        let err = export_svg(&input, &dir).expect_err("scaffold must not encode yet");
        match &err {
            ExportError::Encode(msg) => {
                assert!(
                    msg.contains("encoder") || msg.contains("FR-ASSET-PIPELINE"),
                    "encode error should describe the scaffold gate, got: {msg}"
                );
            }
            other => panic!("expected ExportError::Encode, got {other:?}"),
        }
        // The WEBM/PNG/ICO contract is documented through the error path
        // until encoders land; Display must be non-empty.
        assert!(!err.to_string().is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    // FR-ASSET-PIPELINE-002 — missing output directory must surface as an
    // `Io` error before any encoder work is attempted.
    #[test]
    fn export_svg_missing_output_dir_is_io_error() {
        let input = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let missing = PathBuf::from(format!("/nonexistent/asset-pipeline-002-out"));
        let err = export_svg(&input, &missing).expect_err("missing output dir");
        assert!(
            matches!(err, ExportError::Io { .. }),
            "expected Io error for missing output dir, got {err:?}"
        );
        // `source()` must expose the underlying io::Error cause.
        assert!(err.source().is_some(), "Io variant must chain its source");
    }

    // FR-ASSET-PIPELINE-002 — missing input file is an `Io` error with the
    // offending path preserved for CLI diagnostics.
    #[test]
    fn export_svg_missing_input_is_io_error_with_path() {
        let missing = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("no-such-file.svg");
        let out = std::env::temp_dir();
        let err = export_svg(&missing, &out).expect_err("missing input");
        match &err {
            ExportError::Io { path, source } => {
                assert_eq!(path, &missing);
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            }
            other => panic!("expected Io error, got {other:?}"),
        }
    }
}
