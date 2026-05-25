//! civ-mod-host — manifest-only mod host stub (CIV-0700 Sprint D / v2).
//!
//! Loads and validates `manifest.toml` (or `mod.toml`) from a mod directory.
//! WASM sandboxing and capability enforcement are future work; v2 adds a
//! [`ModRegistry`] with a log-only policy-phase stub invoked from [`ModHost::tick`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

/// Supported mod kinds per CIV-0700 §4.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModType {
    /// Policy injection (Phase 3a).
    Policy,
    /// Economic ledger / production hooks.
    Economic,
    /// Scripted world events.
    Event,
    /// Scenario composition.
    Scenario,
}

/// `[mod]` table — required metadata.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModMeta {
    /// Stable id: `[a-z][a-z0-9-]{0,63}`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Semver string.
    pub version: String,
    /// CivLab API major version (integer, stored as string in TOML).
    pub api_version: String,
    /// One of policy | economic | event | scenario.
    pub mod_type: ModType,
    /// Mod author name or organisation.
    pub author: String,
    /// Short description (max 256 chars).
    pub description: String,
    /// Optional URL to the mod homepage.
    #[serde(default)]
    pub homepage: Option<String>,
    /// SPDX license identifier (e.g. `MIT`).
    #[serde(default)]
    pub license: Option<String>,
}

/// `[dependencies]` table.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModDependencies {
    /// Host API semver range (required).
    #[serde(rename = "civlab-api")]
    pub civlab_api: String,
    /// Optional peer-mod version constraints (`{ "other-mod" = "^1" }`).
    #[serde(default)]
    pub mods: Option<std::collections::BTreeMap<String, String>>,
}

/// `[permissions]` table — all fields optional in file; defaults are false.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct ModPermissions {
    /// Allow reading economy state.
    #[serde(default)]
    pub read_economy: bool,
    /// Allow reading climate state.
    #[serde(default)]
    pub read_climate: bool,
    /// Allow reading military state.
    #[serde(default)]
    pub read_military: bool,
    /// Allow reading diplomacy state.
    #[serde(default)]
    pub read_diplomacy: bool,
    /// Allow reading citizen state.
    #[serde(default)]
    pub read_citizens: bool,
    /// Allow writing policy state.
    #[serde(default)]
    pub write_policy: bool,
    /// Allow writing economy state.
    #[serde(default)]
    pub write_economy: bool,
    /// Allow emitting world events.
    #[serde(default)]
    pub write_events: bool,
    /// Allow modifying scenario configuration.
    #[serde(default)]
    pub write_scenario: bool,
    /// Allow fund-transfer operations.
    #[serde(default)]
    pub transfer_funds: bool,
}

/// `[runtime]` optional overrides.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModRuntime {
    /// Maximum WASM heap in megabytes (host cap: 64).
    pub memory_mb: Option<u32>,
    /// CPU budget in microseconds per tick (host cap: 50).
    pub cpu_us: Option<u32>,
}

/// Parsed manifest (TOML on disk).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModManifest {
    /// Required `[mod]` metadata table.
    #[serde(rename = "mod")]
    pub meta: ModMeta,
    /// Required `[dependencies]` table.
    pub dependencies: ModDependencies,
    /// Optional `[permissions]` table; defaults all to false.
    #[serde(default)]
    pub permissions: ModPermissions,
    /// Optional `[runtime]` overrides.
    #[serde(default)]
    pub runtime: Option<ModRuntime>,
}

/// Errors while loading or validating a manifest.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ManifestError {
    /// Filesystem or IO failure.
    #[error("failed to read manifest at {path}: {message}")]
    Io {
        /// Path attempted.
        path: PathBuf,
        /// Error detail.
        message: String,
    },
    /// TOML parse failure.
    #[error("failed to parse manifest at {path}: {message}")]
    Parse {
        /// Path attempted.
        path: PathBuf,
        /// Error detail.
        message: String,
    },
    /// Post-parse validation failure (CIV-0700 §4.2 subset).
    #[error("invalid manifest at {path}: {message}")]
    Validation {
        /// Path attempted.
        path: PathBuf,
        /// Error detail.
        message: String,
    },
}

/// Loaded mod entry kept by the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedMod {
    /// Directory containing the manifest.
    pub root: PathBuf,
    /// Parsed manifest.
    pub manifest: ModManifest,
}

/// Registry of loaded mod manifests (v2 stub — no WASM guests).
#[derive(Debug, Clone, Default)]
pub struct ModRegistry {
    mods: Vec<LoadedMod>,
}

impl ModRegistry {
    /// Empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Loaded mods in registration order.
    #[must_use]
    pub fn mods(&self) -> &[LoadedMod] {
        &self.mods
    }

    /// Register a loaded mod entry.
    pub fn register(&mut self, entry: LoadedMod) {
        self.mods.push(entry);
    }

    /// Policy-phase stub (Phase 3a): one log line per policy mod with `write_policy`.
    ///
    /// Format: `mod:{id}:policy_phase:tick={tick}` (WASM callbacks not invoked yet).
    #[must_use]
    pub fn on_policy_phase(&self, tick: u64) -> Vec<String> {
        self.mods
            .iter()
            .filter(|m| {
                m.manifest.meta.mod_type == ModType::Policy && m.manifest.permissions.write_policy
            })
            .map(|m| format!("mod:{}:policy_phase:tick={tick}", m.manifest.meta.id))
            .collect()
    }
}

