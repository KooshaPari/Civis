//! Shared physics substrate — the only coupling channel between emergent layers
//! (FR-CIV-PHYS-SUBSTRATE-001; see `docs/design/PHYSICS_COUPLING_SUBSTRATE.md`).
//!
//! # Doctrine (recap)
//!
//! Emergent layers (life, language, faction, religion, trade, architecture,
//! economy, climate, tactics, diplomacy, laws) **must not** call each other's
//! APIs. Their only coupling channel is this substrate: six conserved
//! continuous field arrays, indexed by a shared world grid.
//!
//! # The six fields
//!
//! | Symbol | Field            | Units                          |
//! |--------|------------------|--------------------------------|
//! | `T`    | Temperature      | K (clamped `[0, 6000]`)        |
//! | `M`    | Moisture / water | kg/m³-equivalent               |
//! | `E`    | Energy / joules  | J-equivalent                   |
//! | `F`    | Material flux    | kg-equivalent per cell per tick|
//! | `P`    | Population       | agents / m² (or biomass proxy) |
//! | `B`    | Biomass          | kg-equivalent                  |
//!
//! All six are conserved (Lagrangian) up to sources and sinks; the
//! [`PhysicsFields::evolve`] step enforces that and reports a [`Conservation`]
//! budget per tick.
//!
//! # Sprint 1 scope (per spec §7.1)
//!
//! This file ships the **skeleton only** — `PhysicsFields` plus an identity
//! `evolve` (no operators yet). Wiring into [`Simulation::tick`] between
//! `phase_climate` and `phase_agents` is the next step. Real operators
//! (advection, diffusion, reaction, regrowth) land in Sprints 2–4.
//!
//! The skeleton still buys us three things today:
//!
//! 1. A typed surface that emergent layers will write to (`PhysicsFields::set`).
//! 2. A conservation invariant we can test against the current planet /
//!    climate / voxel state (the empty evolve is `I`, so the conservation
//!    report is trivially `0` delta).
//! 3. A grid descriptor (`GridDescriptor`) that the rest of the substrate
//!    will reuse for indexing.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

/// Identifier of a field in [`PhysicsFields`].
///
/// Used as the type-safe index for [`PhysicsFields::get`] / [`PhysicsFields::set`].
/// The discriminant order matches the field list in the spec table; do not
/// reorder without a migration (tests assert the ordering).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Field {
    /// Temperature (K).
    T = 0,
    /// Moisture / water (kg/m³-equivalent).
    M = 1,
    /// Energy / joules (J-equivalent).
    E = 2,
    /// Material flux (kg-equivalent per cell per tick).
    F = 3,
    /// Population pressure (agents / m²).
    P = 4,
    /// Biomass / resources (kg-equivalent).
    B = 5,
}

impl Field {
    /// All six fields in discriminant order. Useful for `Σ` totals and the
    /// [`PhysicsFields::evolve`] / conservation bookkeeping.
    pub const ALL: [Field; 6] = [Field::T, Field::M, Field::E, Field::F, Field::P, Field::B];

    /// Short name (e.g. `"T"`, `"M"`). Matches the spec table column header.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Field::T => "T",
            Field::M => "M",
            Field::E => "E",
            Field::F => "F",
            Field::P => "P",
            Field::B => "B",
        }
    }
}

/// Shared 3D grid descriptor — same lattice used by `civ-voxel` and the
/// `GeologyMap` plate grid (chunked, indexed by `IVec3`).
///
/// Cell count must equal across all six fields in a [`PhysicsFields`] instance;
/// the constructor enforces this. The lattice is not a chunk-tree yet — Sprint
/// 1 uses a flat `Vec<f32>` per field. Chunked storage is a future optimization
/// gated on profile data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridDescriptor {
    /// Cell count along the x axis.
    pub nx: u32,
    /// Cell count along the y axis.
    pub ny: u32,
    /// Cell count along the z axis.
    pub nz: u32,
}

impl GridDescriptor {
    /// Construct a new grid descriptor. Returns `None` if any axis is zero or
    /// the total cell count would overflow `usize`.
    #[must_use]
    pub fn new(nx: u32, ny: u32, nz: u32) -> Option<Self> {
        if nx == 0 || ny == 0 || nz == 0 {
            return None;
        }
        // Guard against 32-bit overflow: total must fit in `usize`.
        let total = (nx as u64).checked_mul(ny as u64)?.checked_mul(nz as u64)?;
        if total > usize::MAX as u64 {
            return None;
        }
        Some(GridDescriptor { nx, ny, nz })
    }

