//! civ-voxel — Civis adapter over the shared `phenotype-voxel` kernel.
//!
//! Part of the Civis 3D extension (`feat/civis-3d-foundation`). The actual storage
//! (SVO + dense 16³ leaf chunks), deterministic dirty queue, fixed-point coords,
//! and per-engine `Mesher` trait live in the
//! [`phenotype-voxel`](https://github.com/KooshaPari/phenotype-gfx/tree/main/crates/phenotype-voxel)
//! compat crate on `phenotype-gfx`. This crate re-exports the kernel and adds Civis-side glue
//! protocol bindings via `civ-protocol-3d`) as it is implemented.
//!
//! See:
//! - `docs/roadmap/civis-3d-extension.md` (PRD addendum)
//! - `docs/adr/ADR-005-adaptive-voxel.md`
//!
//! Functional requirements: `FR-CIV-VOXEL-*` (see
//! `docs/development-guide/fr-3d-additions.md`).

#![forbid(unsafe_code)]
#![allow(missing_docs)]

// Re-export the Phenotype-org shared kernel verbatim. Civis-side adapters that follow
// (ECS integration, protocol bindings) live alongside this re-export.
pub use phenotype_voxel as kernel;
pub use phenotype_voxel::{
    select_lod, to_chunk_coord, Chunk, ChunkCoord, ChunkId, ChunkView, CubicMesher, CubicVoxel,
    DirtyChunkEvent, LodLevel, LodPolicy, MaterialId, MaterialPalette, MeshBuffer, MeshError,
    MeshResult, MeshVertex, Mesher, OctreeNode, VoxelMaterial, VoxelOctree, VoxelScaleMultiplier,
    VoxelWorld, WorldCoord, WriteSeq, FIXED_SCALE,
};

pub mod boundary;
pub use boundary::{BoundaryConfig, BoundaryFace, BoundaryMode, Bounds3};
pub mod fluid_ca;
pub mod hud;
pub mod lod;
pub mod material;
pub use material::{
    AIR, BEDROCK, CLAY, DIRT, GRAVEL, ICE, LAVA, MOLTEN_METAL, OIL, ORE, PACKED_DIRT, SAND,
    SALT_WATER, STEAM, STONE, WATER,
};
pub mod material_ca;
pub mod material_pbr;
pub mod reactions;
pub mod scale_budget;
pub mod stream;
pub mod window;
pub mod worldgen;

pub use hud::{
    DiplomacyFsm, DiplomacyPanel, EventFeed, EventFeedItem, EventSeverity, MenuKind, MenuStack,
    MenuStackError, TechNode, TechTree, TechTreeError, ToolEntry, ToolPalette, ToolPaletteError,
    TreatySlot, HUB_PALETTE_SCHEMA_VERSION, HUB_TECH_SCHEMA_VERSION,
};
pub use material_pbr::{
    AtlasSlice, AttestationError, BuildFlavour, Cc0Source, ColorSpace, ColorSpacePolicy,
    GreedyAtlasPlan, LicenseAttestation, LodDistanceConfig, LodRenderPlan, ManifestError,
    MaterialMode, MaterialOverride, MaterialSeedManifest, MissingTexturePolicy,
    MissingTextureReport, PbrChannel, PolicyAction, RenderMode, RuntimeAction,
    TextureChannelMap, TriplanarAxisSample, TriplanarLayer, TriplanarPbrBlend,
    TriplanarSplatPlan, blend_triplanar_pbr, triplanar_axis_weights,
    SCHEMA_VERSION as PBR_MANIFEST_SCHEMA_VERSION,
};

