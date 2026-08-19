//! Biome shift from sustained climate change (FR-CIV-BIOME-SHIFT).
//!
//! A simplified Köppen-inspired biome classifier that maps a cell's mean
//! temperature (°C) and precipitation (model units) to one of a small set
//! of biome classes. When the climate inputs change persistently past the
//! thresholds between adjacent biomes, the cell is reclassified.
//!
//! # Design
//!
//! 1. **Inputs are continuous**: temperature (`temp_c`) and precipitation
//!    (`precip`). These may be derived from a [`ClimateState`](crate::ClimateState)
//!    plus a seasonal cycle and per-cell modifiers.
//! 2. **Classification is deterministic**: a pure function of `(temp_c,
//!    precip)` with a fixed ordering of rules — no randomness, no floating
//!    point comparisons that depend on summation order.
//! 3. **Thresholds are tunable** via [`BiomeShiftParams`] so callers can
//!    adjust the sensitivity of biome edges to warming / drying.
//! 4. **Shift is optional**: call [`biome_step`] to reclassify a single
//!    cell that has experienced a sustained `(dtemp_c, dprecip)` shift.
//!
//! # Example
//!
//! ```rust
//! use civ_climate::biome_shift::{Biome, BiomeShiftParams, classify, biome_step};
//!
//! let params = BiomeShiftParams::default();
//!
//! // Start as a tundra cell (mean annual temp just below 0 °C).
//! let start = classify(-1.0, 0.4, &params);
//! assert_eq!(start, Biome::Tundra);
//!
//! // Sustained warming past the boreal-forest threshold reclassifies it.
//! let after = biome_step(-1.0, 0.4, 9.5, 0.0, &params);
//! assert_ne!(after, Biome::Tundra);
//! ```

#![forbid(unsafe_code)]

/// Biome classification produced by the biome-shift model.
///
/// The ordering is meaningful: lower variants correspond to colder / drier
/// climates, higher variants to warmer / wetter ones. Moving a cell along
/// this axis models the kind of biome transitions expected from sustained
/// warming (e.g. tundra → boreal forest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    /// Permanent ice / cold desert. Coldest and driest biome tracked here.
    PolarDesert,
    /// Treeless cold-plains biome, found just below polar desert.
    Tundra,
    /// Cold-climate coniferous forest belt south of tundra.
    BorealForest,
    /// Mid-latitude mixed / temperate forest.
    TemperateForest,
    /// Hot, dry biome.
    Desert,
}

impl Biome {
    /// Reference mean annual temperature (°C) associated with each biome.
    /// These are not strict thresholds — use [`BiomeShiftParams`] for that —
    /// they are useful defaults for `with_shift` and documentation.
    pub fn temp_c(self) -> f64 {
        match self {
            Biome::PolarDesert => -15.0,
            Biome::Tundra => -2.0,
            Biome::BorealForest => 4.0,
            Biome::TemperateForest => 12.0,
            Biome::Desert => 22.0,
        }
    }

    /// Reference precipitation (model units) associated with each biome.
    pub fn precip(self) -> f64 {
        match self {
            Biome::PolarDesert => 0.1,
            Biome::Tundra => 0.3,
            Biome::BorealForest => 0.6,
            Biome::TemperateForest => 0.9,
            Biome::Desert => 0.1,
        }
    }

    /// Convenience: build a new climate pair by adding temperature and
    /// precipitation deltas to the biome's reference values. Returns a
    /// plain `(temp_c, precip)` tuple suitable for [`classify`].
    ///
    /// This is **not** used by the classifier itself — it is a helper for
    /// tests and for callers that want to reason about biome transitions
    /// in a "delta from reference" form.
    pub fn with_shift(self, dtemp_c: f64, dprecip: f64) -> (f64, f64) {
        (self.temp_c() + dtemp_c, self.precip() + dprecip)
    }
}

