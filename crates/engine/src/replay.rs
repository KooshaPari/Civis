//! Replay log support for deterministic simulation playback.

use crate::engine::Simulation;
use civ_tactics::DamageEvent;
use civ_voxel::WorldCoord;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

use crate::hash_chain::{
    chain_advance, chain_root_from_payloads, climate_event_bytes, combat_event_bytes,
    tick_event_bytes, GENESIS, HASH_LEN,
};
use crate::io::{read_text, write_text};
use civ_planet::{Climate, GeologyMap, WeatherCell};
use civ_voxel::MaterialId;

/// A single replayable simulation event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReplayEvent {
    /// A voxel write at a specific tick.
    VoxelWrite {
        tick: u64,
        pos: WorldCoord,
        value: MaterialId,
    },
    /// A queued damage event.
    Damage { tick: u64, event: DamageEvent },
    /// Per-soldier combat (war bridge) with attacker/defender pin ids (FR-CIV-TACTICS-025).
    Combat {
        tick: u64,
        shooter_id: u64,
        target_id: u64,
        event: DamageEvent,
    },
    /// Outcome of a research decision.
    ResearchOutcome {
        tick: u64,
        snapshot_hash: Vec<u8>,
        accepted: bool,
    },
    /// Climate + weather-grid + geology snapshot (FR-CIV-PLANET-060).
    Climate {
        tick: u64,
        climate: Climate,
        weather_grid: Vec<WeatherCell>,
        geology_map: GeologyMap,
    },
    /// End-of-tick marker.
    Tick { tick: u64 },
    /// Emergence-dashboard sample emitted on a sample tick (`emergence_metrics.v1`,
    /// FR-CIV-EMERG-003). Five normalised dashboard tiles recorded for replay parity.
    EmergenceMetrics {
        tick: u64,
        cluster_entropy: f32,
        ideology_homophily: f32,
        sentience_fraction: f32,
        psyche_stability: f32,
        diplomacy_tension: f32,
    },
    /// A recorded boolean RNG draw (FR-CORE-004) — keeps stochastic decisions
    /// reproducible across replays. `result` is the drawn boolean for `probability`.
    RngDraw {
        tick: u64,
        probability: f64,
        result: bool,
    },
    /// Mod manifest registered (`mod.loaded.v1`, FR-MOD-004).
    ModLoaded {
        tick: u64,
        mod_id: String,
        /// Display name from manifest (empty for legacy replay logs).
        #[serde(default)]
        mod_name: String,
        version: String,
        /// `mod.loaded.v1` JSON on the replay bus (empty for legacy logs).
        #[serde(default)]
        bus_json: String,
    },
    /// Mod removed at runtime (`mod.unloaded.v1`, FR-MOD-004 partial).
    ModUnloaded {
        tick: u64,
        mod_id: String,
        mod_name: String,
        reason: String,
        #[serde(default)]
        bus_json: String,
    },
    /// Session persisted to slot or autosave (`session.saved.v1`, CIV-1000).
    SessionSaved {
        tick: u64,
        session_id: String,
        save_id: String,
        slot: String,
        byte_size: u64,
        #[serde(default)]
        bus_json: String,
    },
    /// Mod capability denial (`mod.permission_violation.v1`, FR-MOD-004 partial).
    ModPermissionViolation {
        tick: u64,
        mod_id: String,
        call: String,
        #[serde(default)]
        domain: Option<String>,
        #[serde(default)]
        bus_json: String,
    },
}

/// Persistent replay log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayLog {
    pub events: Vec<ReplayEvent>,
    pub seed: u64,
    pub schema_version: u32,
    /// Latest BLAKE3 chain root after the most recent [`Self::record_tick`].
    #[serde(default)]
    pub running_hash: Option<[u8; HASH_LEN]>,
}

impl Default for ReplayLog {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            seed: 0,
            schema_version: 1,
            running_hash: None,
        }
    }
}

