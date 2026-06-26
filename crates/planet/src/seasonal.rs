//! FR-CIV-CLIMATE — seasonal coupling layer.
//!
//! This module computes per-season modifiers that downstream crates can apply
//! to agriculture/food output (issue #913 seasonal scarcity) and disaster
//! likelihood (issue #868 seasonal clustering). All arithmetic is integer or
//! fixed-point to stay deterministic; no RNG, no tick state.
//!
//! # Design
//! `SeasonalModifiers` is a pure value derived from a [`SeasonKind`] and a
//! [`BiomeKind`].  Callers multiply their base production/probability values by
//! the returned fixed-point ratios (scale = 1 000 = "×1.0").
//!
//! # Contracts (FR-CIV-CLIMATE)
//! - Food output is highest in harvest seasons and lowest in winter.
//! - Droughts cluster in dry / summer seasons; floods cluster in wet / spring seasons.
//! - Modifiers are deterministic for identical (season, biome) inputs.

#![forbid(unsafe_code)]

use crate::geology::BiomeKind;
use crate::weather::SeasonKind;
use serde::{Deserialize, Serialize};

/// Fixed-point scale: 1 000 == "×1.0".
pub const FP_SCALE: i32 = 1_000;

/// Per-season modifiers for a given biome.
///
/// All fields are fixed-point ratios (scale 1 000).  Multiply the base value
/// by the field and divide by [`FP_SCALE`] to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeasonalModifiers {
    /// Agricultural / food production multiplier (1 000 = no change).
    pub food_productivity_fp: i32,
    /// Drought probability multiplier (1 000 = no change).
    pub drought_likelihood_fp: i32,
    /// Flood probability multiplier (1 000 = no change).
    pub flood_likelihood_fp: i32,
    /// Storm / hurricane probability multiplier (1 000 = no change).
    pub storm_likelihood_fp: i32,
    /// Wildfire probability multiplier (1 000 = no change).
    pub wildfire_likelihood_fp: i32,
}

impl SeasonalModifiers {
    /// Neutral modifiers (no seasonal adjustment).
    pub const NEUTRAL: Self = Self {
        food_productivity_fp: FP_SCALE,
        drought_likelihood_fp: FP_SCALE,
        flood_likelihood_fp: FP_SCALE,
        storm_likelihood_fp: FP_SCALE,
        wildfire_likelihood_fp: FP_SCALE,
    };
}

/// Compute [`SeasonalModifiers`] for a given season and biome archetype.
///
/// The returned modifiers express how the current season shifts base rates:
/// - Spring: high rainfall → floods up, droughts down, moderate food growth
/// - Summer: heat → droughts up in arid biomes, harvest peaks in temperate
/// - Autumn: harvest peak in temperate/forest; storms cluster
/// - Winter: food production lowest; droughts/floods minimal; ice risk
///
/// Biome adjusts the base seasonal signal: arid biomes amplify droughts, wet
/// biomes amplify floods, forest biomes amplify wildfires in summer.
#[must_use]
pub fn seasonal_modifiers(season: SeasonKind, biome: BiomeKind) -> SeasonalModifiers {
    let base = base_season_modifiers(season);
    apply_biome_amplification(base, season, biome)
}

fn base_season_modifiers(season: SeasonKind) -> SeasonalModifiers {
    match season {
        SeasonKind::Spring => SeasonalModifiers {
            food_productivity_fp: 900,   // growth phase, not yet harvest
            drought_likelihood_fp: 600,  // wet season — droughts suppressed
            flood_likelihood_fp: 1_500,  // snowmelt + rain — floods elevated
            storm_likelihood_fp: 1_200,  // spring storms common
            wildfire_likelihood_fp: 500, // wet — fires suppressed
        },
        SeasonKind::Summer => SeasonalModifiers {
            food_productivity_fp: 1_100, // peak growth
            drought_likelihood_fp: 1_600, // heat / dry season peak
            flood_likelihood_fp: 700,    // lower baseline
            storm_likelihood_fp: 1_300,  // convective storms
            wildfire_likelihood_fp: 1_700, // peak fire season
        },
        SeasonKind::Autumn => SeasonalModifiers {
            food_productivity_fp: 1_400, // harvest peak
            drought_likelihood_fp: 900,  // cooling off
            flood_likelihood_fp: 1_100,  // autumn rains
            storm_likelihood_fp: 1_400,  // hurricane season
            wildfire_likelihood_fp: 1_200, // dry after summer
        },
        SeasonKind::Winter => SeasonalModifiers {
            food_productivity_fp: 400,   // dormant season
            drought_likelihood_fp: 700,  // frozen — not drought-prone
            flood_likelihood_fp: 800,    // snowpack building
            storm_likelihood_fp: 1_100,  // winter storms
            wildfire_likelihood_fp: 300, // cold / wet
        },
    }
}