    /// Total cell count (`nx * ny * nz`).
    #[must_use]
    pub fn cell_count(&self) -> usize {
        (self.nx as usize) * (self.ny as usize) * (self.nz as usize)
    }

    /// Index of a 3D coordinate `(x, y, z)` in the flat `Vec<f32>` storage.
    ///
    /// Returns `None` if the coordinate is out of bounds. Bounds are checked
    /// in the order `x ∈ [0, nx)`, `y ∈ [0, ny)`, `z ∈ [0, nz)`.
    #[must_use]
    pub fn index(&self, x: u32, y: u32, z: u32) -> Option<usize> {
        if x >= self.nx || y >= self.ny || z >= self.nz {
            return None;
        }
        Some(
            (x as usize) * (self.ny as usize) * (self.nz as usize)
                + (y as usize) * (self.nz as usize)
                + (z as usize),
        )
    }
}

/// The six conserved fields plus the grid that indexes them.
///
/// All six `Vec<f32>`s are the same length (`grid.cell_count()`); the
/// constructor enforces that. Storage is flat (row-major, x-fastest) for
/// cache-friendly scans; chunking is a future optimization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicsFields {
    /// Grid descriptor (shared by all six fields).
    pub grid: GridDescriptor,
    /// Temperature (K). Index: [`Field::T`].
    pub T: Vec<f32>,
    /// Moisture / water (kg/m³-equivalent). Index: [`Field::M`].
    pub M: Vec<f32>,
    /// Energy / joules (J-equivalent). Index: [`Field::E`].
    pub E: Vec<f32>,
    /// Material flux (kg-equivalent per cell per tick). Index: [`Field::F`].
    pub F: Vec<f32>,
    /// Population pressure (agents / m²). Index: [`Field::P`].
    pub P: Vec<f32>,
    /// Biomass / resources (kg-equivalent). Index: [`Field::B`].
    pub B: Vec<f32>,
}

impl PhysicsFields {
    /// Build a fresh fields instance with the given grid, all values
    /// initialised to `init_value`. The init value is typically `0.0`
    /// (cold start) or the projection of the current planet/climate/voxel
    /// state at the integration site.
    #[must_use]
    pub fn with_init(grid: GridDescriptor, init_value: f32) -> Self {
        let n = grid.cell_count();
        PhysicsFields {
            grid,
            T: vec![init_value; n],
            M: vec![init_value; n],
            E: vec![init_value; n],
            F: vec![init_value; n],
            P: vec![init_value; n],
            B: vec![init_value; n],
        }
    }

    /// Borrow the field array for a given [`Field`] discriminant.
    ///
    /// Used by emergent layers that want a read-only gradient view of one
    /// specific field (e.g. `civ-climate` reading `T`).
    #[must_use]
    pub fn get(&self, field: Field) -> &[f32] {
        match field {
            Field::T => &self.T,
            Field::M => &self.M,
            Field::E => &self.E,
            Field::F => &self.F,
            Field::P => &self.P,
            Field::B => &self.B,
        }
    }

    /// Mutable borrow of the field array for a given [`Field`] discriminant.
    ///
    /// Use this only from inside an operator — emergent layers should call
    /// [`Self::set`] (a typed, rate-limited setter) instead, per spec §2.3.
    #[must_use]
    pub fn get_mut(&mut self, field: Field) -> &mut [f32] {
        match field {
            Field::T => &mut self.T,
            Field::M => &mut self.M,
            Field::E => &mut self.E,
            Field::F => &mut self.F,
            Field::P => &mut self.P,
            Field::B => &mut self.B,
        }
    }

    /// Set a single cell's value via a typed, rate-checked setter.
    ///
    /// Sprint 1 ships this as a thin wrapper around `get_mut`; Sprint 5 wires
    /// the per-layer rate limits and conservation clamps (spec §2.3, §5).
    /// Coordinates out of bounds are silently dropped — the setter never
    /// panics, so a runaway write can never break the substrate.
    pub fn set(&mut self, field: Field, x: u32, y: u32, z: u32, value: f32) {
        if let Some(idx) = self.grid.index(x, y, z) {
            let arr = self.get_mut(field);
            arr[idx] = value;
        }
    }

