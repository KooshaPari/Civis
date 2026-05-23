//! civ-voxel — Civis adapter over the shared `phenotype-voxel` kernel.
//!
//! Part of the Civis 3D extension (`feat/civis-3d-foundation`). The actual storage
//! (SVO + dense 16³ leaf chunks), deterministic dirty queue, fixed-point coords,
//! and per-engine `Mesher` trait live in
//! [`phenotype-voxel`](https://github.com/KooshaPari/phenotype-voxel). This crate
//! re-exports the kernel and adds Civis-side glue (ECS integration with `civ-engine`,
//! protocol bindings via `civ-protocol-3d`) as it is implemented.
//!
//! See:
//! - `docs/roadmap/civis-3d-extension.md` (PRD addendum)
//! - `docs/adr/ADR-005-adaptive-voxel.md`
//!
//! Functional requirements: `FR-CIV-VOXEL-*` (see
//! `docs/development-guide/fr-3d-additions.md`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Re-export the Phenotype-org shared kernel verbatim. Civis-side adapters that follow
// (ECS integration, protocol bindings) live alongside this re-export.
pub use phenotype_voxel as kernel;
pub use phenotype_voxel::{
    select_lod, to_chunk_coord, Chunk, ChunkCoord, ChunkId, ChunkView, DirtyChunkEvent, LodLevel,
    LodPolicy, MaterialId, MaterialPalette, MeshBuffer, MeshError, MeshResult, MeshVertex, Mesher,
    OctreeNode, VoxelMaterial, VoxelOctree, VoxelScaleMultiplier, VoxelWorld, WorldCoord, WriteSeq,
    FIXED_SCALE,
};

/// Civis-side schema version. Independent of the kernel's `SCHEMA_VERSION` so we can
/// evolve the adapter without forcing kernel-version bumps.
pub const SCHEMA_VERSION: u32 = 0;

#[cfg(test)]
mod stub_tests {
    use super::*;

    /// FR-CIV-VOXEL-000 — crate compiles, kernel re-exports resolve.
    #[test]
    fn kernel_reexports_resolve() {
        let _: u32 = SCHEMA_VERSION;
        let _: u32 = phenotype_voxel::SCHEMA_VERSION;
        assert_eq!(SCHEMA_VERSION, 0);
    }

    /// FR-CIV-VOXEL-002 (early smoke) — kernel dirty events sort deterministically
    /// when used through the Civis re-export.
    #[test]
    fn dirty_events_sort_deterministically_through_reexport() {
        let mut evts = [
            DirtyChunkEvent {
                chunk_id: ChunkId(2),
                write_seq: WriteSeq(1),
            },
            DirtyChunkEvent {
                chunk_id: ChunkId(1),
                write_seq: WriteSeq(5),
            },
        ];
        evts.sort();
        assert_eq!(evts[0].chunk_id, ChunkId(1));
    }

    /// FR-CIV-VOXEL-005 (early smoke) — VoxelWorld replay is bit-identical when
    /// driven through the Civis re-export.
    #[test]
    fn voxel_world_replay_is_bit_identical_through_reexport() {
        let writes: [(WorldCoord, u8); 3] = [
            (
                WorldCoord {
                    x: 5_000_000,
                    y: 0,
                    z: 0,
                },
                1,
            ),
            (
                WorldCoord {
                    x: 0,
                    y: 5_000_000,
                    z: 0,
                },
                2,
            ),
            (
                WorldCoord {
                    x: 0,
                    y: 0,
                    z: 5_000_000,
                },
                3,
            ),
        ];
        let mut w1: VoxelWorld<u8> = VoxelWorld::new(1_000_000);
        let mut w2: VoxelWorld<u8> = VoxelWorld::new(1_000_000);
        for (pos, v) in writes {
            w1.write(pos, v);
            w2.write(pos, v);
        }
        assert_eq!(w1.drain_dirty(), w2.drain_dirty());
    }
}
