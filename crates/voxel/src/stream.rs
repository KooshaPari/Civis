//! Chunk streaming layer over the `phenotype-voxel` kernel.
//!
//! Lifts the small-diorama voxel world toward a ~20mi×20mi real-world-equivalent
//! target by keeping a bounded *active working set* of dense leaf chunks in RAM and
//! paging the rest in/out:
//!
//! - **Generate-on-demand**: clean (never-edited) chunks are regenerated bit-identically
//!   from the world seed via a deterministic per-chunk [`WorldGen`]. They never need to
//!   touch disk — eviction just drops them.
//! - **Disk-backed cache**: chunks that have been *edited* (dirty) are serialised to a
//!   [`ChunkStorePort`] on eviction and reloaded on demand. Disk is the bound, not compute.
//! - **LRU active set**: [`StreamingWorld`] tracks an LRU of loaded chunks and evicts the
//!   coldest when over the RAM budget.
//! - **Distance-tiered LOD**: [`StreamingWorld::lod_for`] reuses the kernel
//!   [`select_lod`] so far chunks mesh coarser.
//!
//! ## Boundary policy for unloaded neighbours
//!
//! Meshing a chunk needs to know whether its 6 face-neighbours are solid (to cull
//! interior faces). When a neighbour is *not loaded*, we treat it as **empty (air)**
//! for face-culling purposes. This is the documented, deterministic boundary policy:
//! it can over-draw faces at active-set boundaries (a chunk at the edge of the loaded
//! radius renders its outward faces even if the unloaded neighbour would be solid),
//! but it never *hides* geometry and never depends on load order, so meshes remain
//! deterministic given the loaded set. Mass-conservation (CA) only runs over the loaded
//! hot set; unloaded chunks are frozen and conserve trivially.
//!
//! Determinism: every public operation is a pure function of (seed, requested set).
//! Regenerating a region from seed is bit-identical to reloading it — see the tests
//! and `region_regen_is_bit_identical_to_reload`.
//!
//! Functional requirements: `FR-CIV-VOXEL-020..029` (streaming/scale).

use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use phenotype_voxel::{
    select_lod, Chunk, ChunkCoord, LodLevel, LodPolicy, MaterialId, VoxelScaleMultiplier,
};

/// Edge length of a chunk in voxels (streaming math helper).
pub const CHUNK_EDGE: usize = 16;
/// Edge length of a chunk in voxels as `i32`.
pub const CHUNK_EDGE_I32: i32 = CHUNK_EDGE as i32;
const CHUNK_VOXELS: usize = CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE;

/// Deterministic, seeded per-chunk world generator.
///
/// Implementors MUST be a pure function of `(seed, coord)` so that a chunk
/// regenerated later is bit-identical to its first generation. The default
/// [`HeightFieldGen`] provides a value-noise heightfield suitable for the
/// scale benchmark.
pub trait WorldGen: Send + Sync {
    /// Generate the dense voxel payload for `coord`. Length MUST be `CHUNK_VOXELS`.
    fn generate(&self, coord: ChunkCoord) -> Chunk<MaterialId>;
}

/// Configuration for a [`StreamingWorld`].
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// World seed; threaded into every generated chunk.
    pub seed: u64,
    /// Maximum number of dense chunks kept resident in RAM (LRU budget).
    pub active_budget: usize,
    /// Edge length of one voxel in metres (base voxel size). Drives scale: a
    /// 20mi (~32 186 m) side at `base_voxel_m = 4.0` is ~8 046 voxels/side.
    pub base_voxel_m: f32,
    /// Per-engine LOD scale multiplier (kernel `VoxelScaleMultiplier`).
    pub lod_scale: VoxelScaleMultiplier,
    /// LOD policy (near/far thresholds).
    pub lod_policy: LodPolicy,
    /// Optional on-disk directory for the dirty-chunk cache. `None` = RAM-only
    /// (clean chunks still regenerate from seed; dirty chunks are kept resident).
    pub disk_dir: Option<PathBuf>,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            active_budget: 4096,
            base_voxel_m: 4.0,
            lod_scale: VoxelScaleMultiplier::default(),
            lod_policy: LodPolicy::default(),
            disk_dir: None,
        }
    }
}

