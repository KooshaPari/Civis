//! [`AudioMix`] — the top-level bus-mix resource (FR-CIV-AUDIO-001).
//!
//! Owns the per-tier [`BusLevels`] and the optional master mute. Lives
//! as a `bevy::Resource` in the client and as a plain `serde` value
//! in the settings file; the math here does not depend on Bevy.

use serde::{Deserialize, Serialize};

use crate::bus::{BusId, BusLevels};

/// Schema version of the persisted `AudioMix` blob.
///
/// Bumped on breaking changes to the serialised shape (new tier, removed
/// field). Migrations live in the engine save format, not here — this
/// constant is the in-substrate version that the settings file carries.
pub const MIX_SCHEMA_VERSION: u32 = 1;

/// A named starting mix that the player can pick from the settings UI
/// (e.g. "Headphones", "Speakers — TV", "Late Night"). A preset is
/// purely a [`BusLevels`] snapshot + a stable id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioMixPreset {
    /// The §3 default — designer-tuned reference mix.
    Reference,
    /// Quieter score + louder SFX; player-friendly on TVs / laptops.
    Casual,
    /// Loud score, soft SFX; for music-first listening.
    Music,
    /// SFX + UI only; ambient + score muted but the bus still exists
    /// in the mix tree (a muted bus is silent, not removed).
    SilentAmbient,
}

impl AudioMixPreset {
    /// Returns the [`BusLevels`] for this preset.
    pub fn levels(self) -> BusLevels {
        match self {
            AudioMixPreset::Reference => BusLevels::DEFAULTS,
            AudioMixPreset::Casual => BusLevels {
                master: 1.0,
                ambient: 0.30,
                score: 0.20,
                sfx: 0.80,
                ui: 0.60,
            },
            AudioMixPreset::Music => BusLevels {
                master: 1.0,
                ambient: 0.25,
                score: 0.55,
                sfx: 0.55,
                ui: 0.45,
            },
            AudioMixPreset::SilentAmbient => BusLevels {
                master: 1.0,
                ambient: 0.0,
                score: 0.0,
                sfx: 0.70,
                ui: 0.55,
            },
        }
    }
}

/// The runtime audio mix resource.
///
/// Combines the per-tier [`BusLevels`] with a master mute and the
/// schema version. Serialised as the on-disk settings file fragment
/// the player gets when they adjust audio sliders. Designed to be
/// cheap to copy / diff so the settings UI can preview an unapplied
/// edit and then commit it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AudioMix {
    /// Schema version of this on-disk shape.
    pub schema_version: u32,
    /// Currently selected preset, or `None` if the player has hand-edited.
    pub preset: Option<AudioMixPreset>,
    /// Per-tier linear gains.
    pub levels: BusLevels,
    /// `true` ⇒ all output is silenced (master slider still applies;
    /// the mute is a hard 0 dB on the sum bus). Independent of the
    /// per-tier sliders so un-muting does not change tier gains.
    pub master_mute: bool,
}

impl Default for AudioMix {
    fn default() -> Self {
        Self {
            schema_version: MIX_SCHEMA_VERSION,
            preset: Some(AudioMixPreset::Reference),
            levels: BusLevels::DEFAULTS,
            master_mute: false,
        }
    }
}

impl AudioMix {
    /// Apply a preset — overwrites the per-tier levels and clears
    /// `master_mute`. Returns the previous `preset` value.
    pub fn apply_preset(&mut self, preset: AudioMixPreset) -> Option<AudioMixPreset> {
        let prev = self.preset;
        self.preset = Some(preset);
        self.levels = preset.levels();
        self.master_mute = false;
        prev
    }

    /// Effective linear gain for a tier bus, after the master
    /// mute is applied. `0.0` when `master_mute` is on.
    pub fn effective_gain(&self, bus: BusId) -> f32 {
        if self.master_mute {
            0.0
        } else {
            self.levels.get(bus)
        }
    }

    /// Per-tier getter, routed through `effective_gain` so consumers
    /// never accidentally skip the mute. The kira plugin reads this.
    pub fn tier_gain(&self, bus: BusId) -> f32 {
        if bus.is_tier() {
            self.effective_gain(bus)
        } else {
            // Master itself is a pass-through; the mute is its toggle.
            if self.master_mute {
                0.0
            } else {
                1.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mix_is_reference_preset() {
        let m = AudioMix::default();
        assert_eq!(m.preset, Some(AudioMixPreset::Reference));
        assert!(!m.master_mute);
        assert_eq!(m.schema_version, MIX_SCHEMA_VERSION);
    }

    #[test]
    fn apply_preset_clears_mute_and_records_change() {
        let mut m = AudioMix::default();
        m.master_mute = true;

        let prev = m.apply_preset(AudioMixPreset::Casual);
        assert_eq!(prev, Some(AudioMixPreset::Reference));
        assert_eq!(m.preset, Some(AudioMixPreset::Casual));
        assert!(!m.master_mute);
        assert!((m.levels.ambient - 0.30).abs() < f32::EPSILON);
    }

    #[test]
    fn effective_gain_returns_zero_under_master_mute() {
        let mut m = AudioMix::default();
        assert!(m.effective_gain(BusId::Sfx) > 0.0);
        m.master_mute = true;
        assert!(m.effective_gain(BusId::Sfx).abs() < f32::EPSILON);
        assert!(m.effective_gain(BusId::Ambient).abs() < f32::EPSILON);
    }

    #[test]
    fn tier_gain_routes_master_through_mute_toggle() {
        let mut m = AudioMix::default();
        assert!((m.tier_gain(BusId::Master) - 1.0).abs() < f32::EPSILON);
        m.master_mute = true;
        assert!(m.tier_gain(BusId::Master).abs() < f32::EPSILON);
    }

    #[test]
    fn silent_ambient_preset_zeros_score_and_ambient() {
        let lv = AudioMixPreset::SilentAmbient.levels();
        assert!(lv.ambient.abs() < f32::EPSILON);
        assert!(lv.score.abs() < f32::EPSILON);
        assert!(lv.sfx > 0.0);
        assert!(lv.ui > 0.0);
    }

    #[test]
    fn serde_round_trip_preserves_state() {
        let original = AudioMix {
            schema_version: MIX_SCHEMA_VERSION,
            preset: Some(AudioMixPreset::Music),
            levels: AudioMixPreset::Music.levels(),
            master_mute: false,
        };
        let json = serde_json::to_string(&original).unwrap();
        let back: AudioMix = serde_json::from_str(&json).unwrap();
        assert_eq!(back, original);
    }

    /// Covers FR-CIV-AUDIO-001 — five player sliders + master mute;
    /// each tier independently mutable; persisted in settings. We
    /// assert the persistence shape is stable (schema version + serde
    /// round-trip) and that the mute is independent of per-tier edits.
    #[test]
    fn fr_audio_001_mix_persists_and_mute_is_independent() {
        let mut m = AudioMix::default();
        // Per-tier edits do not flip master_mute.
        m.levels.set(BusId::Sfx, 0.20);
        m.levels.set(BusId::Ui, 0.95);
        assert!(!m.master_mute);
        assert!((m.levels.get(BusId::Sfx) - 0.20).abs() < f32::EPSILON);
        assert!((m.levels.get(BusId::Ui) - 0.95).abs() < f32::EPSILON);
        // Persist + restore.
        let json = serde_json::to_string(&m).unwrap();
        let back: AudioMix = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
        assert_eq!(back.schema_version, MIX_SCHEMA_VERSION);
    }
}
