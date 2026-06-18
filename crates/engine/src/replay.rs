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
    /// End-of-tick emergence dashboard sample
    /// (`emergence_metrics.v1`, FR-CIV-EMERG-003). Side-band event:
    /// `record_emergence_metrics` does NOT advance the running hash
    /// chain, so emitting the dashboard block on the bus does not
    /// break replay compatibility. The acceptance test for the
    /// side-band contract is
    /// `replay_emergence_metrics_emit_does_not_change_hash_chain`.
    EmergenceMetrics {
        tick: u64,
        /// Five-tile summary at this tick.
        cluster_entropy: f32,
        ideology_homophily: f32,
        sentience_fraction: f32,
        psyche_stability: f32,
        diplomacy_tension: f32,
        /// Rolling-mean branching ratio `σ̄_W` (charter §3.6).
        #[serde(default)]
        branching_sigma: f32,
        /// Normalised edge-of-chaos score for `branching_sigma`.
        #[serde(default)]
        branching_sigma_score: f32,
        /// Charter regime label for `branching_sigma`.
        #[serde(default)]
        branching_regime: String,
        /// Power-law exponent α for the cluster-size distribution (charter §3.5).
        #[serde(default)]
        power_law_alpha: f32,
        /// Novelty rate: novel config fingerprints per window per civilian (charter §3.4).
        #[serde(default)]
        novelty_rate: f32,
        /// Normalised mutual information between material and faction distributions.
        #[serde(default)]
        mi_material_faction_norm: Option<f32>,
        /// Reconstructed `emergence_metrics.v1` JSON on the replay
        /// bus (empty for legacy replay logs).
        #[serde(default)]
        bus_json: String,
    },
    /// Stochastic draw from the engine RNG (FR-CORE-004 partial).
    ///
    /// Recorded so a replay session can be cross-checked against the same seed.
    /// `kind` is a stable label (e.g. `"diplomacy.kind"`, `"citizen.birth"`).
    /// `value` is the materialized `u64` consumed by the draw — comparisons like
    /// `value < u64::MAX * p` for `gen_bool(p)` are deterministic across runs.
    RngDraw {
        tick: u64,
        kind: String,
        value: u64,
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

    /// Record an `emergence_metrics.v1` side-band event with the
    /// five-tile dashboard summary at `tick` (FR-CIV-EMERG-003). The
    /// event is recorded on the replay bus but does NOT advance the
    /// running hash chain — see
    /// `recompute_running_hash` for the canonical chain payload
    /// filter. This preserves replay-compatibility for downstream
    /// consumers that diff the chain root before/after the dashboard
    /// block is enabled.
    pub fn record_emergence_metrics(
        &mut self,
        tick: u64,
        cluster_entropy: f32,
        ideology_homophily: f32,
        sentience_fraction: f32,
        psyche_stability: f32,
        diplomacy_tension: f32,
        branching_sigma: f32,
        branching_sigma_score: f32,
        branching_regime: &str,
        power_law_alpha: f32,
        novelty_rate: f32,
        mi_material_faction_norm: Option<f32>,
    ) {
        let bus_json = format_emergence_metrics_event_json(
            tick,
            cluster_entropy,
            ideology_homophily,
            sentience_fraction,
            psyche_stability,
            diplomacy_tension,
            branching_sigma,
            branching_sigma_score,
            branching_regime,
            power_law_alpha,
            novelty_rate,
            mi_material_faction_norm,
        );
        self.events.push(ReplayEvent::EmergenceMetrics {
            tick,
            cluster_entropy,
            ideology_homophily,
            sentience_fraction,
            psyche_stability,
            diplomacy_tension,
            branching_sigma,
            branching_sigma_score,
            branching_regime: branching_regime.to_string(),
            power_law_alpha,
            novelty_rate,
            mi_material_faction_norm,
            bus_json,
        });
    }

    /// `emergence_metrics.v1` JSON payloads recorded at a specific
    /// tick (event feed / snapshot). The wire shape is the same as
    /// the bus event so consumers can use either the typed accessor
    /// or the raw string.
    #[must_use]
    pub fn emergence_metrics_bus_at_tick(&self, tick: u64) -> Vec<String> {
        self.events
            .iter()
            .filter_map(|event| match event {
                ReplayEvent::EmergenceMetrics {
                    tick: event_tick,
                    bus_json,
                    ..
                } if *event_tick == tick && !bus_json.is_empty() => Some(bus_json.clone()),
                ReplayEvent::EmergenceMetrics {
                    tick: event_tick,
                    cluster_entropy,
                    ideology_homophily,
                    sentience_fraction,
                    psyche_stability,
                    diplomacy_tension,
                    branching_sigma,
                    branching_sigma_score,
                    branching_regime,
                    power_law_alpha,
                    novelty_rate,
                    mi_material_faction_norm,
                    ..
                } if *event_tick == tick => Some(format_emergence_metrics_event_json(
                    *event_tick,
                    *cluster_entropy,
                    *ideology_homophily,
                    *sentience_fraction,
                    *psyche_stability,
                    *diplomacy_tension,
                    *branching_sigma,
                    *branching_sigma_score,
                    branching_regime,
                    *power_law_alpha,
                    *novelty_rate,
                    *mi_material_faction_norm,
                )),
                _ => None,
            })
            .collect()
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

    /// `emergence_metrics.v1` event counter.
    #[must_use]
    pub fn emergence_metrics_event_count(&self) -> usize {
        self.events
            .iter()
            .filter(|e| matches!(e, ReplayEvent::EmergenceMetrics { .. }))
            .count()
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

    /// Record a stochastic draw (FR-CORE-004 partial).
    pub fn record_rng_draw(&mut self, tick: u64, kind: &str, value: u64) {
        self.events.push(ReplayEvent::RngDraw {
            tick,
            kind: kind.to_string(),
            value,
        });
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

/// Build the `emergence_metrics.v1` replay-bus JSON for a single
/// dashboard sample (FR-CIV-EMERG-003). The wire shape is the
/// dashboard's five-tile summary, prefixed with the standard replay-
/// bus envelope (`event`, `schema`, `tick`).
fn format_emergence_metrics_event_json(
    tick: u64,
    cluster_entropy: f32,
    ideology_homophily: f32,
    sentience_fraction: f32,
    psyche_stability: f32,
    diplomacy_tension: f32,
    branching_sigma: f32,
    branching_sigma_score: f32,
    branching_regime: &str,
    power_law_alpha: f32,
    novelty_rate: f32,
    mi_material_faction_norm: Option<f32>,
) -> String {
    serde_json::json!({
        "event": "emergence_metrics.v1",
        "schema": "emergence_metrics.v1",
        "tick": tick,
        "cluster_entropy": cluster_entropy,
        "ideology_homophily": ideology_homophily,
        "sentience_fraction": sentience_fraction,
        "psyche_stability": psyche_stability,
        "diplomacy_tension": diplomacy_tension,
        "branching_sigma": branching_sigma,
        "branching_sigma_score": branching_sigma_score,
        "branching_regime": branching_regime,
        "power_law_alpha": power_law_alpha,
        "novelty_rate": novelty_rate,
        "mi_material_faction_norm": mi_material_faction_norm,
    })
    .to_string()
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

    /// FR-CIV-EMERG-003: `record_emergence_metrics` records a
    /// side-band `emergence_metrics.v1` event on the replay bus. The
    /// accessor returns the canonical JSON envelope with the five
    /// dashboard fields, and the event counter reports exactly one
    /// per call.
    #[test]
    fn emerg_emerg_003_record_emergence_metrics_emits_bus_event() {
        let mut log = ReplayLog::default();
        log.record_emergence_metrics(50, 0.5, 0.25, 0.0, 1.0, 0.8, 0.95, 0.71, "Edge of chaos (target)", 1.8, 0.003, Some(0.42));
        let at_tick = log.emergence_metrics_bus_at_tick(50);
        assert_eq!(at_tick.len(), 1);
        assert!(at_tick[0].contains("\"event\":\"emergence_metrics.v1\""));
        assert!(at_tick[0].contains("\"cluster_entropy\":0.5"));
        assert!(at_tick[0].contains("\"ideology_homophily\":0.25"));
        assert!(at_tick[0].contains("\"sentience_fraction\":0.0"));
        assert!(at_tick[0].contains("\"psyche_stability\":1.0"));
        assert!(at_tick[0].contains("\"diplomacy_tension\":0.8"));
        assert_eq!(log.emergence_metrics_event_count(), 1);
    }

    /// Criticality metrics (power_law_alpha, novelty_rate, mi_material_faction_norm)
    /// are included in the `emergence_metrics.v1` bus JSON.
    #[test]
    fn emergence_metrics_bus_json_contains_criticality_keys() {
        let mut log = ReplayLog::default();
        log.record_emergence_metrics(
            10, 0.3, 0.1, 0.5, 0.9, 0.2,
            0.95, 0.71, "Edge of chaos (target)",
            2.1, 0.007, Some(0.55),
        );
        let at_tick = log.emergence_metrics_bus_at_tick(10);
        assert_eq!(at_tick.len(), 1);
        let json = &at_tick[0];
        assert!(json.contains("\"power_law_alpha\""), "missing power_law_alpha in bus JSON");
        assert!(json.contains("\"novelty_rate\""), "missing novelty_rate in bus JSON");
        assert!(json.contains("\"mi_material_faction_norm\""), "missing mi_material_faction_norm in bus JSON");
    }

    /// mi_material_faction_norm=None serialises as JSON null.
    #[test]
    fn emergence_metrics_bus_json_mi_none_is_null() {
        let mut log = ReplayLog::default();
        log.record_emergence_metrics(
            20, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, "Subcritical (heat-death risk)",
            0.0, 0.0, None,
        );
        let at_tick = log.emergence_metrics_bus_at_tick(20);
        assert_eq!(at_tick.len(), 1);
        assert!(at_tick[0].contains("\"mi_material_faction_norm\":null"), "None should serialize as null");
    }

    /// FR-CIV-EMERG-003: recording the dashboard block does NOT
    /// advance the running hash chain. The acceptance test name is
    /// pinned in the spec — `replay_emergence_metrics_emit_does_not_
    /// change_hash_chain`. We check that:
    /// 1. `record_emergence_metrics` does not mutate `running_hash`
    ///    directly (it is a side-band).
    /// 2. `recompute_running_hash` ignores the `EmergenceMetrics`
    ///    variant, so the chain root after a 5-tick simulation
    ///    matches before/after the dashboard block is enabled.
    #[test]
    fn replay_emergence_metrics_emit_does_not_change_hash_chain() {
        let mut log = ReplayLog::default();
        log.record_tick(1);
        log.record_tick(2);
        log.record_tick(3);
        // Before dashboard emission:
        let root_before = log.recompute_running_hash();
        // Record a dashboard event at the boundary; the running_hash
        // must NOT change.
        log.record_emergence_metrics(50, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, "Subcritical (heat-death risk)", 0.0, 0.0, None);
        let root_after_record = log.recompute_running_hash();
        assert_eq!(
            root_before, root_after_record,
            "recording emergence_metrics must not change the hash chain"
        );
        // The accessor that returns the chain root from the event
        // log also agrees (sanity check on `recompute_running_hash`).
        assert_eq!(root_before, log.running_hash);
    }
}
