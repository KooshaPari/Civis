//! Ambient soundscape + SFX system for the Civis Bevy client.
//!
//! Feature-gated behind the `audio` cargo feature (which implies `bevy`).

#![cfg(feature = "audio")]

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

// -- Channels ----------------------------------------------------------------

/// Dedicated channel for the looping ambient bed (mute / duck independently).
#[derive(Resource)]
pub struct AmbientChannel;

/// Dedicated channel for one-shot SFX (UI + simulation events).
#[derive(Resource)]
pub struct SfxChannel;

// -- Event kinds -------------------------------------------------------------

/// The catalogue of one-shot sound effects the client can trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SfxKind {
    /// UI button / menu click feedback.
    UiClick,
    /// An agent was born.
    Birth,
    /// An agent died.
    Death,
    /// A disaster fired (fire / flood / quake).
    Disaster,
    /// A building was constructed.
    Build,
    /// A diplomatic act fired.
    Diplomatic,
    /// A technology was unlocked.
    Tech,
}

/// Bevy message other systems write to trigger a one-shot SFX.
#[derive(Message, Debug, Clone, Copy)]
pub struct SfxEvent {
    /// Which catalogue entry to play.
    pub kind: SfxKind,
    /// Linear volume multiplier (`1.0` = unmodified clip gain).
    pub volume: f32,
}

impl SfxEvent {
    /// A unit-volume SFX trigger for `kind`.
    #[must_use]
    pub fn new(kind: SfxKind) -> Self {
        Self { kind, volume: 1.0 }
    }

    /// A SFX trigger for `kind` at an explicit linear `volume`.
    #[must_use]
    pub fn with_volume(kind: SfxKind, volume: f32) -> Self {
        Self { kind, volume }
    }
}

/// Ergonomic one-liner the rest of the client uses to fire a sound effect.
pub fn play_sfx(writer: &mut MessageWriter<SfxEvent>, kind: SfxKind) {
    writer.write(SfxEvent::new(kind));
}

// -- Snapshot-driven triggers (FR-AUDIO-wire) --------------------------------
//
// `civ-server`'s `sim.snapshot` JSON-RPC response carries an
// `audio_events: Vec<SfxTrigger>` field (see `civ_engine::phase_audio`).
// `parse_audio_events_from_snapshot_json` decodes that list and routes
// each entry into a Bevy `SfxEvent` so the existing `drain_sfx_events`
// system plays the right clip without needing a separate audio
// connection.

/// Audio trigger as serialized on the `sim.snapshot.audio_events`
/// JSON-RPC field (FR-AUDIO-wire). Mirrors the engine's
/// `civ_engine::SfxTrigger` enum (`Battle` / `Build` / `Disaster`).
#[derive(Debug, Clone, PartialEq)]
pub enum SnapshotAudioTrigger {
    /// Combat pulse fired this tick.
    Battle {
        /// Linear intensity in `[0, 1]` (closer battles are louder).
        intensity: f32,
    },
    /// Construction completed this tick.
    Build,
    /// Disaster fired this tick.
    Disaster {
        /// Lower-case disaster label (`meteor`, `flood`, `quake`,
        /// `wildfire`, `storm`, `plague`, or `disaster` for unknown
        /// kinds — the umbrella fallback matches the engine).
        kind: String,
        /// Severity in `[0, 1]`; the server clamps out-of-range.
        severity: f32,
    },
}

impl SnapshotAudioTrigger {
    /// Resolve this trigger to a Bevy-side [`SfxKind`] so the audio
    /// plugin can pick a clip. Unknown disaster labels and battle
    /// pulses route to the umbrella `SfxKind::Disaster` sting
    /// (matches the server-side fallback in
    /// `civ_engine::SfxTrigger::Disaster`).
    #[must_use]
    pub fn sfx_kind(&self) -> SfxKind {
        match self {
            Self::Battle { .. } => SfxKind::Disaster,
            Self::Build => SfxKind::Build,
            Self::Disaster { kind, .. } => match kind.as_str() {
                "meteor" | "flood" | "quake" | "wildfire" | "storm" | "plague" => {
                    SfxKind::Disaster
                }
                _ => SfxKind::Disaster,
            },
        }
    }

    /// Linear volume for this trigger. Battle intensity scales the
    /// disaster sting so closer battles are louder; disaster severity
    /// passes through (already `[0, 1]`); build fires at unit gain.
    #[must_use]
    pub fn volume(&self) -> f32 {
        match self {
            Self::Battle { intensity } => intensity.clamp(0.0, 1.0),
            Self::Build => 1.0,
            Self::Disaster { severity, .. } => severity.clamp(0.0, 1.0),
        }
    }

