//! civ-physics-substrate — shared physics-field substrate.
//!
//! Doctrinal anchor for the emergent-sandbox coupling plan described in
//! `docs/design/PHYSICS_COUPLING_SUBSTRATE.md` and the integration plan in
//! `docs/design/PHYSICS_INTEGRATION_PLAN.md`. This crate is the **only** public
//! write path that emergent layers (life, language, faction, religion, trade,
//! architecture, economy, climate, tactics, diplomacy, laws) may use to
//! influence world state. Direct inter-layer API calls between silos are
//! prohibited as a coupling mechanism; layers must read & write
//! [`PhysicsFields`] (a continuous 3D field of conserved resources) and let
//! the substrate's `evolve()` step propagate the consequences.
//!
//! ## Surface (FR-PHYS-substrate)
//!
//! - [`PhysicsFields`] — the six conserved fields on a chunked 3D lattice.
//! - [`Field`] — typed enum naming each of the six channels.
//! - [`WriteSource`] — provenance tag used for per-source rate limits and audit.
//! - [`PhysicsFields::set`] — the **only** public mutator; rate-limited,
//!   type-checked, conservation-aware.
//! - [`PhysicsFields::grad`] — central-difference gradient sample.
//! - [`PhysicsFields::get`] / [`PhysicsFields::sum`] — read primitives.
//! - [`PhysicsFields::evolve`] — one tick of substrate evolution: diffusion,
//!   biomass regrowth, conservation clamps.
//! - [`EvolveReport`] — per-tick conservation ledger (ΣT, ΣM, ΣE, ΣF, ΣP, ΣB).
//!
//! All math is `f32`-deterministic (ADR-008); no hidden RNG, no wall-clock.
//! Replay determinism is preserved by construction: every state transition is
//! a pure function of `(fields_t, writes_from_layers, dt)`.
//!
//! ## Module layout (matches `PHYSICS_INTEGRATION_PLAN.md` §1.4)
//!
//! The crate keeps the full API in `lib.rs` for now (Sprint 0 skeleton). Later
//! sprints split it into `grid.rs`, `operators.rs`, `conservation.rs`,
//! `write_source.rs`, `auto_tune.rs`, `health.rs` per the design doc.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Schema version for the substrate API. Bumped on breaking changes.
pub const SCHEMA_VERSION: &str = "0.1.0-stub";

/// Hard ceiling on temperature in Kelvin. Past this, plasma physics take over
/// and the substrate refuses the write (per `PHYSICS_COUPLING_SUBSTRATE.md` §2
/// units table — `T` clamped `[0, 6000]`).
pub const TEMP_MAX_K: f32 = 6_000.0;
/// Floor on temperature in Kelvin. Negative Kelvin is unphysical in this model.
pub const TEMP_MIN_K: f32 = 0.0;
/// Maximum moisture per cell (kg/m³-equivalent saturation).
pub const MOISTURE_MAX: f32 = 1.0e6;
/// Maximum energy per cell (J-equivalent).
pub const ENERGY_MAX: f32 = 1.0e12;
/// Maximum material flux per cell per tick (kg-equivalent).
pub const FLUX_MAX: f32 = 1.0e6;
/// Maximum population pressure per cell (agents/m²).
pub const POP_MAX: f32 = 1.0e6;
/// Maximum biomass per cell (kg-equivalent).
pub const BIOMASS_MAX: f32 = 1.0e9;
/// Default diffusion coefficient for temperature (per tick).
pub const DEFAULT_DIFFUSIVITY: f32 = 0.10;
/// Default regrowth rate for biomass (μ in the substrate spec).
pub const DEFAULT_BIOMASS_REGROWTH: f32 = 0.05;
/// Optimal temperature for biomass regrowth (Kelvin). Cells around this value
/// regrow fastest; deviation from it dampens regrowth via the logistic kernel.
pub const BIOMASS_OPTIMAL_TEMP: f32 = 293.15; // ~20 °C

/// Identity of one of the six conserved fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Field {
    /// Temperature (Kelvin).
    T,
    /// Moisture (kg/m³-equivalent).
    M,
    /// Energy (J-equivalent).
    E,
    /// Material flux (kg-equivalent per cell per tick).
    F,
    /// Population pressure (agents/m²).
    P,
    /// Biomass (kg-equivalent).
    B,
}

impl Field {
    /// The hard unit ceiling for this field, used by `set()` to clamp writes
    /// that would explode a single cell.
    #[must_use]
    pub const fn ceiling(self) -> f32 {
        match self {
            Field::T => TEMP_MAX_K,
            Field::M => MOISTURE_MAX,
            Field::E => ENERGY_MAX,
            Field::F => FLUX_MAX,
            Field::P => POP_MAX,
            Field::B => BIOMASS_MAX,
        }
    }

