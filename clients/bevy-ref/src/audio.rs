//! Ambient soundscape + SFX system for the Civis Bevy client.
//!
//! [`CivisAudioPlugin`] is self-contained and additive: it inserts an audio
//! resource set and a small event queue, then drains that queue every frame to
//! drive playback. Other systems never touch the audio backend directly — they
//! call [`play_sfx`] (or write a [`SfxEvent`]) and the plugin does the rest.
//!
//! ## What it provides
//! - **Ambient bed** — a looping wind / nature soundscape started at boot on its
//!   own [`AudioChannel<AmbientChannel>`] so it can be ducked / muted
//!   independently of one-shot effects.
//! - **UI click SFX** — [`SfxKind::UiClick`] for button / menu feedback.
//! - **Event SFX hooks** — [`SfxKind::Birth`], [`SfxKind::Death`],
//!   [`SfxKind::Disaster`], [`SfxKind::Build`] for simulation events. Wire these
//!   from `sim_bridge` / `event_feed` by writing a [`SfxEvent`].
//!
//! ## Public API for other systems
//! ```ignore
//! // From any system with `Commands`/`MessageWriter<SfxEvent>` access:
//! fn on_birth(mut sfx: MessageWriter<civ_bevy_ref::audio::SfxEvent>) {
//!     sfx.write(civ_bevy_ref::audio::SfxEvent::new(SfxKind::Birth));
//! }
//! ```
//! [`play_sfx`] is a thin helper for the common `MessageWriter` path.
//!
//! ## Audio assets (CC0 drop-in)
//! This plugin loads its clips from **`assets/audio/`** relative to the client
//! working directory. The default file names live in [`AudioFiles`]:
//!
//! | Slot       | Default path                     | Suggested CC0 source                     |
//! |------------|----------------------------------|------------------------------------------|
//! | ambient    | `assets/audio/ambient_wind.ogg`  | freesound.org / kenney.nl nature beds    |
//! | UI click   | `assets/audio/ui_click.ogg`      | kenney.nl "UI Audio" pack (CC0)          |
//! | birth      | `assets/audio/birth.ogg`         | kenney.nl "Interface Sounds" (CC0)       |
//! | death      | `assets/audio/death.ogg`         | kenney.nl / freesound CC0                |
//! | disaster   | `assets/audio/disaster.ogg`      | freesound.org CC0 (rumble / impact)      |
//! | build      | `assets/audio/build.ogg`         | kenney.nl "Impact Sounds" (CC0)          |
//!
//! When a file is **absent**, `bevy_kira_audio` logs a missing-asset warning and
//! the corresponding sound is simply silent — the app stays green and playable.
//! To ship real audio, drop CC0 `.ogg` files at the paths above (no code change
//! needed). For a fully procedural placeholder (a generated sine tone instead of
//! silence) see the note on [`AudioHandles::resolve`].
//!
//! Feature-gated behind the `audio` cargo feature (which implies `bevy`).

#![cfg(feature = "audio")]

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

// ── Channels ────────────────────────────────────────────────────────────────

/// Dedicated channel for the looping ambient bed (mute / duck independently).
#[derive(Resource)]
pub struct AmbientChannel;

/// Dedicated channel for one-shot SFX (UI + simulation events).
#[derive(Resource)]
pub struct SfxChannel;

// ── Event kinds ─────────────────────────────────────────────────────────────

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
}

/// Bevy message other systems write to trigger a one-shot SFX.
///
/// Prefer the [`play_sfx`] helper, which constructs and sends this for you.
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
///
/// ```ignore
/// fn on_button(mut writer: MessageWriter<SfxEvent>) {
///     play_sfx(&mut writer, SfxKind::UiClick);
/// }
/// ```
pub fn play_sfx(writer: &mut MessageWriter<SfxEvent>, kind: SfxKind) {
    writer.write(SfxEvent::new(kind));
}

// ── Config + handles ────────────────────────────────────────────────────────

