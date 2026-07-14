//! Disaster-spread model.
//!
//! A hazard (`Fire` or `Flood`) is represented on a discrete 2-D grid. Each
//! cell carries an `intensity` in `[0.0, 1.0]`. On every `step`:
//!
//! 1. **Propagation** — burning cells transfer a fraction of their intensity
//!    to each of their 4-neighbours (up/down/left/right). The fraction is
//!    scaled by the hazard's `spread_rate` and damped by the source cell's
//!    remaining intensity, so a hot cell radiates more than a dying one.
//! 2. **Decay** — every active cell's intensity is multiplied by
//!    `1.0 - decay_rate`, so hazards eventually fade to zero even without
//!    fresh fuel. Intensities below `ignition_threshold` are clamped to
//!    zero so the grid returns to a clean state.
//!
//! The model is deterministic, allocation-light, and uses no unsafe code.
//! It deliberately has no external dependencies so it can sit inside
//! `civ-climate` without bloating the dependency graph.
//!
//! # Example
//!
//! ```rust
//! use civ_climate::disaster_spread::{DisasterGrid, DisasterParams, HazardKind};
//!
//! let mut grid = DisasterGrid::new(5, 5);
//! let params = DisasterParams::default();
//!
//! // Seed a fire at the centre of the grid.
//! grid.ignite(2, 2, 1.0, HazardKind::Fire);
//! grid.step(&params);
//! assert!(grid.intensity_at(2, 2) > 0.0);
//! assert!(grid.intensity_at(2, 3) > 0.0);
//! ```

#![forbid(unsafe_code)]

/// The kind of hazard tracked on the disaster grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HazardKind {
    /// Wildfire / structural fire — spreads quickly, decays moderately.
    Fire,
    /// Flooding — spreads slowly, persists longer.
    Flood,
}

impl Default for HazardKind {
    fn default() -> Self {
        // `HazardCell::default()` is inactive, so its kind is never observed
        // by propagation. Keep Fire as the canonical baseline hazard for
        // callers that initialize a cell before igniting it.
        Self::Fire
    }
}

/// Per-cell hazard state. A cell is "active" when `intensity > 0.0`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct HazardCell {
    /// Current intensity in `[0.0, 1.0]`.
    pub intensity: f32,
    /// Hazard kind; meaningless when `intensity == 0.0`.
    pub kind: HazardKind,
}

/// Tunable parameters for the disaster-spread model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisasterParams {
    /// Per-tick spread factor for `Fire` (fraction of the source cell's
    /// intensity radiated to each orthogonal neighbour). Must be in
    /// `[0.0, 1.0]`; values > ~0.25 produce runaway growth when combined
    /// with the default decay rate.
    pub fire_spread_rate: f32,
    /// Per-tick spread factor for `Flood`.
    pub flood_spread_rate: f32,
    /// Per-tick decay factor applied to *every* cell's intensity.
    /// `0.0` = no decay; `1.0` = total extinction every tick.
    pub decay_rate: f32,
    /// Intensities strictly below this value are clamped to zero at the
    /// end of each step. Prevents floating-point "embers" from lingering.
    pub ignition_threshold: f32,
}

impl Default for DisasterParams {
    fn default() -> Self {
        Self {
            // Fire radiates a moderate fraction of its intensity to each
            // neighbour; default keeps the test (3x3 → 1.0 ignition) stable.
            fire_spread_rate: 0.2,
            // Flood spreads slower than fire.
            flood_spread_rate: 0.1,
            // 20% decay per tick — within ~20 ticks a cell without
            // reinforcement drops below the default ignition threshold.
            decay_rate: 0.2,
            // Anything below 1e-3 intensity is treated as extinguished.
            ignition_threshold: 1e-3,
        }
    }
}

/// A fixed-size 2-D grid of hazard cells.
///
/// Stored as a flat `Vec<HazardCell>` in row-major order for cache
/// locality and zero-allocation stepping.
#[derive(Debug, Clone)]
pub struct DisasterGrid {
    width: usize,
    height: usize,
    cells: Vec<HazardCell>,
}

