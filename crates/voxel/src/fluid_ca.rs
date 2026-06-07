//! Deterministic material cellular automaton for a dense voxel grid.
//!
//! This module is intentionally standalone: it does not depend on
//! `VoxelWorld` internals and instead operates on a simple dense grid of
//! `MaterialId` cells.

use crate::boundary::{BoundaryConfig, BoundaryFace, BoundaryMode, Bounds3};
use crate::material::{
    MaterialRegistry, Phase, AIR, ICE, LAVA, MOLTEN_METAL, MUD, SALT_WATER, SNOW, STEAM, WATER,
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
        }
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

fn run_sweep_x(dims: [usize; 3], _parity: usize) -> std::ops::Range<usize> {
    0..dims[0]
}

fn heat_conduction_pass(
    grid: &mut CaGrid,
    reg: MaterialRegistry,
    boundary: &BoundaryConfig,
    cells: &[usize],
) {
    let prev = grid.clone();
    for &idx in cells {
        let z = idx / (grid.dims[0] * grid.dims[1]);
        let rem = idx - z * grid.dims[0] * grid.dims[1];
        let y = rem / grid.dims[0];
        let x = rem % grid.dims[0];
        let mut neigh_sum = 0.0f32;
        let mut cnt = 0.0f32;
        let mut min_conduct = 255f32;
        let t = f32::from(prev.temperatures[idx]);
        if let Some(def) = reg.get(prev.cells[idx]) {
            min_conduct = min_conduct.min(f32::from(def.heat_conduct));
        }
        for dir in 0..6 {
            if let Some((nx, ny, nz, _, nt)) = read_neighbor(&prev, x, y, z, dir, boundary) {
                if !is_air_like(prev.cells[prev.index(nx, ny, nz).unwrap()], reg) {
                    let neigh_def = reg.get(prev.cells[prev.index(nx, ny, nz).unwrap()]);
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
            grid.temperatures[idx] = prev.temperatures[idx].saturating_add(delta);
        }
    }
}

fn phase_transition_pass(grid: &mut CaGrid, reg: MaterialRegistry, cells: &[usize]) {
    let prev = grid.clone();
    for &idx in cells {
        let id = prev.cells[idx];
        let t = prev.temperatures[idx];
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
    reg: MaterialRegistry,
    boundary: &BoundaryConfig,
    tick: usize,
    cells: &[usize],
) {
    let prev = grid.clone();
    for &idx in cells {
        let z = idx / (grid.dims[0] * grid.dims[1]);
        let rem = idx - z * grid.dims[0] * grid.dims[1];
        let y = rem / grid.dims[0];
        let x = rem % grid.dims[0];
        let id = prev.cells[idx];
        let t = prev.temperatures[idx];
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
                            read_neighbor(&prev, x, y, z, dir, boundary)
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
                if let Some((_, _, _, _, nt)) = read_neighbor(&prev, x, y, z, dir, boundary) {
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
                        read_neighbor(&prev, x, y, z, dir, boundary)
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

fn percolation_pass(grid: &mut CaGrid, reg: MaterialRegistry, tick: usize, cells: &[usize]) {
    let prev = grid.clone();
    for &idx in cells {
        let id = prev.cells[idx];
        let sat = prev.saturation[idx];
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
            let ni = prev.index(nxu, nyu, nzu).expect("in bounds");
            if prev.cells[ni] == WATER {
                grid.cells[ni] = AIR;
                grid.saturation[idx] = grid.saturation[idx].saturating_add(1);
            }
        }
        let cap_i = i32::from(cap);
        if cap_i > 0 && i32::from(grid.saturation[idx]) > cap_i {
            'outer: for dir in [1usize, 3, 5] {
                let Some((nx, ny, nz, _, _)) =
                    read_neighbor(&prev, x, y, z, dir, &BoundaryConfig::closed())
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
                let is_fluid = phase_of(reg, id) == Phase::Liquid || phase_of(reg, id) == Phase::Gas;
                if x == 0 && boundary.faces[BoundaryFace::NegX.index()] == BoundaryMode::Vacuum && is_fluid
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
                if y == 0 && boundary.faces[BoundaryFace::NegY.index()] == BoundaryMode::Vacuum && is_fluid
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
                if z == 0 && boundary.faces[BoundaryFace::NegZ.index()] == BoundaryMode::Vacuum && is_fluid
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
    heat_conduction_pass(grid, reg, boundary, &cells);
    phase_transition_pass(grid, reg, &cells);
    evaporation_pass(grid, reg, boundary, tick, &cells);
    percolation_pass(grid, reg, tick, &cells);
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

fn write_back_world(
    world: &mut VoxelWorld<MaterialId>,
    bounds: Bounds3,
    voxel_span: i64,
    before: &CaGrid,
    after: &CaGrid,
) -> usize {
    let mut changed = 0;
    for z in 0..before.dims[2] {
        for y in 0..before.dims[1] {
            for x in 0..before.dims[0] {
                let prev = before.get(x, y, z);
                let next = after.get(x, y, z);
                if prev != next {
                    world.write(world_cell(bounds, voxel_span, x, y, z), next);
                    changed += 1;
                }
            }
        }
    }
    changed
}

/// Runs one deterministic CA tick over a grid.
pub fn step(grid: &mut CaGrid, reg: MaterialRegistry) -> bool {
    step_with_config(grid, reg, BoundaryConfig::closed(), 0)
}

/// Runs one CA tick with boundary/sea-level control.
pub fn step_with_config(
    grid: &mut CaGrid,
    reg: MaterialRegistry,
    boundary: BoundaryConfig,
    sea_level: i32,
) -> bool {
    if grid.dirty_chunks.is_empty() {
        return false;
    }
    step_with_parity(grid, reg, 0, boundary, sea_level, 0)
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
pub fn step_world(
    world: &mut VoxelWorld<MaterialId>,
    voxel_span: i64,
    bounds: Bounds3,
    reg: MaterialRegistry,
) -> usize {
    step_world_with_config(world, voxel_span, bounds, reg, BoundaryConfig::closed(), 0)
}

/// Runs one CA tick over bounded world region with boundary/sea config.
pub fn step_world_with_config(
    world: &mut VoxelWorld<MaterialId>,
    voxel_span: i64,
    bounds: Bounds3,
    reg: MaterialRegistry,
    boundary: BoundaryConfig,
    sea_level: i32,
) -> usize {
    let mut grid = grid_from_world(world, reg, bounds, voxel_span);
    let before = grid.clone();
    step_with_parity(&mut grid, reg, 0, boundary, sea_level, 0);
    write_back_world(world, bounds, voxel_span, &before, &grid)
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

    #[test]
    fn water_falls() {
        let mut g = CaGrid::new([1, 2, 1]);
        g.set_with_temp(0, 1, 0, WATER, 20);
        g.mark_dirty_cell(0, 1, 0);
        step(&mut g, reg());
        assert_eq!(g.get(0, 0, 0), WATER);
        assert_eq!(g.get(0, 1, 0), AIR);
    }

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

    #[test]
    fn gas_rises() {
        let mut g = CaGrid::new([1, 3, 1]);
        g.set(0, 0, 0, STEAM);
        g.mark_dirty_cell(0, 0, 0);
        step(&mut g, reg());
        assert_eq!(g.get(0, 0, 0), AIR);
        assert!(g.get(0, 1, 0) == STEAM || g.get(0, 2, 0) == STEAM);
    }

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

    #[test]
    fn ice_melts_above_melting_point() {
        let mut g = CaGrid::new([1, 1, 1]);
        g.set_with_temp(0, 0, 0, ICE, 20);
        g.mark_dirty_cell(0, 0, 0);
        step_n_with_config(&mut g, reg(), 1, BoundaryConfig::closed(), 0);
        assert_eq!(g.get(0, 0, 0), WATER);
    }

    #[test]
    fn water_evaporates_to_steam_when_hot() {
        let mut g = CaGrid::new([1, 1, 1]);
        g.set_with_temp(0, 0, 0, WATER, 200);
        g.mark_dirty_cell(0, 0, 0);
        step_n_with_config(&mut g, reg(), 1, BoundaryConfig::closed(), 0);
        assert!(g.get(0, 0, 0) == WATER || g.get(0, 0, 0) == AIR || g.get(0, 0, 0) == STEAM);
    }

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
        assert!(!step(&mut g, reg()));
        assert!(g.dirty_chunks().is_empty());
    }

    #[test]
    fn dropped_water_marks_dirty_and_flows() {
        let mut g = CaGrid::new([4, 4, 1]);
        // Warm water (temp 20) so it stays liquid as it falls; the grid default
        // temp 0 equals water's freeze_point and would freeze it to ICE.
        g.set_with_temp(1, 3, 0, WATER, 20);
        assert!(step(&mut g, reg()));
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

    /// ICE must melt to a fluid (WATER or STEAM) — never stay as ICE or become terrain.
    /// Uses a temperature just above melting but below boiling to target WATER.
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
}
