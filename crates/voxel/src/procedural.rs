//! Procedural terrain generation with multi-octave value noise and biome
//! classification for the Civis 3-D god-game.
//!
//! *No external noise libraries* – the `NoiseLayer` is a self-contained 2-D
//! value-noise implementation seeded via an integer hash so results are fully
//! deterministic for a given seed.

/// Configuration that controls how the multi-octave noise is sampled.
#[derive(Debug, Clone, Copy)]
pub struct BiomeNoiseConfig {
    /// Number of noise octaves to accumulate.
    pub octaves: u32,
    /// Frequency multiplier applied at each successive octave.
    pub lacunarity: f32,
    /// Amplitude multiplier applied at each successive octave (0.0–1.0).
    pub gain: f32,
    /// Base frequency of the first octave.
    pub frequency: f32,
    /// Seed mixed into every hash call – changing this produces completely
    /// different terrain.
    pub seed: u64,
}

impl Default for BiomeNoiseConfig {
    fn default() -> Self {
        Self {
            octaves: 4,
            lacunarity: 2.0,
            gain: 0.5,
            frequency: 0.01,
            seed: 0,
        }
    }
}

// ─── NoiseLayer ────────────────────────────────────────────────────────

/// Simple 2-D value noise built from scratch using a seed-based integer hash.
///
/// Values are deterministic for a given `(seed, x, y)` triple.
pub struct NoiseLayer {
    seed: u64,
}

impl NoiseLayer {
    /// Create a new noise layer with the given seed.
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Hash an `(i32, i32)` coordinate pair into a pseudo-random `u32`.
    #[inline]
    fn hash(x: i32, y: i32) -> u32 {
        let mut h = x as u32;
        h ^= y.wrapping_mul(0x9e3779b9) as u32;
        h = h.wrapping_mul(0x85ebca6b);
        h ^= h >> 16;
        h
    }

    /// Sample the noise at floating-point coordinates `x`, `y` (before any
    /// frequency scaling).  Returns a value in `[-1.0, 1.0]`.
    pub fn sample(&self, x: f32, y: f32) -> f32 {
        let ix = x.floor() as i32;
        let iy = y.floor() as i32;
        let fx = x - x.floor();
        let fy = y - y.floor();

        // Smoothstep (Hermite) for natural-looking interpolation.
        let u = fx * fx * (3.0 - 2.0 * fx);
        let v = fy * fy * (3.0 - 2.0 * fy);

        let n00 = self.corner(ix, iy);
        let n10 = self.corner(ix + 1, iy);
        let n01 = self.corner(ix, iy + 1);
        let n11 = self.corner(ix + 1, iy + 1);

        let nx0 = n00 + (n10 - n00) * u;
        let nx1 = n01 + (n11 - n01) * u;

        nx0 + (nx1 - nx0) * v
    }

    /// Map a hashed integer coordinate to a float in `[-1.0, 1.0]`.
    #[inline]
    fn corner(&self, x: i32, y: i32) -> f32 {
        let h = self.hash_i32(x, y);
        // Map u32 → [-1.0, 1.0]
        (h as f32) / (u32::MAX as f32 / 2.0) - 1.0
    }

    /// Integer-hash with the instance seed mixed in.
    fn hash_i32(&self, x: i32, y: i32) -> u32 {
        let mut h = self.seed as u32;
        h ^= x as u32;
        h = h.wrapping_mul(0x45d9f3b);
        h ^= y as u32;
        h = h.wrapping_mul(0x45d9f3b);
        h ^= h >> 16;
        h
    }

    /// Multi-octave (fractal Brownian motion) sample.
    ///
    /// `config.frequency` scales the input coordinates, and successive octaves
    /// apply `lacunarity` / `gain` to produce natural terrain variation.
    pub fn fbm(&self, x: f32, y: f32, config: &BiomeNoiseConfig) -> f32 {
        let mut value = 0.0_f32;
        let mut amplitude = 1.0_f32;
        let mut frequency = config.frequency;
        let mut max_amplitude = 0.0_f32;

        for _ in 0..config.octaves {
            value += amplitude * self.sample(x * frequency, y * frequency);
            max_amplitude += amplitude;
            amplitude *= config.gain;
            frequency *= config.lacunarity;
        }

        // Normalise to [-1.0, 1.0].
        if max_amplitude > 0.0 {
            value / max_amplitude
        } else {
            0.0
        }
    }
}

