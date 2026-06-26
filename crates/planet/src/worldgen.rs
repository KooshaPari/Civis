//! Deterministic seed-based world generation for terrain, biomes, and resources.
//!
//! The generator is intentionally pure: identical inputs always produce
//! identical outputs, and the only stochastic source is the explicit seed in
//! [`WorldgenConfig`].

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::{BiomeKind, Climate, PlanetConfig};

/// Seeded bootstrap configuration for procedural world generation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorldgenConfig {
    /// World seed used for all height, erosion, and resource sampling.
    pub seed: u64,
    /// Grid edge length in cells. The generator emits `size * size` tiles.
    pub size: u32,
    /// Normalized sea level in the range `[0, 1]`.
    pub sea_level: f32,
    /// Terrain roughness multiplier. Higher values produce sharper relief.
    pub roughness: f32,
}

impl WorldgenConfig {
    /// Create a minimal deterministic config.
    #[must_use]
    pub const fn new(seed: u64, size: u32, sea_level: f32, roughness: f32) -> Self {
        Self {
            seed,
            size: size.max(1),
            sea_level: if sea_level < 0.0 {
                0.0
            } else if sea_level > 1.0 {
                1.0
            } else {
                sea_level
            },
            roughness: if roughness < 0.05 { 0.05 } else { roughness },
        }
    }
}

/// Resource bands derived from seed, climate, and terrain position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    /// Sparse desert or polar scrub.
    Sparse,
    /// Common surface resource band.
    Common,
    /// Forest, ore, or fertile band.
    Rich,
    /// Rare deposits and highly fertile cells.
    Abundant,
}

/// One world cell in the generated terrain map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldgenTile {
    /// Quantized normalized height in `[0, 4095]`.
    pub height: u16,
    /// Climate-derived biome assignment.
    pub biome: BiomeKind,
    /// Quantized temperature in milli-degrees Celsius.
    pub temperature_milli_c: i16,
    /// Quantized rainfall in millimeters per year.
    pub rainfall_mm: u16,
    /// Resource richness band.
    pub resources: ResourceKind,
}

/// Dense deterministic map produced by [`generate_world`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldgenMap {
    /// Bootstrap config used to produce this map.
    pub config: WorldgenConfig,
    /// Immutable planet metadata that shaped the climate bands.
    pub planet: PlanetConfig,
    /// Tick 0 climate basis used for the initial bootstrap pass.
    pub climate: Climate,
    /// Row-major tiles, length `size * size`.
    pub tiles: Vec<WorldgenTile>,
}

/// Generate a deterministic world map from explicit seed/config and planet climate.
///
/// The process is:
/// 1. Sample layered simplex noise for a base heightmap.
/// 2. Apply a stable thermal + hydraulic erosion pass.
/// 3. Derive temperature/rainfall from latitude and planet tilt.
/// 4. Classify biomes and resources from the resulting climate bands.
#[must_use]
pub fn generate_world(config: WorldgenConfig, planet: PlanetConfig, climate: Climate) -> WorldgenMap {
    let size = config.size.max(1) as usize;
    let mut heights = base_heights(&config, size);
    erode_heights(&mut heights, &config, size);

    let mut tiles = Vec::with_capacity(size * size);
    for z in 0..size {
        for x in 0..size {
            let idx = x + z * size;
            let normalized_height = heights[idx];
            let (temperature, rainfall) = climate_band(&planet, &climate, &config, x, z, size);
            let biome =
                classify_world_biome(normalized_height, temperature, rainfall, config.sea_level);
            let resources =
                resource_band(&config, normalized_height, temperature, rainfall, x, z, size);
            tiles.push(WorldgenTile {
                height: quantize_height(normalized_height),
                biome,
                temperature_milli_c: ((temperature * 1000.0)
                    .round()
                    .clamp(i16::MIN as f32, i16::MAX as f32)) as i16,
                rainfall_mm: rainfall.round().clamp(0.0, u16::MAX as f32) as u16,
                resources,
            });
        }
    }

    WorldgenMap {
        config,
        planet,
        climate,
        tiles,
    }
}

fn base_heights(config: &WorldgenConfig, size: usize) -> Vec<f32> {
    let mut heights = Vec::with_capacity(size * size);
    let seed = config.seed;
    for z in 0..size {
        for x in 0..size {
            let nx = x as f32 / size as f32;
            let nz = z as f32 / size as f32;
            let ridged = simplex_fbm(seed, nx * 6.0, nz * 6.0, 4);
            let continental = simplex_fbm(seed ^ 0x9e37_79b9_7f4a_7c15, nx * 2.0, nz * 2.0, 3);
            let latitude = (nz - 0.5).abs() * 2.0;
            let polar_bias = 0.18 * latitude;
            let mut h = 0.45 + ridged * 0.22 * config.roughness + continental * 0.18;
            h -= polar_bias * 0.08;
            heights.push(clamp01(h));
        }
    }
    heights
}