/// Replay-specific error.
#[derive(Debug)]
pub enum ReplayError {
    Io(std::io::Error),
    Ron(ron::Error),
    RonSpanned(ron::error::SpannedError),
    /// `.civreplay` container magic bytes do not match.
    InvalidMagic,
    /// Unsupported `.civreplay` container format version.
    UnsupportedFormatVersion(u32),
    /// File shorter than header, payload, or footer.
    Truncated,
    /// RON payload exceeds `u32::MAX` bytes.
    PayloadTooLarge,
    /// Payload is not valid UTF-8.
    InvalidUtf8(std::str::Utf8Error),
    /// Footer SHA-256 does not match header + payload.
    ChecksumMismatch,
    /// Stored [`ReplayLog::running_hash`] does not match the chain recomputed from tick events.
    HashChainMismatch,
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Ron(err) => write!(f, "{err}"),
            Self::RonSpanned(err) => write!(f, "{err}"),
            Self::InvalidMagic => write!(f, "invalid .civreplay magic"),
            Self::UnsupportedFormatVersion(v) => {
                write!(f, "unsupported .civreplay format version {v}")
            }
            Self::Truncated => write!(f, "truncated .civreplay file"),
            Self::PayloadTooLarge => write!(f, ".civreplay RON payload too large"),
            Self::InvalidUtf8(err) => write!(f, "{err}"),
            Self::ChecksumMismatch => write!(f, ".civreplay checksum mismatch"),
            Self::HashChainMismatch => write!(f, "replay hash chain mismatch"),
        }
    }
}

impl std::error::Error for ReplayError {}

impl From<std::io::Error> for ReplayError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ron::error::SpannedError> for ReplayError {
    fn from(value: ron::error::SpannedError) -> Self {
        Self::RonSpanned(value)
    }
}

impl From<ron::Error> for ReplayError {
    fn from(value: ron::Error) -> Self {
        Self::Ron(value)
    }
}

impl From<std::str::Utf8Error> for ReplayError {
    fn from(value: std::str::Utf8Error) -> Self {
        Self::InvalidUtf8(value)
    }
}

impl ReplayLog {
    /// Record a voxel write.
    pub fn record_voxel_write(&mut self, tick: u64, pos: WorldCoord, value: MaterialId) {
        self.events
            .push(ReplayEvent::VoxelWrite { tick, pos, value });
    }

    /// Record a damage event.
    pub fn record_damage(&mut self, tick: u64, event: DamageEvent) {
        self.events.push(ReplayEvent::Damage { tick, event });
    }

    /// Record an emergence-dashboard sample (`emergence_metrics.v1`, FR-CIV-EMERG-003).
    #[allow(clippy::too_many_arguments)]
    pub fn record_emergence_metrics(
        &mut self,
        tick: u64,
        cluster_entropy: f32,
        ideology_homophily: f32,
        sentience_fraction: f32,
        psyche_stability: f32,
        diplomacy_tension: f32,
    ) {
        self.events.push(ReplayEvent::EmergenceMetrics {
            tick,
            cluster_entropy,
            ideology_homophily,
            sentience_fraction,
            psyche_stability,
            diplomacy_tension,
        });
    }

    /// Record a boolean RNG draw (FR-CORE-004) for replay reproducibility.
    pub fn record_rng_draw(&mut self, tick: u64, probability: f64, result: bool) {
        self.events.push(ReplayEvent::RngDraw {
            tick,
            probability,
            result,
        });
    }

    /// Record a per-soldier combat engagement.
    pub fn record_combat(
        &mut self,
        tick: u64,
        shooter_id: u64,
        target_id: u64,
        event: DamageEvent,
    ) {
        self.events.push(ReplayEvent::Combat {
            tick,
            shooter_id,
            target_id,
            event,
        });
        let prev = self.running_hash.unwrap_or(GENESIS);
        let payload = combat_event_bytes(
            tick,
            shooter_id,
            target_id,
            event.center.x,
            event.center.y,
            event.center.z,
            event.radius_voxels,
            event.energy,
            0,
        );
        self.running_hash = Some(chain_advance(&prev, &payload));
    }

    /// Record a research outcome.
    pub fn record_research(&mut self, tick: u64, snapshot_hash: Vec<u8>, accepted: bool) {
        self.events.push(ReplayEvent::ResearchOutcome {
            tick,
            snapshot_hash,
            accepted,
        });
    }

    /// Record a climate snapshot and fold it into the running hash chain
    /// (FR-CIV-PLANET-060).
    pub fn record_climate(
        &mut self,
        tick: u64,
        climate: Climate,
        weather_grid: Vec<WeatherCell>,
        geology_map: GeologyMap,
    ) {
        let payload = climate_event_bytes(tick, &climate, &weather_grid, &geology_map);
        let prev = self.running_hash.unwrap_or(GENESIS);
        self.running_hash = Some(chain_advance(&prev, &payload));
        self.events.push(ReplayEvent::Climate {
            tick,
            climate,
            weather_grid,
            geology_map,
        });
    }