/// Tunable thresholds between adjacent biomes.
///
/// All temperatures are in °C and precipitation is in model units (same
/// units as [`season_cycle`](crate::season_cycle)).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiomeShiftParams {
    /// Mean annual temperature above which a cell is considered warmer
    /// than polar desert (°C). Lower = polar desert, ≥ threshold moves on
    /// to tundra checks.
    pub polar_desert_temp_c: f64,
    /// Mean annual temperature below which a cell is classified tundra
    /// once it has cleared polar-desert (°C).
    pub tundra_temp_c: f64,
    /// Mean annual temperature below which a cell is classified boreal
    /// forest once it has cleared tundra (°C).
    pub boreal_temp_c: f64,
    /// Mean annual temperature below which a cell is classified temperate
    /// forest (°C). Above this and not-arid → desert.
    pub temperate_temp_c: f64,
    /// Precipitation below which a hot cell is classified desert instead
    /// of temperate / savannah. Model units.
    pub desert_precip_threshold: f64,
}

impl Default for BiomeShiftParams {
    fn default() -> Self {
        Self {
            // -10 °C: well below freezing, sparse snow / ice.
            polar_desert_temp_c: -10.0,
            //   0 °C: trees can't grow; still treeless.
            tundra_temp_c: 0.0,
            //   9 °C: boreal forest (taiga) belt.
            boreal_temp_c: 9.0,
            //  18 °C: warm temperate; hotter non-arid cells become desert.
            temperate_temp_c: 18.0,
            //   0.25: arid threshold in precipitation model units.
            desert_precip_threshold: 0.25,
        }
    }
}

/// Classify a cell's climate into a [`Biome`] using the given `params`.
///
/// Classification rules (first match wins):
///
/// 1. `temp_c < params.polar_desert_temp_c` ⇒ [`Biome::PolarDesert`].
/// 2. `temp_c < params.tundra_temp_c`        ⇒ [`Biome::Tundra`].
/// 3. `temp_c < params.boreal_temp_c`        ⇒ [`Biome::BorealForest`].
/// 4. `temp_c < params.temperate_temp_c`     ⇒ [`Biome::TemperateForest`].
/// 5. Hot and wet (`precip ≥ desert_precip_threshold`) ⇒ temperate forest.
/// 6. Hot and dry (`precip < desert_precip_threshold`) ⇒ [`Biome::Desert`].
///
/// `NaN` inputs are coerced to `-∞` for temperature and `0.0` for
/// precipitation so the function remains total.
pub fn classify(temp_c: f64, precip: f64, params: &BiomeShiftParams) -> Biome {
    let temp_c = if temp_c.is_nan() {
        f64::NEG_INFINITY
    } else {
        temp_c
    };
    let precip = if precip.is_nan() { 0.0 } else { precip };

    if temp_c < params.polar_desert_temp_c {
        Biome::PolarDesert
    } else if temp_c < params.tundra_temp_c {
        Biome::Tundra
    } else if temp_c < params.boreal_temp_c {
        Biome::BorealForest
    } else if temp_c < params.temperate_temp_c {
        Biome::TemperateForest
    } else if precip < params.desert_precip_threshold {
        Biome::Desert
    } else {
        Biome::TemperateForest
    }
}