/// Stats snapshot for the perf HUD.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StreamStats {
    /// Chunks currently resident in RAM.
    pub loaded: usize,
    /// Chunks served from regeneration since construction.
    pub regenerated: u64,
    /// Chunks loaded from the disk cache since construction.
    pub disk_loads: u64,
    /// Chunks evicted to disk since construction.
    pub disk_evictions: u64,
    /// Chunks evicted by simply dropping (clean, regenerable).
    pub dropped_evictions: u64,
}

/// Port trait for chunk storage adapters.
/// The domain (`StreamingWorld`) depends on this trait, not on concrete I/O.
pub trait ChunkStorePort: Send + Sync {
    /// Persist an edited chunk at the given coordinate.
    fn put(&self, coord: ChunkCoord, chunk: &Chunk<MaterialId>) -> std::io::Result<()>;
    /// Load an edited chunk if the store contains one.
    fn get(&self, coord: ChunkCoord) -> std::io::Result<Option<Chunk<MaterialId>>>;
    /// Return whether an edited chunk exists for the coordinate.
    fn contains(&self, coord: ChunkCoord) -> bool;
}

/// Filesystem-backed store for *edited* chunks. Clean chunks never reach here.
///
/// Serialisation is via the kernel's `serde` derives on [`Chunk`].
pub struct FsChunkStore {
    dir: PathBuf,
}

impl FsChunkStore {
    /// Open (creating if needed) a store rooted at `dir`.
    pub fn open(dir: PathBuf) -> std::io::Result<Self> {
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    /// Path for a chunk file.
    fn path_for(&self, coord: ChunkCoord) -> PathBuf {
        self.dir
            .join(format!("{}_{}_{}.chunk", coord.cx, coord.cy, coord.cz))
    }
}

impl ChunkStorePort for FsChunkStore {
    fn put(&self, coord: ChunkCoord, chunk: &Chunk<MaterialId>) -> std::io::Result<()> {
        let bytes =
            bincode::serialize(chunk).map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
        fs::write(self.path_for(coord), bytes)
    }
    fn get(&self, coord: ChunkCoord) -> std::io::Result<Option<Chunk<MaterialId>>> {
        let path = self.path_for(coord);
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err),
        };
        let chunk =
            bincode::deserialize(&bytes).map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
        Ok(Some(chunk))
    }
    fn contains(&self, coord: ChunkCoord) -> bool {
        self.path_for(coord).exists()
    }
}

/// A bounded, streaming voxel world: LRU active set + disk-backed dirty cache +
/// deterministic seeded regeneration.
pub struct StreamingWorld<G: WorldGen> {
    cfg: StreamConfig,
    gen: G,
    /// Resident chunks. `dirty` marks chunks that diverge from generation and
    /// therefore must be persisted (not dropped) on eviction.
    resident: HashMap<ChunkCoord, Resident>,
    /// LRU order: front = coldest, back = hottest. Holds the same keys as `resident`.
    lru: std::collections::VecDeque<ChunkCoord>,
    store: Option<Box<dyn ChunkStorePort>>,
    stats: StreamStats,
}

struct Resident {
    chunk: Chunk<MaterialId>,
    dirty: bool,
}

impl<G: WorldGen> StreamingWorld<G> {
    /// Construct a streaming world with an explicit store adapter.
    pub fn new_with_store(
        cfg: StreamConfig,
        gen: G,
        store: Option<Box<dyn ChunkStorePort>>,
    ) -> Self {
        Self {
            cfg,
            gen,
            resident: HashMap::new(),
            lru: VecDeque::new(),
            store,
            stats: StreamStats::default(),
        }
    }

    /// Construct a streaming world. Opens the disk cache if `cfg.disk_dir` is set.
    pub fn new(cfg: StreamConfig, gen: G) -> std::io::Result<Self> {
        let store = cfg
            .disk_dir
            .clone()
            .map(FsChunkStore::open)
            .transpose()?
            .map(|s| Box::new(s) as Box<dyn ChunkStorePort>);
        Ok(Self::new_with_store(cfg, gen, store))
    }

