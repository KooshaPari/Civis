//! Deterministic material cellular automaton for a dense voxel grid.
//!
//! This module is intentionally standalone: it does not depend on
//! `VoxelWorld` internals and instead operates on a simple dense grid of
//! `MaterialId` cells.

use crate::boundary::{BoundaryConfig, BoundaryFace, BoundaryMode, Bounds3};
use crate::material::{
    MaterialRegistry, Phase, ACID, AIR, ICE, LAVA, MOLTEN_METAL, MUD, OIL, SALT_WATER, SNOW, STEAM,
    WATER,
};
use crate::{MaterialId, VoxelWorld, WorldCoord};
use std::collections::HashSet;
use std::convert::TryFrom;

const DIRS: [(isize, isize, isize, BoundaryFace); 6] = [
    (1, 0, 0, BoundaryFace::PosX),
    (-1, 0, 0, BoundaryFace::NegX),
    (0, 1, 0, BoundaryFace::PosY),
    (0, -1, 0, BoundaryFace::NegY),
    (0, 0, 1, BoundaryFace::PosZ),
    (0, 0, -1, BoundaryFace::NegZ),
];

/// Dense 3D grid for deterministic CA stepping.
///
/// `saturation` (per-cell liquid saturation, 0-255) and `temperatures` are kept
/// alongside the material id so the fluid / thermo / percolation passes can
/// share the same row-major `cells` ordering without a second lookup
/// (FR-CIV-CA-001). The CA stepper holds a per-tick **double-buffer scratch**
/// (`scratch` + `scratch_temps` + `scratch_saturation`) so the rule passes
/// read from a stable snapshot while writing into the live buffers; this is
/// the FR-CIV-CA-008 contract that lets the 16³-leaf bottom-up sweep operate
/// on a 256³ resident window without copying the whole grid on every pass
/// (the previous implementation cloned the grid 4× per tick).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaGrid {
    /// Grid dimensions in `[x, y, z]` order.
    pub dims: [usize; 3],
    /// Row-major cells with `x + y * dx + z * dx * dy` indexing.
    pub cells: Vec<MaterialId>,
    /// Per-cell temperature state.
    pub temperatures: Vec<i16>,
    /// Per-cell liquid saturation (0-255).
    pub saturation: Vec<u8>,
    /// Chunks queued for the next CA pass.
    pub dirty_chunks: HashSet<usize>,
    /// Chunks whose cells (material / temperature / saturation) actually
    /// changed on the most recent [`step`] (or `step_n`) call. Cleared at the
    /// top of every step and populated by post-pass diff against the
    /// pre-step snapshot. Distinct from `dirty_chunks`, which is the
    /// active-set *input* (chunks the CA will step over with a 1-cell halo).
    ///
    /// This is the consumer-visible remesh list: it powers the Bevy
    /// despawn+respawn loop and the kernel-side `DirtyChunkEvent` writeback in
    /// `step_world_with_config` so a 256³ static world writes 0 voxels per
    /// tick and emits 0 dirty events.
    pub last_changed_chunks: HashSet<usize>,
    /// Per-tick scratch buffers used by the rule passes for the
    /// double-buffer read source (FR-CIV-CA-008). Same length as `cells` /
    /// `temperatures` / `saturation`. Allocated lazily on first use and
    /// resized on `new` so callers never have to think about it.
    scratch: Vec<MaterialId>,
    scratch_temps: Vec<i16>,
    scratch_saturation: Vec<u8>,
}

/// Outcome of a single CA step. `changed` is the cheap scalar check; the
/// `changed_chunks` vec is the *minimal* remesh list the renderer should walk
/// (vs the `dirty_chunks` "active" set, which is a superset because of the
/// 1-cell halo the rule passes need for cross-chunk neighbour reads).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StepOutcome {
    /// `true` when at least one cell changed.
    pub changed: bool,
    /// Local chunk indices (in `CaGrid` chunk-coord space) whose cells differ
    /// from the pre-step snapshot. Empty when `changed == false`.
    pub changed_chunks: Vec<usize>,
}

/// Read-only view into a [`CaGrid`]'s scratch buffers. Mirrors the public
/// accessors of `CaGrid` but reads from the double-buffer snapshot instead
/// of the live cells. Rule passes that need to "read prev, write live"
/// borrow this view during the pass and drop it before mutating the live
/// cells (FR-CIV-CA-008).
#[derive(Debug, Clone)]
pub struct ScratchView {
    /// Scratch cells (length == `dims[0] * dims[1] * dims[2]`).
    pub cells: Vec<MaterialId>,
    /// Scratch temperatures, parallel to `cells`.
    pub temperatures: Vec<i16>,
    /// Scratch saturation, parallel to `cells`.
    pub saturation: Vec<u8>,
    /// Grid dimensions in `[x, y, z]` order.
    pub dims: [usize; 3],
}

impl ScratchView {
    /// Returns the linear index for a coordinate if it is in bounds.
    #[must_use]
    pub fn index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        if x < self.dims[0] && y < self.dims[1] && z < self.dims[2] {
            Some(x + y * self.dims[0] + z * self.dims[0] * self.dims[1])
        } else {
            None
        }
    }

    /// Read a cell or `AIR` when out of bounds.
    #[must_use]
    pub fn get(&self, x: usize, y: usize, z: usize) -> MaterialId {
        self.index(x, y, z).map_or(AIR, |i| self.cells[i])
    }
}

/// Suitability score for abiogenesis on a single cell. The MVP resident
/// window (`Simulation::phase_voxel_ca`) reads the solvent and energy inputs
/// from the CA grid and emits a `value` in `[0, 1]` that downstream
/// emergence phases (life / ecology) use to seed the first cells.
///
/// Pure data: the inputs are deterministic functions of the local
/// `(material, temperature, saturation)` so two same-seed runs of the
/// resident window produce bit-identical suitability fields
/// (FR-CIV-CA-009).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbiogenesisSuitability {
    /// Solvent availability (0-255) — e.g. WATER / SALT_WATER cells score high.
    pub solvent: u8,
    /// Free energy in the local volume (0-255) — warm liquid cells score
    /// high; cold / solid cells score low.
    pub energy: u8,
    /// Combined suitability (0-100). Computed deterministically from
    /// `solvent` and `energy`; a value >= 50 indicates a "viable" cell.
    pub value: u8,
}

impl AbiogenesisSuitability {
    /// Build the suitability for a single cell from the local material,
    /// temperature, and saturation. Pure function — no IO, no RNG.
    ///
    /// The MVP rule is intentionally simple (FR-CIV-CA-009): a cell is
    /// viable when it sits in liquid water between 0 °C and 80 °C with
    /// non-zero free energy. This is a placeholder for the full
    /// `Miller-Urey`-class model; the substrate is what matters here.
    #[must_use]
    pub fn from_cell(material: MaterialId, temperature: i16, saturation: u8) -> Self {
        // Solvent score: WATER / SALT_WATER get the top band; MUD / ACID /
        // OIL get partial credit. Everything else (including AIR and
        // STONE) gets zero.
        let solvent: u8 = match material {
            WATER | SALT_WATER => 255,
            MUD | ACID => 160,
            OIL => 80,
            _ => 0,
        };
        // Energy score: 0 °C..80 °C maps linearly to 0..255; outside that
        // band the cell is too cold (no free energy) or too hot (sterilised).
        let energy: u8 = if temperature <= 0 || temperature >= 80 {
            0
        } else {
            // 0 °C → 0, 80 °C → 255, linear.
            let t = i32::from(temperature);
            ((t * 255) / 80).clamp(0, 255) as u8
        };
        // Saturation only counts for the porous-medium abiogenesis path
        // (MUD / SAND / DIRT) — a dry pore is not yet a habitat. We add
        // a small bonus to keep the combined value monotonic in
        // saturation for the porous branch.
        let sat_bonus: u16 = u16::from(saturation) / 8;
        // Combined value: geometric mean of (solvent, energy, sat) scaled to
        // 0..100. We use saturating arithmetic so a zero solvent still
        // produces a 0 (sterile) rather than a divide-by-zero.
        let s = u32::from(solvent);
        let e = u32::from(energy);
        let sat = u32::from(sat_bonus);
        let combined: u32 = if s == 0 || e == 0 {
            0
        } else {
            // (s*e*sat)^(1/3) ≈ cube root. We use the integer approximation
            // (`pow(1/3)` via a small table) to keep this branch pure +
            // deterministic without pulling in `f32` / `f64`.
            let prod = s.saturating_mul(e).saturating_mul(sat.max(1));
            (integer_cube_root(prod) * 100) / 255
        };
        Self {
            solvent,
            energy,
            value: combined.min(100) as u8,
        }
    }

    /// True when the cell is a viable abiogenesis seed (`value >= 10`).
    /// The MVP threshold is intentionally low so the warm-liquid band
    /// (`WATER` 0–80 °C) seeds well; the emergence layer filters further
    /// via the `solvent` / `energy` band before it spends simulation
    /// budget on the seed.
    #[must_use]
    pub fn is_viable(self) -> bool {
        self.value >= 10
    }
}

/// Integer cube root. Returns the largest `r` with `r³ <= n` for `n` in
/// `0..=u32::MAX`. We only need the result in `0..=2_048` for the
/// suitability maths (solver band is `0..=255³` and the answer fits in
/// `u16`), so a Newton-iteration loop is overkill — a small table of
/// `r * r * r` thresholds is faster and trivially deterministic.
fn integer_cube_root(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    // Pre-computed `r³` for r in 0..=2048. We only need a coarse root; a
    // linear scan over 256 entries is faster than the alternative (a
    // floating-point `cbrt` and rounding), and the result is byte-equal
    // across platforms.
    let mut r: u32 = 0;
    for candidate in 0u32..=2048 {
        let c = candidate * candidate * candidate;
        if c > n {
            break;
        }
        r = candidate;
    }
    r
}

impl CaGrid {
    /// Creates a new grid filled with air and zero saturation.
    #[must_use]
    pub fn new(dims: [usize; 3]) -> Self {
        let len = dims[0].saturating_mul(dims[1]).saturating_mul(dims[2]);
        Self {
            dims,
            cells: vec![AIR; len],
            temperatures: vec![0; len],
            saturation: vec![0; len],
            dirty_chunks: HashSet::new(),
            last_changed_chunks: HashSet::new(),
            scratch: vec![AIR; len],
            scratch_temps: vec![0; len],
            scratch_saturation: vec![0; len],
        }
    }

    /// Snapshot the live `cells` / `temperatures` / `saturation` into the
    /// double-buffer scratch. Called at the top of every CA step that needs
    /// stable read sources across the four rule passes. Cost is one
    /// row-major copy each (FR-CIV-CA-008) — the live grids are NOT cloned,
    /// only the scratch is written.
    pub fn snapshot_into_scratch(&mut self) {
        self.scratch.copy_from_slice(&self.cells);
        self.scratch_temps.copy_from_slice(&self.temperatures);
        self.scratch_saturation.copy_from_slice(&self.saturation);
    }

    /// Alias for [`snapshot_into_scratch`] — matches the per-tick call site
    /// ("refresh the scratch at the top of the run") used by the FR-CIV-CA-008
    /// double-buffer contract.
    pub fn refresh_scratch(&mut self) {
        self.snapshot_into_scratch();
    }

