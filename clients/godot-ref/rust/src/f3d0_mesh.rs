//! Face-culled mesh for dense 16³ F3D0 `VoxelDelta` chunks.

use civ_protocol_3d::MaterialId;

pub const CHUNK_EDGE: usize = 16;
pub const CHUNK_VOXELS: usize = CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE;

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkMeshArrays {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub indices: Vec<i32>,
}

impl ChunkMeshArrays {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

fn voxel_index(x: usize, y: usize, z: usize) -> usize {
    x + y * CHUNK_EDGE + z * CHUNK_EDGE * CHUNK_EDGE
}

fn is_solid(voxels: &[MaterialId], x: usize, y: usize, z: usize) -> bool {
    voxels[voxel_index(x, y, z)].0 != 0
}

fn neighbor_solid(voxels: &[MaterialId], x: i32, y: i32, z: i32) -> bool {
    if !(0..CHUNK_EDGE as i32).contains(&x)
        || !(0..CHUNK_EDGE as i32).contains(&y)
        || !(0..CHUNK_EDGE as i32).contains(&z)
    {
        return false;
    }
    is_solid(voxels, x as usize, y as usize, z as usize)
}

fn push_quad(
    vertices: &mut Vec<f32>,
    normals: &mut Vec<f32>,
    indices: &mut Vec<i32>,
    base: [f32; 3],
    u: [f32; 3],
    v: [f32; 3],
    normal: [f32; 3],
) {
    let base_index = (vertices.len() / 3) as i32;
    let corners = [
        base,
        [base[0] + u[0], base[1] + u[1], base[2] + u[2]],
        [base[0] + u[0] + v[0], base[1] + u[1] + v[1], base[2] + u[2] + v[2]],
        [base[0] + v[0], base[1] + v[1], base[2] + v[2]],
    ];
    for corner in corners {
        vertices.extend_from_slice(&corner);
        normals.extend_from_slice(&normal);
    }
    indices.extend_from_slice(&[
        base_index,
        base_index + 1,
        base_index + 2,
        base_index,
        base_index + 2,
        base_index + 3,
    ]);
}

pub fn mesh_dense_chunk(voxels: &[MaterialId]) -> ChunkMeshArrays {
    let mut out = ChunkMeshArrays {
        vertices: Vec::new(),
        normals: Vec::new(),
        indices: Vec::new(),
    };
    if voxels.len() != CHUNK_VOXELS {
        return out;
    }
    let edge = CHUNK_EDGE as f32;
    for z in 0..CHUNK_EDGE {
        for y in 0..CHUNK_EDGE {
            for x in 0..CHUNK_EDGE {
                if !is_solid(voxels, x, y, z) {
                    continue;
                }
                let (px, py, pz) = (x as f32, y as f32, z as f32);
                if !neighbor_solid(voxels, x as i32 - 1, y as i32, z as i32) {
                    push_quad(
                        &mut out.vertices,
                        &mut out.normals,
                        &mut out.indices,
                        [px, py, pz],
                        [0.0, edge, 0.0],
                        [0.0, 0.0, edge],
                        [-1.0, 0.0, 0.0],
                    );
                }
                if !neighbor_solid(voxels, x as i32 + 1, y as i32, z as i32) {
                    push_quad(
                        &mut out.vertices,
                        &mut out.normals,
                        &mut out.indices,
                        [px + 1.0, py, pz],
                        [0.0, 0.0, edge],
                        [0.0, edge, 0.0],
                        [1.0, 0.0, 0.0],
                    );
                }
                if !neighbor_solid(voxels, x as i32, y as i32 - 1, z as i32) {
                    push_quad(
                        &mut out.vertices,
                        &mut out.normals,
                        &mut out.indices,
                        [px, py, pz],
                        [edge, 0.0, 0.0],
                        [0.0, 0.0, edge],
                        [0.0, -1.0, 0.0],
                    );
                }
                if !neighbor_solid(voxels, x as i32, y as i32 + 1, z as i32) {
                    push_quad(
                        &mut out.vertices,
                        &mut out.normals,
                        &mut out.indices,
                        [px, py + 1.0, pz],
                        [0.0, 0.0, edge],
                        [edge, 0.0, 0.0],
                        [0.0, 1.0, 0.0],
                    );
                }
                if !neighbor_solid(voxels, x as i32, y as i32, z as i32 - 1) {
                    push_quad(
                        &mut out.vertices,
                        &mut out.normals,
                        &mut out.indices,
                        [px, py, pz],
                        [edge, 0.0, 0.0],
                        [0.0, edge, 0.0],
                        [0.0, 0.0, -1.0],
                    );
                }
                if !neighbor_solid(voxels, x as i32, y as i32, z as i32 + 1) {
                    push_quad(
                        &mut out.vertices,
                        &mut out.normals,
                        &mut out.indices,
                        [px, py, pz + 1.0],
                        [0.0, edge, 0.0],
                        [edge, 0.0, 0.0],
                        [0.0, 0.0, 1.0],
                    );
                }
            }
        }
    }
    out
}

pub fn mesh_chunk_from_material_ids(raw: &[u32]) -> ChunkMeshArrays {
    if raw.len() != CHUNK_VOXELS {
        return ChunkMeshArrays {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        };
    }
    let voxels: Vec<MaterialId> = raw.iter().map(|id| MaterialId(*id as u16)).collect();
    mesh_dense_chunk(&voxels)
}
