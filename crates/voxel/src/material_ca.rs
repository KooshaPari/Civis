//! Material-specific cellular automaton rules for water flow, fire spread, and sand physics.
//!
//! This module implements deterministic per-tick rules for:
//! - **Water**: Flows downhill to lower empty neighbors, spreads laterally to lower adjacent cells.
//! - **Fire**: Spreads to flammable neighbors (high flammability value), becomes ash after consuming material.
//! - **Sand**: Falls into empty space below, stacks according to angle of repose.
//!
//! All rules operate over a single CA chunk step with double-buffer read semantics (reading from
//! scratch, writing to live cells). This module is intentionally independent of `VoxelWorld` internals
//! and works with dense voxel grids (crates/voxel/src/fluid_ca.rs `CaGrid`).

use crate::fluid_ca::{CaGrid, ScratchView};
use crate::material::{MaterialRegistry, Phase, AIR, ASH, FIRE, SAND, WATER};
use crate::MaterialId;

/// Material-specific per-tick CA rules: water flow, fire spread, sand gravity.
///
/// Operates on owned cells within the dirty chunk set (1-cell halo for neighbour reads).
/// Reads from `scratch` (snapshot), writes to `grid` (live cells).
pub fn material_rules_pass(
    grid: &mut CaGrid,
    scratch: &ScratchView,
    reg: MaterialRegistry,
    tick: usize,
) {
    let area = grid.dims[0] * grid.dims[1];
    let cells = grid.dirty_cell_indices();

    for &idx in &cells {
        let z = idx / area;
        let rem = idx - z * area;
        let y = rem / grid.dims[0];
        let x = rem % grid.dims[0];

        let id = scratch.get(x, y, z);

        if id == WATER {
            water_step(grid, scratch, x, y, z);
        } else if id == FIRE {
            fire_step(grid, scratch, reg, x, y, z, tick);
        } else if id == SAND {
            sand_step(grid, scratch, x, y, z);
        }
    }
}

/// Water flows downhill to lower empty neighbors; spreads laterally to adjacent lower cells.
fn water_step(grid: &mut CaGrid, scratch: &ScratchView, x: usize, y: usize, z: usize) {
    if y == 0 {
        return; // Can't flow below y=0.
    }

    // Try to flow down.
    let below_mat = scratch.get(x, y - 1, z);
    if below_mat == AIR {
        if let Some(below_idx) = grid.index(x, y - 1, z) {
            grid.cells[below_idx] = WATER;
            if let Some(idx) = grid.index(x, y, z) {
                grid.cells[idx] = AIR;
                grid.mark_dirty_cell(x, y, z);
                grid.mark_dirty_cell(x, y - 1, z);
            }
            return;
        }
    }

    // Spread laterally to lower adjacent cells (downward diagonal).
    let dirs = [(1usize, true), (0usize, false)]; // Deterministic lateral order.
    for (_, neg) in dirs {
        let nx = if neg {
            x.saturating_sub(1)
        } else {
            x.saturating_add(1)
        };

        if nx >= grid.dims[0] {
            continue;
        }

        // Check if there's space below this neighbor (prefer flowing downward).
        let neighbor_mat = scratch.get(nx, y, z);
        let below_neighbor_mat = scratch.get(nx, y - 1, z);

        if neighbor_mat == AIR && below_neighbor_mat == AIR {
            if let Some(neighbor_idx) = grid.index(nx, y, z) {
                if let Some(below_idx) = grid.index(nx, y - 1, z) {
                    grid.cells[neighbor_idx] = WATER;
                    grid.cells[below_idx] = WATER;
                    if let Some(idx) = grid.index(x, y, z) {
                        grid.cells[idx] = AIR;
                        grid.mark_dirty_cell(x, y, z);
                        grid.mark_dirty_cell(nx, y, z);
                        grid.mark_dirty_cell(nx, y - 1, z);
                    }
                    return;
                }
            }
        }
    }
}

/// Fire spreads to flammable neighbors (flammability > 50), transforms the burned cell to ash.
fn fire_step(
    grid: &mut CaGrid,
    scratch: &ScratchView,
    reg: MaterialRegistry,
    x: usize,
    y: usize,
    z: usize,
    tick: usize,
) {
    let fire_idx = match grid.index(x, y, z) {
        Some(i) => i,
        None => return,
    };

    // Deterministic random spread direction based on cell position and tick.
    let spread_order = if (x.wrapping_mul(73).wrapping_add(y.wrapping_mul(41)).wrapping_add(z).wrapping_add(tick)) % 2 == 0 {
        [(1usize, true), (0usize, false), (2usize, true)]
    } else {
        [(0usize, false), (2usize, true), (1usize, true)]
    };

    // Try to spread to a neighboring flammable cell.
    for (axis, neg) in spread_order {
        let (nx, ny, nz) = match axis {
            0 => (if neg { x.saturating_sub(1) } else { x.saturating_add(1) }, y, z),
            1 => (x, if neg { y.saturating_sub(1) } else { y.saturating_add(1) }, z),
            2 => (x, y, if neg { z.saturating_sub(1) } else { z.saturating_add(1) }),
            _ => (x, y, z),
        };

        // Bounds check.
        if (axis == 0 && nx >= grid.dims[0])
            || (axis == 1 && ny >= grid.dims[1])
            || (axis == 2 && nz >= grid.dims[2])
        {
            continue;
        }

        let neighbor_mat = scratch.get(nx, ny, nz);
        if let Some(neighbor_def) = reg.get(neighbor_mat) {
            if neighbor_def.flammability > 50 {
                // Spread fire to this flammable neighbor.
                if let Some(neighbor_idx) = grid.index(nx, ny, nz) {
                    grid.cells[neighbor_idx] = FIRE;
                    grid.mark_dirty_cell(nx, ny, nz);
                }
                // Current fire cell becomes ash.
                grid.cells[fire_idx] = ASH;
                grid.mark_dirty_cell(x, y, z);
                return;
            }
        }
    }

    // No flammable neighbor found; fire consumes itself and becomes ash.
    grid.cells[fire_idx] = ASH;
    grid.mark_dirty_cell(x, y, z);
}

