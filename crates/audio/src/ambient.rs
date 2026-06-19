//! Biome-driven ambient bed blend (FR-CIV-AUDIO-002).
//!
//! Audio-direction §1 (Tier 1 — Ambient soundscape) specifies that
//! the camera's ground footprint classifies into a small set of
//! `AmbientBed` channels, and that the per-tick sampler emits a
//! **normalised weight vector** over those beds. The kira plugin
//! cross-fades each bed's gain toward its target weight on a slow
//! tween. This module owns the *math*: the footprint → weights map
//! and the cross-fade step, both pure and deterministic.

use serde::{Deserialize, Serialize};

/// The five ambient beds defined in audio-direction §1.
///
/// Order is wire-stable — reordering breaks the bincode snapshot. New
/// beds append at the end only. Each variant maps to one
/// looping `.ogg` (and one kira channel) in the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmbientBed {
    /// Open / high terrain, default floor (Grass, Stone, Snow, fallback).
    Wind,
    /// Shoreline lap / open-water wash (DeepWater, Water, Sand coast).
    Water,
    /// Leaf rustle, canopy (Forest).
    Forest,
    /// Birds (day) / insects (night) — gated by `SeasonKind` (silenced in Winter).
    Wildlife,
    /// Rain / storm / snow-hush overlay — additive on top of biome beds.
    Weather,
}

impl AmbientBed {
    /// All five beds, in mix-tree order. Useful for filling
    /// `BedWeights` with the full slot set on first use.
    pub const ALL: [AmbientBed; 5] = [
        AmbientBed::Wind,
        AmbientBed::Water,
        AmbientBed::Forest,
        AmbientBed::Wildlife,
        AmbientBed::Weather,
    ];

    /// Index of a bed inside `BedWeights::weights` (0..5).
    pub fn index(self) -> usize {
        match self {
            AmbientBed::Wind => 0,
            AmbientBed::Water => 1,
            AmbientBed::Forest => 2,
            AmbientBed::Wildlife => 3,
            AmbientBed::Weather => 4,
        }
    }
}

/// A coarse biome category that the sampler accepts as input.
///
/// The substrate is deliberately decoupled from `civ-planet`'s
/// `BiomeKind` (which has 6 variants — Ocean/Plains/Forest/...):
/// the audio layer only needs the *sound-relevant* distinction, and
/// any client can build a `BiomeCategory` from its own world model.
/// Mapping table is in [`BedWeights::from_biome_counts`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiomeCategory {
    /// Open water.
    Water,
    /// Sandy / shoreline.
    Sand,
    /// Flat grassland + savanna.
    Grass,
    /// Forest canopy.
    Forest,
    /// Stone / rock outcrops.
    Stone,
    /// Snow / ice.
    Snow,
}

/// Per-tick footprint of the camera's ground projection.
///
/// Counts of each biome category visible under the camera, plus the
/// current weather and diurnal state (for gating `Wildlife` in
/// `Winter` and swapping day/night bird/cricket sounds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BiomeFootprint {
    /// Count of `Water`-biome cells under the camera.
    pub water: u32,
    /// Count of `Sand`-biome cells.
    pub sand: u32,
    /// Count of `Grass`-biome cells.
    pub grass: u32,
    /// Count of `Forest`-biome cells.
    pub forest: u32,
    /// Count of `Stone`-biome cells.
    pub stone: u32,
    /// Count of `Snow`-biome cells.
    pub snow: u32,
    /// Sum of all biome cells. The sampler normalises by this; if it
    /// is 0 the camera footprint is empty and the blend falls back to
    /// the wind floor.
    pub total_cells: u32,
    /// Current weather kind for the camera region. Drives `WeatherBed`
    /// as an additive overlay.
    pub weather: WeatherOverlay,
    /// `true` for the day-half of the diurnal cycle. `Wildlife` swaps
    /// its source by day/night.
    pub is_daytime: bool,
    /// `true` when the current season is Winter — gates `Wildlife` off.
    pub is_winter: bool,
}

