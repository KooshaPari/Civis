//! Audio placeholder engine for early development and testing.
//!
//! Provides a headless, dependency-free audio backend that logs every
//! [`SoundEvent`] instead of routing it to a real mixer. This lets UI,
//! gameplay, and scripting code be wired to the audio contract without
//! a Bevy / kira / wgpu dependency — the real engine replaces the
//! placeholder at integration time.
//!
//! ## Invariants
//!
//! - Every public function is pure and deterministic (no RNG, no
//!   `Instant::now`, no I/O). This makes the placeholder replayable
//!   and testable with `--lib` only.
//! - Channel limits are enforced on [`play`]; callers never need to
//!   check capacity themselves.
//! - Per-category volume settings are independent of per-event volume;
//!   the effective volume is `event.volume * category_volume(cat)`,
//!   computed by the engine consumer.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// SoundCategory
// ---------------------------------------------------------------------------

/// Broad classification of audio events.
///
/// Maps to the four-tier bus tree defined in `bus::BusId`, with the
/// addition of `Voice` for future dialogue / narration support.
/// Numeric tags are wire-stable — append only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundCategory {
    /// Adaptive score stems and one-shot music stings.
    Music,
    /// Environmental ambient beds (rain, wind, forest, city hum).
    Ambient,
    /// Reactive one-shot world events (birth, death, build, harvest).
    Effects,
    /// Interface clicks, hovers, confirms, alerts.
    UI,
    /// Future: dialogue lines, narrator narration.
    Voice,
}

impl SoundCategory {
    /// All five categories in display order.
    pub const ALL: [SoundCategory; 5] = [
        SoundCategory::Music,
        SoundCategory::Ambient,
        SoundCategory::Effects,
        SoundCategory::UI,
        SoundCategory::Voice,
    ];

    /// Human-readable label for display / debugging.
    pub fn label(self) -> &'static str {
        match self {
            SoundCategory::Music => "Music",
            SoundCategory::Ambient => "Ambient",
            SoundCategory::Effects => "Effects",
            SoundCategory::UI => "UI",
            SoundCategory::Voice => "Voice",
        }
    }
}

// ---------------------------------------------------------------------------
// SoundEvent
// ---------------------------------------------------------------------------

/// A single audio event that can be submitted to the placeholder engine.
///
/// The `name` field doubles as the channel key — two events sharing the
/// same name will share a channel, with the later event replacing the
/// earlier one. Names must be non-empty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoundEvent {
    /// Broad classification of this event.
    pub category: SoundCategory,
    /// Unique channel name (non-empty). Used as the channel key.
    pub name: String,
    /// Per-event linear volume in `[0.0, 1.0]`. Out-of-range values
    /// are clamped on construction by the convenience constructors.
    pub volume: f32,
    /// Playback pitch multiplier. `1.0` is normal speed; `0.5` is
    /// half-speed; `2.0` is double-speed. No hard range enforced at
    /// the substrate level — clamping is the consumer's responsibility.
    pub pitch: f32,
    /// Stereo pan in `[-1.0, 1.0]`. `-1.0` is hard left, `0.0` is
    /// center, `1.0` is hard right.
    pub pan: f32,
    /// When `true` the event loops until explicitly stopped via
    /// [`AudioPlaceholderEngine::stop`] or [`stop_all`].
    pub looping: bool,
    /// Duration in seconds. Non-looping events are automatically
    /// stopped after this many seconds of wall-clock time (advanced
    /// by [`AudioPlaceholderEngine::tick`]). Looping events ignore
    /// this field and run indefinitely.
    pub duration: f32,
}

impl SoundEvent {
    /// Returns `true` if every numeric field is finite and within
    /// its expected range.
    pub fn is_well_formed(&self) -> bool {
        self.name.is_empty().not()
            && self.volume.is_finite()
            && (0.0..=1.0).contains(&self.volume)
            && self.pitch.is_finite()
            && self.pan.is_finite()
            && (-1.0..=1.0).contains(&self.pan)
            && self.duration.is_finite()
            && self.duration >= 0.0
    }
}

// Trait helper (used once; inlined at call site).
trait BoolExt {
    fn not(self) -> bool;
}
impl BoolExt for bool {
    fn not(self) -> bool {
        !self
    }
}

// ---------------------------------------------------------------------------
// AudioError
// ---------------------------------------------------------------------------

/// Errors returned by [`AudioPlaceholderEngine::play`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioError {
    /// All channel slots are occupied by active (non-completed) events.
    ChannelFull,
    /// No active channel matched the requested name.
    NotFound,
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::ChannelFull => write!(f, "all audio channels are full"),
            AudioError::NotFound => write!(f, "no active channel with that name"),
        }
    }
}