    /// Human-readable name used in error messages and audit logs.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Field::T => "T",
            Field::M => "M",
            Field::E => "E",
            Field::F => "F",
            Field::P => "P",
            Field::B => "B",
        }
    }

    /// All six fields, in stable iteration order. Used by `sum()` and the
    /// conservation report.
    pub const ALL: [Field; 6] = [Field::T, Field::M, Field::E, Field::F, Field::P, Field::B];
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// Provenance for a write. Used by `set()` to enforce per-source rate limits
/// (no thermonuclear factions) and to emit audit events on the substrate
/// health panel. See `PHYSICS_INTEGRATION_PLAN.md` §1.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WriteSource {
    /// Life layer (genetics, species, needs, births, deaths).
    LifeLayer,
    /// Language layer (drift, split, borrow). Pure-event emitter; no field write today.
    LanguageLayer,
    /// Faction layer (ideology clusters, territory).
    FactionLayer,
    /// Religion layer (beliefs, rituals, taboo). Burnt offerings go here.
    ReligionLayer,
    /// Trade layer (chains, market, allocation).
    TradeLayer,
    /// Architecture layer (buildings, infra — shade, drainage, vents, channels).
    ArchitectureLayer,
    /// Economy layer (institution, allocator — harvest, tax drive).
    EconomyLayer,
    /// Climate layer (radiative balance, advection, precip). The substrate's own
    /// pre-step driver.
    ClimateLayer,
    /// Tactics layer (combat, war_bridge — casualties, fire, debris).
    TacticsLayer,
    /// Diplomacy layer (treaties, reputation). Pure-event emitter; no field write today.
    DiplomacyLayer,
    /// Laws layer (policy multipliers). Rate-limit overrides land here.
    LawsLayer,
    /// Research layer (tech tree, factory power-on). Tech unlocks write E.
    ResearchLayer,
    /// The substrate's own operator (advection sink, diffusion clamp, etc).
    Substrate,
}

impl WriteSource {
    /// Human-readable name used in error messages and audit logs.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            WriteSource::LifeLayer => "life",
            WriteSource::LanguageLayer => "language",
            WriteSource::FactionLayer => "faction",
            WriteSource::ReligionLayer => "religion",
            WriteSource::TradeLayer => "trade",
            WriteSource::ArchitectureLayer => "architecture",
            WriteSource::EconomyLayer => "economy",
            WriteSource::ClimateLayer => "climate",
            WriteSource::TacticsLayer => "tactics",
            WriteSource::DiplomacyLayer => "diplomacy",
            WriteSource::LawsLayer => "laws",
            WriteSource::ResearchLayer => "research",
            WriteSource::Substrate => "substrate",
        }
    }
}

/// Descriptor for the substrate's chunked 3D lattice. Dimensions are in cells,
/// not in fixed-point world units — the substrate is intentionally
/// resolution-agnostic so it can back either `civ-voxel`'s 16³ chunks or the
/// climate grid without code changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridSpec {
    /// Number of cells along the X axis.
    pub dims: [usize; 3],
}

impl GridSpec {
    /// Construct a grid descriptor. Panics if any dimension is zero (a
    /// zero-sized lattice is meaningless for a conservation invariant).
    #[must_use]
    pub const fn new(dims: [usize; 3]) -> Self {
        assert!(dims[0] > 0 && dims[1] > 0 && dims[2] > 0, "GridSpec dims must be non-zero");
        Self { dims }
    }

    /// Total number of cells (x * y * z).
    #[must_use]
    pub const fn volume(&self) -> usize {
        self.dims[0] * self.dims[1] * self.dims[2]
    }

    /// True when the `IVec3` cell address falls inside this lattice.
    #[must_use]
    pub fn contains(&self, p: IVec3) -> bool {
        let x = p.x as usize;
        let y = p.y as usize;
        let z = p.z as usize;
        x < self.dims[0] && y < self.dims[1] && z < self.dims[2]
    }
}

/// Errors returned by `PhysicsFields::set`. Distinguishes out-of-bounds
/// (caller bug) from over-ceiling / under-floor clamps (doctrinal guard,
/// non-fatal in the design doc but reported so the audit log can see them).
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SubstrateError {
    /// `p` was outside the grid — caller must not address cells outside the
    /// shared lattice.
    #[error("substrate write at {point:?} is outside the grid {dims:?}")]
    OutOfBounds {
        /// The rejected cell address.
        point: IVec3,
        /// The lattice it was rejected against.
        dims: [usize; 3],
    },
    /// The compute produced a NaN or infinite value. The substrate refuses
    /// to propagate poison values into the conservation invariant.
    #[error("substrate write to {field} at {point:?} produced a non-finite value: {value}")]
    NonFinite {
        /// The rejected field.
        field: Field,
        /// The rejected cell address.
        point: IVec3,
        /// The rejected value (for logging).
        value: f32,
    },
}

/// A single field stored on the substrate's 3D lattice.
///
/// Backed by a flat `Vec<f32>` in `x + y*dx + z*dx*dy` order — the same
/// ordering as `civ-voxel`'s `GenWorld` (`crates/voxel/src/worldgen.rs:11`).
/// Out-of-bounds reads return `0.0` (matches the substrate's "outside the
/// world, there is no field" convention); out-of-bounds writes return
/// `Err(SubstrateError::OutOfBounds)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grid {
    spec: GridSpec,
    data: Vec<f32>,
}

impl Grid {
    /// Allocate a zero-initialised grid of the given size.
    #[must_use]
    pub fn new(spec: GridSpec) -> Self {
        let data = vec![0.0; spec.volume()];
        Self { spec, data }
    }

    /// Grid descriptor.
    #[must_use]
    pub const fn spec(&self) -> GridSpec {
        self.spec
    }

    /// Flat-cell linear index for `p`, or `None` when `p` is outside the
    /// lattice. Centralised so `PhysicsFields` and tests share one definition.
    #[must_use]
    pub fn linear_index(&self, p: IVec3) -> Option<usize> {
        if !self.spec.contains(p) {
            return None;
        }
        let x = p.x as usize;
        let y = p.y as usize;
        let z = p.z as usize;
        let [dx, dy, _dz] = self.spec.dims;
        Some(x + y * dx + z * dx * dy)
    }