impl DisasterGrid {
    /// Create a new grid of the given `width x height`, fully extinguished.
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![HazardCell::default(); width * height];
        Self {
            width,
            height,
            cells,
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

    /// Read the intensity at `(x, y)`. Returns `0.0` for out-of-bounds
    /// coordinates (treats them as permanently non-flammable walls).
    pub fn intensity_at(&self, x: usize, y: usize) -> f32 {
        if let Some(idx) = self.index(x, y) {
            self.cells[idx].intensity
        } else {
            0.0
        }
    }

    /// Ignite (or flood) cell `(x, y)` with the given starting `intensity`.
    /// `intensity` is clamped to `[0.0, 1.0]`. Out-of-bounds ignition is a
    /// silent no-op so callers can blindly spark at the edge of a map.
    pub fn ignite(&mut self, x: usize, y: usize, intensity: f32, kind: HazardKind) {
        if let Some(idx) = self.index(x, y) {
            let clamped = intensity.clamp(0.0, 1.0);
            // Use the larger of existing and new intensity — re-igniting a
            // smouldering cell does not reduce its intensity.
            let cell = &mut self.cells[idx];
            if clamped > cell.intensity {
                cell.intensity = clamped;
                cell.kind = kind;
            }
        }
    }

    /// Advance the simulation by one tick: propagate intensity to
    /// orthogonal neighbours, then apply decay and threshold-clamp.
    pub fn step(&mut self, params: &DisasterParams) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        // --- Pass 1: compute the new grid from the current one. -----------
        // We must not write into `self.cells` while reading other cells or
        // the propagation becomes order-dependent (a left-to-right sweep
        // would bias rightward). Build the delta into a scratch buffer.
        let mut next: Vec<HazardCell> = vec![HazardCell::default(); self.cells.len()];

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let cell = self.cells[idx];
                if cell.intensity <= 0.0 {
                    continue;
                }

                // Keep the source cell's own intensity — it decays in pass 2.
                if next[idx].intensity < cell.intensity {
                    next[idx] = cell;
                }

                let spread_rate = match cell.kind {
                    HazardKind::Fire => params.fire_spread_rate,
                    HazardKind::Flood => params.flood_spread_rate,
                };

                // Each orthogonal neighbour receives `spread_rate * intensity`
                // but only if the neighbour is unlit (or burning cooler) so
                // two adjacent cells do not double-feed.
                let radiation = (spread_rate * cell.intensity).max(0.0);
                if radiation > 0.0 {
                    let neighbours: [(usize, usize); 4] = [
                        (x.wrapping_sub(1), y),
                        (x + 1, y),
                        (x, y.wrapping_sub(1)),
                        (x, y + 1),
                    ];
                    for (nx, ny) in neighbours {
                        if let Some(nidx) = self.index(nx, ny) {
                            let neighbour = &mut next[nidx];
                            // Only transfer if the neighbour is currently
                            // cooler than the radiation being donated.
                            if neighbour.intensity < radiation {
                                neighbour.intensity = radiation;
                                neighbour.kind = cell.kind;
                            }
                        }
                    }
                }
            }
        }

        // --- Pass 2: decay every active cell and clamp to threshold. ------
        let decay = (1.0 - params.decay_rate).clamp(0.0, 1.0);
        let threshold = params.ignition_threshold.max(0.0);
        for cell in next.iter_mut() {
            if cell.intensity > 0.0 {
                cell.intensity *= decay;
                if cell.intensity < threshold {
                    cell.intensity = 0.0;
                }
            }
        }

        self.cells = next;
    }

    /// Total intensity summed across all cells — handy for assertions and
    /// monitoring "how much disaster is on the map right now".
    pub fn total_intensity(&self) -> f32 {
        self.cells.iter().map(|c| c.intensity).sum()
    }

    /// True when every cell is below the ignition threshold.
    pub fn is_quiescent(&self, params: &DisasterParams) -> bool {
        let threshold = params.ignition_threshold.max(0.0);
        self.cells.iter().all(|c| c.intensity < threshold)
    }

    /// Convert `(x, y)` to a flat index, or `None` if out of bounds.
    #[inline]
    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-6;

    fn approx_eq(a: f32, b: f32) {
        assert!(
            (a - b).abs() < EPS,
            "expected {b}, got {a} (diff = {})",
            (a - b).abs()
        );
    }

    /// A seeded fire must spread to its 4-orthogonal neighbours within a
    /// single step, then eventually decay back to a quiescent (zero-intensity)
    /// grid.
    #[test]
    fn fire_spreads_then_decays_to_zero() {
        let mut grid = DisasterGrid::new(5, 5);
        let params = DisasterParams::default();

        // Seed a fully-lit fire at the centre of the grid.
        grid.ignite(2, 2, 1.0, HazardKind::Fire);
        approx_eq(grid.intensity_at(2, 2), 1.0);

        // Step 1: the fire must reach all four orthogonal neighbours.
        grid.step(&params);
        let n = grid.intensity_at(2, 3); // down
        let s = grid.intensity_at(2, 1); // up
        let e = grid.intensity_at(3, 2); // right
        let w = grid.intensity_at(1, 2); // left
        assert!(n > 0.0, "fire should propagate down, got {n}");
        assert!(s > 0.0, "fire should propagate up, got {s}");
        assert!(e > 0.0, "fire should propagate right, got {e}");
        assert!(w > 0.0, "fire should propagate left, got {w}");
        // Diagonals must NOT be ignited by a single step (4-neighbourhood).
        assert_eq!(
            grid.intensity_at(3, 3),
            0.0,
            "diagonals should not ignite in a single step"
        );

        // The source cell must still be burning, just dimmer than 1.0.
        let centre_after_step1 = grid.intensity_at(2, 2);
        assert!(
            centre_after_step1 > 0.0 && centre_after_step1 < 1.0,
            "centre should be decaying, got {centre_after_step1}"
        );

        // Step until the grid is fully quiescent. Cap the loop so a buggy
        // implementation that loops forever fails the test instead of
        // hanging the harness.
        let mut ticks = 0u32;
        while !grid.is_quiescent(&params) {
            grid.step(&params);
            ticks += 1;
            assert!(
                ticks < 10_000,
                "grid never decayed to zero after {ticks} ticks"
            );
        }

        // Final state: every cell is below the threshold (i.e. effectively
        // zero) and the total intensity is zero.
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                assert!(
                    grid.intensity_at(x, y) < params.ignition_threshold,
                    "cell ({x}, {y}) should be quiescent, got {}",
                    grid.intensity_at(x, y)
                );
            }
        }
        approx_eq(grid.total_intensity(), 0.0);
    }

    /// Out-of-bounds `ignite` calls and reads must be silent no-ops, not
    /// panics, so callers can safely sparkle along map edges.
    #[test]
    fn out_of_bounds_is_noop() {
        let mut grid = DisasterGrid::new(3, 3);
        let params = DisasterParams::default();

        // Off-grid ignition — no panic, no side effects.
        grid.ignite(99, 99, 1.0, HazardKind::Fire);
        grid.ignite(usize::MAX, 0, 1.0, HazardKind::Flood);
        grid.step(&params);

        assert!(grid.is_quiescent(&params));
        // Off-grid reads return 0.0.
        approx_eq(grid.intensity_at(99, 99), 0.0);
    }

    /// `ignite` should take the *max* of existing and incoming intensity at
    /// a cell — re-igniting a smouldering ember must not reduce it.
    #[test]
    fn ignite_uses_max_intensity() {
        let mut grid = DisasterGrid::new(2, 2);
        grid.ignite(0, 0, 0.5, HazardKind::Fire);
        grid.ignite(0, 0, 0.2, HazardKind::Flood);
        approx_eq(grid.intensity_at(0, 0), 0.5);

        // Stronger re-ignition should lift the cell.
        grid.ignite(0, 0, 0.9, HazardKind::Fire);
        approx_eq(grid.intensity_at(0, 0), 0.9);
    }

    #[test]
    fn default_cell_is_inactive_with_canonical_fire_kind() {
        let cell = HazardCell::default();

        assert_eq!(cell.intensity, 0.0);
        assert_eq!(cell.kind, HazardKind::Fire);
    }
}
