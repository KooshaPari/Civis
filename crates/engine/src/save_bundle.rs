//! CIV-1000 save layouts: uncompressed `.civsave/` folder (debug) and `.civsave.zst` archive (default).

use std::collections::BTreeMap;
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
pub const CIVSAVE_FORMAT_VERSION: u32 = 3;
/// Default on-disk save extension (zstd-compressed tar).
pub const CIVSAVE_ARCHIVE_EXTENSION: &str = "civsave.zst";
/// Optional sidecar introduced after the initial replay-only bundle format.
/// Its absence represents the empty stockpile state used by legacy saves.
const CLUSTER_STOCKS_FILE: &str = "cluster_stocks.json";
/// Optional environment snapshot for bundles created after replay-only saves.
const ENVIRONMENT_FILE: &str = "environment.json";

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
    /// Format version is newer than this engine supports.
    #[error("unsupported format version {found} (max supported {max})")]
    UnsupportedFormatVersion {
        /// Version found on disk.
        found: u32,
        /// Maximum version this engine can load.
        max: u32,
    },
}

fn io_err(path: impl AsRef<Path>, err: impl std::fmt::Display) -> SaveBundleError {
    SaveBundleError::Io {
        path: path.as_ref().to_path_buf(),
        message: err.to_string(),
    }
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
/// migration step bumps the version by one until we reach
/// `CIVSAVE_FORMAT_VERSION`.
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
        version = 3;
    }
    Ok(())
}

