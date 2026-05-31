//! Deterministic voxel world generation for Civis strata and hydrology.

use crate::material::{AIR, BEDROCK, DIRT, GRAVEL, ORE, STONE, WATER};
use crate::MaterialId;

/// Dense voxel world returned by the generator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenWorld {
    /// World dimensions in `[x, y, z]` order.
    pub dims: [usize; 3],
    /// Dense cell storage in `x + y * dx + z * dx * dy` order, with Y up.
    pub cells: Vec<MaterialId>,
}

/// Returns the deterministic surface height for a world column.
#[must_use]
pub fn surface_height(dims: [usize; 3], seed: u64, x: usize, z: usize) -> usize {
    let dx = dims[0].max(1);
    let dz = dims[2].max(1);
    let base = dims[1].saturating_mul(45) / 100;
    let noise = fbm2(seed, x as f64 / dx as f64, z as f64 / dz as f64);
    let span = (dims[1].saturating_sub(4)).max(1) as f64;
    let height = base as f64 + noise * span * 0.22;
    height.clamp(2.0, (dims[1].saturating_sub(2)).max(2) as f64) as usize
}

/// Generates a deterministic world with strata, water fill, and ore pockets.
#[must_use]
pub fn generate(dims: [usize; 3], seed: u64) -> GenWorld {
    let mut cells = vec![AIR; dims[0] * dims[1] * dims[2]];
    let sea_level = dims[1].saturating_mul(40) / 100;
    for z in 0..dims[2] {
        for x in 0..dims[0] {
            carve_column(&mut cells, dims, seed, sea_level, x, z);
        }
    }
    GenWorld { dims, cells }
}

fn carve_column(cells: &mut [MaterialId], dims: [usize; 3], seed: u64, sea: usize, x: usize, z: usize) {
    let surface = surface_height(dims, seed, x, z);
    let soil = soil_depth(seed, x, z, dims[1]);
    let ore_seed = mix3(seed ^ 0x9e37_79b9_7f4a_7c15, x as u64, z as u64);
    for y in 0..dims[1] {
        let idx = index(dims, x, y, z);
        cells[idx] = cell_material(dims, seed, sea, surface, soil, x, y, z, ore_seed);
    }
}

fn cell_material(
    dims: [usize; 3],
    seed: u64,
    sea: usize,
    surface: usize,
    soil: usize,
    x: usize,
    y: usize,
    z: usize,
    ore_seed: u64,
) -> MaterialId {
    if is_bedrock_shell(dims, x, y, z) || y < bedrock_depth(dims[1]) {
        return BEDROCK;
    }
    if y < surface.saturating_sub(soil).max(1) {
        return stone_or_ore(seed, ore_seed, x, y, z);
    }
    if y < surface {
        return if (x ^ y ^ z) & 1 == 0 { DIRT } else { GRAVEL };
    }
    if y <= sea {
        return WATER;
    }
    AIR
}

fn bedrock_depth(height: usize) -> usize {
    // Keep a thicker, deterministic bedrock base so mid-map columns always
    // expose BEDROCK below STONE below soil when scanned upward.
    (height / 16).clamp(2, 4)
}

fn stone_or_ore(seed: u64, ore_seed: u64, x: usize, y: usize, z: usize) -> MaterialId {
    if ore_pocket(seed, ore_seed, x, y, z) {
        ORE
    } else {
        STONE
    }
}

fn ore_pocket(seed: u64, ore_seed: u64, x: usize, y: usize, z: usize) -> bool {
    let n = hash3(ore_seed, x as u64, y as u64, z as u64);
    let band = hash3(seed ^ 0x4d59_5df4_d0f3_3173, x as u64, y as u64, z as u64);
    n % 29 == 0 && band % 5 != 0
}

fn soil_depth(seed: u64, x: usize, z: usize, height: usize) -> usize {
    let v = hash2(seed ^ 0x6a09_e667_f3bc_c909, x as u64, z as u64);
    let cap = (height / 12).clamp(2, 5);
    2 + (v as usize % cap)
}

fn is_bedrock_shell(dims: [usize; 3], x: usize, y: usize, z: usize) -> bool {
    y == 0 || x == 0 || z == 0 || x + 1 == dims[0] || z + 1 == dims[2]
}

fn index(dims: [usize; 3], x: usize, y: usize, z: usize) -> usize {
    x + y * dims[0] + z * dims[0] * dims[1]
}