/// In-process mod host (manifest-only MVP + v2 policy stub).
#[derive(Debug, Clone, Default)]
pub struct ModHost {
    registry: ModRegistry,
}

impl ModHost {
    /// Empty host with no mods registered.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Manifest registry backing this host.
    #[must_use]
    pub fn registry(&self) -> &ModRegistry {
        &self.registry
    }

    /// Mods currently registered.
    #[must_use]
    pub fn mods(&self) -> &[LoadedMod] {
        self.registry.mods()
    }

    /// Load `manifest.toml` from `mod_dir` and register it.
    pub fn load_manifest_dir(&mut self, mod_dir: impl AsRef<Path>) -> Result<(), ManifestError> {
        let mod_dir = mod_dir.as_ref();
        let manifest_path = mod_dir.join("manifest.toml");
        let manifest = load_manifest(&manifest_path)?;
        self.registry.register(LoadedMod {
            root: mod_dir.to_path_buf(),
            manifest,
        });
        Ok(())
    }

    /// Per-tick hook — runs the policy-phase stub (no WASM yet).
    #[must_use]
    pub fn tick(&self, sim_tick: u64) -> Vec<String> {
        self.registry.on_policy_phase(sim_tick)
    }
}

/// Load and validate a manifest file from `path`.
pub fn load_manifest(path: impl AsRef<Path>) -> Result<ModManifest, ManifestError> {
    let path = path.as_ref().to_path_buf();
    let contents = std::fs::read_to_string(&path).map_err(|e| ManifestError::Io {
        path: path.clone(),
        message: e.to_string(),
    })?;

    let manifest: ModManifest = toml::from_str(&contents).map_err(|e| ManifestError::Parse {
        path: path.clone(),
        message: e.to_string(),
    })?;

    validate_manifest(&manifest, &path)?;
    Ok(manifest)
}

fn validate_manifest(manifest: &ModManifest, path: &Path) -> Result<(), ManifestError> {
    let id = &manifest.meta.id;
    let valid_id = !id.is_empty()
        && id.as_bytes()[0].is_ascii_lowercase()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if !valid_id {
        return Err(ManifestError::Validation {
            path: path.to_path_buf(),
            message: format!("mod.id `{id}` must match [a-z][a-z0-9-]{{0,63}}"),
        });
    }

    if manifest.meta.description.len() > 256 {
        return Err(ManifestError::Validation {
            path: path.to_path_buf(),
            message: "mod.description must be at most 256 characters".into(),
        });
    }

    if manifest.meta.api_version.parse::<u32>().is_err() {
        return Err(ManifestError::Validation {
            path: path.to_path_buf(),
            message: format!(
                "mod.api_version `{}` must be a non-negative integer",
                manifest.meta.api_version
            ),
        });
    }

    if manifest.dependencies.civlab_api.trim().is_empty() {
        return Err(ManifestError::Validation {
            path: path.to_path_buf(),
            message: "dependencies.civlab-api must not be empty".into(),
        });
    }

    if let Some(runtime) = &manifest.runtime {
        if let Some(mb) = runtime.memory_mb {
            if mb > 64 {
                return Err(ManifestError::Validation {
                    path: path.to_path_buf(),
                    message: format!("runtime.memory_mb {mb} exceeds host maximum 64"),
                });
            }
        }
        if let Some(us) = runtime.cpu_us {
            if us > 50 {
                return Err(ManifestError::Validation {
                    path: path.to_path_buf(),
                    message: format!("runtime.cpu_us {us} exceeds host maximum 50"),
                });
            }
        }
    }

    Ok(())
}

/// Repo-relative path to `mods/example-policy` from this crate's manifest dir.
#[must_use]
pub fn example_policy_mod_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../mods/example-policy")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_example_policy_manifest() {
        let dir = example_policy_mod_dir();
        let manifest = load_manifest(dir.join("manifest.toml")).expect("example manifest");

        assert_eq!(manifest.meta.id, "example-policy");
        assert_eq!(manifest.meta.mod_type, ModType::Policy);
        assert!(manifest.permissions.read_economy);
        assert!(manifest.permissions.write_policy);
    }

    #[test]
    fn mod_registry_policy_phase_emits_log_lines() {
        let mut host = ModHost::new();
        host.load_manifest_dir(example_policy_mod_dir())
            .expect("load example mod");

        let lines = host.registry().on_policy_phase(42);
        assert_eq!(lines, vec!["mod:example-policy:policy_phase:tick=42"]);

        let via_tick = host.tick(99);
        assert_eq!(via_tick, vec!["mod:example-policy:policy_phase:tick=99"]);
    }

    #[test]
    fn policy_phase_skips_mods_without_write_policy() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("manifest.toml");
        std::fs::write(
            &path,
            r#"
[mod]
id = "read-only-policy"
name = "x"
version = "0.0.1"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
write_policy = false
"#,
        )
        .expect("write");

        let manifest = load_manifest(&path).expect("manifest");
        let mut registry = ModRegistry::new();
        registry.register(LoadedMod {
            root: dir.path().to_path_buf(),
            manifest,
        });

        assert!(registry.on_policy_phase(1).is_empty());
    }

    #[test]
    fn rejects_invalid_mod_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("manifest.toml");
        std::fs::write(
            &path,
            r#"
[mod]
id = "INVALID"
name = "x"
version = "0.0.1"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"
"#,
        )
        .expect("write");

        let err = load_manifest(&path).expect_err("bad id");
        assert!(matches!(err, ManifestError::Validation { .. }));
    }
}
