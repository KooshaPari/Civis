//! Audio integration for the render layer (FR-AUD-001, FR-AUD-002, FR-AUD-003).
//!
//! This module provides the substrate types that sit between the engine's
//! simulation events and the Kira audio backend wired in the client
//! (`clients/bevy-ref/src/audio.rs`). The math here is pure and testable
//! without any audio engine dependency.
//!
//! ## FR-AUD-001 — Background music via Kira
//!
//! [`KiraMusic`] owns the high-level music state (tension, prosperity, war)
//! and exposes `set_*` mutators that the engine tick loop calls. The client
//! reads the resulting [`MusicState`] each frame and applies Kira tween
//! commands.
//!
//! ## FR-AUD-002 — Music layers fade based on state
//!
//! [`MusicLayers`] tracks per-layer gain targets and provides `fade_in` /
//! `fade_out` that compute the new gain after a duration. The client
//! interpolates toward these targets using Kira's `tween` API.
//!
//! ## FR-AUD-003 — SFX triggered by events
//!
//! [`SfxTriggerEvent`] wraps an event label + optional intensity and produces
//! a [`SfxPlaybackCommand`] that the client maps to a Kira one-shot handle.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// FR-AUD-001: KiraMusic
// ---------------------------------------------------------------------------

/// Continuous game-state signals that drive background music adaptation.
///
/// Each component is in `[0.0, 1.0]`. The client reads [`MusicState`] every
/// frame and maps it to Kira tween targets on the music stems.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MusicState {
    /// Active conflict intensity (0 = peace, 1 = total war).
    pub tension: f32,
    /// Aggregate economy health + population wellbeing (0 = collapse, 1 = golden age).
    pub prosperity: f32,
    /// Active war flag (0 = no war, 1 = at war). Distinct from `tension`
    /// because a cold war has high tension but low war footing.
    pub war: f32,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            tension: 0.0,
            prosperity: 0.5,
            war: 0.0,
        }
    }
}

impl MusicState {
    /// Returns `true` if every component is in `[0.0, 1.0]` and finite.
    pub fn is_well_formed(self) -> bool {
        [self.tension, self.prosperity, self.war]
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
    }
}

/// Kira-backed background music controller (FR-AUD-001).
///
/// The struct holds the authoritative music state and is mutated by the
/// engine tick loop via `set_*` methods. The client polls [`current_state`]
/// each frame and translates it into Kira tween commands.
///
/// No Kira dependency exists here — the client wires the actual audio
/// handles. This keeps the substrate testable with `--lib` only.
///
/// # Examples
///
/// ```
/// use civ_render::audio::KiraMusic;
///
/// let mut music = KiraMusic::init();
/// music.set_tension(0.8);
/// assert!(music.current_state().tension > 0.7);
/// ```
#[derive(Debug, Clone)]
pub struct KiraMusic {
    state: MusicState,
}

impl KiraMusic {
    /// Create a new `KiraMusic` in its default (peaceful) state.
    #[must_use]
    pub fn init() -> Self {
        Self {
            state: MusicState::default(),
        }
    }

    /// Set the tension level, clamped to `[0.0, 1.0]`.
    ///
    /// Higher tension drives the score toward dissonant stems (TensionStem
    /// in the audio substrate).
    pub fn set_tension(&mut self, value: f32) {
        self.state.tension = value.clamp(0.0, 1.0);
    }

    /// Set the prosperity level, clamped to `[0.0, 1.0]`.
    ///
    /// Higher prosperity warms the base stem and enables the Lead melody
    /// at high values.
    pub fn set_prosperity(&mut self, value: f32) {
        self.state.prosperity = value.clamp(0.0, 1.0);
    }

    /// Set the war flag, clamped to `[0.0, 1.0]`.
    ///
    /// When `war > 0`, the rhythm stem intensifies and a percussion
    /// layer is added. This is distinct from `tension` — a cold war
    /// has high tension but `war = 0`.
    pub fn set_war(&mut self, value: f32) {
        self.state.war = value.clamp(0.0, 1.0);
    }

    /// Read the current music state (client polls this each frame).
    #[must_use]
    pub fn current_state(&self) -> MusicState {
        self.state
    }
}

// ---------------------------------------------------------------------------
// FR-AUD-002: MusicLayers
// ---------------------------------------------------------------------------

/// Identifies a music layer in the adaptive score.
///
/// Each layer corresponds to a Kira audio handle that can be faded
/// independently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MusicLayer {
    /// Ambient / environmental bed.
    Ambient,
    /// Rhythmic percussion track.
    Rhythm,
    /// Tension / dissonance overlay.
    Tension,
    /// Lead melody.
    Lead,
    /// Bass / low-end foundation.
    Bass,
}

