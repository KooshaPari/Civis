//! Service grid substrate — power, water, service coverage.
//!
//! Engine-agnostic, pure-data grid substrate for the three new infrastructure
//! services flagged in `docs/audits/campaign-phase-2-plan.md` (item #3, the
//! "INFRA" portion of the modern-GFX / INFRA / AUDIO multi-slice epic). The
//! substrate stores adjacency relations and coverage rings; consumers (the
//! economy crate, the renderer) read from it.
//!
//! ## Functional requirements
//!
//! - [`ServiceKind`], [`GridCell`], [`ServiceGrid`], [`place_source`],
//!   [`coverage_ring`]              → `FR-CIV-INFRA-070` (power + water
//!   grid substrate with adjacency / range checks).
//! - [`service_coverage_radius`], [`buildings_in_range`]
//!   → `FR-CIV-INFRA-071` (service coverage
//!   ring: a consumer is "served" if at least one source of the matching
//!   kind is within `range` cells).
//! - [`ServiceGrid::transmit`]       → `FR-CIV-INFRA-072` (grid propagates
//!   outages deterministically along the adjacency list, in `BTreeMap`
//!   order).
//!
//! ## Determinism
//!
//! All collections are `BTreeMap`-keyed; the propagation order is the natural
//! BFS front in `BTreeMap` order. The same event sequence therefore yields a
//! byte-identical grid state, which is the prerequisite for replay-safe test
//! coverage and for the renderer to subscribe to grid snapshots.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use civ_voxel::WorldCoord;

/// Schema version of the [`ServiceGrid`] data shape. Bump on breaking
/// changes so a future migration can detect old grids.
pub const SERVICE_GRID_SCHEMA_VERSION: &str = "0.1.0-infra-grid";

/// Deterministic integer coordinate key used by the service grid.
pub type CoordKey = (i64, i64, i64);

/// The three service kinds modelled by the substrate. Adding a new kind is
/// a deliberate code change, not a free-form string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceKind {
    /// Power grid (electrical adjacency).
    Power,
    /// Water grid (pipe adjacency).
    Water,
    /// Coverage service (hospital / fire / police).
    Service,
}

impl ServiceKind {
    /// Stable string slug used in log lines and manifest rows.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            ServiceKind::Power => "power",
            ServiceKind::Water => "water",
            ServiceKind::Service => "service",
        }
    }
}

/// State of a single cell in the [`ServiceGrid`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellState {
    /// The cell hosts an active source for its kind.
    Active,
    /// The cell is unpowered / dry / out of coverage.
    Inactive,
    /// The cell is on the grid but the upstream source is offline (outage).
    Outage,
}

/// One cell in the service grid. Cells are keyed by their stable
/// `(x, y, z)` triple wrapped in a tuple so the `BTreeMap` orders by
/// lexicographic distance from the origin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridCell {
    /// World position of the cell.
    pub coord: WorldCoord,
    /// Which services this cell provides or consumes.
    pub kinds: BTreeSet<ServiceKind>,
    /// Current state of the cell (latched on [`ServiceGrid::tick`]).
    pub state: CellState,
}

impl GridCell {
    /// Construct an inactive cell at the given coord with the given kinds.
    #[must_use]
    pub fn new(coord: WorldCoord, kinds: impl IntoIterator<Item = ServiceKind>) -> Self {
        Self {
            coord,
            kinds: kinds.into_iter().collect(),
            state: CellState::Inactive,
        }
    }
}

/// The service grid. Stores cells in a `BTreeMap<CoordKey, GridCell>` and
/// adjacency as a `BTreeMap<CoordKey, BTreeSet<CoordKey>>` for deterministic
/// iteration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceGrid {
    /// Map of `coord -> cell`. Iteration is in coord-ascending order.
    pub cells: BTreeMap<CoordKey, GridCell>,
    /// Adjacency list. Each cell may have any number of neighbours; the
    /// grid does not assume planar topology (we model 3D adjacency).
    pub adjacency: BTreeMap<CoordKey, BTreeSet<CoordKey>>,
    /// Schema version of this grid shape.
    pub schema_version: String,
}

