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

impl BiomeKind {
    /// Return `true` when the affinity label string from a [`SeedDefinition`]
    /// is considered a match for this biome archetype.
    ///
    /// The comparison is case-sensitive and uses a curated alias table so the
    /// genetics crate can stay decoupled from `BiomeKind` while scenario
    /// authors can still use natural names like `"TemperateForest"` or
    /// `"Grassland"`.
    #[must_use]
    pub fn matches_affinity(self, label: &str) -> bool {
        match self {
            BiomeKind::Forest => matches!(
                label,
                "Forest" | "TemperateForest" | "TropicalForest" | "Jungle"
            ),
            BiomeKind::Ocean => matches!(label, "Ocean" | "Tidepool" | "DeepOcean" | "Sea"),
            BiomeKind::Mountain => {
                matches!(label, "Mountain" | "Alpine" | "Highland" | "Volcano")
            }
            BiomeKind::Desert => matches!(label, "Desert" | "Arid" | "Badlands" | "Dunes"),
            BiomeKind::Tundra => {
                matches!(label, "Tundra" | "Arctic" | "Boreal" | "Taiga" | "Permafrost")
            }
            BiomeKind::Plains => {
                matches!(label, "Plains" | "Grassland" | "Savanna" | "Steppe" | "Prairie")
            }
        }
    }
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

    /// Map a normalised horizontal position `(nx, nz) ∈ [0, 1]²` to the
    /// biome archetype of the region it falls in.
    ///
    /// The 16-region grid produced by [`GeologyMap::seed`] is a 1-D latitude
    /// band (region 0 = south pole, region 15 = north pole) derived entirely
    /// from the planet's `radius_km` and `axial_tilt_deg`. For the purposes of
    /// spawn-time biome lookup we map the north–south axis (nz) to the region
    /// index so equatorial spawns land in equatorial biomes and polar spawns in
    /// polar biomes. The east–west axis (nx) is ignored here because the 1-D
    /// band model has no longitudinal variation.
    ///
    /// Returns [`BiomeKind::Plains`] when `regions` is empty (defensive).
    #[must_use]
    pub fn biome_at_normalized(&self, _nx: f32, nz: f32) -> BiomeKind {
        if self.regions.is_empty() {
            return BiomeKind::Plains;
        }
        let n = self.regions.len();
        // Clamp nz to [0, 1] and map to [0, n-1] with rounding.
        let idx = ((nz.clamp(0.0, 1.0) * (n - 1) as f32).round() as usize).min(n - 1);
        self.regions[idx].biome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults_earthlike;

    /// BiomeKind::matches_affinity maps label strings to biome archetypes correctly.
    #[test]
    fn biome_affinity_label_bridge() {
        // Forest matches canonical and alias labels.
        assert!(BiomeKind::Forest.matches_affinity("Forest"));
        assert!(BiomeKind::Forest.matches_affinity("TemperateForest"));
        assert!(BiomeKind::Forest.matches_affinity("TropicalForest"));
        // Ocean matches ocean / coastal aliases.
        assert!(BiomeKind::Ocean.matches_affinity("Ocean"));
        assert!(BiomeKind::Ocean.matches_affinity("Tidepool"));
        // Cross-biome false.
        assert!(!BiomeKind::Ocean.matches_affinity("Forest"));
        assert!(!BiomeKind::Forest.matches_affinity("Ocean"));
        assert!(!BiomeKind::Mountain.matches_affinity("Desert"));
        // Other biomes.
        assert!(BiomeKind::Desert.matches_affinity("Arid"));
        assert!(BiomeKind::Tundra.matches_affinity("Arctic"));
        assert!(BiomeKind::Tundra.matches_affinity("Boreal"));
        assert!(BiomeKind::Plains.matches_affinity("Grassland"));
        assert!(BiomeKind::Plains.matches_affinity("Savanna"));
        assert!(BiomeKind::Mountain.matches_affinity("Alpine"));
        // Unknown label never matches anything.
        assert!(!BiomeKind::Plains.matches_affinity("Bog"));
        assert!(!BiomeKind::Desert.matches_affinity(""));
    }

    /// biome_at_normalized returns polar biomes near edges and equatorial near centre.
    #[test]
    fn biome_at_normalized_maps_latitude_to_region() {
        // Use a planet with high axial tilt so equatorial regions are Forest.
        let (mut planet, _) = defaults_earthlike();
        planet.axial_tilt_deg = 40; // > 30 → equatorial Forest
        let map = GeologyMap::seed(&planet);

        // nz = 0.5 → equatorial → Forest
        let mid = map.biome_at_normalized(0.0, 0.5);
        assert_eq!(
            mid,
            BiomeKind::Forest,
            "mid-latitude should be Forest with high axial tilt"
        );

        // nz = 0.0 → south pole → Tundra or Ocean (depends on radius)
        let south = map.biome_at_normalized(0.0, 0.0);
        assert!(
            south == BiomeKind::Tundra || south == BiomeKind::Ocean,
            "south pole should be polar, got {south:?}"
        );

        // nz = 1.0 → north pole → Tundra or Ocean
        let north = map.biome_at_normalized(0.0, 1.0);
        assert!(
            north == BiomeKind::Tundra || north == BiomeKind::Ocean,
            "north pole should be polar, got {north:?}"
        );
    }

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