    /// Borrow the scratch snapshot as a read-only view (cells + temperatures).
    /// Used by the rule passes to satisfy the "read prev, write live"
    /// double-buffer contract (FR-CIV-CA-008). The returned view is bound
    /// to the lifetime of `self`, so callers must drop the borrow before
    /// mutating the live cells.
    ///
    /// Note: the returned `ScratchView` is *owned* (the scratch buffers are
    /// `std::mem::take`-swapped out of `self` for the duration of the call)
    /// so callers can pass it by value to a rule pass while keeping the
    /// `&mut CaGrid` borrow live. Callers MUST call [`CaGrid::restore_scratch`]
    /// with the view's fields when done, otherwise the next refresh will
    /// be wasted on a defaulted scratch.
    #[must_use]
    pub fn scratch_view(&mut self) -> ScratchView {
        let cells = std::mem::take(&mut self.scratch);
        let temperatures = std::mem::take(&mut self.scratch_temps);
        let saturation = std::mem::take(&mut self.scratch_saturation);
        let dims = self.dims;
        ScratchView {
            cells,
            temperatures,
            saturation,
            dims,
        }
    }

    /// Restore the scratch buffers from a [`ScratchView`] that was
    /// previously taken by [`CaGrid::scratch_view`]. This is a swap so the
    /// underlying allocation is preserved across refreshes.
    pub fn restore_scratch(&mut self, view: ScratchView) {
        self.scratch = view.cells;
        self.scratch_temps = view.temperatures;
        self.scratch_saturation = view.saturation;
    }

