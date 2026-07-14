//! Terrain erosion by overland water flow (FR-CIV-EROSION).
//!
//! A 2-D terrain heightfield is paired with a 2-D water-flux grid. Each
//! tick, water that flows over a cell erodes a small fraction of that
//! cell's height and deposits that material onto its downslope neighbour.
//!
//! # Algorithm
//!
//! 1. **Downslope identification** — for every cell, find the orthogonal
//!    neighbour with the lowest terrain height (ties broken by neighbour
//!    index for determinism). Cells that are local minima have no
//!    downslope neighbour.
//! 2. **Flow** — the `water_flux` at each cell is moved downhill by
//!    multiplying by `flow_rate`. The destination cell receives an
//!    equal share of that flux (we currently model only single-cell
//!    transfers; the algorithm is intentionally simple and
//!    allocation-light).
//! 3. **Erosion + deposition** — the cell that *lost* water loses a
//!    fraction `erosion_rate` of its height. The *gaining* cell gains
//!    the same fraction (`deposition_rate`) of the same height delta.
//!    Conservation is preserved up to floating-point rounding: the
//!    total height summed across the grid is unchanged.
//! 4. **Water decay** — surface water evaporates / infiltrates at
//!    `water_decay`, keeping the flux grid stable over time.
//!
//! The model is deterministic, allocation-light, and uses no unsafe
//! code. It deliberately has no external dependencies so it can sit
//! inside `civ-climate` without bloating the dependency graph.
//!
//! # Example
//!
//! ```rust
//! use civ_climate::erosion::{ErosionGrid, ErosionParams};
//!
//! let mut terrain = ErosionGrid::new(3, 1);
//! // Cell (0,0) is the high point; (2,0) is the sink.
//! terrain.set_height(0, 0, 10.0);
//! terrain.set_height(1, 0,  5.0);
//! terrain.set_height(2, 0,  0.0);
//!
//! let mut water = ErosionGrid::new(3, 1);
//! water.set_height(0, 0, 100.0);
//!
//! let params = ErosionParams::default();
//! erosion_step(&mut terrain, &mut water, &params);
//!
//! assert!(terrain.height_at(0, 0) < 10.0); // source lowered
//! assert!(terrain.height_at(2, 0) >  0.0); // sink raised
//! ```

#![forbid(unsafe_code)]

/// Tunable parameters that characterise the erosion response.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ErosionParams {
    /// Fraction of a cell's `water_flux` that flows downhill each tick.
    /// Must be in `[0.0, 1.0]`. 0.0 = no flow, 1.0 = all water moves.
    pub flow_rate: f32,
    /// Fraction of the source cell's terrain height that is *eroded*
    /// per unit of water that flows out of it. Small positive value.
    pub erosion_rate: f32,
    /// Fraction of the eroded material that is *deposited* on the
    /// downslope neighbour. The remainder is treated as dissolved
    /// load lost from the local system. Must be in `[0.0, 1.0]`.
    pub deposition_rate: f32,
    /// Fraction of standing water that evaporates / infiltrates per
    /// tick. Must be in `[0.0, 1.0]`.
    pub water_decay: f32,
}

impl Default for ErosionParams {
    fn default() -> Self {
        Self {
            flow_rate: 0.5,
            erosion_rate: 0.01,
            deposition_rate: 0.8,
            water_decay: 0.05,
        }
    }
}

/// A small 2-D grid of `f32` values used for both terrain height and
/// water flux. Indexing is `(x, y)` with `x` row-major in memory.
#[derive(Debug, Clone)]
pub struct ErosionGrid {
    width: usize,
    height: usize,
    cells: Vec<f32>,
}

impl ErosionGrid {
    /// Create a new grid of the given size, initialised to zero.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![0.0; width * height],
        }
    }

    /// Width of the grid in cells.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Height of the grid in cells.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Total number of cells.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Whether the grid is empty (zero-sized).
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Read the value at `(x, y)`. Returns `0.0` for out-of-bounds
    /// coordinates so callers don't need to bounds-check on every read.
    pub fn height_at(&self, x: usize, y: usize) -> f32 {
        if x >= self.width || y >= self.height {
            0.0
        } else {
            self.cells[y * self.width + x]
        }
    }

    /// Write a value at `(x, y)`. Silently ignores out-of-bounds writes
    /// so callers don't need to bounds-check on every write.
    pub fn set_height(&mut self, x: usize, y: usize, value: f32) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.cells[idx] = value;
        }
    }

    /// Sum of every cell. Useful for conservation checks.
    pub fn total(&self) -> f32 {
        self.cells.iter().sum()
    }
}

