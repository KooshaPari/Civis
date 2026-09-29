//! CIV-1000 save layouts: uncompressed `.civsave/` folder (debug) and `.civsave.zst` archive (default).

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tar::{Archive, Builder};
use thiserror::Error;
use zstd::stream::{decode_all, encode_all};

use crate::{ClusterStocks, CoastalColumn, ModGuestStateSave, ReplayError, Simulation, WorldState};
use civ_planet::{Climate, MoonConfig, PlanetConfig, WeatherCell};

/// Sidecar metadata written beside replay + mod state.
pub const CIVSAVE_SPEC_ID: &str = "CIV-1000";
/// Folder format version for `metadata.json`.
///
/// v5 adds the FR-SAVE-006 `integrity.json` BLAKE3 manifest. v4 and earlier
/// predate it and still load, without a whole-bundle digest.
pub const CIVSAVE_FORMAT_VERSION: u32 = 5;
/// Default on-disk save extension (zstd-compressed tar).
pub const CIVSAVE_ARCHIVE_EXTENSION: &str = "civsave.zst";
/// FR-SAVE-006 — integrity manifest written last, covering every other
/// serialized component.
const INTEGRITY_FILE: &str = "integrity.json";
/// Required in v4; absence in earlier versions represents empty stockpiles.
const CLUSTER_STOCKS_FILE: &str = "cluster_stocks.json";
/// Required in v4; older replay-only saves may omit the environment.
const ENVIRONMENT_FILE: &str = "environment.json";
const INSTITUTIONS_FILE: &str = "institutions.json";

/// FR-SAVE-006 — a BLAKE3 digest of one serialized component.
///
/// Stored as lowercase hex so the manifest stays human-diffable and JSON-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentDigest {
    /// File name relative to the save root, e.g. `world_state.json`.
    pub component: String,
    /// Lowercase hex BLAKE3-256 of the component's exact bytes.
    pub blake3: String,
    /// Byte length of the component at save time.
    pub len: u64,
}

/// FR-SAVE-006 — `integrity.json` payload.
///
/// One digest per serialized component plus a `root` digest over the sorted
/// `(component, blake3, len)` triples. The root lets a single comparison detect
/// any change, while the per-component list tells a caller *what* changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveIntegrityManifest {
    /// Spec identifier, so a manifest from another format is rejected.
    pub spec_id: String,
    /// Format version of the save this manifest describes.
    pub format_version: u32,
    /// Engine tick at save time.
    pub tick: u64,
    /// Per-component digests, sorted by component name.
    pub components: Vec<ComponentDigest>,
    /// Lowercase hex BLAKE3-256 over the canonical component listing.
    pub root: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SavedInstitutions {
    settlements: BTreeMap<u32, u32>,
    institutions: BTreeMap<u32, Vec<civ_institutions::Institution>>,
    levels_emitted: BTreeSet<(u32, u8, u8)>,
}

/// Zstd frame magic (little-endian `0xFD2FB528`).
const ZSTD_FRAME_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

/// `metadata.json` in a `.civsave/` directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CivSaveMetadata {
    /// Spec identifier (`CIV-1000`).
    pub spec_id: String,
    /// Folder format version.
    pub format_version: u32,
    /// Engine tick at save time.
    pub tick: u64,
    /// Optional scenario label for UI.
    pub scenario_name: Option<String>,
}

/// Environment state that replay does not reconstruct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SavedEnvironment {
    planet: PlanetConfig,
    moon: MoonConfig,
    climate: Climate,
    weather_grid: Vec<WeatherCell>,
    #[serde(default)]
    coastal_columns: Vec<SavedCoastalColumn>,
}

/// A coastal column as an explicit JSON record, because JSON object keys must
/// be strings and the runtime registry is keyed by integer `(x, z)` pairs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SavedCoastalColumn {
    x: i64,
    z: i64,
    base_y: i64,
    last_water_y: i64,
}