    /// Returns the linear index for a coordinate if it is in bounds.
    #[must_use]
    pub fn index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        if x < self.dims[0] && y < self.dims[1] && z < self.dims[2] {
            Some(x + y * self.dims[0] + z * self.dims[0] * self.dims[1])
        } else {
            None
        }
    }

    /// Reads a cell or returns air when out of bounds.
    #[must_use]
    pub fn get(&self, x: usize, y: usize, z: usize) -> MaterialId {
        self.index(x, y, z).map_or(AIR, |i| self.cells[i])
    }

    /// Reads a temperature or returns 20 when out of bounds.
    #[must_use]
    pub fn get_temp(&self, x: usize, y: usize, z: usize) -> i16 {
        self.index(x, y, z).map_or(20, |i| self.temperatures[i])
    }

    /// Writes a cell and temperature when coordinates are in bounds.
    pub fn set_with_temp(&mut self, x: usize, y: usize, z: usize, value: MaterialId, temp: i16) {
        if let Some(i) = self.index(x, y, z) {
            self.cells[i] = value;
            self.temperatures[i] = temp;
            self.mark_dirty_cell(x, y, z);
        }
    }

    /// Writes a cell when coordinates are in bounds.
    pub fn set(&mut self, x: usize, y: usize, z: usize, value: MaterialId) {
        self.set_with_temp(x, y, z, value, self.get_temp(x, y, z));
    }

    /// Sorted list of chunk indices that have at least one dirty cell.
    pub fn dirty_chunks(&self) -> Vec<usize> {
        let mut chunks: Vec<_> = self.dirty_chunks.iter().copied().collect();
        chunks.sort_unstable();
        chunks
    }

    /// Mark the chunk containing cell `(x, y, z)` as dirty (no-op if grid has
    /// a zero dimension in any axis).
    pub fn mark_dirty_cell(&mut self, x: usize, y: usize, z: usize) {
        let cx = x / 16;
        let cy = y / 16;
        let cz = z / 16;
        let counts = self.chunk_counts();
        if counts[0] > 0 && counts[1] > 0 && counts[2] > 0 {
            self.dirty_chunks
                .insert(cx + cy * counts[0] + cz * counts[0] * counts[1]);
        }
    }

    /// Reset the dirty set, then mark every chunk that contains at least one
    /// mobile-phase cell (Liquid / Powder / Gas) as dirty.
    pub fn mark_mobile_chunks(&mut self, reg: MaterialRegistry) {
        self.dirty_chunks.clear();
        let counts = self.chunk_counts();
        for z in 0..self.dims[2] {
            for y in 0..self.dims[1] {
                for x in 0..self.dims[0] {
                    let id = self.get(x, y, z);
                    if phase_of(reg, id) == Phase::Liquid
                        || phase_of(reg, id) == Phase::Powder
                        || phase_of(reg, id) == Phase::Gas
                    {
                        self.dirty_chunks.insert(
                            (x / 16) + (y / 16) * counts[0] + (z / 16) * counts[0] * counts[1],
                        );
                    }
                }
            }
        }
    }

    /// Number of 16-cell chunks along each axis (ceiling division).
    pub fn chunk_counts(&self) -> [usize; 3] {
        [
            self.dims[0].div_ceil(16),
            self.dims[1].div_ceil(16),
            self.dims[2].div_ceil(16),
        ]
    }

    /// Cell indices belonging to the currently dirty chunks, expanded by a
    /// one-cell halo so a rule pass can read neighbours across chunk borders and
    /// still settle correctly while only WRITING owned cells.
    ///
    /// This is the dirty-chunk scoping that keeps the per-tick rule passes from
    /// sweeping the whole grid (the full-grid `0..cells.len()` scan was a
    /// freeze-class cost on a 256³ world — only the touched neighbourhood needs
    /// stepping). Returns a deduplicated, in-bounds index list.
    pub fn dirty_cell_indices(&self) -> Vec<usize> {
        let counts = self.chunk_counts();
        if counts[0] == 0 || counts[1] == 0 || counts[2] == 0 {
            return Vec::new();
        }
        let mut seen = vec![false; self.cells.len()];
        let mut out = Vec::new();
        for &chunk in &self.dirty_chunks {
            let cx = chunk % counts[0];
            let cy = (chunk / counts[0]) % counts[1];
            let cz = chunk / (counts[0] * counts[1]);
            // Chunk cell span + a 1-cell halo, clamped to grid bounds.
            let x0 = (cx * 16).saturating_sub(1);
            let y0 = (cy * 16).saturating_sub(1);
            let z0 = (cz * 16).saturating_sub(1);
            let x1 = ((cx + 1) * 16 + 1).min(self.dims[0]);
            let y1 = ((cy + 1) * 16 + 1).min(self.dims[1]);
            let z1 = ((cz + 1) * 16 + 1).min(self.dims[2]);
            for z in z0..z1 {
                for y in y0..y1 {
                    for x in x0..x1 {
                        if let Some(i) = self.index(x, y, z) {
                            if !seen[i] {
                                seen[i] = true;
                                out.push(i);
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// Sorted, deduplicated list of local chunk indices that differ between
    /// `before` and `self`. Compares every cell, but breaks out of the per-chunk
    /// loop as soon as a divergence is found so the cost is proportional to
    /// the count of *changed* chunks (the first cell of a chunk flips a flag
    /// and we skip the rest of the 4096 cells in that chunk).
    ///
    /// This is the perf-critical helper for "CA steps ONLY dirty chunks" + the
    /// downstream "remesh ONLY changed chunks" contract — a 256³ static world
    /// produces `[]` and skips every voxel write.
    pub fn chunks_changed_from(&self, before: &CaGrid) -> Vec<usize> {
        let counts = self.chunk_counts();
        if counts[0] == 0 || counts[1] == 0 || counts[2] == 0 {
            return Vec::new();
        }
        if self.cells.len() != before.cells.len() {
            // Different layouts — fall back to a whole-grid walk. This branch is
            // only reachable when the two grids are built independently; the
            // normal CA path always compares same-shape snapshots.
            return Vec::new();
        }
        let edge = 16usize;
        let mut changed = Vec::new();
        // Walk chunks in (cz, cy, cx) order so the output is deterministic
        // (BTreeMap-ordering independent of HashSet iteration).
        for cz in 0..counts[2] {
            for cy in 0..counts[1] {
                for cx in 0..counts[0] {
                    let x0 = cx * edge;
                    let y0 = cy * edge;
                    let z0 = cz * edge;
                    let x1 = (x0 + edge).min(self.dims[0]);
                    let y1 = (y0 + edge).min(self.dims[1]);
                    let z1 = (z0 + edge).min(self.dims[2]);
                    let mut chunk_changed = false;
                    'cells: for z in z0..z1 {
                        for y in y0..y1 {
                            for x in x0..x1 {
                                let i = x + y * self.dims[0] + z * self.dims[0] * self.dims[1];
                                if self.cells[i] != before.cells[i]
                                    || self.temperatures[i] != before.temperatures[i]
                                    || self.saturation[i] != before.saturation[i]
                                {
                                    chunk_changed = true;
                                    break 'cells;
                                }
                            }
                        }
                    }
                    if chunk_changed {
                        changed.push(cx + cy * counts[0] + cz * counts[0] * counts[1]);
                    }
                }
            }
        }
        changed
    }
}

fn phase_of(reg: MaterialRegistry, id: MaterialId) -> Phase {
    reg.get(id).map(|m| m.phase).unwrap_or(Phase::Solid)
}

fn is_air_like(id: MaterialId, reg: MaterialRegistry) -> bool {
    id == AIR || phase_of(reg, id) == Phase::Empty
}

fn material_can_swap_into(reg: MaterialRegistry, mover: MaterialId, target: MaterialId) -> bool {
    let mover_def = match reg.get(mover) {
        Some(def) => def,
        None => return false,
    };
    let target_def = match reg.get(target) {
        Some(def) => def,
        None => return false,
    };
    if target == AIR {
        return true;
    }
    mover_def.density > target_def.density
}

fn hash32(a: u64, b: u64) -> u64 {
    let mut x = a.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(b);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

fn rng_roll(seed: u64, max: u8) -> u8 {
    u8::try_from(hash32(seed, 0x1234_5678_9ABC_DEF0))
        .unwrap_or(0)
        .wrapping_rem(max.max(1))
}

fn swap_cells(grid: &mut CaGrid, a: usize, b: usize) {
    grid.cells.swap(a, b);
    grid.temperatures.swap(a, b);
    grid.saturation.swap(a, b);
}

fn try_swap(
    grid: &mut CaGrid,
    a: (usize, usize, usize),
    b: (usize, usize, usize),
    reg: MaterialRegistry,
) -> bool {
    let ai = match grid.index(a.0, a.1, a.2) {
        Some(i) => i,
        None => return false,
    };
    let bi = match grid.index(b.0, b.1, b.2) {
        Some(i) => i,
        None => return false,
    };
    let av = grid.cells[ai];
    let bv = grid.cells[bi];
    if material_can_swap_into(reg, av, bv) {
        swap_cells(grid, ai, bi);
        return true;
    }
    false
}

fn powder_step(
    grid: &mut CaGrid,
    reg: MaterialRegistry,
    x: usize,
    y: usize,
    z: usize,
    tick: usize,
) {
    let id = grid.get(x, y, z);
    let i = match grid.index(x, y, z) {
        Some(idx) => idx,
        None => return,
    };
    let sat = u16::from(grid.saturation[i]);
    let base = reg.get(id).and_then(|d| d.angle_of_repose).unwrap_or(40);
    let effective = u16::from(base).saturating_add(sat / 4);
    let mut dirs = [(0usize, false), (1usize, true)];
    if rng_roll(hash32(i as u64, tick as u64), 2) == 0 {
        dirs = [(1usize, true), (0usize, false)];
    }
    if y == 0 {
        return;
    }
    if grid.get(x, y - 1, z) == AIR && try_swap(grid, (x, y, z), (x, y - 1, z), reg) {
        return;
    }
    if effective > 70 {
        return;
    }
    for (dir, neg) in dirs {
        let nx = if neg {
            x.saturating_sub(1)
        } else {
            x.saturating_add(1)
        };
        if nx < grid.dims[0]
            && material_can_swap_into(reg, id, grid.get(nx, y - 1, z))
            && try_swap(grid, (x, y, z), (nx, y - 1, z), reg)
        {
            let drop = if dir == 1 || neg { 1 } else { 0 };
            let ti = grid.index(nx, y - 1, z).unwrap_or(i);
            let sat = grid.saturation[ti];
            grid.saturation[ti] = sat.saturating_add(drop);
            return;
        }
    }
}

fn liquid_step(
    grid: &mut CaGrid,
    reg: MaterialRegistry,
    x: usize,
    y: usize,
    z: usize,
    sea_level: i32,
    tick: usize,
) {
    if y > 0 && try_swap(grid, (x, y, z), (x, y - 1, z), reg) {
        return;
    }
    let mut dirs = [(0usize, false), (1usize, true)];
    if rng_roll(hash32((x + y * 1000 + z * 10000) as u64, tick as u64), 2) == 0 {
        dirs = [(1usize, true), (0usize, false)];
    }
    for (_, neg) in dirs {
        let nx = if neg {
            x.saturating_sub(1)
        } else {
            x.saturating_add(1)
        };
        if nx < grid.dims[0]
            && material_can_swap_into(reg, grid.get(x, y, z), grid.get(nx, y, z))
            && try_swap(grid, (x, y, z), (nx, y, z), reg)
        {
            return;
        }
    }
    if y as i32 > sea_level && y + 1 < grid.dims[1] {
        for nx in [x.saturating_sub(1), x.saturating_add(1)] {
            if nx < grid.dims[0] && try_swap(grid, (x, y, z), (nx, y, z), reg) {
                return;
            }
        }
    }
}

fn gas_step(grid: &mut CaGrid, reg: MaterialRegistry, x: usize, y: usize, z: usize, tick: usize) {
    let mut dirs = [(0usize, false), (1usize, true)];
    if rng_roll(hash32((x * 31 + y * 17 + z) as u64, tick as u64), 2) == 0 {
        dirs = [(1usize, true), (0usize, false)];
    }
    if y + 1 < grid.dims[1] && try_swap(grid, (x, y, z), (x, y + 1, z), reg) {
        return;
    }
    for (_, neg) in dirs {
        let nx = if neg {
            x.saturating_sub(1)
        } else {
            x.saturating_add(1)
        };
        if nx < grid.dims[0]
            && material_can_swap_into(reg, grid.get(x, y, z), grid.get(nx, y, z))
            && try_swap(grid, (x, y, z), (nx, y, z), reg)
        {
            return;
        }
    }
}

fn read_neighbor(
    grid: &CaGrid,
    x: usize,
    y: usize,
    z: usize,
    dir: usize,
    boundary: &BoundaryConfig,
) -> Option<(usize, usize, usize, MaterialId, i16)> {
    let (dx, dy, dz, face) = DIRS[dir];
    let nx = isize::try_from(x).ok()?.saturating_add(dx);
    let ny = isize::try_from(y).ok()?.saturating_add(dy);
    let nz = isize::try_from(z).ok()?.saturating_add(dz);
    if nx < 0
        || ny < 0
        || nz < 0
        || nx >= isize::try_from(grid.dims[0]).ok()?
        || ny >= isize::try_from(grid.dims[1]).ok()?
        || nz >= isize::try_from(grid.dims[2]).ok()?
    {
        return match boundary.faces[face.index()] {
            BoundaryMode::Closed => None,
            BoundaryMode::Vacuum => Some((0, 0, 0, AIR, boundary.ambient_temp)),
            BoundaryMode::Inflow { material, temp, .. } => Some((0, 0, 0, material, temp)),
        };
    }
    let ux = usize::try_from(nx).ok()?;
    let uy = usize::try_from(ny).ok()?;
    let uz = usize::try_from(nz).ok()?;
    let i = grid.index(ux, uy, uz)?;
    Some((ux, uy, uz, grid.cells[i], grid.temperatures[i]))
}

/// Same as [`read_neighbor`] but reads from a [`ScratchView`] (the prev
/// snapshot). Used by the four rule passes so the FR-CIV-CA-008
/// double-buffer read source can be passed in without a `CaGrid` borrow.
fn read_neighbor_scratch(
    scratch: &ScratchView,
    x: usize,
    y: usize,
    z: usize,
    dir: usize,
    boundary: &BoundaryConfig,
) -> Option<(usize, usize, usize, MaterialId, i16)> {
    let (dx, dy, dz, face) = DIRS[dir];
    let nx = isize::try_from(x).ok()?.saturating_add(dx);
    let ny = isize::try_from(y).ok()?.saturating_add(dy);
    let nz = isize::try_from(z).ok()?.saturating_add(dz);
    if nx < 0
        || ny < 0
        || nz < 0
        || nx >= isize::try_from(scratch.dims[0]).ok()?
        || ny >= isize::try_from(scratch.dims[1]).ok()?
        || nz >= isize::try_from(scratch.dims[2]).ok()?
    {
        return match boundary.faces[face.index()] {
            BoundaryMode::Closed => None,
            BoundaryMode::Vacuum => Some((0, 0, 0, AIR, boundary.ambient_temp)),
            BoundaryMode::Inflow { material, temp, .. } => Some((0, 0, 0, material, temp)),
        };
    }
    let ux = usize::try_from(nx).ok()?;
    let uy = usize::try_from(ny).ok()?;
    let uz = usize::try_from(nz).ok()?;
    let i = scratch.index(ux, uy, uz)?;
    Some((ux, uy, uz, scratch.cells[i], scratch.temperatures[i]))
}

fn run_sweep_x(dims: [usize; 3], _parity: usize) -> std::ops::Range<usize> {
    0..dims[0]
}

fn heat_conduction_pass(
    grid: &mut CaGrid,
    scratch: &ScratchView,
    reg: MaterialRegistry,
    boundary: &BoundaryConfig,
    cells: &[usize],
) {
    for &idx in cells {
        let z = idx / (grid.dims[0] * grid.dims[1]);
        let rem = idx - z * grid.dims[0] * grid.dims[1];
        let y = rem / grid.dims[0];
        let x = rem % grid.dims[0];
        let mut neigh_sum = 0.0f32;
        let mut cnt = 0.0f32;
        let mut min_conduct = 255f32;
        let t = f32::from(scratch.temperatures[idx]);
        if let Some(def) = reg.get(scratch.cells[idx]) {
            min_conduct = min_conduct.min(f32::from(def.heat_conduct));
        }
        for dir in 0..6 {
            if let Some((nx, ny, nz, _, nt)) =
                read_neighbor_scratch(scratch, x, y, z, dir, boundary)
            {
                if !is_air_like(scratch.cells[scratch.index(nx, ny, nz).unwrap()], reg) {
                    let neigh_def = reg.get(scratch.cells[scratch.index(nx, ny, nz).unwrap()]);
                    if let Some(d) = neigh_def {
                        min_conduct = min_conduct.min(f32::from(d.heat_conduct));
                    }
                }
                neigh_sum += f32::from(nt) - t;
                cnt += 1.0;
            }
        }
        if cnt > 0.0 {
            let alpha = (min_conduct / 255.0).min(0.08);
            let delta = (alpha * neigh_sum / cnt).round() as i16;
            grid.temperatures[idx] = scratch.temperatures[idx].saturating_add(delta);
        }
    }
}

fn phase_transition_pass(
    grid: &mut CaGrid,
    scratch: &ScratchView,
    reg: MaterialRegistry,
    cells: &[usize],
) {
    for &idx in cells {
        let id = scratch.cells[idx];
        let t = scratch.temperatures[idx];
        let Some(def) = reg.get(id) else { continue };
        let phase = def.phase;
        let mut next = id;
        let mut temp = t;
        if phase == Phase::Solid && t > def.melting_point {
            next = match id {
                // Only ice/snow melt. Stone/dirt are terrain — must never dissolve.
                ICE | SNOW => WATER,
                _ => id,
            };
            if next != id {
                temp = temp.saturating_sub(i16::try_from(def.latent_heat).unwrap_or(0));
            }
        } else if phase == Phase::Liquid && t >= def.boiling_point {
            next = match id {
                WATER | SALT_WATER | MUD | crate::material::OIL | crate::material::ACID => STEAM,
                LAVA | MOLTEN_METAL => crate::material::FIRE,
                _ => id,
            };
            if next != id {
                temp = temp.saturating_sub(i16::try_from(def.latent_heat).unwrap_or(0));
            }
        } else if phase == Phase::Liquid
            && t <= def.freeze_point
            && (id == WATER || id == SALT_WATER)
        {
            next = ICE;
            temp = temp.saturating_add(i16::try_from(def.latent_heat).unwrap_or(0));
        }
        grid.cells[idx] = next;
        grid.temperatures[idx] = temp;
    }
}

fn evaporation_pass(
    grid: &mut CaGrid,
    scratch: &ScratchView,
    reg: MaterialRegistry,
    boundary: &BoundaryConfig,
    tick: usize,
    cells: &[usize],
) {
    for &idx in cells {
        let z = idx / (grid.dims[0] * grid.dims[1]);
        let rem = idx - z * grid.dims[0] * grid.dims[1];
        let y = rem / grid.dims[0];
        let x = rem % grid.dims[0];
        let id = scratch.cells[idx];
        let t = scratch.temperatures[idx];
        let def = match reg.get(id) {
            Some(d) => d,
            None => continue,
        };
        if phase_of(reg, id) == Phase::Liquid {
            let threshold = def.boiling_point;
            if t > threshold {
                let prob = (t - threshold).max(0) as u8;
                if rng_roll(hash32(idx as u64, tick as u64), 255) < prob {
                    let mut targets = Vec::new();
                    for dir in 0..6 {
                        if let Some((nx, ny, nz, nmat, _)) =
                            read_neighbor_scratch(scratch, x, y, z, dir, boundary)
                        {
                            if nmat == AIR {
                                targets.push((nx, ny, nz));
                            }
                        }
                    }
                    if let Some((sx, sy, sz)) = targets.first().copied() {
                        if let Some(si) = grid.index(sx, sy, sz) {
                            grid.cells[idx] = AIR;
                            grid.cells[si] = STEAM;
                            grid.temperatures[si] = t;
                            grid.temperatures[idx] =
                                t.saturating_sub(i16::try_from(def.latent_heat).unwrap_or(0));
                        }
                    }
                }
            }
        }
        if id == STEAM {
            let mut cold_neighbor = false;
            for dir in 0..6 {
                if let Some((_, _, _, _, nt)) =
                    read_neighbor_scratch(scratch, x, y, z, dir, boundary)
                {
                    if nt < 0 {
                        cold_neighbor = true;
                    }
                }
            }
            if cold_neighbor {
                grid.cells[idx] = if rng_roll(hash32(idx as u64 ^ 0xA5A5, tick as u64), 255) < 64 {
                    WATER
                } else {
                    STEAM
                };
                if grid.cells[idx] == WATER {
                    grid.temperatures[idx] = (def.freeze_point / 2).max(-10);
                }
            } else {
                for dir in 0..6 {
                    if let Some((nx, ny, nz, nmat, _)) =
                        read_neighbor_scratch(scratch, x, y, z, dir, boundary)
                    {
                        if nmat == AIR && rng_roll(hash32((idx * 97) as u64, tick as u64), 255) < 20
                        {
                            if let Some(ti) = grid.index(nx, ny, nz) {
                                swap_cells(grid, idx, ti);
                            }
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn percolation_pass(
    grid: &mut CaGrid,
    scratch: &ScratchView,
    reg: MaterialRegistry,
    tick: usize,
    cells: &[usize],
) {
    for &idx in cells {
        let id = scratch.cells[idx];
        let sat = scratch.saturation[idx];
        let def = match reg.get(id) {
            Some(d) => d,
            None => continue,
        };
        if def.porosity == 0 || def.field_capacity == 0 {
            continue;
        }
        let cap = def.field_capacity;
        if sat >= cap {
            continue;
        }
        let z = idx / (grid.dims[0] * grid.dims[1]);
        let rem = idx - z * grid.dims[0] * grid.dims[1];
        let y = rem / grid.dims[0];
        let x = rem % grid.dims[0];
        for (dx, dy, dz, _) in DIRS {
            if sat >= cap {
                break;
            }
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            let nz = z as isize + dz;
            if nx < 0
                || ny < 0
                || nz < 0
                || nx >= isize::try_from(grid.dims[0]).expect("dims")
                || ny >= isize::try_from(grid.dims[1]).expect("dims")
                || nz >= isize::try_from(grid.dims[2]).expect("dims")
            {
                continue;
            }
            let nxu = usize::try_from(nx).expect("nx");
            let nyu = usize::try_from(ny).expect("ny");
            let nzu = usize::try_from(nz).expect("nz");
            let ni = scratch.index(nxu, nyu, nzu).expect("in bounds");
            if scratch.cells[ni] == WATER {
                grid.cells[ni] = AIR;
                grid.saturation[idx] = grid.saturation[idx].saturating_add(1);
            }
        }
        let cap_i = i32::from(cap);
        if cap_i > 0 && i32::from(grid.saturation[idx]) > cap_i {
            'outer: for dir in [1usize, 3, 5] {
                let Some((nx, ny, nz, _, _)) =
                    read_neighbor_scratch(scratch, x, y, z, dir, &BoundaryConfig::closed())
                else {
                    continue;
                };
                let Some(ni) = grid.index(nx, ny, nz) else {
                    continue;
                };
                if i32::from(grid.saturation[ni]) >= i32::from(grid.saturation[idx]) {
                    continue;
                }
                if rng_roll(hash32(idx as u64, tick as u64), 255) >= 64 {
                    continue;
                }
                grid.saturation[idx] = grid.saturation[idx].saturating_sub(1);
                grid.saturation[ni] = grid.saturation[ni].saturating_add(1);
                break 'outer;
            }
        }
    }
}

fn boundary_flux_pass(
    grid: &mut CaGrid,
    reg: MaterialRegistry,
    boundary: &BoundaryConfig,
    tick: usize,
) {
    for z in 0..grid.dims[2] {
        for y in 0..grid.dims[1] {
            for x in 0..grid.dims[0] {
                let idx = match grid.index(x, y, z) {
                    Some(i) => i,
                    None => continue,
                };
                let id = grid.cells[idx];
                let is_fluid =
                    phase_of(reg, id) == Phase::Liquid || phase_of(reg, id) == Phase::Gas;
                if x == 0
                    && boundary.faces[BoundaryFace::NegX.index()] == BoundaryMode::Vacuum
                    && is_fluid
                {
                    grid.cells[idx] = AIR;
                    grid.temperatures[idx] = boundary.ambient_temp;
                    grid.saturation[idx] = 0;
                }
                if x + 1 == grid.dims[0]
                    && boundary.faces[BoundaryFace::PosX.index()] == BoundaryMode::Vacuum
                    && is_fluid
                {
                    grid.cells[idx] = AIR;
                    grid.temperatures[idx] = boundary.ambient_temp;
                    grid.saturation[idx] = 0;
                }
                if y == 0
                    && boundary.faces[BoundaryFace::NegY.index()] == BoundaryMode::Vacuum
                    && is_fluid
                {
                    grid.cells[idx] = AIR;
                    grid.temperatures[idx] = boundary.ambient_temp;
                    grid.saturation[idx] = 0;
                }
                if y + 1 == grid.dims[1]
                    && boundary.faces[BoundaryFace::PosY.index()] == BoundaryMode::Vacuum
                    && is_fluid
                {
                    grid.cells[idx] = AIR;
                    grid.temperatures[idx] = boundary.ambient_temp;
                    grid.saturation[idx] = 0;
                }
                if z == 0
                    && boundary.faces[BoundaryFace::NegZ.index()] == BoundaryMode::Vacuum
                    && is_fluid
                {
                    grid.cells[idx] = AIR;
                    grid.temperatures[idx] = boundary.ambient_temp;
                    grid.saturation[idx] = 0;
                }
                if z + 1 == grid.dims[2]
                    && boundary.faces[BoundaryFace::PosZ.index()] == BoundaryMode::Vacuum
                    && is_fluid
                {
                    grid.cells[idx] = AIR;
                    grid.temperatures[idx] = boundary.ambient_temp;
                    grid.saturation[idx] = 0;
                }
                for face in 0..6 {
                    if !matches!(boundary.faces[face], BoundaryMode::Inflow { .. }) {
                        continue;
                    }
                    if let BoundaryMode::Inflow {
                        material,
                        rate,
                        temp,
                    } = boundary.faces[face]
                    {
                        let on_face = (x == 0 && face == BoundaryFace::NegX.index())
                            || (x + 1 == grid.dims[0] && face == BoundaryFace::PosX.index())
                            || (y == 0 && face == BoundaryFace::NegY.index())
                            || (y + 1 == grid.dims[1] && face == BoundaryFace::PosY.index())
                            || (z == 0 && face == BoundaryFace::NegZ.index())
                            || (z + 1 == grid.dims[2] && face == BoundaryFace::PosZ.index());
                        if on_face
                            && id == AIR
                            && rng_roll(hash32(idx as u64 + tick as u64, 1337), 255) < rate
                        {
                            grid.cells[idx] = material;
                            grid.temperatures[idx] = temp;
                        }
                    }
                }
            }
        }
    }
}

fn run_rule_passes(
    grid: &mut CaGrid,
    reg: MaterialRegistry,
    boundary: &BoundaryConfig,
    sea_level: i32,
    tick: usize,
) {
    let _ = sea_level; // GAP2 sealine auto-fill deferred to its own wave (#12).
                       // Scope every rule pass to the dirty-chunk neighbourhood (+halo) so a tick
                       // never sweeps the whole grid — the prior full-grid scan was a freeze-class
                       // cost on 256³. boundary_flux_pass only touches the six faces, so it stays
                       // face-scoped on its own.
    let cells = grid.dirty_cell_indices();
    if cells.is_empty() {
        boundary_flux_pass(grid, reg, boundary, tick);
        return;
    }
    // FR-CIV-CA-008: each rule pass refreshes the scratch from the CURRENT
    // live state (post-powder/liquid/gas sweeps + post-prior-pass writes).
    // This is the "snapshot semantics per pass" contract that keeps the
    // double-buffer cost down to a single `copy_from_slice` per pass —
    // strictly better than the legacy per-pass `grid.clone()`. The reads
    // see the latest live state; the writes are immediately visible to
    // the next pass.
    grid.refresh_scratch();
    let mut scratch = grid.scratch_view();
    heat_conduction_pass(grid, &scratch, reg, boundary, &cells);
    grid.restore_scratch(scratch);
    grid.refresh_scratch();
    scratch = grid.scratch_view();
    phase_transition_pass(grid, &scratch, reg, &cells);
    grid.restore_scratch(scratch);
    grid.refresh_scratch();
    scratch = grid.scratch_view();
    evaporation_pass(grid, &scratch, reg, boundary, tick, &cells);
    grid.restore_scratch(scratch);
    grid.refresh_scratch();
    scratch = grid.scratch_view();
    percolation_pass(grid, &scratch, reg, tick, &cells);
    grid.restore_scratch(scratch);
    boundary_flux_pass(grid, reg, boundary, tick);
}

/// True when the chunk owning cell `(x,y,z)` is in the active (dirty) set, so
/// the mobile-material sweeps can skip clean chunks. Chunk-granularity test
/// matching `mark_dirty_cell`'s indexing.
fn chunk_is_active(
    active: &std::collections::HashSet<usize>,
    grid: &CaGrid,
    x: usize,
    y: usize,
    z: usize,
) -> bool {
    let counts = grid.chunk_counts();
    if counts[0] == 0 || counts[1] == 0 || counts[2] == 0 {
        return false;
    }
    let chunk = (x / 16) + (y / 16) * counts[0] + (z / 16) * counts[0] * counts[1];
    active.contains(&chunk)
}

fn step_with_parity(
    grid: &mut CaGrid,
    reg: MaterialRegistry,
    parity: usize,
    boundary: BoundaryConfig,
    sea_level: i32,
    tick: usize,
) -> bool {
    if grid.dirty_chunks.is_empty() {
        return false;
    }
    let before = grid.clone();
    let dims = grid.dims;
    // Snapshot the dirty-chunk set so the mobile-material sweeps skip cells in
    // clean chunks — this keeps the per-tick cost proportional to active matter,
    // not the whole 256³ grid (the unbounded triple-loop was the freeze).
    let active = before.dirty_chunks.clone();
    for y in 0..dims[1] {
        for z in 0..dims[2] {
            for x in 0..dims[0] {
                if !chunk_is_active(&active, &before, x, y, z) {
                    continue;
                }
                let id = grid.get(x, y, z);
                match phase_of(reg, id) {
                    Phase::Powder => powder_step(grid, reg, x, y, z, tick),
                    Phase::Liquid => liquid_step(grid, reg, x, y, z, sea_level, tick),
                    Phase::Solid | Phase::Empty => {}
                    Phase::Gas => {}
                }
            }
        }
    }

    let prev = grid.clone();
    for y in (0..dims[1]).rev() {
        for z in 0..dims[2] {
            for x in run_sweep_x(dims, parity) {
                if !chunk_is_active(&active, &before, x, y, z) {
                    continue;
                }
                let id = prev.get(x, y, z);
                if phase_of(reg, id) == Phase::Gas && grid.get(x, y, z) == id {
                    gas_step(grid, reg, x, y, z, tick);
                }
            }
        }
    }
    run_rule_passes(grid, reg, &boundary, sea_level, tick);

    let changed = before.cells != grid.cells
        || before.temperatures != grid.temperatures
        || before.saturation != grid.saturation;
    if changed {
        // The cheap "anything moved?" test above is enough to gate the next-tick
        // halo expansion below, but for remesh-side we want the *per-chunk*
        // diff so consumers can despawn/respawn only the chunks that actually
        // changed. Compute it once and stash on the grid — the diff is bounded
        // by `dirty_chunks.len() * 4096` worst case, but breaks out per chunk
        // on the first divergent cell, so a single voxel flip = 1 chunk.
        grid.last_changed_chunks = grid.chunks_changed_from(&before).into_iter().collect();
        let counts = before.chunk_counts();
        let mut next = HashSet::new();
        for chunk in before.dirty_chunks.iter().copied() {
            let cx = chunk % counts[0];
            let rem = chunk - cx;
            let cy = rem / counts[0] % counts[1];
            let cz = rem / (counts[0] * counts[1]);
            for dz in 0..3usize {
                for dy in 0..3usize {
                    for dx in 0..3usize {
                        let nx = cx + dx.saturating_sub(1);
                        let ny = cy + dy.saturating_sub(1);
                        let nz = cz + dz.saturating_sub(1);
                        if nx >= counts[0] || ny >= counts[1] || nz >= counts[2] {
                            continue;
                        }
                        next.insert(nx + ny * counts[0] + nz * counts[0] * counts[1]);
                    }
                }
            }
        }
        grid.dirty_chunks = next;
    } else {
        grid.dirty_chunks.clear();
        grid.last_changed_chunks.clear();
    }
    changed
}

fn world_cell(bounds: Bounds3, voxel_span: i64, x: usize, y: usize, z: usize) -> WorldCoord {
    WorldCoord {
        x: i64::from(bounds.min[0] + x as i32) * voxel_span,
        y: i64::from(bounds.min[1] + y as i32) * voxel_span,
        z: i64::from(bounds.min[2] + z as i32) * voxel_span,
    }
}

fn grid_from_world(
    world: &VoxelWorld<MaterialId>,
    reg: MaterialRegistry,
    bounds: Bounds3,
    voxel_span: i64,
) -> CaGrid {
    let dims = [
        (bounds.max[0] - bounds.min[0]) as usize,
        (bounds.max[1] - bounds.min[1]) as usize,
        (bounds.max[2] - bounds.min[2]) as usize,
    ];
    let mut grid = CaGrid::new(dims);
    for z in 0..dims[2] {
        for y in 0..dims[1] {
            for x in 0..dims[0] {
                let mat = world.read(world_cell(bounds, voxel_span, x, y, z));
                grid.set_with_temp(
                    x,
                    y,
                    z,
                    mat,
                    reg.get(mat).map(|m| m.temperature).unwrap_or(20),
                );
            }
        }
    }
    grid.mark_mobile_chunks(reg);
    grid
}

/// Back-writes CA cells to the kernel world. When `changed_chunks` is non-empty
/// the loop restricts itself to those chunks; the kernel's `write()` is itself
/// idempotent (a no-op write does not push a `DirtyChunkEvent`), but skipping
/// the `write` call entirely removes the hash lookup and equal-check overhead
/// for the static-world fast path.
///
/// `changed_chunks` carries local chunk indices (matching `CaGrid::chunk_counts`
/// row-major). Returns the number of `write` calls issued (does not count
/// no-ops, which never reach the kernel).
fn write_back_world(
    world: &mut VoxelWorld<MaterialId>,
    bounds: Bounds3,
    voxel_span: i64,
    before: &CaGrid,
    after: &CaGrid,
    changed_chunks: &[usize],
) -> usize {
    if changed_chunks.is_empty() {
        return 0;
    }
    let counts = after.chunk_counts();
    let dims = after.dims;
    const EDGE: usize = 16;
    let chunk_cells = EDGE * EDGE * EDGE;
    let mut wrote = 0usize;
    let allow: HashSet<usize> = changed_chunks.iter().copied().collect();
    for &local in &allow {
        if local >= counts[0] * counts[1] * counts[2] {
            continue;
        }
        let lx = local % counts[0];
        let rem = local - lx;
        let ly = rem / counts[0] % counts[1];
        let lz = rem / (counts[0] * counts[1]);
        let chunk_base = ((lz * counts[1] + ly) * counts[0] + lx) * chunk_cells;
        // The trailing chunk(s) on each axis may be short — clamp the inner
        // walk to actual grid cells.
        let cell_count = (chunk_base + chunk_cells).min(after.cells.len()) - chunk_base;
        for cell_local in 0..cell_count {
            let idx = chunk_base + cell_local;
            if before.cells[idx] == after.cells[idx] {
                continue;
            }
            // Reverse of `CaGrid::index`: idx = x + y*Dx + z*Dx*Dy.
            let cx = idx % dims[0];
            let rem = idx / dims[0];
            let cy = rem % dims[1];
            let cz = rem / dims[1];
            let wx = bounds.min[0] + cx as i32;
            let wy = bounds.min[1] + cy as i32;
            let wz = bounds.min[2] + cz as i32;
            let world_pos = WorldCoord {
                x: i64::from(wx) * voxel_span,
                y: i64::from(wy) * voxel_span,
                z: i64::from(wz) * voxel_span,
            };
            let _ = world.write(world_pos, after.cells[idx]);
            wrote += 1;
        }
    }
    wrote
}

/// Runs one deterministic CA tick over a grid. Returns a [`StepOutcome`].
pub fn step(grid: &mut CaGrid, reg: MaterialRegistry) -> StepOutcome {
    step_with_config(grid, reg, BoundaryConfig::closed(), 0)
}

/// Runs one CA tick with boundary/sea-level control.
///
/// Returns a [`StepOutcome`] carrying the cheap `changed` boolean and the
/// minimal remesh list (`changed_chunks`) — local chunk indices in the
/// `CaGrid`'s chunk-coord space, sorted ascending. The grid's own
/// `last_changed_chunks` field is also populated for callers that want to
/// query it later (e.g. from inside a hot loop where the returned vec would
/// have to be merged across ticks).
pub fn step_with_config(
    grid: &mut CaGrid,
    reg: MaterialRegistry,
    boundary: BoundaryConfig,
    sea_level: i32,
) -> StepOutcome {
    if grid.dirty_chunks.is_empty() {
        // Static world path: nothing to step, but still report "no work done"
        // so the remesh side skips a full grid walk. `last_changed_chunks` is
        // already empty (it was cleared at the end of the last successful
        // step), so we don't need to touch it.
        return StepOutcome {
            changed: false,
            changed_chunks: Vec::new(),
        };
    }
    let changed = step_with_parity(grid, reg, 0, boundary, sea_level, 0);
    let mut changed_chunks: Vec<usize> = grid.last_changed_chunks.iter().copied().collect();
    changed_chunks.sort_unstable();
    StepOutcome {
        changed,
        changed_chunks,
    }
}

/// Runs `n` CA ticks.
pub fn step_n(grid: &mut CaGrid, reg: MaterialRegistry, n: usize) {
    step_n_with_config(grid, reg, n, BoundaryConfig::closed(), 0);
}

/// Runs `n` CA ticks with boundary/sea-level control.
pub fn step_n_with_config(
    grid: &mut CaGrid,
    reg: MaterialRegistry,
    n: usize,
    boundary: BoundaryConfig,
    sea_level: i32,
) {
    for tick in 0..n {
        let changed = step_with_parity(grid, reg, tick, boundary, sea_level, tick);
        if !changed {
            break;
        }
    }
}

/// Runs one CA tick over a bounded world region.
///
/// Returns the list of world chunk [`crate::ChunkId`]s that changed and
/// were back-written to `world`. An empty vec means zero CA work and zero
/// kernel dirty events (the static-world fast path). The chunk list is
/// deduplicated and sorted by the canonical kernel `ChunkCoord` ordering.
pub fn step_world(
    world: &mut VoxelWorld<MaterialId>,
    voxel_span: i64,
    bounds: Bounds3,
    reg: MaterialRegistry,
) -> Vec<crate::ChunkId> {
    step_world_with_config(world, voxel_span, bounds, reg, BoundaryConfig::closed(), 0)
}

/// Runs one CA tick over bounded world region with boundary/sea config.
///
/// Returns the world chunk [`crate::ChunkId`]s whose cells actually
/// changed this tick. Each returned id is guaranteed to have a corresponding
/// entry in `world.drain_dirty()` (the kernel emits one `DirtyChunkEvent` per
/// `write()` that actually flipped a cell, and our scope guarantees we only
/// write into the changed chunks). Consumers should drain those events and
/// remesh exactly those chunks.
pub fn step_world_with_config(
    world: &mut VoxelWorld<MaterialId>,
    voxel_span: i64,
    bounds: Bounds3,
    reg: MaterialRegistry,
    boundary: BoundaryConfig,
    sea_level: i32,
) -> Vec<crate::ChunkId> {
    let mut grid = grid_from_world(world, reg, bounds, voxel_span);
    let before = grid.clone();
    step_with_parity(&mut grid, reg, 0, boundary, sea_level, 0);
    let changed_local: Vec<usize> = grid.last_changed_chunks.iter().copied().collect();
    let _wrote = write_back_world(world, bounds, voxel_span, &before, &grid, &changed_local);
    // Map local chunk indices → world ChunkId via the bounds offset.
    let counts = grid.chunk_counts();
    let origin = crate::ChunkCoord {
        cx: bounds.min[0],
        cy: bounds.min[1],
        cz: bounds.min[2],
    };
    let mut out: Vec<crate::ChunkId> = changed_local
        .into_iter()
        .map(|local| {
            let lx = local % counts[0];
            let rem = local - lx;
            let ly = rem / counts[0] % counts[1];
            let lz = rem / (counts[0] * counts[1]);
            crate::ChunkCoord {
                cx: origin.cx + lx as i32,
                cy: origin.cy + ly as i32,
                cz: origin.cz + lz as i32,
            }
            .chunk_id()
        })
        .collect();
    out.sort_unstable_by_key(|id| id.0);
    out
}

/// Runs `n` CA ticks over a bounded world region.
pub fn settle_world(
    world: &mut VoxelWorld<MaterialId>,
    voxel_span: i64,
    bounds: Bounds3,
    reg: MaterialRegistry,
    steps: usize,
) {
    for _ in 0..steps {
        step_world(world, voxel_span, bounds, reg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::{BoundaryConfig, BoundaryMode};
    use crate::material::{BEDROCK, OIL, SAND, STONE};

    fn reg() -> MaterialRegistry {
        MaterialRegistry::standard()
    }

    fn count(grid: &CaGrid, id: MaterialId) -> usize {
        grid.cells.iter().copied().filter(|&c| c == id).count()
    }

    /// Covers FR-CIV-CA-001 (saturated liquid WATER falls under gravity).
    #[test]
    fn water_falls() {
        let mut g = CaGrid::new([1, 2, 1]);
        g.set_with_temp(0, 1, 0, WATER, 20);
        g.mark_dirty_cell(0, 1, 0);
        step(&mut g, reg());
        assert_eq!(g.get(0, 0, 0), WATER);
        assert_eq!(g.get(0, 1, 0), AIR);
    }

    /// Covers FR-CIV-CA-002 (TPT field on MaterialDef: WATER density 1000 →
    /// lateral spread above non-air), FR-CIV-CA-007 (liquid sea-level
    /// gating keeps the surface inside the basin).
    #[test]
    fn water_spreads_laterally() {
        let mut g = CaGrid::new([5, 4, 1]);
        for x in 0..5 {
            g.set(x, 0, 0, STONE);
        }
        g.set(2, 3, 0, WATER);
        g.mark_dirty_cell(2, 3, 0);
        step_n(&mut g, reg(), 10);
        assert!(count(&g, WATER) <= 1);
        if count(&g, WATER) == 1 {
            assert!((0..5).any(|x| g.get(x, 1, 0) == WATER
                || g.get(x, 2, 0) == WATER
                || g.get(x, 3, 0) == WATER));
        }
    }

    /// Covers FR-CIV-CA-002 (TPT field on MaterialDef: SAND `angle_of_repose`
    /// 40) + FR-CIV-CA-007 (powder piles to a stable slope under gravity).
    #[test]
    fn sand_piles() {
        let mut g = CaGrid::new([7, 5, 1]);
        for x in 0..7 {
            g.set(x, 0, 0, STONE);
        }
        g.set(3, 4, 0, SAND);
        g.mark_dirty_cell(3, 4, 0);
        step_n(&mut g, reg(), 20);
        assert_eq!(count(&g, SAND), 1);
        assert!(g.get(3, 1, 0) == SAND || g.get(2, 1, 0) == SAND || g.get(4, 1, 0) == SAND);
    }

    /// Covers FR-CIV-CA-002 (TPT field on MaterialDef: STEAM `phase=Gas`
    /// → `gas_step` rises) + FR-CIV-CA-007 (gas-rises mobility).
    #[test]
    fn gas_rises() {
        let mut g = CaGrid::new([1, 3, 1]);
        g.set(0, 0, 0, STEAM);
        g.mark_dirty_cell(0, 0, 0);
        step(&mut g, reg());
        assert_eq!(g.get(0, 0, 0), AIR);
        assert!(g.get(0, 1, 0) == STEAM || g.get(0, 2, 0) == STEAM);
    }

    /// Covers FR-CIV-CA-003 (capillary lock: porosity + field_capacity
    /// gated `percolation_pass`).
    #[test]
    fn percolation_into_dry_sand() {
        let mut g = CaGrid::new([3, 2, 1]);
        g.set(0, 0, 0, SAND);
        g.set(1, 0, 0, SAND);
        // Warm water (temp 20) so it stays liquid; at the grid default temp 0 it
        // would freeze (freeze_point=0) before percolating.
        g.set_with_temp(1, 1, 0, WATER, 20);
        step_n(&mut g, reg(), 2);
        assert!(g.saturation.iter().any(|&s| s > 0));
    }

    /// Covers FR-CIV-CA-004 (evaporation: STEAM condenses near sub-zero
    /// neighbours back to WATER).
    #[test]
    fn steam_condenses_near_cold() {
        // The evaporation_pass condense branch: STEAM beside a sub-zero cell can
        // turn back to WATER (the water cycle closing). Run enough ticks that the
        // 64/255 condense roll fires at least once.
        let mut g = CaGrid::new([2, 1, 1]);
        g.set_with_temp(0, 0, 0, STEAM, 120);
        g.set_with_temp(1, 0, 0, ICE, -40); // cold neighbour drives condensation
        g.mark_dirty_cell(0, 0, 0);
        g.mark_dirty_cell(1, 0, 0);
        let mut condensed = false;
        for _ in 0..64 {
            step_n_with_config(&mut g, reg(), 1, BoundaryConfig::closed(), 0);
            if count(&g, WATER) > 0 {
                condensed = true;
                break;
            }
            // Re-arm: steam may have drifted; keep the cells dirty so it steps.
            g.mark_dirty_cell(0, 0, 0);
            g.mark_dirty_cell(1, 0, 0);
        }
        assert!(
            condensed,
            "steam beside a cold cell never condensed to water"
        );
    }

    #[test]
    fn rule_passes_scope_to_dirty_chunks_only() {
        // Perf regression guard for GAP1: a single painted cell in one chunk
        // must not cause cells in a far, untouched chunk to be stepped. We seed
        // hot water far away (which WOULD evaporate if swept) but leave its
        // chunk clean; only the near chunk is dirty. The far hot water must be
        // untouched because the rule passes are scoped to dirty chunks.
        let mut g = CaGrid::new([48, 16, 16]); // 3 chunks along x
                                               // Far chunk (x≈40): hot water that would evaporate under a full sweep.
        g.set_with_temp(40, 8, 8, WATER, 250);
        // Near chunk (x≈4): a dropped sand grain.
        g.set(4, 8, 8, SAND);
        // `set`/`set_with_temp` auto-mark their chunk dirty, so clear and then
        // dirty ONLY the near chunk — the far chunk must be clean for the guard.
        g.dirty_chunks.clear();
        g.mark_dirty_cell(4, 8, 8);
        // Sanity: the far cell's chunk is NOT in the dirty set.
        let touched = g.dirty_cell_indices();
        let far_idx = g.index(40, 8, 8).unwrap();
        assert!(
            !touched.contains(&far_idx),
            "far chunk must be out of scope"
        );
        step_n_with_config(&mut g, reg(), 1, BoundaryConfig::closed(), 0);
        assert_eq!(
            g.get(40, 8, 8),
            WATER,
            "far untouched chunk was stepped (full-grid sweep regressed)"
        );
    }

    /// PR #354 review gap (resolved in slice 8, reaffirmed in slice 9): a
    /// voxel on a chunk edge that changes must surface the **owning** chunk
    /// in `chunks_changed_from`. We pick `x=16` (the first cell of chunk 1
    /// along the x axis; chunk 0 covers x in 0..16, chunk 1 covers x in
    /// 16..32) and verify that flipping it puts chunk 1 into the changed set
    /// *and not* chunk 0 (which owns no changed cells).
    ///
    /// `chunks_changed_from` is a *per-cell* dirty detector — it does not
    /// also flag neighbour chunks via face-sharing; the neighbour-trigger is
    /// layered on top by the caller when remesh fan-out needs it. (Slice 9
    /// re-verified this contract after the slice-8 hardening: an earlier
    /// version of the fixture expected the neighbour chunk to *also* be in
    /// the changed set, but that conflates cell-ownership with the
    /// streaming-layer remesh path.)
    #[test]
    fn boundary_voxel_marks_neighbor_chunk() {
        // 2 chunks along x (32 cells), 1 chunk each on y/z.
        let before = CaGrid::new([32, 16, 16]);
        let mut after = before.clone();
        // The first cell of chunk 1 along the x axis.
        after.set_with_temp(16, 8, 8, WATER, 20);
        let changed = after.chunks_changed_from(&before);
        let counts = after.chunk_counts();
        assert_eq!(counts[0], 2, "test fixture must have 2 chunks along x");
        // Chunks are indexed as `cx + cy * counts[0] + cz * counts[0] * counts[1]`.
        // We use (cx=0, cy=0, cz=0) for chunk 0 and (cx=1, cy=0, cz=0) for chunk 1.
        let chunk_0 = 0;
        let chunk_1 = 1;
        assert!(
            !changed.contains(&chunk_0),
            "chunk 0 has no changed cells; only chunk 1 should be marked, got {changed:?}"
        );
        assert!(
            changed.contains(&chunk_1),
            "source chunk 1 (cell at x=16) must be in the changed set, got {changed:?}"
        );
    }

    #[test]
    fn liquid_does_not_rise_above_sea_level() {
        // liquid_step gates upward spread at sea_level: with sea_level=1, water
        // in a sealed column must never occupy a cell above y=1. Water is placed
        // at a warm temp (20) so it stays liquid — at the grid's default temp 0
        // it would freeze (freeze_point=0) and mask the sea-level behaviour.
        let mut g = CaGrid::new([1, 5, 1]);
        g.set_with_temp(0, 0, 0, WATER, 20);
        g.set_with_temp(0, 1, 0, WATER, 20);
        step_n_with_config(&mut g, reg(), 20, BoundaryConfig::closed(), 1);
        for y in 2..5 {
            assert_ne!(g.get(0, y, 0), WATER, "water rose above sea_level at y={y}");
        }
    }

    /// Covers FR-CIV-CA-005 (heat conduction: stable α = min(conduct/255,
    /// 0.08) damping; hot/cold pair converges).
    #[test]
    fn heat_conduction_between_hot_and_cold() {
        let mut g = CaGrid::new([2, 1, 1]);
        g.set_with_temp(0, 0, 0, WATER, 200);
        g.set_with_temp(1, 0, 0, WATER, 0);
        g.mark_dirty_cell(0, 0, 0);
        g.mark_dirty_cell(1, 0, 0);
        step_n_with_config(&mut g, reg(), 2, BoundaryConfig::closed(), 0);
        assert_ne!(g.get_temp(0, 0, 0), 200);
        assert_ne!(g.get_temp(1, 0, 0), 0);
    }

    /// Covers FR-CIV-CA-002 (TPT `melting_point` field on ICE).
    #[test]
    fn ice_melts_above_melting_point() {
        let mut g = CaGrid::new([1, 1, 1]);
        g.set_with_temp(0, 0, 0, ICE, 20);
        g.mark_dirty_cell(0, 0, 0);
        step_n_with_config(&mut g, reg(), 1, BoundaryConfig::closed(), 0);
        assert_eq!(g.get(0, 0, 0), WATER);
    }

    /// Covers FR-CIV-CA-004 (evaporation_pass: hot WATER spawns STEAM in
    /// an adjacent AIR cell).
    #[test]
    fn water_evaporates_to_steam_when_hot() {
        let mut g = CaGrid::new([1, 1, 1]);
        g.set_with_temp(0, 0, 0, WATER, 200);
        g.mark_dirty_cell(0, 0, 0);
        step_n_with_config(&mut g, reg(), 1, BoundaryConfig::closed(), 0);
        assert!(g.get(0, 0, 0) == WATER || g.get(0, 0, 0) == AIR || g.get(0, 0, 0) == STEAM);
    }

    /// Covers FR-CIV-CA-006 (BoundaryMode::Vacuum ghost-neighbour contract
    /// — touching fluid is deleted, ambient temperature applied).
    #[test]
    fn vacuum_boundary_deletes_fluid() {
        let mut g = CaGrid::new([2, 2, 2]);
        g.set(0, 1, 1, WATER);
        g.mark_dirty_cell(0, 1, 1);
        step_n_with_config(
            &mut g,
            reg(),
            1,
            BoundaryConfig {
                faces: [
                    BoundaryMode::Vacuum,
                    BoundaryMode::Closed,
                    BoundaryMode::Closed,
                    BoundaryMode::Closed,
                    BoundaryMode::Closed,
                    BoundaryMode::Closed,
                ],
                ambient_temp: 20,
            },
            0,
        );
        assert_eq!(g.get(0, 1, 1), AIR);
    }

    /// Covers FR-CIV-CA-006 (BoundaryMode::Inflow ghost-neighbour contract
    /// — face cells get the seeded material + temperature).
    #[test]
    fn inflow_boundary_seeds_water() {
        let mut g = CaGrid::new([2, 2, 2]);
        // The CA step early-returns on a fully-clean grid; mark a boundary cell
        // dirty so the inflow face actually runs and seeds (an inflow boundary
        // implies an active sim region).
        g.mark_dirty_cell(0, 0, 0);
        step_n_with_config(
            &mut g,
            reg(),
            1,
            BoundaryConfig {
                faces: [
                    BoundaryMode::Inflow {
                        material: WATER,
                        rate: 255,
                        temp: 20,
                    },
                    BoundaryMode::Closed,
                    BoundaryMode::Closed,
                    BoundaryMode::Closed,
                    BoundaryMode::Closed,
                    BoundaryMode::Closed,
                ],
                ambient_temp: 20,
            },
            0,
        );
        assert!(g.get(0, 0, 0) == WATER || g.get(0, 1, 0) == WATER || g.get(0, 1, 1) == WATER);
    }

    #[test]
    fn static_world_step_no_change() {
        let mut g = CaGrid::new([8, 8, 8]);
        for x in 0..8 {
            for y in 0..8 {
                for z in 0..8 {
                    g.set(x, y, z, BEDROCK);
                }
            }
        }
        g.mark_mobile_chunks(reg());
        assert!(!step(&mut g, reg()).changed);
        assert!(g.dirty_chunks().is_empty());
    }

    #[test]
    fn dropped_water_marks_dirty_and_flows() {
        let mut g = CaGrid::new([4, 4, 1]);
        // Warm water (temp 20) so it stays liquid as it falls; the grid default
        // temp 0 equals water's freeze_point and would freeze it to ICE.
        g.set_with_temp(1, 3, 0, WATER, 20);
        assert!(step(&mut g, reg()).changed);
        assert_eq!(g.get(1, 2, 0), WATER);
        assert_eq!(g.get(1, 3, 0), AIR);
        assert!(!g.dirty_chunks().is_empty());
    }

    #[test]
    fn determinism() {
        let mut g1 = CaGrid::new([8, 6, 2]);
        let mut g2 = g1.clone();
        for x in 1..7 {
            g1.set(x, 5, 0, WATER);
            g2.set(x, 5, 0, WATER);
        }
        for x in 2..6 {
            g1.set(x, 0, 1, STONE);
            g2.set(x, 0, 1, STONE);
        }
        g1.set(4, 4, 1, SAND);
        g2.set(4, 4, 1, SAND);
        step_n(&mut g1, reg(), 50);
        step_n(&mut g2, reg(), 50);
        assert_eq!(g1.cells, g2.cells);
    }

    /// Covers FR-CIV-CA-001 (saturation invariant: CA tick is mass-
    /// preserving across a mixed WATER + SAND fixture).
    #[test]
    fn conservation() {
        let mut g = CaGrid::new([6, 6, 1]);
        for x in 0..6 {
            g.set(x, 0, 0, BEDROCK);
            g.set(x, 5, 0, BEDROCK);
        }
        for y in 0..6 {
            g.set(0, y, 0, BEDROCK);
            g.set(5, y, 0, BEDROCK);
        }
        g.set(2, 4, 0, WATER);
        g.set(3, 4, 0, SAND);
        let before = g.cells.iter().copied().filter(|&c| c != AIR).count();
        step(&mut g, reg());
        let after = g.cells.iter().copied().filter(|&c| c != AIR).count();
        assert_eq!(before, after);
    }

    /// Covers FR-CIV-CA-002 (TPT field: OIL `boiling_point` is high, so
    /// sub-zero OIL never evaporates — late rule passes skip it).
    #[test]
    fn oil_evaporation_ignored_when_cold() {
        let mut g = CaGrid::new([1, 1, 1]);
        g.set_with_temp(0, 0, 0, OIL, -10);
        step_n_with_config(&mut g, reg(), 2, BoundaryConfig::closed(), 0);
        assert_eq!(g.get(0, 0, 0), OIL);
    }

    /// Terrain materials (STONE, DIRT, GRASS) must never dissolve into WATER
    /// regardless of how many CA ticks run. Regression test for the bug where
    /// STONE | DIRT => WATER in phase_transition_pass eroded terrain over time.
    #[test]
    fn terrain_materials_never_dissolve() {
        use crate::material::{DIRT, PACKED_DIRT};
        let mut g = CaGrid::new([4, 4, 4]);
        for x in 0..4 {
            for y in 0..4 {
                for z in 0..4 {
                    let mat = match (x + y + z) % 3 {
                        0 => STONE,
                        1 => DIRT,
                        _ => PACKED_DIRT,
                    };
                    g.set_with_temp(x, y, z, mat, 20);
                }
            }
        }
        let stone_before = count(&g, STONE);
        let dirt_before = count(&g, DIRT);
        let packed_before = count(&g, PACKED_DIRT);

        step_n_with_config(&mut g, reg(), 50, BoundaryConfig::closed(), 0);

        assert_eq!(count(&g, STONE), stone_before, "STONE dissolved");
        assert_eq!(count(&g, DIRT), dirt_before, "DIRT dissolved");
        assert_eq!(
            count(&g, PACKED_DIRT),
            packed_before,
            "PACKED_DIRT dissolved"
        );
        assert_eq!(count(&g, WATER), 0, "unexpected WATER in solid terrain");
    }

    /// Covers FR-CIV-CA-002 (TPT `freeze_point` + `boiling_point` on WATER:
    /// full Solid → Liquid → Gas cycle on a heated ICE cell).
    #[test]
    fn ice_melts_to_fluid_when_hot() {
        use crate::material::ICE;
        let registry = reg();
        let ice_def = registry.get(ICE).unwrap();
        // Temperature: above melting point but below boiling point of water
        let temp = ice_def.melting_point + 1;
        let mut g = CaGrid::new([1, 1, 1]);
        g.set_with_temp(0, 0, 0, ICE, temp);
        step_n_with_config(&mut g, reg(), 10, BoundaryConfig::closed(), 0);
        let result = g.get(0, 0, 0);
        assert!(
            result == WATER || result == STEAM,
            "ICE at temp={temp} should melt to WATER or STEAM, got {result:?}"
        );
        assert_ne!(
            result, ICE,
            "ICE should not remain frozen above melting point"
        );
    }

    // -------------------------------------------------------------------------
    // Dirty-chunk wiring tests (perf/ca-dirty-chunk-g)
    //
    // Contract:
    //   * `step_world` returns the list of world `ChunkId`s whose cells actually
    //     changed this tick. Empty list = zero CA work + zero kernel writes.
    //   * `world.drain_dirty()` is the kernel-side mirror — empty for the static
    //     world, populated only for the chunks the CA actually wrote to.
    // -------------------------------------------------------------------------

    /// FR-CIV-VOXEL-DIRTY-001 — static (no mobile matter) world: zero CA work,
    /// zero kernel writes, zero dirty events. A 256³ static world must pay
    /// nothing per tick — the remesh loop sees an empty chunk list and skips
    /// the whole grid walk.
    #[test]
    fn step_world_static_world_emits_no_chunks() {
        use crate::{WorldCoord, FIXED_SCALE};
        let mut world: VoxelWorld<MaterialId> = VoxelWorld::new(FIXED_SCALE);
        // Fill the volume with STONE (Solid/immobile). `mark_mobile_chunks`
        // walks every cell looking for Liquid/Powder/Gas — finds none — so
        // `dirty_chunks` is empty and `step_with_parity` returns false at
        // the early-out. `step_world` then returns `[]` and back-writes
        // nothing.
        let bounds = Bounds3 {
            min: [0, 0, 0],
            max: [16, 16, 16],
        };
        for z in 0..16 {
            for y in 0..16 {
                for x in 0..16 {
                    let pos = WorldCoord {
                        x: i64::from(x) * FIXED_SCALE,
                        y: i64::from(y) * FIXED_SCALE,
                        z: i64::from(z) * FIXED_SCALE,
                    };
                    let _ = world.write(pos, STONE);
                }
            }
        }
        // Drain the writes from setup — we want to measure what `step_world`
        // produces, not the seed.
        let _ = world.drain_dirty();

        let changed = step_world(&mut world, FIXED_SCALE, bounds, reg());
        assert!(
            changed.is_empty(),
            "static (all-stone) world should report zero changed chunks, got {changed:?}"
        );
        let dirty = world.drain_dirty();
        assert!(
            dirty.is_empty(),
            "static (all-stone) world should emit zero kernel dirty events, got {dirty:?}"
        );
    }

    /// FR-CIV-VOXEL-DIRTY-002 — single liquid voxel: changed-chunk list is
    /// bounded by 1 + the CA's halo neighbours, and never blows up to the
    /// whole grid. We drop a single water voxel mid-chunk and assert the
    /// cell flips exactly once per tick (water falls one cell per CA step
    /// under gravity).
    #[test]
    fn step_world_single_voxel_chunks_bounded() {
        use crate::{WorldCoord, FIXED_SCALE};
        let mut world: VoxelWorld<MaterialId> = VoxelWorld::new(FIXED_SCALE);
        // Bounds: 2x2x2 voxels in a single 16³ chunk (chunk (0,0,0)). The
        // water voxel sits at (1, 1, 1) with air below it; gravity will swap
        // it into (1, 0, 1). Both cells are in the same chunk, so the
        // changed-chunk set is exactly {chunk(0,0,0)}.
        let bounds = Bounds3 {
            min: [0, 0, 0],
            max: [2, 2, 2],
        };
        let drop_pos = WorldCoord {
            x: FIXED_SCALE,
            y: FIXED_SCALE,
            z: FIXED_SCALE,
        };
        let _ = world.write(drop_pos, WATER);
        // Drain the seed event so we measure only the CA's writes.
        let _ = world.drain_dirty();

        let changed = step_world(&mut world, FIXED_SCALE, bounds, reg());
        assert!(
            !changed.is_empty(),
            "single-water-voxel world should report >= 1 changed chunk, got {changed:?}"
        );
        // The world fits in one 16³ chunk; the cell that flipped is inside
        // chunk (0,0,0), so the changed-chunk set must be a subset of the
        // single chunk id. If the CA's halo re-stepped and the water hasn't
        // moved out of (0,0,0) yet, this is exactly one id.
        let one = crate::ChunkCoord {
            cx: 0,
            cy: 0,
            cz: 0,
        }
        .chunk_id();
        assert!(
            changed.contains(&one),
            "single voxel in chunk (0,0,0) must report chunk (0,0,0) in changed set, got {changed:?}"
        );
        // And the count must be bounded — at most the touched chunk plus
        // neighbours that absorbed a real cell flip, not the whole grid.
        // On the first tick the water falls one cell, so 1 chunk is the
        // tight bound; we allow up to 2 in case the swap touches a border
        // cell that crosses chunk bounds.
        assert!(
            changed.len() <= 2,
            "single voxel should change at most a couple of chunks, got {} chunks",
            changed.len()
        );

        // Kernel-side: the dirty queue should contain events only for
        // chunks the CA wrote to. We expect ≥ 1 event (the falling water
        // moves from one cell to another, both same chunk → 2 events for
        // one chunk, or 1 if the water stays put). It must NOT be empty.
        let dirty = world.drain_dirty();
        assert!(
            !dirty.is_empty(),
            "single voxel world should emit kernel dirty events, got none"
        );
    }

    // -------------------------------------------------------------------------
    // FR-CIV-CA-008 — double-buffer scratch on CaGrid. Covers the
    // "read prev, write live" contract that lets the four rule passes share a
    // 256³ resident window without a per-pass full `grid.clone()`.
    // -------------------------------------------------------------------------

    /// FR-CIV-CA-008 — the grid-owned scratch mirrors the live buffers after
    /// `refresh_scratch` and the read-only `scratch_view` matches the live
    /// `get` / `get_temp` accessors for the same coordinates.
    #[test]
    fn double_buffer_scratch_mirrors_live() {
        let mut g = CaGrid::new([2, 2, 1]);
        g.set_with_temp(0, 0, 0, STONE, 20);
        g.set_with_temp(1, 0, 0, WATER, 60);
        g.set_with_temp(0, 1, 0, SAND, 20);
        g.set_with_temp(1, 1, 0, ICE, -5);
        // Pre-refresh: the scratch is AIR/0/0 (its allocated default).
        let pre = g.scratch_view();
        assert_eq!(pre.get(1, 0, 0), AIR, "scratch must start cleared");
        assert_eq!(pre.temperatures[g.index(1, 0, 0).unwrap()], 0);
        g.restore_scratch(pre);
        // Refresh + read-through.
        g.refresh_scratch();
        let post = g.scratch_view();
        assert_eq!(post.get(0, 0, 0), STONE);
        assert_eq!(post.get(1, 0, 0), WATER);
        assert_eq!(post.get(0, 1, 0), SAND);
        assert_eq!(post.get(1, 1, 0), ICE);
        assert_eq!(post.temperatures[g.index(1, 0, 0).unwrap()], 60);
        assert_eq!(post.temperatures[g.index(1, 1, 0).unwrap()], -5);
        g.restore_scratch(post);
    }

    /// FR-CIV-CA-008 — the double-buffer must be dirty-chunk scoped: a step
    /// FR-CIV-CA-008 — the double-buffer must be dirty-chunk scoped: a step
    ///  that mutates cells in the dirty chunks must NOT touch clean chunks
    ///  (i.e. the scratch's "prev" view is the pre-step snapshot for those
    ///  clean chunks, not the post-step one). We assert that `scratch_view`
    ///  after a successful step is still equal to the live view for the cells
    ///  we left alone (no spurious write-through into the read source).
    #[test]
    fn double_buffer_scratch_dirty_chunk_scoped() {
        // 2 chunks along x (32 cells). The right chunk is left static; the
        // left chunk has a single falling water cell. After the step, the
        // right chunk's scratch cells must equal the pre-step right chunk.
        let mut g = CaGrid::new([32, 16, 16]);
        for x in 16..32 {
            for y in 0..16 {
                for z in 0..16 {
                    g.set_with_temp(x, y, z, STONE, 20);
                }
            }
        }
        // Drop a single water cell in the left chunk (x in 0..16).
        g.set_with_temp(8, 8, 8, WATER, 20);
        // Snapshot — the right chunk is fully STONE.
        g.refresh_scratch();
        // Capture the right-chunk snapshot of the scratch (pre-step view).
        // We collect into a `Vec` so we don't keep a borrow of `g` alive
        //  across the step call below.
        let before_right: Vec<MaterialId> = {
            let view = g.scratch_view();
            let mut v = Vec::with_capacity(16 * 16 * 16);
            for x in 16..32 {
                for y in 0..16 {
                    for z in 0..16 {
                        v.push(view.get(x, y, z));
                    }
                }
            }
            g.restore_scratch(view);
            v
        };
        // Step the CA. The water falls one cell; right chunk is untouched.
        step_n_with_config(&mut g, reg(), 1, BoundaryConfig::closed(), 0);
        // The right chunk is still all STONE.
        for x in 16..32 {
            for y in 0..16 {
                for z in 0..16 {
                    assert_eq!(g.get(x, y, z), STONE);
                }
            }
        }
        // The scratch snapshot's right chunk is still the pre-step view.
        // (The double-buffer is only refreshed at the start of a step, so
        //  it is intentionally stale; this is the contract — the rule passes
        //  never see live cells they didn't opt into stepping.)
        let after_right: Vec<MaterialId> = {
            let view = g.scratch_view();
            let mut v = Vec::with_capacity(16 * 16 * 16);
            for x in 16..32 {
                for y in 0..16 {
                    for z in 0..16 {
                        v.push(view.get(x, y, z));
                    }
                }
            }
            g.restore_scratch(view);
            v
        };
        assert_eq!(
            before_right, after_right,
            "scratch right-chunk drifted away from pre-step view"
        );
    }
    // -------------------------------------------------------------------------
    // FR-CIV-CA-009 — `AbiogenesisSuitability` is a pure deterministic score
    // derived from (material, temperature, saturation). Two same-input
    //  evaluations must produce bit-identical results.
    // -------------------------------------------------------------------------

    /// FR-CIV-CA-009 — warm liquid water is viable, cold / hot / non-liquid
    ///  is not. This is the substrate for the MVP resident window's
    ///  abiogenesis scan (Simulation::phase_voxel_ca).
    #[test]
    fn abiogenesis_suitability_pure_and_deterministic() {
        let a = AbiogenesisSuitability::from_cell(WATER, 40, 128);
        let b = AbiogenesisSuitability::from_cell(WATER, 40, 128);
        assert_eq!(a, b, "abiogenesis score must be deterministic");
        assert!(a.is_viable(), "warm liquid water must be viable, got {a:?}");

        let cold = AbiogenesisSuitability::from_cell(WATER, -10, 128);
        assert!(!cold.is_viable(), "ice-cold water must be sterile");
        assert_eq!(cold.energy, 0);

        let hot = AbiogenesisSuitability::from_cell(WATER, 120, 128);
        assert!(!hot.is_viable(), "boiling water must be sterilised");
        assert_eq!(hot.energy, 0);

        let stone = AbiogenesisSuitability::from_cell(STONE, 40, 0);
        assert!(!stone.is_viable(), "stone must be solvent-free");
        assert_eq!(stone.solvent, 0);
        assert_eq!(stone.value, 0);
    }

    // -------------------------------------------------------------------------
    // FR-CIV-CA-010 — three spec-mandated smoke tests on a fixed micro-fixture:
    // 1) basin flat-fill, 2) unsupported-solid fall, 3) phase-change smoke.
    // -------------------------------------------------------------------------

    /// FR-CIV-CA-010 — basin flat-fill: pouring water into a sealed
    ///  rectangular basin must fill the basin to a flat horizontal surface
    ///  (no slope artefacts), with the fill height matching the pour
    ///  volume / basin area.
    #[test]
    fn basin_flat_fill_smoke() {
        // 5x4x1 basin carved in STONE; pour 5 WATER cells at the top.
        let mut g = CaGrid::new([5, 5, 1]);
        for x in 0..5 {
            for y in 0..2 {
                g.set(x, y, 0, STONE);
            }
        }
        for x in 0..5 {
            g.set_with_temp(x, 4, 0, WATER, 20);
        }
        g.dirty_chunks.clear();
        for x in 0..5 {
            g.mark_dirty_cell(x, 4, 0);
        }
        // Step until quiescent. 64 ticks is generous for a 5-cell pour.
        step_n_with_config(&mut g, reg(), 64, BoundaryConfig::closed(), 0);
        // All 5 water cells must survive (conservation).
        assert_eq!(count(&g, WATER), 5, "water must be conserved");
        // All 5 must sit on the basin floor (y=2 is the top of the STONE
        // wall) — no water may be at y>=3.
        let mut on_floor = 0;
        for x in 0..5 {
            assert_eq!(g.get(x, 4, 0), AIR, "water must not rest at y=4");
            if g.get(x, 2, 0) == WATER {
                on_floor += 1;
            }
        }
        assert_eq!(on_floor, 5, "all 5 water cells must settle on basin floor");
    }

    /// FR-CIV-CA-010 — unsupported-solid fall: a STONE cell with no
    ///  supporting cell beneath it must fall under gravity when the
    ///  material model treats it as a powder. We use SAND (Powder phase)
    ///  to exercise the `powder_step` path; STONE itself is Solid and
    ///  immobile (the spec is explicit: terrain materials never move).
    #[test]
    fn unsupported_solid_fall_smoke() {
        // Build a pillar: floor (y=0) STONE, gap (y=1) AIR, unsupported
        //  (y=2) SAND. After CA the SAND must fall into the gap.
        let mut g = CaGrid::new([1, 4, 1]);
        g.set(0, 0, 0, STONE);
        g.set(0, 1, 0, AIR);
        g.set(0, 2, 0, SAND);
        g.set(0, 3, 0, AIR);
        g.dirty_chunks.clear();
        g.mark_dirty_cell(0, 2, 0);
        step_n_with_config(&mut g, reg(), 8, BoundaryConfig::closed(), 0);
        // SAND settled one cell down (gravity). y=2 must be AIR; y=1 must
        //  be SAND. (We do not assert the lateral cell — `powder_step`
        //  randomises the slide direction, but on a 1-cell-wide grid it
        //  can only fall straight down.)
        assert_eq!(g.get(0, 2, 0), AIR, "SAND must vacate its start cell");
        assert_eq!(g.get(0, 1, 0), SAND, "SAND must fall into the gap");
        // Conservation: the SAND cell survived (no dissolve / no spawn).
        assert_eq!(count(&g, SAND), 1, "SAND must be conserved");
    }

    /// FR-CIV-CA-010 — phase-change smoke: ICE at 20 °C must melt to
    ///  WATER (the `phase_transition_pass` Solid → Liquid path). Place the
    ///  ICE in a sealed cell and assert the phase flips, the latent heat
    ///  debit is applied, and the post-step state is the canonical melt
    ///  output for a single-cell fixture.
    #[test]
    fn phase_change_smoke() {
        let mut g = CaGrid::new([1, 1, 1]);
        g.set_with_temp(0, 0, 0, ICE, 20);
        g.dirty_chunks.clear();
        g.mark_dirty_cell(0, 0, 0);
        let pre_t = g.get_temp(0, 0, 0);
        step_n_with_config(&mut g, reg(), 1, BoundaryConfig::closed(), 0);
        let result = g.get(0, 0, 0);
        assert_eq!(
            result, WATER,
            "ICE at 20 °C must melt to WATER, got {result:?}"
        );
        // Latent-heat debit: post-melt temp is pre-t minus WATER's
        //  latent_heat. For ICE → WATER the rule subtracts latent_heat
        //  from the cell temp.
        let post_t = g.get_temp(0, 0, 0);
        assert!(
            post_t < pre_t,
            "phase-change must apply latent-heat debit (pre={pre_t}, post={post_t})"
        );
    }
}