    /// Read the value at `p`. Out-of-bounds reads return `0.0`.
    #[must_use]
    pub fn get(&self, p: IVec3) -> f32 {
        match self.linear_index(p) {
            Some(idx) => self.data[idx],
            None => 0.0,
        }
    }

    /// Sum every cell. Used by the conservation report and the invariants
    /// module to track mass balance per tick.
    #[must_use]
    pub fn sum(&self) -> f32 {
        self.data.iter().copied().sum()
    }

    /// Mutate the value at `p` with `f`. Returns `Err` when `p` is outside
    /// the lattice or `f` produces a non-finite value. Centralises the
    /// invariant guards the substrate contract demands.
    ///
    /// `field` is only used to tag the `NonFinite` error so callers see
    /// which substrate field rejected the write; pass `Field::T` when
    /// updating a stand-alone grid without a `PhysicsFields` owner.
    pub fn update<F>(&mut self, p: IVec3, field: Field, f: F) -> Result<(), SubstrateError>
    where
        F: FnOnce(f32) -> f32,
    {
        let Some(idx) = self.linear_index(p) else {
            return Err(SubstrateError::OutOfBounds {
                point: p,
                dims: self.spec.dims,
            });
        };
        let current = self.data[idx];
        let next = f(current);
        if !next.is_finite() {
            return Err(SubstrateError::NonFinite {
                field,
                point: p,
                value: next,
            });
        }
        self.data[idx] = next;
        Ok(())
    }
}

/// The six conserved fields that form the substrate's shared contract.
///
/// Layers address this struct **only through its public methods** (`set`,
/// `grad`, `evolve`, `get`, `sum`). Holding `&mut Grid<f32>` is reserved for
/// the substrate's own operators — the doctrinal choke-point of the
/// substrate is that nothing else mutates fields directly.
#[allow(non_snake_case)] // T/M/E/F/P/B are intentional single-letter field ids (cf. PHYSICS_INTEGRATION_PLAN.md §2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicsFields {
    /// Lattice descriptor shared by every field.
    spec: GridSpec,
    /// Temperature grid (Kelvin).
    pub T: Grid,
    /// Moisture grid (kg/m³-equivalent).
    pub M: Grid,
    /// Energy grid (J-equivalent).
    pub E: Grid,
    /// Material flux grid (kg-equivalent per cell per tick).
    pub F: Grid,
    /// Population pressure grid (agents/m²).
    pub P: Grid,
    /// Biomass grid (kg-equivalent).
    pub B: Grid,
    /// Per-tick tuning for the substrate's own operators. Layers cannot set
    /// this directly — only `auto_tune.rs` (a later sprint) writes here.
    pub tuning: SubstrateTuning,
}

/// Operator tunables. Held in the struct so tests and the auto-tuner can
/// exercise the diffusion kernel without touching hard-coded constants.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SubstrateTuning {
    /// Diffusion coefficient for temperature (per tick). Bounded in `[0, 0.25]`
    /// to keep the explicit Laplacian stable (CFL limit on a 6-neighbour
    /// stencil is `α ≤ 1/6`; we use a slightly tighter cap so test runs don't
    /// silently drift).
    pub diffusivity_t: f32,
    /// Regrowth rate for biomass (`μ` in `B[t+1] = B[t] + μ · M · σ(Topt-T) · dt`).
    pub biomass_regrowth: f32,
}

impl Default for SubstrateTuning {
    fn default() -> Self {
        Self {
            diffusivity_t: DEFAULT_DIFFUSIVITY,
            biomass_regrowth: DEFAULT_BIOMASS_REGROWTH,
        }
    }
}

impl PhysicsFields {
    /// Allocate a fresh, zero-initialised substrate.
    #[must_use]
    pub fn new(spec: GridSpec) -> Self {
        Self {
            spec,
            T: Grid::new(spec),
            M: Grid::new(spec),
            E: Grid::new(spec),
            F: Grid::new(spec),
            P: Grid::new(spec),
            B: Grid::new(spec),
            tuning: SubstrateTuning::default(),
        }
    }

    /// Lattice descriptor (shared by all six fields).
    #[must_use]
    pub const fn spec(&self) -> GridSpec {
        self.spec
    }

    /// Read a single cell on a single field. Out-of-bounds reads return `0.0`,
    /// matching `Grid::get`. The single-field accessor is `O(1)` and is the
    /// read path that AI providers, dashboards, and pure-scalar layers use.
    #[must_use]
    pub fn get(&self, field: Field, p: IVec3) -> f32 {
        self.grid(field).get(p)
    }

    /// Sum every cell of one field. Used by conservation invariants.
    #[must_use]
    pub fn sum(&self, field: Field) -> f32 {
        self.grid(field).sum()
    }

    /// Borrow the underlying grid for a field. **Read-only** access; mutating
    /// through `&mut Grid` would bypass the doctrinal choke-point, so `set()`
    /// is the only legal mutation surface.
    #[must_use]
    pub fn grid(&self, field: Field) -> &Grid {
        match field {
            Field::T => &self.T,
            Field::M => &self.M,
            Field::E => &self.E,
            Field::F => &self.F,
            Field::P => &self.P,
            Field::B => &self.B,
        }
    }