/// A single layer's fade state (FR-AUD-002).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayerState {
    /// Current gain in `[0.0, 1.0]`.
    pub gain: f32,
    /// Target gain the client should tween toward.
    pub target: f32,
    /// Fade duration in seconds.
    pub duration_secs: f32,
}

impl Default for LayerState {
    fn default() -> Self {
        Self {
            gain: 0.0,
            target: 0.0,
            duration_secs: 0.0,
        }
    }
}

/// Manages per-layer fade in/out for adaptive music (FR-AUD-002).
///
/// The engine calls `fade_in` / `fade_out` to set layer targets; the client
/// polls [`layer_state`] each frame and interpolates toward the target using
/// Kira's tween system.
///
/// # Examples
///
/// ```
/// use civ_render::audio::{MusicLayers, MusicLayer};
///
/// let mut layers = MusicLayers::new();
/// layers.fade_in(MusicLayer::Tension, 1.5);
/// let ts = layers.layer_state(MusicLayer::Tension);
/// assert!(ts.target > 0.0);
/// assert!((ts.duration_secs - 1.5).abs() < f32::EPSILON);
/// ```
#[derive(Debug, Clone)]
pub struct MusicLayers {
    ambient: LayerState,
    rhythm: LayerState,
    tension: LayerState,
    lead: LayerState,
    bass: LayerState,
}

impl Default for MusicLayers {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicLayers {
    /// Create a new `MusicLayers` with all layers at zero gain.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ambient: LayerState::default(),
            rhythm: LayerState::default(),
            tension: LayerState::default(),
            lead: LayerState::default(),
            bass: LayerState::default(),
        }
    }

    /// Fade a layer in to full gain over `duration_secs`.
    pub fn fade_in(&mut self, layer: MusicLayer, duration_secs: f32) {
        let state = self.mut_layer(layer);
        state.target = 1.0;
        state.duration_secs = duration_secs;
    }

    /// Fade a layer out to silence over `duration_secs`.
    pub fn fade_out(&mut self, layer: MusicLayer, duration_secs: f32) {
        let state = self.mut_layer(layer);
        state.target = 0.0;
        state.duration_secs = duration_secs;
    }

    /// Read the current state of a layer (client polls this per frame).
    #[must_use]
    pub fn layer_state(&self, layer: MusicLayer) -> LayerState {
        match layer {
            MusicLayer::Ambient => self.ambient,
            MusicLayer::Rhythm => self.rhythm,
            MusicLayer::Tension => self.tension,
            MusicLayer::Lead => self.lead,
            MusicLayer::Bass => self.bass,
        }
    }

    fn mut_layer(&mut self, layer: MusicLayer) -> &mut LayerState {
        match layer {
            MusicLayer::Ambient => &mut self.ambient,
            MusicLayer::Rhythm => &mut self.rhythm,
            MusicLayer::Tension => &mut self.tension,
            MusicLayer::Lead => &mut self.lead,
            MusicLayer::Bass => &mut self.bass,
        }
    }
}

// ---------------------------------------------------------------------------
// FR-AUD-003: SfxTriggerEvent
// ---------------------------------------------------------------------------

/// Identifies a category of reactive SFX event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SfxEventKind {
    /// War was declared.
    WarDeclared,
    /// A district collapsed.
    DistrictCollapsed,
    /// A building was completed.
    BuildingComplete,
    /// A technology was researched.
    TechResearched,
    /// A disaster struck.
    DisasterStruck,
    /// A population milestone was reached.
    PopulationMilestone,
    /// A trade route was established.
    TradeRouteEstablished,
    /// A treaty was signed.
    TreatySigned,
}

/// Playback command produced by [`SfxTriggerEvent::trigger_event`] (FR-AUD-003).
///
/// The client maps `event_kind` → audio handle and applies `volume` to the
/// Kira one-shot instance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SfxPlaybackCommand {
    /// Which event triggered this SFX.
    pub event_kind: SfxEventKind,
    /// Playback volume in `[0.0, 1.0]`.
    pub volume: f32,
}

/// Reactive SFX trigger that maps simulation events to playback commands
/// (FR-AUD-003).
///
/// The engine creates an `SfxTriggerEvent` when a notable event fires and
/// calls `trigger_event` to produce a [`SfxPlaybackCommand`] the client
/// feeds to the Kira one-shot pool.
///
/// # Examples
///
/// ```
/// use civ_render::audio::{SfxTriggerEvent, SfxEventKind};
///
/// let trigger = SfxTriggerEvent::new();
/// let cmd = trigger.trigger_event(SfxEventKind::WarDeclared, 0.9);
/// assert_eq!(cmd.event_kind, SfxEventKind::WarDeclared);
/// assert!((cmd.volume - 0.9).abs() < f32::EPSILON);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct SfxTriggerEvent {
    // Reserved for future state (e.g. cooldown tracking).
}

