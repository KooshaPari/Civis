//! SOTA procedural map generation for the Civis desktop reference client.
//!
//! Pipeline (deterministic, single fixed [`SEED`]):
//! 1. Continental mask from low-frequency domain-warped fBm (organic coasts,
//!    no radial falloff — only a gentle taper in the outer ~8% so the border
//!    is clean ocean).
//! 2. Elevation = continental base fBm + ridged-multifractal mountain RANGES
//!    biased along low-frequency tectonic bands + fine hill detail.
//! 3. Thermal erosion (talus-angle relaxation) to soften peaks and cut
//!    valleys/sediment, plus a cheap flow-accumulation pass for rivers.
//! 4. Sea level at [`WATER_LEVEL`]; ocean basins sit below it.
//! 5. Whittaker-style biomes (temperature x moisture) for per-vertex colour.
//!
//! The full height/biome field is computed once into a [`OnceLock`] cache so
//! [`terrain_height`] and the mesh builder share identical, deterministic data.

use std::sync::OnceLock;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// Grid resolution (vertices per side). 384 gives crisp relief on a 3090 Ti.
pub const GRID: usize = 384;
/// World extent in metres along X and Z (mesh spans 0..`WORLD_SIZE`).
pub const WORLD_SIZE: f32 = 256.0;
/// Maximum terrain altitude in metres — dramatic vertical for oblique cameras.
pub const HEIGHT_SCALE: f32 = 200.0;
/// Sea level: land rises clearly above, ocean basins sit below.
pub const WATER_LEVEL: f32 = 0.32 * HEIGHT_SCALE;

/// Fixed seed so generation is fully reproducible across runs.
const SEED: u32 = 0x5EED_C171;
/// Shallow-ocean band thickness below sea level (for coast classification).
const SHALLOW_BAND: f32 = 0.05 * HEIGHT_SCALE;

// ---------------------------------------------------------------------------
// Cached field
// ---------------------------------------------------------------------------

/// Fully generated terrain field: post-erosion heights + per-cell biome ids.
struct Field {
    height: Vec<f32>,
    biome: Vec<u8>,
}

static FIELD: OnceLock<Field> = OnceLock::new();

fn field() -> &'static Field {
    FIELD.get_or_init(generate_field)
}

#[inline]
fn idx(x: usize, z: usize) -> usize {
    z * GRID + x
}

// ---------------------------------------------------------------------------
// Public API (signatures preserved for decorations/sim_bridge/atmosphere/etc.)
// ---------------------------------------------------------------------------