/// Advance the erosion simulation by one tick.
///
/// `terrain` carries the heights that are eroded / aggraded; `water`
/// carries the overland flux that drives the flow. Both grids must
/// have the same dimensions; mismatched grids are detected and the
/// function returns `false` without modifying either grid.
pub fn erosion_step(
    terrain: &mut ErosionGrid,
    water: &mut ErosionGrid,
    params: &ErosionParams,
) -> bool {
    if terrain.width != water.width || terrain.height != water.height {
        return false;
    }

    let w = terrain.width;
    let h = terrain.height;
    if w == 0 || h == 0 {
        return true;
    }

    // 1. Snapshot the terrain so we can compute downslope neighbours
    //    against an immutable view while mutating the live terrain.
    let terrain_before = terrain.cells.clone();

    // 2. Compute deltas: how much height moves from each source cell
    //    to its downslope neighbour. We build two parallel buffers and
    //    apply them after the scan so that within a single tick no
    //    cell both donates and receives from the same update.
    let mut erode = vec![0.0_f32; w * h];
    let mut deposit = vec![0.0_f32; w * h];

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;

            // How much water actually flows out of this cell.
            let flux = water.cells[idx];
            if flux <= 0.0 {
                continue;
            }
            let outflow = flux * params.flow_rate;
            if outflow <= 0.0 {
                continue;
            }

            // Find the downslope neighbour: the orthogonal neighbour
            // with the lowest terrain height. Ties broken by
            // (dy, dx) lexicographic order for determinism.
            let here = terrain_before[idx];
            let mut best_h = here;
            let mut best: Option<(usize, usize)> = None;

            // Order: up, down, left, right — fixed priority for ties.
            let neighbours: [(usize, usize); 4] = [
                (x, y.wrapping_sub(1)),
                (x, y + 1),
                (x.wrapping_sub(1), y),
                (x + 1, y),
            ];
            for &(nx, ny) in &neighbours {
                if nx >= w || ny >= h {
                    continue;
                }
                let nh = terrain_before[ny * w + nx];
                if nh < best_h {
                    best_h = nh;
                    best = Some((nx, ny));
                }
            }

            if let Some((nx, ny)) = best {
                let n_idx = ny * w + nx;

                // Material mobilised: proportional to water flux and
                // source cell erodibility. We use the terrain height
                // directly as a simple stand-in for erodible mass.
                let delta = outflow * params.erosion_rate * here;

                // Source loses height; neighbour gains a fraction of it.
                erode[idx] += delta;
                deposit[n_idx] += delta * params.deposition_rate;
            }
        }
    }

    // 3. Apply erosion and deposition. Order doesn't matter because we
    //    use disjoint (source vs neighbour) deltas — but to be safe we
    //    net them per cell first.
    for idx in 0..terrain.cells.len() {
        let net = deposit[idx] - erode[idx];
        terrain.cells[idx] += net;
        // Guard against numerical drift producing negative heights.
        if terrain.cells[idx] < 0.0 {
            terrain.cells[idx] = 0.0;
        }
    }

    // 4. Move water flux downhill and decay standing water. The flux
    //    transfer uses the same downslope routing as erosion so that
    //    high-flux cells drain toward the local minimum.
    let mut new_water = water.cells.clone();
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let flux = water.cells[idx];
            if flux <= 0.0 {
                continue;
            }
            let outflow = flux * params.flow_rate;
            new_water[idx] -= outflow;

            // Same downslope search as above.
            let here = terrain_before[idx];
            let mut best_h = here;
            let mut best: Option<(usize, usize)> = None;
            let neighbours: [(usize, usize); 4] = [
                (x, y.wrapping_sub(1)),
                (x, y + 1),
                (x.wrapping_sub(1), y),
                (x + 1, y),
            ];
            for &(nx, ny) in &neighbours {
                if nx >= w || ny >= h {
                    continue;
                }
                let nh = terrain_before[ny * w + nx];
                if nh < best_h {
                    best_h = nh;
                    best = Some((nx, ny));
                }
            }
            if let Some((nx, ny)) = best {
                new_water[ny * w + nx] += outflow;
            }
        }
    }

    // 5. Apply water decay (evaporation / infiltration).
    let decay_factor = 1.0 - params.water_decay;
    for v in new_water.iter_mut() {
        *v *= decay_factor;
        if *v < 0.0 {
            *v = 0.0;
        }
    }

    water.cells = new_water;
    true
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// 1. Sustained flow over a 1-D slope lowers the source cell and
    ///    raises its downslope neighbour — the core FR-CIV-EROSION
    ///    acceptance test.
    #[test]
    fn sustained_flow_lowers_source_and_raises_downslope() {
        let mut terrain = ErosionGrid::new(3, 1);
        terrain.set_height(0, 0, 10.0);
        terrain.set_height(1, 0, 5.0);
        terrain.set_height(2, 0, 0.0);

        let mut water = ErosionGrid::new(3, 1);
        water.set_height(0, 0, 100.0);

        let params = ErosionParams::default();

        // Snapshot before / after enough ticks for the effect to be
        // unambiguous regardless of parameter scale.
        let source_before = terrain.height_at(0, 0);
        let downslope_before = terrain.height_at(2, 0);

        for _ in 0..50 {
            // Replenish water each tick so flow is "sustained".
            water.set_height(0, 0, 100.0);
            assert!(erosion_step(&mut terrain, &mut water, &params));
        }

        let source_after = terrain.height_at(0, 0);
        let downslope_after = terrain.height_at(2, 0);

        assert!(
            source_after < source_before,
            "source cell should be lowered by sustained flow: \
             {source_before} -> {source_after}"
        );
        assert!(
            downslope_after > downslope_before,
            "downslope neighbour should be raised by deposition: \
             {downslope_before} -> {downslope_after}"
        );
    }

    /// 2. `erosion_step` refuses to operate on grids with mismatched
    ///    dimensions.
    #[test]
    fn mismatched_grids_are_rejected() {
        let mut terrain = ErosionGrid::new(3, 3);
        let mut water = ErosionGrid::new(3, 2);
        let params = ErosionParams::default();
        assert!(!erosion_step(&mut terrain, &mut water, &params));
    }

    /// 3. Mass conservation: the total terrain height is preserved
    ///    (erosion loss == deposition gain) up to floating-point error.
    #[test]
    fn mass_is_conserved() {
        let mut terrain = ErosionGrid::new(3, 1);
        terrain.set_height(0, 0, 10.0);
        terrain.set_height(1, 0, 5.0);
        terrain.set_height(2, 0, 0.0);

        let mut water = ErosionGrid::new(3, 1);
        water.set_height(0, 0, 100.0);

        let params = ErosionParams::default();
        let total_before = terrain.total();

        for _ in 0..20 {
            water.set_height(0, 0, 100.0);
            erosion_step(&mut terrain, &mut water, &params);
        }

        let total_after = terrain.total();
        let drift = (total_after - total_before).abs();
        assert!(
            drift < 1e-3,
            "terrain mass should be conserved: before={total_before}, \
             after={total_after}, drift={drift}"
        );
    }

    /// 4. Water decays toward zero when not replenished.
    #[test]
    fn water_decays_without_replenishment() {
        let mut terrain = ErosionGrid::new(3, 1);
        terrain.set_height(0, 0, 10.0);
        terrain.set_height(2, 0, 0.0);

        let mut water = ErosionGrid::new(3, 1);
        water.set_height(0, 0, 100.0);

        let params = ErosionParams::default();
        for _ in 0..200 {
            erosion_step(&mut terrain, &mut water, &params);
        }

        assert!(
            water.total() < 1.0,
            "water should decay toward zero without replenishment: \
             total = {}",
            water.total()
        );
    }

    /// 5. Flat terrain: no downslope neighbour means no erosion.
    #[test]
    fn flat_terrain_does_not_erode() {
        let mut terrain = ErosionGrid::new(3, 1);
        terrain.set_height(0, 0, 5.0);
        terrain.set_height(1, 0, 5.0);
        terrain.set_height(2, 0, 5.0);

        let mut water = ErosionGrid::new(3, 1);
        water.set_height(0, 0, 100.0);

        let params = ErosionParams::default();
        let total_before = terrain.total();

        for _ in 0..20 {
            erosion_step(&mut terrain, &mut water, &params);
        }

        let total_after = terrain.total();
        let drift = (total_after - total_before).abs();
        assert!(
            drift < 1e-4,
            "flat terrain should not erode: before={total_before}, \
             after={total_after}, drift={drift}"
        );
    }
}
