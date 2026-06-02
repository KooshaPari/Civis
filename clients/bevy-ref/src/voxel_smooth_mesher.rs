//! Terrain surface meshing for bevy-ref voxel chunks.
//!
//! The internal CA material grid stays untouched; this module only transforms a
//! dense 16³ chunk into a render mesh using a smooth extractor. The default path
//! is `Surface Nets` via the `surface-nets` crate (`v0.1.0`) with a per-material
//! density callback so material/physics signals can eventually drive softness.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

use civ_voxel::{
    material::{MaterialDef, Phase, AIR},
    MaterialId, MeshBuffer, MeshVertex,
};
use civ_voxel::material::MaterialRegistry;

const CHUNK_EDGE: usize = 16;
/// Apron ring (in voxels) carried around each chunk so the blur and Surface Nets
/// can see neighbour-chunk voxels and round across seams. A 2-voxel apron is what
/// a 5x5x5 (`BLUR_RADIUS = 2`) blur needs; with only 1 the outer blur ring falls
/// out of bounds and chunk faces step. Kept in sync with the apron built by
/// `voxel_sim::slice_chunk_with_apron` via `SMOOTH_MESH_PADDED_EDGE`.
const APRON: usize = 2;
const CHUNK_EDGE_PADDED: usize = CHUNK_EDGE + 2 * APRON;
pub const SMOOTH_MESH_PADDED_EDGE: usize = CHUNK_EDGE_PADDED;
/// Blur half-width. 1 -> a 3x3x3 Gaussian neighbourhood (27 samples). Dropped from
/// 2 (5x5x5, 125 samples) because the synchronous 144-chunk mesh at radius 2 cost
/// ~14s on load (perf log: mesh=13.93s) and blocked the autoshot capture window. The
/// relief SHAPE comes from worldgen, not blur radius, and geometry is already 99%
/// interpolated (smooth), so radius 1 keeps the molded look at ~5x less mesh cost.
/// The 2-voxel `APRON` stays, so chunk seams are still better-fed than the original
/// 1-voxel apron. (Reserve: raise back to 2 once meshing is async/threaded.)
const BLUR_RADIUS: isize = 1;
const SOLID_CENTER_BIAS: f32 = 0.08;
// Iso-surface tuning. The surface is the shared `solid_occ` contour at `ISO_LEVEL`.
// `ISO_LEVEL` near 0.5 keeps the surface gently rounded; a low value over-favors
// solid and re-hardens the field back to near-binary/cubic. `ISO_SPAN` is the
// gradient steepness across the boundary: a gentle span (~1.0) keeps the extracted
// surface SMOOTH — a steep span snaps Surface Nets vertices to the grid and reads
// cubic/stepped. Watertightness comes from the SHARED `solid_occ` field, not from a
// hard iso bias. The blur stays 3x3x3 to match the 1-voxel chunk apron.
const ISO_LEVEL: f32 = 0.47;
const ISO_SPAN: f32 = 1.0;
// Slope-aware softening: on steep transitions (near-vertical faces, where the local
// `solid_occ` is mid-range rather than saturated) the span eases toward
// `ISO_SPAN_STEEP` so those faces round instead of stepping. Saturated interior /
// pure-air keep the base `ISO_SPAN`.
const ISO_SPAN_STEEP: f32 = 0.8;

static SMOOTH_CHUNKS: AtomicU64 = AtomicU64::new(0);
static CUBIC_CHUNKS: AtomicU64 = AtomicU64::new(0);

use surface_nets::surface_net;

/// Output mesh path for terrain generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainMesherMode {
    /// Surface Nets / Surface Nets + density field.
    Smooth,
    /// Legacy cubic mesher for fallback/legacy chunks.
    Cubic,
}

/// Run-time selectable mesher mode (env override).
///
/// `CIVIS_VOXEL_MESHER`:
/// - `cubic` -> legacy cubic faces
/// - anything else / unset -> smooth surface extractor
#[must_use]
pub fn resolved_mesher_mode() -> TerrainMesherMode {
    match std::env::var("CIVIS_VOXEL_MESHER")
        .unwrap_or_else(|_| "smooth".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "cubic" => TerrainMesherMode::Cubic,
        "smooth" | "" => TerrainMesherMode::Smooth,
        _ => TerrainMesherMode::Smooth,
    }
}

