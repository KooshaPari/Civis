use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::building::{BuildingBlueprint, RecipeDefinition};
use crate::material::MaterialSpec;

/// Supported on-disk manifest encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModManifestFormat {
    /// JSON manifest.
    Json,
    /// RON manifest.
    Ron,
}

/// Static mod metadata.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModMetadata {
    /// Stable mod id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Version string.
    pub version: String,
    /// Author or organization.
    pub author: String,
    /// Human-readable description.
    pub description: String,
    /// Optional entrypoint file.
    #[serde(default)]
    pub entrypoint: Option<String>,
}

/// On-disk mod manifest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModManifest {
    /// Metadata block.
    #[serde(rename = "mod")]
    pub metadata: ModMetadata,
    /// Materials contributed by the mod.
    #[serde(default)]
    pub materials: Vec<MaterialSpec>,
    /// Buildings contributed by the mod.
    #[serde(default)]
    pub buildings: Vec<BuildingBlueprint>,
    /// Recipes contributed by the mod.
    #[serde(default)]
    pub recipes: Vec<RecipeDefinition>,
    /// Events the mod wants to subscribe to.
    #[serde(default)]
    pub events: Vec<SimulationEventFilter>,
}

/// Event subscription filter.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimulationEventFilter {
    /// Birth events.
    Birth,
    /// Death events.
    Death,
    /// Tech events.
    Tech,
}

/// Manifest loading error.
#[derive(Debug, Error)]
pub enum ManifestError {
    /// IO failure.
    #[error("failed to read manifest at {path}: {source}")]
    Io {
        /// Path attempted.
        path: PathBuf,
        /// Source error.
        source: std::io::Error,
    },
    /// Parse failure.
    #[error("failed to parse manifest at {path}: {message}")]
    Parse {
        /// Path attempted.
        path: PathBuf,
        /// Error detail.
        message: String,
    },
    /// Missing manifest file.
    #[error("no manifest found in {path}")]
    NotFound {
        /// Folder inspected.
        path: PathBuf,
    },
}

/// Load a single manifest file from disk.
pub fn load_manifest_file(path: impl AsRef<Path>) -> Result<ModManifest, ManifestError> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path).map_err(|source| ManifestError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_manifest(&contents, path)
}

/// Load every manifest in a `mods/` directory.
pub fn load_manifests_from_dir(path: impl AsRef<Path>) -> Result<Vec<ModManifest>, ManifestError> {
    let path = path.as_ref();
    let mut manifests = Vec::new();
    let entries = fs::read_dir(path).map_err(|source| ManifestError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| ManifestError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let file_path = entry.path();
        if file_path.is_dir() {
            continue;
        }
        if let Some(stem) = file_path.file_stem().and_then(|s| s.to_str()) {
            if stem.starts_with("manifest") || stem.starts_with("mod") {
                manifests.push(load_manifest_file(&file_path)?);
            }
        }
    }

    if manifests.is_empty() {
        return Err(ManifestError::NotFound {
            path: path.to_path_buf(),
        });
    }

    Ok(manifests)
}

fn parse_manifest(contents: &str, path: &Path) -> Result<ModManifest, ManifestError> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => serde_json::from_str(contents).map_err(|source| ManifestError::Parse {
            path: path.to_path_buf(),
            message: source.to_string(),
        }),
        Some("ron") => ron::from_str(contents).map_err(|source| ManifestError::Parse {
            path: path.to_path_buf(),
            message: source.to_string(),
        }),
        _ => serde_json::from_str(contents)
            .or_else(|json_err| ron::from_str(contents).map_err(|ron_err| (json_err, ron_err)))
            .map_err(|(json_err, ron_err)| ManifestError::Parse {
                path: path.to_path_buf(),
                message: format!("json: {json_err}; ron: {ron_err}"),
            }),
    }
}