fn apply_biome_amplification(
    mut m: SeasonalModifiers,
    season: SeasonKind,
    biome: BiomeKind,
) -> SeasonalModifiers {
    match biome {
        // Arid biomes strongly amplify summer droughts.
        BiomeKind::Desert | BiomeKind::Shrubland | BiomeKind::Steppe => {
            if matches!(season, SeasonKind::Summer) {
                m.drought_likelihood_fp = (m.drought_likelihood_fp * 1_500) / FP_SCALE;
                m.wildfire_likelihood_fp = (m.wildfire_likelihood_fp * 1_300) / FP_SCALE;
            }
            m.food_productivity_fp = (m.food_productivity_fp * 600) / FP_SCALE;
        }
        // Wet biomes amplify spring/autumn floods and dampen droughts.
        BiomeKind::Wetland | BiomeKind::Rainforest | BiomeKind::Mangrove => {
            if matches!(season, SeasonKind::Spring | SeasonKind::Autumn) {
                m.flood_likelihood_fp = (m.flood_likelihood_fp * 1_400) / FP_SCALE;
            }
            m.drought_likelihood_fp = (m.drought_likelihood_fp * 500) / FP_SCALE;
        }
        // Forests amplify summer wildfires.
        BiomeKind::Forest | BiomeKind::Taiga | BiomeKind::Savanna => {
            if matches!(season, SeasonKind::Summer | SeasonKind::Autumn) {
                m.wildfire_likelihood_fp = (m.wildfire_likelihood_fp * 1_400) / FP_SCALE;
            }
            // Good food yield in forest biomes (foraging/agriculture mix)
            m.food_productivity_fp = (m.food_productivity_fp * 1_100) / FP_SCALE;
        }
        // Tundra / Polar: very low food all year; freeze/thaw flood risk.
        BiomeKind::Tundra | BiomeKind::Glacier | BiomeKind::Alpine => {
            m.food_productivity_fp = (m.food_productivity_fp * 300) / FP_SCALE;
            if matches!(season, SeasonKind::Spring) {
                m.flood_likelihood_fp = (m.flood_likelihood_fp * 1_600) / FP_SCALE; // ice melt
            }
        }
        // Plains / grassland: solid harvest season; high wildfire in dry summer.
        BiomeKind::Plains | BiomeKind::Grassland => {
            if matches!(season, SeasonKind::Autumn) {
                m.food_productivity_fp = (m.food_productivity_fp * 1_300) / FP_SCALE;
            }
            if matches!(season, SeasonKind::Summer) {
                m.wildfire_likelihood_fp = (m.wildfire_likelihood_fp * 1_200) / FP_SCALE;
            }
        }
        // Coastal / mountain: moderate adjustments.
        BiomeKind::Beach | BiomeKind::Mountain => {
            if matches!(season, SeasonKind::Summer) {
                m.storm_likelihood_fp = (m.storm_likelihood_fp * 1_200) / FP_SCALE;
            }
        }
        // Ocean: storms peak in summer/autumn.
        BiomeKind::Ocean => {
            if matches!(season, SeasonKind::Summer | SeasonKind::Autumn) {
                m.storm_likelihood_fp = (m.storm_likelihood_fp * 1_500) / FP_SCALE;
            }
        }
    }
    m
}