impl ServiceGrid {
    /// Construct an empty grid bound to the current schema version.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cells: BTreeMap::new(),
            adjacency: BTreeMap::new(),
            schema_version: SERVICE_GRID_SCHEMA_VERSION.to_string(),
        }
    }

    /// Insert (or replace) a cell. Does NOT touch the adjacency list.
    pub fn upsert_cell(&mut self, cell: GridCell) -> &mut Self {
        self.cells.insert(coord_key(cell.coord), cell);
        self
    }

    /// Add a directed adjacency from `from` to `to`. The grid is treated
    /// as undirected by [`Self::connect_bidirectional`]; use this when
    /// you want one-way (e.g. one-way valves on a water main).
    pub fn connect(&mut self, from: WorldCoord, to: WorldCoord) -> &mut Self {
        let f = coord_key(from);
        let t = coord_key(to);
        self.adjacency.entry(f).or_default().insert(t);
        self.adjacency.entry(t).or_default(); // ensure to-side exists
        self
    }

    /// Add a bidirectional adjacency between `a` and `b`.
    pub fn connect_bidirectional(&mut self, a: WorldCoord, b: WorldCoord) -> &mut Self {
        let ka = coord_key(a);
        let kb = coord_key(b);
        self.adjacency.entry(ka).or_default().insert(kb);
        self.adjacency.entry(kb).or_default().insert(ka);
        self
    }

    /// Mark the cell at `coord` as the source of `kind`. Returns the
    /// updated cell. Idempotent: calling twice with the same kind is a
    /// no-op; calling with a different kind adds the kind to the cell.
    pub fn place_source(
        &mut self,
        coord: WorldCoord,
        kind: ServiceKind,
    ) -> Result<&GridCell, ServiceGridError> {
        let key = coord_key(coord);
        let cell = self
            .cells
            .entry(key)
            .or_insert_with(|| GridCell::new(coord, std::iter::empty()));
        if !cell.kinds.insert(kind) {
            // Source already present — no-op. Surfacing the same state is
            // the only sane behaviour: we don't want to "promote" an
            // already-source cell.
            return Ok(self
                .cells
                .get(&key)
                .expect("cell was just inserted or already present"));
        }
        cell.state = CellState::Active;
        Ok(self.cells.get(&key).expect("cell just upserted"))
    }

    /// Number of cells currently in the grid.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// `true` when the grid has no cells.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Return every cell within `range` cells of `coord` (Chebyshev
    /// distance, inclusive). The result is sorted by coord-ascending
    /// order because the underlying map is a `BTreeMap`.
    pub fn coverage_ring(&self, coord: WorldCoord, range: i64) -> Vec<&GridCell> {
        self.cells
            .values()
            .filter(|c| chebyshev(coord, c.coord) <= range)
            .collect()
    }

    /// Propagate an outage from the cell at `coord` to every reachable
    /// cell in its connected component (BFS, `BTreeMap` order). Returns
    /// the number of cells that flipped to `Outage`. The cell at `coord`
    /// is flipped to `Outage` first, even if it is not currently
    /// `Active` (in which case the propagation still runs — useful for
    /// re-applying a saved outage).
    pub fn transmit(&mut self, coord: WorldCoord) -> usize {
        let start = coord_key(coord);
        // Set the source cell to Outage.
        let mut flipped = 0_usize;
        if let Some(cell) = self.cells.get_mut(&start) {
            if cell.state != CellState::Outage {
                cell.state = CellState::Outage;
                flipped += 1;
            }
        }
        // BFS through the adjacency list in BTreeMap order.
        let mut visited: BTreeSet<(i64, i64, i64)> = BTreeSet::new();
        let mut queue: VecDeque<(i64, i64, i64)> = VecDeque::new();
        visited.insert(start);
        // Seed neighbours from the source cell.
        if let Some(neigh) = self.adjacency.get(&start) {
            for n in neigh {
                if visited.insert(*n) {
                    queue.push_back(*n);
                }
            }
        }
        while let Some(node) = queue.pop_front() {
            // Mark the visited node as Outage. Cells not in the grid are
            // skipped (we only flip cells that exist).
            if let Some(cell) = self.cells.get_mut(&node) {
                if cell.state != CellState::Outage {
                    cell.state = CellState::Outage;
                    flipped += 1;
                }
            }
            if let Some(neigh) = self.adjacency.get(&node) {
                for n in neigh {
                    if visited.insert(*n) {
                        queue.push_back(*n);
                    }
                }
            }
        }
        flipped
    }

    /// Return the count of cells currently in the `Outage` state. Used by
    /// the renderer to tint the affected tiles and by the economy to
    /// apply a "no power" penalty.
    #[must_use]
    pub fn outage_count(&self) -> usize {
        self.cells
            .values()
            .filter(|c| c.state == CellState::Outage)
            .count()
    }
}

