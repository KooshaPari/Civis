//! Standalone terrain generator copied from `civ-watch`.
//!
//! Kept local so the Bevy standalone binary stays self-contained and does not
//! pull the HTTP/watch stack into the executable.

pub const SIZE: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    DeepWater,
    Water,
    Sand,
    Grass,
    Forest,
    Stone,
    Snow,
}

impl Biome {
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

#[derive(Debug, Clone)]
pub struct Terrain {
    pub size: usize,
    pub heights: Vec<f32>,
    pub biomes: Vec<Biome>,
}

impl Terrain {
    pub fn generate(seed: u64) -> Self {
        let mut heights = vec![0.0; SIZE * SIZE];
        let octaves: [(f32, f32); 6] = [
            (1.5, 0.45),
            (3.0, 0.25),
            (6.0, 0.14),
            (12.0, 0.08),
            (24.0, 0.05),
            (48.0, 0.025),
        ];
        for y in 0..SIZE {
            for x in 0..SIZE {
                let fx = x as f32 / SIZE as f32;
                let fy = y as f32 / SIZE as f32;
                let mut h = 0.0;
                for (freq, amp) in octaves {
                    h += value_noise(fx * freq, fy * freq, seed) * amp;
                }
                let dx = fx - 0.5;
                let dy = fy - 0.5;
                let r = (dx * dx + dy * dy).sqrt() * 2.0;
                let island = (1.0 - r.powf(1.55)).clamp(0.0, 1.0);
                let ridge = ((1.0 - (dx.abs() * 1.7).min(1.0)) * (1.0 - (dy.abs() * 1.7).min(1.0)))
                    .clamp(0.0, 1.0);
                h = h * 0.48 + island * 0.34 + ridge * 0.18;
                heights[y * SIZE + x] = h.clamp(0.0, 1.0);
            }
        }
        let biomes = heights.iter().copied().map(Biome::from_height).collect();
        Self {
            size: SIZE,
            heights,
            biomes,
        }
    }

    pub fn heights_fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for height in &self.heights {
            for byte in height.to_bits().to_le_bytes() {
                h ^= u64::from(byte);
                h = h.wrapping_mul(0x100000001b3);
            }
        }
        h
    }
}

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