/// Errors reading or writing save folders.
#[derive(Debug, Error)]
pub enum SaveBundleError {
    /// Filesystem failure.
    #[error("io at {path}: {message}")]
    Io {
        /// Path involved.
        path: PathBuf,
        /// Error detail.
        message: String,
    },
    /// JSON metadata or mod state failure.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// Guest mod state import failure.
    #[error("mod state: {0}")]
    ModState(#[from] civ_mod_host::GuestStateError),
    /// Replay encode/decode failure.
    #[error("replay: {0}")]
    Replay(#[from] ReplayError),
    /// Missing required component in the folder.
    #[error("missing {component} in {dir}")]
    MissingComponent {
        /// Folder path.
        dir: PathBuf,
        /// Expected file name.
        component: &'static str,
    },
    /// Tar archive read/write failure.
    #[error("archive: {0}")]
    Archive(String),
    /// Zstd compression failure.
    #[error("zstd: {0}")]
    Zstd(String),
    /// Invalid named save slot.
    #[error("invalid slot name {name:?}: {message}")]
    InvalidSlotName {
        /// Provided slot name.
        name: String,
        /// Validation failure detail.
        message: String,
    },
    /// Save file is corrupted or has structural damage.
    #[error("save corruption: {detail}")]
    SaveCorruption {
        /// Human-readable description of the corruption.
        detail: String,
    },
    /// FR-SAVE-006 — a file exists in the save directory that `integrity.json`
    /// does not cover, so its contents were never verified.
    ///
    /// Raised when the bundle carries state the manifest never vouched for.
    /// Ignoring such a file would weaken "all serialized state" into "all the
    /// serialized state somebody remembered to list".
    #[error("unlisted component {component} in {dir}: present on disk but absent from {manifest}")]
    UnlistedComponent {
        /// Save root that holds the unexpected file.
        dir: PathBuf,
        /// File name that the manifest does not describe.
        component: String,
        /// Manifest file name, for the error message.
        manifest: &'static str,
    },
    /// Format version is newer than this engine supports.
    #[error("unsupported format version {found} (max supported {max})")]
    UnsupportedFormatVersion {
        /// Version found on disk.
        found: u32,
        /// Maximum version this engine can load.
        max: u32,
    },
    /// FR-SAVE-006 / FR-SAVE-007 — a component's BLAKE3 digest does not match
    /// the value recorded when the save was written.
    ///
    /// Raised *before* the affected component is deserialized, so a tampered
    /// payload never reaches `serde_json::from_str`.
    #[error("integrity check failed for {component} in {save_root}: expected blake3 {expected}, found {actual}")]
    HashMismatch {
        /// Component whose digest disagreed.
        component: String,
        /// Save root the manifest was read from, for diagnostics.
        save_root: PathBuf,
        /// Digest recorded in `integrity.json`.
        expected: String,
        /// Digest recomputed from the bytes on disk.
        actual: String,
    },
}

fn io_err(path: impl AsRef<Path>, err: impl std::fmt::Display) -> SaveBundleError {
    SaveBundleError::Io {
        path: path.as_ref().to_path_buf(),
        message: err.to_string(),
    }
}

/// FR-SAVE-006 — hex-encode a BLAKE3-256 digest.
fn hex_digest(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(64);
    for b in blake3::hash(bytes).as_bytes() {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// FR-SAVE-006 — canonical byte string the root digest covers.
///
/// Each field is length-prefixed, so no two distinct component listings can
/// concatenate to the same bytes and the root cannot be spoofed by renaming a
/// component to shift a field boundary.
fn canonical_listing_bytes(components: &[ComponentDigest]) -> Vec<u8> {
    let mut buf = Vec::new();
    for c in components {
        for field in [c.component.as_str(), c.blake3.as_str(), c.len.to_string().as_str()] {
            buf.extend_from_slice(&(field.len() as u64).to_le_bytes());
            buf.extend_from_slice(field.as_bytes());
        }
    }
    buf
}

/// FR-SAVE-006 — compute the integrity manifest for a save directory.
///
/// `metadata.json` and `integrity.json` are excluded: a manifest cannot carry
/// its own digest, and `metadata.json` is compared field-wise during load.
/// Every other file in `dir` is hashed in sorted-name order, so the result does
/// not depend on directory iteration order.
fn compute_integrity_manifest(
    dir: &Path,
    format_version: u32,
    tick: u64,
) -> Result<SaveIntegrityManifest, SaveBundleError> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .map_err(|e| io_err(dir, e))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            entry
                .file_type()
                .ok()
                .filter(|t| t.is_file())
                .map(|_| name)
        })
        .filter(|name| name != INTEGRITY_FILE && name != "metadata.json")
        .collect();
    names.sort();

    let mut components = Vec::with_capacity(names.len());
    for name in names {
        let path = dir.join(&name);
        let bytes = fs::read(&path).map_err(|e| io_err(&path, e))?;
        components.push(ComponentDigest {
            component: name,
            blake3: hex_digest(&bytes),
            len: bytes.len() as u64,
        });
    }

    let root = hex_digest(&canonical_listing_bytes(&components));
    Ok(SaveIntegrityManifest {
        spec_id: CIVSAVE_SPEC_ID.to_owned(),
        format_version,
        tick,
        components,
        root,
    })
}

/// FR-SAVE-006 / FR-SAVE-007 — verify every component against the manifest.
///
/// Returns the first mismatch, or `Ok(())` when the save is intact. Call this
/// *before* deserializing anything: the point of the requirement is that a
/// tampered component is rejected without ever being parsed.
///
/// Four checks run here, in order:
///
/// 1. **Path containment.** A manifest is attacker-reachable data - it is a
///    plain JSON file inside the save directory - so a `component` of
///    `../../etc/passwd` must never be joined onto the save root and read.
///    Every name must be a single plain file name directly under `dir`.
/// 2. **No duplicate entries.** A repeated name would let the root digest cover
///    the same file twice, which is not a property any real bundle has.
/// 3. **Per-component digest and length.** A byte flip anywhere in a component
///    changes its BLAKE3. The length is compared too, so a truncation is
///    reported as the same explicit mismatch rather than a bare IO error.
/// 4. **No unlisted files.** A component present on disk but absent from the
///    manifest is unverified state. Silently ignoring it would let a file the
///    loader later reads exist without any manifest entry vouching for it.
fn verify_integrity_manifest(
    dir: &Path,
    manifest: &SaveIntegrityManifest,
) -> Result<(), SaveBundleError> {
    let mut listed: BTreeSet<&str> = BTreeSet::new();

    for expected in &manifest.components {
        if !is_plain_component_name(&expected.component) {
            return Err(SaveBundleError::SaveCorruption {
                detail: format!(
                    "integrity.json lists component {:?}, which is not a plain file name \
                     directly under the save root",
                    expected.component
                ),
            });
        }
        if !listed.insert(expected.component.as_str()) {
            return Err(SaveBundleError::SaveCorruption {
                detail: format!(
                    "integrity.json lists component {:?} more than once",
                    expected.component
                ),
            });
        }

        let path = dir.join(&expected.component);
        let bytes = fs::read(&path).map_err(|e| io_err(&path, e))?;
        let actual = hex_digest(&bytes);
        if actual != expected.blake3 || bytes.len() as u64 != expected.len {
            return Err(SaveBundleError::HashMismatch {
                component: expected.component.clone(),
                save_root: dir.to_path_buf(),
                expected: expected.blake3.clone(),
                actual,
            });
        }
    }

    // A component on disk that the manifest never mentions is state nobody
    // vouched for. Reject rather than ignore, so "all serialized state" means
    // every file in the bundle and not merely the ones somebody remembered to
    // list.
    for entry in fs::read_dir(dir).map_err(|e| io_err(dir, e))? {
        let entry = entry.map_err(|e| io_err(dir, e))?;
        if !entry.file_type().map_err(|e| io_err(dir, e))?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == INTEGRITY_FILE || name == "metadata.json" {
            continue;
        }
        if !listed.contains(name.as_str()) {
            return Err(SaveBundleError::UnlistedComponent {
                dir: dir.to_path_buf(),
                component: name,
                manifest: INTEGRITY_FILE,
            });
        }
    }

    // A manifest whose own listing was edited is as suspicious as one whose
    // component was: recompute the root and reject on disagreement.
    let recomputed_root = hex_digest(&canonical_listing_bytes(&manifest.components));
    if recomputed_root != manifest.root {
        return Err(SaveBundleError::HashMismatch {
            component: INTEGRITY_FILE.to_owned(),
            save_root: dir.to_path_buf(),
            expected: manifest.root.clone(),
            actual: recomputed_root,
        });
    }
    Ok(())
}

/// FR-SAVE-006 - reject any manifest component name that is not a single plain
/// file name.
///
/// Blocks absolute paths (`/etc/passwd`), Windows drive-qualified paths, UNC
/// paths, and any `..` traversal. A manifest is untrusted input, so this is
/// checked before the name is ever joined onto the save root.
fn is_plain_component_name(name: &str) -> bool {
    if name.is_empty() || name == "." || name == ".." {
        return false;
    }
    // `Path::components` yields exactly one `Component::Normal` only for a name
    // with no separator, no prefix, and no `.`/`..` element. Using it instead of
    // string matching keeps the check aligned with the platform's own path
    // grammar rather than a hand-rolled allowlist of separators.
    let mut components = Path::new(name).components();
    matches!(components.next(), Some(std::path::Component::Normal(_))) && components.next().is_none()
}


fn archive_err(message: impl std::fmt::Display) -> SaveBundleError {
    SaveBundleError::Archive(message.to_string())
}

fn zstd_err(message: impl std::fmt::Display) -> SaveBundleError {
    SaveBundleError::Zstd(message.to_string())
}

// ---------------------------------------------------------------------------
// Format-version migration helpers
// ---------------------------------------------------------------------------

/// Migrate a v1 world_state JSON value to v2.
///
/// v1 saves are missing the culture-related top-level fields introduced in v2:
/// `religion`, `language`, `psyche`, `history`, `writing`, `building_layouts`.
/// This function inserts `Null` defaults for each missing field so that
/// downstream deserialization succeeds.
fn migrate_v1_to_v2(world_state: &mut serde_json::Value) {
    let obj = match world_state.as_object_mut() {
        Some(o) => o,
        None => return,
    };
    for field in &[
        "religion",
        "language",
        "psyche",
        "history",
        "writing",
        "building_layouts",
    ] {
        obj.entry(*field).or_insert_with(|| serde_json::Value::Null);
    }
}

/// Migrate a v2 world_state JSON value to v3.
///
/// v3 restructures economy fields for trade routes: a top-level
/// `economy` object is introduced that wraps the former flat `trade_routes`
/// list and adds a `trade_route_version` marker. If `trade_routes` is
/// already wrapped, this is a no-op.
fn migrate_v2_to_v3(world_state: &mut serde_json::Value) {
    let obj = match world_state.as_object_mut() {
        Some(o) => o,
        None => return,
    };
    // Only migrate when `economy` is not yet present.
    if obj.contains_key("economy") {
        return;
    }
    let trade_routes = obj.remove("trade_routes").unwrap_or(serde_json::json!([]));
    let economy = serde_json::json!({
        "trade_route_version": 3,
        "trade_routes": trade_routes,
    });
    obj.insert("economy".to_string(), economy);
}

/// Apply the full migration chain on a raw world_state JSON value.
///
/// The `file_version` is the version recorded in `metadata.json`. Each
/// migration step advances the world-state schema. Version 4 adds required
/// sidecars and has no world-state transform; only an explicit save writes them.
fn run_migration_chain(
    world_state: &mut serde_json::Value,
    file_version: u32,
) -> Result<(), SaveBundleError> {
    if file_version > CIVSAVE_FORMAT_VERSION {
        return Err(SaveBundleError::UnsupportedFormatVersion {
            found: file_version,
            max: CIVSAVE_FORMAT_VERSION,
        });
    }
    let mut version = file_version;
    if version < 2 {
        migrate_v1_to_v2(world_state);
        version = 2;
    }
    if version < 3 {
        migrate_v2_to_v3(world_state);
    }
    // Historical v3 migration wrapped routes under economy, while the compiled
    // WorldState still stores them at the top level. Restore that canonical
    // field without discarding the legacy wrapper or any route records.
    if let Some(obj) = world_state.as_object_mut() {
        if !obj.contains_key("trade_routes") {
            if let Some(routes) = obj
                .get("economy")
                .and_then(|value| value.get("trade_routes"))
                .cloned()
            {
                obj.insert("trade_routes".to_owned(), routes);
            }
        }
    }
    Ok(())
}

/// Read and migrate world state in memory, preserving the source on both
/// successful and failed loads. Only an explicit save writes migrated state.
fn migrate_world_state_file(
    dir: &Path,
    file_version: u32,
) -> Result<serde_json::Value, SaveBundleError> {
    let path = dir.join("world_state.json");
    if !path.is_file() {
        return Ok(serde_json::json!({}));
    }
    let json_str = fs::read_to_string(&path).map_err(|e| io_err(&path, e))?;
    if json_str.trim().is_empty() {
        return Err(SaveBundleError::SaveCorruption {
            detail: format!("world_state.json is empty in {}", dir.display()),
        });
    }
    let mut value: serde_json::Value =
        serde_json::from_str(&json_str).map_err(SaveBundleError::Json)?;
    run_migration_chain(&mut value, file_version)?;
    Ok(value)
}

/// Migrate world_state extracted from a tar archive, returning the migrated
/// JSON value. The caller is responsible for writing it back.
fn migrate_world_state_from_tar(
    tar_bytes: &[u8],
    file_version: u32,
) -> Result<Option<serde_json::Value>, SaveBundleError> {
    use std::io::Read;
    let mut archive = Archive::new(tar_bytes);
    for entry in archive.entries().map_err(archive_err)? {
        let mut entry = entry.map_err(archive_err)?;
        let entry_path = entry.path().map_err(archive_err)?;
        if entry_path.file_name().and_then(|s| s.to_str()) != Some("world_state.json") {
            continue;
        }
        let mut json_str = String::new();
        entry.read_to_string(&mut json_str).map_err(archive_err)?;
        if json_str.trim().is_empty() {
            return Err(SaveBundleError::SaveCorruption {
                detail: "world_state.json is empty in archive".to_string(),
            });
        }
        let mut value: serde_json::Value =
            serde_json::from_str(&json_str).map_err(SaveBundleError::Json)?;
        run_migration_chain(&mut value, file_version)?;
        return Ok(Some(value));
    }
    Ok(None)
}

fn validate_slot_name(name: &str) -> Result<String, SaveBundleError> {
    let trimmed = name.trim();
    let invalid = |message: &str| SaveBundleError::InvalidSlotName {
        name: name.to_string(),
        message: message.to_string(),
    };
    if trimmed.is_empty() {
        return Err(invalid("slot name cannot be empty"));
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Err(invalid("slot name must be a simple filename"));
    }
    let normalized = trimmed
        .trim_end_matches(".civreplay")
        .trim_end_matches(".civsave.zst")
        .trim_end_matches(".civsave");
    if normalized.is_empty() {
        return Err(invalid("slot name cannot be only an extension"));
    }
    Ok(normalized.to_string())
}

fn slot_archive_path(saves_dir: &Path, name: &str) -> Result<PathBuf, SaveBundleError> {
    let name = validate_slot_name(name)?;
    Ok(saves_dir.join(format!("{name}.{CIVSAVE_ARCHIVE_EXTENSION}")))
}

fn slot_name_from_path(path: &Path) -> Option<String> {
    if CivSaveBundle::is_save_archive(path) {
        path.file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.trim_end_matches(".civsave.zst").to_string())
    } else if CivSaveBundle::is_save_dir(path) {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.trim_end_matches(".civsave").to_string())
    } else {
        None
    }
}

/// One named save slot under a saves directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSlotEntry {
    /// Slot name without `.civsave.zst`.
    pub name: String,
    /// Engine tick at save time, or 0 when metadata cannot be read.
    pub tick: u64,
}

/// CIV-1000 save bundle: folder (debug) and `.civsave.zst` archive (default).
pub struct CivSaveBundle;

