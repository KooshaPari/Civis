//! FR-CIV-PLANET-040 — deterministic geology seed layer.
//!
//! `GeologyMap::seed` is purely config-derived, produces a stable `Vec<RegionBiome>`
//! for every call with the same `PlanetConfig`, and never touches tick or RNG.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::PlanetConfig;

/// The six canonical biome archetypes for a planet region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiomeKind {
    /// Open water — radius-derived; large planets have proportionally more ocean.
    Ocean,
    /// Flat grassland and savanna.
    Plains,
    /// Temperate and tropical forest.
    Forest,
    /// High-altitude or tectonic uplift terrain.
    Mountain,
    /// Arid low-humidity terrain.
    Desert,
    /// Cold polar or high-altitude terrain.
    Tundra,
}

/// A single region's biome assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionBiome {
    /// Stable integer identifier for this region (0-based).
    pub region_id: u32,
    /// Assigned biome archetype.
    pub biome: BiomeKind,
}

/// Deterministic planet-wide geology map derived from [`PlanetConfig`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeologyMap {
    /// One entry per region; length == `NUM_REGIONS` (16).
    pub regions: Vec<RegionBiome>,
}

impl GeologyMap {
    /// Derive a deterministic geology map from `planet_config`.
    ///
    /// # Determinism guarantee
    /// The result depends only on `planet_config` fields; no external state,
    /// no RNG, no tick. Identical inputs always produce identical outputs.
    ///
    /// # Region count
    /// Uses a fixed grid of 16 canonical regions, each assigned a biome based
    /// on its normalised latitude and `radius_km` / `axial_tilt_deg`.
    ///
    /// # Biome model (all integer arithmetic)
    /// Each region `r` in `0..16` is assigned a latitude band in `[-8, +8]`.
    /// - `|band| >= 7` → Tundra (poles)
    /// - `|band| >= 5` → Mountain (sub-polar uplift)
    /// - Ocean fraction = `radius_km * 8 / 6_371` clamped to `[0, 16]` — regions
    ///   with `r < ocean_regions` get Ocean.
    /// - Remaining equatorial band: Desert when `axial_tilt_deg < 10`, Forest
    ///   when `axial_tilt_deg > 30`, else Plains (temperate).
    pub fn seed(planet_config: &PlanetConfig) -> GeologyMap {
        const NUM_REGIONS: u32 = 16;
        // Ocean fraction: earth-sized planet (6_371 km) → 8 ocean regions out of 16.
        // Scale linearly; clamp to [0, NUM_REGIONS].
        let ocean_regions =
            ((planet_config.radius_km as u64 * 8) / 6_371).min(NUM_REGIONS as u64) as u32;

        let mut regions = Vec::with_capacity(NUM_REGIONS as usize);
        for r in 0..NUM_REGIONS {
            // Latitude band in [-8, +8], centre-mapped from region index.
            // r=0 → band=-8 (south pole), r=15 → band=+8 (north pole)
            let band: i32 = -8 + (r as i32 * 16 / (NUM_REGIONS as i32 - 1));

            let biome = if r < ocean_regions {
                BiomeKind::Ocean
            } else if band.unsigned_abs() >= 7 {
                BiomeKind::Tundra
            } else if band.unsigned_abs() >= 5 {
                BiomeKind::Mountain
            } else if planet_config.axial_tilt_deg < 10 {
                BiomeKind::Desert
            } else if planet_config.axial_tilt_deg > 30 {
                BiomeKind::Forest
            } else {
                BiomeKind::Plains
            };

            regions.push(RegionBiome {
                region_id: r,
                biome,
            });
        }

        GeologyMap { regions }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults_earthlike;

    /// FR-CIV-PLANET-040 — same PlanetConfig always produces a bit-identical GeologyMap.
    #[test]
    fn geology_map_is_stable_for_same_planet_config() {
        let (planet, _) = defaults_earthlike();
        let a = GeologyMap::seed(&planet);
        let b = GeologyMap::seed(&planet);
        assert_eq!(
            a, b,
            "GeologyMap must be identical across two calls with the same PlanetConfig"
        );

        // Sensitivity: changing radius_km by 1000 km must produce a different map
        // (ocean fraction changes, verifying config is actually consumed).
        let mut tweaked = planet;
        tweaked.radius_km = planet.radius_km + 1_000;
        let c = GeologyMap::seed(&tweaked);
        // The tweak shifts ocean_regions; maps must differ.
        assert_ne!(
            a.regions.iter().map(|r| r.biome as u8).collect::<Vec<_>>(),
            c.regions.iter().map(|r| r.biome as u8).collect::<Vec<_>>(),
            "radius_km delta of 1000 km must change the geology map"
        );
    }

    /// FR-CIV-PLANET-040 — the equatorial biome is driven by `axial_tilt_deg`:
    /// arid (<10°) → Desert, humid (>30°) → Forest, temperate → Plains. Covers
    /// the three tilt-dependent branches of `seed`.
    #[test]
    fn equatorial_biome_follows_axial_tilt() {
        let (planet, _) = defaults_earthlike();

        let mut arid = planet;
        arid.axial_tilt_deg = 5; // < 10 -> Desert
        assert!(
            GeologyMap::seed(&arid)
                .regions
                .iter()
                .any(|r| r.biome == BiomeKind::Desert),
            "low axial tilt must yield Desert regions"
        );

        let mut humid = planet;
        humid.axial_tilt_deg = 40; // > 30 -> Forest
        assert!(
            GeologyMap::seed(&humid)
                .regions
                .iter()
                .any(|r| r.biome == BiomeKind::Forest),
            "high axial tilt must yield Forest regions"
        );

        // Earthlike default (23°) is temperate -> Plains at the equator.
        assert!(
            GeologyMap::seed(&planet)
                .regions
                .iter()
                .any(|r| r.biome == BiomeKind::Plains),
            "temperate axial tilt must yield Plains regions"
        );
    }
}