    /// Sum of all cells in the given field (used by the conservation budget).
    #[must_use]
    pub fn sum(&self, field: Field) -> f32 {
        self.get(field).iter().copied().sum()
    }

    /// Total field sums, keyed by [`Field`]. Convenient for invariant reporting.
    #[must_use]
    pub fn totals(&self) -> Totals {
        Totals {
            T: self.sum(Field::T),
            M: self.sum(Field::M),
            E: self.sum(Field::E),
            F: self.sum(Field::F),
            P: self.sum(Field::P),
            B: self.sum(Field::B),
        }
    }
}

/// Per-field totals — the conservation report.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Totals {
    /// Σ T (K·cells).
    pub T: f32,
    /// Σ M (kg/m³·cells).
    pub M: f32,
    /// Σ E (J·cells).
    pub E: f32,
    /// Σ F (kg·cells per tick).
    pub F: f32,
    /// Σ P (agents·cells).
    pub P: f32,
    /// Σ B (kg·cells).
    pub B: f32,
}

impl Totals {
    /// Field total by discriminant. Symmetric with [`PhysicsFields::sum`].
    #[must_use]
    pub fn get(&self, field: Field) -> f32 {
        match field {
            Field::T => self.T,
            Field::M => self.M,
            Field::E => self.E,
            Field::F => self.F,
            Field::P => self.P,
            Field::B => self.B,
        }
    }

    /// Per-field delta vs. a prior `Totals`. `delta - prior` for every field.
    #[must_use]
    pub fn delta(&self, prior: &Totals) -> Totals {
        Totals {
            T: self.T - prior.T,
            M: self.M - prior.M,
            E: self.E - prior.E,
            F: self.F - prior.F,
            P: self.P - prior.P,
            B: self.B - prior.B,
        }
    }
}

/// Report from one [`PhysicsFields::evolve`] step. Captures the conservation
/// budget (`leakage = |delta totals|`) and the dimensionless ratios from
/// spec §4.2 (α, β, γ) so the `civ-engine` invariant can trip the
/// "Class-4 zone" tripwire when they leave the recommended band.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EvolveReport {
    /// Per-field conservation deltas.
    pub conservation: Totals,
    /// Diffusivity ratio α = D_T / (D_M + D_E). Spec target: `[0.3, 2.0]`.
    /// `0.0` means "no operator wired yet" (Sprint 1 state).
    pub alpha: f32,
    /// Write-rate ratio β = max layer write rate / substrate sink rate.
    /// Spec target: `[0.05, 0.5]`. `0.0` in Sprint 1.
    pub beta: f32,
    /// Memory ratio γ = reaction timescale / tick. Spec target: `[0.1, 10]`.
    /// `0.0` in Sprint 1.
    pub gamma: f32,
}

impl EvolveReport {
    /// True iff α, β, γ are all inside the spec §4.2 Class-4 zone.
    /// Always `false` in Sprint 1 (all three ratios are `0.0`).
    #[must_use]
    pub fn in_class4_zone(&self) -> bool {
        const ALPHA_LO: f32 = 0.3;
        const ALPHA_HI: f32 = 2.0;
        const BETA_LO: f32 = 0.05;
        const BETA_HI: f32 = 0.5;
        const GAMMA_LO: f32 = 0.1;
        const GAMMA_HI: f32 = 10.0;
        self.alpha >= ALPHA_LO
            && self.alpha <= ALPHA_HI
            && self.beta >= BETA_LO
            && self.beta <= BETA_HI
            && self.gamma >= GAMMA_LO
            && self.gamma <= GAMMA_HI
    }
}

