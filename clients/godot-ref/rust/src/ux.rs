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
    /// Vehicle spawn kind.
    Vehicle,
    /// Airport hub building.
    Airport,
    /// Harbor / trade port building.
    Port,
    /// Hangar / barracks building.
    Hangar,
}

impl SpawnKind {
    /// Wire / UI label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Civilian => "civilian",
            Self::Vehicle => "vehicle",
            Self::Airport => "airport",
            Self::Port => "port",
            Self::Hangar => "hangar",
        }
    }

    /// Whether server JSON-RPC / civ-watch spawn is implemented for this kind.
    pub const fn is_wired(self) -> bool {
        true
    }

    /// Whether FR-CIV-UX-004 expects drag-release placement (not click-only).
    pub const fn uses_drag_place(self) -> bool {
        matches!(
            self,
            Self::Vehicle | Self::Airport | Self::Port | Self::Hangar
        )
    }

    /// Whether a long drag spawns a convoy along the path (FR-CIV-UX-004).
    pub const fn uses_convoy_drag(self) -> bool {
        matches!(
            self,
            Self::Vehicle | Self::Airport | Self::Port | Self::Hangar
        )
    }
}

/// Minimum normalized drag distance (cells / grid_size) before drag-release placement fires.
pub const SPAWN_DRAG_MIN_NORM: f32 = 4.0 / 128.0;

/// Spacing between convoy spawns along a drag path (8 cells on a 128 grid).
pub const CONVOY_SPACING_NORM: f32 = 8.0 / 128.0;

/// Maximum entities placed by one drag gesture.
pub const CONVOY_MAX_SPAWNS: usize = 32;

/// True when drag from `start` to `end` exceeds the placement threshold.
pub fn spawn_drag_exceeds_threshold(start: (f32, f32), end: (f32, f32)) -> bool {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    (dx * dx + dy * dy).sqrt() >= SPAWN_DRAG_MIN_NORM
}

/// Sample normalized positions along a drag segment for convoy placement.
pub fn convoy_positions(start: (f32, f32), end: (f32, f32)) -> Vec<(f32, f32)> {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < SPAWN_DRAG_MIN_NORM {
        return vec![(
            end.0.clamp(0.0, 1.0),
            end.1.clamp(0.0, 1.0),
        )];
    }
    let steps = ((dist / CONVOY_SPACING_NORM).floor() as usize).min(CONVOY_MAX_SPAWNS.saturating_sub(1));
    if steps == 0 {
        return vec![(
            end.0.clamp(0.0, 1.0),
            end.1.clamp(0.0, 1.0),
        )];
    }
    (0..=steps)
        .map(|i| {
            let t = i as f32 / steps as f32;
            (
                (start.0 + dx * t).clamp(0.0, 1.0),
                (start.1 + dy * t).clamp(0.0, 1.0),
            )
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_kind_labels_and_flags_match_ui_expectations() {
        assert_eq!(SpawnKind::Civilian.label(), "civilian");
        assert_eq!(SpawnKind::Vehicle.label(), "vehicle");
        assert_eq!(SpawnKind::Airport.label(), "airport");
        assert_eq!(SpawnKind::Port.label(), "port");
        assert_eq!(SpawnKind::Hangar.label(), "hangar");

        for kind in [
            SpawnKind::Civilian,
            SpawnKind::Vehicle,
            SpawnKind::Airport,
            SpawnKind::Port,
            SpawnKind::Hangar,
        ] {
            assert!(kind.is_wired());
        }

        assert!(!SpawnKind::Civilian.uses_drag_place());
        assert!(!SpawnKind::Civilian.uses_convoy_drag());
        for kind in [
            SpawnKind::Vehicle,
            SpawnKind::Airport,
            SpawnKind::Port,
            SpawnKind::Hangar,
        ] {
            assert!(kind.uses_drag_place());
            assert!(kind.uses_convoy_drag());
        }
    }

    #[test]
    fn spawn_helpers_preserve_payload_and_sequence() {
        let body = spawn_civilian_body(1.25, 0.75, -3);
        assert_eq!(body.x, 1.25);
        assert_eq!(body.y, 0.75);
        assert_eq!(body.faction, -3);

        let event = entity_create_from_spawn(&body, 41);
        assert_eq!(event.entity_id, 41);
        assert_eq!(event.faction, u32::MAX - 2);

        let batch = spawn_batch_events(&[body.clone(), body], 100);
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].entity_id, 100);
        assert_eq!(batch[1].entity_id, 101);
    }

    #[test]
    fn drag_threshold_and_convoy_sampling_are_clamped() {
        assert!(!spawn_drag_exceeds_threshold((0.0, 0.0), (0.01, 0.01)));
        assert!(spawn_drag_exceeds_threshold((0.0, 0.0), (0.0, SPAWN_DRAG_MIN_NORM)));

        let short = convoy_positions((0.2, 0.2), (0.21, 0.22));
        assert_eq!(short, vec![(0.21, 0.22)]);

        let long = convoy_positions((0.1, 0.1), (0.9, 0.9));
        assert_eq!(long.first().copied(), Some((0.1, 0.1)));
        assert_eq!(long.last().copied(), Some((0.9, 0.9)));
        assert!(long.len() > 1);
        assert!(long.len() <= CONVOY_MAX_SPAWNS);
    }

    #[test]
    fn timelapse_validation_and_advancement_are_deterministic() {
        for speed in [0, 1, 2, 4, 8] {
            assert!(validate_timelapse_speed(speed).is_ok());
        }
        assert_eq!(
            validate_timelapse_speed(3),
            Err("speed must be 0, 1, 2, 4, or 8")
        );

        let mut view = TimelapseView::at_tick(9_999, DEFAULT_ERA_LENGTH_TICKS);
        assert_eq!(view.era, 1);
        view.advance_frame(0, DEFAULT_ERA_LENGTH_TICKS);
        assert_eq!(view.tick, 9_999);
        view.advance_frame(4, DEFAULT_ERA_LENGTH_TICKS);
        assert_eq!(view.tick, 10_003);
        assert_eq!(view.era, 2);

        let mut target = TimelapseView::at_tick(0, 1);
        target.advance_to(5, 2, 1);
        assert_eq!(target.tick, 5);
        assert_eq!(target.era, 5);
    }
}
