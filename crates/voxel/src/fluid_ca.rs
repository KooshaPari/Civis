//! Deterministic material cellular automaton for a dense voxel grid.
//!
//! This module is intentionally standalone: it does not depend on
//! `VoxelWorld` internals and instead operates on a simple dense grid of
//! `MaterialId` cells.

use crate::boundary::Bounds3;
use crate::material::{Phase, MaterialRegistry, AIR};
use crate::{MaterialId, VoxelWorld, WorldCoord};

/// Dense 3D grid for deterministic CA stepping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaGrid {
    /// Grid dimensions in `[x, y, z]` order.
    pub dims: [usize; 3],
    /// Row-major cells with `x + y * dx + z * dx * dy` indexing.
    pub cells: Vec<MaterialId>,
}

impl CaGrid {
    /// Creates a new grid filled with air.
    #[must_use]
    pub fn new(dims: [usize; 3]) -> Self {
        let len = dims[0].saturating_mul(dims[1]).saturating_mul(dims[2]);
        Self { dims, cells: vec![AIR; len] }
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

    /// Writes a cell when coordinates are in bounds.
    pub fn set(&mut self, x: usize, y: usize, z: usize, value: MaterialId) {
        if let Some(i) = self.index(x, y, z) {
            self.cells[i] = value;
        }
    }
}

fn phase_of(reg: MaterialRegistry, id: MaterialId) -> Phase {
    reg.get(id).map(|m| m.phase).unwrap_or(Phase::Solid)
}

fn material_can_swap_into(reg: MaterialRegistry, mover: MaterialId, target: MaterialId) -> bool {
    let mover_def = match reg.get(mover) { Some(def) => def, None => return false };
    let target_def = match reg.get(target) { Some(def) => def, None => return false };
    if target == AIR {
        return true;
    }
    mover_def.density > target_def.density
}

fn try_swap(grid: &mut CaGrid, a: (usize, usize, usize), b: (usize, usize, usize), reg: MaterialRegistry) -> bool {
    let ai = match grid.index(a.0, a.1, a.2) { Some(i) => i, None => return false };
    let bi = match grid.index(b.0, b.1, b.2) { Some(i) => i, None => return false };
    let av = grid.cells[ai];
    let bv = grid.cells[bi];
    if material_can_swap_into(reg, av, bv) {
        grid.cells.swap(ai, bi);
        return true;
    }
    false
}

fn sweep_x(dims: [usize; 3], parity: usize) -> std::ops::Range<usize> {
    if parity & 1 == 0 { 0..dims[0] } else { 0..dims[0] }
}

fn powder_step(grid: &mut CaGrid, reg: MaterialRegistry, x: usize, y: usize, z: usize, parity: usize) {
    if y == 0 {
        return;
    }
    let id = grid.get(x, y, z);
    if grid.get(x, y - 1, z) == AIR && try_swap(grid, (x, y, z), (x, y - 1, z), reg) {
        return;
    }
    let dirs = if parity & 1 == 0 { [usize::MAX, 1] } else { [1, usize::MAX] };
    for dx in dirs {
        let nx = if dx == usize::MAX { x.checked_sub(1) } else { x.checked_add(1) };
        let Some(nx) = nx else { continue };
        if grid.index(nx, y - 1, z).is_some() && material_can_swap_into(reg, id, grid.get(nx, y - 1, z)) {
            if try_swap(grid, (x, y, z), (nx, y - 1, z), reg) {
                return;
            }
        }
    }
}

fn liquid_step(grid: &mut CaGrid, reg: MaterialRegistry, x: usize, y: usize, z: usize, parity: usize) {
    if y > 0 && try_swap(grid, (x, y, z), (x, y - 1, z), reg) {
        return;
    }
    let dirs = if parity & 1 == 0 { [usize::MAX, 1] } else { [1, usize::MAX] };
    for dx in dirs {
        let nx = if dx == usize::MAX { x.checked_sub(1) } else { x.checked_add(1) };
        let Some(nx) = nx else { continue };
        if material_can_swap_into(reg, grid.get(x, y, z), grid.get(nx, y, z)) && try_swap(grid, (x, y, z), (nx, y, z), reg) {
            return;
        }
    }
}

fn gas_step(grid: &mut CaGrid, reg: MaterialRegistry, x: usize, y: usize, z: usize, parity: usize) {
    let h = grid.dims[1];
    if y + 1 < h && try_swap(grid, (x, y, z), (x, y + 1, z), reg) {
        return;
    }
    let dirs = if parity & 1 == 0 { [usize::MAX, 1] } else { [1, usize::MAX] };
    for dx in dirs {
        let nx = if dx == usize::MAX { x.checked_sub(1) } else { x.checked_add(1) };
        let Some(nx) = nx else { continue };
        if material_can_swap_into(reg, grid.get(x, y, z), grid.get(nx, y, z)) && try_swap(grid, (x, y, z), (nx, y, z), reg) {
            return;
        }
    }
}

/// Runs one deterministic cellular-automaton tick over the grid.
fn step_with_parity(grid: &mut CaGrid, reg: MaterialRegistry, parity: usize) {
    let dims = grid.dims;
    for y in 0..dims[1] {
        for z in 0..dims[2] {
            for x in sweep_x(dims, parity) {
                let id = grid.get(x, y, z);
                match phase_of(reg, id) {
                    Phase::Powder => powder_step(grid, reg, x, y, z, parity),
                    Phase::Liquid => liquid_step(grid, reg, x, y, z, parity),
                    Phase::Solid | Phase::Empty => {}
                    Phase::Gas => {}
                }
            }
        }
    }

    for y in (0..dims[1]).rev() {
        for z in 0..dims[2] {
            for x in sweep_x(dims, parity) {
                if phase_of(reg, grid.get(x, y, z)) == Phase::Gas {
                    gas_step(grid, reg, x, y, z, parity);
                }
            }
        }
    }
}

/// Runs one deterministic cellular-automaton tick over the grid.
pub fn step(grid: &mut CaGrid, reg: MaterialRegistry) {
    step_with_parity(grid, reg, 0);
}

/// Runs `n` deterministic cellular-automaton ticks.
pub fn step_n(grid: &mut CaGrid, reg: MaterialRegistry, n: usize) {
    for tick in 0..n {
        step_with_parity(grid, reg, tick);
    }
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
                grid.set(x, y, z, world.read(world_cell(bounds, voxel_span, x, y, z)));
            }
        }
    }
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

