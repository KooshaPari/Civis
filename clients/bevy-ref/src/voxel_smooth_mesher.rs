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
/// Blur half-width. 2 -> a 5x5x5 Gaussian neighbourhood (needs `APRON >= 2`).
const BLUR_RADIUS: isize = 2;
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

/// Build smooth per-material buffers.
pub fn build_smooth_meshes(
    voxels: &[MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE],
    padded_voxels: &[MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    saturation: Option<&[u8]>,
    registry: &MaterialRegistry,
) -> Vec<MeshBuffer> {
    SMOOTH_CHUNKS.fetch_add(1, Ordering::Relaxed);
    let mut materials = unique_materials(voxels);
    materials.sort_unstable_by_key(|id| id.0);
    materials
        .into_iter()
        .filter_map(|material_id| {
            let def = registry.get(material_id)?;
            let density = build_material_density(
                *voxels,
                *padded_voxels,
                saturation.map(<[u8]>::to_vec),
                material_id,
                *def,
            );
            Some(build_surface_nets(material_id, density))
        })
        .collect()
}

#[must_use]
pub fn mesher_chunk_stats() -> (u64, u64) {
    (
        SMOOTH_CHUNKS.load(Ordering::Relaxed),
        CUBIC_CHUNKS.load(Ordering::Relaxed),
    )
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
    _voxels: [MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE],
    padded_voxels: [MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    saturation: Option<Vec<u8>>,
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
    let default_sat = 0u8;
    move |x, y, z| {
        let (solid_occ, material_occ, sat) = sample_blurred_occupancy(
            &padded_voxels,
            saturation.as_deref(),
            default_sat,
            material,
            x,
            y,
            z,
        );
        if material_occ <= 0.0 {
            return 1.0;
        }
        let span = slope_aware_span(solid_occ);
        let density = (ISO_LEVEL - solid_occ) * span * sharpness;
        let saturation_soften = 1.0 - sat * 0.25;
        (density * saturation_soften).clamp(-1.0, 1.0)
    }
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

fn sample_blurred_occupancy(
    padded_voxels: &[MaterialId; CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED],
    saturation: Option<&[u8]>,
    default_sat: u8,
    material: MaterialId,
    x: usize,
    y: usize,
    z: usize,
) -> (f32, f32, f32) {
    let mut solid_occ = 0.0f32;
    let mut material_occ = 0.0f32;
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
                let sx = sx as usize;
                let sy = sy as usize;
                let sz = sz as usize;
                let idx = sx + sy * CHUNK_EDGE_PADDED + sz * CHUNK_EDGE_PADDED * CHUNK_EDGE_PADDED;
                if dx == 0 && dy == 0 && dz == 0 {
                    w += SOLID_CENTER_BIAS;
                }
                let is_solid = padded_voxels[idx] != AIR;
                let is_match = (padded_voxels[idx] == material) as u8;
                solid_occ += if is_solid { 1.0 } else { 0.0 } * w;
                material_occ += f32::from(is_match) * w;
                let sat = saturation
                    .and_then(|arr| arr.get(idx))
                    .copied()
                    .unwrap_or(default_sat);
                sat_acc += f32::from(sat) / 255.0 * w;
                weight_sum += w;
            }
        }
    }
    let inv = if weight_sum > 0.0 { 1.0 / weight_sum } else { 0.0 };
    (solid_occ * inv, material_occ * inv, sat_acc * inv)
}

#[inline]
fn weight_for_offset(dx: isize, dy: isize, dz: isize) -> f32 {
    // Gaussian falloff over the (radius-2) neighbourhood. A wide-but-decaying kernel
    // rounds the field for smoothness while still giving thin spans + chunk seams
    // enough neighbour support to stay solid; sigma ~1.1 keeps the 5x5x5 from
    // washing the surface out.
    let d2 = (dx * dx + dy * dy + dz * dz) as f32;
    (-d2 / (2.0 * 1.1 * 1.1)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_voxel::{material::MaterialRegistry, MaterialId};

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
        let mut chunk = [MaterialId(0); CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
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
        chunk[0] = MaterialId(1);
        let registry = MaterialRegistry::standard();
        let def = registry.get(MaterialId(1)).expect("material exists");
        let density = build_material_density(
            &chunk,
            &padded,
            None,
            MaterialId(1),
            *def,
        );
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
        let chunk = [MaterialId(0); CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
        let registry = MaterialRegistry::standard();
        let dirt = registry.get(MaterialId(1)).expect("material exists");
        let stone = registry.get(MaterialId(2)).expect("material exists");
        let dirt_density = build_material_density(&chunk, &padded, None, MaterialId(1), *dirt);
        let stone_density = build_material_density(&chunk, &padded, None, MaterialId(2), *stone);
        let interface = 8;
        let y = 8;
        let z = 8;
        let dirt_density_here = dirt_density(interface, y, z);
        let stone_density_here = stone_density(interface, y, z);
        assert!(dirt_density_here < 0.0);
        assert!(stone_density_here < 0.0);
    }
}