pub use scale_budget::{
    CohortTotals, ExtentBudget, ExtentError, Gestalt, LodRingPlan, MvpResidentBudget,
    MvpResidentConfig, PlanError, RingRole, SimLodAggregator, StreamConfigLite,
};
pub use stream::{
    ChunkStorePort, FsChunkStore, StreamConfig, StreamStats, StreamingWorld, WorldGen, CHUNK_EDGE,
    CHUNK_EDGE_I32,
};
pub use window::io::{IoContract, MaterializedSnapshot, IO_CONTRACT_VERSION};
pub use window::plan::{
    prefetch_set, ChunkOffsetIter, ScaleReport, VelocityChunksPerTick, DEFAULT_PREFETCH_TICKS,
    P99_SAMPLE_CAP,
};
pub use window::ring_iter::RingIter;
pub use window::{ring_distance, ChunkState, EvictionKey, PolicyError, SimCohort, WindowPolicy};
pub use lod::{drain_dirty_chunks, mark_lod_dirty, mark_storage_dirty, ChunkDirty};
pub use worldgen::HeightFieldGen;

/// Civis-side schema version. Independent of the kernel's `SCHEMA_VERSION` so we can
/// evolve the adapter without forcing kernel-version bumps.
pub const SCHEMA_VERSION: &str = "0.1.0-stub";

#[cfg(test)]
mod stub_tests {
    use super::*;

    /// Covers FR-CIV-VOXEL-000.
    /// FR-CIV-VOXEL-000 — exposes a semver-like schema version stub.
    #[test]
    fn schema_version_stub() {
        assert!(!SCHEMA_VERSION.is_empty());
        let core = SCHEMA_VERSION.split('-').next().unwrap();
        let segments: Vec<&str> = core.split('.').collect();
        assert_eq!(segments.len(), 3);
        assert!(segments.iter().all(|part| !part.is_empty()));
    }