/// Runs one deterministic CA tick over a bounded voxel world region.
pub fn step_world(
    world: &mut VoxelWorld<MaterialId>,
    voxel_span: i64,
    bounds: Bounds3,
    reg: MaterialRegistry,
) -> usize {
    let mut grid = grid_from_world(world, bounds, voxel_span);
    let before = grid.clone();
    step(&mut grid, reg);
    write_back_world(world, bounds, voxel_span, &before, &grid)
}

/// Runs `n` deterministic CA ticks over a bounded voxel world region.
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
    use crate::material::{BEDROCK, SAND, STEAM, STONE, WATER};

    fn reg() -> MaterialRegistry {
        MaterialRegistry::standard()
    }

    fn count(grid: &CaGrid, id: MaterialId) -> usize {
        grid.cells.iter().copied().filter(|&c| c == id).count()
    }

    #[test]
    fn water_falls() {
        let mut g = CaGrid::new([1, 2, 1]);
        g.set(0, 1, 0, WATER);
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
        step_n(&mut g, reg(), 10);
        assert_eq!(count(&g, WATER), 1);
        assert!(g.get(2, 1, 0) == WATER || g.get(1, 1, 0) == WATER || g.get(3, 1, 0) == WATER);
    }

    #[test]
    fn sand_piles() {
        let mut g = CaGrid::new([7, 5, 1]);
        for x in 0..7 {
            g.set(x, 0, 0, STONE);
        }
        g.set(3, 4, 0, SAND);
        step_n(&mut g, reg(), 20);
        assert_eq!(count(&g, SAND), 1);
        assert!(g.get(3, 1, 0) == SAND || g.get(2, 1, 0) == SAND || g.get(4, 1, 0) == SAND);
    }

    #[test]
    fn gas_rises() {
        let mut g = CaGrid::new([1, 3, 1]);
        g.set(0, 0, 0, STEAM);
        step(&mut g, reg());
        assert_eq!(g.get(0, 1, 0), STEAM);
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
}