/// Where each clip is loaded from under `assets/`. Override before adding the
/// plugin to point at a different pack.
#[derive(Resource, Debug, Clone)]
pub struct AudioFiles {
    /// Looping ambient bed (wind / nature).
    pub ambient: String,
    /// UI click.
    pub ui_click: String,
    /// Agent birth.
    pub birth: String,
    /// Agent death.
    pub death: String,
    /// Disaster.
    pub disaster: String,
    /// Building constructed.
    pub build: String,
}

impl Default for AudioFiles {
    fn default() -> Self {
        Self {
            ambient: "audio/ambient_wind.ogg".to_string(),
            ui_click: "audio/ui_click.ogg".to_string(),
            birth: "audio/birth.ogg".to_string(),
            death: "audio/death.ogg".to_string(),
            disaster: "audio/disaster.ogg".to_string(),
            build: "audio/build.ogg".to_string(),
        }
    }
}

/// Loaded clip handles, populated at startup from [`AudioFiles`].
#[derive(Resource, Default)]
pub struct AudioHandles {
    /// Looping ambient bed.
    pub ambient: Handle<bevy_kira_audio::AudioSource>,
    /// UI click.
    pub ui_click: Handle<bevy_kira_audio::AudioSource>,
    /// Agent birth.
    pub birth: Handle<bevy_kira_audio::AudioSource>,
    /// Agent death.
    pub death: Handle<bevy_kira_audio::AudioSource>,
    /// Disaster.
    pub disaster: Handle<bevy_kira_audio::AudioSource>,
    /// Building constructed.
    pub build: Handle<bevy_kira_audio::AudioSource>,
}

impl AudioHandles {
    /// Resolve a [`SfxKind`] to its loaded clip handle.
    ///
    /// NOTE (procedural placeholder): if you want a *generated tone* instead of
    /// silence when a CC0 file is missing, build a `kira` `StaticSoundData` from
    /// a sine sample buffer and register it as an `AudioSource` here, returning
    /// that handle as the fallback. The asset-file path above is preferred so
    /// the default keeps zero baked binary data in the repo.
    #[must_use]
    pub fn for_kind(&self, kind: SfxKind) -> Handle<bevy_kira_audio::AudioSource> {
        match kind {
            SfxKind::UiClick => self.ui_click.clone(),
            SfxKind::Birth => self.birth.clone(),
            SfxKind::Death => self.death.clone(),
            SfxKind::Disaster => self.disaster.clone(),
            SfxKind::Build => self.build.clone(),
        }
    }
}

/// Startup ambient-bed volume (linear).
pub const AMBIENT_VOLUME: f32 = 0.35;

// ── Plugin ──────────────────────────────────────────────────────────────────

/// Ambient soundscape + SFX plugin for the Civis Bevy client.
///
/// Named `CivisAudioPlugin` to avoid clashing with Bevy's built-in `AudioPlugin`
/// and `bevy_kira_audio`'s `AudioPlugin` (which this plugin pulls in for you).
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
            .add_message::<SfxEvent>()
            .add_systems(Startup, (load_audio, start_ambient).chain())
            .add_systems(Update, drain_sfx_events);
    }
}

/// Load every clip handle from [`AudioFiles`] (missing files warn, not panic).
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
}

/// Kick off the looping ambient bed on the dedicated ambient channel.
fn start_ambient(channel: Res<AudioChannel<AmbientChannel>>, handles: Res<AudioHandles>) {
    channel
        .play(handles.ambient.clone())
        .looped()
        .with_volume(AMBIENT_VOLUME);
}

/// Drain queued [`SfxEvent`]s and play each on the SFX channel.
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
    }

    #[test]
    fn for_kind_maps_each_variant_to_a_handle_slot() {
        let handles = AudioHandles::default();
        // Distinct match arms compile + each returns a (default/weak) handle.
        for kind in [
            SfxKind::UiClick,
            SfxKind::Birth,
            SfxKind::Death,
            SfxKind::Disaster,
            SfxKind::Build,
        ] {
            let _ = handles.for_kind(kind);
        }
    }
}