// ─── BiomeClassifier ──────────────────────────────────────────────────

/// Maps a `(height, moisture)` pair to one of eight biome codes.
///
/// ```text
///  0 = ocean      1 = beach      2 = grassland   3 = forest
///  4 = mountain   5 = snow       6 = desert      7 = swamp
/// ```
pub struct BiomeClassifier;

impl BiomeClassifier {
    /// Classify a cell given its normalised `height` and `moisture`
    /// (both in `0.0 – 1.0`).
    pub fn classify(height: f32, moisture: f32) -> u8 {
        if height < 0.10 {
            return BIOME_OCEAN;
        }
        if height < 0.15 {
            return BIOME_BEACH;
        }
        if height > 0.90 {
            return BIOME_SNOW;
        }
        if height > 0.80 {
            return BIOME_MOUNTAIN;
        }

        // Midland biomes determined by moisture.
        if moisture < 0.25 {
            BIOME_DESERT
        } else if moisture < 0.50 {
            BIOME_GRASSLAND
        } else if moisture < 0.75 {
            BIOME_FOREST
        } else {
            BIOME_SWAMP
        }
    }
}

// Biome constant table (re-exported for test convenience).
pub const BIOME_OCEAN: u8 = 0;
pub const BIOME_BEACH: u8 = 1;
pub const BIOME_GRASSLAND: u8 = 2;
pub const BIOME_FOREST: u8 = 3;
pub const BIOME_MOUNTAIN: u8 = 4;
pub const BIOME_SNOW: u8 = 5;
pub const BIOME_DESERT: u8 = 6;
pub const BIOME_SWAMP: u8 = 7;

// ─── TerrainBiomeMap ───────────────────────────────────────────────────

/// A 2-D grid storing both the height field and the biome classification for
/// every cell.
pub struct TerrainBiomeMap {
    /// Width in cells.
    pub width: usize,
    /// Height in cells.
    pub height: usize,
    /// Per-cell biome code (index = `y * width + x`).
    pub biome_data: Vec<u8>,
    /// Per-cell height in `[0.0, 1.0]` (index = `y * width + x`).
    pub height_data: Vec<f32>,
}

impl TerrainBiomeMap {
    /// Allocate a new map of the given dimensions, filled with zeroes.
    pub fn new(width: usize, height: usize) -> Self {
        let len = width * height;
        Self {
            width,
            height,
            biome_data: vec![0; len],
            height_data: vec![0.0; len],
        }
    }

    /// Generate the full terrain using multi-octave value noise for height and
    /// a second noise layer for moisture.  The `config` controls the height
    /// noise; moisture uses the same parameters but with `seed + 1` to produce
    /// an independent field.
    pub fn generate(&mut self, seed: u64) {
        let mut config = BiomeNoiseConfig::default();
        config.seed = seed;

        let height_noise = NoiseLayer::new(seed);
        let moisture_noise = NoiseLayer::new(seed.wrapping_add(1));

        for y in 0..self.height {
            for x in 0..self.width {
                let xf = x as f32;
                let yf = y as f32;

                // Height: remap from [-1,1] → [0,1].
                let raw_h = height_noise.fbm(xf, yf, &config);
                let h = (raw_h + 1.0) * 0.5;
                let h = h.clamp(0.0, 1.0);

                // Moisture: independent noise, remapped to [0,1].
                let raw_m = moisture_noise.fbm(xf, yf, &config);
                let m = (raw_m + 1.0) * 0.5;
                let m = m.clamp(0.0, 1.0);

                let idx = y * self.width + x;
                self.height_data[idx] = h;
                self.biome_data[idx] = BiomeClassifier::classify(h, m);
            }
        }
    }

    /// Look up the height at `(x, y)` (returns `None` on out-of-bounds).
    pub fn height_at(&self, x: usize, y: usize) -> Option<f32> {
        if x < self.width && y < self.height {
            Some(self.height_data[y * self.width + x])
        } else {
            None
        }
    }

    /// Look up the biome code at `(x, y)`.
    pub fn biome_at(&self, x: usize, y: usize) -> Option<u8> {
        if x < self.width && y < self.height {
            Some(self.biome_data[y * self.width + x])
        } else {
            None
        }
    }

    /// Arithmetic mean of all height values.
    pub fn average_height(&self) -> f32 {
        if self.height_data.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.height_data.iter().sum();
        sum / self.height_data.len() as f32
    }