impl std::error::Error for AudioError {}

// ---------------------------------------------------------------------------
// AudioPlaceholderEngine
// ---------------------------------------------------------------------------

/// Headless audio engine that records events in a log and tracks active
/// channels without touching any hardware or audio framework.
///
/// The engine is **not** `Send`/`Sync` by default (it owns `Vec` and
/// `HashMap`); the consumer is expected to drive it on a single thread
/// (typically the Bevy main-thread tick).
pub struct AudioPlaceholderEngine {
    /// Append-only log of every event ever submitted to [`play`].
    /// The log is never pruned — the consumer is responsible for
    /// rotating or truncating it if it grows too large.
    pub events_log: Vec<SoundEvent>,
    /// Currently active (non-completed) channels, keyed by
    /// [`SoundEvent::name`]. Playing a new event with the same name
    /// replaces the existing entry.
    pub active_channels: HashMap<String, SoundEvent>,
    /// Maximum number of concurrent active channels.
    pub max_channels: u32,
    /// Per-category linear volume multipliers in `[0.0, 1.0]`.
    /// Applied on top of per-event volume by the engine consumer.
    category_volumes: HashMap<SoundCategory, f32>,
    /// Elapsed wall-clock time per active channel (seconds).
    /// Used by [`tick`] to auto-stop non-looping events.
    channels_elapsed: HashMap<String, f32>,
}

impl AudioPlaceholderEngine {
    /// Default per-category volume. All categories start at `1.0`
    /// (unity — no attenuation).
    const DEFAULT_CATEGORY_VOLUME: f32 = 1.0;

    /// Create a new placeholder engine with the given channel limit.
    ///
    /// Pass `0` for an engine that rejects all events (useful for
    /// testing the `ChannelFull` error path).
    pub fn new(max_channels: u32) -> Self {
        let category_volumes = SoundCategory::ALL
            .iter()
            .copied()
            .map(|c| (c, Self::DEFAULT_CATEGORY_VOLUME))
            .collect();

        Self {
            events_log: Vec::new(),
            active_channels: HashMap::new(),
            max_channels,
            category_volumes,
            channels_elapsed: HashMap::new(),
        }
    }

    /// Submit a sound event for playback.
    ///
    /// If the event's name matches an existing active channel, the old
    /// entry is replaced and the elapsed timer resets. Returns
    /// [`AudioError::ChannelFull`] if all slots are occupied and the
    /// name is not already active.
    pub fn play(&mut self, event: SoundEvent) -> Result<(), AudioError> {
        self.events_log.push(event.clone());

        let name = event.name.clone();

        // If the name is already active, we can always replace it
        // (does not consume an additional slot).
        if self.active_channels.contains_key(&name) {
            self.active_channels.insert(name.clone(), event);
            self.channels_elapsed.insert(name, 0.0);
            return Ok(());
        }

        // New name — check capacity.
        if self.active_channels.len() as u32 >= self.max_channels {
            return Err(AudioError::ChannelFull);
        }

        self.channels_elapsed.insert(name.clone(), 0.0);
        self.active_channels.insert(name, event);
        Ok(())
    }

    /// Stop the active channel with the given name.
    ///
    /// Returns `Err(NotFound)` if no active channel matches.
    pub fn stop(&mut self, name: &str) -> Result<(), AudioError> {
        match self.active_channels.remove(name) {
            Some(_) => {
                self.channels_elapsed.remove(name);
                Ok(())
            }
            None => Err(AudioError::NotFound),
        }
    }

    /// Stop all active channels and reset elapsed timers.
    pub fn stop_all(&mut self) {
        self.active_channels.clear();
        self.channels_elapsed.clear();
    }

    /// Advance the engine clock by `dt` seconds.
    ///
    /// Non-looping events whose cumulative elapsed time meets or
    /// exceeds their [`SoundEvent::duration`] are automatically
    /// removed from the active set. Looping events are unaffected.
    pub fn tick(&mut self, dt: f32) {
        let dt = dt.max(0.0);
        let mut to_remove: Vec<String> = Vec::new();

        for (_name, elapsed) in &mut self.channels_elapsed {
            *elapsed += dt;
        }

        for (name, event) in &self.active_channels {
            if !event.looping {
                let elapsed = self.channels_elapsed.get(name).copied().unwrap_or(0.0);
                if elapsed >= event.duration {
                    to_remove.push(name.clone());
                }
            }
        }

        for name in to_remove {
            self.active_channels.remove(&name);
            self.channels_elapsed.remove(&name);
        }
    }

    /// Immutable view of all currently active channels.
    pub fn get_active(&self) -> Vec<&SoundEvent> {
        self.active_channels.values().collect()
    }