impl SfxTriggerEvent {
    /// Create a new trigger.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Convert a simulation event into a playback command.
    ///
    /// `intensity` is clamped to `[0.0, 1.0]` and used as the volume.
    #[must_use]
    pub fn trigger_event(&self, event_kind: SfxEventKind, intensity: f32) -> SfxPlaybackCommand {
        SfxPlaybackCommand {
            event_kind,
            volume: intensity.clamp(0.0, 1.0),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- FR-AUD-001: KiraMusic tests ---

    #[test]
    fn kira_music_init_creates_default_state() {
        let music = KiraMusic::init();
        let state = music.current_state();
        assert!(state.is_well_formed());
        assert!(state.tension.abs() < f32::EPSILON);
        assert!((state.prosperity - 0.5).abs() < f32::EPSILON);
        assert!(state.war.abs() < f32::EPSILON);
    }

    #[test]
    fn kira_music_set_tension_clamps() {
        let mut music = KiraMusic::init();
        music.set_tension(1.5);
        assert!((music.current_state().tension - 1.0).abs() < f32::EPSILON);
        music.set_tension(-0.3);
        assert!(music.current_state().tension.abs() < f32::EPSILON);
    }

    #[test]
    fn kira_music_set_prosperity_clamps() {
        let mut music = KiraMusic::init();
        music.set_prosperity(2.0);
        assert!((music.current_state().prosperity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn kira_music_set_war_clamps() {
        let mut music = KiraMusic::init();
        music.set_war(0.7);
        assert!((music.current_state().war - 0.7).abs() < f32::EPSILON);
        music.set_war(-0.1);
        assert!(music.current_state().war.abs() < f32::EPSILON);
    }

    // --- FR-AUD-002: MusicLayers tests ---

    #[test]
    fn music_layers_new_starts_all_silent() {
        let layers = MusicLayers::new();
        for &layer in &[
            MusicLayer::Ambient,
            MusicLayer::Rhythm,
            MusicLayer::Tension,
            MusicLayer::Lead,
            MusicLayer::Bass,
        ] {
            let s = layers.layer_state(layer);
            assert!(s.gain.abs() < f32::EPSILON);
            assert!(s.target.abs() < f32::EPSILON);
        }
    }

    #[test]
    fn music_layers_fade_in_sets_target_and_duration() {
        let mut layers = MusicLayers::new();
        layers.fade_in(MusicLayer::Tension, 2.0);
        let s = layers.layer_state(MusicLayer::Tension);
        assert!((s.target - 1.0).abs() < f32::EPSILON);
        assert!((s.duration_secs - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn music_layers_fade_out_sets_target_zero() {
        let mut layers = MusicLayers::new();
        layers.fade_in(MusicLayer::Lead, 1.0);
        layers.fade_out(MusicLayer::Lead, 0.5);
        let s = layers.layer_state(MusicLayer::Lead);
        assert!(s.target.abs() < f32::EPSILON);
        assert!((s.duration_secs - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn music_layers_fade_in_does_not_affect_other_layers() {
        let mut layers = MusicLayers::new();
        layers.fade_in(MusicLayer::Rhythm, 1.0);
        let ambient = layers.layer_state(MusicLayer::Ambient);
        assert!(ambient.target.abs() < f32::EPSILON);
    }

    // --- FR-AUD-003: SfxTriggerEvent tests ---

    #[test]
    fn sfx_trigger_event_produces_playback_command() {
        let trigger = SfxTriggerEvent::new();
        let cmd = trigger.trigger_event(SfxEventKind::WarDeclared, 0.9);
        assert_eq!(cmd.event_kind, SfxEventKind::WarDeclared);
        assert!((cmd.volume - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn sfx_trigger_event_clamps_intensity() {
        let trigger = SfxTriggerEvent::new();
        let cmd = trigger.trigger_event(SfxEventKind::DisasterStruck, 1.5);
        assert!((cmd.volume - 1.0).abs() < f32::EPSILON);

        let cmd = trigger.trigger_event(SfxEventKind::DistrictCollapsed, -0.5);
        assert!(cmd.volume.abs() < f32::EPSILON);
    }

    #[test]
    fn sfx_trigger_event_all_kinds_produce_commands() {
        let trigger = SfxTriggerEvent::new();
        let kinds = [
            SfxEventKind::WarDeclared,
            SfxEventKind::DistrictCollapsed,
            SfxEventKind::BuildingComplete,
            SfxEventKind::TechResearched,
            SfxEventKind::DisasterStruck,
            SfxEventKind::PopulationMilestone,
            SfxEventKind::TradeRouteEstablished,
            SfxEventKind::TreatySigned,
        ];
        for kind in kinds {
            let cmd = trigger.trigger_event(kind, 0.5);
            assert_eq!(cmd.event_kind, kind);
            assert!((cmd.volume - 0.5).abs() < f32::EPSILON);
        }
    }
}
