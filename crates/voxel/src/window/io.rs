//! Shared chunk IO contracts + save snapshot for the streaming window.
//!
//! Lifts the [`crate::stream::ChunkStore`] disk contract to a
//! versioned, serialisable manifest that downstream clients (Bevy,
//! Godot, Unreal) can negotiate against without depending on the
//! kernel's internal `bincode` shape. Three types, all `Copy`-able
//! where possible, all serialisable, all round-trippable through
//! `bincode` for replay:
//!
//! - [`IoContract`] — the on-disk manifest for a single edited chunk.
//!   Field-level stable so a save written by client A can be loaded
//!   by client B at a different kernel patch level. Implements
//!   FR-CIV-SCALE-006.
//! - [`MaterializedSnapshot`] — the save-format header for a
//!   materialised world region: a sorted list of resident chunk
//!   coords + their IO contracts + the policy that produced them.
//!   Implements FR-CIV-SCALE-007.
//!
//! Pure data — no `fs`, no `std::io`. The streaming layer wires the
//! bytes to disk via [`crate::stream::ChunkStore`]; these types are
//! the *contract*, not the transport.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::window::WindowPolicy;
use phenotype_voxel::ChunkCoord;

/// On-disk manifest version for a single edited chunk. Bumped when
/// the field shape changes in a way that requires a migration step
/// (e.g. adding a new field with a derived default).
///
/// The constant is duplicated in the snapshot header so a save
/// can be rejected if its contract version is outside the loader's
/// supported range, without having to read the full payload first.
pub const IO_CONTRACT_VERSION: u16 = 1;

/// Stable manifest for a single edited chunk on disk. The bytes
/// returned by [`MaterializedSnapshot::to_bincode`] are a sequence
/// of these (one per resident chunk), so a loader can stream them
/// in order rather than building a full in-memory index.
///
/// Field order is **load-bearing** for `bincode` — do not reorder
/// without bumping [`IO_CONTRACT_VERSION`]. The struct derives
/// `Eq` / `Hash` so it can index into a `HashMap` when the streaming
/// layer needs fast membership checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IoContract {
    /// On-disk manifest version (matches [`IO_CONTRACT_VERSION`] at
    /// the time of write).
    pub version: u16,
    /// Chunk coordinate this contract describes.
    pub coord: ChunkCoord,
    /// Voxel-edit count for this chunk. The save loader uses this
    /// to skip chunks that match the seed-derived regen (count 0 →
    /// re-derive from `WorldGen` instead of paying the disk-read
    /// cost). Matches the `WriteSeq` invariant on the kernel side.
    pub edit_count: u32,
    /// Last write sequence number the kernel assigned to this
    /// chunk. Saved alongside the chunk so a re-load that observes
    /// a *lower* `write_seq` than the in-memory regen can prefer the
    /// disk version. The streaming layer is the single source of
    /// truth for this counter.
    pub write_seq: u32,
    /// `true` if the chunk on disk is a *delta* over the seeded
    /// regen (e.g. only the dirty voxels are stored). `false` means
    /// the on-disk bytes are the full chunk payload. The streaming
    /// layer is free to switch this when storage is cheap; the
    /// manifest records whichever was chosen at write time.
    pub is_delta: bool,
}

impl IoContract {
    /// Construct a fresh `IoContract` for a resident chunk.
    ///
    /// `edit_count` is the number of *voxel writes* (not per-tick
    /// events) the chunk has absorbed; the streaming layer tracks
    /// this. `write_seq` is the kernel's monotonic per-chunk counter.
    /// `is_delta` is the writer's choice (see field docs).
    #[must_use]
    pub const fn new(coord: ChunkCoord, edit_count: u32, write_seq: u32, is_delta: bool) -> Self {
        Self {
            version: IO_CONTRACT_VERSION,
            coord,
            edit_count,
            write_seq,
            is_delta,
        }
    }

    /// True if this contract is byte-compatible with the current
    /// [`IO_CONTRACT_VERSION`]. Loaders SHOULD drop or migrate
    /// contracts that return `false` here.
    #[must_use]
    pub const fn is_current(&self) -> bool {
        self.version == IO_CONTRACT_VERSION
    }
}

