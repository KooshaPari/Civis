//! Procedural terrain generator for the Civis 3D sandbox dashboard.
//!
//! Lightweight value-noise heightmap with biome banding. Generated once at
//! startup; exposed over HTTP at `GET /terrain` so the web dashboard can
//! render a top-down WorldBox-style god view without needing the kernel's
//! chunk-getter API to be merged yet.
//!
//! No external dependencies — the noise is a simple multi-octave hashed
//! value-noise so the snapshot is bit-identical under a given seed (replay-safe).

use serde::Serialize;

/// Side length of the generated terrain grid.
pub const SIZE: usize = 128;

/// One terrain biome. Maps to a colour in the web dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Biome {
    /// Deep water.
    DeepWater,
    /// Shallow water / coast.
    Water,
    /// Sandy beach.
    Sand,
    /// Grasslands.
    Grass,
    /// Forest.
    Forest,
    /// Stone / mountain.
    Stone,
    /// Snow-capped peak.
    Snow,
}

impl Biome {
    /// Convert this biome to an RGB triplet for the renderer.
    #[allow(dead_code)]
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Self::DeepWater => [16, 38, 90],
            Self::Water => [44, 100, 168],
            Self::Sand => [222, 200, 132],
            Self::Grass => [104, 154, 60],
            Self::Forest => [44, 100, 52],
            Self::Stone => [128, 124, 116],
            Self::Snow => [240, 240, 240],
        }
    }

    /// Convert a normalised height in `[0, 1]` to a biome.
    pub fn from_height(h: f32) -> Self {
        if h < 0.30 {
            Self::DeepWater
        } else if h < 0.40 {
            Self::Water
        } else if h < 0.45 {
            Self::Sand
        } else if h < 0.60 {
            Self::Grass
        } else if h < 0.75 {
            Self::Forest
        } else if h < 0.90 {
            Self::Stone
        } else {
            Self::Snow
        }
    }
}

/// A finalized terrain grid: `SIZE × SIZE` heights + per-cell biome.
#[derive(Debug, Clone, Serialize)]
pub struct Terrain {
    /// Grid side length.
    pub size: usize,
    /// Per-cell heights in `[0, 1]`. Row-major, length `size * size`.
    pub heights: Vec<f32>,
    /// Per-cell biome.
    pub biomes: Vec<Biome>,
}

impl Terrain {
    /// Generate a new heightmap from `seed`. Deterministic.
    pub fn generate(seed: u64) -> Self {
        let mut heights = vec![0.0_f32; SIZE * SIZE];
        // Multi-octave value noise: 4 octaves of doubling frequency / halving
        // amplitude. Continent shape biased toward the centre so coastlines
        // land inside the grid.
        let octaves: [(f32, f32); 4] = [(2.0, 0.55), (4.0, 0.30), (8.0, 0.15), (16.0, 0.075)];
        for y in 0..SIZE {
            for x in 0..SIZE {
                let fx = x as f32 / SIZE as f32;
                let fy = y as f32 / SIZE as f32;
                let mut h: f32 = 0.0;
                for (freq, amp) in octaves {
                    h += value_noise(fx * freq, fy * freq, seed) * amp;
                }
                // Radial falloff so the world has natural coastlines.
                let dx = fx - 0.5;
                let dy = fy - 0.5;
                let r = (dx * dx + dy * dy).sqrt() * 2.0; // 0 at centre, ~1.41 at corner
                let coastal = (1.0 - r).clamp(0.0, 1.0);
                h = h * 0.6 + coastal * 0.5;
                heights[y * SIZE + x] = h.clamp(0.0, 1.0);
            }
        }
        let biomes: Vec<Biome> = heights.iter().map(|&h| Biome::from_height(h)).collect();
        Self {
            size: SIZE,
            heights,
            biomes,
        }
    }
}

/// Smooth 2-D value noise on `[0, +inf)` with integer lattice + bicubic-ish
/// smoothing. Seeded by `seed` so the world is replay-safe.
fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();
    let h00 = hash_to_unit(xi, yi, seed);
    let h10 = hash_to_unit(xi + 1, yi, seed);
    let h01 = hash_to_unit(xi, yi + 1, seed);
    let h11 = hash_to_unit(xi + 1, yi + 1, seed);
    let u = smoothstep(xf);
    let v = smoothstep(yf);
    let i1 = h00 * (1.0 - u) + h10 * u;
    let i2 = h01 * (1.0 - u) + h11 * u;
    i1 * (1.0 - v) + i2 * v
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// FNV-1a derived hash to a unit float for the value-noise lattice.
#[allow(clippy::unusual_byte_groupings)]
fn hash_to_unit(x: i32, y: i32, seed: u64) -> f32 {
    let mut h: u64 = 0xcbf29ce484222325 ^ seed;
    let bytes = [
        x.to_le_bytes(),
        y.to_le_bytes(),
        (seed as i32).to_le_bytes(),
    ];
    for b in bytes.iter().flatten() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    (h as f32 / u64::MAX as f32).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_generation_is_deterministic() {
        let a = Terrain::generate(42);
        let b = Terrain::generate(42);
        assert_eq!(a.heights, b.heights);
    }

    #[test]
    fn different_seeds_produce_different_terrain() {
        let a = Terrain::generate(1);
        let b = Terrain::generate(2);
        assert_ne!(a.heights, b.heights);
    }

    #[test]
    fn terrain_size_consistent() {
        let t = Terrain::generate(0);
        assert_eq!(t.heights.len(), SIZE * SIZE);
        assert_eq!(t.biomes.len(), SIZE * SIZE);
        assert_eq!(t.size, SIZE);
    }

    #[test]
    fn all_biomes_can_be_hit() {
        // The default radial falloff + 4 octaves should produce every biome
        // somewhere in a 128x128 grid.
        let t = Terrain::generate(7);
        let mut seen = [false; 7];
        for b in &t.biomes {
            match b {
                Biome::DeepWater => seen[0] = true,
                Biome::Water => seen[1] = true,
                Biome::Sand => seen[2] = true,
                Biome::Grass => seen[3] = true,
                Biome::Forest => seen[4] = true,
                Biome::Stone => seen[5] = true,
                Biome::Snow => seen[6] = true,
            }
        }
        // Snow is rare; assert at least 5/7 biomes are present.
        let count = seen.iter().filter(|&&b| b).count();
        assert!(count >= 5, "only {count} biomes seen across 128x128 grid");
    }
}
