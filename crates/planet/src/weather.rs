//! FR-CIV-PLANET-030 - deterministic per-cell weather simulation.
//!
//! The simulation is driven by climate + time and produces one weather cell
//! per region. Each cell exposes temperature, precipitation, and storm state
//! so downstream clients can render the current weather without recomputing it.

#![forbid(unsafe_code)]

use crate::Climate;
use serde::{Deserialize, Serialize};

const FULL_TURN_FP: i64 = 360_000;
const FP_SCALE: i64 = 1_000;
const MAX_LAT_FP: i64 = 90_000;

/// Coarse weather state for a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherKind {
    /// Clear skies.
    Clear,
    /// Rainfall.
    Rain,
    /// Snowfall.
    Snow,
    /// Storm conditions.
    Storm,
}

/// Seasonal bucket used by the weather generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeasonKind {
    /// Spring.
    Spring,
    /// Summer.
    Summer,
    /// Autumn.
    Autumn,
    /// Winter.
    Winter,
}

/// Deterministic weather result for a single region.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherCell {
    /// Region identifier.
    pub region_id: u32,
    /// Latitude in fixed-point degrees.
    pub latitude_fp: i32,
    /// Current season.
    pub season: SeasonKind,
    /// Weather classification.
    pub kind: WeatherKind,
    /// Temperature in fixed-point Celsius.
    pub temp_c_fp: i32,
    /// Precipitation in fixed-point millimeters.
    pub precip_mm_fp: i32,
    /// Storm intensity in fixed-point units.
    pub storm_intensity_fp: i32,
}

fn sin_fp(angle_fp: i64) -> i32 {
    let radians =
        (angle_fp.rem_euclid(FULL_TURN_FP) as f64 / FULL_TURN_FP as f64) * std::f64::consts::TAU;
    (radians.sin() * FP_SCALE as f64).round() as i32
}

fn season_from_year_phase(year_phase: f32) -> SeasonKind {
    match year_phase.rem_euclid(1.0) {
        phase if phase < 0.25 => SeasonKind::Spring,
        phase if phase < 0.5 => SeasonKind::Summer,
        phase if phase < 0.75 => SeasonKind::Autumn,
        _ => SeasonKind::Winter,
    }
}

fn lat_from_region(region_id: u32, num_regions: u32) -> i64 {
    let n = num_regions.max(1) as i64;
    if n == 1 {
        0
    } else {
        -MAX_LAT_FP + (2 * MAX_LAT_FP * region_id as i64) / (n - 1)
    }
}

fn temp_baseline_for_lat(lat_fp: i64) -> i64 {
    let equator_boost = ((MAX_LAT_FP - lat_fp.abs()) * 40) / MAX_LAT_FP;
    8_000 + equator_boost * FP_SCALE
}

fn season_offset(season: SeasonKind, lat_fp: i64) -> i64 {
    let hemisphere = if lat_fp >= 0 { 1 } else { -1 };
    match season {
        SeasonKind::Spring => 1_500 * hemisphere,
        SeasonKind::Summer => 5_500 * hemisphere,
        SeasonKind::Autumn => -500 * hemisphere,
        SeasonKind::Winter => -6_500 * hemisphere,
    }
}

fn precipitation_from_temp_and_moisture(temp_c_fp: i64, moisture_fp: i64, storm_fp: i64) -> i64 {
    let warm_bonus = ((temp_c_fp - 5_000).max(0) * 6) / 10;
    let cold_bonus = ((0 - temp_c_fp).max(0) * 4) / 10;
    let mut precip = moisture_fp + warm_bonus + cold_bonus + storm_fp / 4;
    if temp_c_fp <= 0 {
        precip += 1_200;
    }
    precip.max(0)
}

fn weather_kind_from(temp_c_fp: i64, precip_mm_fp: i64, storm_intensity_fp: i64) -> WeatherKind {
    if storm_intensity_fp >= 1_500 {
        WeatherKind::Storm
    } else if temp_c_fp <= 0 && precip_mm_fp > 250 {
        WeatherKind::Snow
    } else if precip_mm_fp > 200 {
        WeatherKind::Rain
    } else {
        WeatherKind::Clear
    }
}

