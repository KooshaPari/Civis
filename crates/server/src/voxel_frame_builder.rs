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
    to_chunk_coord, ChunkId, DirtyChunkEvent, MaterialId, VoxelWorld, WorldCoord, FIXED_SCALE,
};

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
    let mut current: Option<DirtyChunkEvent> = None;
    for ev in events {
        match current {
            Some(prev) if prev.chunk_id == ev.chunk_id => {
                // Same chunk — keep the highest write_seq.
                current = Some(*ev);
            }
            Some(prev) => {
                // Chunk transition — flush the previous chunk's delta.
                deltas.push(build_chunk_delta(prev, world)?);
                current = Some(*ev);
            }
            None => {
                current = Some(*ev);
            }
        }
    }
    if let Some(last) = current {
        deltas.push(build_chunk_delta(last, world)?);
    }

    Ok(VoxelDeltaFrame { tick, deltas })
}

fn build_chunk_delta(
    event: DirtyChunkEvent,
    _world: &VoxelWorld<MaterialId>,
) -> Result<VoxelChunkDelta, VoxelFrameBuilderError> {
    // We don't have a `VoxelWorld::chunk(coord) -> Option<&Chunk>` API yet —
    // the kernel exposes read(world_coord) and chunk_count(). Until the
    // chunk-access API lands in P-V1.2, build a zero-payload delta that still
    // carries the event so consumers know to re-fetch the chunk. The kernel
    // upgrade in a follow-up PR replaces this with the actual dense payload.
    Ok(VoxelChunkDelta {
        event,
        voxels: Vec::new(),
    })
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
    let cx = (c.cx as u32) as u64;
    let cy = (c.cy as u32) as u64;
    let cz = (c.cz as u32) as u64;
    ChunkId((cx << 40) | (cy << 16) | (cz & 0xFFFF))
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_voxel::WriteSeq;

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
        let world: VoxelWorld<MaterialId> = VoxelWorld::new(FIXED_SCALE);
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
        let world: VoxelWorld<MaterialId> = VoxelWorld::new(FIXED_SCALE);
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
}