    /// Record a `mod.loaded.v1` lifecycle event with replay-bus JSON (FR-MOD-004 partial).
    pub fn record_mod_loaded(&mut self, record: &civ_mod_host::ModLoadedRecord) {
        let bus_json = civ_mod_host::format_mod_loaded_event_json(record);
        self.events.push(ReplayEvent::ModLoaded {
            tick: record.tick,
            mod_id: record.mod_id.clone(),
            mod_name: record.mod_name.clone(),
            version: record.version.clone(),
            bus_json,
        });
    }

    /// Record a `mod.unloaded.v1` lifecycle event with replay-bus JSON (FR-MOD-004 partial).
    pub fn record_mod_unloaded(&mut self, record: &civ_mod_host::ModUnloadedRecord) {
        let bus_json = civ_mod_host::format_mod_unloaded_event_json(record);
        self.events.push(ReplayEvent::ModUnloaded {
            tick: record.tick,
            mod_id: record.mod_id.clone(),
            mod_name: record.mod_name.clone(),
            reason: record.reason.clone(),
            bus_json,
        });
    }

    /// Record a `session.saved.v1` event with replay-bus JSON (FR-SAVE-002 partial).
    pub fn record_session_saved(
        &mut self,
        session_id: &str,
        save_id: &str,
        slot: &str,
        tick: u64,
        byte_size: u64,
    ) {
        let bus_json = civ_save_db::format_session_saved_event_json(
            session_id, save_id, slot, tick, byte_size,
        );
        self.events.push(ReplayEvent::SessionSaved {
            tick,
            session_id: session_id.to_string(),
            save_id: save_id.to_string(),
            slot: slot.to_string(),
            byte_size,
            bus_json,
        });
    }

    /// Record a `mod.permission_violation.v1` event with replay-bus JSON (CIV-0700).
    pub fn record_mod_permission_violation(
        &mut self,
        mod_id: &str,
        tick: u64,
        call: &str,
        domain: Option<civ_mod_host::WorldDomain>,
    ) {
        let bus_json =
            civ_mod_host::format_mod_permission_violation_json(mod_id, tick, call, domain);
        self.events.push(ReplayEvent::ModPermissionViolation {
            tick,
            mod_id: mod_id.to_string(),
            call: call.to_string(),
            domain: domain.map(|d| format!("{d:?}")),
            bus_json,
        });
    }

    /// `mod.permission_violation.v1` JSON payloads recorded at a specific tick.
    #[must_use]
    pub fn mod_permission_violation_bus_at_tick(&self, tick: u64) -> Vec<String> {
        self.events
            .iter()
            .filter_map(|event| match event {
                ReplayEvent::ModPermissionViolation {
                    tick: event_tick,
                    bus_json,
                    ..
                } if *event_tick == tick && !bus_json.is_empty() => Some(bus_json.clone()),
                ReplayEvent::ModPermissionViolation {
                    tick: event_tick,
                    mod_id,
                    call,
                    domain,
                    ..
                } if *event_tick == tick => {
                    let parsed_domain = domain.as_deref().and_then(parse_world_domain_label);
                    Some(civ_mod_host::format_mod_permission_violation_json(
                        mod_id,
                        *event_tick,
                        call,
                        parsed_domain,
                    ))
                }
                _ => None,
            })
            .collect()
    }

    /// `mod.loaded.v1` JSON payloads recorded at a specific tick (event feed / snapshot).
    #[must_use]
    pub fn mod_loaded_bus_at_tick(&self, tick: u64) -> Vec<String> {
        self.events
            .iter()
            .filter_map(|event| match event {
                ReplayEvent::ModLoaded {
                    tick: event_tick,
                    bus_json,
                    ..
                } if *event_tick == tick && !bus_json.is_empty() => Some(bus_json.clone()),
                ReplayEvent::ModLoaded {
                    tick: event_tick,
                    mod_id,
                    mod_name,
                    version,
                    ..
                } if *event_tick == tick => Some(civ_mod_host::format_mod_loaded_event_json(
                    &civ_mod_host::ModLoadedRecord {
                        mod_id: mod_id.clone(),
                        mod_name: mod_name.clone(),
                        version: version.clone(),
                        tick: *event_tick,
                    },
                )),
                _ => None,
            })
            .collect()
    }

    /// Record a `mod.permission_violation.v1` event from replay-bus JSON emitted by mod-host.
    pub fn record_mod_permission_violation_bus(&mut self, tick: u64, bus_json: &str) {
        let parsed = serde_json::from_str::<serde_json::Value>(bus_json).unwrap_or_default();
        let mod_id = parsed
            .get("mod_id")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        let call = parsed
            .get("call")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        let domain = parsed
            .get("domain")
            .and_then(|value| value.as_str())
            .map(|label| label.to_string());
        self.events.push(ReplayEvent::ModPermissionViolation {
            tick,
            mod_id,
            call,
            domain,
            bus_json: bus_json.to_string(),
        });
    }