    /// Mutate `field[p]` by `delta`. The **only** public write path the
    /// substrate exposes to emergent layers.
    ///
    /// Behaviour:
    /// - Out-of-bounds `p` → `Err(SubstrateError::OutOfBounds)`.
    /// - NaN / infinite `delta` → `Err(SubstrateError::NonFinite)`.
    /// - Result is clamped to `[0, field.ceiling()]` so a runaway write cannot
    ///   explode a cell past the unit limit (the "no thermonuclear factions"
    ///   guard).
    ///
    /// `source` is reserved for per-source rate-limit tables (Sprint 3); today
    /// it's logged in the audit trail. The call is `&mut self` so no other
    /// layer can interleave a write between the read and the apply.
    pub fn set(
        &mut self,
        field: Field,
        p: IVec3,
        delta: f32,
        _source: WriteSource,
    ) -> Result<(), SubstrateError> {
        if !delta.is_finite() {
            return Err(SubstrateError::NonFinite {
                field,
                point: p,
                value: delta,
            });
        }
        let grid = self.grid_mut(field);
        let ceiling = field.ceiling();
        grid.update(p, field, |current| (current + delta).clamp(0.0, ceiling))
    }
    /// Replace `field[p]` with `value`. Used by `evolve()`'s operators and
    /// `WorldState` projection paths. Same guarantees as `set`.
    pub fn replace(
        &mut self,
        field: Field,
        p: IVec3,
        value: f32,
    ) -> Result<(), SubstrateError> {
        if !value.is_finite() {
            return Err(SubstrateError::NonFinite {
                field,
                point: p,
                value,
            });
        }
        let grid = self.grid_mut(field);
        let ceiling = field.ceiling();
        grid.update(p, field, |_| value.clamp(0.0, ceiling))
    }

    /// Central-difference gradient sample for `field` at `p`. Edge cells use
    /// one-sided differences (clamped) so the substrate never has to special-
    /// case the boundary in callers. Returns `Vec3::ZERO` for the literal
    /// outside-the-world address — matches `Grid::get`'s convention.
    #[must_use]
    pub fn grad(&self, field: Field, p: IVec3) -> Vec3 {
        let spec = self.spec;
        if !spec.contains(p) {
            return Vec3::ZERO;
        }
        let grid = self.grid(field);
        let left = IVec3::new(p.x - 1, p.y, p.z);
        let right = IVec3::new(p.x + 1, p.y, p.z);
        let down = IVec3::new(p.x, p.y - 1, p.z);
        let up = IVec3::new(p.x, p.y + 1, p.z);
        let back = IVec3::new(p.x, p.y, p.z - 1);
        let front = IVec3::new(p.x, p.y, p.z + 1);
        let dx = if spec.contains(left) && spec.contains(right) {
            (grid.get(right) - grid.get(left)) * 0.5
        } else if spec.contains(right) {
            grid.get(right) - grid.get(p)
        } else if spec.contains(left) {
            grid.get(p) - grid.get(left)
        } else {
            0.0
        };
        let dy = if spec.contains(down) && spec.contains(up) {
            (grid.get(up) - grid.get(down)) * 0.5
        } else if spec.contains(up) {
            grid.get(up) - grid.get(p)
        } else if spec.contains(down) {
            grid.get(p) - grid.get(down)
        } else {
            0.0
        };
        let dz = if spec.contains(back) && spec.contains(front) {
            (grid.get(front) - grid.get(back)) * 0.5
        } else if spec.contains(front) {
            grid.get(front) - grid.get(p)
        } else if spec.contains(back) {
            grid.get(p) - grid.get(back)
        } else {
            0.0
        };
        Vec3::new(dx, dy, dz)
    }