    /// Immutable view of the full event log (oldest first).
    pub fn get_log(&self) -> Vec<&SoundEvent> {
        self.events_log.iter().collect()
    }

    /// Per-category linear volume multiplier.
    pub fn category_volume(&self, cat: SoundCategory) -> f32 {
        self.category_volumes
            .get(&cat)
            .copied()
            .unwrap_or(Self::DEFAULT_CATEGORY_VOLUME)
    }

    /// Set the per-category volume, clamped to `[0.0, 1.0]`.
    /// Returns the post-clamp value actually written.
    pub fn set_category_volume(&mut self, cat: SoundCategory, vol: f32) -> f32 {
        let clamped = vol.clamp(0.0, 1.0);
        self.category_volumes.insert(cat, clamped);
        clamped
    }
}

// ---------------------------------------------------------------------------
// Convenience constructors
// ---------------------------------------------------------------------------

/// Create a music-track event with sensible defaults (volume 0.7,
/// pitch 1.0, center pan, looping, 30 s duration).
pub fn music_track(name: &str) -> SoundEvent {
    SoundEvent {
        category: SoundCategory::Music,
        name: name.to_owned(),
        volume: 0.7,
        pitch: 1.0,
        pan: 0.0,
        looping: true,
        duration: 30.0,
    }
}

/// Create an ambient-bed event (volume 0.5, pitch 1.0, center pan,
/// looping, 60 s duration).
pub fn ambient(name: &str) -> SoundEvent {
    SoundEvent {
        category: SoundCategory::Ambient,
        name: name.to_owned(),
        volume: 0.5,
        pitch: 1.0,
        pan: 0.0,
        looping: true,
        duration: 60.0,
    }
}

/// Create a one-shot sound effect (volume 0.8, pitch 1.0, center pan,
/// non-looping, 2 s duration).
pub fn sfx(name: &str) -> SoundEvent {
    SoundEvent {
        category: SoundCategory::Effects,
        name: name.to_owned(),
        volume: 0.8,
        pitch: 1.0,
        pan: 0.0,
        looping: false,
        duration: 2.0,
    }
}

