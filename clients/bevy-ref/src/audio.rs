//! Ambient soundscape + SFX system for the Civis Bevy client.
//!
//! Feature-gated behind the `audio` cargo feature (which implies `bevy`).

#![cfg(feature = "audio")]

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::MusicCues;

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
            .init_resource::<MusicCues>()
            .add_message::<SfxEvent>()
            .add_systems(Startup, (load_audio, start_ambient).chain())
            .add_systems(Update, (drain_sfx_events, toggle_mute))
            .add_systems(Update, sync_music_cue_volume.after(toggle_mute));
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
    cues: Res<MusicCues>,
    ambient: Res<AudioChannel<AmbientChannel>>,
    sfx: Res<AudioChannel<SfxChannel>>,
) {
    if !keys.just_pressed(KeyCode::KeyK) {
        return;
    }
    state.muted = !state.muted;
    let ambient_vol = if state.muted {
        0.0
    } else {
        ambient_volume_for_music_cues(&cues)
    };
    let sfx_vol = if state.muted { 0.0 } else { state.sfx_volume };
    ambient.set_volume(ambient_vol);
    ambient.set_playback_rate(if state.muted {
        1.0
    } else {
        ambient_playback_rate_for_music_cues(&cues) as f64
    });
    sfx.set_volume(sfx_vol);
}

/// Reference BPM for the ambient bed clip (neutral presentation tempo).
pub const AMBIENT_REFERENCE_BPM: f32 = 100.0;

/// Clamp for ambient playback-rate mapping from cue tempo.
pub const AMBIENT_PLAYBACK_RATE_MIN: f32 = 0.85;
pub const AMBIENT_PLAYBACK_RATE_MAX: f32 = 1.25;

/// Mood-dependent multiplier for the single ambient bed until mood-specific stems exist.
#[must_use]
pub fn ambient_mood_gain(mood: &str) -> f32 {
    match mood.trim().to_ascii_lowercase().as_str() {
        "calm" | "peaceful" | "serene" => 0.55,
        "neutral" | "settled" => 0.75,
        "tense" | "unrest" => 0.9,
        "danger" | "war" | "crisis" => 1.0,
        _ => 0.75,
    }
}

/// Playback-rate multiplier from a cue tempo (BPM), relative to [`AMBIENT_REFERENCE_BPM`].
///
/// Missing tempo keeps rate `1.0`. Values are clamped so the single ambient bed
/// stays musical rather than chipmunked.
#[must_use]
pub fn ambient_playback_rate_for_tempo_bpm(tempo_bpm: Option<u16>) -> f32 {
    let Some(bpm) = tempo_bpm.filter(|&b| b > 0) else {
        return 1.0;
    };
    (f32::from(bpm) / AMBIENT_REFERENCE_BPM)
        .clamp(AMBIENT_PLAYBACK_RATE_MIN, AMBIENT_PLAYBACK_RATE_MAX)
}

/// Ambient-bed volume implied by all current cluster cues.
///
/// Until the client ships separate stems, the strongest mood-adjusted cue drives
/// the existing ambient channel. An empty cue map preserves the normal ambient bed.
#[must_use]
pub fn ambient_volume_for_music_cues(cues: &MusicCues) -> f32 {
    let cue_gain = cues
        .0
        .values()
        .map(|cue| cue.intensity.clamp(0.0, 1.0) * ambient_mood_gain(&cue.mood))
        .fold(0.0_f32, f32::max);
    if cues.0.is_empty() {
        AMBIENT_VOLUME
    } else {
        AMBIENT_VOLUME * cue_gain
    }
}

/// Ambient playback rate from the dominant (loudest mood-adjusted) cue's tempo.
#[must_use]
pub fn ambient_playback_rate_for_music_cues(cues: &MusicCues) -> f32 {
    let dominant = cues.0.values().max_by(|a, b| {
        let ga = a.intensity.clamp(0.0, 1.0) * ambient_mood_gain(&a.mood);
        let gb = b.intensity.clamp(0.0, 1.0) * ambient_mood_gain(&b.mood);
        ga.partial_cmp(&gb).unwrap_or(std::cmp::Ordering::Equal)
    });
    match dominant {
        Some(cue) => ambient_playback_rate_for_tempo_bpm(cue.tempo_bpm),
        None => 1.0,
    }
}

/// Apply the strongest snapshot cue to the ambient channel's volume + tempo.
fn sync_music_cue_volume(
    cues: Res<MusicCues>,
    state: Res<AudioState>,
    ambient: Res<AudioChannel<AmbientChannel>>,
) {
    if !cues.is_changed() {
        return;
    }
    let volume = if state.muted {
        0.0
    } else {
        ambient_volume_for_music_cues(&cues)
    };
    ambient.set_volume(volume as f64);
    ambient.set_playback_rate(ambient_playback_rate_for_music_cues(&cues) as f64);
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

    #[test]
    fn music_cues_use_the_loudest_mood_adjusted_cluster() {
        let cues = MusicCues(std::collections::BTreeMap::from([
            (
                1,
                crate::MusicCueWire {
                    mood: "calm".to_string(),
                    intensity: 1.0,
                    tempo_bpm: Some(80),
                },
            ),
            (
                2,
                crate::MusicCueWire {
                    mood: "danger".to_string(),
                    intensity: 0.8,
                    tempo_bpm: Some(140),
                },
            ),
        ]));
        assert!((ambient_volume_for_music_cues(&cues) - AMBIENT_VOLUME * 0.8).abs() < f32::EPSILON);
        // Dominant cue is danger@0.8 → tempo 140 → 1.25 clamp.
        assert!((ambient_playback_rate_for_music_cues(&cues) - 1.25).abs() < f32::EPSILON);
    }

    #[test]
    fn tempo_bpm_maps_to_clamped_playback_rate() {
        assert!((ambient_playback_rate_for_tempo_bpm(None) - 1.0).abs() < f32::EPSILON);
        assert!((ambient_playback_rate_for_tempo_bpm(Some(100)) - 1.0).abs() < f32::EPSILON);
        assert!((ambient_playback_rate_for_tempo_bpm(Some(50)) - 0.85).abs() < f32::EPSILON);
        assert!((ambient_playback_rate_for_tempo_bpm(Some(200)) - 1.25).abs() < f32::EPSILON);
    }
}
