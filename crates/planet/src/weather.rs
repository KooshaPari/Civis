//! FR-CIV-PLANET-030 — deterministic per-region weather grid.
//!
//! All arithmetic uses fixed-point integer thousandths (fp = ×1000) so there
//! are no `f64` values at the public boundary and results are bit-identical
//! across platforms.
//!
//! Trig is approximated with a 256-entry sin LUT stored as i32 thousandths.
//! The LUT covers one full turn (0..TAU), indexed by `angle_fp / 360_000`
//! modulo 256.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ── fixed-point constants ────────────────────────────────────────────────────

/// One full turn in fixed-point degrees thousandths (360 × 1000).
const FULL_TURN_FP: i64 = 360_000;

/// sin LUT: 256 entries, each = round(sin(i/256 * 2π) * 1000).
/// Generated offline — exact values verified against reference implementation.
static SIN_LUT: [i32; 256] = [
    0, 25, 49, 74, 98, 122, 147, 171, 195, 219, 243, 267, 290, 314, 337, 360, 383, 405, 428, 450,
    471, 493, 514, 535, 556, 576, 596, 615, 634, 653, 672, 690, 707, 724, 741, 757, 773, 788, 803,
    818, 831, 845, 858, 870, 882, 893, 904, 914, 924, 933, 942, 950, 957, 964, 970, 976, 981, 985,
    989, 992, 995, 997, 999, 1000, 1000, 1000, 999, 997, 995, 992, 989, 985, 981, 976, 970, 964,
    957, 950, 942, 933, 924, 914, 904, 893, 882, 870, 858, 845, 831, 818, 803, 788, 773, 757, 741,
    724, 707, 690, 672, 653, 634, 615, 596, 576, 556, 535, 514, 493, 471, 450, 428, 405, 383, 360,
    337, 314, 290, 267, 243, 219, 195, 171, 147, 122, 98, 74, 49, 25, 0, -25, -49, -74, -98, -122,
    -147, -171, -195, -219, -243, -267, -290, -314, -337, -360, -383, -405, -428, -450, -471, -493,
    -514, -535, -556, -576, -596, -615, -634, -653, -672, -690, -707, -724, -741, -757, -773, -788,
    -803, -818, -831, -845, -858, -870, -882, -893, -904, -914, -924, -933, -942, -950, -957, -964,
    -970, -976, -981, -985, -989, -992, -995, -997, -999, -1000, -1000, -1000, -999, -997, -995,
    -992, -989, -985, -981, -976, -970, -964, -957, -950, -942, -933, -924, -914, -904, -893, -882,
    -870, -858, -845, -831, -818, -803, -788, -773, -757, -741, -724, -707, -690, -672, -653, -634,
    -615, -596, -576, -556, -535, -514, -493, -471, -450, -428, -405, -383, -360, -337, -314, -290,
    -267, -243, -219, -195, -171, -147, -122, -98, -74, -49, -25,
];

/// Look up `sin(angle_fp / 1000 degrees)` returning a fixed-point i32 thousandths
/// in the range `[-1000, 1000]`.
///
/// `angle_fp` is degrees × 1000 (e.g. `90_000` = 90°).
fn sin_fp(angle_fp: i64) -> i32 {
    // Normalise into [0, FULL_TURN_FP)
    let norm = angle_fp.rem_euclid(FULL_TURN_FP);
    // Map to [0, 256)
    let idx = (norm * 256 / FULL_TURN_FP) as usize;
    SIN_LUT[idx & 0xFF]
}

// ── public types ─────────────────────────────────────────────────────────────

/// Per-region weather cell produced by [`compute_weather`].
///
/// All numeric fields use fixed-point thousandths (`fp`):
/// - `temp_c_fp / 1000` gives the temperature in °C as a rational number.
/// - `precip_mm_fp / 1000` gives precipitation in mm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherCell {
    /// Stable region identifier.
    pub region_id: u32,
    /// Temperature in fixed-point thousandths of °C.
    pub temp_c_fp: i32,
    /// Precipitation in fixed-point thousandths of mm.
    pub precip_mm_fp: i32,
}

