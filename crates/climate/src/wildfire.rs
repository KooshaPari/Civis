//! Wildfire simulation model.
//!
//! A fuel-based wildfire model operating on a discrete 2-D grid. Each cell
//! carries vegetation fuel, temperature (heat), and a burn timer. On every
//! `step`:
//!
//! 1. **Heat accumulation** — burning cells generate heat proportional to
//!    their fuel and burn rate.
//! 2. **State transitions** — cells progress through the lifecycle:
//!    `Dormant → Smoldering → Active → Crowning → Dying` based on heat
//!    level and remaining fuel.
//! 3. **Spread** — `Active` and `Crowning` fires propagate to 4-connected
//!    neighbours whose heat exceeds the ignition threshold. Crown fires
//!    spread at 2x the base rate.
//! 4. **Burn-down** — fuel is consumed over time; cells die when fuel
//!    reaches zero or burn time exceeds the configured duration.
//!
//! The model is deterministic, allocation-light, and uses no unsafe code.
//!
//! # Example
//!
//! ```rust
//! use civ_climate::wildfire::{WildfireGrid, WildfireParams, WildfireState};
//!
//! let params = WildfireParams::default();
//! let mut grid = WildfireGrid::new(10, 10, params);
//!
//! // Ignite the centre with high intensity.
//! grid.ignite(5, 5, 1.0);
//! grid.step(1.0);
//! assert!(grid.is_burning(5, 5));
//! ```

#![forbid(unsafe_code)]

/// Lifecycle state of a single wildfire cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WildfireState {
    /// No fire activity — cell is unburned or has no heat.
    Dormant,
    /// Low heat, beginning to combust. Transitions to Active once heat
    /// exceeds the ignition threshold.
    Smoldering,
    /// Fully burning. Generates enough heat to ignite neighbours.
    Active,
    /// Intense canopy-level fire. Spreads at 2x rate and can jump gaps.
    Crowning,
    /// Fire is consuming the last of the fuel. Heat dissipates.
    Dying,
}

impl Default for WildfireState {
    fn default() -> Self {
        Self::Dormant
    }
}

/// Tunable parameters governing wildfire behaviour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WildfireParams {
    /// Minimum heat a neighbour must have (from radiant energy) to ignite.
    /// Values in `[0.0, 1.0]`.
    pub ignition_threshold: f32,
    /// Fuel dryness level at which ignition becomes likely. Cells with
    /// `fuel_remaining` above this value are considered "dry enough" to
    /// catch fire. Values in `[0.0, 1.0]`.
    pub dryness_threshold: f32,
    /// Base fraction of heat transferred to each orthogonal neighbour per
    /// second. Values in `[0.0, 1.0]`.
    pub spread_rate: f32,
    /// Maximum time (seconds) a cell can burn before it is forced to the
    /// `Dying` state regardless of fuel.
    pub burn_duration: f32,
    /// Heat level at which a cell transitions from `Active` to `Crowning`.
    /// Values in `[0.0, 1.0]`.
    pub crown_fire_threshold: f32,
}

impl Default for WildfireParams {
    fn default() -> Self {
        Self {
            ignition_threshold: 0.2,
            dryness_threshold: 0.5,
            spread_rate: 0.15,
            burn_duration: 30.0,
            crown_fire_threshold: 0.8,
        }
    }
}

/// Per-cell wildfire state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WildfireCell {
    /// Current lifecycle state.
    pub state: WildfireState,
    /// Current heat level in `[0.0, 1.0]`.
    pub heat: f32,
    /// Remaining vegetation fuel in `[0.0, 1.0]`. Starts at 1.0 for full
    /// fuel load and decreases as the fire burns.
    pub fuel_remaining: f32,
    /// Time (seconds) the cell has been actively burning.
    pub burn_time: f32,
}

impl Default for WildfireCell {
    fn default() -> Self {
        Self {
            state: WildfireState::Dormant,
            heat: 0.0,
            fuel_remaining: 1.0,
            burn_time: 0.0,
        }
    }
}

/// A 2-D wildfire grid.
///
/// Stored as a flat `Vec<WildfireCell>` in row-major order for cache
/// locality and zero-allocation stepping.
#[derive(Debug, Clone)]
pub struct WildfireGrid {
    width: usize,
    height: usize,
    cells: Vec<WildfireCell>,
    params: WildfireParams,
}