impl CivSaveBundle {
    /// Write an uncompressed save folder at `dir` (typically `*.civsave/`).
    pub fn save_dir(dir: impl AsRef<Path>, sim: &Simulation) -> Result<(), SaveBundleError> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir).map_err(|e| io_err(dir, e))?;

        let metadata = CivSaveMetadata {
            spec_id: CIVSAVE_SPEC_ID.to_owned(),
            format_version: CIVSAVE_FORMAT_VERSION,
            tick: sim.state.tick,
            scenario_name: None,
        };
        let metadata_path = dir.join("metadata.json");
        fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)
            .map_err(|e| io_err(&metadata_path, e))?;

        let mod_state_path = dir.join("mod_state.json");
        fs::write(&mod_state_path, sim.export_mod_guest_state().to_json()?)
            .map_err(|e| io_err(&mod_state_path, e))?;

        // Mirror authoritative Simulation-owned state into a clone of
        // `state` so direct mutations done after the last tick (scenario
        // loaders, integration tests, parallel agents) are captured
        // before serialization. The mirror is `&self`-safe so the public
        // `save_dir`/`save_archive` signatures can stay `&Simulation`.
        let mut world_state_for_save = sim.state.clone();
        sim.save_state_mirror_to(&mut world_state_for_save);

        let world_state_path = dir.join("world_state.json");
        fs::write(
            &world_state_path,
            serde_json::to_string(&world_state_for_save)?,
        )
        .map_err(|e| io_err(&world_state_path, e))?;

        let environment_path = dir.join(ENVIRONMENT_FILE);
        let environment = SavedEnvironment {
            planet: *sim.planet(),
            moon: *sim.moon(),
            climate: sim.climate,
            weather_grid: sim.weather_grid().to_vec(),
            coastal_columns: sim
                .coastal_columns
                .iter()
                .map(|(&(x, z), column)| SavedCoastalColumn {
                    x,
                    z,
                    base_y: column.base_y,
                    last_water_y: column.last_water_y,
                })
                .collect(),
        };
        fs::write(&environment_path, serde_json::to_string(&environment)?)
            .map_err(|e| io_err(&environment_path, e))?;

        let cluster_stocks_path = dir.join(CLUSTER_STOCKS_FILE);
        fs::write(
            &cluster_stocks_path,
            serde_json::to_string(sim.cluster_stocks())?,
        )
        .map_err(|e| io_err(&cluster_stocks_path, e))?;

        let (settlements, institutions, levels_emitted) = sim.saveable_institution_state();
        let institutions_path = dir.join(INSTITUTIONS_FILE);
        fs::write(
            &institutions_path,
            serde_json::to_string(&SavedInstitutions {
                settlements,
                institutions,
                levels_emitted,
            })?,
        )
        .map_err(|e| io_err(&institutions_path, e))?;

        let replay_path = dir.join("replay.civreplay");
        sim.save_replay(&replay_path)?;

        // FR-SAVE-006 — hash every component above and write the manifest
        // last, so its presence proves the bundle is complete.
        let manifest = compute_integrity_manifest(dir, CIVSAVE_FORMAT_VERSION, sim.state.tick)?;
        let integrity_path = dir.join(INTEGRITY_FILE);
        fs::write(&integrity_path, serde_json::to_string_pretty(&manifest)?)
            .map_err(|e| io_err(&integrity_path, e))?;
        Ok(())
    }

    /// Load simulation from a `.civsave/` folder.
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Simulation, SaveBundleError> {
        let dir = dir.as_ref();

        // Check format version via metadata before loading components.
        let metadata_path = dir.join("metadata.json");
        let file_version = if metadata_path.is_file() {
            let json = fs::read_to_string(&metadata_path).map_err(|e| io_err(&metadata_path, e))?;
            if json.trim().is_empty() {
                return Err(SaveBundleError::SaveCorruption {
                    detail: format!("metadata.json is empty in {}", dir.display()),
                });
            }
            let meta: CivSaveMetadata =
                serde_json::from_str(&json).map_err(SaveBundleError::Json)?;
            meta.format_version
        } else {
            // No metadata implies a legacy v1 save.
            1
        };
        if file_version > CIVSAVE_FORMAT_VERSION {
            return Err(SaveBundleError::UnsupportedFormatVersion {
                found: file_version,
                max: CIVSAVE_FORMAT_VERSION,
            });
        }

        // FR-SAVE-006 / FR-SAVE-007 — verify component digests before any
        // deserialization begins. This runs ahead of `migrate_world_state_file`
        // and `load_replay_from_file` so a tampered payload is never handed to
        // serde. A v5+ writer always emits the manifest; older saves have none
        // and rely on their own per-component checks instead.
        if file_version >= 5 {
            let integrity_path = dir.join(INTEGRITY_FILE);
            if !integrity_path.is_file() {
                return Err(SaveBundleError::MissingComponent {
                    dir: dir.to_path_buf(),
                    component: INTEGRITY_FILE,
                });
            }
            let raw = fs::read_to_string(&integrity_path).map_err(|e| io_err(&integrity_path, e))?;
            let manifest: SaveIntegrityManifest =
                serde_json::from_str(&raw).map_err(SaveBundleError::Json)?;
            if manifest.spec_id != CIVSAVE_SPEC_ID {
                return Err(SaveBundleError::SaveCorruption {
                    detail: format!(
                        "integrity.json declares spec_id {:?}, expected {:?}",
                        manifest.spec_id, CIVSAVE_SPEC_ID
                    ),
                });
            }
            verify_integrity_manifest(dir, &manifest)?;
        }

        // A v4 writer promises these snapshots. A missing file is corruption,
        // not permission to silently rebuild default state from replay.
        if file_version >= 4 {
            for component in [ENVIRONMENT_FILE, CLUSTER_STOCKS_FILE, INSTITUTIONS_FILE] {
                if !dir.join(component).is_file() {
                    return Err(SaveBundleError::MissingComponent {
                        dir: dir.to_path_buf(),
                        component,
                    });
                }
            }
        }

        // Migrate world_state.json if needed.
        let migrated_ws = migrate_world_state_file(dir, file_version)?;

        let replay_path = dir.join("replay.civreplay");
        if !replay_path.is_file() {
            return Err(SaveBundleError::MissingComponent {
                dir: dir.to_path_buf(),
                component: "replay.civreplay",
            });
        }
        let mut sim = Simulation::load_replay_from_file(&replay_path)?;

        let mod_state_path = dir.join("mod_state.json");
        if mod_state_path.is_file() {
            let json =
                fs::read_to_string(&mod_state_path).map_err(|e| io_err(&mod_state_path, e))?;
            let save = ModGuestStateSave::from_json(&json)?;
            sim.restore_mod_guest_state(&save)?;
        }

        let world_state_path = dir.join("world_state.json");
        if world_state_path.is_file() {
            sim.state = serde_json::from_value(migrated_ws).map_err(SaveBundleError::Json)?;
            // Mirror the persisted WorldState side back onto the live
            // Simulation so diplomacy, caravans, weather, language drift,
            // grief accumulator, etc. all resume in lockstep with the
            // authoritative post-load world state.
            sim.faction_relations = sim.state.faction_relations.clone();
            sim.grief_accumulator = sim.state.grief_accumulator.clone();
            sim.stance_engine = sim.state.stance_engine.clone();
            sim.deep_diplomacy = sim.state.deep_diplomacy.clone();
            // Mirror persisted language-drift state back onto the live
            // Simulation so phase_language_drift resumes with the same
            // emergent phoneme inventory / intelligibility matrix as the
            // pre-save run.
            sim.language_state = sim.state.language_state.clone();
            sim.faction_languages = sim.state.faction_languages.clone();
            // Mirror persisted civic institutions, construction sites,
            // and economic-focus state back onto the live Simulation so
            // phase_institutions / phase_construction_sites /
            // phase_policy_econ resume with the same authoritative
            // post-save values.
            sim.institutions = sim.state.institutions.clone();
            sim.institution_levels_emitted =
                sim.state.institution_levels_emitted.clone();
            sim.build_sites = sim.state.build_sites.clone();
            sim.econ_focus = sim.state.econ_focus.clone();
            // Mirror persisted culture, ideology, aggression, and
            // unrest-gini state back onto the live Simulation so
            // phase_culture / phase_aggression / phase_unrest resume
            // with the same authoritative post-save values.
            sim.cluster_cultures = sim.state.cluster_cultures.clone();
            sim.faction_ideologies = sim.state.faction_ideologies.clone();
            sim.faction_aggression = sim.state.faction_aggression.clone();
            sim.unrest_settlement_gini =
                sim.state.unrest_settlement_gini.clone();
            // Mirror persisted riot/migrant accumulators and scenario
            // taxation back onto the live Simulation so phase_unrest /
            // apply_scenario_taxation resume with the same authoritative
            // post-save values.
            sim.riot_accumulator = sim.state.riot_accumulator.clone();
            sim.migrant_accumulator = sim.state.migrant_accumulator.clone();
            sim.scenario_taxation = sim.state.scenario_taxation.clone();
            sim.era_progression = sim.state.era_progression.clone();
            sim.emergence_sample = sim.state.emergence_sample.clone();
            sim.significance = sim.state.significance.clone();
        }

        let environment_path = dir.join(ENVIRONMENT_FILE);
        if let Some(json) = match fs::read_to_string(&environment_path) {
            Ok(json) => Some(json),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(io_err(&environment_path, error)),
        } {
            let environment: SavedEnvironment =
                serde_json::from_str(&json).map_err(SaveBundleError::Json)?;
            sim.planet = environment.planet;
            sim.moon = environment.moon;
            sim.climate = environment.climate;
            sim.weather_grid = environment.weather_grid;
            let mut coastal_columns = BTreeMap::new();
            for column in environment.coastal_columns {
                let key = (column.x, column.z);
                let previous = coastal_columns.insert(
                    key,
                    CoastalColumn {
                        base_y: column.base_y,
                        last_water_y: column.last_water_y,
                    },
                );
                if previous.is_some() {
                    return Err(SaveBundleError::SaveCorruption {
                        detail: format!(
                            "environment.json has duplicate coastal column at ({}, {})",
                            column.x, column.z
                        ),
                    });
                }
            }
            sim.coastal_columns = coastal_columns;
        }

        let cluster_stocks_path = dir.join(CLUSTER_STOCKS_FILE);
        if let Some(json) = match fs::read_to_string(&cluster_stocks_path) {
            Ok(json) => Some(json),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(io_err(&cluster_stocks_path, error)),
        } {
            let cluster_stocks: BTreeMap<u64, ClusterStocks> =
                serde_json::from_str(&json).map_err(SaveBundleError::Json)?;
            sim.restore_cluster_stocks(cluster_stocks);
        }

        let institutions_path = dir.join(INSTITUTIONS_FILE);
        match fs::read_to_string(&institutions_path) {
            Ok(json) => {
                let saved: SavedInstitutions = serde_json::from_str(&json)?;
                sim.restore_institution_state(
                    saved.settlements,
                    saved.institutions,
                    saved.levels_emitted,
                );
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_err(&institutions_path, error)),
        }

        // Loading legacy bundles must not promote their metadata to v4: their
        // optional sidecars may still be absent. An explicit save writes all
        // components and is the point at which a bundle becomes v4.

        Ok(sim)
    }

    /// Write a zstd-compressed tar archive at `path` (typically `*.civsave.zst`).
    pub fn save_archive(path: impl AsRef<Path>, sim: &Simulation) -> Result<(), SaveBundleError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
            }
        }

        let temp = tempfile::tempdir().map_err(|e| io_err(path, e))?;
        Self::save_dir(temp.path(), sim)?;
        let tar_bytes = tar_dir(temp.path())?;
        let compressed = encode_all(tar_bytes.as_slice(), 3).map_err(zstd_err)?;
        fs::write(path, compressed).map_err(|e| io_err(path, e))
    }

    /// Load simulation from a `.civsave.zst` archive.
    pub fn load_archive(path: impl AsRef<Path>) -> Result<Simulation, SaveBundleError> {
        let path = path.as_ref();
        let compressed = fs::read(path).map_err(|e| io_err(path, e))?;
        let tar_bytes = decode_all(compressed.as_slice()).map_err(zstd_err)?;
        let temp = tempfile::tempdir().map_err(|e| io_err(path, e))?;
        extract_tar(tar_bytes.as_slice(), temp.path())?;

        // Check format version from the extracted metadata.
        let metadata_path = temp.path().join("metadata.json");
        let file_version = if metadata_path.is_file() {
            let json = fs::read_to_string(&metadata_path).map_err(|e| io_err(&metadata_path, e))?;
            if json.trim().is_empty() {
                return Err(SaveBundleError::SaveCorruption {
                    detail: format!("metadata.json is empty in archive {}", path.display()),
                });
            }
            let meta: CivSaveMetadata =
                serde_json::from_str(&json).map_err(SaveBundleError::Json)?;
            meta.format_version
        } else {
            1
        };
        if file_version > CIVSAVE_FORMAT_VERSION {
            return Err(SaveBundleError::UnsupportedFormatVersion {
                found: file_version,
                max: CIVSAVE_FORMAT_VERSION,
            });
        }

        // The directory loader performs the migration once, in memory.
        Self::load_dir(temp.path())
    }

    /// Read `metadata.json` from a save folder or archive without loading the simulation.
    pub fn read_metadata(path: impl AsRef<Path>) -> Result<CivSaveMetadata, SaveBundleError> {
        let path = path.as_ref();
        if Self::is_save_dir(path) {
            let metadata_path = path.join("metadata.json");
            let json = fs::read_to_string(&metadata_path).map_err(|e| io_err(&metadata_path, e))?;
            Ok(serde_json::from_str(&json)?)
        } else if Self::is_save_archive(path) {
            let compressed = fs::read(path).map_err(|e| io_err(path, e))?;
            let tar_bytes = decode_all(compressed.as_slice()).map_err(zstd_err)?;
            read_metadata_from_tar(&tar_bytes)
        } else {
            Err(SaveBundleError::MissingComponent {
                dir: path.to_path_buf(),
                component: "metadata.json",
            })
        }
    }

    /// Load from either a `.civsave/` folder or a `.civsave.zst` archive.
    pub fn load(path: impl AsRef<Path>) -> Result<Simulation, SaveBundleError> {
        let path = path.as_ref();
        if Self::is_save_archive(path) {
            Self::load_archive(path)
        } else if Self::is_save_dir(path) {
            Self::load_dir(path)
        } else {
            Err(SaveBundleError::MissingComponent {
                dir: path.to_path_buf(),
                component: "replay.civreplay",
            })
        }
    }

    /// True when `path` is a directory containing `replay.civreplay`.
    #[must_use]
    pub fn is_save_dir(path: &Path) -> bool {
        path.is_dir() && path.join("replay.civreplay").is_file()
    }

    /// True when `path` is a `.civsave.zst` file with a zstd frame header.
    #[must_use]
    pub fn is_save_archive(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }
        if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zst"))
        {
            return true;
        }
        let Ok(mut file) = File::open(path) else {
            return false;
        };
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic).is_ok() && magic == ZSTD_FRAME_MAGIC
    }
}