    /// Convert this snapshot trigger into a Bevy [`SfxEvent`] for the
    /// audio plugin's message channel.
    #[must_use]
    pub fn into_sfx_event(self) -> SfxEvent {
        let volume = self.volume();
        let kind = self.sfx_kind();
        SfxEvent::with_volume(kind, volume)
    }
}

/// Parse the `audio_events` list from a `sim.snapshot` JSON-RPC
/// response text (FR-AUDIO-wire). Returns an empty `Vec` when the
/// field is absent, when the snapshot is malformed, or when the tick
/// is quiet — all three are the wire-level expression of "no audio
/// this tick". The helper never panics: malformed trigger entries are
/// skipped and logged via the returned counts (used by tests).
#[must_use]
pub fn parse_audio_events_from_snapshot_json(text: &str) -> Vec<SnapshotAudioTrigger> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return Vec::new();
    };
    let Some(arr) = value
        .get("result")
        .and_then(|r| r.get("audio_events"))
        .and_then(|v| v.as_array())
    else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(arr.len());
    for entry in arr {
        if let Some(trigger) = parse_one_audio_event(entry) {
            out.push(trigger);
        }
    }
    out
}

fn parse_one_audio_event(value: &serde_json::Value) -> Option<SnapshotAudioTrigger> {
    let obj = value.as_object()?;
    // The serialized `civ_engine::SfxTrigger` is an externally-tagged
    // enum, so each variant lives under its own key. Probe each key
    // independently — the first matching key wins, mirroring serde's
    // default behaviour. Returns `None` when none of the keys match
    // so the caller can skip a malformed entry without panicking.
    if let Some(battle) = obj.get("Battle") {
        let intensity = battle.get("intensity")?.as_f64()? as f32;
        return Some(SnapshotAudioTrigger::Battle { intensity });
    }
    if obj.contains_key("Build") {
        return Some(SnapshotAudioTrigger::Build);
    }
    if let Some(disaster) = obj.get("Disaster") {
        let kind = disaster.get("kind")?.as_str()?.to_string();
        let severity = disaster.get("severity")?.as_f64()? as f32;
        return Some(SnapshotAudioTrigger::Disaster { kind, severity });
    }
    None
}

/// Convert a batch of snapshot triggers into Bevy [`SfxEvent`]s and
/// write them through `writer`. Returns the number of events emitted
/// so callers can log "played N audio cues" without re-counting.
pub fn forward_snapshot_audio_to_sfx<W: MessageWriter<SfxEvent>>(
    writer: &mut W,
    triggers: Vec<SnapshotAudioTrigger>,
) -> usize {
    let mut count = 0usize;
    for trigger in triggers {
        writer.write(trigger.into_sfx_event());
        count += 1;
    }
    count
}

// -- Config + handles --------------------------------------------------------

/// Where each clip is loaded from under `assets/`.
#[derive(Resource, Debug, Clone)]
pub struct AudioFiles {
    pub ambient: String,
    pub ui_click: String,
    pub birth: String,
    pub death: String,
    pub disaster: String,
    pub build: String,
    pub diplomatic: String,
    pub tech: String,
}

impl Default for AudioFiles {
    fn default() -> Self {
        Self {
            ambient: "audio/ambient_wind.ogg".to_string(),
            ui_click: "audio/ui_click.ogg".to_string(),
            birth: "audio/birth.ogg".to_string(),
            death: "audio/death.ogg".to_string(),
            disaster: "audio/sfx_disaster.ogg".to_string(),
            build: "audio/build.ogg".to_string(),
            diplomatic: "audio/sfx_diplomatic.ogg".to_string(),
            tech: "audio/sfx_tech.ogg".to_string(),
        }
    }
}

/// Loaded clip handles, populated at startup from [`AudioFiles`].
#[derive(Resource, Default)]
pub struct AudioHandles {
    pub ambient: Handle<bevy_kira_audio::AudioSource>,
    pub ui_click: Handle<bevy_kira_audio::AudioSource>,
    pub birth: Handle<bevy_kira_audio::AudioSource>,
    pub death: Handle<bevy_kira_audio::AudioSource>,
    pub disaster: Handle<bevy_kira_audio::AudioSource>,
    pub build: Handle<bevy_kira_audio::AudioSource>,
    pub diplomatic: Handle<bevy_kira_audio::AudioSource>,
    pub tech: Handle<bevy_kira_audio::AudioSource>,
}

impl AudioHandles {
    /// Resolve a [`SfxKind`] to its loaded clip handle.
    #[must_use]
    pub fn for_kind(&self, kind: SfxKind) -> Handle<bevy_kira_audio::AudioSource> {
        match kind {
            SfxKind::UiClick => self.ui_click.clone(),
            SfxKind::Birth => self.birth.clone(),
            SfxKind::Death => self.death.clone(),
            SfxKind::Disaster => self.disaster.clone(),
            SfxKind::Build => self.build.clone(),
            SfxKind::Diplomatic => self.diplomatic.clone(),
            SfxKind::Tech => self.tech.clone(),
        }
    }
}