/// Apply a fixed-point seasonal modifier to a base value.
///
/// `base_value * modifier_fp / FP_SCALE`, saturating at `i64::MAX`.
#[must_use]
pub fn apply_modifier(base_value: i64, modifier_fp: i32) -> i64 {
    base_value
        .saturating_mul(modifier_fp as i64)
        .saturating_div(FP_SCALE as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    // FR-CIV-CLIMATE-1: seasons cycle deterministically
    #[test]
    fn modifiers_are_deterministic_for_same_inputs() {
        let a = seasonal_modifiers(SeasonKind::Summer, BiomeKind::Plains);
        let b = seasonal_modifiers(SeasonKind::Summer, BiomeKind::Plains);
        assert_eq!(a, b);

        let c = seasonal_modifiers(SeasonKind::Winter, BiomeKind::Forest);
        let d = seasonal_modifiers(SeasonKind::Winter, BiomeKind::Forest);
        assert_eq!(c, d);
    }

    // FR-CIV-CLIMATE-2: food/disasters vary by season
    #[test]
    fn food_productivity_peaks_in_autumn_plains() {
        let spring = seasonal_modifiers(SeasonKind::Spring, BiomeKind::Plains);
        let summer = seasonal_modifiers(SeasonKind::Summer, BiomeKind::Plains);
        let autumn = seasonal_modifiers(SeasonKind::Autumn, BiomeKind::Plains);
        let winter = seasonal_modifiers(SeasonKind::Winter, BiomeKind::Plains);

        // Autumn should be the highest (harvest)
        assert!(autumn.food_productivity_fp > summer.food_productivity_fp);
        assert!(autumn.food_productivity_fp > spring.food_productivity_fp);
        assert!(autumn.food_productivity_fp > winter.food_productivity_fp);
        // Winter should be the lowest (dormant)
        assert!(winter.food_productivity_fp < spring.food_productivity_fp);
    }

    // FR-CIV-CLIMATE-3: droughts cluster in summer dry season
    #[test]
    fn drought_peaks_in_summer_desert() {
        let spring = seasonal_modifiers(SeasonKind::Spring, BiomeKind::Desert);
        let summer = seasonal_modifiers(SeasonKind::Summer, BiomeKind::Desert);

        assert!(summer.drought_likelihood_fp > spring.drought_likelihood_fp);
    }

    // FR-CIV-CLIMATE-4: floods cluster in spring wet season
    #[test]
    fn floods_peak_in_spring_wetland() {
        let spring = seasonal_modifiers(SeasonKind::Spring, BiomeKind::Wetland);
        let summer = seasonal_modifiers(SeasonKind::Summer, BiomeKind::Wetland);

        assert!(spring.flood_likelihood_fp > summer.flood_likelihood_fp);
    }

    // Wildfire peaks in summer/autumn for forests
    #[test]
    fn wildfire_peaks_in_summer_forest() {
        let spring = seasonal_modifiers(SeasonKind::Spring, BiomeKind::Forest);
        let summer = seasonal_modifiers(SeasonKind::Summer, BiomeKind::Forest);
        let winter = seasonal_modifiers(SeasonKind::Winter, BiomeKind::Forest);

        assert!(summer.wildfire_likelihood_fp > spring.wildfire_likelihood_fp);
        assert!(summer.wildfire_likelihood_fp > winter.wildfire_likelihood_fp);
    }

    // Tundra has very low food productivity all year
    #[test]
    fn tundra_food_is_low_all_seasons() {
        for season in [
            SeasonKind::Spring,
            SeasonKind::Summer,
            SeasonKind::Autumn,
            SeasonKind::Winter,
        ] {
            let m = seasonal_modifiers(season, BiomeKind::Tundra);
            assert!(
                m.food_productivity_fp < FP_SCALE,
                "Tundra food should be below baseline in {season:?}, got {}",
                m.food_productivity_fp
            );
        }
    }

    // apply_modifier correctly scales values
    #[test]
    fn apply_modifier_scales_correctly() {
        assert_eq!(apply_modifier(1000, FP_SCALE), 1000); // neutral
        assert_eq!(apply_modifier(1000, 1_500), 1500); // +50%
        assert_eq!(apply_modifier(1000, 500), 500); // -50%
        assert_eq!(apply_modifier(0, 2_000), 0); // zero stays zero
    }

    // All biome × season combinations produce valid (non-negative) modifiers
    #[test]
    fn all_biome_season_combinations_produce_valid_modifiers() {
        let seasons = [
            SeasonKind::Spring,
            SeasonKind::Summer,
            SeasonKind::Autumn,
            SeasonKind::Winter,
        ];
        let biomes = [
            BiomeKind::Ocean,
            BiomeKind::Plains,
            BiomeKind::Forest,
            BiomeKind::Mountain,
            BiomeKind::Desert,
            BiomeKind::Tundra,
            BiomeKind::Beach,
            BiomeKind::Savanna,
            BiomeKind::Grassland,
            BiomeKind::Rainforest,
            BiomeKind::Taiga,
            BiomeKind::Glacier,
            BiomeKind::Wetland,
            BiomeKind::Shrubland,
            BiomeKind::Steppe,
            BiomeKind::Alpine,
            BiomeKind::Mangrove,
        ];
        for season in seasons {
            for biome in biomes {
                let m = seasonal_modifiers(season, biome);
                assert!(m.food_productivity_fp >= 0);
                assert!(m.drought_likelihood_fp >= 0);
                assert!(m.flood_likelihood_fp >= 0);
                assert!(m.storm_likelihood_fp >= 0);
                assert!(m.wildfire_likelihood_fp >= 0);
            }
        }
    }
}