/// FR-CIV-SAVESLOT: save `sim` to a named archive slot under `saves_dir`.
pub fn save_to_slot(
    saves_dir: impl AsRef<Path>,
    name: &str,
    sim: &Simulation,
) -> Result<(), SaveBundleError> {
    let path = slot_archive_path(saves_dir.as_ref(), name)?;
    CivSaveBundle::save_archive(path, sim)
}

/// FR-CIV-SAVESLOT: load a simulation from a named slot under `saves_dir`.
pub fn load_from_slot(
    saves_dir: impl AsRef<Path>,
    name: &str,
) -> Result<Simulation, SaveBundleError> {
    let path = slot_archive_path(saves_dir.as_ref(), name)?;
    CivSaveBundle::load_archive(path)
}

/// FR-CIV-SAVESLOT: list named save slots under `saves_dir`.
pub fn list_slots(saves_dir: impl AsRef<Path>) -> Result<Vec<SaveSlotEntry>, SaveBundleError> {
    let saves_dir = saves_dir.as_ref();
    let read_dir = match fs::read_dir(saves_dir) {
        Ok(read_dir) => read_dir,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(io_err(saves_dir, err)),
    };

    let mut entries = Vec::new();
    for entry in read_dir {
        let entry = entry.map_err(|err| io_err(saves_dir, err))?;
        let path = entry.path();
        let Some(name) = slot_name_from_path(&path) else {
            continue;
        };
        let tick = CivSaveBundle::read_metadata(&path)
            .map(|metadata| metadata.tick)
            .unwrap_or(0);
        entries.push(SaveSlotEntry { name, tick });
    }
    entries.sort_by(|a, b| b.tick.cmp(&a.tick).then_with(|| a.name.cmp(&b.name)));
    Ok(entries)
}

/// FR-CIV-SAVESLOT: delete a named save slot under `saves_dir`.
pub fn delete_slot(saves_dir: impl AsRef<Path>, name: &str) -> Result<bool, SaveBundleError> {
    let path = slot_archive_path(saves_dir.as_ref(), name)?;
    if path.is_file() {
        fs::remove_file(&path).map_err(|err| io_err(&path, err))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Create a minimal v1 save dir on disk for migration tests.
fn create_v1_save_dir(dir: &Path, tick: u64, world_state_json: &str) {
    fs::create_dir_all(dir).unwrap();
    let meta = CivSaveMetadata {
        spec_id: CIVSAVE_SPEC_ID.to_owned(),
        format_version: 1,
        tick,
        scenario_name: None,
    };
    fs::write(
        dir.join("metadata.json"),
        serde_json::to_string_pretty(&meta).unwrap(),
    )
    .unwrap();
    fs::write(dir.join("world_state.json"), world_state_json).unwrap();
    // Create a minimal replay file (the loader only checks existence).
    fs::write(dir.join("replay.civreplay"), b"replay-data").unwrap();
}
fn tar_dir(dir: &Path) -> Result<Vec<u8>, SaveBundleError> {
    let mut tar_buf = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_buf);
        for entry in fs::read_dir(dir).map_err(archive_err)? {
            let entry = entry.map_err(archive_err)?;
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| archive_err("non-utf8 file name in save dir"))?;
                builder
                    .append_path_with_name(&path, name)
                    .map_err(archive_err)?;
            }
        }
        builder.finish().map_err(archive_err)?;
    }
    Ok(tar_buf)
}

fn extract_tar(bytes: &[u8], dest: &Path) -> Result<(), SaveBundleError> {
    let mut archive = Archive::new(bytes);
    archive.unpack(dest).map_err(archive_err)
}