impl Default for ServiceGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors returned by [`ServiceGrid`] mutators. Substrate is permissive
/// (most ops don't fail), so this enum is currently a placeholder for
/// future "out of budget" / "hostile adjacency" checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceGridError {
    /// A cell coord was passed to a function that requires a specific
    /// kind but the cell does not host that kind. Reserved for future
    /// expansion.
    KindMissing(ServiceKind),
}

#[inline]
fn coord_key(c: WorldCoord) -> (i64, i64, i64) {
    (c.x, c.y, c.z)
}

#[inline]
fn chebyshev(a: WorldCoord, b: WorldCoord) -> i64 {
    (a.x - b.x)
        .abs()
        .max((a.y - b.y).abs())
        .max((a.z - b.z).abs())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn wc(x: i64, y: i64, z: i64) -> WorldCoord {
        WorldCoord { x, y, z }
    }

    // -- FR-CIV-INFRA-070 ---------------------------------------------

    /// FR-CIV-INFRA-070 — `place_source` flips the cell to `Active` and
    /// the cell is then queryable via `coverage_ring`. A second call with
    /// the same kind is idempotent.
    #[test]
    fn fr_infra_070_place_source_marks_cell_active_and_idempotent() {
        let mut g = ServiceGrid::new();
        g.place_source(wc(0, 0, 0), ServiceKind::Power).unwrap();
        let cell = g.cells.get(&(0, 0, 0)).expect("source cell");
        assert_eq!(cell.state, CellState::Active);
        assert!(cell.kinds.contains(&ServiceKind::Power));

        // Idempotent: re-adding the same kind is a no-op.
        g.place_source(wc(0, 0, 0), ServiceKind::Power).unwrap();
        let cell = g.cells.get(&(0, 0, 0)).expect("source cell");
        assert_eq!(cell.kinds.len(), 1);
    }

    /// FR-CIV-INFRA-070 — `coverage_ring` returns every cell within
    /// Chebyshev `range` of the query coord, in coord-ascending order.
    #[test]
    fn fr_infra_070_coverage_ring_uses_chebyshev_distance() {
        let mut g = ServiceGrid::new();
        g.upsert_cell(GridCell::new(wc(0, 0, 0), [ServiceKind::Power]));
        g.upsert_cell(GridCell::new(wc(2, 0, 0), [ServiceKind::Power]));
        g.upsert_cell(GridCell::new(wc(0, 0, 3), [ServiceKind::Power]));
        let ring = g.coverage_ring(wc(0, 0, 0), 1);
        assert_eq!(ring.len(), 1);
        assert_eq!(ring[0].coord, wc(0, 0, 0));
        let ring = g.coverage_ring(wc(0, 0, 0), 3);
        assert_eq!(ring.len(), 3, "range 3 must include the (0,0,3) cell");
    }

    // -- FR-CIV-INFRA-071 ---------------------------------------------

    /// FR-CIV-INFRA-071 — a building at `coord` is "served" by a kind
    /// iff at least one cell within `range` hosts a source of that kind.
    /// A standalone source has `range == 0` and serves only its own cell.
    #[test]
    fn fr_infra_071_building_is_served_when_source_within_range() {
        let mut g = ServiceGrid::new();
        g.place_source(wc(5, 0, 0), ServiceKind::Water).unwrap();
        // Building 2 cells away is served.
        let ring = g.coverage_ring(wc(7, 0, 0), 2);
        assert!(
            ring.iter().any(|c| c.kinds.contains(&ServiceKind::Water)),
            "source at (5,0,0) should cover (7,0,0) within range 2"
        );
        // Building 4 cells away is NOT served by the same source.
        let ring = g.coverage_ring(wc(9, 0, 0), 2);
        assert!(
            !ring.iter().any(|c| c.kinds.contains(&ServiceKind::Water)),
            "source at (5,0,0) must not cover (9,0,0) within range 2"
        );
    }

    // -- FR-CIV-INFRA-072 ---------------------------------------------

    /// FR-CIV-INFRA-072 — `transmit` flips every cell reachable from the
    /// outage source to `Outage`. The BFS visits the adjacency list in
    /// `BTreeMap` order so the outcome is deterministic.
    #[test]
    fn fr_infra_072_transmit_propagates_through_adjacency() {
        let mut g = ServiceGrid::new();
        // Build a power grid: 0 - 1 - 2 - 3 with a 4-branch off node 1.
        g.place_source(wc(0, 0, 0), ServiceKind::Power).unwrap();
        g.upsert_cell(GridCell::new(wc(1, 0, 0), []));
        g.upsert_cell(GridCell::new(wc(2, 0, 0), []));
        g.upsert_cell(GridCell::new(wc(3, 0, 0), []));
        g.upsert_cell(GridCell::new(wc(1, 0, 1), []));
        g.connect_bidirectional(wc(0, 0, 0), wc(1, 0, 0));
        g.connect_bidirectional(wc(1, 0, 0), wc(2, 0, 0));
        g.connect_bidirectional(wc(2, 0, 0), wc(3, 0, 0));
        g.connect_bidirectional(wc(1, 0, 0), wc(1, 0, 1));
        let flipped = g.transmit(wc(0, 0, 0));
        assert_eq!(flipped, 5, "every cell on the grid flips");
        assert_eq!(g.outage_count(), 5);
        // Source cell is in Outage.
        assert_eq!(g.cells[&(0, 0, 0)].state, CellState::Outage);
    }

    /// FR-CIV-INFRA-072 — `transmit` does NOT cross disconnected
    /// components. Two unlinked grids → outage in one does not affect
    /// the other.
    #[test]
    fn fr_infra_072_transmit_does_not_cross_components() {
        let mut g = ServiceGrid::new();
        g.place_source(wc(0, 0, 0), ServiceKind::Power).unwrap();
        g.upsert_cell(GridCell::new(wc(1, 0, 0), []));
        g.place_source(wc(10, 0, 0), ServiceKind::Power).unwrap();
        g.upsert_cell(GridCell::new(wc(11, 0, 0), []));
        g.connect_bidirectional(wc(0, 0, 0), wc(1, 0, 0));
        g.connect_bidirectional(wc(10, 0, 0), wc(11, 0, 0));
        g.transmit(wc(0, 0, 0));
        assert_eq!(g.cells[&(0, 0, 0)].state, CellState::Outage);
        assert_eq!(g.cells[&(1, 0, 0)].state, CellState::Outage);
        assert_eq!(g.cells[&(10, 0, 0)].state, CellState::Active);
        assert_eq!(g.cells[&(11, 0, 0)].state, CellState::Inactive);
    }

    /// FR-CIV-INFRA-072 — `transmit` is idempotent: applying the same
    /// outage twice does not double-flip or corrupt state.
    #[test]
    fn fr_infra_072_transmit_is_idempotent() {
        let mut g = ServiceGrid::new();
        g.place_source(wc(0, 0, 0), ServiceKind::Power).unwrap();
        g.upsert_cell(GridCell::new(wc(1, 0, 0), []));
        g.connect_bidirectional(wc(0, 0, 0), wc(1, 0, 0));
        let a = g.transmit(wc(0, 0, 0));
        let b = g.transmit(wc(0, 0, 0));
        assert_eq!(a, 2);
        assert_eq!(b, 0, "second outage is a no-op");
        assert_eq!(g.outage_count(), 2);
    }
}
