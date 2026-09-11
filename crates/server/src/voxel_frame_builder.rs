//! Convert engine voxel state into `civ-protocol-3d` frames.
//!
//! Pure transformation — given a slice of `DirtyChunkEvent` (from
//! `Simulation::last_tick_voxel_events()`) and the current `VoxelWorld`, produce
//! a `VoxelDeltaFrame` whose deltas carry the dense leaf payload each event
//! refers to. Determinism is preserved: input events are already sorted by
//! `(chunk_id, write_seq)` by the kernel, and the builder walks them in order
//! without ever leaking HashMap iteration.

use civ_protocol_3d::{VoxelChunkDelta, VoxelDeltaFrame};
use civ_voxel::{
    to_chunk_coord, ChunkCoord, ChunkId, DirtyChunkEvent, MaterialId, VoxelWorld, WorldCoord,
    WriteSeq, FIXED_SCALE,
};
use std::collections::BTreeMap;

const CHUNK_EDGE: i32 = 16;

/// Errors a frame builder may return. Currently only one case — the dirty event
/// refers to a chunk that no longer exists in the world (which should not
/// happen during normal play, but the error is exposed so callers can decide
/// whether to skip or surface it).
#[derive(Debug, thiserror::Error)]
pub enum VoxelFrameBuilderError {
    /// The world has no chunk at the coordinate decoded from `chunk_id`.
    #[error("voxel frame builder: chunk {chunk_id:?} not present in world")]
    ChunkNotFound {
        /// The chunk ID that was looked up but absent.
        chunk_id: ChunkId,
    },
}

/// Build a `VoxelDeltaFrame` for one tick.
///
/// `tick` — server tick the events were drained at.
/// `events` — pre-sorted `(chunk_id, write_seq)` events from the kernel.
/// `world` — current voxel state (so each delta carries the *post-write* chunk
///           payload).
///
/// Deltas are deduplicated by `chunk_id`: multiple writes to the same chunk
/// within a tick produce a single delta carrying the latest payload. The
/// `event` recorded on the delta is the *last* (highest `write_seq`) event for
/// that chunk in the input slice, which preserves total-ordering replay.
pub fn build_voxel_delta_frame(
    tick: u64,
    events: &[DirtyChunkEvent],
    world: &VoxelWorld<MaterialId>,
) -> Result<VoxelDeltaFrame, VoxelFrameBuilderError> {
    if events.is_empty() {
        return Ok(VoxelDeltaFrame {
            tick,
            deltas: Vec::new(),
        });
    }

    // Group by chunk_id while keeping the highest write_seq event per chunk.
    // Input is already sorted, so we walk it once.
    let mut deltas: Vec<VoxelChunkDelta> = Vec::new();
    let coordinates = chunk_coordinates(world);
    let mut current: Option<DirtyChunkEvent> = None;
    for ev in events {
        match current {
            Some(prev) if prev.chunk_id == ev.chunk_id => {
                // Same chunk — keep the highest write_seq.
                current = Some(*ev);
            }
            Some(prev) => {
                // Chunk transition — flush the previous chunk's delta.
                deltas.push(build_chunk_delta(prev, world, &coordinates)?);
                current = Some(*ev);
            }
            None => {
                current = Some(*ev);
            }
        }
    }
    if let Some(last) = current {
        deltas.push(build_chunk_delta(last, world, &coordinates)?);
    }

    Ok(VoxelDeltaFrame { tick, deltas })
}

fn build_chunk_delta(
    event: DirtyChunkEvent,
    world: &VoxelWorld<MaterialId>,
    coordinates: &BTreeMap<ChunkId, ChunkCoord>,
) -> Result<VoxelChunkDelta, VoxelFrameBuilderError> {
    let missing = || VoxelFrameBuilderError::ChunkNotFound {
        chunk_id: event.chunk_id,
    };
    let coord = *coordinates.get(&event.chunk_id).ok_or_else(missing)?;
    let voxels = if let Some(chunk) = world.chunk(coord) {
        chunk.voxels.clone()
    } else if let Some(material) = world.octree().uniform_value(coord) {
        vec![material; 16 * 16 * 16]
    } else {
        return Err(missing());
    };
    Ok(VoxelChunkDelta { event, voxels })
}

