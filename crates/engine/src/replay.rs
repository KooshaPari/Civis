//! Replay log support for deterministic simulation playback.

use crate::engine::Simulation;
use civ_tactics::DamageEvent;
use civ_voxel::WorldCoord;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

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
    /// Outcome of a research decision.
    ResearchOutcome {
        tick: u64,
        snapshot_hash: Vec<u8>,
        accepted: bool,
    },
    /// End-of-tick marker.
    Tick { tick: u64 },
}

/// Persistent replay log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayLog {
    pub events: Vec<ReplayEvent>,
    pub seed: u64,
    pub schema_version: u32,
}

impl Default for ReplayLog {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            seed: 0,
            schema_version: 1,
        }
    }
}

/// Replay-specific error.
#[derive(Debug)]
pub enum ReplayError {
    Io(std::io::Error),
    Ron(ron::Error),
    RonSpanned(ron::error::SpannedError),
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Ron(err) => write!(f, "{err}"),
            Self::RonSpanned(err) => write!(f, "{err}"),
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

    /// Record a research outcome.
    pub fn record_research(&mut self, tick: u64, snapshot_hash: Vec<u8>, accepted: bool) {
        self.events.push(ReplayEvent::ResearchOutcome {
            tick,
            snapshot_hash,
            accepted,
        });
    }

    /// Record a tick marker.
    pub fn record_tick(&mut self, tick: u64) {
        self.events.push(ReplayEvent::Tick { tick });
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
            }
        }
        Ok(())
    }
}