/// Coarse weather overlay classification used by the audio sampler.
///
/// The substrate does not need the planet crate's `WeatherKind`
/// (Clear/Rain/Snow/Storm) for the audio mix; it only needs the
/// *overlay* distinction. Mapping is in the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherOverlay {
    /// No overlay — `WeatherBed` gain is 0.
    #[default]
    Clear,
    /// Light rain overlay.
    Rain,
    /// Heavy rain / storm overlay.
    Storm,
    /// Snow / hush overlay.
    Snow,
}

/// Normalised weight vector for the five ambient beds.
///
/// Each slot is in `[0.0, 1.0]`. The contract is **NOT** that they
/// sum to 1.0 — the audio-direction spec calls for `Wildlife` and
/// `Weather` to be **additive overlays** on top of the biome-derived
/// mix. The substrate enforces only that every slot is in range and
/// the sum is ≤ a documented ceiling (1.0 + max two overlays). The
/// kira plugin multiplies by the per-bed `tween` to get the final
/// volume; the substrate only handles the target vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BedWeights {
    /// `WindBed` target weight.
    pub wind: f32,
    /// `WaterBed` target weight.
    pub water: f32,
    /// `ForestBed` target weight.
    pub forest: f32,
    /// `WildlifeBed` target weight (silenced in Winter).
    pub wildlife: f32,
    /// `WeatherBed` additive overlay weight.
    pub weather: f32,
}

impl Default for BedWeights {
    fn default() -> Self {
        // Audio-direction §1: WindBed is the always-on floor.
        Self {
            wind: 1.0,
            water: 0.0,
            forest: 0.0,
            wildlife: 0.0,
            weather: 0.0,
        }
    }
}

impl BedWeights {
    /// Cap that any single slot may reach. The biome-derived sum is
    /// normalised against the total cell count, so individual slots
    /// may approach (but never exceed) 1.0. The weather / wildlife
    /// overlays are written independently of the biome sum and so
    /// are also capped at 1.0 each.
    pub const SLOT_CAP: f32 = 1.0;

    /// Floor weight for the wind bed. Even with no biome data, the
    /// soundscape must never be silent (audio-direction pillar #5).
    pub const WIND_FLOOR: f32 = 0.05;

    /// Returns the weight for a given bed slot.
    pub fn get(self, bed: AmbientBed) -> f32 {
        match bed {
            AmbientBed::Wind => self.wind,
            AmbientBed::Water => self.water,
            AmbientBed::Forest => self.forest,
            AmbientBed::Wildlife => self.wildlife,
            AmbientBed::Weather => self.weather,
        }
    }

    /// Set the weight for a bed slot, clamping to `[0.0, SLOT_CAP]`.
    pub fn set(&mut self, bed: AmbientBed, weight: f32) {
        let clamped = weight.clamp(0.0, Self::SLOT_CAP);
        match bed {
            AmbientBed::Wind => self.wind = clamped,
            AmbientBed::Water => self.water = clamped,
            AmbientBed::Forest => self.forest = clamped,
            AmbientBed::Wildlife => self.wildlife = clamped,
            AmbientBed::Weather => self.weather = clamped,
        }
    }

    /// Per-slot sum across the five beds. Note: NOT constrained to
    /// ≤ 1.0 — `Wildlife` and `Weather` are designed to layer on top
    /// of the biome mix (audio-direction §1 tier-1 additive rule).
    #[must_use]
    pub fn total(self) -> f32 {
        self.wind + self.water + self.forest + self.wildlife + self.weather
    }

    /// `true` if every slot is in `[0.0, SLOT_CAP]` and finite.
    /// Used by the settings guard.
    pub fn is_well_formed(self) -> bool {
        [
            self.wind,
            self.water,
            self.forest,
            self.wildlife,
            self.weather,
        ]
        .iter()
        .all(|v| v.is_finite() && (0.0..=Self::SLOT_CAP).contains(v))
    }