fn chunk_coordinates(world: &VoxelWorld<MaterialId>) -> BTreeMap<ChunkId, ChunkCoord> {
    world
        .octree()
        .nodes
        .keys()
        .copied()
        .chain(world.chunks_dense().map(|(coord, _)| coord))
        .map(|coord| (coord.chunk_id(), coord))
        .collect()
}

/// Full authoritative terrain for an attaching client, including compacted
/// uniform chunks. Snapshot events use sequence zero as a baseline; subsequent
/// dirty frames retain the kernel's actual write sequence. No dirty state is drained.
pub fn build_voxel_snapshot_frame(
    tick: u64,
    world: &VoxelWorld<MaterialId>,
) -> Result<VoxelDeltaFrame, VoxelFrameBuilderError> {
    let coordinates = chunk_coordinates(world);
    let deltas = coordinates
        .keys()
        .map(|&chunk_id| {
            build_chunk_delta(
                DirtyChunkEvent {
                    chunk_id,
                    write_seq: WriteSeq(0),
                },
                world,
                &coordinates,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(VoxelDeltaFrame { tick, deltas })
}

/// Helper for callers that want to convert a world position into the (chunk_id, _)
/// pair expected by the kernel's dirty events. Mirrors the kernel's internal
/// `chunk_id_for` function so consumers do not need to reach into it.
///
/// (Intentionally re-derived here so the server crate doesn't take a private
/// dependency on the kernel's internals — when the kernel exposes
/// `ChunkCoord -> ChunkId` directly, switch to that.)
#[must_use]
pub fn world_coord_to_chunk_id(pos: WorldCoord) -> ChunkId {
    let c = to_chunk_coord(pos, FIXED_SCALE, CHUNK_EDGE);
    c.chunk_id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dirty_payload_contains_authoritative_materials_in_dense_index_order() {
        let mut world = VoxelWorld::new(FIXED_SCALE);
        let position = WorldCoord {
            x: -FIXED_SCALE,
            y: 2 * FIXED_SCALE,
            z: 4 * FIXED_SCALE,
        };
        world.write(position, MaterialId(7));
        let events = world.drain_dirty();
        let frame = build_voxel_delta_frame(4, &events, &world).expect("dense frame");
        assert_eq!(frame.deltas.len(), 1);
        assert_eq!(
            frame.deltas[0].event.chunk_id,
            world_coord_to_chunk_id(position)
        );
        assert_eq!(frame.deltas[0].voxels.len(), 4096);
        assert_eq!(frame.deltas[0].voxels[15 + 2 * 16 + 4 * 256], MaterialId(7));
        assert_eq!(frame.deltas[0].voxels[0], MaterialId(0));
    }

    #[test]
    fn snapshot_includes_compacted_air_and_solid_chunks_without_draining_events() {
        let mut world = VoxelWorld::new(FIXED_SCALE);
        for z in 0..16 {
            for y in 0..16 {
                for x in 0..16 {
                    world.write(
                        WorldCoord {
                            x: x * FIXED_SCALE,
                            y: y * FIXED_SCALE,
                            z: z * FIXED_SCALE,
                        },
                        MaterialId(3),
                    );
                }
            }
        }
        let erased = WorldCoord {
            x: 16 * FIXED_SCALE,
            y: 0,
            z: 0,
        };
        world.write(erased, MaterialId(9));
        world.write(erased, MaterialId(0));
        assert_eq!(world.compact(), 2);
        let frame = build_voxel_snapshot_frame(12, &world).expect("uniform snapshot");
        assert_eq!(frame.tick, 12);
        assert_eq!(frame.deltas.len(), 2);
        assert!(frame.deltas[0].voxels.iter().all(|&m| m == MaterialId(3)));
        assert!(frame.deltas[1].voxels.iter().all(|&m| m == MaterialId(0)));
        assert_eq!(frame.deltas[1].voxels.len(), 4096);
        let events = world.drain_dirty();
        assert!(!events.is_empty());
        let dirty = build_voxel_delta_frame(13, &events, &world).expect("compacted dirty frame");
        assert_eq!(dirty.deltas.len(), 2);
        assert!(dirty.deltas[1].voxels.iter().all(|&m| m == MaterialId(0)));
    }

    #[test]
    fn absent_dirty_chunk_is_an_error_instead_of_an_empty_payload() {
        let world = VoxelWorld::new(FIXED_SCALE);
        let result = build_voxel_delta_frame(
            1,
            &[DirtyChunkEvent {
                chunk_id: ChunkId(8),
                write_seq: WriteSeq(1),
            }],
            &world,
        );
        assert!(matches!(
            result,
            Err(VoxelFrameBuilderError::ChunkNotFound {
                chunk_id: ChunkId(8)
            })
        ));
    }

    /// FR-CIV-PROTO3D-010 — empty event slice produces an empty frame.
    #[test]
    fn empty_events_produce_empty_frame() {
        let world: VoxelWorld<MaterialId> = VoxelWorld::new(FIXED_SCALE);
        let frame = build_voxel_delta_frame(42, &[], &world).expect("frame");
        assert_eq!(frame.tick, 42);
        assert!(frame.deltas.is_empty());
    }

    /// FR-CIV-PROTO3D-011 — multiple writes to the same chunk collapse to a
    /// single delta carrying the highest-write_seq event.
    #[test]
    fn multiple_writes_same_chunk_collapse_to_one_delta() {
        let mut world: VoxelWorld<MaterialId> = VoxelWorld::new(FIXED_SCALE);
        world.write(
            WorldCoord {
                x: 0,
                y: 0,
                z: 7 * 16 * FIXED_SCALE,
            },
            MaterialId(1),
        );
        let events = vec![
            DirtyChunkEvent {
                chunk_id: ChunkId(7),
                write_seq: WriteSeq(1),
            },
            DirtyChunkEvent {
                chunk_id: ChunkId(7),
                write_seq: WriteSeq(2),
            },
            DirtyChunkEvent {
                chunk_id: ChunkId(7),
                write_seq: WriteSeq(3),
            },
        ];
        let frame = build_voxel_delta_frame(1, &events, &world).expect("frame");
        assert_eq!(frame.deltas.len(), 1);
        assert_eq!(frame.deltas[0].event.write_seq, WriteSeq(3));
    }

    /// FR-CIV-PROTO3D-012 — events across multiple chunks produce one delta per
    /// chunk in their input (sorted) order.
    #[test]
    fn events_across_chunks_produce_one_delta_each() {
        let mut world: VoxelWorld<MaterialId> = VoxelWorld::new(FIXED_SCALE);
        for z in 1..=3 {
            world.write(
                WorldCoord {
                    x: 0,
                    y: 0,
                    z: z * 16 * FIXED_SCALE,
                },
                MaterialId(z as u16),
            );
        }
        let events = vec![
            DirtyChunkEvent {
                chunk_id: ChunkId(1),
                write_seq: WriteSeq(5),
            },
            DirtyChunkEvent {
                chunk_id: ChunkId(2),
                write_seq: WriteSeq(1),
            },
            DirtyChunkEvent {
                chunk_id: ChunkId(2),
                write_seq: WriteSeq(10),
            },
            DirtyChunkEvent {
                chunk_id: ChunkId(3),
                write_seq: WriteSeq(2),
            },
        ];
        let frame = build_voxel_delta_frame(0, &events, &world).expect("frame");
        assert_eq!(frame.deltas.len(), 3);
        assert_eq!(frame.deltas[0].event.chunk_id, ChunkId(1));
        assert_eq!(frame.deltas[1].event.chunk_id, ChunkId(2));
        assert_eq!(frame.deltas[1].event.write_seq, WriteSeq(10));
        assert_eq!(frame.deltas[2].event.chunk_id, ChunkId(3));
    }

    #[test]
    fn world_coord_to_chunk_id_is_deterministic_and_separates_chunks() {
        let origin = WorldCoord { x: 0, y: 0, z: 0 };
        let a = world_coord_to_chunk_id(origin);
        assert_eq!(a, world_coord_to_chunk_id(origin));
        let far = WorldCoord {
            x: 100 * FIXED_SCALE * i64::from(CHUNK_EDGE),
            y: 0,
            z: 0,
        };
        assert_ne!(world_coord_to_chunk_id(far), a);
    }
}
