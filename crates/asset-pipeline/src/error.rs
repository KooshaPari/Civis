//! Error types for the asset-pipeline export pipeline.
//!
//! All fallible operations in [`crate::export_svg`] (and any future siblings)
//! return [`ExportError`]. Three variants:
//!
//! - [`ExportError::Io`] — filesystem read/write or path resolution
//! - [`ExportError::Parse`] — input file parse failure (e.g. malformed SVG)
//! - [`ExportError::Encode`] — encoder rejected the input (e.g. fmt mismatch,
//!   WEBM codec not wired yet — see FR-ASSET-PIPELINE-002)

use std::fmt;
use std::path::PathBuf;

/// All errors produced by `asset_pipeline` operations.
#[derive(Debug)]
pub enum ExportError {
    /// Filesystem I/O failed for `path`. The wrapped [`std::io::Error`] is the
    /// underlying cause via [`std::error::Error::source`].
    Io {
        /// Path on which the operation failed.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// Parse error (e.g. malformed SVG). Carries a human-readable message.
    Parse(String),
    /// Encode error (e.g. WEBM codec not yet wired, see FR-ASSET-PIPELINE-002).
    /// Carries a human-readable message.
    Encode(String),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::Io { path, source } => {
                write!(f, "io error on path {:?}: {}", path, source)
            }
            ExportError::Parse(s) => write!(f, "parse error: {}", s),
            ExportError::Encode(s) => write!(f, "encode error: {}", s),
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExportError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
