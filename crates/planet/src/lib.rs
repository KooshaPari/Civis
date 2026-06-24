//! civ-planet - deterministic single-planet geology, weather, day/night, and tides
//!
//! The API is intentionally small and pure so replay/state sync can derive the
//! same climate inputs from tick alone.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod geology;
pub use geology::{BiomeKind, GeologyMap, RegionBiome};
pub mod weather;
pub use weather::{compute_weather, SeasonKind, WeatherCell, WeatherKind};

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

/// Marker version of this crate's public schema.
pub const SCHEMA_VERSION: &str = "0.1.0-stub";

/// Immutable planet parameters used to derive climate phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanetConfig {
    /// Planet radius in kilometers.
    pub radius_km: u32,
    /// Axial tilt in degrees.
    pub axial_tilt_deg: u16,
    /// Length of one day in ticks.
    pub day_length_ticks: u32,
    /// Length of one year in ticks.
    pub year_length_ticks: u32,
}

/// Immutable moon parameters used to derive tide modulation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MoonConfig {
    /// Length of one lunar orbit in ticks.
    pub orbit_period_ticks: u32,
    /// Maximum tidal offset applied by the moon.
    pub tidal_amplitude: f32,
}

/// Deterministic climate snapshot for a single tick.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Climate {
    /// Simulation tick this climate was derived from.
    pub tick: u64,
    /// Fractional phase through the current day in `[0, 1)`.
    pub day_phase: f32,
    /// Fractional phase through the current year in `[0, 1)`.
    pub year_phase: f32,
    /// Fractional phase through the current lunar orbit in `[0, 1)`.
    pub moon_phase: f32,
    /// Tide offset induced by the moon.
    pub tide_offset: f32,
}

fn phase(tick: u64, period_ticks: u32) -> f32 {
    let period = period_ticks.max(1) as f32;
    (tick % period_ticks.max(1) as u64) as f32 / period
}

/// Compute the deterministic climate snapshot for a given tick.
pub fn compute_climate(tick: u64, planet: &PlanetConfig, moon: &MoonConfig) -> Climate {
    let day_phase = phase(tick, planet.day_length_ticks);
    let year_phase = phase(tick, planet.year_length_ticks);
    let moon_phase = phase(tick, moon.orbit_period_ticks);
    let tide_offset = moon.tidal_amplitude * (TAU * moon_phase).sin();

    Climate {
        tick,
        day_phase,
        year_phase,
        moon_phase,
        tide_offset,
    }
}

/// Determine whether the supplied climate falls within the daytime window.
pub fn is_daytime(climate: &Climate) -> bool {
    (0.25..=0.75).contains(&climate.day_phase)
}

/// Earth-like defaults used by the 3D extension foundation.
pub fn defaults_earthlike() -> (PlanetConfig, MoonConfig) {
    (
        PlanetConfig {
            radius_km: 6_371,
            axial_tilt_deg: 23,
            day_length_ticks: 24_000,
            year_length_ticks: 8_766_000,
        },
        MoonConfig {
            orbit_period_ticks: 240_000,
            tidal_amplitude: 0.6,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Covers FR-CIV-PLANET-000 — exposes a semver-like schema version stub.
    #[test]
    fn schema_version_stub() {
        assert!(!SCHEMA_VERSION.is_empty());
        let core = SCHEMA_VERSION.split('-').next().unwrap();
        let segments: Vec<&str> = core.split('.').collect();
        assert_eq!(segments.len(), 3);
        assert!(segments.iter().all(|part| !part.is_empty()));
    }

    /// Covers FR-CIV-PLANET-001 - day phase wraps deterministically across the day length.
    #[test]
    fn day_phase_wraps_deterministically() {
        let planet = PlanetConfig {
            radius_km: 1,
            axial_tilt_deg: 0,
            day_length_ticks: 10,
            year_length_ticks: 100,
        };
        let moon = MoonConfig {
            orbit_period_ticks: 20,
            tidal_amplitude: 1.0,
        };

        let a = compute_climate(3, &planet, &moon);
        let b = compute_climate(13, &planet, &moon);

        assert_eq!(a.day_phase, 0.3);
        assert_eq!(b.day_phase, 0.3);
    }

    /// Covers FR-CIV-PLANET-002 - moon tide offset remains within the configured amplitude.
    #[test]
    fn tide_offset_stays_within_amplitude() {
        let planet = PlanetConfig {
            radius_km: 1,
            axial_tilt_deg: 0,
            day_length_ticks: 10,
            year_length_ticks: 100,
        };
        let moon = MoonConfig {
            orbit_period_ticks: 12,
            tidal_amplitude: 0.75,
        };

        for tick in 0..48_u64 {
            let climate = compute_climate(tick, &planet, &moon);
            assert!(climate.tide_offset <= moon.tidal_amplitude);
            assert!(climate.tide_offset >= -moon.tidal_amplitude);
        }
    }

    /// Covers FR-CIV-PLANET-003 - daytime is true at noon and false at midnight.
    #[test]
    fn daytime_matches_expected_window() {
        let climate_noon = Climate {
            tick: 0,
            day_phase: 0.5,
            year_phase: 0.0,
            moon_phase: 0.0,
            tide_offset: 0.0,
        };
        let climate_midnight = Climate {
            tick: 0,
            day_phase: 0.0,
            year_phase: 0.0,
            moon_phase: 0.0,
            tide_offset: 0.0,
        };

        assert!(is_daytime(&climate_noon));
        assert!(!is_daytime(&climate_midnight));
    }

    /// Covers FR-CIV-PLANET-004 - earthlike defaults are non-zero and plausible.
    #[test]
    fn defaults_are_sane() {
        let (planet, moon) = defaults_earthlike();

        assert!(planet.radius_km > 0);
        assert!(planet.axial_tilt_deg > 0);
        assert!(planet.day_length_ticks > 0);
        assert!(planet.year_length_ticks > 0);
        assert!(moon.orbit_period_ticks > 0);
        assert!(moon.tidal_amplitude > 0.0);
    }

    /// Covers FR-CIV-PLANET-005 - climate computation is pure and bit-identical across calls.
    #[test]
    fn climate_is_bit_identical_across_calls() {
        let (planet, moon) = defaults_earthlike();
        let a = compute_climate(123_456, &planet, &moon);
        let b = compute_climate(123_456, &planet, &moon);

        assert_eq!(a.tick, b.tick);
        assert_eq!(a.day_phase.to_bits(), b.day_phase.to_bits());
        assert_eq!(a.year_phase.to_bits(), b.year_phase.to_bits());
        assert_eq!(a.moon_phase.to_bits(), b.moon_phase.to_bits());
        assert_eq!(a.tide_offset.to_bits(), b.tide_offset.to_bits());
    }
}