/// Save-format header for a materialised region of the world.
///
/// A `MaterializedSnapshot` is the unit of persistence the save
/// format stores: a sorted, deduplicated list of
/// `coord → IoContract` for the region's resident set, plus the
/// [`WindowPolicy`] that produced the working set so a reload
/// reconstructs the same rings.
///
/// The snapshot is **pure data** (no chunk bytes — the chunk bytes
/// live in the [`crate::stream::ChunkStore`]). The save loader is
/// expected to read the snapshot header first, then open a
/// `ChunkStore` rooted at the snapshot's `disk_dir_name` to fetch
/// the actual voxel payloads in a second pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterializedSnapshot {
    /// Schema version of the save format. Bumped when the on-disk
    /// shape of [`MaterializedSnapshot`] changes in a way that
    /// requires a migration step.
    pub schema_version: u16,
    /// World seed at the time the snapshot was taken. The reload
    /// must use the same seed for clean-chunk regen to match.
    pub seed: u64,
    /// The [`WindowPolicy`] that was active at snapshot time. A
    /// reload uses this to reconstruct the working set: chunks
    /// inside the inner ring are loaded eagerly; chunks outside
    /// are fetched on demand.
    pub policy: WindowPolicy,
    /// Sorted, deduplicated list of resident-chunk manifests.
    /// Sorted by `(cx, cy, cz)` so a binary-search loader can
    /// resolve membership in O(log n).
    pub chunks: Vec<IoContract>,
    /// Logical name of the directory the chunk bytes were stored
    /// under (relative to the save root). The save loader resolves
    /// this against the save root to open a [`crate::stream::ChunkStore`].
    /// Kept short and machine-friendly (e.g. `"chunks"`).
    pub disk_dir_name: String,
}

impl MaterializedSnapshot {
    /// Schema version of the save format. Bumped on incompatible
    /// shape changes.
    pub const SCHEMA_VERSION: u16 = 1;

    /// Build a snapshot from a list of contracts, a seed, and a
    /// policy. The contracts are sorted and deduplicated in place
    /// (in `chunks`) so the result is canonical.
    ///
    /// The streaming layer typically calls this with the resident
    /// set's contracts, in iteration order; the canonicalisation
    /// step is cheap (O(n log n)) and the in-place form is what
    /// `bincode` serialises.
    #[must_use]
    pub fn from_parts(seed: u64, policy: WindowPolicy, mut chunks: Vec<IoContract>) -> Self {
        chunks.sort_unstable_by_key(|c| (c.coord.cx, c.coord.cy, c.coord.cz));
        chunks.dedup();
        Self {
            schema_version: Self::SCHEMA_VERSION,
            seed,
            policy,
            chunks,
            disk_dir_name: "chunks".to_string(),
        }
    }

    /// True if the snapshot's schema version is the one the loader
    /// expects. Loaders SHOULD drop or migrate snapshots that
    /// return `false`.
    #[must_use]
    pub const fn is_current(&self) -> bool {
        self.schema_version == Self::SCHEMA_VERSION
    }