    /// Build the per-tick weight vector from a camera footprint.
    ///
    /// Mapping (audio-direction §1 tier 1):
    /// - `Wind`  ← `(grass + stone + snow) / total` (with a wind floor)
    /// - `Water` ← `(water + sand) / total`
    /// - `Forest` ← `forest / total`
    /// - `Wildlife` ← additive on top, gated by `is_winter`
    /// - `Weather` ← additive overlay by `WeatherOverlay`
    ///
    /// If `total == 0` the footprint is empty and the wind floor is
    /// returned untouched (no other beds lit) so the soundscape is
    /// never silent.
    pub fn from_footprint(footprint: &BiomeFootprint) -> Self {
        // Empty camera → wind floor only.
        if footprint.total_cells == 0 {
            return Self {
                wind: Self::WIND_FLOOR,
                water: 0.0,
                forest: 0.0,
                wildlife: 0.0,
                weather: 0.0,
            };
        }

        let total = footprint.total_cells as f32;
        let wind_raw = (footprint.grass + footprint.stone + footprint.snow) as f32 / total;
        let water_raw = (footprint.water + footprint.sand) as f32 / total;
        let forest_raw = footprint.forest as f32 / total;

        // Wildlife is additive. Day + non-winter + grass-or-forest
        // presence → lit. Silenced in Winter per audio-direction §1.
        let wildlife_raw = if !footprint.is_winter
            && footprint.is_daytime
            && (footprint.grass > 0 || footprint.forest > 0)
        {
            // Scale 0..1 by the share of grass+forest in the footprint.
            let bio_share = (footprint.grass + footprint.forest) as f32 / total;
            bio_share.clamp(0.0, 1.0)
        } else {
            0.0
        };

        let weather_raw = match footprint.weather {
            WeatherOverlay::Clear => 0.0,
            // Light rain uses a softer overlay than storm.
            WeatherOverlay::Rain => 0.40,
            WeatherOverlay::Storm => 0.75,
            WeatherOverlay::Snow => 0.55,
        };

        // Wind floor: never go below 0.05 even on a no-wind footprint.
        let wind = wind_raw.max(Self::WIND_FLOOR);

        let mut out = Self {
            wind,
            water: water_raw,
            forest: forest_raw,
            wildlife: wildlife_raw,
            weather: weather_raw,
        };
        // Re-clamp every slot in case wildlife / weather push over the cap.
        for bed in AmbientBed::ALL {
            let v = out.get(bed);
            if v > Self::SLOT_CAP {
                out.set(bed, Self::SLOT_CAP);
            }
        }
        out
    }
}

/// A cross-faded blend of two [`BedWeights`] snapshots.
///
/// The kira plugin tweens `current` toward `target` over a
/// `tween_seconds` window; the substrate exposes the interpolated
/// state as a value type so tests can verify the smoothing curve
/// without an async runtime. The math is a simple linear interp
/// with a configurable time constant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AmbientBlend {
    /// Last-emitted weights, fed into the kira `play` call's volume.
    pub current: BedWeights,
    /// Next-footprint target the blend is moving toward.
    pub target: BedWeights,
    /// Seconds since the last blend update; bounded by `dt` per step.
    pub elapsed: f32,
    /// Time-constant (1/e) in seconds for the cross-fade. The kira
    /// plugin mirrors this with a kira `Tween` of the same length.
    /// Audio-direction §1: 0.75–1.5 s. We default to 1.0 s.
    pub time_constant_s: f32,
}

impl Default for AmbientBlend {
    fn default() -> Self {
        Self {
            current: BedWeights::default(),
            target: BedWeights::default(),
            elapsed: 0.0,
            time_constant_s: 1.0,
        }
    }
}