    /// Fraction of cells classified as the given `biome` (0.0 – 1.0).
    pub fn biome_coverage(&self, biome: u8) -> f32 {
        if self.biome_data.is_empty() {
            return 0.0;
        }
        let count = self.biome_data.iter().filter(|&&b| b == biome).count();
        count as f32 / self.biome_data.len() as f32
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── 1. Height map values are in [0.0, 1.0] ──────────────────────────
    #[test]
    fn generate_height_in_range() {
        let mut map = TerrainBiomeMap::new(64, 64);
        map.generate(42);
        for &h in &map.height_data {
            assert!(h >= 0.0 && h <= 1.0, "height out of range: {h}");
        }
    }

    // ── 2. Biome codes are valid ────────────────────────────────────────
    #[test]
    fn generate_valid_biome_codes() {
        let mut map = TerrainBiomeMap::new(128, 128);
        map.generate(123);
        for &b in &map.biome_data {
            assert!(b <= BIOME_SWAMP, "invalid biome code: {b}");
        }
    }

    // ── 3. Ocean biome appears for low heights ──────────────────────────
    #[test]
    fn classifier_ocean_for_low_height() {
        assert_eq!(BiomeClassifier::classify(0.0, 0.5), BIOME_OCEAN);
        assert_eq!(BiomeClassifier::classify(0.05, 0.5), BIOME_OCEAN);
    }

    // ── 4. Beach biome for coastal heights ──────────────────────────────
    #[test]
    fn classifier_beach_for_coast() {
        assert_eq!(BiomeClassifier::classify(0.12, 0.5), BIOME_BEACH);
    }

    // ── 5. Snow biome for extreme heights ───────────────────────────────
    #[test]
    fn classifier_snow_for_peak() {
        assert_eq!(BiomeClassifier::classify(0.95, 0.5), BIOME_SNOW);
    }

    // ── 6. Desert vs forest based on moisture at same height ────────────
    #[test]
    fn classifier_moisture_determines_biome() {
        let h = 0.40; // midland
        assert_eq!(BiomeClassifier::classify(h, 0.10), BIOME_DESERT);
        assert_eq!(BiomeClassifier::classify(h, 0.35), BIOME_GRASSLAND);
        assert_eq!(BiomeClassifier::classify(h, 0.60), BIOME_FOREST);
        assert_eq!(BiomeClassifier::classify(h, 0.85), BIOME_SWAMP);
    }

    // ── 7. Noise layer is deterministic ─────────────────────────────────
    #[test]
    fn noise_deterministic() {
        let n = NoiseLayer::new(777);
        let a = n.sample(3.5, 7.2);
        let b = n.sample(3.5, 7.2);
        assert!((a - b).abs() < f32::EPSILON);
    }

    // ── 8. Different seeds produce different noise ──────────────────────
    #[test]
    fn different_seeds_differ() {
        let n1 = NoiseLayer::new(1);
        let n2 = NoiseLayer::new(2);
        let v1 = n1.sample(10.0, 10.0);
        let v2 = n2.sample(10.0, 10.0);
        assert!((v1 - v2).abs() > f32::EPSILON);
    }

    // ── 9. Average height is reasonable (not 0, in [0,1]) ───────────────
    #[test]
    fn average_height_reasonable() {
        let mut map = TerrainBiomeMap::new(256, 256);
        map.generate(999);
        let avg = map.average_height();
        assert!(avg > 0.0 && avg < 1.0, "average height out of range: {avg}");
    }

    // ── 10. Biome coverage sums to ≤ 1.0 and covers all cells ───────────
    #[test]
    fn biome_coverage_covers_all_cells() {
        let mut map = TerrainBiomeMap::new(100, 100);
        map.generate(55);
        let total: f32 = (0..=BIOME_SWAMP)
            .map(|b| map.biome_coverage(b))
            .sum();
        assert!(
            (total - 1.0).abs() < f32::EPSILON,
            "biome coverages should sum to 1.0, got {total}"
        );
    }

    // ── Bonus: height_at / biome_at in-bounds & out-of-bounds ───────────
    #[test]
    fn accessors_in_bounds_and_out() {
        let mut map = TerrainBiomeMap::new(10, 10);
        map.generate(1);
        assert!(map.height_at(0, 0).is_some());
        assert!(map.biome_at(9, 9).is_some());
        assert!(map.height_at(10, 0).is_none());
        assert!(map.biome_at(0, 10).is_none());
    }
}