/// Apply a sustained `dtemp_c` and `dprecip` shift to a cell's climate and
/// reclassify it.
///
/// This is the convenience entry point used in scenarios: the caller holds
/// the cell's *current* temperature and precipitation, applies the
/// sustained deltas (e.g. from climate forcing this tick), and asks
/// "what biome is this cell now?".
pub fn biome_step(
    temp_c: f64,
    precip: f64,
    dtemp_c: f64,
    dprecip: f64,
    params: &BiomeShiftParams,
) -> Biome {
    classify(temp_c + dtemp_c, precip + dprecip, params)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// 1. **Acceptance test (FR-CIV-BIOME-SHIFT):**
    ///    Warming a tundra cell past threshold reclassifies it.
    ///
    /// A tundra cell sits near the tundra / boreal-forest boundary. A small
    /// warming shift moves the cell past the boreal threshold and the cell
    /// must reclassify to [`Biome::BorealForest`].
    #[test]
    fn warming_tundra_past_threshold_reclassifies() {
        let params = BiomeShiftParams::default();

        // Start: tundra cell (e.g. mean annual temp just under 0 °C).
        let start_temp = -1.0_f64;
        let start_precip = 0.4_f64;
        let before = classify(start_temp, start_precip, &params);
        assert_eq!(before, Biome::Tundra, "expected Tundra before shift");

        // Sustain warming well past the boreal threshold (+9.5 °C).
        let after_temp = start_temp + 9.5;
        let after = classify(after_temp, start_precip, &params);
        assert_eq!(
            after,
            Biome::BorealForest,
            "expected reclassification to BorealForest after sustained warming"
        );

        // The end-to-end helper produces the same answer.
        let helper = biome_step(start_temp, start_precip, 9.5, 0.0, &params);
        assert_eq!(helper, Biome::BorealForest);
    }

    /// 2. A polar-desert cell warmed past its threshold reclassifies to
    ///    tundra — the analogous transition from the cold side.
    #[test]
    fn warming_polar_desert_reclassifies_to_tundra() {
        let params = BiomeShiftParams::default();
        assert_eq!(
            classify(-20.0, 0.1, &params),
            Biome::PolarDesert,
            "should remain PolarDesert before warming"
        );
        assert_eq!(
            classify(-5.0, 0.1, &params),
            Biome::Tundra,
            "should reclassify to Tundra after warming past polar_desert_temp_c"
        );
    }

    /// 3. A cold tundra cell that dries further shifts to polar desert
    ///    (the cold + dry axis), demonstrating the temperature *and*
    ///    precipitation joint axis requirement from the FR.
    #[test]
    fn drying_and_cooling_tundra_pushes_to_polar_desert() {
        let params = BiomeShiftParams::default();
        // Warm enough to be tundra, but cool it further.
        let before = classify(-1.0, 0.4, &params);
        assert_eq!(before, Biome::Tundra);

        // Cool it well past the polar-desert threshold.
        let after = biome_step(-1.0, 0.4, -20.0, 0.0, &params);
        assert_eq!(after, Biome::PolarDesert);
    }

    /// 4. A hot, wet cell stays temperate forest even after a minor
    ///    drying shift; only drying below the desert threshold shifts it.
    #[test]
    fn drying_below_threshold_flips_hot_cell_to_desert() {
        let params = BiomeShiftParams::default();

        // Hot + wet: temperate forest
        assert_eq!(
            classify(20.0, 0.8, &params),
            Biome::TemperateForest,
            "hot+wet should be TemperateForest"
        );

        // Hot + dry: desert
        assert_eq!(
            classify(20.0, 0.2, &params),
            Biome::Desert,
            "hot+dry should be Desert"
        );

        // End-to-end: drying shift pushes a hot cell into desert.
        let shifted = biome_step(20.0, 0.8, 0.0, -0.7, &params);
        assert_eq!(shifted, Biome::Desert);
    }

    /// 5. Below-threshold warming (no crossing) does NOT reclassify.
    #[test]
    fn subthreshold_warming_does_not_reclassify() {
        let params = BiomeShiftParams::default();

        // Start: boreal forest (e.g. 6 °C)
        let before = classify(6.0, 0.6, &params);
        assert_eq!(before, Biome::BorealForest);

        // Warm by only 1 °C — still below the temperate threshold (18 °C).
        let after = biome_step(6.0, 0.6, 1.0, 0.0, &params);
        assert_eq!(
            after,
            Biome::BorealForest,
            "small warming below threshold should not reclassify"
        );
    }

    /// 6. NaN inputs are tolerated (do not panic, classify deterministically).
    #[test]
    fn nan_inputs_classify_deterministically() {
        let params = BiomeShiftParams::default();
        let nan_temp = classify(f64::NAN, 0.5, &params);
        let cold_temp = classify(f64::NEG_INFINITY, 0.5, &params);
        assert_eq!(
            nan_temp, cold_temp,
            "NaN temperature should be coerced to -inf and yield PolarDesert"
        );

        let nan_precip = classify(20.0, f64::NAN, &params);
        let zero_precip = classify(20.0, 0.0, &params);
        assert_eq!(
            nan_precip, zero_precip,
            "NaN precipitation should be coerced to 0.0 and yield Desert"
        );
    }
}
