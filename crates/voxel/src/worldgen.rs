//! Deterministic voxel world generation for Civis strata and hydrology.

use crate::material::{AIR, BEDROCK, DIRT, ORE, PLANT, STONE, WATER};
use crate::MaterialId;

/// Dense voxel world returned by the generator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenWorld {
    /// World dimensions in `[x, y, z]` order.
    pub dims: [usize; 3],
    /// Dense cell storage in `x + y * dx + z * dx * dy` order, with Y up.
    pub cells: Vec<MaterialId>,
}

/// Spatial frequency of the terrain noise, in noise-cells across the world.
/// The previous generator sampled fbm over the [0,1] domain, so the lowest octave
/// spanned the WHOLE map as a single broad dome -> a flat-looking plateau. Sampling
/// over [0, TERRAIN_FREQ] puts several hills/valleys across the map so the (proven)
/// smooth mesher has real relief to round.
const TERRAIN_FREQ: f64 = 5.0;

/// Returns the sea level in voxel units for the world.
#[must_use]
pub const fn sea_level(dims: [usize; 3]) -> usize {
    dims[1].saturating_mul(40) / 100
}

/// Returns the deterministic surface height for a world column.
#[must_use]
pub fn surface_height(dims: [usize; 3], seed: u64, x: usize, z: usize) -> usize {
    let dx = dims[0].max(1);
    let dz = dims[2].max(1);
    let _sea = dims[1] as f64 * 0.40;
    // Base sits a touch BELOW sea level so that low-relief regions form genuine
    // interior lakes/seas (water fills above the seabed up to sea level), while
    // hills still rise well above it. Previously base (0.50H) was above sea and
    // relief was all-positive, so the surface was ALWAYS >= sea and almost no
    // water ever generated — the "lack of visible water" bug.
    let base = dims[1] as f64 * 0.36;
    let u = x as f64 / dx as f64;
    let v = z as f64 / dz as f64;
    let noise = fbm2(seed, u * TERRAIN_FREQ, v * TERRAIN_FREQ);
    // Signed relief in ~[-1, 1]: valleys dip below base (and below sea -> water),
    // hills rise above it, so coasts and basins emerge across the interior.
    let relief = noise.clamp(-1.0, 1.0);
    let amplitude = (dims[1] as f64 * 0.34).max(2.0);
    let land = base + relief * amplitude;
    // Radial falloff (1 at centre -> 0 at edges) drags elevation toward a sub-sea
    // floor at the boundary so the world is ringed by ocean, not a cliff.
    let falloff = edge_falloff(u, v);
    let edge_floor = dims[1] as f64 * 0.30; // below sea -> boundary ocean
    let height = edge_floor + (land - edge_floor) * falloff;
    let max_surface = (dims[1] as f64 * 0.92).max(2.0);
    height.clamp(2.0, max_surface) as usize
}

/// Smooth radial falloff in `[0, 1]`: ~1 across the interior, easing to 0 at the
/// world edge so elevation above sea level tapers to a sloped coastline.
fn edge_falloff(u: f64, v: f64) -> f64 {
    // Distance from centre in normalized [-1, 1] space, taken on the dominant axis
    // so square worlds taper evenly on all four sides.
    let du = (u - 0.5).abs() * 2.0;
    let dv = (v - 0.5).abs() * 2.0;
    let edge = du.max(dv).clamp(0.0, 1.0);
    // Flat interior until ~0.7 out, then smooth taper to 0 at the boundary.
    let t = ((edge - 0.7) / 0.3).clamp(0.0, 1.0);
    1.0 - smoothstep(t)
}

/// Generates a deterministic world with strata, water fill, and ore pockets.
#[must_use]
pub fn generate(dims: [usize; 3], seed: u64) -> GenWorld {
    let mut cells = vec![AIR; dims[0] * dims[1] * dims[2]];
    let sea_level = sea_level(dims);
    for z in 0..dims[2] {
        for x in 0..dims[0] {
            carve_column(&mut cells, dims, seed, sea_level, x, z);
        }
    }
    GenWorld { dims, cells }
}

fn carve_column(
    cells: &mut [MaterialId],
    dims: [usize; 3],
    seed: u64,
    sea: usize,
    x: usize,
    z: usize,
) {
    let surface = surface_height(dims, seed, x, z);
    let soil = soil_depth(seed, x, z, dims[1]).clamp(1, 2);
    let ore_seed = mix3(seed ^ 0x9e37_79b9_7f4a_7c15, x as u64, z as u64);
    let params = CellMaterialParams {
        dims,
        seed,
        sea,
        surface,
        soil,
        ore_seed,
    };
    for y in 0..dims[1] {
        let idx = index(dims, x, y, z);
        cells[idx] = cell_material(&params, x, y, z);
    }
}