/// Build the GRID x GRID terrain mesh with positions, finite-difference
/// normals, biome vertex colours, tiled UVs and indices.
pub fn terrain_mesh() -> Mesh {
    let f = field();
    let mut positions = Vec::with_capacity(GRID * GRID);
    let mut normals = Vec::with_capacity(GRID * GRID);
    let mut colors = Vec::with_capacity(GRID * GRID);
    let mut uvs = Vec::with_capacity(GRID * GRID);
    let half = WORLD_SIZE * 0.5;
    let cell = WORLD_SIZE / (GRID - 1) as f32;

    for z in 0..GRID {
        for x in 0..GRID {
            let h = f.height[idx(x, z)];
            let wx = x as f32 * cell;
            let wz = z as f32 * cell;
            positions.push([wx - half, h, wz - half]);
            normals.push(fd_normal(x, z, cell));
            colors.push(biome_color(f.biome[idx(x, z)], h));
            uvs.push([wx / 16.0, wz / 16.0]);
        }
    }

    let mut indices = Vec::with_capacity((GRID - 1) * (GRID - 1) * 6);
    for z in 0..GRID - 1 {
        for x in 0..GRID - 1 {
            let i = (z * GRID + x) as u32;
            let g = GRID as u32;
            indices.extend_from_slice(&[i, i + g, i + 1, i + 1, i + g, i + g + 1]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Sample procedural terrain surface height at mesh-space XZ (0..[`WORLD_SIZE`]).
#[must_use]
pub fn terrain_surface_y(x: f32, z: f32) -> f32 {
    terrain_height(x.clamp(0.0, WORLD_SIZE), z.clamp(0.0, WORLD_SIZE))
}

/// Final (post-erosion) terrain height at mesh-space XZ, bilinearly sampled
/// from the cached field so decorations/civilians seat exactly on the surface.
pub fn terrain_height(x: f32, z: f32) -> f32 {
    let f = field();
    let cell = WORLD_SIZE / (GRID - 1) as f32;
    let gx = (x / cell).clamp(0.0, (GRID - 1) as f32);
    let gz = (z / cell).clamp(0.0, (GRID - 1) as f32);
    let x0 = gx.floor() as usize;
    let z0 = gz.floor() as usize;
    let x1 = (x0 + 1).min(GRID - 1);
    let z1 = (z0 + 1).min(GRID - 1);
    let tx = gx - x0 as f32;
    let tz = gz - z0 as f32;
    let a = lerp(f.height[idx(x0, z0)], f.height[idx(x1, z0)], tx);
    let b = lerp(f.height[idx(x0, z1)], f.height[idx(x1, z1)], tx);
    lerp(a, b, tz)
}

/// Legacy height-banded palette, kept exported for callers that colour by
/// elevation alone (e.g. materials gradient documentation). Biome colouring
/// in the mesh uses [`biome_color`] instead.
/// Map terrain elevation to a PBR [`crate::materials::Biome`] height band.
///
/// Only compiled with `pbr-textures`; uses the same normalised thresholds as
/// [`crate::materials::Biome::from_height_norm`].
#[cfg(feature = "pbr-textures")]
#[must_use]
pub fn pbr_biome_at_height(height: f32) -> crate::materials::Biome {
    crate::materials::Biome::from_height_norm(height / HEIGHT_SCALE)
}

pub fn color_for_height(height: f32) -> [f32; 4] {
    let t = height / HEIGHT_SCALE;
    if t < WATER_LEVEL / HEIGHT_SCALE {
        [0.20, 0.40, 0.86, 1.0]
    } else if t < 0.36 {
        [0.86, 0.78, 0.52, 1.0]
    } else if t < 0.56 {
        [0.28, 0.58, 0.24, 1.0]
    } else if t < 0.72 {
        [0.12, 0.34, 0.12, 1.0]
    } else if t < 0.88 {
        [0.50, 0.50, 0.52, 1.0]
    } else {
        [0.97, 0.97, 0.97, 1.0]
    }
}

// ---------------------------------------------------------------------------
// Field generation pipeline
// ---------------------------------------------------------------------------

fn generate_field() -> Field {
    let mut height = vec![0.0_f32; GRID * GRID];
    let cell = WORLD_SIZE / (GRID - 1) as f32;
    for z in 0..GRID {
        for x in 0..GRID {
            let wx = x as f32 * cell;
            let wz = z as f32 * cell;
            height[idx(x, z)] = raw_elevation(wx, wz);
        }
    }
    erode_thermal(&mut height, 28);
    let flow = flow_accumulation(&height);
    carve_rivers(&mut height, &flow);
    let biome = classify_all(&height, &flow);
    Field { height, biome }
}

/// Raw (pre-erosion) elevation in metres for mesh-space world XZ.
fn raw_elevation(wx: f32, wz: f32) -> f32 {
    let nx = wx / WORLD_SIZE;
    let nz = wz / WORLD_SIZE;
    let mask = continental_mask(nx, nz);
    if mask <= 0.0 {
        // Ocean floor: dip below sea level, deeper further from shore.
        return (WATER_LEVEL - SHALLOW_BAND - (-mask) * 0.22 * HEIGHT_SCALE).max(0.0);
    }
    let base = fbm(nx * 4.5, nz * 4.5, 7, 2.0, 0.5, SEED ^ 0x1111);
    let hills = fbm(nx * 16.0, nz * 16.0, 4, 2.0, 0.5, SEED ^ 0x2222) * 0.18;
    let mountains = ridged_ranges(nx, nz);
    // Land sits above sea level; lowlands dominate, ranges spike upward.
    let land01 = (base * 0.45 + hills + mountains).clamp(0.0, 1.0);
    let coast = mask.min(0.18) / 0.18; // ramp up from shoreline over a margin
    WATER_LEVEL + coast * land01 * (HEIGHT_SCALE - WATER_LEVEL)
}

/// Continental land/ocean field via domain-warped low-frequency fBm.
/// Returns >0 on land (magnitude ~= inland distance), <=0 on ocean.
fn continental_mask(nx: f32, nz: f32) -> f32 {
    let warp = 0.55;
    let ax = fbm(nx * 2.0 + 5.2, nz * 2.0 + 1.3, 4, 2.0, 0.5, SEED ^ 0xA1) - 0.5;
    let az = fbm(nx * 2.0 + 9.7, nz * 2.0 + 4.8, 4, 2.0, 0.5, SEED ^ 0xB2) - 0.5;
    let sx = nx + warp * ax;
    let sz = nz + warp * az;
    let c = fbm(sx * 3.0, sz * 3.0, 6, 2.0, 0.5, SEED ^ 0xC3);
    // Sea-level threshold tuned for ~55-65% land before the edge taper.
    let mut m = c - 0.46;
    // Gentle ocean taper in the outermost ~8% (clean border).
    let edge = edge_falloff(nx, nz);
    m -= (1.0 - edge) * 0.6;
    m
}

/// 1.0 in the interior, ramping to 0 across the outer ~8% of each axis.
fn edge_falloff(nx: f32, nz: f32) -> f32 {
    let m = 0.08;
    let ex = (nx / m).min((1.0 - nx) / m).clamp(0.0, 1.0);
    let ez = (nz / m).min((1.0 - nz) / m).clamp(0.0, 1.0);
    smooth(ex.min(ez))
}

/// Ridged multifractal mountains, biased to lie along low-frequency tectonic
/// bands so ranges form distinct regions rather than blanketing the map.
fn ridged_ranges(nx: f32, nz: f32) -> f32 {
    let warp = fbm(nx * 1.5 + 2.0, nz * 1.5 + 7.0, 3, 2.0, 0.5, SEED ^ 0xD4) - 0.5;
    let band = fbm(nx * 2.2 + warp, nz * 2.2 - warp, 3, 2.0, 0.5, SEED ^ 0xE5);
    let belt = ((band - 0.5).abs() * 4.0).clamp(0.0, 1.0); // 0 on the ridge axis
    let tectonic = (1.0 - belt).powi(2); // strongest along the band centre
    let ridge = ridged_fbm(nx * 9.0, nz * 9.0, 6, SEED ^ 0xF6);
    ridge * tectonic * 0.85
}

/// Thermal erosion: iteratively shed material from cells whose drop to the
/// lowest neighbour exceeds the talus angle, depositing into that neighbour.
fn erode_thermal(height: &mut [f32], iterations: usize) {
    let cell = WORLD_SIZE / (GRID - 1) as f32;
    let talus = 0.9 * cell; // max stable height difference per cell
    let factor = 0.45;
    let mut delta = vec![0.0_f32; height.len()];
    for _ in 0..iterations {
        delta.iter_mut().for_each(|d| *d = 0.0);
        for z in 1..GRID - 1 {
            for x in 1..GRID - 1 {
                erode_cell(height, &mut delta, x, z, talus, factor);
            }
        }
        for i in 0..height.len() {
            height[i] = (height[i] + delta[i]).max(0.0);
        }
    }
}

/// One cell's thermal contribution: push excess toward the steepest-down
/// neighbour (4-connected) past the talus threshold.
fn erode_cell(h: &[f32], delta: &mut [f32], x: usize, z: usize, talus: f32, factor: f32) {
    let here = h[idx(x, z)];
    let nbrs = [idx(x - 1, z), idx(x + 1, z), idx(x, z - 1), idx(x, z + 1)];
    let mut lowest = usize::MAX;
    let mut max_drop = talus;
    for &n in &nbrs {
        let drop = here - h[n];
        if drop > max_drop {
            max_drop = drop;
            lowest = n;
        }
    }
    if lowest != usize::MAX {
        let move_amt = (max_drop - talus) * factor;
        delta[idx(x, z)] -= move_amt;
        delta[lowest] += move_amt;
    }
}

/// Cheap flow accumulation: each land cell sends one unit downhill along the
/// steepest descent; summed arrivals approximate drainage for river tinting.
fn flow_accumulation(height: &[f32]) -> Vec<f32> {
    let mut order: Vec<usize> = (0..height.len()).collect();
    order.sort_unstable_by(|&a, &b| height[b].partial_cmp(&height[a]).unwrap());
    let mut flow = vec![1.0_f32; height.len()];
    for &i in &order {
        let (x, z) = (i % GRID, i / GRID);
        if x == 0 || z == 0 || x == GRID - 1 || z == GRID - 1 {
            continue;
        }
        if let Some(low) = steepest_down(height, x, z) {
            flow[low] += flow[i];
        }
    }
    flow
}

/// Index of the lowest 4-connected neighbour strictly below `(x,z)`, if any.
fn steepest_down(h: &[f32], x: usize, z: usize) -> Option<usize> {
    let here = h[idx(x, z)];
    let nbrs = [idx(x - 1, z), idx(x + 1, z), idx(x, z - 1), idx(x, z + 1)];
    let mut best = None;
    let mut best_h = here;
    for &n in &nbrs {
        if h[n] < best_h {
            best_h = h[n];
            best = Some(n);
        }
    }
    best
}

/// Slightly carve channels where drainage is high and the cell is above water.
fn carve_rivers(height: &mut [f32], flow: &[f32]) {
    for i in 0..height.len() {
        if height[i] > WATER_LEVEL && flow[i] > RIVER_FLOW {
            let carve = ((flow[i] - RIVER_FLOW) / RIVER_FLOW).min(1.0) * 2.5;
            height[i] = (height[i] - carve).max(WATER_LEVEL - 0.5);
        }
    }
}

/// Drainage threshold (cells) above which a vertex reads as a river.
const RIVER_FLOW: f32 = 140.0;

// ---------------------------------------------------------------------------
// Biomes (Whittaker-style: temperature x moisture)
// ---------------------------------------------------------------------------

// Biome ids.
const B_DEEP_OCEAN: u8 = 0;
const B_SHALLOW_OCEAN: u8 = 1;
const B_BEACH: u8 = 2;
const B_DESERT: u8 = 3;
const B_SAVANNA: u8 = 4;
const B_GRASSLAND: u8 = 5;
const B_FOREST: u8 = 6;
const B_TAIGA: u8 = 7;
const B_TUNDRA: u8 = 8;
const B_ROCK: u8 = 9;
const B_SNOW: u8 = 10;
const B_RIVER: u8 = 11;

fn classify_all(height: &[f32], flow: &[f32]) -> Vec<u8> {
    let mut out = vec![0_u8; height.len()];
    for z in 0..GRID {
        for x in 0..GRID {
            let i = idx(x, z);
            out[i] = classify_biome(x, z, height[i], flow[i], height);
        }
    }
    out
}

/// Classify one cell by temperature (latitude - altitude lapse) and moisture
/// (proximity to water + prevailing-wind noise).
fn classify_biome(x: usize, z: usize, h: f32, flow: f32, height: &[f32]) -> u8 {
    if h < WATER_LEVEL - SHALLOW_BAND {
        return B_DEEP_OCEAN;
    }
    if h < WATER_LEVEL {
        return B_SHALLOW_OCEAN;
    }
    if h < WATER_LEVEL + 1.5 {
        return B_BEACH;
    }
    if h > WATER_LEVEL && flow > RIVER_FLOW {
        return B_RIVER;
    }
    let nx = x as f32 / (GRID - 1) as f32;
    let nz = z as f32 / (GRID - 1) as f32;
    let alt = (h - WATER_LEVEL) / (HEIGHT_SCALE - WATER_LEVEL); // 0..1 above sea
    let lat = (nz - 0.5).abs() * 2.0; // 0 at equator, 1 at poles
    let temp =
        (1.0 - lat) - alt * 0.85 + (fbm(nx * 6.0, nz * 6.0, 3, 2.0, 0.5, SEED ^ 0x77) - 0.5) * 0.15;
    let moist = moisture(nx, nz, height);
    biome_from_climate(temp, moist, alt)
}

/// Moisture proxy: distance to nearby water modulated by a prevailing-wind
/// noise field, both in 0..1 (wetter near coasts / windward).
fn moisture(nx: f32, nz: f32, height: &[f32]) -> f32 {
    let cell = WORLD_SIZE / (GRID - 1) as f32;
    let gx = (nx * (GRID - 1) as f32) as usize;
    let gz = (nz * (GRID - 1) as f32) as usize;
    let mut near = 1.0_f32;
    let r = 6;
    for dz in -r..=r {
        for dx in -r..=r {
            let sx = gx as i32 + dx;
            let sz = gz as i32 + dz;
            if sx < 0 || sz < 0 || sx >= GRID as i32 || sz >= GRID as i32 {
                continue;
            }
            if height[idx(sx as usize, sz as usize)] < WATER_LEVEL {
                let d = ((dx * dx + dz * dz) as f32).sqrt() * cell;
                near = near.min(d / (r as f32 * cell));
            }
        }
    }
    let wind = fbm(nx * 5.0 + 3.0, nz * 5.0, 4, 2.0, 0.5, SEED ^ 0x88);
    ((1.0 - near) * 0.6 + wind * 0.4).clamp(0.0, 1.0)
}

/// Whittaker lookup: pick a land biome from temperature, moisture, altitude.
fn biome_from_climate(temp: f32, moist: f32, alt: f32) -> u8 {
    if alt > 0.82 {
        return if temp < 0.35 { B_SNOW } else { B_ROCK };
    }
    if temp < 0.18 {
        return B_TUNDRA;
    }
    if temp < 0.4 {
        return if moist > 0.45 { B_TAIGA } else { B_TUNDRA };
    }
    if temp > 0.72 && moist < 0.3 {
        return B_DESERT;
    }
    if temp > 0.6 && moist < 0.45 {
        return B_SAVANNA;
    }
    if moist > 0.55 {
        return B_FOREST;
    }
    B_GRASSLAND
}

/// Per-vertex colour for a biome id (saturated but natural), with a subtle
/// altitude shade so relief reads even within a single biome.
fn biome_color(biome: u8, h: f32) -> [f32; 4] {
    let shade = 0.88 + 0.12 * (h / HEIGHT_SCALE);
    let c = match biome {
        B_DEEP_OCEAN => [0.06, 0.18, 0.42],
        B_SHALLOW_OCEAN => [0.16, 0.38, 0.70],
        B_BEACH => [0.84, 0.78, 0.56],
        B_DESERT => [0.80, 0.71, 0.40],
        B_SAVANNA => [0.66, 0.66, 0.32],
        B_GRASSLAND => [0.36, 0.62, 0.28],
        B_FOREST => [0.16, 0.42, 0.18],
        B_TAIGA => [0.20, 0.40, 0.32],
        B_TUNDRA => [0.62, 0.64, 0.58],
        B_ROCK => [0.46, 0.45, 0.44],
        B_SNOW => [0.95, 0.96, 0.98],
        B_RIVER => [0.22, 0.46, 0.78],
        _ => [0.5, 0.5, 0.5],
    };
    if biome <= B_SHALLOW_OCEAN {
        [c[0], c[1], c[2], 1.0]
    } else {
        [c[0] * shade, c[1] * shade, c[2] * shade, 1.0]
    }
}

// ---------------------------------------------------------------------------
// Normals
// ---------------------------------------------------------------------------

/// Finite-difference normal from the cached height field (unit length).
fn fd_normal(x: usize, z: usize, cell: f32) -> [f32; 3] {
    let f = field();
    let xl = f.height[idx(x.saturating_sub(1), z)];
    let xr = f.height[idx((x + 1).min(GRID - 1), z)];
    let zl = f.height[idx(x, z.saturating_sub(1))];
    let zr = f.height[idx(x, (z + 1).min(GRID - 1))];
    let dx = (xr - xl) / (2.0 * cell);
    let dz = (zr - zl) / (2.0 * cell);
    let n = Vec3::new(-dx, 1.0, -dz).normalize();
    [n.x, n.y, n.z]
}

// ---------------------------------------------------------------------------
// Noise primitives
// ---------------------------------------------------------------------------

/// Fractional Brownian motion in 0..1 (octave sum of value noise).
fn fbm(x: f32, z: f32, octaves: u32, lacunarity: f32, gain: f32, seed: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut norm = 0.0;
    let so = seed as f32 * 0.000_113;
    for _ in 0..octaves {
        sum += value_noise(x * freq + so, z * freq - so) * amp;
        norm += amp;
        freq *= lacunarity;
        amp *= gain;
    }
    (sum / norm).clamp(0.0, 1.0)
}

/// Ridged multifractal fBm in 0..1 (sharp ridges via 1-|n|, squared).
fn ridged_fbm(x: f32, z: f32, octaves: u32, seed: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut norm = 0.0;
    let so = seed as f32 * 0.000_131;
    for _ in 0..octaves {
        let n = value_noise(x * freq + so, z * freq + so) * 2.0 - 1.0;
        let r = 1.0 - n.abs();
        sum += r * r * amp;
        norm += amp;
        freq *= 2.0;
        amp *= 0.5;
    }
    (sum / norm).clamp(0.0, 1.0)
}

pub fn value_noise(x: f32, z: f32) -> f32 {
    let xi = x.floor();
    let zi = z.floor();
    let xf = x - xi;
    let zf = z - zi;
    let u = smooth(xf);
    let v = smooth(zf);
    let h00 = hash(xi, zi);
    let h10 = hash(xi + 1.0, zi);
    let h01 = hash(xi, zi + 1.0);
    let h11 = hash(xi + 1.0, zi + 1.0);
    let a = lerp(h00, h10, u);
    let b = lerp(h01, h11, u);
    lerp(a, b, v)
}

pub fn hash(x: f32, z: f32) -> f32 {
    ((x * 127.1 + z * 311.7).sin() * 43_758.547).fract().abs()
}

pub fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_surface_y_matches_height_inside_bounds() {
        let y = terrain_surface_y(64.0, 128.0);
        assert_eq!(y, terrain_height(64.0, 128.0));
    }

    #[test]
    fn terrain_surface_y_clamps_out_of_bounds() {
        let y = terrain_surface_y(-10.0, 999.0);
        assert_eq!(y, terrain_height(0.0, WORLD_SIZE));
    }

    #[test]
    fn height_in_range() {
        for z in 0..GRID {
            for x in 0..GRID {
                let h = field().height[idx(x, z)];
                assert!((0.0..=HEIGHT_SCALE).contains(&h), "h={h}");
            }
        }
    }

    #[test]
    fn not_flat_distant_points_differ() {
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for &h in &field().height {
            min = min.min(h);
            max = max.max(h);
        }
        // Real relief: peaks vs basins span a large fraction of the scale.
        assert!(max - min > 0.4 * HEIGHT_SCALE, "range={}", max - min);
    }

    #[test]
    fn land_ocean_ratio_reasonable() {
        let land = field().height.iter().filter(|&&h| h >= WATER_LEVEL).count();
        let ratio = land as f32 / field().height.len() as f32;
        assert!((0.4..=0.8).contains(&ratio), "land ratio={ratio}");
    }

    #[test]
    fn normals_unit_length() {
        let cell = WORLD_SIZE / (GRID - 1) as f32;
        for &(x, z) in &[(0, 0), (1, 1), (GRID / 2, GRID / 2), (GRID - 1, GRID - 1)] {
            let n = fd_normal(x, z, cell);
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-3, "len={len}");
        }
    }

    #[test]
    fn biome_variety_at_least_five() {
        let mut seen = [false; 12];
        for &b in &field().biome {
            seen[b as usize] = true;
        }
        let distinct = seen.iter().filter(|&&s| s).count();
        assert!(distinct >= 5, "only {distinct} distinct biomes");
    }

    #[cfg(feature = "pbr-textures")]
    #[test]
    fn pbr_biome_at_height_uses_height_norm_bands() {
        let beach = pbr_biome_at_height(0.20 * HEIGHT_SCALE);
        assert_eq!(beach, crate::materials::Biome::SandBeach);
        let snow = pbr_biome_at_height(0.95 * HEIGHT_SCALE);
        assert_eq!(snow, crate::materials::Biome::SnowPure);
    }

    #[test]
    fn deterministic_same_seed() {
        // Recompute the field from scratch and compare to the cached one.
        let again = generate_field();
        let cached = field();
        assert_eq!(again.height.len(), cached.height.len());
        for (a, b) in again.height.iter().zip(cached.height.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
        assert_eq!(again.biome, cached.biome);
    }
}
