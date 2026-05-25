//! civ-mod-host — manifest-only mod host stub (CIV-0700 Sprint D / v2).
//!
//! Loads and validates `manifest.toml` (or `mod.toml`) from a mod directory.
//! WASM sandboxing and capability enforcement are future work; v2 adds a
//! [`ModRegistry`] with a log-only policy-phase stub invoked from [`ModHost::tick`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod wasm_guest;

use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;
use wasm_guest::{invoke_policy_tick, MOD_WASM_NAME};

pub use wasm_guest::{WasmGuestError, MOD_WASM_NAME as MOD_WASM_FILE};

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
    /// `.civmod` ZIP read or structure failure.
    #[error("failed to read civmod archive at {path}: {message}")]
    Archive {
        /// Archive path attempted.
        path: PathBuf,
        /// Error detail.
        message: String,
    },
}

/// Root manifest path inside a `.civmod` ZIP archive.
pub const CIVMOD_MANIFEST_NAME: &str = "manifest.toml";

/// `mod.loaded.v1` structured lifecycle record (FR-MOD-004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModLoadedRecord {
    /// Stable mod id from manifest.
    pub mod_id: String,
    /// Display name.
    pub mod_name: String,
    /// Semver string.
    pub version: String,
    /// Simulation tick at load time.
    pub tick: u64,
}

/// Loaded mod entry kept by the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedMod {
    /// Directory or `.civmod` archive path.
    pub root: PathBuf,
    /// Parsed manifest.
    pub manifest: ModManifest,
    /// Optional `mod.wasm` bytes when present beside manifest or in archive.
    pub wasm_bytes: Option<Vec<u8>>,
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

    /// Military-phase stub (P-W1): one log line per mod with `read_military`.
    ///
    /// Format: `mod:{id}:military_phase:tick={tick}` (WASM callbacks not invoked yet).
    #[must_use]
    pub fn on_military_phase(&self, tick: u64) -> Vec<String> {
        self.mods
            .iter()
            .filter(|m| m.manifest.permissions.read_military)
            .map(|m| format!("mod:{}:military_phase:tick={tick}", m.manifest.meta.id))
            .collect()
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

    /// Economic-phase stub (Phase 3b).
    #[must_use]
    pub fn on_economy_phase(&self, tick: u64) -> Vec<String> {
        self.mods
            .iter()
            .filter(|m| m.manifest.meta.mod_type == ModType::Economic)
            .map(|m| format!("mod:{}:economy_phase:tick={tick}", m.manifest.meta.id))
            .collect()
    }
}

/// In-process mod host (manifest + WASM guest execution).
#[derive(Debug, Clone, Default)]
pub struct ModHost {
    registry: ModRegistry,
    loaded_records: Vec<ModLoadedRecord>,
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

    /// `mod.loaded.v1` records emitted on successful loads.
    #[must_use]
    pub fn loaded_records(&self) -> &[ModLoadedRecord] {
        &self.loaded_records
    }

    /// `mod.loaded.v1` lifecycle strings (replay / watch consumers).
    #[must_use]
    pub fn loaded_events(&self) -> Vec<String> {
        self.loaded_records
            .iter()
            .map(format_mod_loaded_event)
            .collect()
    }

    /// Load `manifest.toml` from `mod_dir` and register it.
    pub fn load_manifest_dir(&mut self, mod_dir: impl AsRef<Path>) -> Result<(), ManifestError> {
        let mod_dir = mod_dir.as_ref();
        let manifest_path = mod_dir.join(CIVMOD_MANIFEST_NAME);
        let manifest = load_manifest(&manifest_path)?;
        let wasm_bytes = read_wasm_file(mod_dir.join(MOD_WASM_NAME));
        self.push_loaded(&manifest, 0);
        self.registry.register(LoadedMod {
            root: mod_dir.to_path_buf(),
            manifest,
            wasm_bytes,
        });
        Ok(())
    }

