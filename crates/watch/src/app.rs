//! Shared app state, snapshot DTOs, and helpers.

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU16, AtomicU8},
        Arc,
    },
    time::Duration,
};

use axum::body::Bytes;
use axum::http::HeaderValue;
use civ_economy::Stocks;
use civ_engine::{DiplomacyKind, JobType, ModBrowserEntry, Simulation};
use civ_laws::LawDb;
use civ_save_db::SaveDb;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex, RwLock};

use crate::terrain::Terrain;

pub(crate) fn env_u16(name: &str, default: u16) -> u16 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

pub(crate) fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

pub(crate) fn resolve_data_dir() -> PathBuf {
    std::env::var("CIVIS_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub(crate) fn resolve_session_id() -> String {
    std::env::var("CIVIS_SESSION_ID")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SampleCivilian {
    pub(crate) age: u32,
    pub(crate) health: f64,
    pub(crate) ideology: f64,
    pub(crate) welfare: f64,
    pub(crate) job: Option<JobLabel>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum JobLabel {
    Farmer,
    Warrior,
    Scholar,
    Trader,
    Priest,
    Admin,
    Unemployed,
}

impl From<JobType> for JobLabel {
    fn from(value: JobType) -> Self {
        match value {
            JobType::Farmer => Self::Farmer,
            JobType::Warrior => Self::Warrior,
            JobType::Scholar => Self::Scholar,
            JobType::Trader => Self::Trader,
            JobType::Priest => Self::Priest,
            JobType::Admin => Self::Admin,
            JobType::Unemployed => Self::Unemployed,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CivPin {
    pub(crate) idx: u32,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) dx: f32,
    pub(crate) dy: f32,
    pub(crate) job: Option<JobLabel>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MilitaryPin {
    pub(crate) id: u64,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) unit_type: String,
    pub(crate) faction: u32,
    pub(crate) strength: f32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DamagePulse {
    pub(crate) x: f32,
    pub(crate) y: f32,
    /// Engaging unit pin id when damage came from military contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) unit_a: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) unit_b: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DisasterEvent {
    pub(crate) tick: u64,
    pub(crate) kind: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) radius: f32,
    pub(crate) severity: f32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Faction {
    pub(crate) id: u32,
    pub(crate) color: [u8; 3],
    pub(crate) capital: [f32; 2],
    pub(crate) radius: f32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub(crate) enum BuildingKind {
    Residential,
    Commercial,
    Industrial,
    Civic,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Building {
    pub(crate) id: u32,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) kind: BuildingKind,
    pub(crate) era: u8,
    pub(crate) faction_id: u32,
    pub(crate) occupants: u32,
    pub(crate) capacity: u32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HousingStats {
    pub(crate) total_capacity: u32,
    pub(crate) occupied: u32,
    pub(crate) homeless: u32,
    pub(crate) vacancy_rate: f32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Road {
    pub(crate) from: [f32; 2],
    pub(crate) to: [f32; 2],
    pub(crate) width: f32,
    pub(crate) kind: RoadKind,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) enum RoadKind {
    Trail,
    Dirt,
    Paved,
    Highway,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TradeRoute {
    pub(crate) from_faction: u32,
    pub(crate) to_faction: u32,
    pub(crate) goods: String,
    pub(crate) volume: f32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GameEvent {
    pub(crate) tick: u64,
    pub(crate) kind: String,
    pub(crate) message: String,
    pub(crate) faction_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TechNode {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) era_min: u16,
    pub(crate) unlocked: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct InstitutionRow {
    pub(crate) id: u32,
    pub(crate) kind: String,
    pub(crate) balance_joules: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EconomySnapshot {
    pub(crate) energy_budget: f64,
    pub(crate) faction_treasury: Vec<FactionTreasury>,
    pub(crate) production_rates: ProductionRates,
    pub(crate) institutions: Vec<InstitutionRow>,
    pub(crate) resources: ResourceSnapshot,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WeatherSnapshot {
    pub(crate) season: String,
    pub(crate) temperature: f32,
    pub(crate) wind_speed: f32,
    pub(crate) precipitation: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ResourceSnapshot {
    pub(crate) food: f64,
    pub(crate) wood: f64,
    pub(crate) metal: f64,
    pub(crate) energy: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PopulationPulse {
    pub(crate) tick: u64,
    pub(crate) entity_id: u64,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DiplomacyPulse {
    pub(crate) tick: u64,
    pub(crate) faction_a: u32,
    pub(crate) faction_b: u32,
    pub(crate) kind: DiplomacyKind,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FactionTreasury {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) balance: f64,
    pub(crate) trade_balance: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProductionRates {
    pub(crate) food_per_tick: f64,
    pub(crate) wood_per_tick: f64,
    pub(crate) metal_per_tick: f64,
    pub(crate) energy_per_tick: f64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TradeTickSummary {
    pub(crate) balances: std::collections::HashMap<u32, f64>,
    pub(crate) volume: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Snapshot {
    pub(crate) tick: u64,
    pub(crate) tick_dt_ms: u32,
    pub(crate) current_era: u16,
    pub(crate) population: u64,
    pub(crate) settlement_count: u32,
    pub(crate) cluster_stocks: std::collections::BTreeMap<u64, Stocks>,
    pub(crate) voxel_dirty_count: usize,
    pub(crate) voxel_chunk_count: usize,
    pub(crate) sample_civilians: Vec<SampleCivilian>,
    pub(crate) civ_pins: Vec<CivPin>,
    pub(crate) factions: Vec<Faction>,
    pub(crate) buildings: Vec<Building>,
    pub(crate) housing_stats: HousingStats,
    pub(crate) roads: Vec<Road>,
    pub(crate) trade_routes: Vec<TradeRoute>,
    pub(crate) economy: EconomySnapshot,
    pub(crate) trade_volume_this_tick: f64,
    pub(crate) births_this_tick: u32,
    pub(crate) deaths_this_tick: u32,
    pub(crate) diplomacy_events: Vec<DiplomacyPulse>,
    pub(crate) military_units: Vec<MilitaryPin>,
    pub(crate) damage_events: Vec<DamagePulse>,
    pub(crate) damage_events_count: u32,
    pub(crate) disaster_events: Vec<DisasterEvent>,
    pub(crate) birth_events: Vec<PopulationPulse>,
    pub(crate) death_events: Vec<PopulationPulse>,
    pub(crate) tech_tree: Vec<TechNode>,
    pub(crate) events: Vec<GameEvent>,
    pub(crate) is_day: bool,
    pub(crate) weather: WeatherSnapshot,
    pub(crate) speed: u8,
    /// Loaded CivLab mods for dashboard mod browser (FR-CIV-TACTICS-054).
    pub(crate) mods: Vec<ModBrowserEntry>,
}

/// Pre-serialized terrain JSON and a stable ETag for cheap repeat fetches.
#[derive(Clone)]
pub(crate) struct TerrainCache {
    pub(crate) body: Bytes,
    pub(crate) etag: HeaderValue,
}

impl TerrainCache {
    pub(crate) fn from_terrain(terrain: &Terrain) -> Self {
        let body = Bytes::from(serde_json::to_vec(terrain).expect("terrain serializes"));
        let etag = format!("\"{:016x}\"", terrain.heights_fingerprint());
        Self {
            body,
            etag: HeaderValue::from_str(&etag).expect("valid etag"),
        }
    }
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) latest: Arc<RwLock<Option<Snapshot>>>,
    pub(crate) tx: broadcast::Sender<Snapshot>,
    pub(crate) terrain: Arc<Terrain>,
    pub(crate) terrain_cache: TerrainCache,
    pub(crate) laws: Arc<LawDb>,
    pub(crate) sim: Arc<Mutex<Simulation>>,
    pub(crate) military: Arc<Mutex<Vec<MilitaryPin>>>,
    pub(crate) target_era: Arc<AtomicU16>,
    pub(crate) speed: Arc<AtomicU8>,
    pub(crate) saves_dir: Arc<PathBuf>,
    pub(crate) mods_dir: Arc<PathBuf>,
    pub(crate) session_id: String,
    pub(crate) save_db: Arc<SaveDb>,
    pub(crate) http: reqwest::Client,
}

#[derive(Debug, Serialize)]
pub(crate) struct ControlOk {
    pub(crate) ok: bool,
    pub(crate) message: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PlaceVoxelReq {
    pub(crate) x: i64,
    pub(crate) y: i64,
    pub(crate) z: i64,
    pub(crate) material: u16,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SpawnCivilianReq {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) faction: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SpawnEntityReq {
    pub(crate) kind: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) faction: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DamageReq {
    pub(crate) x: i64,
    pub(crate) y: i64,
    pub(crate) z: i64,
    pub(crate) radius: u8,
    pub(crate) energy: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SpeedReq {
    pub(crate) speed: u8,
}

pub(crate) const PRODUCTION_SLOTS: [&str; 5] = ["slot-1", "slot-2", "slot-3", "slot-4", "slot-5"];
pub(crate) const AUTOSAVE_RING_MAX: usize = 10;

#[derive(Debug, Deserialize)]
pub(crate) struct SaveReq {
    pub(crate) filename: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SlotReq {
    pub(crate) slot: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SaveResponse {
    pub(crate) ok: bool,
    pub(crate) path: String,
    pub(crate) tick: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct LoadResponse {
    pub(crate) ok: bool,
    pub(crate) tick: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct SaveListEntry {
    pub(crate) name: String,
    pub(crate) size_bytes: u64,
    pub(crate) modified: Option<u64>,
    pub(crate) save_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) save_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tick: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ModCatalogEntry {
    pub(crate) source: String,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) mod_type: String,
    pub(crate) kind: String,
    pub(crate) installed: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) signed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) author_pubkey_hex: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InstallModReq {
    pub(crate) source: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UnloadModReq {
    pub(crate) mod_id: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReloadModReq {
    pub(crate) mod_id: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PublishModReq {
    pub(crate) source: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct PublishModResponse {
    pub(crate) ok: bool,
    pub(crate) published_source: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PublishedModEntry {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) source: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UploadModReq {
    pub(crate) filename: String,
    pub(crate) data_base64: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct UploadModResponse {
    pub(crate) ok: bool,
    pub(crate) source: String,
}

pub(crate) const REMOTE_MOD_MAX_BYTES: usize = 50 * 1024 * 1024;
pub(crate) const REMOTE_FETCH_TIMEOUT: Duration = Duration::from_secs(30);
pub(crate) const REMOTE_MOD_META_NAME: &str = "meta.json";
pub(crate) const REMOTE_MOD_ARCHIVE_NAME: &str = "mod.civmod";
pub(crate) const REMOTE_REGISTRY_NAME: &str = "remote-registry.json";

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct RemoteModRegistry {
    #[serde(default)]
    pub(crate) require_registry: bool,
    #[serde(default)]
    pub(crate) entries: Vec<RemoteModRegistryEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RemoteModRegistryEntry {
    pub(crate) url_prefix: String,
    #[serde(default)]
    pub(crate) mod_id: Option<String>,
    #[serde(default)]
    pub(crate) require_signature: bool,
    #[serde(default)]
    pub(crate) allowed_pubkeys: Vec<String>,
}

pub(crate) fn default_law_db() -> LawDb {
    LawDb::load_ron(
        r#"(
            version: 0,
            laws: [
                (
                    id: "mass_conservation",
                    kind: Conservation,
                    era_min: 0,
                    inputs: [],
                    outputs: [],
                    losses: [],
                    dependencies: [],
                ),
                (
                    id: "steel",
                    kind: Material,
                    era_min: 4,
                    inputs: ["iron_ore", "coal"],
                    outputs: ["steel_ingot"],
                    losses: ["slag"],
                    dependencies: ["mass_conservation"],
                ),
                (
                    id: "fusion_power",
                    kind: FictionalExtension,
                    era_min: 9,
                    inputs: ["deuterium"],
                    outputs: ["energy"],
                    losses: ["helium_4"],
                    dependencies: ["mass_conservation"],
                ),
            ],
        )"#,
    )
    .expect("sample law db")
}

#[derive(Debug, Deserialize)]
pub(crate) struct FetchModReq {
    pub(crate) url: String,
    #[serde(default)]
    pub(crate) mod_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct FetchModResponse {
    pub(crate) ok: bool,
    pub(crate) id: String,
    pub(crate) source: String,
    pub(crate) path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RemoteModMeta {
    pub(crate) id: String,
    pub(crate) url: String,
    pub(crate) fetched_at: u64,
    #[serde(default)]
    pub(crate) signed: bool,
    #[serde(default)]
    pub(crate) author_pubkey_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RemoteModEntry {
    pub(crate) id: String,
    pub(crate) path: String,
    pub(crate) fetched_at: u64,
    pub(crate) url: String,
    pub(crate) signed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) author_pubkey_hex: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `env_u16` falls back to the default when the variable is unset or
    /// unparsable (no env mutation -> race-free in the parallel test runner).
    #[test]
    fn env_u16_defaults_when_var_absent() {
        // PID-unique key cannot pre-exist in the host environment.
        let key = format!("CIVIS_TEST_ABSENT_{}", std::process::id());
        assert_eq!(env_u16(&key, 8080), 8080);
        assert_eq!(env_u16(&key, 0), 0);
    }

    /// `env_u64` falls back to the default when the variable is unset or
    /// unparsable (no env mutation -> race-free in the parallel test runner).
    #[test]
    fn env_u64_defaults_when_var_absent() {
        let key = format!("CIVIS_TEST_ABSENT_{}", std::process::id());
        assert_eq!(env_u64(&key, 42), 42);
        assert_eq!(env_u64(&key, 0), 0);
    }

    /// `resolve_session_id` is always a usable, non-empty identifier — a fresh
    /// UUID when unset, or the configured value — regardless of environment.
    #[test]
    fn resolve_session_id_is_non_empty() {
        assert!(!resolve_session_id().is_empty());
        // Two unset calls generate distinct UUIDs (no accidental constant).
        if std::env::var("CIVIS_SESSION_ID").is_err() {
            assert_ne!(resolve_session_id(), resolve_session_id());
        }
    }

    /// `resolve_data_dir` always yields a real path (defaults to `.`).
    #[test]
    fn resolve_data_dir_is_a_path() {
        assert!(!resolve_data_dir().as_os_str().is_empty());
    }

    /// The fixed production save slots and autosave ring bound are stable.
    #[test]
    fn production_slot_table_is_stable() {
        assert_eq!(PRODUCTION_SLOTS.len(), 5);
        assert_eq!(PRODUCTION_SLOTS[0], "slot-1");
        assert_eq!(PRODUCTION_SLOTS[4], "slot-5");
        assert_eq!(AUTOSAVE_RING_MAX, 10);
    }
}