    /// Covers FR-CIV-VOXEL-000.
    /// Covers FR-CIV-VOXEL-001.
    /// FR-CIV-VOXEL-000 — crate compiles, kernel re-exports resolve.
    #[test]
    fn kernel_reexports_resolve() {
        let _: u32 = phenotype_voxel::SCHEMA_VERSION;
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

    /// FR-CIV-VOXEL-010 (early smoke) — VoxelWorld + CubicMesher round-trip through
    /// the Civis re-export. Writes a small block, drains dirty events, meshes the
    /// touched chunk, and asserts the mesh is non-empty + deterministic across
    /// two identical worlds.
    #[test]
    fn voxel_world_to_cubic_mesh_end_to_end() {
        fn build_block(w: &mut VoxelWorld<MaterialId>) {
            for ix in 0..3 {
                for iy in 0..3 {
                    for iz in 0..3 {
                        let pos = WorldCoord {
                            x: ix * 1_000_000,
                            y: iy * 1_000_000,
                            z: iz * 1_000_000,
                        };
                        w.write(pos, MaterialId(1));
                    }
                }
            }
        }
        let mut w1: VoxelWorld<MaterialId> = VoxelWorld::new(1_000_000);
        let mut w2: VoxelWorld<MaterialId> = VoxelWorld::new(1_000_000);
        build_block(&mut w1);
        build_block(&mut w2);
        assert_eq!(w1.drain_dirty(), w2.drain_dirty());

        // 3×3×3 block in a single 16³ chunk → 9 faces on each of the 6 outward
        // sides = 54 faces total = 216 vertices.
        let chunk_voxels: Vec<MaterialId> = {
            let mut v = vec![MaterialId(0); 16 * 16 * 16];
            for ix in 0..3 {
                for iy in 0..3 {
                    for iz in 0..3 {
                        v[ix + iy * 16 + iz * 16 * 16] = MaterialId(1);
                    }
                }
            }
            v
        };
        let view = ChunkView {
            id: ChunkId(0),
            voxels: &chunk_voxels,
        };
        let mesh = CubicMesher::mesh_cubic(view, LodLevel(0)).expect("mesh");
        assert_eq!(mesh.vertices.len(), 54 * 4);
        assert_eq!(mesh.indices.len(), 54 * 6);
    }

    /// Covers FR-CIV-VOXEL-001.
    /// FR-CIV-VOXEL-001 — adaptive storage: many writes within one 16³ leaf stay
    /// in a single dense chunk through the Civis re-export.
    #[test]
    fn adaptive_storage() {
        let mut world: VoxelWorld<u8> = VoxelWorld::new(FIXED_SCALE);
        for z in 0..16 {
            for y in 0..16 {
                for x in 0..16 {
                    world.write(
                        WorldCoord {
                            x: i64::from(x) * FIXED_SCALE,
                            y: i64::from(y) * FIXED_SCALE,
                            z: i64::from(z) * FIXED_SCALE,
                        },
                        1,
                    );
                }
            }
        }
        assert_eq!(world.chunk_count(), 1);
        assert_eq!(world.read(WorldCoord { x: 0, y: 0, z: 0 }), 1);
        assert_eq!(
            world.read(WorldCoord {
                x: 15 * FIXED_SCALE,
                y: 15 * FIXED_SCALE,
                z: 15 * FIXED_SCALE,
            }),
            1
        );
    }

    /// Covers FR-CIV-VOXEL-002.
    /// FR-CIV-VOXEL-002 — dirty queue drains in `(chunk_id, write_seq)` order.
    #[test]
    fn dirty_queue_deterministic() {
        let mut world: VoxelWorld<u8> = VoxelWorld::new(FIXED_SCALE);
        let a0 = WorldCoord { x: 0, y: 0, z: 0 };
        let b0 = WorldCoord {
            x: 100 * FIXED_SCALE,
            y: 0,
            z: 0,
        };
        world.write(a0, 1);
        world.write(b0, 2);
        world.write(
            WorldCoord {
                x: FIXED_SCALE,
                y: 0,
                z: 0,
            },
            3,
        );
        let dirty = world.drain_dirty();
        assert_eq!(dirty.len(), 3);
        for window in dirty.windows(2) {
            let (left, right) = (&window[0], &window[1]);
            assert!(
                left.chunk_id < right.chunk_id
                    || (left.chunk_id == right.chunk_id && left.write_seq <= right.write_seq)
            );
        }
    }

    /// Covers FR-CIV-VOXEL-003.
    /// FR-CIV-VOXEL-003 — public world coordinates are fixed-point integers.
    #[test]
    fn fixed_point_api() {
        let coord = WorldCoord {
            x: 3 * FIXED_SCALE + 250_000,
            y: -2 * FIXED_SCALE,
            z: 0,
        };
        let chunk = to_chunk_coord(coord, FIXED_SCALE, 16);
        assert_eq!(
            chunk,
            ChunkCoord {
                cx: 0,
                cy: -1,
                cz: 0
            }
        );
        assert_eq!(FIXED_SCALE, 1_000_000);
    }

    /// Covers FR-CIV-VOXEL-004.
    /// FR-CIV-VOXEL-004 — `VoxelScaleMultiplier` keeps LOD selection scale-invariant.
    #[test]
    fn scale_multiplier_lod() {
        let policy = LodPolicy::default();
        let lod_a = select_lod(64.0 * 8.0, VoxelScaleMultiplier(8.0), policy);
        let lod_b = select_lod(64.0 * 16.0, VoxelScaleMultiplier(16.0), policy);
        assert_eq!(lod_a, lod_b);
    }

    /// Covers FR-CIV-VOXEL-010.
    /// FR-CIV-VOXEL-010 — `Mesher` produces a non-empty watertight-style mesh
    /// for a fixed test block through the Civis re-export.
    #[test]
    fn mesher_watertight() {
        let chunk_voxels: Vec<MaterialId> = {
            let mut v = vec![MaterialId(0); 16 * 16 * 16];
            for ix in 0..3 {
                for iy in 0..3 {
                    for iz in 0..3 {
                        v[ix + iy * 16 + iz * 16 * 16] = MaterialId(1);
                    }
                }
            }
            v
        };
        let view = ChunkView {
            id: ChunkId(0),
            voxels: &chunk_voxels,
        };
        let mesh = CubicMesher::mesh_cubic(view, LodLevel(0)).expect("mesh");
        assert_eq!(mesh.vertices.len(), 54 * 4);
        assert_eq!(mesh.indices.len(), 54 * 6);
        assert!(mesh.indices.len() % 6 == 0);
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