#[derive(Copy, Clone)]
struct CellMaterialParams {
    dims: [usize; 3],
    seed: u64,
    sea: usize,
    surface: usize,
    soil: usize,
    ore_seed: u64,
}

fn cell_material(params: &CellMaterialParams, x: usize, y: usize, z: usize) -> MaterialId {
    let CellMaterialParams {
        dims,
        seed,
        sea,
        surface,
        soil,
        ore_seed,
    } = *params;
    if is_bedrock_shell(dims, x, y, z) || y < bedrock_depth(dims[1]) {
        return BEDROCK;
    }
    if y < surface.saturating_sub(soil).max(1) {
        return stone_or_ore(seed, ore_seed, x, y, z);
    }
    if y + 1 == surface {
        return surface_cover(seed, x, z);
    }
    if soil >= 2 && y + 2 == surface {
        return DIRT;
    }
    // Only fill with water if this cell is above the terrain surface AND below sea
    // level. Columns where surface >= sea get no water — prevents flat blue slabs
    // floating over elevated terrain.
    if y > surface && y <= sea {
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

fn is_bedrock_shell(_dims: [usize; 3], _x: usize, y: usize, _z: usize) -> bool {
    y == 0
}

fn surface_cover(seed: u64, x: usize, z: usize) -> MaterialId {
    let v = hash2(seed ^ 0xabcd_ef01_2345_6789, x as u64, z as u64);
    if v & 0b1111 < 0x0d {
        PLANT
    } else {
        DIRT
    }
}

fn index(dims: [usize; 3], x: usize, y: usize, z: usize) -> usize {
    x + y * dims[0] + z * dims[0] * dims[1]
}

fn fbm2(seed: u64, x: f64, z: f64) -> f64 {
    let seed_basis = splitmix64(seed ^ 0xa5a5_5a5a_1234_5678);
    let off_x = ((seed_basis & 0xffff_ffff) as f64) / (u32::MAX as f64) * 1000.0;
    let off_z = (splitmix64(seed_basis) & 0xffff_ffff) as f64 / (u32::MAX as f64) * 1000.0;
    let freq_mix = (splitmix64(seed_basis ^ 0x9e37_79b9_7f4a_7c15) & 0x1f) as f64;
    let freq_scale = 0.5 + (freq_mix / 31.0) * 1.5;
    let x = x + off_x * 0.001;
    let z = z + off_z * 0.001;
    let mut total = 0.0;
    let mut amp = 1.0;
    let mut freq = freq_scale;
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
    use crate::material::{MaterialRegistry, BEDROCK, PLANT};

    fn dims() -> [usize; 3] {
        [32, 48, 32]
    }

    #[test]
    fn deterministic() {
        let a = generate(dims(), 7);
        let b = generate(dims(), 7);
        assert_eq!(a, b);
    }

    /// Water must only appear in columns where terrain surface is below sea level.
    /// Regression test for the flat-blue-slab bug where water filled all y<=sea
    /// regardless of terrain height, producing water planes over elevated land.
    #[test]
    fn water_only_fills_depressions() {
        let dims = [64usize, 32, 64];
        let seed = 42u64;
        let w = generate(dims, seed);
        let sea = sea_level(dims);
        let mut violations = 0usize;
        for z in 0..dims[2] {
            for x in 0..dims[0] {
                let surface = surface_height(dims, seed, x, z);
                if surface >= sea {
                    // Column is above sea — must have no WATER cells at all
                    for y in 0..dims[1] {
                        if w.cells[index(dims, x, y, z)] == WATER {
                            violations += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(
            violations, 0,
            "found {violations} WATER cells in above-sea columns (flat-slab regression)"
        );
    }

    #[test]
    fn worldgen_emits_water() {
        let w = generate([96, 64, 96], 7);
        let water = w.cells.iter().filter(|c| **c == WATER).count();
        let total = w.cells.len();
        println!(
            "[worldgen] WATER voxels = {water} / {total} ({:.1}%)",
            100.0 * water as f64 / total as f64
        );
        assert!(water > 0, "worldgen emitted no WATER voxels");
        // Sloped coasts + sea level should fill a non-trivial fraction.
        assert!(
            water > total / 1000,
            "expected meaningful water coverage, got {water}/{total}"
        );
    }

    /// surface_height relief invariant (#4 worldgen): the heightmap must VARY across
    /// the map (rolling hills, not a flat plateau) and be deterministic per seed.
    #[test]
    fn surface_height_has_relief_and_is_deterministic() {
        let d = [96, 64, 96];
        let mut min_h = usize::MAX;
        let mut max_h = 0usize;
        for z in (0..d[2]).step_by(4) {
            for x in (0..d[0]).step_by(4) {
                let h = surface_height(d, 11, x, z);
                min_h = min_h.min(h);
                max_h = max_h.max(h);
                // determinism: same (seed,x,z) -> same height.
                assert_eq!(h, surface_height(d, 11, x, z));
            }
        }
        println!(
            "[worldgen] surface_height range = {min_h}..={max_h} (relief = {})",
            max_h - min_h
        );
        assert!(
            max_h > min_h,
            "surface is flat — no relief (min={min_h} max={max_h})"
        );
        // Meaningful relief, not a 1-voxel ripple, relative to the 64-tall world.
        assert!(max_h - min_h >= 6, "relief too small: {}", max_h - min_h);
    }

    #[test]
    fn different_seeds_produce_different_terrain() {
        let dims = [64usize, 32usize, 64usize];
        let mut total = 0usize;
        let mut changed = 0usize;
        for z in 0..dims[2] {
            for x in 0..dims[0] {
                let h1 = surface_height(dims, 1, x, z);
                let h2 = surface_height(dims, 999, x, z);
                if h1 != h2 {
                    changed += 1;
                }
                total += 1;
            }
        }

        let percent = (changed as f64 / total as f64) * 100.0;
        println!("[worldgen] different-seed terrain diff = {changed}/{total} ({percent:.2}%)");
        assert!(
            percent >= 20.0,
            "expected >=20% differing surface cells, found {percent:.2}%"
        );
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
                saw_soil |= m == DIRT || m == PLANT;
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
        let sea = sea_level(w.dims);
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

    /// Measured guard for the "lack of visible water" bug: water must be a
    /// meaningful fraction of the world, not a 1-voxel edge sliver. Averaged over
    /// several seeds so it does not hinge on one lucky map.
    #[test]
    fn water_is_a_meaningful_fraction_of_the_world() {
        let mut total_water = 0usize;
        let mut total_non_air = 0usize;
        for seed in [1u64, 7, 19, 101, 4242] {
            let w = generate(dims(), seed);
            for &m in &w.cells {
                if m != AIR {
                    total_non_air += 1;
                    if m == WATER {
                        total_water += 1;
                    }
                }
            }
        }
        let frac = total_water as f64 / total_non_air.max(1) as f64;
        assert!(
            frac > 0.05,
            "water is only {:.1}% of solid cells — worldgen barely floods (lack-of-visible-water regression)",
            frac * 100.0
        );
    }

    /// Water must have an exposed surface (AIR directly above some water cell) so
    /// it actually renders as a visible water plane, not a buried pocket.
    #[test]
    fn water_has_an_exposed_surface() {
        let w = generate(dims(), 19);
        let mut exposed = false;
        for z in 0..w.dims[2] {
            for y in 0..w.dims[1].saturating_sub(1) {
                for x in 0..w.dims[0] {
                    if w.cells[index(w.dims, x, y, z)] == WATER
                        && w.cells[index(w.dims, x, y + 1, z)] == AIR
                    {
                        exposed = true;
                    }
                }
            }
        }
        assert!(
            exposed,
            "no water cell has AIR above it — water is fully buried (invisible)"
        );
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

    #[test]
    fn edges_are_not_full_height_bedrock() {
        let w = generate(dims(), 41);
        for z in 0..w.dims[2] {
            for y in (1..w.dims[1]).rev() {
                let m0 = w.cells[index(w.dims, 0, y, z)];
                if m0 == AIR {
                    continue;
                }
                assert_ne!(m0, BEDROCK);
                break;
            }
            for y in (1..w.dims[1]).rev() {
                let m1 = w.cells[index(w.dims, w.dims[0] - 1, y, z)];
                if m1 == AIR {
                    continue;
                }
                assert_ne!(m1, BEDROCK);
                break;
            }
        }
        for x in 0..w.dims[0] {
            for y in (1..w.dims[1]).rev() {
                let m0 = w.cells[index(w.dims, x, y, 0)];
                if m0 == AIR {
                    continue;
                }
                assert_ne!(m0, BEDROCK);
                break;
            }
            for y in (1..w.dims[1]).rev() {
                let m1 = w.cells[index(w.dims, x, y, w.dims[2] - 1)];
                if m1 == AIR {
                    continue;
                }
                assert_ne!(m1, BEDROCK);
                break;
            }
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