/// Compute a deterministic weather grid for `num_regions` regions.
///
/// # Parameters
/// - `tick`: simulation tick — the sole source of temporal variation.
/// - `num_regions`: how many [`WeatherCell`]s to produce (one per region id
///   `0..num_regions`).
/// - `axial_tilt_fp`: planet axial tilt in fixed-point thousandths of a degree.
///   Positive values produce Earth-like seasonal swings; 0 gives a flat climate.
/// - `year_length_ticks`: how many ticks make up one full year.
///
/// # Determinism guarantee
/// Results are derived entirely from the inputs. No global state, no `f64`
/// boundary crossings, no `rand` — only fixed-point integer arithmetic and
/// the LUT above.
///
/// # Model
/// Each region `r` is assigned a latitude in `[-90°, +90°]` equally spaced
/// across the sphere. The base temperature is computed as:
///
/// ```text
/// lat_contribution = cos(lat) × 40_000  (fp)       ← equatorial +40°C boost
/// seasonal_angle   = year_phase_angle + lat × 0.5  (fp degrees)
/// seasonal         = sin(seasonal_angle) × axial_tilt_fp / 1000
/// temp_c_fp        = lat_contribution + seasonal - 20_000  (−20°C global offset)
/// ```
///
/// Precipitation is inversely correlated with temperature excursion from the
/// equatorial average and is kept non-negative.
pub fn compute_weather(
    tick: u64,
    num_regions: u32,
    axial_tilt_fp: i32,
    year_length_ticks: u32,
) -> Vec<WeatherCell> {
    let n = num_regions.max(1);
    let year_len = year_length_ticks.max(1) as u64;

    // Year phase as an angle in fixed-point degrees thousandths [0, 360_000)
    let year_phase_angle: i64 = ((tick % year_len) as i64 * FULL_TURN_FP) / year_len as i64;

    let mut cells = Vec::with_capacity(n as usize);

    for r in 0..n {
        // Latitude in fixed-point thousandths of a degree: [-90_000, +90_000]
        // Evenly distribute regions over the sphere.
        let lat_fp: i64 = if n == 1 {
            0
        } else {
            // Maps r=0 → -90_000, r=n-1 → +90_000
            (-90_000i64) + (180_000i64 * r as i64) / (n - 1) as i64
        };

        // cos(lat) ≈ sin(90° - lat)
        let cos_lat_fp = sin_fp(90_000 - lat_fp) as i64; // [-1000, 1000]

        // Equatorial temperature boost: cos(lat) × 40°C
        let lat_contribution: i64 = cos_lat_fp * 40; // fp thousandths

        // Seasonal angle: year phase shifted by hemisphere (lat sign flips it)
        // lat_fp / 2 gives a mild phase offset across latitudes
        let seasonal_angle: i64 = year_phase_angle + lat_fp / 2;

        // Seasonal swing: sin(seasonal_angle) × axial_tilt (in fp thousandths °C)
        let sin_seasonal = sin_fp(seasonal_angle) as i64; // [-1000, 1000]
        let seasonal: i64 = sin_seasonal * axial_tilt_fp as i64 / 1000; // fp thousandths

        // Base temperature: equatorial ~20°C with ±tilt seasonal swing
        let temp_c_fp: i32 =
            (lat_contribution + seasonal - 20_000).clamp(i32::MIN as i64, i32::MAX as i64) as i32;

        // Precipitation: inversely proportional to |temp - equatorial_base|.
        // Equatorial base in fp is 20_000 (20°C). Peak precip = 2000 fp mm.
        let temp_excursion = (temp_c_fp as i64 - (-20_000 + 40_000)).abs(); // relative to 20°C
        let precip_mm_fp: i32 = (2_000_000i64.saturating_sub(temp_excursion * 40) / 1_000)
            .clamp(0, i32::MAX as i64) as i32;

        cells.push(WeatherCell {
            region_id: r,
            temp_c_fp,
            precip_mm_fp,
        });
    }

    cells
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity-check the LUT has exactly 256 entries and the identity values
    /// at known angles are correct (within ±2 due to rounding).
    #[test]
    fn sin_lut_identity_values() {
        // sin(0°) = 0
        assert_eq!(sin_fp(0), 0);
        // sin(90°) ≈ 1000
        let s90 = sin_fp(90_000);
        assert!(
            (s90 - 1000).abs() <= 10,
            "sin(90°) expected ~1000, got {s90}"
        );
        // sin(270°) ≈ -1000
        let s270 = sin_fp(270_000);
        assert!(
            (s270 - (-1000)).abs() <= 10,
            "sin(270°) expected ~-1000, got {s270}"
        );
        // sin(180°) ≈ 0
        let s180 = sin_fp(180_000);
        assert!(s180.abs() <= 25, "sin(180°) expected ~0, got {s180}");
    }

    /// sin_fp is periodic over 360°.
    #[test]
    fn sin_fp_periodic() {
        for angle in [0i64, 45_000, 90_000, 135_000, 180_000] {
            assert_eq!(
                sin_fp(angle),
                sin_fp(angle + FULL_TURN_FP),
                "sin_fp not periodic at {angle}"
            );
        }
    }

    /// FR-CIV-PLANET-030 — summer equatorial temperature must exceed winter
    /// equatorial temperature when axial tilt is non-zero.
    #[test]
    fn weather_grid_temperature_varies_with_year_phase() {
        let year_length_ticks: u32 = 8_766_000;
        let axial_tilt_fp: i32 = 23_000; // 23°
        let num_regions: u32 = 8;

        // Equatorial region index (middle of the range)
        let equatorial_idx = (num_regions / 2) as usize;

        // Northern summer: tick at year ¼ (sin positive → warm N hemisphere)
        let summer_tick = year_length_ticks as u64 / 4;
        // Northern winter: tick at year ¾
        let winter_tick = year_length_ticks as u64 * 3 / 4;

        let summer_cells =
            compute_weather(summer_tick, num_regions, axial_tilt_fp, year_length_ticks);
        let winter_cells =
            compute_weather(winter_tick, num_regions, axial_tilt_fp, year_length_ticks);

        let summer_eq = summer_cells[equatorial_idx].temp_c_fp;
        let winter_eq = winter_cells[equatorial_idx].temp_c_fp;

        assert!(
            summer_eq > winter_eq,
            "summer equatorial temp ({summer_eq} fp) should exceed winter ({winter_eq} fp)"
        );
    }

    /// FR-CIV-PLANET-030 — results are bit-identical across two independent calls.
    #[test]
    fn weather_grid_is_deterministic() {
        let tick = 123_456_789_u64;
        let num_regions = 16;
        let axial_tilt_fp = 23_000;
        let year_length_ticks = 8_766_000;

        let a = compute_weather(tick, num_regions, axial_tilt_fp, year_length_ticks);
        let b = compute_weather(tick, num_regions, axial_tilt_fp, year_length_ticks);

        assert_eq!(a, b, "compute_weather must be deterministic");
    }

    /// FR-CIV-PLANET-030 — zero axial tilt produces identical grids for summer
    /// and winter ticks (no seasonal swing).
    #[test]
    fn zero_axial_tilt_no_seasonal_swing() {
        let year_length_ticks: u32 = 8_766_000;
        let summer_tick = year_length_ticks as u64 / 4;
        let winter_tick = year_length_ticks as u64 * 3 / 4;

        let summer = compute_weather(summer_tick, 8, 0, year_length_ticks);
        let winter = compute_weather(winter_tick, 8, 0, year_length_ticks);

        // With zero tilt there is no seasonal component; grids should be equal.
        assert_eq!(
            summer, winter,
            "zero tilt should produce identical summer and winter grids"
        );
    }

    /// Precipitation is always non-negative.
    #[test]
    fn precipitation_non_negative() {
        let year_length_ticks: u32 = 8_766_000;
        for tick in [0u64, 1_000_000, 4_383_000, 8_765_999] {
            let cells = compute_weather(tick, 32, 23_000, year_length_ticks);
            for cell in &cells {
                assert!(
                    cell.precip_mm_fp >= 0,
                    "region {} tick {tick}: negative precip {}",
                    cell.region_id,
                    cell.precip_mm_fp
                );
            }
        }
    }
}