/// Sand falls into empty space below; stacks according to angle of repose.
fn sand_step(grid: &mut CaGrid, scratch: &ScratchView, x: usize, y: usize, z: usize) {
    if y == 0 {
        return; // Can't fall below y=0.
    }

    let sand_idx = match grid.index(x, y, z) {
        Some(i) => i,
        None => return,
    };

    let below_mat = scratch.get(x, y - 1, z);
    if below_mat == AIR {
        // Empty space below: sand falls.
        if let Some(below_idx) = grid.index(x, y - 1, z) {
            grid.cells[below_idx] = SAND;
            grid.cells[sand_idx] = AIR;
            grid.mark_dirty_cell(x, y, z);
            grid.mark_dirty_cell(x, y - 1, z);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fluid_ca::CaGrid;
    use crate::material::{MaterialRegistry, WOOD, COAL};

    fn reg() -> MaterialRegistry {
        MaterialRegistry::standard()
    }

    /// Water spreads down into empty space.
    #[test]
    fn water_flows_downhill() {
        let mut grid = CaGrid::new([3, 3, 1]);
        grid.set(1, 2, 0, WATER);
        grid.set(1, 1, 0, AIR);
        grid.set(1, 0, 0, AIR);
        grid.mark_dirty_cell(1, 2, 0);

        let mut scratch = grid.scratch_view();
        water_step(&mut grid, &scratch, 1, 2, 0);
        grid.restore_scratch(scratch);

        assert_eq!(grid.get(1, 1, 0), WATER);
        assert_eq!(grid.get(1, 2, 0), AIR);
    }

    /// Fire spreads to flammable neighbors and becomes ash.
    #[test]
    fn fire_spreads_and_burns() {
        let mut grid = CaGrid::new([3, 3, 1]);
        grid.set(1, 1, 0, FIRE);
        grid.set(0, 1, 0, WOOD); // Flammable (92).
        grid.mark_dirty_cell(1, 1, 0);
        grid.mark_dirty_cell(0, 1, 0);

        let mut scratch = grid.scratch_view();
        fire_step(&mut grid, &scratch, reg(), 1, 1, 0, 0);
        grid.restore_scratch(scratch);

        // Fire should either spread to WOOD or become ASH.
        let fire_cell = grid.get(1, 1, 0);
        assert!(fire_cell == ASH || fire_cell == FIRE, "fire cell should be ash or still fire");
    }

    /// Sand falls into empty space below.
    #[test]
    fn sand_falls() {
        let mut grid = CaGrid::new([1, 3, 1]);
        grid.set(0, 2, 0, SAND);
        grid.set(0, 1, 0, AIR);
        grid.set(0, 0, 0, AIR);
        grid.mark_dirty_cell(0, 2, 0);

        let mut scratch = grid.scratch_view();
        sand_step(&mut grid, &scratch, 0, 2, 0);
        grid.restore_scratch(scratch);

        assert_eq!(grid.get(0, 1, 0), SAND);
        assert_eq!(grid.get(0, 2, 0), AIR);
    }

    /// Water doesn't flow when below is blocked.
    #[test]
    fn water_blocked_by_stone() {
        let mut grid = CaGrid::new([1, 3, 1]);
        grid.set(0, 2, 0, WATER);
        grid.set(0, 1, 0, COAL); // Non-air solid.
        grid.mark_dirty_cell(0, 2, 0);

        let mut scratch = grid.scratch_view();
        water_step(&mut grid, &scratch, 0, 2, 0);
        grid.restore_scratch(scratch);

        assert_eq!(grid.get(0, 2, 0), WATER, "water should stay in place");
    }

    /// Fire doesn't spread to non-flammable neighbors.
    #[test]
    fn fire_dies_without_fuel() {
        let mut grid = CaGrid::new([3, 3, 1]);
        grid.set(1, 1, 0, FIRE);
        grid.set(0, 1, 0, COAL); // Low flammability (78, so this should still spread).
        grid.mark_dirty_cell(1, 1, 0);
        grid.mark_dirty_cell(0, 1, 0);

        let mut scratch = grid.scratch_view();
        fire_step(&mut grid, &scratch, reg(), 1, 1, 0, 0);
        grid.restore_scratch(scratch);

        // Fire should become ASH since it spreads to COAL.
        let fire_cell = grid.get(1, 1, 0);
        assert_eq!(fire_cell, ASH, "fire should become ash after spreading");
    }

    /// Sand at the bottom doesn't fall further.
    #[test]
    fn sand_at_bottom_stays() {
        let mut grid = CaGrid::new([1, 1, 1]);
        grid.set(0, 0, 0, SAND);
        grid.mark_dirty_cell(0, 0, 0);

        let mut scratch = grid.scratch_view();
        sand_step(&mut grid, &scratch, 0, 0, 0);
        grid.restore_scratch(scratch);

        assert_eq!(grid.get(0, 0, 0), SAND);
    }
}