fn erode_heights(heights: &mut [f32], config: &WorldgenConfig, size: usize) {
    let mut scratch = heights.to_vec();
    let passes = 2 + ((config.roughness * 2.0).round() as usize).min(4);
    for pass in 0..passes {
        for z in 0..size {
            for x in 0..size {
                let idx = x + z * size;
                let current = scratch[idx];
                let mut flow = 0.0f32;
                let mut count = 0.0f32;
                for (nx, nz) in neighbors(x, z, size) {
                    let nidx = nx + nz * size;
                    flow += scratch[nidx];
                    count += 1.0;
                }
                let avg = if count > 0.0 { flow / count } else { current };
                let thermal = current + (avg - current) * 0.18;
                let hydraulic = thermal - river_cut(config.seed, x, z, pass) * 0.04;
                heights[idx] = clamp01(hydraulic);
            }
        }
        scratch.copy_from_slice(heights);
    }
}

fn river_cut(seed: u64, x: usize, z: usize, pass: usize) -> f32 {
    let n = hash01(seed ^ 0x51eb_851e_51eb_851e, x as u64, z as u64, pass as u64);
    if n > 0.82 {
        (n - 0.82) / 0.18
    } else {
        0.0
    }
}

fn climate_band(
    planet: &PlanetConfig,
    climate: &Climate,
    config: &WorldgenConfig,
    x: usize,
    z: usize,
    size: usize,
) -> (f32, f32) {
    let lat = ((z as f32 + 0.5) / size as f32 - 0.5).abs() * 2.0;
    let tilt = (planet.axial_tilt_deg as f32 / 90.0).clamp(0.0, 1.0);
    let seasonal = (climate.year_phase * std::f32::consts::TAU).sin() * 0.08;
    let temperature = 30.0 - lat * (42.0 + tilt * 16.0) + seasonal * 10.0;
    let rain_base = 1200.0 - lat * (700.0 + config.roughness * 140.0);
    let rain_noise = simplex_value(config.seed ^ 0x7f4a_7c15_9e37_79b9, x as f32 * 0.07, z as f32 * 0.07, 3);
    let rainfall = (rain_base + rain_noise * 220.0).max(0.0);
    (temperature, rainfall)
}

fn classify_world_biome(height: f32, temperature: f32, rainfall: f32, sea_level: f32) -> BiomeKind {
    if height < sea_level - 0.01 {
        return BiomeKind::Ocean;
    }
    if height < sea_level + 0.02 {
        return BiomeKind::Beach;
    }
    if temperature <= -5.0 && height > 0.72 {
        return BiomeKind::Glacier;
    }
    if height > 0.76 {
        return BiomeKind::Mountain;
    }
    if temperature > 22.0 {
        if rainfall < 300.0 {
            BiomeKind::Desert
        } else if rainfall < 850.0 {
            BiomeKind::Savanna
        } else {
            BiomeKind::Rainforest
        }
    } else if temperature >= 2.0 {
        if rainfall < 260.0 {
            BiomeKind::Grassland
        } else if rainfall < 900.0 {
            BiomeKind::Forest
        } else {
            BiomeKind::Wetland
        }
    } else if temperature >= -8.0 {
        BiomeKind::Taiga
    } else {
        BiomeKind::Tundra
    }
}

fn resource_band(
    config: &WorldgenConfig,
    height: f32,
    temperature: f32,
    rainfall: f32,
    x: usize,
    z: usize,
    size: usize,
) -> ResourceKind {
    let noise = hash01(config.seed ^ 0xa5a5_5a5a_1234_5678, x as u64, z as u64, size as u64);
    let fertility = rainfall / 1500.0 + (temperature.max(-20.0) + 20.0) / 60.0 - (height - 0.5).abs();
    if fertility > 1.1 || noise > 0.95 {
        ResourceKind::Abundant
    } else if fertility > 0.65 || noise > 0.72 {
        ResourceKind::Rich
    } else if fertility > 0.25 || noise > 0.42 {
        ResourceKind::Common
    } else {
        ResourceKind::Sparse
    }
}

fn quantize_height(height: f32) -> u16 {
    (height.clamp(0.0, 1.0) * 4095.0).round() as u16
}

fn neighbors(x: usize, z: usize, size: usize) -> impl Iterator<Item = (usize, usize)> {
    let mut items = [(x, z); 4];
    let mut len = 0usize;
    if x > 0 {
        items[len] = (x - 1, z);
        len += 1;
    }
    if x + 1 < size {
        items[len] = (x + 1, z);
        len += 1;
    }
    if z > 0 {
        items[len] = (x, z - 1);
        len += 1;
    }
    if z + 1 < size {
        items[len] = (x, z + 1);
        len += 1;
    }
    items[..len].iter().copied()
}