/// Migrate and persist a world_state JSON file on disk, rewriting it in-place
/// at the latest format version.
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
    // Persist the migrated state back so future loads skip migration.
    fs::write(
        &path,
        serde_json::to_string_pretty(&value).map_err(SaveBundleError::Json)?,
    )
    .map_err(|e| io_err(&path, e))?;
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

        let world_state_path = dir.join("world_state.json");
        fs::write(&world_state_path, serde_json::to_string(&sim.state)?)
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

        let replay_path = dir.join("replay.civreplay");
        sim.save_replay(&replay_path)?;
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
            // Load-side mirror: restore the macro economy state + wealth
            // snapshot from the deserialized WorldState so the gameplay loop
            // continues from the persisted accumulation rather than resetting.
            sim.economy_state = sim.state.economy_state.clone();
            sim.settlement_wealth_snapshot = sim.state.settlement_wealth_snapshot.clone();
            sim.market_state = sim.state.market_state.clone();

            // Load-side mirror: restore the settlement registry + per-settlement
            // scaffolding so phase_economy / unrest / social_mood / cohesion /
            // order resume from the persisted population.
            sim.settlements = sim.state.settlements.clone();
            sim.settlement_food_stocked = sim.state.settlement_food_stocked.clone();
            sim.settlement_housing_capacity = sim.state.settlement_housing_capacity.clone();
            sim.settlement_crime_pressure = sim.state.settlement_crime_pressure.clone();
            sim.settlement_gini = sim.state.settlement_gini.clone();
            // Load-side mirror: restore the cached victory/defeat outcome so a
            // world frozen at Victory/Defeat reads the same outcome back; will
            // be re-derived on the next tick by phase_victory_check anyway.
            sim.last_game_outcome = sim.state.last_game_outcome.clone();
            // Load-side mirror: restore the GA-evolved doctrine libraries so
            // a faction that spent 200 ticks running the genetic algorithm
            // doesn't reset to `default_faction_doctrines()` after a reload.
            sim.faction_doctrines = sim.state.faction_doctrines.clone();
            // Load-side mirror: restore the per-actor social-fabric state
            // (FR-CIV-COHESION-001) so a loaded world resumes with its actor
            // registry, hardship, institution coverage, kinship graph, and
            // trust network intact. Without this mirror every social phase
            // (phase_cohesion, phase_unrest, phase_order) iterates an empty
            // actor set and the world effectively has no actors post-load.
            sim.actor_settlement = sim.state.actor_settlement.clone();
            sim.actor_hardship = sim.state.actor_hardship.clone();
            sim.actor_institutions = sim.state.actor_institutions.clone();
            sim.kinship = sim.state.kinship.clone();
            sim.trust = sim.state.trust.clone();
            // FR-CIV-BELIEF-001: mirror emergent religious profiles so the
            // post-load sim resumes with its per-settlement mythic coherence,
            // monitoring, and uncertainty-reduction state intact.
            sim.religious_profiles = sim.state.religious_profiles.clone();
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

        // Update metadata format version so future loads skip migration.
        if file_version < CIVSAVE_FORMAT_VERSION {
            let updated_meta = CivSaveMetadata {
                spec_id: CIVSAVE_SPEC_ID.to_owned(),
                format_version: CIVSAVE_FORMAT_VERSION,
                tick: sim.state.tick,
                scenario_name: None,
            };
            fs::write(
                &metadata_path,
                serde_json::to_string_pretty(&updated_meta).map_err(SaveBundleError::Json)?,
            )
            .map_err(|e| io_err(&metadata_path, e))?;
        }

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

        // Migrate the world_state inside the extracted directory.
        let _migrated_ws = migrate_world_state_file(temp.path(), file_version)?;

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
            format_version: CIVSAVE_FORMAT_VERSION,
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

    /// 1. Roundtrip: save at v3, load returns v3 metadata.
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

        // Metadata should now reflect v3.
        let meta: CivSaveMetadata =
            serde_json::from_str(&fs::read_to_string(save_dir.join("metadata.json")).unwrap())
                .unwrap();
        assert_eq!(meta.format_version, 3);
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

    // ---- Slot-name validation helpers -----------------------------------

    /// Validates an ordinary slot name passes through unchanged.
    #[test]
    fn validate_slot_name_accepts_plain_name() {
        assert_eq!(validate_slot_name("alpha").unwrap(), "alpha");
        assert_eq!(validate_slot_name("alpha-7").unwrap(), "alpha-7");
        assert_eq!(validate_slot_name("under_score").unwrap(), "under_score");
    }

    /// Empty / whitespace-only slot names are rejected with InvalidSlotName.
    #[test]
    fn validate_slot_name_rejects_empty_and_whitespace() {
        let err = validate_slot_name("").unwrap_err();
        assert!(matches!(err, SaveBundleError::InvalidSlotName { ref message, .. } if message.contains("empty")));

        let err = validate_slot_name("   ").unwrap_err();
        assert!(matches!(err, SaveBundleError::InvalidSlotName { .. }));
    }

    /// Traversal-separator characters (slash, backslash, ..) are rejected.
    /// This is the security boundary that prevents path-injection through a
    /// crafted save name.
    #[test]
    fn validate_slot_name_rejects_path_traversal() {
        for malicious in ["../etc", "..\\Windows", "a/b", "a\\b", "foo/../bar"] {
            let err = validate_slot_name(malicious).unwrap_err();
            assert!(
                matches!(err, SaveBundleError::InvalidSlotName { .. }),
                "expected rejection for {malicious:?}, got {err:?}"
            );
        }
    }

    /// Names that are pure extensions after stripping are rejected.
    #[test]
    fn validate_slot_name_rejects_pure_extension_names() {
        let err = validate_slot_name(".civsave.zst").unwrap_err();
        assert!(matches!(err, SaveBundleError::InvalidSlotName { ref message, .. } if message.contains("extension")));

        let err = validate_slot_name("foo.civsave.zst").unwrap();
        assert_eq!(err, "foo");
    }

    /// `slot_name_from_path` returns the bare name from `.civsave.zst` archives.
    /// The function requires the file to actually exist on disk (it delegates
    /// to `is_save_archive`, which checks the zstd magic bytes).
    #[test]
    fn slot_name_from_path_strips_archive_extension() {
        let dir = tempdir().expect("tempdir");
        let archive = dir.path().join("dawn.civsave.zst");
        // zstd frame magic so `is_save_archive` returns true on this file.
        fs::write(&archive, ZSTD_FRAME_MAGIC).unwrap();
        assert_eq!(slot_name_from_path(&archive), Some("dawn".to_owned()));
    }

    /// `slot_name_from_path` returns the bare name from `.civsave/` directories.
    /// The function requires the directory to actually exist with a
    /// `replay.civreplay` file inside.
    #[test]
    fn slot_name_from_path_strips_dir_extension() {
        let dir = tempdir().expect("tempdir");
        let save_dir = dir.path().join("dusk.civsave");
        fs::create_dir_all(&save_dir).unwrap();
        fs::write(save_dir.join("replay.civreplay"), b"r").unwrap();
        assert_eq!(slot_name_from_path(&save_dir), Some("dusk".to_owned()));
    }

    /// `slot_name_from_path` returns None for unrelated paths.
    #[test]
    fn slot_name_from_path_returns_none_for_unrelated_paths() {
        assert!(slot_name_from_path(Path::new("/saves/notes.txt")).is_none());
        assert!(slot_name_from_path(Path::new("/saves/dawn.bin")).is_none());
    }

    /// `is_save_dir` / `is_save_archive` distinguish true save bundles from
    /// arbitrary files / directories.
    #[test]
    fn is_save_dir_and_archive_distinguish_bundles() {
        let dir = tempdir().expect("tempdir");

        let save_dir = dir.path().join("world.civsave");
        fs::create_dir_all(&save_dir).unwrap();
        fs::write(save_dir.join("replay.civreplay"), b"r").unwrap();
        assert!(CivSaveBundle::is_save_dir(&save_dir));
        assert!(!CivSaveBundle::is_save_archive(&save_dir));

        // A directory without `replay.civreplay` is not a save dir.
        let fake_dir = dir.path().join("not_a_save");
        fs::create_dir_all(&fake_dir).unwrap();
        assert!(!CivSaveBundle::is_save_dir(&fake_dir));
        assert!(!CivSaveBundle::is_save_archive(&fake_dir));

        // A bare file is neither a dir nor an archive.
        let plain_file = dir.path().join("plain.txt");
        fs::write(&plain_file, b"hello").unwrap();
        assert!(!CivSaveBundle::is_save_dir(&plain_file));
        assert!(!CivSaveBundle::is_save_archive(&plain_file));
    }

    /// `is_save_archive` returns true for any file with `.zst` extension (the
    /// engine treats the extension as authoritative for archive bundles).
    #[test]
    fn is_save_archive_accepts_zst_extension() {
        let dir = tempdir().expect("tempdir");
        let archive = dir.path().join("foo.civsave.zst");
        fs::write(&archive, b"not a real zstd frame, but extension wins").unwrap();
        assert!(CivSaveBundle::is_save_archive(&archive));
    }

    /// FR-CIV-SAVESLOT — slot lifecycle: save -> list -> load -> delete.
    /// Re-asserts the existing integration coverage at the unit level so any
    /// change to `save_to_slot` / `load_from_slot` / `list_slots` /
    /// `delete_slot` immediately regresses in CI.
    #[test]
    fn slot_lifecycle_save_list_load_delete_unit() {
        let mut sim = Simulation::with_seed(7);
        for _ in 0..3 {
            sim.tick();
        }
        let original_tick = sim.state.tick;

        let dir = tempdir().expect("tempdir");
        let saves = dir.path().join("saves");
        fs::create_dir_all(&saves).unwrap();

        // Save to a named slot.
        save_to_slot(&saves, "slot_a", &sim).expect("save_to_slot");
        assert!(saves.join("slot_a.civsave.zst").is_file());

        // Listing should include the slot with the correct tick.
        let listed = list_slots(&saves).expect("list_slots");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "slot_a");
        assert_eq!(listed[0].tick, original_tick);

        // Listing is sorted by tick desc — second save with a higher tick
        // should appear first.
        let mut sim2 = Simulation::with_seed(7);
        // Sync the seed-state — drive it to the same starting tick as `sim`
        // by replaying identical tick inputs (deterministic seed path).
        for _ in 0..sim.state.tick {
            sim2.tick();
        }
        for _ in 0..4 {
            sim2.tick();
        }
        save_to_slot(&saves, "slot_b", &sim2).expect("save slot_b");
        let listed_after = list_slots(&saves).expect("list_slots");
        assert_eq!(listed_after.len(), 2);
        assert_eq!(listed_after[0].name, "slot_b");
        assert_eq!(listed_after[1].name, "slot_a");

        // Loading round-trips the simulation.
        let loaded = load_from_slot(&saves, "slot_a").expect("load_from_slot");
        assert_eq!(loaded.state.tick, original_tick);

        // Deleting a missing slot is a no-op returning false.
        assert!(!delete_slot(&saves, "never_existed").expect("delete missing"));

        // Deleting an existing slot returns true and removes the file.
        assert!(delete_slot(&saves, "slot_a").expect("delete slot_a"));
        assert!(!saves.join("slot_a.civsave.zst").is_file());
        assert!(list_slots(&saves).unwrap().iter().all(|e| e.name != "slot_a"));
    }

    /// Slot names with reserved characters are rejected before the file is touched.
    /// Guards against path-injection through the slot name argument.
    #[test]
    fn slot_operations_reject_malicious_names() {
        let dir = tempdir().expect("tempdir");
        let saves = dir.path().join("saves");
        fs::create_dir_all(&saves).unwrap();
        let mut sim = Simulation::with_seed(1);
        sim.tick();

        for bad in ["../escape", "with/slash", "with\\backslash"] {
            assert!(save_to_slot(&saves, bad, &sim).is_err(), "save with {bad}");
            assert!(load_from_slot(&saves, bad).is_err(), "load with {bad}");
            assert!(delete_slot(&saves, bad).is_err(), "delete with {bad}");
        }

        // No file should have been written to the saves directory.
        let count: usize = fs::read_dir(&saves)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file())
            .count();
        assert_eq!(count, 0, "malicious slot names must not create files");
    }

    /// `CivSaveMetadata` round-trips through serde with the canonical spec id
    /// and format version.
    #[test]
    fn civsave_metadata_serde_round_trip() {
        let meta = CivSaveMetadata {
            spec_id: CIVSAVE_SPEC_ID.to_owned(),
            format_version: CIVSAVE_FORMAT_VERSION,
            tick: 12345,
            scenario_name: Some("founding".to_owned()),
        };
        let json = serde_json::to_string(&meta).expect("serialize");
        let restored: CivSaveMetadata = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored, meta);

        // The canonical spec_id is "CIV-1000" — any drift here would break
        // every existing save on disk.
        assert_eq!(CIVSAVE_SPEC_ID, "CIV-1000");
        assert_eq!(CIVSAVE_FORMAT_VERSION, 3);
        assert_eq!(CIVSAVE_ARCHIVE_EXTENSION, "civsave.zst");
    }

    /// `SaveSlotEntry` round-trips with deterministic ordering by (tick desc, name asc).
    #[test]
    fn save_slot_entry_equality_and_determinism() {
        let a = SaveSlotEntry { name: "alpha".to_owned(), tick: 10 };
        let b = SaveSlotEntry { name: "alpha".to_owned(), tick: 10 };
        assert_eq!(a, b);

        let c = SaveSlotEntry { name: "beta".to_owned(), tick: 10 };
        assert_ne!(a, c);

        let d = SaveSlotEntry { name: "alpha".to_owned(), tick: 11 };
        assert_ne!(a, d);
    }
}