    /// Number of voxels along one side of the world for the configured scale and a
    /// given side length in metres (e.g. 20mi). Used for capacity/disk estimates.
    pub fn voxels_per_side(&self, side_metres: f32) -> u64 {
        (side_metres / self.cfg.base_voxel_m).ceil() as u64
    }

    /// Ensure `coord` is resident, loading from disk or regenerating from seed.
    /// Touches LRU. Triggers eviction if over budget.
    pub fn load(&mut self, coord: ChunkCoord) -> std::io::Result<()> {
        self.load_no_evict(coord)?;
        self.evict_to_budget()
    }

    /// Ensure every chunk in `coords` is resident (a streaming radius around the
    /// camera), then evict anything outside the budget. Returns the set actually
    /// resident after the call. Order-independent given the same `coords`.
    pub fn load_set(&mut self, coords: &[ChunkCoord]) -> std::io::Result<()> {
        let mut ordered = coords.to_vec();
        ordered.sort_unstable_by_key(|coord| (coord.cx, coord.cy, coord.cz));
        ordered.dedup();
        if self.cfg.active_budget < ordered.len() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "active_budget below requested set",
            ));
        }
        for coord in &ordered {
            self.load_no_evict(*coord)?;
        }
        for coord in ordered {
            self.touch(coord);
        }
        self.evict_to_budget()
    }

    /// Borrow a resident chunk, or `None` if not loaded.
    pub fn get(&self, coord: ChunkCoord) -> Option<&Chunk<MaterialId>> {
        self.resident.get(&coord).map(|resident| &resident.chunk)
    }

    /// Apply a voxel edit to a resident chunk, marking it dirty so it is persisted
    /// on eviction rather than dropped. `idx` is the dense `x + y*E + z*E*E` index.
    /// No-op returning `false` if the chunk is not resident.
    pub fn edit(&mut self, coord: ChunkCoord, idx: usize, value: MaterialId) -> bool {
        if idx >= CHUNK_VOXELS {
            return false;
        }
        let Some(resident) = self.resident.get_mut(&coord) else {
            return false;
        };
        resident.chunk.voxels[idx] = value;
        resident.dirty = true;
        self.touch(coord);
        true
    }

    /// LOD level for a chunk at `world_distance_m` from the camera.
    pub fn lod_for(&self, world_distance_m: f32) -> LodLevel {
        select_lod(world_distance_m, self.cfg.lod_scale, self.cfg.lod_policy)
    }

    /// Evict the coldest chunks until within `active_budget`. Dirty chunks are
    /// persisted to disk (if configured); clean chunks are dropped (regenerable).
    fn evict_to_budget(&mut self) -> std::io::Result<()> {
        while self.resident.len() > self.cfg.active_budget {
            let Some(coord) = self.lru.pop_front() else {
                break;
            };
            let Some(resident) = self.resident.get(&coord) else {
                continue;
            };
            if resident.dirty && self.store.is_none() {
                self.lru.push_back(coord);
                break;
            }
            let resident = self.resident.remove(&coord).expect("lru/resident mismatch");
            if resident.dirty {
                if let Some(store) = &self.store {
                    store.put(coord, &resident.chunk)?;
                    self.stats.disk_evictions += 1;
                }
            } else {
                self.stats.dropped_evictions += 1;
            }
        }
        Ok(())
    }

    /// Coords currently resident.
    pub fn resident_coords(&self) -> Vec<ChunkCoord> {
        self.resident.keys().copied().collect()
    }

    /// Current stats snapshot.
    pub fn stats(&self) -> StreamStats {
        StreamStats {
            loaded: self.resident.len(),
            ..self.stats
        }
    }

    fn load_no_evict(&mut self, coord: ChunkCoord) -> std::io::Result<()> {
        if self.resident.contains_key(&coord) {
            self.touch(coord);
            return Ok(());
        }
        let (chunk, dirty) = if let Some(store) = &self.store {
            if let Some(chunk) = store.get(coord)? {
                self.stats.disk_loads += 1;
                (chunk, true)
            } else {
                self.stats.regenerated += 1;
                (self.gen.generate(coord), false)
            }
        } else {
            self.stats.regenerated += 1;
            (self.gen.generate(coord), false)
        };
        self.resident.insert(coord, Resident { chunk, dirty });
        self.touch(coord);
        Ok(())
    }

    fn touch(&mut self, coord: ChunkCoord) {
        if let Some(pos) = self.lru.iter().position(|c| *c == coord) {
            self.lru.remove(pos);
        }
        self.lru.push_back(coord);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HeightFieldGen;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(label: &str) -> PathBuf {
        let uniq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("civis-voxel-{label}-{stamp}-{uniq}"))
    }

    fn coords(seed: u64, count: usize) -> Vec<ChunkCoord> {
        (0..count)
            .map(|i| ChunkCoord {
                cx: (seed as i32).wrapping_add(i as i32),
                cy: (i as i32) - 2,
                cz: (i as i32 * 3) - 1,
            })
            .collect()
    }

    fn world(cfg: StreamConfig) -> StreamingWorld<HeightFieldGen> {
        StreamingWorld::new(
            cfg,
            HeightFieldGen {
                seed: 7,
                base_voxel_m: 4.0,
                sea_level_m: 0.0,
            },
        )
        .expect("world")
    }


    /// FR-CIV-VOXEL-022 - edit returns false when the chunk is not resident.
    #[test]
    fn edit_returns_false_when_chunk_not_resident() {
        let mut w = world(StreamConfig::default());
        let coord = ChunkCoord { cx: 0, cy: 0, cz: 0 };
        // No load() called: chunk is not resident, edit must return false.
        let result = w.edit(coord, 0, MaterialId(1));
        assert!(!result, "edit on unloaded chunk should return false");
        assert!(w.get(coord).is_none(), "chunk should remain unloaded");
    }

    /// FR-CIV-VOXEL-022 - edit returns false when idx is out of bounds.
    #[test]
    fn edit_returns_false_for_out_of_bounds_idx() {
        let mut w = world(StreamConfig {
            active_budget: 4,
            ..StreamConfig::default()
        });
        let coord = ChunkCoord { cx: 0, cy: 0, cz: 0 };
        w.load(coord).expect("load");
        // CHUNK_VOXELS = 16^3 = 4096; idx >= 4096 is out of bounds.
        let result = w.edit(coord, CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE, MaterialId(99));
        assert!(!result, "edit with idx >= CHUNK_VOXELS should return false");
    }
    #[test]
    fn region_regen_is_bit_identical_to_reload() {
        let dir = temp_dir("regen");
        let cfg = StreamConfig {
            active_budget: 4,
            disk_dir: Some(dir.clone()),
            ..StreamConfig::default()
        };
        let mut a = world(cfg.clone());
        let mut b = world(cfg);
        let coords = coords(11, 4);
        a.load_set(&coords).expect("load a");
        b.load_set(&coords).expect("load b");
        for coord in &coords {
            assert_eq!(a.get(*coord).unwrap().voxels, b.get(*coord).unwrap().voxels);
        }
        let edited = coords[0];
        assert!(a.edit(edited, 0, MaterialId(1)));
        a.load(ChunkCoord {
            cx: 99,
            cy: 0,
            cz: 0,
        })
        .expect("force eviction");
        let fresh = HeightFieldGen {
            seed: 7,
            base_voxel_m: 4.0,
            sea_level_m: 0.0,
        };
        let mut c = world(StreamConfig {
            active_budget: 4,
            disk_dir: Some(dir),
            ..StreamConfig::default()
        });
        c.load_set(&coords).expect("reload all");
        for coord in &coords {
            let actual = c.get(*coord).unwrap();
            if *coord == edited {
                let mut expected_dirty = fresh.generate(*coord);
                expected_dirty.voxels[0] = MaterialId(1);
                assert_eq!(actual.voxels, expected_dirty.voxels);
            } else {
                assert_eq!(actual.voxels, fresh.generate(*coord).voxels);
            }
        }
    }

    #[test]
    fn lru_evicts_to_budget() {
        let coords = coords(3, 4);
        let mut w = world(StreamConfig {
            active_budget: 3,
            ..StreamConfig::default()
        });
        assert!(w.load_set(&coords).is_err());
        let mut w = world(StreamConfig {
            active_budget: 4,
            ..StreamConfig::default()
        });
        w.load_set(&coords).expect("budget");
        assert_eq!(w.resident_coords().len(), 4);
        let extra = ChunkCoord {
            cx: 99,
            cy: 99,
            cz: 99,
        };
        w.load(extra).expect("extra");
        assert_eq!(w.resident_coords().len(), 4);
    }

    #[test]
    fn dirty_persisted_clean_dropped() {
        let dir = temp_dir("stats");
        let mut w = world(StreamConfig {
            active_budget: 1,
            disk_dir: Some(dir),
            ..StreamConfig::default()
        });
        let a = ChunkCoord {
            cx: 0,
            cy: 0,
            cz: 0,
        };
        let b = ChunkCoord {
            cx: 1,
            cy: 0,
            cz: 0,
        };
        w.load(a).expect("a");
        assert!(w.edit(a, 0, MaterialId(1)));
        w.load(b).expect("b");
        let stats = w.stats();
        assert_eq!(stats.disk_evictions, 1);
        assert_eq!(stats.dropped_evictions, 0);
    }

    #[test]
    fn lod_for_monotonic() {
        let w = world(StreamConfig::default());
        let near = w.lod_for(10.0);
        let far = w.lod_for(10_000.0);
        assert!(near.0 <= far.0);
    }

    #[test]
    fn voxels_per_side_is_monotonic_ceil_division() {
        let cfg = StreamConfig::default();
        let w = world(cfg.clone());
        assert_eq!(w.voxels_per_side(0.0), 0);
        assert!(w.voxels_per_side(100.0) >= w.voxels_per_side(10.0));
        assert!(w.voxels_per_side(cfg.base_voxel_m) >= 1);
    }

    #[test]
    fn fresh_world_has_no_resident_chunks() {
        let w = world(StreamConfig::default());
        assert!(w.resident_coords().is_empty());
        assert_eq!(w.stats().loaded, 0);
    }

    #[test]
    fn load_set_accounts_for_duplicate_coords_once() {
        let a = ChunkCoord {
            cx: 1,
            cy: 2,
            cz: 3,
        };
        let b = ChunkCoord {
            cx: 4,
            cy: 5,
            cz: 6,
        };
        let mut w = world(StreamConfig {
            active_budget: 2,
            ..StreamConfig::default()
        });
        w.load_set(&[a, b, a]).expect("deduped working set fits");
        assert_eq!(w.resident_coords().len(), 2);
        assert_eq!(w.stats().loaded, 2);
    }

    /// Perf smoke: streaming a large radius (a full active-set page-in) must stay
    /// within a generous per-frame budget. This guards the streaming hot path
    /// against accidental O(n²) regressions; it is intentionally loose so it does
    /// not flake on shared CI, while still catching gross slowdowns.
    #[test]
    fn perf_smoke_large_radius_within_budget() {
        use std::time::Instant;
        // Radius 8 chunk cube ≈ 17³ = 4913 chunks resident — a realistic page-in.
        let r = 8i32;
        let mut set = Vec::new();
        for cx in -r..=r {
            for cy in -2..=2 {
                for cz in -r..=r {
                    set.push(ChunkCoord { cx, cy, cz });
                }
            }
        }
        let mut w = world(StreamConfig {
            active_budget: set.len(),
            ..StreamConfig::default()
        });
        let start = Instant::now();
        w.load_set(&set).expect("page-in");
        let elapsed = start.elapsed();
        assert_eq!(w.resident_coords().len(), set.len());
        // Cold page-in of ~5k generated chunks. Budget is deliberately loose
        // (2s) — a hot per-frame radius shift touches only the delta, not the
        // whole set. A regression that pushes this past 2s is a real problem.
        assert!(
            elapsed.as_secs_f32() < 2.0,
            "large-radius page-in took {elapsed:?}, over 2s budget"
        );
    }
}