/// Create a UI-click event (volume 0.4, pitch 1.0, center pan,
/// non-looping, 0.5 s duration).
pub fn ui_click() -> SoundEvent {
    SoundEvent {
        category: SoundCategory::UI,
        name: "ui_click".to_owned(),
        volume: 0.4,
        pitch: 1.0,
        pan: 0.0,
        looping: false,
        duration: 0.5,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- play / stop basics ------------------------------------------------

    #[test]
    fn play_places_event_in_active_channels() {
        let mut eng = AudioPlaceholderEngine::new(4);
        let evt = sfx("hit");
        assert!(eng.play(evt.clone()).is_ok());
        assert_eq!(eng.active_channels.len(), 1);
        assert_eq!(eng.active_channels.get("hit"), Some(&evt));
    }

    #[test]
    fn stop_removes_named_channel() {
        let mut eng = AudioPlaceholderEngine::new(4);
        eng.play(sfx("hit")).unwrap();
        assert!(eng.stop("hit").is_ok());
        assert!(eng.active_channels.is_empty());
    }

    #[test]
    fn stop_returns_not_found_for_unknown_name() {
        let mut eng = AudioPlaceholderEngine::new(4);
        assert_eq!(eng.stop("nope"), Err(AudioError::NotFound));
    }

    // -- channel limits ----------------------------------------------------

    #[test]
    fn play_returns_channel_full_when_at_capacity() {
        let mut eng = AudioPlaceholderEngine::new(2);
        eng.play(sfx("a")).unwrap();
        eng.play(sfx("b")).unwrap();
        assert_eq!(eng.play(sfx("c")), Err(AudioError::ChannelFull));
    }

    #[test]
    fn replace_same_name_does_not_consume_extra_slot() {
        let mut eng = AudioPlaceholderEngine::new(1);
        eng.play(sfx("a")).unwrap();
        // Replacing the same name should succeed even though we're at capacity.
        assert!(eng.play(sfx("a")).is_ok());
        assert_eq!(eng.active_channels.len(), 1);
    }

    // -- logging -----------------------------------------------------------

    #[test]
    fn every_play_is_logged_in_order() {
        let mut eng = AudioPlaceholderEngine::new(8);
        eng.play(sfx("a")).unwrap();
        eng.play(sfx("b")).unwrap();
        let log = eng.get_log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].name, "a");
        assert_eq!(log[1].name, "b");
    }

    // -- categories --------------------------------------------------------

    #[test]
    fn category_volume_starts_at_unity() {
        let eng = AudioPlaceholderEngine::new(4);
        for cat in SoundCategory::ALL {
            assert!((eng.category_volume(cat) - 1.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn set_category_volume_clamps_and_persists() {
        let mut eng = AudioPlaceholderEngine::new(4);
        let written = eng.set_category_volume(SoundCategory::Effects, 2.5);
        assert!((written - 1.0).abs() < f32::EPSILON);
        assert!((eng.category_volume(SoundCategory::Effects) - 1.0).abs() < f32::EPSILON);
        let written = eng.set_category_volume(SoundCategory::Music, -0.3);
        assert!(written.abs() < f32::EPSILON);
    }

    // -- convenience constructors ------------------------------------------

    #[test]
    fn music_track_returns_looping_music_event() {
        let evt = music_track("orchestra");
        assert_eq!(evt.category, SoundCategory::Music);
        assert!(evt.looping);
        assert_eq!(evt.name, "orchestra");
    }

    #[test]
    fn sfx_returns_non_looping_effect_event() {
        let evt = sfx("explosion");
        assert_eq!(evt.category, SoundCategory::Effects);
        assert!(!evt.looping);
        assert_eq!(evt.name, "explosion");
    }

    #[test]
    fn ui_click_returns_non_looping_ui_event() {
        let evt = ui_click();
        assert_eq!(evt.category, SoundCategory::UI);
        assert!(!evt.looping);
        assert_eq!(evt.name, "ui_click");
    }

    // -- tick / auto-stop --------------------------------------------------

    #[test]
    fn tick_removes_expired_non_looping_events() {
        let mut eng = AudioPlaceholderEngine::new(4);
        eng.play(SoundEvent {
            category: SoundCategory::Effects,
            name: "short".to_owned(),
            volume: 0.8,
            pitch: 1.0,
            pan: 0.0,
            looping: false,
            duration: 1.0,
        })
        .unwrap();

        eng.tick(0.5);
        assert!(eng.active_channels.contains_key("short"));

        eng.tick(0.6); // elapsed = 1.1 >= duration 1.0
        assert!(!eng.active_channels.contains_key("short"));
    }

    #[test]
    fn tick_does_not_remove_looping_events() {
        let mut eng = AudioPlaceholderEngine::new(4);
        eng.play(ambient("rain")).unwrap();
        eng.tick(1000.0);
        assert!(eng.active_channels.contains_key("rain"));
    }

    #[test]
    fn play_replaces_active_event_and_resets_elapsed() {
        let mut eng = AudioPlaceholderEngine::new(4);
        eng.play(SoundEvent {
            category: SoundCategory::Effects,
            name: "hit".to_owned(),
            volume: 0.5,
            pitch: 1.0,
            pan: 0.0,
            looping: false,
            duration: 1.0,
        })
        .unwrap();
        eng.tick(0.9);
        // Replace with same name — elapsed resets.
        eng.play(SoundEvent {
            category: SoundCategory::Effects,
            name: "hit".to_owned(),
            volume: 0.8,
            pitch: 1.5,
            pan: 0.0,
            looping: false,
            duration: 1.0,
        })
        .unwrap();
        eng.tick(0.9);
        // Should still be active because elapsed was reset to 0.9.
        assert!(eng.active_channels.contains_key("hit"));
    }

    // -- well-formed checks ------------------------------------------------

    #[test]
    fn sound_event_well_formed_for_valid_values() {
        let evt = SoundEvent {
            category: SoundCategory::Music,
            name: "track".to_owned(),
            volume: 0.5,
            pitch: 1.0,
            pan: -0.5,
            looping: true,
            duration: 10.0,
        };
        assert!(evt.is_well_formed());
    }

    #[test]
    fn sound_event_not_well_formed_for_empty_name() {
        let evt = SoundEvent {
            category: SoundCategory::Music,
            name: String::new(),
            volume: 0.5,
            pitch: 1.0,
            pan: 0.0,
            looping: false,
            duration: 5.0,
        };
        assert!(!evt.is_well_formed());
    }

    #[test]
    fn sound_event_not_well_formed_for_out_of_range_volume() {
        let evt = SoundEvent {
            category: SoundCategory::Effects,
            name: "boom".to_owned(),
            volume: 1.5,
            pitch: 1.0,
            pan: 0.0,
            looping: false,
            duration: 1.0,
        };
        assert!(!evt.is_well_formed());
    }

    // -- stop_all ----------------------------------------------------------

    #[test]
    fn stop_all_clears_everything() {
        let mut eng = AudioPlaceholderEngine::new(4);
        eng.play(sfx("a")).unwrap();
        eng.play(ambient("b")).unwrap();
        eng.stop_all();
        assert!(eng.active_channels.is_empty());
    }
}