/// Compute the current weather for each region.
#[must_use]
pub fn compute_weather(climate: &Climate, tick: u64, num_regions: u32) -> Vec<WeatherCell> {
    let n = num_regions.max(1);
    let season = season_from_year_phase(climate.year_phase);
    let day_angle = climate.day_phase.rem_euclid(1.0) as f64 * std::f64::consts::TAU;
    let day_heat_fp = (day_angle.sin() * 2_000.0).round() as i64;
    let pressure_wave_fp =
        sin_fp((tick as i64 * 900) + (climate.year_phase * 360_000.0) as i64) as i64;
    let mut cells = Vec::with_capacity(n as usize);

    for region_id in 0..n {
        let lat_fp = lat_from_region(region_id, n);
        let hemisphere = if lat_fp >= 0 { 1 } else { -1 };
        let latitude_cool_fp = (lat_fp.abs() * 7_000) / MAX_LAT_FP;
        let baseline_fp = temp_baseline_for_lat(lat_fp);
        let season_fp = season_offset(season, lat_fp);
        let wave_fp = pressure_wave_fp * (1 + hemisphere) / 2 + pressure_wave_fp.abs() / 3;
        let temp_c_fp = (baseline_fp + season_fp + day_heat_fp - latitude_cool_fp + wave_fp / 2)
            .clamp(-60_000, 55_000) as i32;

        let moisture_fp = if matches!(season, SeasonKind::Spring | SeasonKind::Autumn) {
            900
        } else if matches!(season, SeasonKind::Winter) {
            700
        } else {
            500
        } + ((tick as i64 + region_id as i64 * 97) % 700) * 2;

        let storm_seed_fp = (day_heat_fp.abs() + pressure_wave_fp.abs()) / 2;
        let storm_intensity_fp = ((storm_seed_fp / 2) + moisture_fp - (temp_c_fp as i64 / 4).abs())
            .clamp(0, 10_000) as i32;
        let precip_mm_fp = precipitation_from_temp_and_moisture(
            temp_c_fp as i64,
            moisture_fp,
            storm_intensity_fp as i64,
        )
        .clamp(0, 20_000) as i32;
        let kind = weather_kind_from(
            temp_c_fp as i64,
            precip_mm_fp as i64,
            storm_intensity_fp as i64,
        );

        cells.push(WeatherCell {
            region_id,
            latitude_fp: lat_fp as i32,
            season,
            kind,
            temp_c_fp,
            precip_mm_fp,
            storm_intensity_fp,
        });
    }

    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    fn climate(year_phase: f32, day_phase: f32) -> Climate {
        Climate {
            tick: 0,
            day_phase,
            year_phase,
            moon_phase: 0.0,
            tide_offset: 0.0,
        }
    }

    #[test]
    fn seasons_follow_year_phase() {
        assert_eq!(season_from_year_phase(0.00), SeasonKind::Spring);
        assert_eq!(season_from_year_phase(0.30), SeasonKind::Summer);
        assert_eq!(season_from_year_phase(0.60), SeasonKind::Autumn);
        assert_eq!(season_from_year_phase(0.90), SeasonKind::Winter);
    }

    #[test]
    fn summer_is_warmer_than_winter_at_equator() {
        let summer = compute_weather(&climate(0.30, 0.5), 10_000, 9);
        let winter = compute_weather(&climate(0.90, 0.5), 10_000, 9);
        let center = 4;

        assert!(summer[center].temp_c_fp > winter[center].temp_c_fp);
    }

    #[test]
    fn warm_cells_rain_and_cold_cells_snow() {
        let rain_cells = compute_weather(&climate(0.30, 0.5), 1_234, 8);
        let snow_cells = compute_weather(&climate(0.90, 0.5), 1_234, 8);

        assert!(rain_cells.iter().any(|cell| cell.kind == WeatherKind::Rain));
        assert!(snow_cells.iter().any(|cell| cell.kind == WeatherKind::Snow));
    }

    #[test]
    fn storms_appear_with_time_variation() {
        let climate = climate(0.45, 0.9);
        let stormy = compute_weather(&climate, 99_999, 16);

        assert!(stormy.iter().any(|cell| cell.kind == WeatherKind::Storm));
        assert!(stormy.iter().all(|cell| cell.precip_mm_fp >= 0));
    }

    #[test]
    fn deterministic_for_same_inputs() {
        let climate = climate(0.2, 0.7);
        let a = compute_weather(&climate, 42, 12);
        let b = compute_weather(&climate, 42, 12);

        assert_eq!(a, b);
    }
}
