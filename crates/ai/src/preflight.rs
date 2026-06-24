//! Loud-failure preflight (FR-CIV-AI-009).
//!
//! At startup, verify every **required** model artifact named by [`AiConfig`]
//! exists on disk. Missing → **named, loud failure**, no silent "AI off"
//! (`CLAUDE.md` "Optionality and failure behavior"). Multiple missing artifacts
//! are listed semicolon-separated.

use std::path::Path;

use crate::config::AiConfig;

/// A required model artifact to verify at startup.
#[derive(Debug, Clone)]
pub struct RequiredArtifact {
    /// Human-facing model name for the failure message.
    pub name: String,
    /// On-disk path that must exist.
    pub path: String,
}

/// Outcome of preflight. `Ok` lists the artifacts that were verified present.
pub type PreflightResult = Result<Vec<String>, PreflightError>;

/// Loud, named preflight failure listing every missing artifact.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
#[error("civ-ai preflight failed: {0}")]
pub struct PreflightError(pub String);

/// Verify all `required` artifacts exist. Missing ones produce a single loud,
/// semicolon-separated error (matching the repo's "named items" failure style).
///
/// # Errors
/// Returns [`PreflightError`] if any required artifact is missing on disk.
pub fn check_artifacts(required: &[RequiredArtifact]) -> PreflightResult {
    let mut present = Vec::new();
    let mut missing = Vec::new();
    for art in required {
        if Path::new(&art.path).exists() {
            present.push(art.name.clone());
        } else {
            missing.push(format!("missing model '{}' at {}", art.name, art.path));
        }
    }
    if missing.is_empty() {
        Ok(present)
    } else {
        Err(PreflightError(missing.join("; ")))
    }
}

/// Derive the **required** artifact set from config.
///
/// In P1, only the local SLM path (when set) is treated as a required on-disk
/// artifact. Cloud creds are validated at the call site (loud
/// [`crate::AiError::Unavailable`]), not here.
#[must_use]
pub fn required_artifacts(config: &AiConfig) -> Vec<RequiredArtifact> {
    let mut out = Vec::new();
    if let Some(path) = &config.local_model_path {
        out.push(RequiredArtifact {
            name: config.narrator_model.clone(),
            path: path.clone(),
        });
    }
    out
}

/// Convenience: run preflight for `config`'s required artifacts.
///
/// # Errors
/// Returns [`PreflightError`] if any required artifact is missing.
pub fn preflight(config: &AiConfig) -> PreflightResult {
    check_artifacts(&required_artifacts(config))
}