    /// `session.saved.v1` JSON payloads recorded at a specific tick (event feed / snapshot).
    #[must_use]
    pub fn session_saved_bus_at_tick(&self, tick: u64) -> Vec<String> {
        self.events
            .iter()
            .filter_map(|event| match event {
                ReplayEvent::SessionSaved {
                    tick: event_tick,
                    bus_json,
                    ..
                } if *event_tick == tick && !bus_json.is_empty() => Some(bus_json.clone()),
                ReplayEvent::SessionSaved {
                    tick: event_tick,
                    session_id,
                    save_id,
                    slot,
                    byte_size,
                    ..
                } if *event_tick == tick => Some(civ_save_db::format_session_saved_event_json(
                    session_id,
                    save_id,
                    slot,
                    *event_tick,
                    *byte_size,
                )),
                _ => None,
            })
            .collect()
    }

    /// Collect `mod.loaded.v1` JSON payloads from recorded mod-load events.
    #[must_use]
    pub fn mod_loaded_bus_events(&self) -> Vec<String> {
        self.events
            .iter()
            .filter_map(|event| match event {
                ReplayEvent::ModLoaded { bus_json, .. } if !bus_json.is_empty() => {
                    Some(bus_json.clone())
                }
                ReplayEvent::ModLoaded {
                    tick,
                    mod_id,
                    mod_name,
                    version,
                    ..
                } => Some(civ_mod_host::format_mod_loaded_event_json(
                    &civ_mod_host::ModLoadedRecord {
                        mod_id: mod_id.clone(),
                        mod_name: mod_name.clone(),
                        version: version.clone(),
                        tick: *tick,
                    },
                )),
                _ => None,
            })
            .collect()
    }

    pub fn record_tick(&mut self, tick: u64) {
        self.events.push(ReplayEvent::Tick { tick });
        let prev = self.running_hash.unwrap_or(GENESIS);
        self.running_hash = Some(chain_advance(&prev, &tick_event_bytes(tick)));
    }

    /// Recompute the hash-chain root from tick + combat + climate markers in event order.
    #[must_use]
    pub fn recompute_running_hash(&self) -> Option<[u8; HASH_LEN]> {
        chain_root_from_payloads(self.events.iter().filter_map(|event| match event {
            ReplayEvent::Tick { tick } => Some(tick_event_bytes(*tick).to_vec()),
            ReplayEvent::Combat {
                tick,
                shooter_id,
                target_id,
                event,
            } => Some(combat_event_bytes(
                *tick,
                *shooter_id,
                *target_id,
                event.center.x,
                event.center.y,
                event.center.z,
                event.radius_voxels,
                event.energy,
                0,
            )),
            ReplayEvent::Climate {
                tick,
                climate,
                weather_grid,
                geology_map,
            } => Some(climate_event_bytes(
                *tick,
                climate,
                weather_grid,
                geology_map,
            )),
            _ => None,
        }))
    }

    /// Verify that a stored [`Self::running_hash`] matches the chain from tick events.
    ///
    /// Logs without a stored root (legacy) skip verification.
    pub fn verify_hash_chain(&self) -> Result<(), ReplayError> {
        let Some(stored) = self.running_hash else {
            return Ok(());
        };
        let expected = self
            .recompute_running_hash()
            .ok_or(ReplayError::HashChainMismatch)?;
        if stored != expected {
            return Err(ReplayError::HashChainMismatch);
        }
        Ok(())
    }

    /// Alias for [`Self::running_hash`] (hash-chain root after the last recorded tick).
    #[must_use]
    pub fn hash_chain_root(&self) -> Option<[u8; HASH_LEN]> {
        self.running_hash
    }