    /// Load `manifest.toml` from a `.civmod` ZIP archive and register it.
    pub fn load_civmod_archive(
        &mut self,
        archive_path: impl AsRef<Path>,
    ) -> Result<(), ManifestError> {
        let archive_path = archive_path.as_ref().to_path_buf();
        let (manifest, wasm_bytes) = read_civmod_archive(&archive_path)?;
        self.push_loaded(&manifest, 0);
        self.registry.register(LoadedMod {
            root: archive_path,
            manifest,
            wasm_bytes,
        });
        Ok(())
    }

    /// Military-phase hook (P-W1).
    #[must_use]
    pub fn military_tick(&self, sim_tick: u64) -> Vec<String> {
        self.registry.on_military_phase(sim_tick)
    }

    /// Per-tick hook — phase stubs + WASM `civlab_policy_tick` when loaded.
    #[must_use]
    pub fn tick(&self, sim_tick: u64) -> Vec<String> {
        let mut lines = self.registry.on_policy_phase(sim_tick);
        lines.extend(self.registry.on_economy_phase(sim_tick));
        for entry in self.registry.mods() {
            let Some(wasm) = entry.wasm_bytes.as_ref() else {
                continue;
            };
            if entry.manifest.meta.mod_type != ModType::Policy
                || !entry.manifest.permissions.write_policy
            {
                continue;
            }
            match invoke_policy_tick(wasm) {
                Ok(code) => lines.push(format!(
                    "mod:{}:wasm_policy_tick:tick={sim_tick}:code={code}",
                    entry.manifest.meta.id
                )),
                Err(err) => lines.push(format_mod_error_event(
                    &entry.manifest.meta.id,
                    sim_tick,
                    &err.to_string(),
                )),
            }
        }
        lines
    }

    fn push_loaded(&mut self, manifest: &ModManifest, tick: u64) {
        self.loaded_records.push(ModLoadedRecord {
            mod_id: manifest.meta.id.clone(),
            mod_name: manifest.meta.name.clone(),
            version: manifest.meta.version.clone(),
            tick,
        });
    }
}

/// Format a `mod.loaded.v1` lifecycle event (EVENT_TAXONOMY / FR-MOD-004).
#[must_use]
pub fn format_mod_loaded_event(record: &ModLoadedRecord) -> String {
    format!(
        "mod.loaded.v1 mod_id={} mod_name={} version={} tick={}",
        record.mod_id, record.mod_name, record.version, record.tick
    )
}

/// Format `mod.error.v1` for host-side guest failures.
#[must_use]
pub fn format_mod_error_event(mod_id: &str, tick: u64, message: &str) -> String {
    format!("mod.error.v1 mod_id={mod_id} tick={tick} message={message}")
}

fn read_wasm_file(path: PathBuf) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

/// Parse and validate manifest TOML from memory.
pub fn parse_manifest(contents: &str, path: &Path) -> Result<ModManifest, ManifestError> {
    let manifest: ModManifest = toml::from_str(contents).map_err(|e| ManifestError::Parse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    validate_manifest(&manifest, path)?;
    Ok(manifest)
}

/// Read and validate `manifest.toml` at the root of a `.civmod` ZIP archive.
pub fn read_manifest_from_civmod(archive_path: &Path) -> Result<ModManifest, ManifestError> {
    read_civmod_archive(archive_path).map(|(manifest, _)| manifest)
}

/// Read manifest and optional `mod.wasm` from a `.civmod` ZIP archive.
pub fn read_civmod_archive(
    archive_path: &Path,
) -> Result<(ModManifest, Option<Vec<u8>>), ManifestError> {
    let file = std::fs::File::open(archive_path).map_err(|e| ManifestError::Archive {
        path: archive_path.to_path_buf(),
        message: e.to_string(),
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| ManifestError::Archive {
        path: archive_path.to_path_buf(),
        message: e.to_string(),
    })?;

    let mut manifest_contents: Option<String> = None;
    let mut wasm_bytes: Option<Vec<u8>> = None;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| ManifestError::Archive {
            path: archive_path.to_path_buf(),
            message: e.to_string(),
        })?;
        let name = entry.name().to_string();
        if entry.is_dir() {
            continue;
        }
        if is_unsafe_zip_entry_name(&name) {
            return Err(ManifestError::Archive {
                path: archive_path.to_path_buf(),
                message: format!("unsafe zip entry path: {name}"),
            });
        }
        if name == CIVMOD_MANIFEST_NAME {
            let mut buf = String::new();
            entry
                .read_to_string(&mut buf)
                .map_err(|e| ManifestError::Archive {
                    path: archive_path.to_path_buf(),
                    message: e.to_string(),
                })?;
            manifest_contents = Some(buf);
        } else if name == MOD_WASM_NAME {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| ManifestError::Archive {
                    path: archive_path.to_path_buf(),
                    message: e.to_string(),
                })?;
            wasm_bytes = Some(buf);
        }
    }

    let contents = manifest_contents.ok_or_else(|| ManifestError::Archive {
        path: archive_path.to_path_buf(),
        message: format!("missing root {CIVMOD_MANIFEST_NAME} in archive"),
    })?;

    let manifest = parse_manifest(&contents, archive_path)?;
    Ok((manifest, wasm_bytes))
}