    /// Number of resident chunks in the snapshot.
    #[must_use]
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// True if the snapshot has no resident chunks (an empty
    /// region; valid but uninteresting).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Bincode round-trip the snapshot. The streaming layer is
    /// free to use this directly; the round-trip is byte-exact for
    /// the same input (modulo platform endianness, which bincode
    /// pins to little-endian).
    pub fn to_bincode(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Bincode de-serialise a snapshot previously written with
    /// [`Self::to_bincode`]. Returns `Err` on shape mismatch (the
    /// streaming layer is expected to map this to a user-visible
    /// "save version too old / too new" error).
    pub fn from_bincode(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coord(cx: i32, cy: i32, cz: i32) -> ChunkCoord {
        ChunkCoord { cx, cy, cz }
    }

    // ---- IoContract ----

    /// FR-CIV-SCALE-006 — `IoContract` is `Copy`, hashes consistently,
    /// and the `is_current` check agrees with the constant.
    #[test]
    fn fr_civ_scale_006_io_contract_is_copy_hashable_and_versioned() {
        let a = IoContract::new(coord(1, 2, 3), 5, 7, false);
        let b = a; // Copy
        assert_eq!(a, b);
        // Hashable (compile-time check via HashMap).
        let mut map = std::collections::HashMap::new();
        map.insert(a, "alpha");
        assert_eq!(map.get(&b).copied(), Some("alpha"));
        // Versioned.
        assert_eq!(a.version, IO_CONTRACT_VERSION);
        assert!(a.is_current());
    }

    /// A contract with an older `version` is *not* current.
    #[test]
    fn io_contract_stale_version_is_not_current() {
        let c = IoContract {
            version: IO_CONTRACT_VERSION - 1,
            coord: coord(0, 0, 0),
            edit_count: 0,
            write_seq: 0,
            is_delta: false,
        };
        assert!(!c.is_current());
    }

    /// The `is_delta` flag is preserved across a `Copy`.
    #[test]
    fn io_contract_is_delta_flag_preserved() {
        let full = IoContract::new(coord(0, 0, 0), 0, 0, false);
        let delta = IoContract::new(coord(1, 0, 0), 3, 3, true);
        assert!(!full.is_delta);
        assert!(delta.is_delta);
    }

    // ---- MaterializedSnapshot ----

    /// FR-CIV-SCALE-007 — a snapshot's `chunks` field is sorted +
    /// deduplicated, and the schema version is set.
    #[test]
    fn fr_civ_scale_007_materialized_snapshot_canonicalises_chunks() {
        let policy = WindowPolicy::default();
        let chunks = vec![
            IoContract::new(coord(2, 0, 0), 1, 1, false),
            IoContract::new(coord(0, 0, 0), 1, 1, false),
            IoContract::new(coord(1, 0, 0), 1, 1, false),
            IoContract::new(coord(0, 0, 0), 1, 1, false), // dup
        ];
        let snap = MaterializedSnapshot::from_parts(7, policy, chunks);
        assert_eq!(snap.chunks.len(), 3, "duplicates are removed");
        let coords: Vec<ChunkCoord> = snap.chunks.iter().map(|c| c.coord).collect();
        assert_eq!(
            coords,
            vec![coord(0, 0, 0), coord(1, 0, 0), coord(2, 0, 0)],
            "chunks are sorted lexicographically"
        );
        assert_eq!(snap.schema_version, MaterializedSnapshot::SCHEMA_VERSION);
        assert!(snap.is_current());
    }

    /// Round-tripping a snapshot through bincode yields the same
    /// bytes, and `from_bincode` deserialises back to the original
    /// value. This is the save/load contract: a write followed by
    /// a read must be loss-less.
    #[test]
    fn materialized_snapshot_bincode_round_trip_is_loss_less() {
        let policy = WindowPolicy::default();
        let chunks = vec![
            IoContract::new(coord(-1, 0, 0), 2, 5, true),
            IoContract::new(coord(0, 0, 0), 0, 0, false),
            IoContract::new(coord(1, 2, -3), 1, 1, false),
        ];
        let snap = MaterializedSnapshot::from_parts(42, policy, chunks);
        let bytes = snap.to_bincode().expect("serialize");
        let back = MaterializedSnapshot::from_bincode(&bytes).expect("deserialize");
        assert_eq!(snap, back);
    }

    /// An empty snapshot is valid and reports `is_empty() == true`.
    #[test]
    fn materialized_snapshot_empty_is_valid() {
        let snap = MaterializedSnapshot::from_parts(0, WindowPolicy::default(), Vec::new());
        assert!(snap.is_empty());
        assert_eq!(snap.len(), 0);
        // Round-trips cleanly.
        let bytes = snap.to_bincode().expect("serialize");
        let back = MaterializedSnapshot::from_bincode(&bytes).expect("deserialize");
        assert_eq!(snap, back);
    }

    /// A snapshot with a stale `schema_version` is not current; the
    /// loader is expected to reject it.
    #[test]
    fn materialized_snapshot_stale_schema_is_not_current() {
        let mut snap = MaterializedSnapshot::from_parts(0, WindowPolicy::default(), Vec::new());
        snap.schema_version = MaterializedSnapshot::SCHEMA_VERSION - 1;
        assert!(!snap.is_current());
    }
}