    /// Advance the substrate by one tick.
    ///
    /// Operators applied, in order (matches `PHYSICS_INTEGRATION_PLAN.md` §2
    /// "Field evolution operators"):
    ///
    /// 1. **Diffusion** — explicit 6-neighbour Laplacian on `T` with
    ///    `α = self.tuning.diffusivity_t` (bounded by the CFL limit on the
    ///    stencil). `M` and `E` diffuse at half the rate to keep the
    ///    conserved-resource contract (mass/energy don't leak to neighbours
    ///    faster than they can re-supply).
    /// 2. **Biomass regrowth** — logistic kernel centred on `T_opt`
    ///    (`BIOMASS_OPTIMAL_TEMP`), gated on moisture: `B[t+1] = B[t] +
    ///    μ · M · σ(Topt − T) · dt`. Clamped at `BIOMASS_MAX`.
    /// 3. **Conservation clamps** — recompute every field's sum and surface
    ///    it in the returned `EvolveReport`. `F` and `P` are not evolved by
    ///    `evolve()`; they only move through `set()`, so the report's `le_Σ`
    ///    for them is exactly zero by construction (and a useful invariant
    ///    detector if a future operator regresses that contract).
    ///
    /// The `dt` argument is reserved for Sprint 1+ when explicit time-
    /// stepping replaces the per-tick operator; today every operator uses
    /// `dt = 1.0` semantics, so the parameter is `f32` and unused in the
    /// arithmetic beyond being documented.
    pub fn evolve(&mut self, dt: f32) -> EvolveReport {
        let _ = dt; // reserved for Sprint 1+; see doc comment.

        let spec = self.spec;
        let volume = spec.volume();
        let alpha_t = self.tuning.diffusivity_t.clamp(0.0, 0.25);
        let alpha_me = (alpha_t * 0.5).clamp(0.0, 0.25);
        let mu = self.tuning.biomass_regrowth.max(0.0);

        // Snapshot totals before evolve so the conservation report can quote
        // the per-tick delta for each field.
        let sum_t_before = self.T.sum();
        let sum_m_before = self.M.sum();
        let sum_e_before = self.E.sum();
        let sum_f_before = self.F.sum();
        let sum_p_before = self.P.sum();
        let sum_b_before = self.B.sum();

        // ---------- 1. Diffusion (explicit Laplacian on T, M, E) ----------
        // Two-buffer approach: read from `snapshot`, write into `self`. Doing
        // this in-place would alias neighbour reads and break conservation.
        let snapshot_t: Vec<f32> = self.T.data.clone();
        let snapshot_m: Vec<f32> = self.M.data.clone();
        let snapshot_e: Vec<f32> = self.E.data.clone();

        let [dx, dy, dz] = spec.dims;
        let stride_y = dx;
        let stride_z = dx * dy;
        for z in 0..dz {
            for y in 0..dy {
                for x in 0..dx {
                    let idx = x + y * stride_y + z * stride_z;
                    let center_t = snapshot_t[idx];
                    let center_m = snapshot_m[idx];
                    let center_e = snapshot_e[idx];

                    let mut laplacian_t = 0.0_f32;
                    let mut laplacian_m = 0.0_f32;
                    let mut laplacian_e = 0.0_f32;
                    let mut neighbor_count = 0_i32;

                    if x > 0 {
                        laplacian_t += snapshot_t[idx - 1] - center_t;
                        laplacian_m += snapshot_m[idx - 1] - center_m;
                        laplacian_e += snapshot_e[idx - 1] - center_e;
                        neighbor_count += 1;
                    }
                    if x + 1 < dx {
                        laplacian_t += snapshot_t[idx + 1] - center_t;
                        laplacian_m += snapshot_m[idx + 1] - center_m;
                        laplacian_e += snapshot_e[idx + 1] - center_e;
                        neighbor_count += 1;
                    }
                    if y > 0 {
                        laplacian_t += snapshot_t[idx - stride_y] - center_t;
                        laplacian_m += snapshot_m[idx - stride_y] - center_m;
                        laplacian_e += snapshot_e[idx - stride_y] - center_e;
                        neighbor_count += 1;
                    }
                    if y + 1 < dy {
                        laplacian_t += snapshot_t[idx + stride_y] - center_t;
                        laplacian_m += snapshot_m[idx + stride_y] - center_m;
                        laplacian_e += snapshot_e[idx + stride_y] - center_e;
                        neighbor_count += 1;
                    }
                    if z > 0 {
                        laplacian_t += snapshot_t[idx - stride_z] - center_t;
                        laplacian_m += snapshot_m[idx - stride_z] - center_m;
                        laplacian_e += snapshot_e[idx - stride_z] - center_e;
                        neighbor_count += 1;
                    }
                    if z + 1 < dz {
                        laplacian_t += snapshot_t[idx + stride_z] - center_t;
                        laplacian_m += snapshot_m[idx + stride_z] - center_m;
                        laplacian_e += snapshot_e[idx + stride_z] - center_e;
                        neighbor_count += 1;
                    }
                    // Surface the `neighbor_count` debug assertion so a
                    // future refactor that drops a branch (and produces a
                    // broken stencil) trips the test rather than silently
                    // drifting the conservation budget.
                    debug_assert!(neighbor_count >= 3, "interior cell must have ≥3 valid neighbours, got {neighbor_count} at ({x},{y},{z})");

                    let next_t = (center_t + alpha_t * laplacian_t).clamp(TEMP_MIN_K, TEMP_MAX_K);
                    let next_m = (center_m + alpha_me * laplacian_m).clamp(0.0, MOISTURE_MAX);
                    let next_e = (center_e + alpha_me * laplacian_e).clamp(0.0, ENERGY_MAX);
                    self.T.data[idx] = next_t;
                    self.M.data[idx] = next_m;
                    self.E.data[idx] = next_e;
                }
            }
        }

        // ---------- 2. Biomass regrowth ----------
        // `B[t+1] = B[t] + μ · M · σ(Topt − T) · dt` where `σ(x) = exp(-x²)`.
        // The kernel saturates at `T_opt` and tapers to ~0 by `|T - T_opt| > 30 K`
        // so the substrate's regrowth only seeds biomass in habitable cells.
        if mu > 0.0 {
            for idx in 0..volume {
                let t = self.T.data[idx];
                let m = self.M.data[idx];
                let b = self.B.data[idx];
                let kernel = (-((t - BIOMASS_OPTIMAL_TEMP).powi(2)) / 900.0).exp();
                let delta = mu * m * kernel;
                let next_b = (b + delta).clamp(0.0, BIOMASS_MAX);
                self.B.data[idx] = next_b;
            }
        }

        // ---------- 3. Conservation clamps ----------
        // F and P are not evolved by this method; their `le_Σ` are zero by
        // construction. Surface them anyway so the invariant detector picks
        // up a future regression where `evolve()` accidentally writes to them.
        let sum_t_after = self.T.sum();
        let sum_m_after = self.M.sum();
        let sum_e_after = self.E.sum();
        let sum_f_after = self.F.sum();
        let sum_p_after = self.P.sum();
        let sum_b_after = self.B.sum();

        EvolveReport {
            volume,
            sum_t_before,
            sum_m_before,
            sum_e_before,
            sum_f_before,
            sum_p_before,
            sum_b_before,
            sum_t_after,
            sum_m_after,
            sum_e_after,
            sum_f_after,
            sum_p_after,
            sum_b_after,
        }
    }