    /// Save the replay log as RON.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ReplayError> {
        let contents = ron::to_string(self)?;
        write_text(path, &contents)?;
        Ok(())
    }

    /// Load the replay log from RON.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ReplayError> {
        let contents = read_text(path)?;
        let log = ron::from_str(&contents)?;
        Ok(log)
    }


    /// Count [`ReplayEvent::RngDraw`] markers in this log.
    #[must_use]
    pub fn rng_draw_event_count(&self) -> usize {
        self.events
            .iter()
            .filter(|e| matches!(e, ReplayEvent::RngDraw { .. }))
            .count()
    }

    /// Count [`ReplayEvent::Combat`] markers in this log.
    #[must_use]
    pub fn combat_event_count(&self) -> usize {
        self.events
            .iter()
            .filter(|e| matches!(e, ReplayEvent::Combat { .. }))
            .count()
    }

    /// Replay all events into a simulation.
    pub fn replay(&self, into: &mut Simulation) -> Result<(), ReplayError> {
        for event in &self.events {
            match event {
                ReplayEvent::VoxelWrite { tick, pos, value } => {
                    into.apply_replay_voxel_write(*tick, *pos, *value);
                }
                ReplayEvent::Damage { tick, event } => {
                    into.apply_replay_damage(*tick, event);
                }
                ReplayEvent::Combat {
                    tick,
                    shooter_id: _,
                    target_id: _,
                    event,
                } => {
                    into.apply_replay_combat(*tick, event);
                }
                ReplayEvent::ResearchOutcome {
                    tick,
                    snapshot_hash,
                    accepted,
                } => {
                    into.apply_replay_research(*tick, snapshot_hash.clone(), *accepted);
                }
                ReplayEvent::Tick { tick } => {
                    into.apply_replay_tick(*tick);
                }
                ReplayEvent::Climate { .. } => {}
                ReplayEvent::ModLoaded { .. } => {}
                ReplayEvent::ModUnloaded { .. } => {}
                ReplayEvent::SessionSaved { .. } => {}
                ReplayEvent::ModPermissionViolation { .. } => {}
                // Informational markers — no world-state mutation on replay.
                ReplayEvent::EmergenceMetrics { .. } => {}
                ReplayEvent::RngDraw { .. } => {}
            }
        }
        Ok(())
    }
}

fn parse_world_domain_label(label: &str) -> Option<civ_mod_host::WorldDomain> {
    match label {
        "Economy" => Some(civ_mod_host::WorldDomain::Economy),
        "Military" => Some(civ_mod_host::WorldDomain::Military),
        "Climate" => Some(civ_mod_host::WorldDomain::Climate),
        "Diplomacy" => Some(civ_mod_host::WorldDomain::Diplomacy),
        "Citizens" => Some(civ_mod_host::WorldDomain::Citizens),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mod_permission_violation_records_bus_json_at_tick() {
        let mut log = ReplayLog::default();
        let bus_json =
            civ_mod_host::format_mod_permission_violation_json("demo-mod", 42, "action_emit", None);
        log.record_mod_permission_violation_bus(42, &bus_json);
        let at_tick = log.mod_permission_violation_bus_at_tick(42);
        assert_eq!(at_tick.len(), 1);
        assert!(at_tick[0].contains("mod.permission_violation.v1"));
        assert!(at_tick[0].contains("demo-mod"));
    }

    #[test]
    fn session_saved_records_bus_json_at_tick() {
        let mut log = ReplayLog::default();
        log.record_session_saved("sess-1", "save-abc", "slot-1", 42, 2048);
        assert_eq!(log.session_saved_bus_at_tick(42).len(), 1);
    }

    #[test]
    fn rng_draw_records_event_and_counts() {
        let mut log = ReplayLog::default();
        log.record_rng_draw(7, "diplomacy.kind", 42);
        log.record_rng_draw(8, "citizen.birth", 99);
        assert_eq!(log.rng_draw_event_count(), 2);
        let draws: Vec<&ReplayEvent> = log
            .events
            .iter()
            .filter(|e| matches!(e, ReplayEvent::RngDraw { .. }))
            .collect();
        assert_eq!(draws.len(), 2);
    }

    #[test]
    fn rng_draw_round_trips_through_save_load() {
        let mut log = ReplayLog::default();
        log.record_rng_draw(11, "diplomacy.kind", 12345);
        let file = tempfile::NamedTempFile::new().expect("temp file");
        log.save(file.path()).expect("save replay log");
        let loaded = ReplayLog::load(file.path()).expect("load replay log");
        assert_eq!(loaded.events, log.events);
        assert_eq!(loaded.rng_draw_event_count(), 1);
    }

    #[test]
    fn session_saved_round_trips_through_save_load() {
        let mut log = ReplayLog::default();
        log.record_session_saved("sess-1", "save-abc", "slot-1", 42, 2048);
        let file = tempfile::NamedTempFile::new().expect("temp file");
        log.save(file.path()).expect("save replay log");
        let loaded = ReplayLog::load(file.path()).expect("load replay log");
        assert_eq!(loaded.events, log.events);
        assert_eq!(
            loaded.session_saved_bus_at_tick(42),
            log.session_saved_bus_at_tick(42)
        );
    }
}