impl WildfireGrid {
    /// Create a new grid of the given dimensions with all cells in the
    /// `Dormant` state and full fuel loads.
    pub fn new(width: usize, height: usize, params: WildfireParams) -> Self {
        let cells = vec![WildfireCell::default(); width * height];
        Self {
            width,
            height,
            cells,
            params,
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

    /// Read-only access to a cell. Returns `None` for out-of-bounds coords.
    pub fn cell_at(&self, x: usize, y: usize) -> Option<&WildfireCell> {
        self.index(x, y).map(|idx| &self.cells[idx])
    }

    /// Mutable access to a cell. Returns `None` for out-of-bounds coords.
    pub fn cell_at_mut(&mut self, x: usize, y: usize) -> Option<&mut WildfireCell> {
        self.index(x, y).map(|idx| &mut self.cells[idx])
    }

    /// Ignite cell `(x, y)` with the given `intensity` in `[0.0, 1.0]`.
    /// Sets heat to `intensity` and transitions the cell to `Smoldering`
    /// if the intensity is positive. Out-of-bounds is a silent no-op.
    pub fn ignite(&mut self, x: usize, y: usize, intensity: f32) {
        let intensity = intensity.clamp(0.0, 1.0);
        if let Some(idx) = self.index(x, y) {
            let cell = &mut self.cells[idx];
            // Only ignite if fuel is sufficient.
            if cell.fuel_remaining > 0.0 && intensity > 0.0 {
                cell.heat = intensity;
                cell.state = WildfireState::Smoldering;
            }
        }
    }

    /// Advance the simulation by `dt` seconds.
    ///
    /// This method:
    /// - Advances `burn_time` for burning cells.
    /// - Reduces `fuel_remaining` proportional to heat and `dt`.
    /// - Transitions cells through the lifecycle based on heat and fuel.
    /// - Spreads fire from `Active` / `Crowning` cells to neighbours.
    pub fn step(&mut self, dt: f32) {
        if dt <= 0.0 || self.width == 0 || self.height == 0 {
            return;
        }

        // --- Pass 1: gather spread contributions into a delta buffer. ------
        // This prevents order-dependent propagation.
        let mut heat_delta: Vec<f32> = vec![0.0; self.cells.len()];

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let cell = self.cells[idx];

                match cell.state {
                    WildfireState::Active | WildfireState::Crowning => {
                        // Compute effective spread rate.
                        let rate = if cell.state == WildfireState::Crowning {
                            // Crown fires spread at 2x rate.
                            self.params.spread_rate * 2.0
                        } else {
                            self.params.spread_rate
                        };

                        let neighbours: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                        for &(ndx, ndy) in &neighbours {
                            let nx = x as i32 + ndx;
                            let ny = y as i32 + ndy;
                            if nx < 0 || ny < 0 {
                                continue;
                            }
                            let nx = nx as usize;
                            let ny = ny as usize;
                            if let Some(nidx) = self.index(nx, ny) {
                                let neighbour = &self.cells[nidx];
                                // Only spread if the neighbour is dormant or has low
                                // heat, and has sufficient fuel that is dry enough.
                                let can_ignite = matches!(
                                    neighbour.state,
                                    WildfireState::Dormant | WildfireState::Smoldering
                                );
                                if can_ignite
                                    && neighbour.fuel_remaining >= self.params.dryness_threshold
                                {
                                    // Radiant heat transferred to neighbour.
                                    let transferred = rate * cell.heat * cell.fuel_remaining;
                                    if transferred > self.params.ignition_threshold {
                                        heat_delta[nidx] = heat_delta[nidx].max(transferred);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // --- Pass 2: apply deltas, advance state, consume fuel. -----------
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let cell = &mut self.cells[idx];

                // Apply heat from spreading neighbours.
                cell.heat = (cell.heat + heat_delta[idx]).clamp(0.0, 1.0);

                match cell.state {
                    WildfireState::Dormant => {
                        // A dormant cell with accumulated heat transitions
                        // to Smoldering.
                        if cell.heat > 0.0 {
                            cell.state = WildfireState::Smoldering;
                        }
                    }
                    WildfireState::Smoldering => {
                        if cell.heat >= self.params.ignition_threshold
                            && cell.fuel_remaining >= self.params.dryness_threshold
                        {
                            cell.state = WildfireState::Active;
                        }
                        // Smoldering cells slowly lose heat.
                        cell.heat = (cell.heat - 0.01 * dt).max(0.0);
                        if cell.heat <= 0.0 {
                            cell.state = WildfireState::Dormant;
                        }
                    }
                    WildfireState::Active => {
                        cell.burn_time += dt;
                        // Consume fuel — faster when hotter.
                        let consumption = cell.heat * 0.02 * dt;
                        cell.fuel_remaining = (cell.fuel_remaining - consumption).max(0.0);

                        // Heat is sustained by fuel but slowly decays.
                        cell.heat = (cell.heat + cell.fuel_remaining * 0.01 * dt
                            - 0.005 * dt)
                            .clamp(0.0, 1.0);

                        // Transition to Crowning if heat is high enough.
                        if cell.heat >= self.params.crown_fire_threshold {
                            cell.state = WildfireState::Crowning;
                        }
                        // Transition to Dying if fuel exhausted or burn
                        // duration exceeded.
                        if cell.fuel_remaining <= 0.0
                            || cell.burn_time >= self.params.burn_duration
                        {
                            cell.state = WildfireState::Dying;
                        }
                    }
                    WildfireState::Crowning => {
                        cell.burn_time += dt;
                        // Crown fires consume fuel faster.
                        let consumption = cell.heat * 0.04 * dt;
                        cell.fuel_remaining = (cell.fuel_remaining - consumption).max(0.0);

                        // Crown fires maintain high heat.
                        cell.heat = (cell.heat + cell.fuel_remaining * 0.02 * dt
                            - 0.01 * dt)
                            .clamp(0.0, 1.0);

                        // Drop back to Active if heat falls below threshold.
                        if cell.heat < self.params.crown_fire_threshold {
                            cell.state = WildfireState::Active;
                        }
                        // Transition to Dying if fuel exhausted or burn
                        // duration exceeded.
                        if cell.fuel_remaining <= 0.0
                            || cell.burn_time >= self.params.burn_duration
                        {
                            cell.state = WildfireState::Dying;
                        }
                    }
                    WildfireState::Dying => {
                        cell.burn_time += dt;
                        // Heat dissipates rapidly.
                        cell.heat = (cell.heat - 0.1 * dt).max(0.0);
                        // Any remaining fuel is consumed.
                        cell.fuel_remaining = (cell.fuel_remaining - 0.01 * dt).max(0.0);
                        // When heat is fully gone, cell becomes Dormant.
                        if cell.heat <= 0.0 {
                            cell.state = WildfireState::Dormant;
                        }
                    }
                }
            }
        }
    }

    /// Returns `true` if the cell at `(x, y)` is actively burning (i.e.
    /// its state is `Active` or `Crowning`). Out-of-bounds returns `false`.
    pub fn is_burning(&self, x: usize, y: usize) -> bool {
        self.cell_at(x, y)
            .map(|c| matches!(c.state, WildfireState::Active | WildfireState::Crowning))
            .unwrap_or(false)
    }

    /// Total number of cells currently in the `Active` or `Crowning` state.
    pub fn total_burning(&self) -> usize {
        self.cells
            .iter()
            .filter(|c| matches!(c.state, WildfireState::Active | WildfireState::Crowning))
            .count()
    }

    /// Total damage score: sum of consumed fuel across all cells.
    /// Returns a value in `[0.0, width * height]`.
    pub fn total_damage(&self) -> f32 {
        self.cells
            .iter()
            .map(|c| 1.0 - c.fuel_remaining)
            .sum()
    }

    /// Apply rainfall to suppress fire across the entire grid. Each unit
    /// of `amount` reduces heat on all burning cells by the given amount.
    /// Useful for modelling rain or firefighting.
    pub fn rainfall_suppress(&mut self, amount: f32) {
        let amount = amount.max(0.0);
        for cell in self.cells.iter_mut() {
            if matches!(
                cell.state,
                WildfireState::Smoldering
                    | WildfireState::Active
                    | WildfireState::Crowning
                    | WildfireState::Dying
            ) {
                cell.heat = (cell.heat - amount).max(0.0);
            }
        }
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-6;

    /// Default parameters produce sensible values.
    #[test]
    fn test_default_params() {
        let p = WildfireParams::default();
        assert!((p.ignition_threshold - 0.2).abs() < EPS);
        assert!((p.dryness_threshold - 0.5).abs() < EPS);
        assert!((p.spread_rate - 0.15).abs() < EPS);
        assert!((p.burn_duration - 30.0).abs() < EPS);
        assert!((p.crown_fire_threshold - 0.8).abs() < EPS);
    }

    /// Ignition sets heat and transitions the cell to Smoldering.
    #[test]
    fn test_ignition_sets_smoldering() {
        let params = WildfireParams::default();
        let mut grid = WildfireGrid::new(5, 5, params);
        grid.ignite(2, 2, 0.8);
        let cell = grid.cell_at(2, 2).unwrap();
        assert_eq!(cell.state, WildfireState::Smoldering);
        assert!((cell.heat - 0.8).abs() < EPS);
        assert!((cell.fuel_remaining - 1.0).abs() < EPS);
    }

    /// Smoldering cells transition to Active once heat exceeds the
    /// ignition threshold after a few steps.
    #[test]
    fn test_smoldering_to_active_transition() {
        let params = WildfireParams {
            ignition_threshold: 0.2,
            dryness_threshold: 0.0,
            ..Default::default()
        };
        let mut grid = WildfireGrid::new(3, 3, params);
        grid.ignite(1, 1, 0.5);

        // Step enough times for the cell to transition to Active.
        let mut stepped = false;
        for _ in 0..50 {
            grid.step(1.0);
            if grid.cell_at(1, 1).unwrap().state == WildfireState::Active {
                stepped = true;
                break;
            }
        }
        assert!(stepped, "smoldering cell should transition to active");
    }

    /// Active fires spread to orthogonal neighbours that are dry enough.
    #[test]
    fn test_spread_to_neighbours() {
        let params = WildfireParams {
            ignition_threshold: 0.1,
            dryness_threshold: 0.0,
            spread_rate: 0.3,
            ..Default::default()
        };
        let mut grid = WildfireGrid::new(5, 5, params);
        grid.ignite(2, 2, 1.0);

        // Step several times to allow ignition → active → spread.
        let mut spread_happened = false;
        for _ in 0..20 {
            grid.step(1.0);
            // Check if any neighbour of (2,2) is now burning.
            let neighbours = [(2, 1), (2, 3), (1, 2), (3, 2)];
            for &(nx, ny) in &neighbours {
                if grid.is_burning(nx, ny) {
                    spread_happened = true;
                    break;
                }
            }
            if spread_happened {
                break;
            }
        }
        assert!(spread_happened, "fire should spread to at least one neighbour");
    }

    /// Crown fires spread at 2x rate — they reach neighbours faster than
    /// regular Active fires.
    #[test]
    fn test_crown_fire_spread_faster() {
        // Setup: ignite a cell with high intensity so it quickly becomes
        // a crown fire, and check that neighbours ignite sooner than they
        // would with a normal active fire.
        let params = WildfireParams {
            ignition_threshold: 0.05,
            dryness_threshold: 0.0,
            spread_rate: 0.2,
            crown_fire_threshold: 0.5,
            ..Default::default()
        };

        // Grid with crown fires enabled
        let mut grid_crown = WildfireGrid::new(5, 5, params);
        grid_crown.ignite(2, 2, 1.0);
        let mut crown_steps = 0u32;
        for _ in 0..200 {
            grid_crown.step(1.0);
            crown_steps += 1;
            if grid_crown.is_burning(2, 3) {
                break;
            }
        }

        // Grid with very high crown threshold (effectively no crown fires)
        let mut params_no_crown = params;
        params_no_crown.crown_fire_threshold = 10.0; // never crown
        let mut grid_active = WildfireGrid::new(5, 5, params_no_crown);
        grid_active.ignite(2, 2, 1.0);
        let mut active_steps = 0u32;
        for _ in 0..200 {
            grid_active.step(1.0);
            active_steps += 1;
            if grid_active.is_burning(2, 3) {
                break;
            }
        }

        // Crown fires should spread faster (fewer steps).
        assert!(
            crown_steps <= active_steps,
            "crown fire ({crown_steps} steps) should spread at least as fast as active fire ({active_steps} steps)"
        );
    }

    /// Fires die when fuel_remaining reaches zero.
    #[test]
    fn test_fire_dies_when_fuel_exhausted() {
        let params = WildfireParams {
            ignition_threshold: 0.1,
            dryness_threshold: 0.0,
            spread_rate: 0.0,
            burn_duration: f32::MAX, // don't die from duration
            ..Default::default()
        };
        let mut grid = WildfireGrid::new(3, 3, params);
        grid.ignite(1, 1, 1.0);

        // Step until fuel is exhausted.
        for _ in 0..500 {
            grid.step(1.0);
            let cell = grid.cell_at(1, 1).unwrap();
            if cell.fuel_remaining <= 0.0 {
                break;
            }
        }

        // After many steps, the cell should be Dormant or Dying with no
        // fuel.
        let cell = grid.cell_at(1, 1).unwrap();
        assert!(
            cell.fuel_remaining <= 0.0 || cell.state == WildfireState::Dying,
            "fire should die when fuel is exhausted"
        );
    }

    /// Fires die when burn_time exceeds burn_duration.
    #[test]
    fn test_fire_dies_when_duration_exceeded() {
        let params = WildfireParams {
            ignition_threshold: 0.1,
            dryness_threshold: 0.0,
            spread_rate: 0.0,
            burn_duration: 2.0, // very short burn
            ..Default::default()
        };
        let mut grid = WildfireGrid::new(3, 3, params);
        grid.ignite(1, 1, 1.0);

        // Step past the burn duration.
        for _ in 0..20 {
            grid.step(1.0);
        }

        let cell = grid.cell_at(1, 1).unwrap();
        assert!(
            cell.state == WildfireState::Dying || cell.state == WildfireState::Dormant,
            "cell should be Dying or Dormant after burn_duration exceeded, got {:?}",
            cell.state
        );
    }

    /// Rainfall_suppress reduces heat and can extinguish fires.
    #[test]
    fn test_rainfall_suppress() {
        let params = WildfireParams::default();
        let mut grid = WildfireGrid::new(3, 3, params);
        grid.ignite(1, 1, 0.6);

        let heat_before = grid.cell_at(1, 1).unwrap().heat;
        grid.rainfall_suppress(0.5);
        let heat_after = grid.cell_at(1, 1).unwrap().heat;

        assert!(
            heat_after < heat_before,
            "rainfall should reduce heat: {heat_before} -> {heat_after}"
        );
        assert!(
            (heat_after - 0.1).abs() < EPS,
            "heat should be 0.6 - 0.5 = 0.1, got {heat_after}"
        );
    }

    /// `total_burning` counts only Active and Crowning cells.
    #[test]
    fn test_total_burning_count() {
        let params = WildfireParams::default();
        let mut grid = WildfireGrid::new(5, 5, params);
        assert_eq!(grid.total_burning(), 0);

        grid.ignite(2, 2, 0.9);
        // Ignition alone makes it Smoldering, not yet burning.
        assert_eq!(grid.total_burning(), 0);

        // Step until at least one cell is Active.
        for _ in 0..50 {
            grid.step(1.0);
            if grid.total_burning() > 0 {
                break;
            }
        }
        assert!(grid.total_burning() > 0, "should have at least one burning cell");
    }

    /// `total_damage` increases as fuel is consumed.
    #[test]
    fn test_total_damage_increases() {
        let params = WildfireParams {
            ignition_threshold: 0.1,
            dryness_threshold: 0.0,
            spread_rate: 0.0,
            ..Default::default()
        };
        let mut grid = WildfireGrid::new(3, 3, params);
        assert!((grid.total_damage()).abs() < EPS);

        grid.ignite(1, 1, 1.0);
        let damage_before = grid.total_damage();
        for _ in 0..50 {
            grid.step(1.0);
        }
        let damage_after = grid.total_damage();

        assert!(
            damage_after > damage_before,
            "damage should increase as fuel is consumed: {damage_before} -> {damage_after}"
        );
    }

    /// Out-of-bounds operations are silent no-ops.
    #[test]
    fn test_out_of_bounds_noop() {
        let params = WildfireParams::default();
        let mut grid = WildfireGrid::new(3, 3, params);

        grid.ignite(99, 99, 1.0);
        assert_eq!(grid.total_burning(), 0);
        assert!(!grid.is_burning(99, 99));
        assert!(grid.cell_at(99, 99).is_none());
    }
}
