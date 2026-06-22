//! Era-aware audio configuration helpers.

/// The player's current civilization era.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameEra {
    /// Early survival, primitive tools, fire, and hand drums.
    #[default]
    Stone,
    /// First organized settlements and metalworking.
    Bronze,
    /// Larger fortified societies and disciplined military culture.
    Iron,
    /// Feudal courts, cathedrals, and regional trade networks.
    Medieval,
    /// Courtly refinement, exploration, and early orchestration.
    Renaissance,
    /// Mechanization, steam power, and factory cities.
    Industrial,
    /// Electrified, globalized, and synthetic modern culture.
    Modern,
}

/// Audio settings associated with a specific era.
#[derive(Debug, Clone, PartialEq)]
pub struct EraAudioConfig {
    /// The ambient or looped music track used for the era.
    pub ambient_track: String,
    /// Base tempo for music selection or sequencing.
    pub music_tempo: f32,
    /// Instrument family or asset identifiers used by the arrangement layer.
    pub instrument_set: Vec<String>,
}

/// Return the era-specific audio configuration.
pub fn audio_config_for_era(era: &GameEra) -> EraAudioConfig {
    match era {
        GameEra::Stone => EraAudioConfig {
            ambient_track: "audio/stone_embers.ogg".to_string(),
            music_tempo: 72.0,
            instrument_set: vec![
                "hand_drum".to_string(),
                "bone_flute".to_string(),
                "stone_rattle".to_string(),
            ],
        },
        GameEra::Bronze => EraAudioConfig {
            ambient_track: "audio/bronze_harvest.ogg".to_string(),
            music_tempo: 84.0,
            instrument_set: vec![
                "lyre".to_string(),
                "reed_pipe".to_string(),
                "bronze_chimes".to_string(),
            ],
        },
        GameEra::Iron => EraAudioConfig {
            ambient_track: "audio/iron_fortress.ogg".to_string(),
            music_tempo: 96.0,
            instrument_set: vec![
                "war_drum".to_string(),
                "horn".to_string(),
                "string_ensemble".to_string(),
            ],
        },
        GameEra::Medieval => EraAudioConfig {
            ambient_track: "audio/medieval_court.ogg".to_string(),
            music_tempo: 102.0,
            instrument_set: vec![
                "lute".to_string(),
                "hurdy_gurdy".to_string(),
                "fiddle".to_string(),
            ],
        },
        GameEra::Renaissance => EraAudioConfig {
            ambient_track: "audio/renaissance_citadel.ogg".to_string(),
            music_tempo: 108.0,
            instrument_set: vec![
                "harpsichord".to_string(),
                "violin".to_string(),
                "flute".to_string(),
            ],
        },
        GameEra::Industrial => EraAudioConfig {
            ambient_track: "audio/industrial_forge.ogg".to_string(),
            music_tempo: 116.0,
            instrument_set: vec![
                "piano".to_string(),
                "brass_section".to_string(),
                "steam_percussion".to_string(),
            ],
        },
        GameEra::Modern => EraAudioConfig {
            ambient_track: "audio/modern_metropolis.ogg".to_string(),
            music_tempo: 124.0,
            instrument_set: vec![
                "synth_pad".to_string(),
                "electric_bass".to_string(),
                "drum_kit".to_string(),
            ],
        },
    }
}

/// Convert a tech count into the current civilization era.
pub fn era_from_tech_level(tech_count: u32) -> GameEra {
    match tech_count {
        0..=2 => GameEra::Stone,
        3..=5 => GameEra::Bronze,
        6..=9 => GameEra::Iron,
        10..=14 => GameEra::Medieval,
        15..=19 => GameEra::Renaissance,
        20..=29 => GameEra::Industrial,
        _ => GameEra::Modern,
    }
}

#[cfg(test)]
mod tests {
    use super::{audio_config_for_era, era_from_tech_level, EraAudioConfig, GameEra};

    #[test]
    fn era_thresholds_match_spec() {
        assert_eq!(era_from_tech_level(0), GameEra::Stone);
        assert_eq!(era_from_tech_level(3), GameEra::Bronze);
        assert_eq!(era_from_tech_level(6), GameEra::Iron);
        assert_eq!(era_from_tech_level(10), GameEra::Medieval);
        assert_eq!(era_from_tech_level(15), GameEra::Renaissance);
        assert_eq!(era_from_tech_level(20), GameEra::Industrial);
        assert_eq!(era_from_tech_level(30), GameEra::Modern);
    }

    #[test]
    fn stone_audio_config_is_default_campfire_theme() {
        let config = audio_config_for_era(&GameEra::default());
        assert_eq!(
            config,
            EraAudioConfig {
                ambient_track: "audio/stone_embers.ogg".to_string(),
                music_tempo: 72.0,
                instrument_set: vec![
                    "hand_drum".to_string(),
                    "bone_flute".to_string(),
                    "stone_rattle".to_string(),
                ],
            }
        );
    }

    #[test]
    fn modern_audio_config_has_electronic_instrumentation() {
        let config = audio_config_for_era(&GameEra::Modern);
        assert_eq!(config.ambient_track, "audio/modern_metropolis.ogg");
        assert_eq!(config.music_tempo, 124.0);
        assert_eq!(
            config.instrument_set,
            vec![
                "synth_pad".to_string(),
                "electric_bass".to_string(),
                "drum_kit".to_string(),
            ]
        );
    }
}