/// Internal hook for density shaping. Returns signed distance value where positive
/// means empty/air and negative means solid.

/// Number of sample positions per axis the mesher evaluates density at. Surface
/// Nets memoizes density over `resolution + 1` corners per axis.
const SAMPLE_EDGE: usize = CHUNK_EDGE_PADDED + 1;

/// Per-chunk blurred fields that are MATERIAL-INDEPENDENT, so they are computed
/// ONCE per chunk and shared across every material mesh instead of being re-blurred
/// per material (the per-material redundancy was the ~33s synchronous-mesh stall:
/// the 5x5x5 solid_occ blur ran once per material × ~5 materials × 144 chunks).
struct ChunkBlurField {
    /// Blurred occupancy of ANY solid (non-AIR) at each sample corner, `[0,1]`.
    solid: Vec<f32>,
    /// Blurred saturation at each sample corner, `[0,1]`.
    sat: Vec<f32>,
}

/// Build smooth per-material buffers.
pub fn build_smooth_meshes(
    voxels: &[MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE],
    padded_voxels: &[MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    saturation: Option<&[u8]>,
    registry: &MaterialRegistry,
) -> Vec<MeshBuffer> {
    SMOOTH_CHUNKS.fetch_add(1, Ordering::Relaxed);
    // Shared, material-independent solid/saturation blur — blurred ONCE per chunk.
    let field = std::sync::Arc::new(build_chunk_blur_field(padded_voxels, saturation));
    let mut materials = unique_materials(voxels);
    materials.sort_unstable_by_key(|id| id.0);
    materials
        .into_iter()
        .filter_map(|material_id| {
            let def = registry.get(material_id)?;
            let density = build_material_density(
                *padded_voxels,
                std::sync::Arc::clone(&field),
                material_id,
                *def,
            );
            Some(build_surface_nets(material_id, density))
        })
        .collect()
}

/// Precompute the blurred solid + saturation fields over every sample corner once.
fn build_chunk_blur_field(
    padded_voxels: &[MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    saturation: Option<&[u8]>,
) -> ChunkBlurField {
    let mut solid = vec![0.0f32; SAMPLE_EDGE * SAMPLE_EDGE * SAMPLE_EDGE];
    let mut sat = vec![0.0f32; SAMPLE_EDGE * SAMPLE_EDGE * SAMPLE_EDGE];
    for z in 0..SAMPLE_EDGE {
        for y in 0..SAMPLE_EDGE {
            for x in 0..SAMPLE_EDGE {
                let (s, a) = sample_blurred_solid(padded_voxels, saturation, x, y, z);
                let idx = x + y * SAMPLE_EDGE + z * SAMPLE_EDGE * SAMPLE_EDGE;
                solid[idx] = s;
                sat[idx] = a;
            }
        }
    }
    ChunkBlurField { solid, sat }
}

#[must_use]
pub fn mesher_chunk_stats() -> (u64, u64) {
    (
        SMOOTH_CHUNKS.load(Ordering::Relaxed),
        CUBIC_CHUNKS.load(Ordering::Relaxed),
    )
}

/// Record that a chunk took the legacy cubic path (the smooth path self-records
/// in `build_smooth_meshes`). Lets `mesher_chunk_stats` report a true smooth-vs-cubic
/// split so a runtime diagnostic can tell whether terrain is actually meshing smooth.
pub fn record_cubic_chunk() {
    CUBIC_CHUNKS.fetch_add(1, Ordering::Relaxed);
}

fn unique_materials(voxels: &[MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE]) -> Vec<MaterialId> {
    let mut unique = HashSet::new();
    for &id in voxels.iter() {
        if id != AIR {
            unique.insert(id);
        }
    }
    unique.into_iter().collect()
}

fn build_surface_nets(
    material_id: MaterialId,
    density: impl Fn(usize, usize, usize) -> f32 + 'static,
) -> MeshBuffer {
    let (positions, normals, indices) = surface_net(CHUNK_EDGE_PADDED, &density, true);
    let mut vertices = Vec::with_capacity(positions.len());
    for (position, normal) in positions.iter().zip(normals.iter()) {
        let normal = normalize_or_unit_up(*normal);
        let position = [
            position[0] - APRON as f32,
            position[1] - APRON as f32,
            position[2] - APRON as f32,
        ];
        let uv = [
            (position[0].clamp(0.0, CHUNK_EDGE as f32 - 1.0)) / CHUNK_EDGE as f32,
            (position[2].clamp(0.0, CHUNK_EDGE as f32 - 1.0)) / CHUNK_EDGE as f32,
        ];
        vertices.push(MeshVertex {
            position,
            normal,
            uv,
            material: material_id,
        });
    }
    let indices = indices
        .into_iter()
        .filter_map(|i| u32::try_from(i).ok())
        .collect();
    MeshBuffer { vertices, indices }
}

fn build_material_density(
    padded_voxels: [MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    field: std::sync::Arc<ChunkBlurField>,
    material: MaterialId,
    def: MaterialDef,
) -> impl Fn(usize, usize, usize) -> f32 + 'static {
    let softness = surface_softness(&def);
    let phase_boost = match def.phase {
        Phase::Liquid | Phase::Powder => 0.35f32,
        Phase::Gas => 0.5,
        Phase::Solid => 1.0,
        Phase::Empty => 0.75,
    };
    let sharpness = (softness * phase_boost).clamp(0.25, 1.0);
    move |x, y, z| {
        // Ownership gate stays per-material but is now a CHEAP center+6-neighbour
        // presence check, not a full blur. The SURFACE shape uses the SHARED solid
        // field (material-independent) so adjacent materials meet watertight and the
        // expensive blur runs once per chunk, not once per material.
        if !material_present_near(&padded_voxels, material, x, y, z) {
            return 1.0;
        }
        let idx = x + y * SAMPLE_EDGE + z * SAMPLE_EDGE * SAMPLE_EDGE;
        let solid_occ = field.solid[idx];
        let sat = field.sat[idx];
        let span = slope_aware_span(solid_occ);
        let density = (ISO_LEVEL - solid_occ) * span * sharpness;
        let saturation_soften = 1.0 - sat * 0.25;
        (density * saturation_soften).clamp(-1.0, 1.0)
    }
}

/// Cheap per-material ownership test: is this material at, or directly adjacent to,
/// the sample corner? Replaces the old per-material full blur for the gate (the
/// surface shape comes from the shared solid field). Adjacency keeps a material's
/// mesh covering its share of the interface so neighbours meet without a seam gap.
#[inline]
fn material_present_near(
    padded_voxels: &[MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    material: MaterialId,
    x: usize,
    y: usize,
    z: usize,
) -> bool {
    const OFFSETS: [(isize, isize, isize); 7] = [
        (0, 0, 0),
        (-1, 0, 0),
        (1, 0, 0),
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
    ];
    for (dx, dy, dz) in OFFSETS {
        let sx = x as isize + dx;
        let sy = y as isize + dy;
        let sz = z as isize + dz;
        if sx < 0
            || sy < 0
            || sz < 0
            || sx >= CHUNK_EDGE_PADDED as isize
            || sy >= CHUNK_EDGE_PADDED as isize
            || sz >= CHUNK_EDGE_PADDED as isize
        {
            continue;
        }
        let idx = sx as usize
            + sy as usize * CHUNK_EDGE_PADDED
            + sz as usize * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED;
        if padded_voxels[idx] == material {
            return true;
        }
    }
    false
}

/// Span used to scale the signed distance, eased on steep transitions.
///
/// Cells whose blurred `solid_occ` sits near `ISO_LEVEL` are the surface-defining
/// (slope/boundary) cells — exactly where a steep span snaps Surface Nets vertices
/// to the grid and reads stepped. Near the iso contour the span eases toward
/// `ISO_SPAN_STEEP` so those faces round; well inside solid or air it keeps the
/// base `ISO_SPAN`. `t` is 1.0 at the contour and falls to 0.0 by half a band away.
#[inline]
fn slope_aware_span(solid_occ: f32) -> f32 {
    const BAND: f32 = 0.5;
    let t = (1.0 - (solid_occ - ISO_LEVEL).abs() / BAND).clamp(0.0, 1.0);
    ISO_SPAN + (ISO_SPAN_STEEP - ISO_SPAN) * t
}

fn surface_softness(def: &MaterialDef) -> f32 {
    match def.phase {
        Phase::Liquid => 0.55,
        Phase::Powder => 0.68,
        Phase::Gas => 0.62,
        Phase::Solid => 1.0,
        Phase::Empty => 0.8,
    }
}

fn normalize_or_unit_up(mut normal: [f32; 3]) -> [f32; 3] {
    let mag_sq = normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
    if mag_sq <= f32::EPSILON {
        return [0.0, 1.0, 0.0];
    }
    let inv = 1.0 / mag_sq.sqrt();
    normal[0] *= inv;
    normal[1] *= inv;
    normal[2] *= inv;
    normal
}

/// Blur the material-INDEPENDENT solid occupancy + saturation at one sample corner.
/// `solid_occ` = weighted fraction of any non-AIR neighbour (the shared surface
/// field); `sat` = weighted saturation. No per-material work — that is gated cheaply
/// in `material_present_near`.
fn sample_blurred_solid(
    padded_voxels: &[MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    saturation: Option<&[u8]>,
    x: usize,
    y: usize,
    z: usize,
) -> (f32, f32) {
    const DEFAULT_SAT: u8 = 0;
    let mut solid_occ = 0.0f32;
    let mut sat_acc = 0.0f32;
    let mut weight_sum = 0.0f32;
    for dz in -BLUR_RADIUS..=BLUR_RADIUS {
        for dy in -BLUR_RADIUS..=BLUR_RADIUS {
            for dx in -BLUR_RADIUS..=BLUR_RADIUS {
                let sx = x as isize + dx;
                let sy = y as isize + dy;
                let sz = z as isize + dz;
                let mut w = weight_for_offset(dx, dy, dz);
                let in_bounds = !(sx < 0
                    || sy < 0
                    || sz < 0
                    || sx >= CHUNK_EDGE_PADDED as isize
                    || sy >= CHUNK_EDGE_PADDED as isize
                    || sz >= CHUNK_EDGE_PADDED as isize);
                if !in_bounds {
                    // Out-of-world is air, but still keep full kernel sample
                    // weight for boundary consistency across chunk seams.
                    weight_sum += w;
                    continue;
                }
                let idx = sx as usize
                    + sy as usize * CHUNK_EDGE_PADDED
                    + sz as usize * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED;
                if dx == 0 && dy == 0 && dz == 0 {
                    w += SOLID_CENTER_BIAS;
                }
                if padded_voxels[idx] != AIR {
                    solid_occ += w;
                }
                let sat = saturation
                    .and_then(|arr| arr.get(idx))
                    .copied()
                    .unwrap_or(DEFAULT_SAT);
                sat_acc += f32::from(sat) / 255.0 * w;
                weight_sum += w;
            }
        }
    }
    let inv = if weight_sum > 0.0 { 1.0 / weight_sum } else { 0.0 };
    (solid_occ * inv, sat_acc * inv)
}

/// Gaussian weight for a kernel offset, by squared distance.
///
/// PERF: the blur runs ~125 samples per density evaluation and the mesher
/// evaluates density at ~9k corners per material per chunk (memoized), so a naive
/// `exp()` here is ~10^8+ transcendental calls on a 144-chunk world load (the
/// observed ~37s stall). Squared distance over a radius-2 kernel only takes the
/// values {0,1,2,3,4,5,6,8,9,12}, so we return precomputed `exp(-d²/(2·1.1²))`
/// constants via a match — same Gaussian, no runtime `exp`.
#[inline]
fn weight_for_offset(dx: isize, dy: isize, dz: isize) -> f32 {
    match dx * dx + dy * dy + dz * dz {
        0 => 1.0,
        1 => 0.661_514_7,
        2 => 0.437_601_6,
        3 => 0.289_479_9,
        4 => 0.191_495_2,
        5 => 0.126_676_9,
        6 => 0.083_798_6,
        8 => 0.036_670_4,
        9 => 0.024_258_0,
        _ => 0.007_022_2, // 12 (kernel corner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_voxel::{material::MaterialRegistry, MaterialId};

    /// Reference Gaussian the `weight_for_offset` match table mirrors.
    fn gaussian_reference(dx: isize, dy: isize, dz: isize) -> f32 {
        let d2 = (dx * dx + dy * dy + dz * dz) as f32;
        (-d2 / (2.0 * 1.1 * 1.1)).exp()
    }

    /// Build a single material's density closure over a padded chunk, mirroring the
    /// per-chunk shared-blur path `build_smooth_meshes` uses.
    fn density_for(
        padded: &[MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
        material: MaterialId,
    ) -> impl Fn(usize, usize, usize) -> f32 {
        let registry = MaterialRegistry::standard();
        let def = *registry.get(material).expect("material exists");
        let field = std::sync::Arc::new(build_chunk_blur_field(padded, None));
        build_material_density(*padded, field, material, def)
    }

    #[test]
    fn weight_table_matches_gaussian() {
        for dz in -BLUR_RADIUS..=BLUR_RADIUS {
            for dy in -BLUR_RADIUS..=BLUR_RADIUS {
                for dx in -BLUR_RADIUS..=BLUR_RADIUS {
                    let table = weight_for_offset(dx, dy, dz);
                    let exact = gaussian_reference(dx, dy, dz);
                    assert!(
                        (table - exact).abs() < 1e-4,
                        "weight mismatch at ({dx},{dy},{dz}): table={table} exact={exact}"
                    );
                }
            }
        }
    }

    #[test]
    fn normalize_or_unit_up_returns_default_for_zero() {
        assert_eq!(normalize_or_unit_up([0.0, 0.0, 0.0]), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn smooth_mesher_handles_single_cube() {
        let mut chunk = [MaterialId(0); CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
        let mut padded = [AIR; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED];
        padded[1 + 1 * CHUNK_EDGE_PADDED + 1 * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED] = MaterialId(1);
        chunk[0] = MaterialId(1);
        let registry = MaterialRegistry::standard();
        let bufs = build_smooth_meshes(&chunk, &padded, None, &registry);
        assert!(!bufs.is_empty());
        assert!(bufs.iter().all(|buf| !buf.vertices.is_empty()));
    }

    #[test]
    fn density_field_varies_across_half_filled_boundary() {
        let mut padded = [AIR; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED];
        for z in 0..CHUNK_EDGE_PADDED {
            for y in 0..CHUNK_EDGE_PADDED {
                for x in 0..CHUNK_EDGE_PADDED {
                    if x <= 8 {
                        padded[x + y * CHUNK_EDGE_PADDED + z * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED] =
                            MaterialId(1);
                    }
                }
            }
        }
        let density = density_for(&padded, MaterialId(1));
        let air = density(13, 10, 10);
        let solid = density(4, 10, 10);
        let boundary = density(8, 10, 10);
        assert!(air > 0.75);
        assert!(solid < -0.75);
        assert!(solid < -0.5);
        assert!(boundary < 0.0);
        assert!(boundary > solid);
        assert!(boundary < air);
    }

    #[test]
    fn interface_does_not_create_seam_holes() {
        let mut padded = [AIR; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED];
        for z in 0..CHUNK_EDGE_PADDED {
            for y in 0..CHUNK_EDGE_PADDED {
                for x in 0..CHUNK_EDGE_PADDED {
                    if x <= 8 {
                        padded[x + y * CHUNK_EDGE_PADDED + z * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED] =
                            MaterialId(1);
                    } else {
                        padded[x + y * CHUNK_EDGE_PADDED + z * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED] =
                            MaterialId(2);
                    }
                }
            }
        }
        let dirt_density = density_for(&padded, MaterialId(1));
        let stone_density = density_for(&padded, MaterialId(2));
        let interface = 8;
        let y = 8;
        let z = 8;
        let dirt_density_here = dirt_density(interface, y, z);
        let stone_density_here = stone_density(interface, y, z);
        assert!(dirt_density_here < 0.0);
        assert!(stone_density_here < 0.0);
    }
}