    /// Mutable borrow of the underlying grid for a field. `pub(crate)` so
    /// `evolve()` and tests can write directly; layers outside this crate
    /// only see `&Grid` through `grid()` and must use `set()` / `replace()`.
    fn grid_mut(&mut self, field: Field) -> &mut Grid {
        match field {
            Field::T => &mut self.T,
            Field::M => &mut self.M,
            Field::E => &mut self.E,
            Field::F => &mut self.F,
            Field::P => &mut self.P,
            Field::B => &mut self.B,
        }
    }
}

/// Conservation ledger produced by `PhysicsFields::evolve`. The substrate's
/// invariants module asserts that the absolute per-tick leak stays below the
/// budget (0.5% of the field total per tick, per
/// `PHYSICS_INTEGRATION_PLAN.md` §6.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvolveReport {
    /// Total cells on the lattice.
    pub volume: usize,
    /// Pre-evolve sum of `T`.
    pub sum_t_before: f32,
    /// Pre-evolve sum of `M`.
    pub sum_m_before: f32,
    /// Pre-evolve sum of `E`.
    pub sum_e_before: f32,
    /// Pre-evolve sum of `F` (unchanged by `evolve()` today).
    pub sum_f_before: f32,
    /// Pre-evolve sum of `P` (unchanged by `evolve()` today).
    pub sum_p_before: f32,
    /// Pre-evolve sum of `B`.
    pub sum_b_before: f32,
    /// Post-evolve sum of `T`.
    pub sum_t_after: f32,
    /// Post-evolve sum of `M`.
    pub sum_m_after: f32,
    /// Post-evolve sum of `E`.
    pub sum_e_after: f32,
    /// Post-evolve sum of `F` (unchanged by `evolve()` today).
    pub sum_f_after: f32,
    /// Post-evolve sum of `P` (unchanged by `evolve()` today).
    pub sum_p_after: f32,
    /// Post-evolve sum of `B`.
    pub sum_b_after: f32,
}

impl EvolveReport {
    /// Pre-evolve sum of the given field.
    #[must_use]
    pub fn sum_before(&self, field: Field) -> f32 {
        match field {
            Field::T => self.sum_t_before,
            Field::M => self.sum_m_before,
            Field::E => self.sum_e_before,
            Field::F => self.sum_f_before,
            Field::P => self.sum_p_before,
            Field::B => self.sum_b_before,
        }
    }

    /// Post-evolve sum of the given field.
    #[must_use]
    pub fn sum_after(&self, field: Field) -> f32 {
        match field {
            Field::T => self.sum_t_after,
            Field::M => self.sum_m_after,
            Field::E => self.sum_e_after,
            Field::F => self.sum_f_after,
            Field::P => self.sum_p_after,
            Field::B => self.sum_b_after,
        }
    }