fn is_unsafe_zip_entry_name(name: &str) -> bool {
    name.contains("..")
        || name.starts_with('/')
        || name.starts_with('\\')
        || name.contains('\\')
        || name.contains('/')
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
    use std::io::Write;

    use super::*;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    const MINIMAL_POLICY_MANIFEST: &str = r#"
[mod]
id = "zip-policy"
name = "ZIP Policy"
version = "0.1.0"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
write_policy = true
"#;

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
    fn mod_registry_military_phase_emits_for_read_military() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("manifest.toml");
        std::fs::write(
            &path,
            r#"
[mod]
id = "mil-observer"
name = "Military Observer"
version = "0.0.1"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
read_military = true
write_policy = false
"#,
        )
        .expect("write");

        let mut host = ModHost::new();
        host.load_manifest_dir(dir.path()).expect("load mod");

        let lines = host.registry().on_military_phase(7);
        assert_eq!(lines, vec!["mod:mil-observer:military_phase:tick=7"]);
        assert_eq!(
            host.military_tick(8),
            vec!["mod:mil-observer:military_phase:tick=8"]
        );
    }

    #[test]
    fn mod_registry_policy_phase_emits_log_lines() {
        let mut host = ModHost::new();
        host.load_manifest_dir(example_policy_mod_dir())
            .expect("load example mod");

        let lines = host.registry().on_policy_phase(42);
        assert_eq!(lines, vec!["mod:example-policy:policy_phase:tick=42"]);

        let via_tick = host.tick(99);
        assert!(via_tick
            .iter()
            .any(|l| l == "mod:example-policy:policy_phase:tick=99"));
    }

    #[test]
    fn wasm_policy_tick_invokes_civlab_export() {
        const WAT: &str = r#"
            (module
              (func (export "civlab_policy_tick") (result i32)
                i32.const 42)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        assert_eq!(invoke_policy_tick(&wasm).expect("invoke"), 42);
    }

    #[test]
    fn load_civmod_with_wasm_invokes_on_tick() {
        let dir = tempfile::tempdir().expect("tempdir");
        let civmod = dir.path().join("policy.civmod");
        const WAT: &str = r#"
            (module
              (func (export "civlab_policy_tick") (result i32)
                i32.const 7)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let file = std::fs::File::create(&civmod).expect("create");
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file(CIVMOD_MANIFEST_NAME, options)
            .expect("manifest");
        zip.write_all(MINIMAL_POLICY_MANIFEST.as_bytes())
            .expect("write");
        zip.start_file(MOD_WASM_NAME, options).expect("wasm");
        zip.write_all(&wasm).expect("write wasm");
        zip.finish().expect("finish");

        let mut host = ModHost::new();
        host.load_civmod_archive(&civmod).expect("load");
        let lines = host.tick(1);
        assert!(lines
            .iter()
            .any(|l| l.contains("wasm_policy_tick") && l.contains("code=7")));
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
            wasm_bytes: None,
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