/// Startup ambient-bed volume (linear).
pub const AMBIENT_VOLUME: f32 = 0.35;

/// Runtime audio state -- mute toggle and per-channel volume.
#[derive(Resource)]
pub struct AudioState {
    pub ambient_volume: f32,
    pub sfx_volume: f32,
    pub muted: bool,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            ambient_volume: AMBIENT_VOLUME,
            sfx_volume: 0.7,
            muted: false,
        }
    }
}

// -- Plugin ------------------------------------------------------------------

/// Ambient soundscape + SFX plugin for the Civis Bevy client.
#[derive(Default)]
pub struct CivisAudioPlugin;

impl Plugin for CivisAudioPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_kira_audio::AudioPlugin>() {
            app.add_plugins(bevy_kira_audio::AudioPlugin);
        }
        app.add_audio_channel::<AmbientChannel>()
            .add_audio_channel::<SfxChannel>()
            .init_resource::<AudioFiles>()
            .init_resource::<AudioHandles>()
            .init_resource::<AudioState>()
            .add_message::<SfxEvent>()
            .add_systems(Startup, (load_audio, start_ambient).chain())
            .add_systems(Update, (drain_sfx_events, toggle_mute));
    }
}

fn load_audio(
    asset_server: Res<AssetServer>,
    files: Res<AudioFiles>,
    mut handles: ResMut<AudioHandles>,
) {
    handles.ambient = asset_server.load(files.ambient.clone());
    handles.ui_click = asset_server.load(files.ui_click.clone());
    handles.birth = asset_server.load(files.birth.clone());
    handles.death = asset_server.load(files.death.clone());
    handles.disaster = asset_server.load(files.disaster.clone());
    handles.build = asset_server.load(files.build.clone());
    handles.diplomatic = asset_server.load(files.diplomatic.clone());
    handles.tech = asset_server.load(files.tech.clone());
}

fn start_ambient(channel: Res<AudioChannel<AmbientChannel>>, handles: Res<AudioHandles>) {
    channel
        .play(handles.ambient.clone())
        .looped()
        .with_volume(AMBIENT_VOLUME);
}

fn drain_sfx_events(
    mut events: MessageReader<SfxEvent>,
    channel: Res<AudioChannel<SfxChannel>>,
    handles: Res<AudioHandles>,
) {
    for event in events.read() {
        channel
            .play(handles.for_kind(event.kind))
            .with_volume(event.volume);
    }
}

/// Toggle mute on/off with the K key; updates both channel volumes.
fn toggle_mute(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<AudioState>,
    ambient: Res<AudioChannel<AmbientChannel>>,
    sfx: Res<AudioChannel<SfxChannel>>,
) {
    if !keys.just_pressed(KeyCode::KeyK) {
        return;
    }
    state.muted = !state.muted;
    let ambient_vol = if state.muted { 0.0 } else { state.ambient_volume };
    let sfx_vol = if state.muted { 0.0 } else { state.sfx_volume };
    ambient.set_volume(ambient_vol);
    sfx.set_volume(sfx_vol);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sfx_event_defaults_to_unit_volume() {
        let event = SfxEvent::new(SfxKind::Birth);
        assert_eq!(event.kind, SfxKind::Birth);
        assert!((event.volume - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn sfx_event_with_volume_preserves_kind() {
        let event = SfxEvent::with_volume(SfxKind::Disaster, 0.5);
        assert_eq!(event.kind, SfxKind::Disaster);
        assert!((event.volume - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn audio_files_default_paths_live_under_audio_dir() {
        let files = AudioFiles::default();
        assert!(files.ambient.starts_with("audio/"));
        assert!(files.ui_click.ends_with(".ogg"));
        assert!(files.diplomatic.starts_with("audio/"));
        assert!(files.tech.starts_with("audio/"));
    }

    #[test]
    fn for_kind_maps_each_variant_to_a_handle_slot() {
        let handles = AudioHandles::default();
        for kind in [
            SfxKind::UiClick,
            SfxKind::Birth,
            SfxKind::Death,
            SfxKind::Disaster,
            SfxKind::Build,
            SfxKind::Diplomatic,
            SfxKind::Tech,
        ] {
            let _ = handles.for_kind(kind);
        }
    }

    #[test]
    fn audio_state_default_is_unmuted() {
        let state = AudioState::default();
        assert!(!state.muted);
        assert!((state.ambient_volume - AMBIENT_VOLUME).abs() < f32::EPSILON);
        assert!(state.sfx_volume > 0.0);
    }
}