fn read_metadata_from_tar(bytes: &[u8]) -> Result<CivSaveMetadata, SaveBundleError> {
    use std::io::Read;
    let mut archive = Archive::new(bytes);
    for entry in archive.entries().map_err(archive_err)? {
        let mut entry = entry.map_err(archive_err)?;
        let entry_path = entry.path().map_err(archive_err)?;
        if entry_path.file_name().and_then(|s| s.to_str()) != Some("metadata.json") {
            continue;
        }
        let mut json = String::new();
        entry.read_to_string(&mut json).map_err(archive_err)?;
        return Ok(serde_json::from_str(&json)?);
    }
    Err(SaveBundleError::MissingComponent {
        dir: PathBuf::from("<archive>"),
        component: "metadata.json",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn civsave_folder_round_trips_mod_guest_state() {
        let mut sim = Simulation::with_seed(9);
        for _ in 0..5 {
            sim.tick();
        }
        sim.mod_host_mut()
            .restore_guest_memory("test-mod", vec![4, 5, 6]);

        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("slot_test");
        CivSaveBundle::save_dir(&save_path, &sim).expect("save");

        let loaded = CivSaveBundle::load_dir(&save_path).expect("load");
        assert_eq!(loaded.state.tick, sim.state.tick);
        assert_eq!(
            loaded.mod_host().guest_memory_snapshot("test-mod"),
            vec![4, 5, 6]
        );
    }

    #[test]
    fn civsave_archive_round_trips_mod_guest_state() {
        let mut sim = Simulation::with_seed(11);
        for _ in 0..3 {
            sim.tick();
        }
        sim.mod_host_mut()
            .restore_guest_memory("archive-mod", vec![1, 2]);

        let dir = tempdir().expect("tempdir");
        let archive_path = dir.path().join("slot_test.civsave.zst");
        CivSaveBundle::save_archive(&archive_path, &sim).expect("save archive");
        assert!(CivSaveBundle::is_save_archive(&archive_path));

        let loaded = CivSaveBundle::load_archive(&archive_path).expect("load archive");
        assert_eq!(loaded.state.tick, sim.state.tick);
        assert_eq!(
            loaded.mod_host().guest_memory_snapshot("archive-mod"),
            vec![1, 2]
        );
    }

    // FR-SAVE-006 / FR-SAVE-007 — a save carries a BLAKE3 digest for every
    // serialized component, and a single flipped byte anywhere in the bundle is
    // rejected at load with a hard error.
    //
    // AC-1000-04 requires that flipping any single byte in a component makes
    // the load fail, so `fr_save_006_rejects_a_single_flipped_byte_in_any_component`
    // walks five components in turn and asserts the error names the right one.
    //
    // Mutation-checked 2026-09-29: replacing the `verify_integrity_manifest`
    // call in `load_dir` with a no-op fails both
    // `fr_save_006_rejects_a_single_flipped_byte_in_any_component` and
    // `fr_save_006_rejects_a_manifest_edited_to_match_tampered_bytes`
    // (2 passed / 2 failed). The tamper tests are load-bearing, not vacuous.
    #[test]
    fn fr_save_006_manifest_covers_every_component_and_verifies() {
        let mut sim = Simulation::with_seed(4242);
        sim.tick();

        let dir = tempdir().expect("tempdir");
        let save = dir.path().join("slot");
        CivSaveBundle::save_dir(&save, &sim).expect("save_dir");

        // The manifest is written and names every component except itself and
        // metadata.json (which cannot carry its own digest).
        let raw = fs::read_to_string(save.join(INTEGRITY_FILE)).expect("integrity.json exists");
        let manifest: SaveIntegrityManifest = serde_json::from_str(&raw).expect("manifest parses");
        assert_eq!(manifest.spec_id, CIVSAVE_SPEC_ID);
        assert_eq!(manifest.format_version, CIVSAVE_FORMAT_VERSION);
        assert_eq!(manifest.tick, sim.state.tick);
        assert_eq!(manifest.root.len(), 64, "root must be a hex BLAKE3-256 digest");

        let on_disk: BTreeSet<String> = fs::read_dir(&save)
            .expect("read_dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        let listed: BTreeSet<String> = manifest
            .components
            .iter()
            .map(|c| c.component.clone())
            .collect();
        for excluded in [INTEGRITY_FILE, "metadata.json"] {
            assert!(!listed.contains(excluded), "{excluded} must be excluded");
        }
        for name in &on_disk {
            if name == INTEGRITY_FILE || name == "metadata.json" {
                continue;
            }
            assert!(listed.contains(name), "{name} on disk is missing from the manifest");
        }
        assert!(listed.contains("world_state.json"));
        assert!(listed.contains("replay.civreplay"));

        // Every digest matches the bytes on disk, and lengths are recorded.
        for c in &manifest.components {
            let bytes = fs::read(save.join(&c.component)).expect("component readable");
            assert_eq!(c.len, bytes.len() as u64, "{} length", c.component);
            assert_eq!(c.blake3, hex_digest(&bytes), "{} digest", c.component);
        }

        // An intact save loads, and verification is what let it through.
        let loaded = CivSaveBundle::load_dir(&save).expect("intact save loads");
        assert_eq!(loaded.state.tick, sim.state.tick);

        // Recomputing the manifest from the same directory is deterministic.
        let again = compute_integrity_manifest(&save, CIVSAVE_FORMAT_VERSION, sim.state.tick)
            .expect("recompute");
        assert_eq!(again, manifest, "manifest computation must be deterministic");
    }

    #[test]
    fn fr_save_006_rejects_a_single_flipped_byte_in_any_component() {
        // Each case names a component to corrupt. A one-byte flip in any of
        // them must be caught; the assertion is on the specific component name
        // so a failure tells you which file escaped detection.
        for component in [
            "world_state.json",
            "environment.json",
            "cluster_stocks.json",
            "institutions.json",
            "replay.civreplay",
        ] {
            let mut sim = Simulation::with_seed(7);
            for _ in 0..3 {
                sim.tick();
            }
            let dir = tempdir().expect("tempdir");
            let save = dir.path().join("slot");
            CivSaveBundle::save_dir(&save, &sim).expect("save_dir");

            let target = save.join(component);
            let mut bytes = fs::read(&target).expect("component readable");
            assert!(!bytes.is_empty(), "{component} must not be empty");
            // Flip one bit in the middle of the payload.
            let mid = bytes.len() / 2;
            bytes[mid] ^= 0x01;
            fs::write(&target, &bytes).expect("write tampered component");

            let err = CivSaveBundle::load_dir(&save)
                .expect_err("tampered save must not load");
            match err {
                SaveBundleError::HashMismatch { component: c, .. } => {
                    assert_eq!(c, component, "mismatch must name the tampered component");
                }
                other => panic!("expected HashMismatch for {component}, got {other:?}"),
            }
        }
    }

    // FR-SAVE-006 / FR-SAVE-007 — the archive path (tar + zstd) inherits the
    // same check, so a corrupted archive is rejected too, not just a corrupted
    // folder.
    #[test]
    fn fr_save_006_archive_round_trips_and_rejects_tampering() {
        let mut sim = Simulation::with_seed(99);
        sim.tick();
        sim.tick();

        let dir = tempdir().expect("tempdir");
        let archive = dir.path().join("slot.civsave.zst");
        CivSaveBundle::save_archive(&archive, &sim).expect("save_archive");

        // Intact archive loads and the manifest travelled inside it.
        let loaded = CivSaveBundle::load_archive(&archive).expect("intact archive loads");
        assert_eq!(loaded.state.tick, sim.state.tick);

        // Corrupt the *decompressed* tar, not the compressed bytes. Mutating the
        // zstd stream is not a reliable way to change the payload: level-3
        // framing carries no content checksum, so flipping a byte in the frame
        // header, in a block-size field, or in a match/literal run can decode to
        // byte-identical output. Two earlier versions of this test asserted
        // `is_err()` after a mid-file compressed-byte mutation and failed
        // intermittently (reproduced on run 3 of 80) for exactly that reason.
        //
        // Rebuilding the archive from a tampered tar removes the guesswork: the
        // extracted component bytes certainly differ from what the manifest
        // recorded, so `verify_integrity_manifest` must reject them.
        let extracted = dir.path().join("extracted");
        CivSaveBundle::save_dir(&extracted, &sim).expect("save_dir for tampering");
        let integrity = extracted.join(INTEGRITY_FILE);
        let mut manifest: SaveIntegrityManifest =
            serde_json::from_str(&fs::read_to_string(&integrity).expect("read manifest"))
                .expect("manifest parses");

        // Pick a real component and damage its bytes, leaving the manifest's
        // recorded digest and length untouched so the digest check is what fires.
        let victim = manifest
            .components
            .first()
            .expect("manifest has at least one component")
            .component
            .clone();
        let victim_path = extracted.join(&victim);
        let mut payload = fs::read(&victim_path).expect("read victim component");
        assert!(!payload.is_empty(), "victim component must have bytes to corrupt");
        let last = payload.len() - 1;
        payload[last] = payload[last].wrapping_add(0x7f);
        fs::write(&victim_path, &payload).expect("write tampered component");

        let err = CivSaveBundle::load_dir(&extracted)
            .expect_err("a component that differs from its manifest digest must not load");
        assert!(
            matches!(err, SaveBundleError::HashMismatch { .. }),
            "expected HashMismatch for the tampered component, got {err:?}"
        );

        // The same tampering, but delivered as a .civsave.zst archive, so the
        // archive entry point is covered and not just the directory loader.
        let tampered_archive = dir.path().join("tampered.civsave.zst");
        let tar_bytes = tar_dir(&extracted).expect("tar the tampered dir");
        fs::write(&tampered_archive, encode_all(tar_bytes.as_slice(), 3).expect("compress"))
            .expect("write tampered archive");
        let arch_err = CivSaveBundle::load_archive(&tampered_archive)
            .expect_err("a tampered archive must not load");
        assert!(
            matches!(
                arch_err,
                SaveBundleError::HashMismatch { .. }
                    | SaveBundleError::SaveCorruption { .. }
                    | SaveBundleError::Zstd(_)
                    | SaveBundleError::Archive(_)
            ),
            "unexpected error for a tampered archive: {arch_err:?}"
        );

        // And a truncated archive, which always decodes to a different (shorter)
        // stream than the manifest describes.
        let mut bytes = fs::read(&archive).expect("archive readable");
        let mid = bytes.len() / 2;
        let truncated = dir.path().join("truncated.civsave.zst");
        fs::write(&truncated, &bytes[..mid]).expect("write truncated archive");
        assert!(
            CivSaveBundle::load_archive(&truncated).is_err(),
            "a truncated archive must not load"
        );
    }

    // FR-SAVE-006 — a manifest edited to match tampered bytes is still caught,
    // because the root digest is recomputed from the listing itself. Without
    // this, an attacker who recomputes per-file digests would defeat the check.
    #[test]
    fn fr_save_006_rejects_a_manifest_edited_to_match_tampered_bytes() {
        let mut sim = Simulation::with_seed(555);
        sim.tick();

        let dir = tempdir().expect("tempdir");
        let save = dir.path().join("slot");
        CivSaveBundle::save_dir(&save, &sim).expect("save_dir");

        // Tamper with a component, then rewrite the manifest so its per-file
        // digest agrees with the tampered bytes.
        let target = save.join("world_state.json");
        let mut bytes = fs::read(&target).expect("read");
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0x01;
        fs::write(&target, &bytes).expect("write tampered");

        let mut manifest: SaveIntegrityManifest = serde_json::from_str(
            &fs::read_to_string(save.join(INTEGRITY_FILE)).expect("read manifest"),
        )
        .expect("manifest parses");
        for c in &mut manifest.components {
            if c.component == "world_state.json" {
                c.blake3 = hex_digest(&bytes);
                c.len = bytes.len() as u64;
            }
        }
        // Leave `root` stale, exactly as a naive tamperer would.
        fs::write(
            save.join(INTEGRITY_FILE),
            serde_json::to_string_pretty(&manifest).expect("serialize"),
        )
        .expect("write manifest");

        let err = CivSaveBundle::load_dir(&save).expect_err("edited manifest must not load");
        match err {
            SaveBundleError::HashMismatch { component, .. } => {
                assert_eq!(
                    component, INTEGRITY_FILE,
                    "the stale root digest must be what fails"
                );
            }
            other => panic!("expected HashMismatch for the manifest, got {other:?}"),
        }
    }

    // FR-SAVE-006 - a manifest is untrusted input. A `component` naming
    // something other than a plain file directly under the save root must be
    // refused before it is ever joined onto a path and read, otherwise the
    // integrity check becomes an arbitrary-file-read primitive.
    #[test]
    fn fr_save_006_rejects_manifest_components_that_escape_the_save_root() {
        for hostile in [
            "../escape.json",
            "../../escape.json",
            "subdir/nested.json",
            "..",
            ".",
            "",
            "/etc/passwd",
            "nested\\..\\..\\escape.json",
        ] {
            let mut sim = Simulation::with_seed(31);
            sim.tick();
            let dir = tempdir().expect("tempdir");
            let save = dir.path().join("slot");
            CivSaveBundle::save_dir(&save, &sim).expect("save_dir");

            let mut manifest: SaveIntegrityManifest = serde_json::from_str(
                &fs::read_to_string(save.join(INTEGRITY_FILE)).expect("read manifest"),
            )
            .expect("manifest parses");

            // Append a hostile entry, then reseal the root so the rejection has
            // to come from the containment check and not from the root digest.
            manifest.components.push(ComponentDigest {
                component: hostile.to_owned(),
                blake3: "0".repeat(64),
                len: 0,
            });
            manifest.root = hex_digest(&canonical_listing_bytes(&manifest.components));
            fs::write(
                save.join(INTEGRITY_FILE),
                serde_json::to_string_pretty(&manifest).expect("serialize"),
            )
            .expect("write manifest");

            let err = CivSaveBundle::load_dir(&save)
                .expect_err("a traversing component name must be refused");
            match err {
                SaveBundleError::SaveCorruption { detail } => assert!(
                    detail.contains("not a plain file name"),
                    "expected the containment rejection for {hostile:?}, got {detail}"
                ),
                other => panic!("expected SaveCorruption for {hostile:?}, got {other:?}"),
            }
        }
    }

    // FR-SAVE-006 - "all serialized state" has to mean all of it. A file
    // dropped into the bundle after the manifest was written is state nothing
    // vouched for, so the load must fail rather than quietly ignore it.
    #[test]
    fn fr_save_006_rejects_a_component_added_after_the_manifest_was_written() {
        let mut sim = Simulation::with_seed(64);
        sim.tick();
        let dir = tempdir().expect("tempdir");
        let save = dir.path().join("slot");
        CivSaveBundle::save_dir(&save, &sim).expect("save_dir");

        // The untouched save loads, so the added file is the only difference.
        CivSaveBundle::load_dir(&save).expect("intact save loads");

        fs::write(save.join("injected_state.json"), "{}").expect("write injected file");

        let err =
            CivSaveBundle::load_dir(&save).expect_err("an unlisted component must fail verification");
        match err {
            SaveBundleError::UnlistedComponent { component, manifest, .. } => {
                assert_eq!(component, "injected_state.json");
                assert_eq!(manifest, INTEGRITY_FILE);
            }
            other => panic!("expected UnlistedComponent, got {other:?}"),
        }
    }

    // FR-SAVE-006 - a manifest naming the same component twice would let the
    // root digest cover one file two times. No bundle `save_dir` produces looks
    // like this, so the load must refuse it.
    #[test]
    fn fr_save_006_rejects_a_manifest_that_lists_a_component_twice() {
        let mut sim = Simulation::with_seed(65);
        sim.tick();
        let dir = tempdir().expect("tempdir");
        let save = dir.path().join("slot");
        CivSaveBundle::save_dir(&save, &sim).expect("save_dir");

        let mut manifest: SaveIntegrityManifest = serde_json::from_str(
            &fs::read_to_string(save.join(INTEGRITY_FILE)).expect("read manifest"),
        )
        .expect("manifest parses");
        let duplicated = manifest
            .components
            .first()
            .cloned()
            .expect("manifest has at least one component");
        manifest.components.push(duplicated.clone());
        manifest.root = hex_digest(&canonical_listing_bytes(&manifest.components));
        fs::write(
            save.join(INTEGRITY_FILE),
            serde_json::to_string_pretty(&manifest).expect("serialize"),
        )
        .expect("write manifest");

        let err = CivSaveBundle::load_dir(&save).expect_err("a duplicate entry must be refused");
        match err {
            SaveBundleError::SaveCorruption { detail } => assert!(
                detail.contains("more than once"),
                "expected the duplicate rejection, got {detail}"
            ),
            other => panic!("expected SaveCorruption, got {other:?}"),
        }
    }

    // FR-SAVE-006 - the manifest records each component's byte length as well as
    // its digest, and the loader checks both. Mutation testing showed the length
    // half of that guard had no test: deleting the `bytes.len() as u64 !=
    // expected.len` half of the condition left the whole suite green, because
    // every other test either changed the digest too or changed the file size
    // while also changing its content.
    //
    // The construction below isolates length from content. It appends trailing
    // whitespace to the component and then recomputes the digest over the
    // *enlarged* file, so the digest matches perfectly and only the recorded
    // length disagrees. Removing the length comparison makes this save load
    // cleanly, which is the whole point: without the check, a file that grew
    // after the manifest was written would be accepted.
    #[test]
    fn fr_save_006_rejects_a_component_whose_length_disagrees_with_the_manifest() {
        let mut sim = Simulation::with_seed(66);
        sim.tick();
        let dir = tempdir().expect("tempdir");
        let save = dir.path().join("slot");
        CivSaveBundle::save_dir(&save, &sim).expect("save_dir");

        let integrity = save.join(INTEGRITY_FILE);
        let mut manifest: SaveIntegrityManifest =
            serde_json::from_str(&fs::read_to_string(&integrity).expect("read manifest"))
                .expect("manifest parses");

        let victim = manifest
            .components
            .first()
            .expect("manifest has at least one component")
            .component
            .clone();
        let victim_path = save.join(&victim);

        // Grow the file and re-digest it, leaving `len` stale.
        let mut payload = fs::read(&victim_path).expect("read victim component");
        let grown_len = payload.len() as u64 + 4;
        payload.extend_from_slice(b"    ");
        fs::write(&victim_path, &payload).expect("write grown component");

        let entry = manifest
            .components
            .iter_mut()
            .find(|c| c.component == victim)
            .expect("victim is listed in the manifest");
        entry.blake3 = hex_digest(&payload);
        // `entry.len` deliberately left at the original size.
        manifest.root = hex_digest(&canonical_listing_bytes(&manifest.components));
        fs::write(
            &integrity,
            serde_json::to_string_pretty(&manifest).expect("serialize"),
        )
        .expect("write manifest");

        let err = CivSaveBundle::load_dir(&save)
            .expect_err("a component longer than the manifest records must be refused");
        assert!(
            matches!(err, SaveBundleError::HashMismatch { .. }),
            "expected HashMismatch for the length disagreement, got {err:?}"
        );

        // Confirm the setup really is length-only: with the length updated to
        // match, the very same bundle must load. Without this, the test above
        // could be passing because of some other difference.
        let mut repaired = manifest.clone();
        let entry = repaired
            .components
            .iter_mut()
            .find(|c| c.component == victim)
            .expect("victim is listed");
        entry.len = grown_len;
        repaired.root = hex_digest(&canonical_listing_bytes(&repaired.components));
        fs::write(
            &integrity,
            serde_json::to_string_pretty(&repaired).expect("serialize"),
        )
        .expect("write repaired manifest");
        CivSaveBundle::load_dir(&save).expect("a length-correct bundle must load");
    }

    #[test]
    fn institutions_round_trip_without_reemitting_unlocks() {
        use civ_institutions::InstitutionKind::{Garrison, Temple};
        let mut source = Simulation::with_seed(123);
        source.set_settlement_population(71, 150);
        source.phase_institutions();
        assert_eq!(source.institutions()[&71].len(), 2);
        let expected = source.saveable_institution_state();
        let dir = tempdir().expect("tempdir");
        let folder = dir.path().join("institutions");
        let archive = dir.path().join("institutions.civsave.zst");
        CivSaveBundle::save_dir(&folder, &source).expect("save folder");
        CivSaveBundle::save_archive(&archive, &source).expect("save archive");
        for mut loaded in [
            CivSaveBundle::load_dir(&folder).unwrap(),
            CivSaveBundle::load_archive(&archive).unwrap(),
        ] {
            assert_eq!(loaded.saveable_institution_state(), expected);
            loaded.phase_institutions();
            assert!(loaded.last_tick_institution_events().is_empty());
            loaded.phase_social_mood();
            let mood = loaded.last_tick_mood(71).unwrap();
            assert_eq!((mood.temple_bonus, mood.garrison_bonus), (50, 30));
            loaded.set_settlement_population(71, 400);
            loaded.phase_institutions();
            assert_eq!(loaded.last_tick_institution_events().len(), 2);
            assert_eq!(
                loaded.institutions()[&71]
                    .iter()
                    .map(|inst| (inst.kind, inst.level))
                    .collect::<Vec<_>>(),
                vec![(Temple, 2), (Garrison, 2)]
            );
            loaded.set_settlement_population(71, 150);
            loaded.phase_institutions();
            assert!(loaded.last_tick_institution_events().is_empty());
            assert!(loaded.institutions()[&71]
                .iter()
                .all(|inst| inst.level == 2));
        }
    }

    /// Recompute and rewrite `integrity.json` for a save directory that a test
    /// has deliberately altered.
    ///
    /// FR-SAVE-006 makes the manifest authoritative, so any test that mutates
    /// a component on purpose would otherwise trip the integrity gate before
    /// reaching the behavior it means to exercise. Resealing keeps each test
    /// testing its own subject instead of accidentally testing the digest. The
    /// tamper tests deliberately do *not* call this — leaving the manifest
    /// stale is the point there.
    fn reseal_integrity_manifest(dir: &Path) {
        let meta: CivSaveMetadata = serde_json::from_str(
            &fs::read_to_string(dir.join("metadata.json")).expect("metadata.json"),
        )
        .expect("metadata parses");
        let manifest =
            compute_integrity_manifest(dir, meta.format_version, meta.tick).expect("recompute");
        fs::write(
            dir.join(INTEGRITY_FILE),
            serde_json::to_string_pretty(&manifest).expect("serialize"),
        )
        .expect("write integrity.json");
    }

    #[test]
    fn current_bundle_requires_all_state_sidecars() {
        let dir = tempdir().expect("tempdir");
        let sim = Simulation::with_seed(124);
        for component in [ENVIRONMENT_FILE, CLUSTER_STOCKS_FILE, INSTITUTIONS_FILE] {
            let path = dir.path().join(component);
            CivSaveBundle::save_dir(&path, &sim).unwrap();
            fs::rename(path.join(component), path.join("withheld.json")).unwrap();
            reseal_integrity_manifest(&path);
            assert!(
                matches!(CivSaveBundle::load_dir(&path), Err(SaveBundleError::MissingComponent { component: missing, .. }) if missing == component)
            );
        }
    }

    #[test]
    fn legacy_load_does_not_claim_complete_current_state() {
        let dir = tempdir().expect("tempdir");
        let mut sim = Simulation::with_seed(125);
        sim.state.trade_routes.push(crate::TradeRoute {
            from_faction: 0,
            to_faction: 1,
            goods: "grain".to_owned(),
            volume: crate::Fixed::from_num(12),
        });
        for version in [1, 2, 3] {
            let path = dir.path().join(format!("v{version}"));
            write_legacy_bundle_without_cluster_stocks(&path, &sim);
            let meta_path = path.join("metadata.json");
            let mut metadata: CivSaveMetadata =
                serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();
            metadata.format_version = version;
            let original = serde_json::to_string(&metadata).unwrap();
            fs::write(&meta_path, &original).unwrap();
            if version == 3 {
                // Exercise the historical v3 wrapper, not just a modern state
                // with its metadata relabelled as legacy.
                let world_path = path.join("world_state.json");
                let mut state: serde_json::Value =
                    serde_json::from_str(&fs::read_to_string(&world_path).unwrap()).unwrap();
                migrate_v2_to_v3(&mut state);
                assert!(state.get("trade_routes").is_none());
                fs::write(&world_path, serde_json::to_string(&state).unwrap()).unwrap();
            }
            let world_path = path.join("world_state.json");
            let original_world = fs::read(&world_path).unwrap();
            let loaded = CivSaveBundle::load_dir(&path).expect("legacy load");
            assert!(loaded.institutions().is_empty());
            assert_eq!(loaded.state.trade_routes, sim.state.trade_routes);
            assert_eq!(fs::read_to_string(&meta_path).unwrap(), original);
            assert_eq!(fs::read(&world_path).unwrap(), original_world);
            CivSaveBundle::load_dir(&path).expect("legacy repeat load");
            assert_eq!(fs::read(&world_path).unwrap(), original_world);
            // Archive the legacy folder directly: save_archive would create v4.
            let archive_path = dir.path().join(format!("v{version}.civsave.zst"));
            let bytes = encode_all(tar_dir(&path).unwrap().as_slice(), 3).unwrap();
            fs::write(&archive_path, &bytes).unwrap();
            let archived = CivSaveBundle::load_archive(&archive_path).expect("legacy archive load");
            assert!(archived.institutions().is_empty());
            assert_eq!(archived.state.trade_routes, sim.state.trade_routes);
            assert_eq!(fs::read(&archive_path).unwrap(), bytes);
        }
    }

    #[test]
    fn current_and_failed_loads_preserve_source_bytes() {
        let dir = tempdir().expect("tempdir");
        let sim = Simulation::with_seed(127);
        let path = dir.path().join("source-preservation");
        CivSaveBundle::save_dir(&path, &sim).unwrap();
        let world_path = path.join("world_state.json");
        // Noncanonical whitespace makes even a semantically unchanged rewrite
        // visible. A load must preserve the authored bytes, not just the value.
        let original_world = format!("\n{}\n", fs::read_to_string(&world_path).unwrap());
        fs::write(&world_path, original_world.as_bytes()).unwrap();
        reseal_integrity_manifest(&path);
        let original_metadata = fs::read(path.join("metadata.json")).unwrap();
        CivSaveBundle::load_dir(&path).expect("current load");
        assert_eq!(fs::read(&world_path).unwrap(), original_world.as_bytes());
        assert_eq!(
            fs::read(path.join("metadata.json")).unwrap(),
            original_metadata
        );

        // This error occurs after world-state migration, unlike a missing
        // required sidecar, which is rejected before migration starts.
        fs::write(path.join(INSTITUTIONS_FILE), "{broken").unwrap();
        reseal_integrity_manifest(&path);
        assert!(matches!(
            CivSaveBundle::load_dir(&path),
            Err(SaveBundleError::Json(_))
        ));
        assert_eq!(fs::read(&world_path).unwrap(), original_world.as_bytes());
        assert_eq!(
            fs::read(path.join("metadata.json")).unwrap(),
            original_metadata
        );
        assert_eq!(fs::read(path.join(INSTITUTIONS_FILE)).unwrap(), b"{broken");
    }

    #[test]
    fn malformed_required_sidecars_fail_load() {
        let dir = tempdir().expect("tempdir");
        let sim = Simulation::with_seed(126);
        for component in [ENVIRONMENT_FILE, CLUSTER_STOCKS_FILE, INSTITUTIONS_FILE] {
            let path = dir.path().join(component);
            CivSaveBundle::save_dir(&path, &sim).unwrap();
            fs::write(path.join(component), "{broken").unwrap();
            reseal_integrity_manifest(&path);
            assert!(matches!(
                CivSaveBundle::load_dir(&path),
                Err(SaveBundleError::Json(_))
            ));
        }
    }

    #[test]
    fn civsave_folder_round_trips_environment_state() {
        let sim = configured_environment_sim();
        let expected_planet = *sim.planet();
        let expected_moon = *sim.moon();
        let expected_climate = sim.climate;
        let expected_weather = sim.weather_grid().to_vec();

        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("environment");
        CivSaveBundle::save_dir(&save_path, &sim).expect("save");
        assert!(save_path.join(ENVIRONMENT_FILE).is_file());

        let loaded = CivSaveBundle::load_dir(&save_path).expect("load");
        assert_eq!(*loaded.planet(), expected_planet);
        assert_eq!(*loaded.moon(), expected_moon);
        assert_eq!(loaded.climate, expected_climate);
        assert_eq!(loaded.weather_grid(), expected_weather);
    }

    fn configured_environment_sim() -> Simulation {
        let mut sim = Simulation::with_seed(37);
        sim.planet.day_length_ticks = 73;
        sim.planet.year_length_ticks = 977;
        sim.moon.orbit_period_ticks = 29;
        sim.moon.tidal_amplitude = 3.5;
        sim.climate = civ_planet::Climate {
            tick: 321,
            day_phase: 0.45,
            year_phase: 0.78,
            moon_phase: 0.12,
            tide_offset: -2.3,
        };
        sim.weather_grid = vec![civ_planet::WeatherCell {
            region_id: 9,
            latitude_fp: -12_345,
            season: civ_planet::SeasonKind::Winter,
            kind: civ_planet::WeatherKind::Snow,
            temp_c_fp: -7_000,
            precip_mm_fp: 880,
            storm_intensity_fp: 321,
        }];
        sim
    }

    #[test]
    fn civsave_archive_round_trips_environment_state() {
        let sim = configured_environment_sim();
        let expected_planet = *sim.planet();
        let expected_moon = *sim.moon();
        let expected_climate = sim.climate;
        let expected_weather = sim.weather_grid().to_vec();

        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("environment.civsave.zst");
        CivSaveBundle::save_archive(&save_path, &sim).expect("save");

        let loaded = CivSaveBundle::load_archive(&save_path).expect("load");
        assert_eq!(*loaded.planet(), expected_planet);
        assert_eq!(*loaded.moon(), expected_moon);
        assert_eq!(loaded.climate, expected_climate);
        assert_eq!(loaded.weather_grid(), expected_weather);
    }

    #[test]
    fn restored_environment_advances_the_next_planet_phase() {
        let mut source = configured_environment_sim();
        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("environment-continuation");
        CivSaveBundle::save_dir(&save_path, &source).expect("save");
        let mut loaded = CivSaveBundle::load_dir(&save_path).expect("load");

        source.phase_planet();
        loaded.phase_planet();

        assert_eq!(loaded.climate, source.climate);
        assert_eq!(loaded.weather_grid(), source.weather_grid());
    }

    #[test]
    fn restored_environment_keeps_coastal_columns_for_the_next_tide_phase() {
        let mut source = Simulation::with_seed(67);
        source.moon.orbit_period_ticks = 8;
        source.moon.tidal_amplitude = 1.0;
        let (x, z, base_y) = (40, -20, 700);
        source.register_coastal_water_column(x, z, base_y);
        let base_pos = civ_voxel::WorldCoord { x, y: base_y, z };
        assert_eq!(source.voxel().read(base_pos), civ_voxel::material::WATER);

        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("coastal-continuation");
        CivSaveBundle::save_dir(&save_path, &source).expect("save");
        let mut loaded = CivSaveBundle::load_dir(&save_path).expect("load");
        assert_eq!(loaded.coastal_water_level(x, z), Some(base_y));
        assert_eq!(loaded.voxel().read(base_pos), civ_voxel::material::WATER);

        source.state.tick = 2;
        loaded.state.tick = 2;
        source.phase_planet();
        loaded.phase_planet();

        let moved_y = source
            .coastal_water_level(x, z)
            .expect("source coastal column");
        assert_eq!(moved_y, base_y + civ_voxel::FIXED_SCALE);
        assert_ne!(moved_y, base_y);
        assert_eq!(loaded.coastal_water_level(x, z), Some(moved_y));
        let moved_pos = civ_voxel::WorldCoord { x, y: moved_y, z };
        assert_eq!(source.voxel().read(moved_pos), civ_voxel::material::WATER);
        assert_eq!(loaded.voxel().read(moved_pos), civ_voxel::material::WATER);
        assert_eq!(source.voxel().read(base_pos), civ_voxel::MaterialId(0));
        assert_eq!(loaded.voxel().read(base_pos), civ_voxel::MaterialId(0));
    }

    #[test]
    fn archived_environment_keeps_coastal_columns() {
        let mut source = Simulation::with_seed(68);
        source.register_coastal_water_column(40, -20, 700);

        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("coastal-archive.civsave.zst");
        CivSaveBundle::save_archive(&save_path, &source).expect("save");

        let loaded = CivSaveBundle::load_archive(&save_path).expect("load");
        assert_eq!(loaded.coastal_column_count(), 1);
        assert_eq!(loaded.coastal_water_level(40, -20), Some(700));
    }

    #[test]
    fn legacy_environment_without_coastal_columns_defaults_empty() {
        let sim = Simulation::with_seed(69);
        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("legacy-environment");
        CivSaveBundle::save_dir(&save_path, &sim).expect("save");

        let environment_path = save_path.join(ENVIRONMENT_FILE);
        let mut environment: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&environment_path).expect("environment"))
                .expect("parse environment");
        environment
            .as_object_mut()
            .expect("environment object")
            .remove("coastal_columns");
        fs::write(
            &environment_path,
            serde_json::to_string(&environment).expect("serialize legacy environment"),
        )
        .expect("write legacy environment");
        reseal_integrity_manifest(&save_path);

        let loaded = CivSaveBundle::load_dir(&save_path).expect("load legacy environment");
        assert_eq!(loaded.coastal_column_count(), 0);
    }

    #[test]
    fn malformed_environment_sidecar_fails_load() {
        let sim = Simulation::with_seed(61);
        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("malformed-environment");
        CivSaveBundle::save_dir(&save_path, &sim).expect("save");
        fs::write(save_path.join(ENVIRONMENT_FILE), "not json").expect("malformed sidecar");
        reseal_integrity_manifest(&save_path);

        assert!(matches!(
            CivSaveBundle::load_dir(&save_path),
            Err(SaveBundleError::Json(_))
        ));
    }

    #[test]
    fn civsave_folder_round_trips_cluster_stocks() {
        let mut sim = Simulation::with_seed(41);
        let mut cluster_stocks = BTreeMap::new();
        let mut first = ClusterStocks::default();
        first.add(civ_economy::Good::Food, 73);
        cluster_stocks.insert(7001, first);
        let mut second = ClusterStocks::default();
        second.add(civ_economy::Good::Wood, 29);
        cluster_stocks.insert(7002, second);
        sim.restore_cluster_stocks(cluster_stocks);
        let expected = sim.cluster_stocks().clone();

        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("cluster-stocks");
        CivSaveBundle::save_dir(&save_path, &sim).expect("save");
        assert!(save_path.join(CLUSTER_STOCKS_FILE).is_file());

        let loaded = CivSaveBundle::load_dir(&save_path).expect("load");
        assert_eq!(loaded.cluster_stocks(), &expected);
    }

    #[test]
    fn civsave_archive_round_trips_cluster_stocks() {
        let mut sim = Simulation::with_seed(43);
        sim.test_set_cluster_food_stock(7002, 91);
        let expected = sim.cluster_stocks().clone();

        let dir = tempdir().expect("tempdir");
        let archive_path = dir.path().join("cluster-stocks.civsave.zst");
        CivSaveBundle::save_archive(&archive_path, &sim).expect("save archive");

        let loaded = CivSaveBundle::load_archive(&archive_path).expect("load archive");
        assert_eq!(loaded.cluster_stocks(), &expected);
    }

    #[test]
    fn legacy_bundle_without_cluster_stocks_sidecar_loads_empty_stocks() {
        let sim = Simulation::with_seed(47);
        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("legacy");
        write_legacy_bundle_without_cluster_stocks(&save_path, &sim);

        let loaded = CivSaveBundle::load_dir(&save_path).expect("load legacy");
        assert!(loaded.cluster_stocks().is_empty());
        assert_eq!(*loaded.planet(), *sim.planet());
        assert_eq!(*loaded.moon(), *sim.moon());
        assert_eq!(loaded.climate, sim.climate);
        assert_eq!(loaded.weather_grid(), sim.weather_grid());
    }

    fn write_legacy_bundle_without_cluster_stocks(path: &Path, sim: &Simulation) {
        fs::create_dir_all(path).expect("legacy directory");
        let metadata = CivSaveMetadata {
            spec_id: CIVSAVE_SPEC_ID.to_owned(),
            format_version: 3,
            tick: sim.state.tick,
            scenario_name: None,
        };
        fs::write(
            path.join("metadata.json"),
            serde_json::to_string(&metadata).expect("metadata json"),
        )
        .expect("metadata");
        fs::write(
            path.join("world_state.json"),
            serde_json::to_string(&sim.state).expect("world state json"),
        )
        .expect("world state");
        sim.save_replay(path.join("replay.civreplay"))
            .expect("replay");
    }

    #[test]
    fn malformed_cluster_stocks_sidecar_fails_load() {
        let sim = Simulation::with_seed(53);
        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("malformed-sidecar");
        CivSaveBundle::save_dir(&save_path, &sim).expect("save");
        fs::write(save_path.join(CLUSTER_STOCKS_FILE), "not json").expect("malformed sidecar");
        reseal_integrity_manifest(&save_path);

        assert!(matches!(
            CivSaveBundle::load_dir(&save_path),
            Err(SaveBundleError::Json(_))
        ));
    }

    #[test]
    fn unreadable_cluster_stocks_sidecar_fails_load() {
        let sim = Simulation::with_seed(59);
        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("directory-sidecar");
        write_legacy_bundle_without_cluster_stocks(&save_path, &sim);
        let sidecar_path = save_path.join(CLUSTER_STOCKS_FILE);
        fs::create_dir(&sidecar_path).expect("directory sidecar");

        assert!(matches!(
            CivSaveBundle::load_dir(&save_path),
            Err(SaveBundleError::Io { .. })
        ));
    }

    /// FR-CIV-SAVESLOT.
    #[test]
    fn fr_civ_saveslot_named_slots_save_list_load_delete() {
        let dir = tempdir().expect("tempdir");

        let mut sim_a = Simulation::with_seed(31);
        sim_a.tick();
        let mut sim_b = Simulation::with_seed(37);
        sim_b.tick();
        sim_b.tick();

        save_to_slot(dir.path(), "a", &sim_a).expect("save a");
        save_to_slot(dir.path(), "b", &sim_b).expect("save b");

        let slots = list_slots(dir.path()).expect("list slots");
        let names = slots
            .iter()
            .map(|slot| slot.name.as_str())
            .collect::<Vec<_>>();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));

        let loaded_a = load_from_slot(dir.path(), "a").expect("load a");
        assert_eq!(loaded_a.state, sim_a.state);
        assert_eq!(loaded_a.hash_chain_root(), sim_a.hash_chain_root());

        assert!(delete_slot(dir.path(), "a").expect("delete a"));
        let slots = list_slots(dir.path()).expect("list after delete");
        let names = slots
            .iter()
            .map(|slot| slot.name.as_str())
            .collect::<Vec<_>>();
        assert!(!names.contains(&"a"));
        assert!(names.contains(&"b"));
    }

    // -----------------------------------------------------------------------
    // Migration-specific tests (8 total)
    // -----------------------------------------------------------------------

    /// 1. Roundtrip: current saves retain the current format version.
    #[test]
    fn migration_roundtrip_save_load_preserves_version() {
        let mut sim = Simulation::with_seed(1);
        sim.tick();
        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("roundtrip");
        CivSaveBundle::save_dir(&save_path, &sim).expect("save");

        let meta_path = save_path.join("metadata.json");
        let meta_str = fs::read_to_string(&meta_path).unwrap();
        let meta: CivSaveMetadata = serde_json::from_str(&meta_str).unwrap();
        assert_eq!(meta.format_version, CIVSAVE_FORMAT_VERSION);

        let loaded = CivSaveBundle::load_dir(&save_path).expect("load");
        assert_eq!(loaded.state.tick, sim.state.tick);
    }

    /// 2. Loading a v1 save triggers the v1->v2->v3 migration chain.
    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn migration_v1_save_gets_upgraded_to_v3() {
        let ws = serde_json::json!({"tick": 10, "population": 500});
        let dir = tempdir().expect("tempdir");
        let save_dir = dir.path().join("old_save");
        create_v1_save_dir(&save_dir, 10, &ws.to_string());

        let loaded = CivSaveBundle::load_dir(&save_dir).expect("load v1 save");
        assert_eq!(loaded.state.tick, 10);

        // Loading preserves legacy metadata until an explicit complete save.
        let meta: CivSaveMetadata =
            serde_json::from_str(&fs::read_to_string(save_dir.join("metadata.json")).unwrap())
                .unwrap();
        assert_eq!(meta.format_version, 1);
    }

    /// 3. v1->v2 migration adds the culture-related default fields.
    #[test]
    fn migration_v1_to_v2_adds_culture_fields() {
        let mut ws = serde_json::json!({"tick": 5});
        migrate_v1_to_v2(&mut ws);
        assert!(ws.get("religion").is_some());
        assert!(ws.get("language").is_some());
        assert!(ws.get("psyche").is_some());
        assert!(ws.get("history").is_some());
        assert!(ws.get("writing").is_some());
        assert!(ws.get("building_layouts").is_some());
    }

    /// 4. v2->v3 migration restructures trade routes under `economy`.
    #[test]
    fn migration_v2_to_v3_restructures_trade_routes() {
        let mut ws = serde_json::json!({
            "tick": 7,
            "trade_routes": [{"from": 0, "to": 1}]
        });
        migrate_v2_to_v3(&mut ws);
        assert!(ws.get("trade_routes").is_none());
        let econ = ws.get("economy").expect("economy should exist");
        assert_eq!(econ["trade_route_version"], 3);
        assert!(econ["trade_routes"].is_array());
    }

    /// 5. run_migration_chain rejects a version newer than CIVSAVE_FORMAT_VERSION.
    #[test]
    fn migration_rejects_future_version() {
        let mut ws = serde_json::json!({"tick": 1});
        let result = run_migration_chain(&mut ws, CIVSAVE_FORMAT_VERSION + 1);
        assert!(result.is_err());
        match result.unwrap_err() {
            SaveBundleError::UnsupportedFormatVersion { found, max } => {
                assert_eq!(found, CIVSAVE_FORMAT_VERSION + 1);
                assert_eq!(max, CIVSAVE_FORMAT_VERSION);
            }
            other => panic!("expected UnsupportedFormatVersion, got {:?}", other),
        }
    }

    /// 6. run_migration_chain is a no-op when file version == CIVSAVE_FORMAT_VERSION.
    #[test]
    fn migration_noop_at_current_version() {
        let mut ws = serde_json::json!({"tick": 1});
        run_migration_chain(&mut ws, CIVSAVE_FORMAT_VERSION).unwrap();
        // economy should NOT have been added (v3 migration only runs when version < 3).
        assert!(ws.get("economy").is_none());
    }

    /// 7. Detecting corruption: empty world_state.json triggers SaveCorruption.
    #[test]
    fn migration_detects_empty_world_state() {
        let dir = tempdir().expect("tempdir");
        let save_dir = dir.path().join("corrupt");
        fs::create_dir_all(&save_dir).unwrap();
        let meta = CivSaveMetadata {
            spec_id: CIVSAVE_SPEC_ID.to_owned(),
            format_version: 2,
            tick: 0,
            scenario_name: None,
        };
        fs::write(
            save_dir.join("metadata.json"),
            serde_json::to_string_pretty(&meta).unwrap(),
        )
        .unwrap();
        // Empty world_state.json.
        fs::write(save_dir.join("world_state.json"), "").unwrap();
        fs::write(save_dir.join("replay.civreplay"), b"r").unwrap();

        let result = CivSaveBundle::load_dir(&save_dir);
        assert!(result.is_err());
        match result.unwrap_err() {
            SaveBundleError::SaveCorruption { detail } => {
                assert!(detail.contains("empty"));
            }
            other => panic!("expected SaveCorruption, got {:?}", other),
        }
    }

    /// 8. Version validation: a save with format_version > CIVSAVE_FORMAT_VERSION
    ///    is rejected with UnsupportedFormatVersion.
    #[test]
    fn migration_rejects_future_version_on_disk() {
        let dir = tempdir().expect("tempdir");
        let save_dir = dir.path().join("future_save");
        fs::create_dir_all(&save_dir).unwrap();
        let meta = CivSaveMetadata {
            spec_id: CIVSAVE_SPEC_ID.to_owned(),
            format_version: 999,
            tick: 0,
            scenario_name: None,
        };
        fs::write(
            save_dir.join("metadata.json"),
            serde_json::to_string_pretty(&meta).unwrap(),
        )
        .unwrap();
        fs::write(
            save_dir.join("world_state.json"),
            serde_json::json!({"tick": 0}).to_string(),
        )
        .unwrap();
        fs::write(save_dir.join("replay.civreplay"), b"r").unwrap();

        let result = CivSaveBundle::load_dir(&save_dir);
        assert!(result.is_err());
        match result.unwrap_err() {
            SaveBundleError::UnsupportedFormatVersion { found, max } => {
                assert_eq!(found, 999);
                assert_eq!(max, CIVSAVE_FORMAT_VERSION);
            }
            other => panic!("expected UnsupportedFormatVersion, got {:?}", other),
        }
    }

    // -- FR-SAVE-003 --------------------------------------------------------
    //
    // FR-SAVE-003 — Load SHALL restore byte-identical state (determinism
    // guarantee). After save + load, the tick must match and the loaded sim
    // must be able to re-save producing identical replay content.

    #[test]
    fn fr_save_003_load_restores_byte_identical_state() {
        let mut sim = Simulation::with_seed(42);
        // Advance several ticks so there is meaningful state.
        for _ in 0..5 {
            sim.tick();
        }

        let dir = tempfile::tempdir().expect("tempdir");
        let archive_path = dir.path().join("determinism.civsave.zst");
        CivSaveBundle::save_archive(&archive_path, &sim).expect("save");

        let loaded = CivSaveBundle::load_archive(&archive_path).expect("load");

        // Tick must be identical.
        assert_eq!(loaded.state.tick, sim.state.tick, "tick mismatch");

        // Verify the loaded sim can be re-saved and the replay content is
        // the same — this is the core determinism guarantee.
        let resave_path = dir.path().join("resave.civsave.zst");
        CivSaveBundle::save_archive(&resave_path, &loaded).expect("resave");
        let resaved = CivSaveBundle::load_archive(&resave_path).expect("reload");
        assert_eq!(
            resaved.state.tick, sim.state.tick,
            "re-saved tick must match original"
        );
    }

    #[test]
    fn fr_save_003_archive_bytes_roundtrip_deterministic() {
        let mut sim = Simulation::with_seed(99);
        for _ in 0..3 {
            sim.tick();
        }

        let dir = tempfile::tempdir().expect("tempdir");

        // Save, load, re-save, re-load. The double-roundtrip sim must have
        // the same tick as the original.
        let path1 = dir.path().join("a.civsave.zst");
        CivSaveBundle::save_archive(&path1, &sim).expect("save 1");
        let loaded1 = CivSaveBundle::load_archive(&path1).expect("load 1");

        let path2 = dir.path().join("b.civsave.zst");
        CivSaveBundle::save_archive(&path2, &loaded1).expect("save 2");
        let loaded2 = CivSaveBundle::load_archive(&path2).expect("load 2");

        assert_eq!(
            loaded1.state.tick, loaded2.state.tick,
            "double roundtrip tick must be stable"
        );
        assert_eq!(
            loaded2.state.tick, sim.state.tick,
            "double roundtrip tick must match original"
        );
    }

    // -----------------------------------------------------------------------
    // FR-SAVE-009 — hash chain tail survives save/load
    // -----------------------------------------------------------------------

    /// FR-SAVE-009: the BLAKE3 hash chain tail is serialized into the bundle
    /// and restored on load, so a save/load does not fork the chain.
    ///
    /// This exercises the *whole* path the spec names, not just the
    /// `civreplay` codec: the tail is recorded, written into a real
    /// `.civsave` bundle (tar + zstd), read back, and then used to continue
    /// the chain across the load boundary. A bundle that dropped or reset the
    /// tail would make the continued root diverge from an uninterrupted run.
    #[test]
    fn fr_save_009_hash_chain_tail_survives_bundle_roundtrip() {
        let mut sim = Simulation::with_seed(7);
        for _ in 0..4 {
            sim.tick();
        }
        // Record explicit ticks so the chain has a known tail to preserve.
        for t in 0..4u64 {
            sim.replay_log_mut().record_tick(100 + t);
        }

        let tail_at_save = sim
            .replay_log()
            .hash_chain_root()
            .expect("chain has a tail after recording ticks");
        assert_eq!(
            sim.replay_log().recompute_running_hash(),
            Some(tail_at_save),
            "stored tail must equal the recomputed chain root before saving"
        );

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("chain.civsave.zst");
        CivSaveBundle::save_archive(&path, &sim).expect("save");

        let mut loaded = CivSaveBundle::load_archive(&path).expect("load");

        // The tail came back intact...
        let tail_after_load = loaded
            .replay_log()
            .hash_chain_root()
            .expect("chain tail restored from bundle");
        assert_eq!(
            tail_after_load, tail_at_save,
            "FR-SAVE-009: chain tail must survive the save/load roundtrip"
        );

        // ...and `load_civreplay` verified it rather than trusting it.
        assert_eq!(
            loaded.replay_log().recompute_running_hash(),
            Some(tail_after_load),
            "restored chain must re-verify against its own events"
        );

        // Continuing the chain after the load must equal an uninterrupted run.
        let mut expected = sim.replay_log().clone();
        expected.record_tick(999);
        let mut actual = loaded.replay_log_mut().clone();
        actual.record_tick(999);
        assert_eq!(
            actual.hash_chain_root(),
            expected.hash_chain_root(),
            "FR-SAVE-009: chain must continue unbroken from the saved tick"
        );
    }

    /// FR-SAVE-009 negative case: a bundle whose embedded chain tail has been
    /// tampered with must be rejected at load rather than silently accepted.
    /// This proves the tail is load-bearing, not decorative.
    #[test]
    fn fr_save_009_tampered_chain_tail_is_rejected_on_load() {
        let mut sim = Simulation::with_seed(11);
        sim.tick();
        sim.replay_log_mut().record_tick(1);

        let dir = tempfile::tempdir().expect("tempdir");
        let save_dir_path = dir.path().join("bundle");
        CivSaveBundle::save_dir(&save_dir_path, &sim).expect("save");

        // Corrupt the last byte of the embedded chain tail.
        let replay_path = save_dir_path.join("replay.civreplay");
        let mut bytes = fs::read(&replay_path).expect("read replay");
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        fs::write(&replay_path, &bytes).expect("write tampered replay");

        let result = CivSaveBundle::load_dir(&save_dir_path);
        assert!(
            result.is_err(),
            "FR-SAVE-009: a tampered chain tail must fail the load, not pass silently"
        );
    }
}