fn fbm2(seed: u64, x: f64, z: f64) -> f64 {
    let mut total = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut norm = 0.0;
    for octave in 0..4 {
        total += value_noise2(seed.wrapping_add(octave as u64), x * freq, z * freq) * amp;
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    total / norm.max(f64::EPSILON)
}

fn value_noise2(seed: u64, x: f64, z: f64) -> f64 {
    let xi = x.floor() as i64;
    let zi = z.floor() as i64;
    let xf = smoothstep(x - xi as f64);
    let zf = smoothstep(z - zi as f64);
    let v00 = hash01(seed, xi, zi);
    let v10 = hash01(seed, xi + 1, zi);
    let v01 = hash01(seed, xi, zi + 1);
    let v11 = hash01(seed, xi + 1, zi + 1);
    let a = lerp(v00, v10, xf);
    let b = lerp(v01, v11, xf);
    lerp(a, b, zf) * 2.0 - 1.0
}

fn hash01(seed: u64, x: i64, z: i64) -> f64 {
    let h = mix3(seed, x as u64, z as u64);
    ((h >> 11) as f64) / ((1u64 << 53) as f64)
}

fn hash2(seed: u64, x: u64, z: u64) -> u64 {
    splitmix64(seed ^ mix3(0x1234_5678_9abc_def0, x, z))
}

fn hash3(seed: u64, x: u64, y: u64, z: u64) -> u64 {
    splitmix64(seed ^ mix3(x, y, z))
}

fn mix3(a: u64, b: u64, c: u64) -> u64 {
    let mut v = a ^ b.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    v ^= c.wrapping_add(0xbf58_476d_1ce4_e5b9);
    splitmix64(v)
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn smoothstep(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::{MaterialRegistry, BEDROCK};

    fn dims() -> [usize; 3] {
        [32, 48, 32]
    }

    #[test]
    fn deterministic() {
        let a = generate(dims(), 7);
        let b = generate(dims(), 7);
        assert_eq!(a, b);
    }

    #[test]
    fn different_seed_differs() {
        let a = generate(dims(), 1);
        let b = generate(dims(), 2);
        assert_ne!(a.cells, b.cells);
    }

    #[test]
    fn has_strata() {
        let w = generate(dims(), 11);
        let x = w.dims[0] / 2;
        let z = w.dims[2] / 2;
        let surface = surface_height(w.dims, 11, x, z);
        let mut saw_bedrock = false;
        let mut saw_stone = false;
        let mut saw_soil = false;
        for y in 0..w.dims[1] {
            let m = w.cells[index(w.dims, x, y, z)];
            if y == 0 {
                assert_eq!(m, BEDROCK);
            } else if y < surface.saturating_sub(2) {
                saw_bedrock |= m == BEDROCK;
                saw_stone |= m == STONE;
            } else if y < surface {
                saw_soil |= m == DIRT || m == GRAVEL;
            } else if y > surface {
                assert!(m == AIR || m == WATER);
            }
        }
        assert!(saw_stone);
        assert!(saw_soil);
        assert!(saw_bedrock || surface <= 3);
    }

    #[test]
    fn water_at_or_below_sea_level() {
        let w = generate(dims(), 19);
        let sea = w.dims[1] * 40 / 100;
        let mut water_found = false;
        for z in 0..w.dims[2] {
            for y in 0..w.dims[1] {
                for x in 0..w.dims[0] {
                    let m = w.cells[index(w.dims, x, y, z)];
                    if m == WATER {
                        water_found = true;
                        assert!(y <= sea);
                    }
                }
            }
        }
        assert!(water_found);
    }

    #[test]
    fn bedrock_floor() {
        let w = generate(dims(), 21);
        for z in 0..w.dims[2] {
            for x in 0..w.dims[0] {
                assert_eq!(w.cells[index(w.dims, x, 0, z)], BEDROCK);
            }
        }
    }

    #[test]
    fn valid_ids() {
        let w = generate(dims(), 13);
        let registry = MaterialRegistry::standard();
        for cell in w.cells {
            assert!(registry.get(cell).is_some(), "invalid id {}", cell.0);
        }
    }
}

// --- Streaming heightfield generator (FR-CIV-VOXEL-021) ---------------------
//
// Coexists with the dense `GenWorld`/`generate` path above. The dense path backs
// the non-streaming `voxel_sim`; `HeightFieldGen` below drives the chunk-streaming
// layer (`crate::stream::StreamingWorld`). Regenerated chunks are bit-identical for
// a fixed `(seed, coord)` so streaming and reloads stay deterministic.

use crate::stream::{WorldGen, CHUNK_EDGE, CHUNK_EDGE_I32};
use phenotype_voxel::{Chunk, ChunkCoord};

const CHUNK_VOXELS: usize = CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE;

/// Height-field generator with deterministic hash noise.
pub struct HeightFieldGen {
    /// World seed threaded into the hash.
    pub seed: u64,
    /// Base voxel size in metres.
    pub base_voxel_m: f32,
    /// Sea level in metres.
    pub sea_level_m: f32,
}

fn hf_mix64(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

fn hf_hash2(seed: u64, x: i32, z: i32) -> u32 {
    let x = x as u64;
    let z = z as u64;
    hf_mix64(seed ^ x.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ z.wrapping_mul(0xbf58_476d_1ce4_e5b9))
        as u32
}

fn hf_height_voxels(seed: u64, world_x: i32, world_z: i32, sea_level_voxels: i32) -> i32 {
    let coarse = (hf_hash2(seed, world_x >> 2, world_z >> 2) & 0xff) as i32 - 128;
    let fine = (hf_hash2(seed ^ 0xA5A5_A5A5_A5A5_A5A5, world_x, world_z) & 0x1f) as i32 - 16;
    sea_level_voxels + coarse / 2 + fine / 4
}

impl WorldGen for HeightFieldGen {
    fn generate(&self, coord: ChunkCoord) -> Chunk<MaterialId> {
        let sea_level_voxels = (self.sea_level_m / self.base_voxel_m).round() as i32;
        let mut voxels = vec![MaterialId(0); CHUNK_VOXELS];
        for lz in 0..CHUNK_EDGE {
            for ly in 0..CHUNK_EDGE {
                for lx in 0..CHUNK_EDGE {
                    let world_x = coord.cx * CHUNK_EDGE_I32 + lx as i32;
                    let world_y = coord.cy * CHUNK_EDGE_I32 + ly as i32;
                    let world_z = coord.cz * CHUNK_EDGE_I32 + lz as i32;
                    let height = hf_height_voxels(self.seed, world_x, world_z, sea_level_voxels);
                    let idx = lx + ly * CHUNK_EDGE + lz * CHUNK_EDGE * CHUNK_EDGE;
                    voxels[idx] = if world_y <= height {
                        MaterialId(1)
                    } else {
                        MaterialId(0)
                    };
                }
            }
        }
        Chunk { voxels }
    }
}
