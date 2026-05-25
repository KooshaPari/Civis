//! UX protocol helpers — spawn events and era timelapse tick advancement.
//!
//! Pure Rust so unit tests do not require the Godot runtime.

use serde::Serialize;

/// civ-watch `POST /control/spawn_civilian` JSON body.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpawnCivilianBody {
    /// World X coordinate.
    pub x: f32,
    /// World Y coordinate (Godot uses Z as ground plane; mapped at call site).
    pub y: f32,
    /// Faction id for the new civilian.
    pub faction: i32,
}

/// Protocol entity-create event emitted per successful spawn (FR-CIV-UX-000).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityCreateEvent {
    /// Monotonic entity id assigned by the protocol layer.
    pub entity_id: u64,
    /// Faction id copied from the spawn request.
    pub faction: u32,
}

/// Build the spawn request body sent to civ-watch.
pub fn spawn_civilian_body(x: f32, y: f32, faction: i32) -> SpawnCivilianBody {
    SpawnCivilianBody { x, y, faction }
}

/// Map one spawn request to one entity-create event.
pub fn entity_create_from_spawn(body: &SpawnCivilianBody, entity_seq: u64) -> EntityCreateEvent {
    EntityCreateEvent {
        entity_id: entity_seq,
        faction: body.faction as u32,
    }
}

/// Process N spawn requests; returns exactly N entity-create events.
pub fn spawn_batch_events(spawns: &[SpawnCivilianBody], start_id: u64) -> Vec<EntityCreateEvent> {
    spawns
        .iter()
        .enumerate()
        .map(|(i, body)| entity_create_from_spawn(body, start_id + i as u64))
        .collect()
}

/// Default ticks per era for timelapse UI (Godot + tests).
pub const DEFAULT_ERA_LENGTH_TICKS: u32 = 5_000;

/// Spawn palette entry (FR-CIV-UX-006); only [`SpawnKind::Civilian`] is wired on server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnKind {
    /// Civilian entity (wired).
    Civilian,
    /// Vehicle placeholder.
    Vehicle,
    /// Airport / port placeholder.
    Airport,
}

impl SpawnKind {
    /// Wire / UI label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Civilian => "civilian",
            Self::Vehicle => "vehicle",
            Self::Airport => "airport",
        }
    }

    /// Whether server JSON-RPC / civ-watch spawn is implemented for this kind.
    pub const fn is_wired(self) -> bool {
        true
    }
}

/// Valid timelapse speed multipliers (matches civ-watch `/control/speed`).
pub const TIMELAPSE_SPEEDS: [u8; 4] = [1, 2, 4, 8];

/// Validate a timelapse speed multiplier (`0` = paused).
pub fn validate_timelapse_speed(speed: u8) -> Result<(), &'static str> {
    if speed == 0 || TIMELAPSE_SPEEDS.contains(&speed) {
        Ok(())
    } else {
        Err("speed must be 0, 1, 2, 4, or 8")
    }
}

/// Deterministic era timelapse view state derived from tick alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelapseView {
    /// Current simulation tick.
    pub tick: u64,
    /// Era index at this tick.
    pub era: u16,
}

impl TimelapseView {
    /// Snapshot of era view at a given tick.
    pub fn at_tick(tick: u64, era_length_ticks: u32) -> Self {
        Self {
            tick,
            era: era_at_tick(tick, era_length_ticks),
        }
    }

    /// Advance one wall-clock frame at `speed` (0 = paused).
    pub fn advance_frame(&mut self, speed: u8, era_length_ticks: u32) {
        if speed == 0 {
            return;
        }
        self.tick += u64::from(speed);
        self.era = era_at_tick(self.tick, era_length_ticks);
    }

    /// Advance until `target_tick`, one frame per loop iteration at `speed`.
    pub fn advance_to(&mut self, target_tick: u64, speed: u8, era_length_ticks: u32) {
        let frame_steps = u64::from(speed.max(1));
        while self.tick < target_tick {
            let remaining = target_tick - self.tick;
            self.tick += frame_steps.min(remaining);
            self.era = era_at_tick(self.tick, era_length_ticks);
        }
    }
}

fn era_at_tick(tick: u64, era_length_ticks: u32) -> u16 {
    let period = era_length_ticks.max(1) as u64;
    (tick / period) as u16
}
