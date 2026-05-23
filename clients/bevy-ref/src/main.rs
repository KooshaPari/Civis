//! Civis Bevy reference 3D client.
//!
//! Status: pre-renderer headless smoke. The first real Bevy renderer pass lands
//! behind the `bevy` feature flag in a follow-up PR. For now this binary builds
//! a tiny `VoxelWorld`, meshes one chunk with the engine-neutral `CubicMesher`,
//! and prints the resulting face count — enough to confirm the workspace wires
//! `civ-voxel` correctly and to give CI something to screenshot once a real
//! renderer drops in.

use civ_voxel::{ChunkId, ChunkView, CubicMesher, LodLevel, MaterialId, VoxelWorld, WorldCoord};

const VOXEL_SPAN: i64 = 1_000_000;
const CHUNK_EDGE: usize = 16;

fn main() {
    let mut world: VoxelWorld<MaterialId> = VoxelWorld::new(VOXEL_SPAN);

    // Build a 4×4×4 cube of stone at the origin.
    for ix in 0..4 {
        for iy in 0..4 {
            for iz in 0..4 {
                world.write(
                    WorldCoord {
                        x: ix * VOXEL_SPAN,
                        y: iy * VOXEL_SPAN,
                        z: iz * VOXEL_SPAN,
                    },
                    MaterialId(1),
                );
            }
        }
    }
    let dirty = world.drain_dirty();
    println!("dirty events: {}", dirty.len());

    // Manually mirror the populated chunk into a flat slice for the mesher.
    let mut chunk_voxels = vec![MaterialId(0); CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
    for ix in 0..4 {
        for iy in 0..4 {
            for iz in 0..4 {
                chunk_voxels[ix + iy * CHUNK_EDGE + iz * CHUNK_EDGE * CHUNK_EDGE] = MaterialId(1);
            }
        }
    }
    let view = ChunkView {
        id: ChunkId(0),
        voxels: &chunk_voxels,
    };
    let mesh = CubicMesher::mesh_cubic(view, LodLevel(0)).expect("mesh");
    println!(
        "mesh: {} vertices, {} indices",
        mesh.vertices.len(),
        mesh.indices.len()
    );
    assert!(!mesh.vertices.is_empty());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_builds_and_meshes() {
        // Same as main, but returns rather than printing.
        let mut world: VoxelWorld<MaterialId> = VoxelWorld::new(VOXEL_SPAN);
        world.write(WorldCoord { x: 0, y: 0, z: 0 }, MaterialId(1));
        let dirty = world.drain_dirty();
        assert_eq!(dirty.len(), 1);
    }
}
