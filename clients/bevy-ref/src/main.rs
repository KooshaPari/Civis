//! Civis Bevy reference 3D client.
//!
//! Headless by default — builds a tiny `VoxelWorld`, meshes one chunk with the
//! engine-neutral `CubicMesher`, and prints the resulting face count. Useful
//! for CI screenshot regressions and agent-driven smoke runs.
//!
//! Real interactive Bevy rendering lives in `civ-bevy-window` behind the
//! `bevy` feature:
//!
//! ```bash
//! cargo run -p civ-bevy-ref --features bevy --bin civ-bevy-window
//! ```
//!
//! The live WebSocket attach path only exists in that feature-gated binary,
//! so this default headless smoke stays valid without a running server.

use civ_voxel::{ChunkId, ChunkView, CubicMesher, LodLevel, MaterialId, VoxelWorld, WorldCoord};

const VOXEL_SPAN: i64 = 1_000_000;
const CHUNK_EDGE: usize = 16;

fn main() {
    let mut world: VoxelWorld<MaterialId> = VoxelWorld::new(VOXEL_SPAN);

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
