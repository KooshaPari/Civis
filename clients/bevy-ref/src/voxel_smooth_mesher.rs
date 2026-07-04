//! Terrain surface meshing for bevy-ref voxel chunks.
//!
//! The internal CA material grid stays untouched; this module only transforms a
//! dense 16³ chunk into a render mesh using a smooth extractor. The default path
//! is `Surface Nets` via the `surface-nets` crate (`v0.1.0`) with a per-material
//! density callback so material/physics signals can eventually drive softness.

use std::collections::HashSet;

use civ_voxel::{
    material::{MaterialDef, Phase, AIR},
    MaterialId, MeshBuffer, MeshVertex,
};
use civ_voxel::material::MaterialRegistry;

const CHUNK_EDGE: usize = 16;

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
    saturation: Option<&[u8]>,
    registry: &MaterialRegistry,
) -> Vec<MeshBuffer> {
    let mut materials = unique_materials(voxels);
    materials.sort_unstable_by_key(|id| id.0);
    materials
        .into_iter()
        .filter_map(|material_id| {
            let def = registry.get(material_id)?;
            let density = build_material_density(*voxels, saturation.map(<[u8]>::to_vec), material_id, *def);
            Some(build_surface_nets(material_id, density))
        })
        .collect()
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
    let (positions, normals, indices) = surface_net(CHUNK_EDGE, &density, true);
    let mut vertices = Vec::with_capacity(positions.len());
    for (position, normal) in positions.iter().zip(normals.iter()) {
        let normal = normalize_or_unit_up(*normal);
        vertices.push(MeshVertex {
            position: *position,
            normal,
            uv: [position[0] / CHUNK_EDGE as f32, position[2] / CHUNK_EDGE as f32],
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
    voxels: [MaterialId; CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE],
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
    let default_sat = 0u8;
    move |x, y, z| {
        let mut occupancy = 0.0f32;
        let mut sat_acc = 0.0f32;
        let mut sample_count = 0.0f32;
        let base_x = x.saturating_sub(1);
        let base_y = y.saturating_sub(1);
        let base_z = z.saturating_sub(1);
        let mut max_occ = 0.0f32;
        for dz in 0..2usize {
            for dy in 0..2usize {
                for dx in 0..2usize {
                    let sx = base_x + dx;
                    let sy = base_y + dy;
                    let sz = base_z + dz;
                    if sx >= CHUNK_EDGE || sy >= CHUNK_EDGE || sz >= CHUNK_EDGE {
                        continue;
                    }
                    let idx = sx + sy * CHUNK_EDGE + sz * CHUNK_EDGE * CHUNK_EDGE;
                    let match_target = f32::from((voxels[idx] == material) as u8);
                    occupancy += match_target;
                    let sat = saturation
                        .as_ref()
                        .and_then(|arr| arr.get(idx))
                        .copied()
                        .unwrap_or(default_sat);
                    sat_acc += f32::from(sat) / 255.0;
                    sample_count += 1.0;
                    max_occ = max_occ.max(match_target);
                }
            }
        }
        let occ = if sample_count > 0.0 { occupancy / sample_count } else { 0.0 };
        let sat = if sample_count > 0.0 { sat_acc / sample_count } else { 0.0 };
        let density = 1.0 - 2.0 * occ;
        let saturation_soften = 1.0 - sat * 0.25;
        let edge_softness = 1.0 - (max_occ * 0.2);
        let normalized = (density * softness * saturation_soften * phase_boost * edge_softness)
            .clamp(-1.0, 1.0);
        if max_occ == 0.0 { 1.0 } else if max_occ == 1.0 { normalized } else { normalized * 0.85 }
    }
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
        chunk[0] = MaterialId(1);
        let registry = MaterialRegistry::standard();
        let bufs = build_smooth_meshes(&chunk, None, &registry);
        assert!(!bufs.is_empty());
        assert!(bufs.iter().all(|buf| !buf.vertices.is_empty()));
    }
}
