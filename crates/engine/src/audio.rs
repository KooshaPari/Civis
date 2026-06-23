//! Audio configuration helpers for the engine.
//!
//! The crate only needs a lightweight public surface here so the top-level
//! `civ-engine` re-exports stay coherent. The actual runtime integration can
//! grow later without changing the API names used by downstream crates.

#![allow(missing_docs)]

/// Coarse era buckets used to select audio packs / mixing presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameEra {
    #[default]
    Ancient,
    Classical,
    Medieval,
    Renaissance,
    Industrial,
    Modern,
    Future,
}

/// Minimal audio configuration for a given era.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EraAudioConfig {
    /// Era this config was derived from.
    pub era: GameEra,
    /// Relative music intensity / layering hint.
    pub music_intensity: f32,
    /// Relative ambient intensity / layering hint.
    pub ambient_intensity: f32,
}

impl Default for EraAudioConfig {
    fn default() -> Self {
        Self {
            era: GameEra::Ancient,
            music_intensity: 0.25,
            ambient_intensity: 0.35,
        }
    }
}

/// Map a technology level to a coarse game era.
#[must_use]
pub fn era_from_tech_level(tech_level: u32) -> GameEra {
    match tech_level {
        0..=1 => GameEra::Ancient,
        2..=3 => GameEra::Classical,
        4..=5 => GameEra::Medieval,
        6..=7 => GameEra::Renaissance,
        8..=9 => GameEra::Industrial,
        10..=11 => GameEra::Modern,
        _ => GameEra::Future,
    }
}

/// Build an audio preset for an era.
#[must_use]
pub fn audio_config_for_era(era: GameEra) -> EraAudioConfig {
    match era {
        GameEra::Ancient => EraAudioConfig {
            era,
            music_intensity: 0.25,
            ambient_intensity: 0.35,
        },
        GameEra::Classical => EraAudioConfig {
            era,
            music_intensity: 0.35,
            ambient_intensity: 0.4,
        },
        GameEra::Medieval => EraAudioConfig {
            era,
            music_intensity: 0.45,
            ambient_intensity: 0.45,
        },
        GameEra::Renaissance => EraAudioConfig {
            era,
            music_intensity: 0.55,
            ambient_intensity: 0.5,
        },
        GameEra::Industrial => EraAudioConfig {
            era,
            music_intensity: 0.65,
            ambient_intensity: 0.55,
        },
        GameEra::Modern => EraAudioConfig {
            era,
            music_intensity: 0.75,
            ambient_intensity: 0.6,
        },
        GameEra::Future => EraAudioConfig {
            era,
            music_intensity: 0.85,
            ambient_intensity: 0.65,
        },
    }
}