fn simplex_fbm(seed: u64, x: f32, y: f32, octaves: usize) -> f32 {
    let mut total = 0.0f32;
    let mut amplitude = 1.0f32;
    let mut frequency = 1.0f32;
    let mut norm = 0.0f32;
    for octave in 0..octaves {
        total += simplex2(seed ^ octave as u64, x * frequency, y * frequency) * amplitude;
        norm += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    if norm == 0.0 { 0.0 } else { total / norm }
}

fn simplex_value(seed: u64, x: f32, y: f32, octaves: usize) -> f32 {
    simplex_fbm(seed, x, y, octaves) * 0.5 + 0.5
}

fn simplex2(seed: u64, x: f32, y: f32) -> f32 {
    const F2: f32 = 0.366_025_4;
    const G2: f32 = 0.211_324_87;

    let s = (x + y) * F2;
    let i = (x + s).floor();
    let j = (y + s).floor();
    let t = (i + j) * G2;
    let x0 = x - (i - t);
    let y0 = y - (j - t);

    let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
    let x1 = x0 - i1 as f32 + G2;
    let y1 = y0 - j1 as f32 + G2;
    let x2 = x0 - 1.0 + 2.0 * G2;
    let y2 = y0 - 1.0 + 2.0 * G2;

    let n0 = corner(seed, i as i32, j as i32, x0, y0);
    let n1 = corner(seed, i as i32 + i1, j as i32 + j1, x1, y1);
    let n2 = corner(seed, i as i32 + 1, j as i32 + 1, x2, y2);
    70.0 * (n0 + n1 + n2)
}

fn corner(seed: u64, i: i32, j: i32, x: f32, y: f32) -> f32 {
    let t = 0.5 - x * x - y * y;
    if t <= 0.0 {
        return 0.0;
    }
    let grad = gradient(seed, i, j);
    let t2 = t * t;
    t2 * t2 * (grad.0 * x + grad.1 * y)
}

fn gradient(seed: u64, x: i32, y: i32) -> (f32, f32) {
    match (hash01(seed, x as u64, y as u64, 0) * 8.0).floor() as u8 {
        0 => (1.0, 1.0),
        1 => (-1.0, 1.0),
        2 => (1.0, -1.0),
        3 => (-1.0, -1.0),
        4 => (1.0, 0.0),
        5 => (-1.0, 0.0),
        6 => (0.0, 1.0),
        _ => (0.0, -1.0),
    }
}

fn hash01(seed: u64, x: u64, y: u64, lane: u64) -> f32 {
    let mut v = seed ^ x.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    v ^= y.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    v ^= lane.wrapping_mul(0x94d0_49bb_1331_11eb);
    let mixed = splitmix64(v);
    ((mixed >> 11) as f32) / ((1u64 << 53) as f32)
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults_earthlike;

    #[test]
    fn same_seed_produces_identical_heightmap() {
        let (planet, moon) = defaults_earthlike();
        let climate = crate::compute_climate(0, &planet, &moon);
        let cfg = WorldgenConfig::new(42, 64, 0.42, 0.8);
        let a = generate_world(cfg, planet, climate);
        let b = generate_world(cfg, planet, climate);
        assert_eq!(
            a.tiles.iter().map(|t| t.height).collect::<Vec<_>>(),
            b.tiles.iter().map(|t| t.height).collect::<Vec<_>>()
        );
    }

    #[test]
    fn different_seeds_differ() {
        let (planet, moon) = defaults_earthlike();
        let climate = crate::compute_climate(0, &planet, &moon);
        let a = generate_world(WorldgenConfig::new(1, 64, 0.42, 0.8), planet, climate);
        let b = generate_world(WorldgenConfig::new(2, 64, 0.42, 0.8), planet, climate);
        assert_ne!(
            a.tiles.iter().map(|t| t.height).collect::<Vec<_>>(),
            b.tiles.iter().map(|t| t.height).collect::<Vec<_>>()
        );
    }

    #[test]
    fn produces_biomes_and_resources() {
        let (planet, moon) = defaults_earthlike();
        let climate = crate::compute_climate(0, &planet, &moon);
        let map = generate_world(WorldgenConfig::new(7, 48, 0.4, 1.1), planet, climate);
        assert_eq!(map.tiles.len(), 48 * 48);
        assert!(map.tiles.iter().any(|t| t.biome == BiomeKind::Ocean));
        assert!(map.tiles.iter().any(|t| matches!(t.resources, ResourceKind::Rich | ResourceKind::Abundant)));
    }
}
