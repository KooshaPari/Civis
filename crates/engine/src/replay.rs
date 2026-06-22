//! Replay log support for deterministic simulation playback.

use crate::engine::Simulation;
use civ_tactics::DamageEvent;
use civ_voxel::WorldCoord;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

use crate::hash_chain::{chain_root_from_ticks, tick_event_bytes, tick_hash, GENESIS, HASH_LEN};
use crate::io::{read_text, write_text};
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
    /// End-of-tick marker.
    Tick { tick: u64 },
    /// Mod manifest registered (`mod.loaded.v1`, FR-MOD-004).
    ModLoaded {
        tick: u64,
        mod_id: String,
        version: String,
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
    }

    /// Record a research outcome.
    pub fn record_research(&mut self, tick: u64, snapshot_hash: Vec<u8>, accepted: bool) {
        self.events.push(ReplayEvent::ResearchOutcome {
            tick,
            snapshot_hash,
            accepted,
        });
    }

    /// Record a tick marker and extend the per-tick hash chain (FR-CORE-005 partial).
    pub fn record_mod_loaded(&mut self, tick: u64, mod_id: &str, version: &str) {
        self.events.push(ReplayEvent::ModLoaded {
            tick,
            mod_id: mod_id.to_string(),
            version: version.to_string(),
        });
    }

    pub fn record_tick(&mut self, tick: u64) {
        self.events.push(ReplayEvent::Tick { tick });
        let prev = self.running_hash.unwrap_or(GENESIS);
        self.running_hash = Some(tick_hash(&prev, &tick_event_bytes(tick)));
    }

    /// Recompute the hash-chain root from [`ReplayEvent::Tick`] markers in event order.
    #[must_use]
    pub fn recompute_running_hash(&self) -> Option<[u8; HASH_LEN]> {
        chain_root_from_ticks(self.events.iter().filter_map(|event| match event {
            ReplayEvent::Tick { tick } => Some(*tick),
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
                ReplayEvent::ModLoaded { .. } => {}
            }
        }
        Ok(())
    }
}