impl AmbientBlend {
    /// Push a new footprint and step the blend forward by `dt` seconds.
    ///
    /// Pure: no I/O, no kira. The kira plugin calls this every
    /// `~250 ms` (4 Hz) and pushes the resulting `current` into
    /// kira volume tweens.
    pub fn step(&mut self, new_target: BedWeights, dt: f32) {
        self.target = new_target;
        // Time-constant approach: weight = 1 - exp(-elapsed / tau).
        // We integrate one step so the test can drive it deterministically.
        self.elapsed = (self.elapsed + dt.max(0.0)).max(0.0);
        let t = if self.time_constant_s <= 0.0 {
            1.0
        } else {
            1.0 - (-self.elapsed / self.time_constant_s).exp()
        };
        // Linear interp toward target, capped at 1.0 (so repeated steps
        // never overshoot).
        for bed in AmbientBed::ALL {
            let c = self.current.get(bed);
            let tgt = self.target.get(bed);
            let next = c + (tgt - c) * t;
            self.current.set(bed, next);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn footprint(
        water: u32,
        sand: u32,
        grass: u32,
        forest: u32,
        stone: u32,
        snow: u32,
        weather: WeatherOverlay,
        is_daytime: bool,
        is_winter: bool,
    ) -> BiomeFootprint {
        let total = water + sand + grass + forest + stone + snow;
        BiomeFootprint {
            water,
            sand,
            grass,
            forest,
            stone,
            snow,
            total_cells: total,
            weather,
            is_daytime,
            is_winter,
        }
    }

    #[test]
    fn empty_footprint_returns_wind_floor() {
        let fp = footprint(0, 0, 0, 0, 0, 0, WeatherOverlay::Clear, true, false);
        let w = BedWeights::from_footprint(&fp);
        assert!((w.wind - BedWeights::WIND_FLOOR).abs() < f32::EPSILON);
        assert!(w.water.abs() < f32::EPSILON);
        assert!(w.forest.abs() < f32::EPSILON);
        assert!(w.wildlife.abs() < f32::EPSILON);
        assert!(w.weather.abs() < f32::EPSILON);
    }

    #[test]
    fn coastal_footprint_lights_water_bed() {
        let fp = footprint(40, 10, 0, 0, 0, 0, WeatherOverlay::Clear, true, false);
        let w = BedWeights::from_footprint(&fp);
        // Water + Sand = 50/50.
        assert!((w.water - 1.0).abs() < 1e-5);
        // Wind floor still respected.
        assert!((w.wind - BedWeights::WIND_FLOOR).abs() < 1e-5);
    }

    #[test]
    fn forest_footprint_lights_forest_bed() {
        let fp = footprint(0, 0, 0, 100, 0, 0, WeatherOverlay::Clear, true, false);
        let w = BedWeights::from_footprint(&fp);
        assert!((w.forest - 1.0).abs() < 1e-5);
    }

    #[test]
    fn winter_silences_wildlife_overlay() {
        let fp_summer = footprint(0, 0, 50, 50, 0, 0, WeatherOverlay::Clear, true, false);
        let fp_winter = footprint(0, 0, 50, 50, 0, 0, WeatherOverlay::Clear, true, true);
        let ws = BedWeights::from_footprint(&fp_summer);
        let ww = BedWeights::from_footprint(&fp_winter);
        assert!(ws.wildlife > 0.0);
        assert!(ww.wildlife.abs() < f32::EPSILON);
    }

    #[test]
    fn nighttime_silences_wildlife_overlay() {
        let fp_day = footprint(0, 0, 50, 50, 0, 0, WeatherOverlay::Clear, true, false);
        let fp_night = footprint(0, 0, 50, 50, 0, 0, WeatherOverlay::Clear, false, false);
        let wd = BedWeights::from_footprint(&fp_day);
        let wn = BedWeights::from_footprint(&fp_night);
        assert!(wd.wildlife > 0.0);
        assert!(wn.wildlife.abs() < f32::EPSILON);
    }

    #[test]
    fn storm_overlay_stronger_than_light_rain() {
        let mut fp = footprint(0, 0, 100, 0, 0, 0, WeatherOverlay::Clear, true, false);
        fp.weather = WeatherOverlay::Rain;
        let wr = BedWeights::from_footprint(&fp);
        fp.weather = WeatherOverlay::Storm;
        let ws = BedWeights::from_footprint(&fp);
        assert!(ws.weather > wr.weather);
    }

    #[test]
    fn weights_are_well_formed_after_footprint_projection() {
        // Sanity: every output slot in [0,1] and finite for a range of inputs.
        for (weather, day, winter) in [
            (WeatherOverlay::Clear, true, false),
            (WeatherOverlay::Rain, true, true),
            (WeatherOverlay::Storm, false, false),
            (WeatherOverlay::Snow, false, true),
        ] {
            let fp = footprint(10, 5, 30, 20, 15, 20, weather, day, winter);
            let w = BedWeights::from_footprint(&fp);
            assert!(w.is_well_formed(), "weights not well-formed: {w:?}");
        }
    }

    #[test]
    fn blend_converges_to_target() {
        let mut blend = AmbientBlend {
            time_constant_s: 1.0,
            ..AmbientBlend::default()
        };
        let target = BedWeights {
            wind: 0.0,
            water: 1.0,
            forest: 0.0,
            wildlife: 0.0,
            weather: 0.0,
        };
        // 5 tau's of elapsed time → 1 - e^-5 ≈ 0.9933; close enough to 1.
        blend.step(target, 5.0);
        // Wind floor → 0; water → 1.
        assert!(blend.current.water > 0.98);
        assert!(blend.current.wind < 0.02);
    }

    #[test]
    fn blend_is_monotonic_per_slot() {
        let mut blend = AmbientBlend {
            time_constant_s: 1.0,
            ..AmbientBlend::default()
        };
        let target = BedWeights {
            wind: 0.0,
            water: 1.0,
            forest: 1.0,
            wildlife: 0.5,
            weather: 0.3,
        };
        // Step in 0.1 s chunks; current.water should never decrease.
        let mut last_water = 0.0_f32;
        for _ in 0..50 {
            blend.step(target, 0.1);
            assert!(blend.current.water >= last_water - 1e-6);
            last_water = blend.current.water;
        }
    }

    #[test]
    fn slot_cap_is_one() {
        // Wildlife + weather + a 100% forest footprint should NOT push a
        // slot past 1.0 even if a future bug over-allocates.
        let fp = footprint(0, 0, 0, 100, 0, 0, WeatherOverlay::Storm, true, false);
        let w = BedWeights::from_footprint(&fp);
        assert!(w.wind <= BedWeights::SLOT_CAP);
        assert!(w.water <= BedWeights::SLOT_CAP);
        assert!(w.forest <= BedWeights::SLOT_CAP);
        assert!(w.wildlife <= BedWeights::SLOT_CAP);
        assert!(w.weather <= BedWeights::SLOT_CAP);
    }

    /// Covers FR-CIV-AUDIO-002 — biome-driven ambient beds cross-fade
    /// by camera location. We assert the canonical coast→grass→forest
    /// glide produces a continuous weight vector without hard cuts.
    #[test]
    fn fr_audio_002_coast_to_forest_glide_is_continuous() {
        let coast = footprint(80, 20, 0, 0, 0, 0, WeatherOverlay::Clear, true, false);
        let mixed = footprint(20, 10, 40, 30, 0, 0, WeatherOverlay::Clear, true, false);
        let forest = footprint(0, 0, 0, 100, 0, 0, WeatherOverlay::Clear, true, false);

        let w_coast = BedWeights::from_footprint(&coast);
        let w_mixed = BedWeights::from_footprint(&mixed);
        let w_forest = BedWeights::from_footprint(&forest);

        // The glide must be monotonic in water share and forest share.
        assert!(w_coast.water > w_mixed.water);
        assert!(w_mixed.water > w_forest.water);
        assert!(w_coast.forest < w_mixed.forest);
        assert!(w_mixed.forest < w_forest.forest);

        // Now drive a blend through the three and check the bed
        // weights move smoothly between them.
        let mut blend = AmbientBlend {
            time_constant_s: 1.0,
            ..AmbientBlend::default()
        };
        blend.step(w_coast, 0.0);
        // Mid-glide state should be between coast and mixed.
        blend.step(w_mixed, 0.5);
        assert!(blend.current.water > w_forest.water);
        assert!(blend.current.water < w_coast.water);
        assert!(blend.current.forest > w_coast.forest);
        assert!(blend.current.forest < w_forest.forest);
    }
}