impl PhysicsFields {
    /// Run one tick of substrate evolution.
    ///
    /// **Sprint 1 (this PR)**: identity. `Totals` are unchanged; `alpha`,
    /// `beta`, `gamma` all report `0.0`. No conservation violation is
    /// possible (delta = 0). The wiring is here so the integration site
    /// (`phase_substrate`) can be added to `Simulation::tick` and the
    /// invariant scaffolded without changing observable behavior.
    ///
    /// **Sprints 2–4** (per spec §7.2–§7.4): replace the body with
    /// operator-split stages (advection, diffusion, reaction, regrowth,
    /// dispersal) plus typed, rate-limited setters.
    #[must_use]
    pub fn evolve(&mut self, _dt: f32) -> EvolveReport {
        let totals_before = self.totals();
        // Identity: no field is written.
        let totals_after = self.totals();
        EvolveReport {
            conservation: totals_after.delta(&totals_before),
            alpha: 0.0,
            beta: 0.0,
            gamma: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_descriptor_rejects_zero_axes() {
        assert!(GridDescriptor::new(0, 4, 4).is_none());
        assert!(GridDescriptor::new(4, 0, 4).is_none());
        assert!(GridDescriptor::new(4, 4, 0).is_none());
        assert!(GridDescriptor::new(4, 4, 4).is_some());
    }

    #[test]
    fn grid_descriptor_indexing_is_deterministic() {
        let g = GridDescriptor::new(2, 3, 4).unwrap();
        assert_eq!(g.cell_count(), 24);
        // x-fastest, then y, then z
        assert_eq!(g.index(0, 0, 0), Some(0));
        assert_eq!(g.index(1, 0, 0), Some(12));
        assert_eq!(g.index(0, 1, 0), Some(4));
        assert_eq!(g.index(0, 0, 1), Some(1));
    }

    #[test]
    fn grid_descriptor_out_of_bounds_is_none() {
        let g = GridDescriptor::new(2, 3, 4).unwrap();
        assert!(g.index(2, 0, 0).is_none());
        assert!(g.index(0, 3, 0).is_none());
        assert!(g.index(0, 0, 4).is_none());
    }

    #[test]
    fn field_all_returns_six_in_order() {
        assert_eq!(Field::ALL.len(), 6);
        assert_eq!(Field::ALL[0] as u8, 0);
        assert_eq!(Field::ALL[5] as u8, 5);
        // Names match spec table column headers.
        assert_eq!(Field::T.as_str(), "T");
        assert_eq!(Field::M.as_str(), "M");
        assert_eq!(Field::E.as_str(), "E");
        assert_eq!(Field::F.as_str(), "F");
        assert_eq!(Field::P.as_str(), "P");
        assert_eq!(Field::B.as_str(), "B");
    }

    #[test]
    fn fields_init_zeroes_all_six() {
        let g = GridDescriptor::new(2, 2, 2).unwrap();
        let f = PhysicsFields::with_init(g, 0.0);
        for field in Field::ALL {
            assert!(f.get(field).iter().all(|&v| v == 0.0));
        }
    }

    #[test]
    fn fields_set_writes_via_typed_setter() {
        let g = GridDescriptor::new(2, 2, 2).unwrap();
        let mut f = PhysicsFields::with_init(g, 0.0);
        f.set(Field::T, 1, 1, 1, 273.15);
        // Only the targeted cell changed.
        assert_eq!(f.T[f.grid.index(1, 1, 1).unwrap()], 273.15);
        assert_eq!(f.T.iter().filter(|&&v| v != 0.0).count(), 1);
        // Out-of-bounds writes are silently dropped (no panic).
        f.set(Field::T, 99, 0, 0, 999.0);
        assert!(f.T.iter().all(|&v| v < 999.0));
    }

    #[test]
    fn evolve_sprint1_is_identity_and_reports_zero_deltas() {
        let g = GridDescriptor::new(2, 2, 2).unwrap();
        let mut f = PhysicsFields::with_init(g, 1.0);
        let report = f.evolve(0.1);
        // Identity: no field changes.
        for field in Field::ALL {
            assert_eq!(report.conservation.get(field), 0.0);
            assert!(f.get(field).iter().all(|&v| v == 1.0));
        }
        // Spec §4.2 ratios are 0.0 in Sprint 1 (no operator wired).
        assert_eq!(report.alpha, 0.0);
        assert_eq!(report.beta, 0.0);
        assert_eq!(report.gamma, 0.0);
        // Class-4 zone is therefore NOT satisfied — the invariant should
        // trip in the next sprint when real operators land.
        assert!(!report.in_class4_zone());
    }

    #[test]
    fn totals_delta_is_subtraction_per_field() {
        let g = GridDescriptor::new(2, 2, 2).unwrap();
        let mut f = PhysicsFields::with_init(g.clone(), 0.0);
        let before = f.totals();
        f.T.fill(2.0);
        f.M.fill(0.5);
        let after = f.totals();
        let d = after.delta(&before);
        assert_eq!(d.T, 16.0); // 8 cells * 2.0
        assert_eq!(d.M, 4.0); // 8 cells * 0.5
        assert_eq!(d.E, 0.0);
        assert_eq!(d.F, 0.0);
        assert_eq!(d.P, 0.0);
        assert_eq!(d.B, 0.0);
    }
}
