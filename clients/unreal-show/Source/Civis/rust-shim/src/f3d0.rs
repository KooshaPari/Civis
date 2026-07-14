//! Mirrors `CivF3d0ChunkMesh.cpp` math for offline compile-check (no UE required).

pub const CHUNK_EDGE: i32 = 16;
pub const CHUNK_VOXELS: usize = (CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE) as usize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub fn chunk_world_origin_from_id(chunk_raw: u64) -> Vec3 {
    let mut cx = ((chunk_raw >> 40) & 0xFFFFFF) as i64;
    let mut cy = ((chunk_raw >> 16) & 0xFFFFFF) as i64;
    let mut cz = (chunk_raw & 0xFFFF) as i64;
    if cx & 0x800000 != 0 {
        cx |= !0xFFFFFFi64;
    }
    if cy & 0x800000 != 0 {
        cy |= !0xFFFFFFi64;
    }
    if cz & 0x8000 != 0 {
        cz |= !0xFFFFi64;
    }
    let edge = CHUNK_EDGE as f32;
    Vec3 {
        x: cx as f32 * edge,
        y: cy as f32 * edge,
        z: cz as f32 * edge,
    }
}

fn voxel_index(x: i32, y: i32, z: i32) -> usize {
    (x + y * CHUNK_EDGE + z * CHUNK_EDGE * CHUNK_EDGE) as usize
}

fn is_solid(voxels: &[i32], x: i32, y: i32, z: i32) -> bool {
    voxels[voxel_index(x, y, z)] != 0
}

fn neighbor_solid(voxels: &[i32], x: i32, y: i32, z: i32) -> bool {
    if x < 0 || y < 0 || z < 0 || x >= CHUNK_EDGE || y >= CHUNK_EDGE || z >= CHUNK_EDGE {
        return false;
    }
    is_solid(voxels, x, y, z)
}

/// Returns triangle index count for exposed faces (6 indices per quad).
pub fn dense_chunk_mesh_triangle_count(material_ids: &[i32]) -> usize {
    if material_ids.len() != CHUNK_VOXELS {
        return 0;
    }
    let mut triangles = 0usize;
    for z in 0..CHUNK_EDGE {
        for y in 0..CHUNK_EDGE {
            for x in 0..CHUNK_EDGE {
                if !is_solid(material_ids, x, y, z) {
                    continue;
                }
                for (dx, dy, dz) in [
                    (-1, 0, 0),
                    (1, 0, 0),
                    (0, -1, 0),
                    (0, 1, 0),
                    (0, 0, -1),
                    (0, 0, 1),
                ] {
                    if !neighbor_solid(material_ids, x + dx, y + dy, z + dz) {
                        triangles += 6;
                    }
                }
            }
        }
    }
    triangles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_world_origin_matches_cpp_layout() {
        let chunk_id = (2u64 << 40) | (3u64 << 16) | 4u64;
        let origin = chunk_world_origin_from_id(chunk_id);
        assert_eq!(origin.x, 32.0);
        assert_eq!(origin.y, 48.0);
        assert_eq!(origin.z, 64.0);
    }

    #[test]
    fn single_voxel_produces_six_faces() {
        let mut voxels = vec![0i32; CHUNK_VOXELS];
        voxels[voxel_index(0, 0, 0)] = 1;
        assert_eq!(dense_chunk_mesh_triangle_count(&voxels), 36);
    }

    #[test]
    fn empty_chunk_has_no_triangles() {
        let voxels = vec![0i32; CHUNK_VOXELS];
        assert_eq!(dense_chunk_mesh_triangle_count(&voxels), 0);
    }
}