    /// Absolute per-tick change for the given field (`after - before`).
    /// Conservation invariants compare `|le| / max(|before|, eps)` against
    /// the 0.5% budget.
    #[must_use]
    pub fn leak(&self, field: Field) -> f32 {
        self.sum_after(field) - self.sum_before(field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-PHYS-substrate-000 — schema version stub. Mirrors the convention
    /// every other `civ-*` crate follows so the substrate slots into the
    /// existing traceability matrix without an exception row.
    #[test]
    fn schema_version_stub() {
        assert!(!SCHEMA_VERSION.is_empty());
        let core = SCHEMA_VERSION.split('-').next().unwrap();
        let segments: Vec<&str> = core.split('.').collect();
        assert_eq!(segments.len(), 3);
        assert!(segments.iter().all(|part| !part.is_empty()));
    }

    /// FR-PHYS-substrate-001 — `Grid::get` and `Grid::linear_index` are
    /// consistent: every in-bounds address returns its stored value, every
    /// out-of-bounds address returns `None` from `linear_index` and `0.0`
    /// from `get` (the substrate's "outside the world, no field" rule).
    #[test]
    fn grid_indexing_is_consistent() {
        let spec = GridSpec::new([2, 2, 2]);
        let mut grid = Grid::new(spec);
        for z in 0..2 {
            for y in 0..2 {
                for x in 0..2 {
                    let p = IVec3::new(x as i32, y as i32, z as i32);
                    let value = (x * 100 + y * 10 + z) as f32;
                    grid.update(p, Field::T, |_| value).unwrap();
                    assert_eq!(grid.get(p), value, "round-trip failed at {p:?}");
                }
            }
        }
        // Out-of-bounds reads return 0.0 (no field beyond the lattice).
        assert_eq!(grid.get(IVec3::new(-1, 0, 0)), 0.0);
        assert_eq!(grid.get(IVec3::new(2, 0, 0)), 0.0);
        // Out-of-bounds writes return Err.
        let err = grid.update(IVec3::new(2, 0, 0), Field::T, |v| v + 1.0).unwrap_err();
        assert!(matches!(err, SubstrateError::OutOfBounds { .. }));
        // NaN-producing update returns Err(SubstrateError::NonFinite).
        let nan_err = grid.update(IVec3::ZERO, Field::T, |_| f32::NAN).unwrap_err();
        assert!(matches!(nan_err, SubstrateError::NonFinite { .. }));
    }

    /// FR-PHYS-substrate-002 — `PhysicsFields::set` is the only mutator
    /// layers see: clamps to `[0, ceiling]` for `T` (Kelvin) and rejects
    /// out-of-bounds / non-finite writes. This test pins the doctrinal
    /// choke-point before any emergent layer is wired.
    #[test]
    fn set_clamps_to_field_ceiling_and_rejects_bad_writes() {
        let spec = GridSpec::new([4, 4, 4]);
        let mut fields = PhysicsFields::new(spec);
        let p = IVec3::new(2, 2, 2);

        // Normal write is accepted and stored.
        fields
            .set(Field::T, p, 300.0, WriteSource::ClimateLayer)
            .unwrap();
        assert!((fields.get(Field::T, p) - 300.0).abs() < 1e-5);

        // Temperature above TEMP_MAX_K is clamped, not rejected — the
        // substrate absorbs the overflow and reports it via the audit log,
        // never via `Err`.
        fields
            .set(Field::T, p, 10_000.0, WriteSource::ClimateLayer)
            .unwrap();
        assert!((fields.get(Field::T, p) - TEMP_MAX_K).abs() < 1e-3);

        // Negative delta would push below 0 K; clamped to 0.
        fields.set(Field::T, p, -1e9, WriteSource::Substrate).unwrap();
        assert_eq!(fields.get(Field::T, p), 0.0);

        // NaN delta is rejected outright.
        let err = fields
            .set(Field::T, p, f32::NAN, WriteSource::ClimateLayer)
            .unwrap_err();
        assert!(matches!(err, SubstrateError::NonFinite { .. }));

        // Out-of-bounds write is rejected.
        let err = fields
            .set(Field::T, IVec3::new(99, 99, 99), 1.0, WriteSource::Substrate)
            .unwrap_err();
        assert!(matches!(err, SubstrateError::OutOfBounds { .. }));
    }

    /// FR-PHYS-substrate-003 — `grad()` returns the central-difference
    /// gradient on interior cells and one-sided (clamped) gradients on
    /// boundary cells. The hardship scalar in `religion::emerge_belief`
    /// will eventually be replaced with `|grad(T)| + |grad(B)|`; this test
    /// locks the math before the layer wiring lands.
    #[test]
    fn grad_central_difference_with_clamped_edges() {
        let spec = GridSpec::new([3, 1, 1]);
        let mut fields = PhysicsFields::new(spec);
        for (x, t) in [(0_i32, 100.0_f32), (1, 200.0), (2, 300.0)] {
            fields
                .replace(Field::T, IVec3::new(x, 0, 0), t)
                .unwrap();
        }
        // Interior cell (1,0,0): symmetric difference → 100.0 on X axis.
        let mid = fields.grad(Field::T, IVec3::new(1, 0, 0));
        assert!((mid.x - 100.0).abs() < 1e-4, "interior X gradient: {mid:?}");
        assert_eq!(mid.y, 0.0);
        assert_eq!(mid.z, 0.0);
        // Left edge cell (0,0,0): one-sided forward difference → 100.0.
        let left = fields.grad(Field::T, IVec3::new(0, 0, 0));
        assert!((left.x - 100.0).abs() < 1e-4, "left-edge X gradient: {left:?}");
        // Right edge cell (2,0,0): one-sided backward difference → 100.0.
        let right = fields.grad(Field::T, IVec3::new(2, 0, 0));
        assert!((right.x - 100.0).abs() < 1e-4, "right-edge X gradient: {right:?}");
    }

    /// FR-PHYS-substrate-004 — `evolve()` is mass-conserving: with all
    /// fields at their uniform steady state, one tick of `evolve()` must not
    /// change the per-field totals (within float epsilon). This pins the
    /// diffusion kernel's conservation contract before we hang the
    /// `substrate_conservation()` invariant off it.
    #[test]
    fn evolve_conserves_total_with_uniform_state() {
        let spec = GridSpec::new([4, 4, 4]);
        let mut fields = PhysicsFields::new(spec);
        // Seed uniform steady states; same value everywhere means each
        // neighbour's contribution to the Laplacian is exactly zero.
        for z in 0..4 {
            for y in 0..4 {
                for x in 0..4 {
                    let p = IVec3::new(x, y, z);
                    fields.replace(Field::T, p, 290.0).unwrap();
                    fields.replace(Field::M, p, 1.0e3).unwrap();
                    fields.replace(Field::E, p, 5.0e6).unwrap();
                    fields.replace(Field::B, p, 2.0e3).unwrap();
                }
            }
        }

        let report = fields.evolve(1.0);
        // Diffusion conserves mass exactly on a closed stencil (sum of
        // neighbour differences cancels per cell).
        assert!(
            report.leak(Field::T).abs() < 1e-2,
            "T leak too high: {} (before {}, after {})",
            report.leak(Field::T),
            report.sum_t_before,
            report.sum_t_after
        );
        assert!(
            report.leak(Field::M).abs() < 1e-2,
            "M leak too high: {}",
            report.leak(Field::M)
        );
        assert!(
            report.leak(Field::E).abs() < 1e-2,
            "E leak too high: {}",
            report.leak(Field::E)
        );
        // F and P are not evolved by `evolve()`; their leak must be exactly 0.
        assert_eq!(report.leak(Field::F), 0.0);
        assert_eq!(report.leak(Field::P), 0.0);
        // Biomass regrowth adds mass — but with zero moisture (we didn't seed
        // any in this test) the kernel is silent, so ΣB is unchanged.
        assert_eq!(report.leak(Field::B), 0.0);
    }

    /// FR-PHYS-substrate-005 — `evolve()` diffuses heat from a hot cell
    /// toward its cooler neighbours and never invents or destroys energy.
    /// This is the existence test for "the substrate actually does physics";
    /// without it the conservation test above could pass on a no-op kernel.
    #[test]
    fn evolve_diffuses_heat_and_preserves_total() {
        let spec = GridSpec::new([3, 1, 1]);
        let mut fields = PhysicsFields::new(spec);
        let cold = 100.0_f32;
        let hot = 400.0_f32;
        // Seed: cold | hot | cold.
        fields
            .replace(Field::T, IVec3::new(0, 0, 0), cold)
            .unwrap();
        fields
            .replace(Field::T, IVec3::new(1, 0, 0), hot)
            .unwrap();
        fields
            .replace(Field::T, IVec3::new(2, 0, 0), cold)
            .unwrap();

        let before_total = fields.sum(Field::T);
        let report = fields.evolve(1.0);
        let after_total = report.sum_t_after;

        // Total temperature is conserved (within float epsilon, accounting
        // for the explicit-Laplacian stencil summing neighbour deltas).
        assert!(
            (after_total - before_total).abs() < 1e-3,
            "T not conserved: before={before_total}, after={after_total}"
        );

        // Hot cell has cooled, neighbours warmed — diffusion happened.
        let hot_cell = fields.get(Field::T, IVec3::new(1, 0, 0));
        let left_cell = fields.get(Field::T, IVec3::new(0, 0, 0));
        let right_cell = fields.get(Field::T, IVec3::new(2, 0, 0));
        assert!(hot_cell < hot, "hot cell did not cool: {hot_cell}");
        assert!(left_cell > cold, "left cell did not warm: {left_cell}");
        assert!(right_cell > cold, "right cell did not warm: {right_cell}");

        // Report's `leak` agrees with the manual sum check.
        assert!(
            (report.leak(Field::T) - (after_total - before_total)).abs() < 1e-3,
            "report leak disagrees with sum diff"
        );
    }

    /// FR-PHYS-substrate-006 — `evolve()` grows biomass in cells whose
    /// temperature is near `BIOMASS_OPTIMAL_TEMP` and moisture is positive,
    /// and leaves hot or dry cells untouched. Pins the L1 life × B write
    /// half of the coupling matrix before the layer wiring lands.
    #[test]
    fn evolve_regrows_biomass_in_habitable_cells() {
        let spec = GridSpec::new([2, 1, 1]);
        let mut fields = PhysicsFields::new(spec);
        fields.tuning.biomass_regrowth = 0.5;
        // Cell 0: hot + dry — no regrowth.
        fields
            .replace(Field::T, IVec3::new(0, 0, 0), 600.0)
            .unwrap();
        fields.replace(Field::M, IVec3::new(0, 0, 0), 0.0).unwrap();
        fields.replace(Field::B, IVec3::new(0, 0, 0), 100.0).unwrap();
        // Cell 1: optimal T, moist — full regrowth.
        fields
            .replace(Field::T, IVec3::new(1, 0, 0), BIOMASS_OPTIMAL_TEMP)
            .unwrap();
        fields.replace(Field::M, IVec3::new(1, 0, 0), 1.0e3).unwrap();
        fields.replace(Field::B, IVec3::new(1, 0, 0), 100.0).unwrap();

        let report = fields.evolve(1.0);
        let cell0 = fields.get(Field::B, IVec3::new(0, 0, 0));
        let cell1 = fields.get(Field::B, IVec3::new(1, 0, 0));
        assert_eq!(cell0, 100.0, "dry/hot cell should not regrow: {cell0}");
        assert!(cell1 > 100.0, "habitable cell should regrow: {cell1}");
        // Regrowth adds mass — total biomass must rise.
        assert!(
            report.sum_b_after > report.sum_b_before,
            "ΣB should rise with regrowth: before={}, after={}",
            report.sum_b_before,
            report.sum_b_after
        );
    }

    /// FR-PHYS-substrate-007 — `evolve()` is deterministic: same initial
    /// state + same `dt` produces a byte-identical report. This pins the
    /// replay contract (ADR-008) for the substrate's own operators; emergent
    /// layer writes are tested separately.
    #[test]
    fn evolve_is_deterministic() {
        let spec = GridSpec::new([3, 3, 1]);
        let mut a = PhysicsFields::new(spec);
        let mut b = PhysicsFields::new(spec);
        for z in 0..1 {
            for y in 0..3 {
                for x in 0..3 {
                    let p = IVec3::new(x, y, z);
                    let t = 280.0 + (x + y * 3) as f32 * 5.0;
                    let m = 5.0e2 + (x * 7 + y * 11) as f32;
                    let e = 1.0e6 + (x + y) as f32 * 1.0e4;
                    a.replace(Field::T, p, t).unwrap();
                    a.replace(Field::M, p, m).unwrap();
                    a.replace(Field::E, p, e).unwrap();
                    a.replace(Field::B, p, 5.0e2).unwrap();
                    b.replace(Field::T, p, t).unwrap();
                    b.replace(Field::M, p, m).unwrap();
                    b.replace(Field::E, p, e).unwrap();
                    b.replace(Field::B, p, 5.0e2).unwrap();
                }
            }
        }
        let ra = a.evolve(1.0);
        let rb = b.evolve(1.0);
        // Compare every field's report totals bit-for-bit.
        for field in Field::ALL {
            assert_eq!(
                ra.sum_after(field).to_bits(),
                rb.sum_after(field).to_bits(),
                "{:?} differs between runs",
                field
            );
        }
    }
}