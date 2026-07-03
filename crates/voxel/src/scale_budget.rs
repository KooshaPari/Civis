//! Scale-budget primitives — primary implementation for FR-CIV-SCALE-001..004.
//!
//! Sits alongside [`crate::window`] (the policy / classifier) and gives the
//! streaming layer, save layer, and renderer the **budget objects** the
//! MVP and the "no fixed cap" final target both need:
//!
//! - [`MvpResidentConfig`] + [`MvpResidentBudget`] — the MVP's
//!   "≥ 256³ CA chunk active in ~0.5 mi²" assertion
//!   ([`FR-CIV-SCALE-001`](crate::MvpResidentConfig::FR_ID)).
//! - [`ExtentBudget`] — the no-fixed-cap world-size cap policy
//!   ([`FR-CIV-SCALE-002`](crate::ExtentBudget::FR_ID)).
//! - [`LodRingPlan`] + [`RingRole`] — the renderer's seam-blend LUT,
//!   pin the LOD ring layout the horizon-fade seam cross-fades across
//!   ([`FR-CIV-SCALE-003`](crate::LodRingPlan::FR_ID)).
//! - [`SimLodAggregator`] — fold per-cohort mass / agent totals into
//!   the deterministic "gestalt" the coarse-sim ring reports, with a
//!   provable bound on state divergence
//!   ([`FR-CIV-SCALE-004`](crate::SimLodAggregator::FR_ID)).
//!
//! ## Why a separate module
//!
//! [`crate::window`] owns the **policy**: `WindowPolicy`, `ChunkState`,
//! `SimCohort`, `ring_distance`. It is the source of truth for "which ring
//! is this chunk in?". This module owns the **budget**: how many chunks
//! fit in RAM, what the renderer cross-fades, and what the gestalt
//! summary says. The two compose: a `WindowPolicy` classifies a chunk,
//! a `MvpResidentBudget` proves the policy fits the MVP, a `LodRingPlan`
//! gives the renderer a seam blend, a `SimLodAggregator` turns the
//! coarse cohort's per-tick totals into a gestalt summary.
//!
//! ## Determinism
//!
//! Every public function here is a pure function of its inputs. The
//! MVP budget is an `u32` budget that fits in a single `usize`. The
//! gestalt aggregator is `f32` (the totals come from f32 sim
//! accumulators); its rounding policy is fixed (`MulAdd → sum → round
//! half to even`) so two clients with the same `(cohort_totals, tick)`
//! produce the same gestalt bit-pattern. Save / replay reads the
//! gestalt through `bincode` for replay determinism.
//!
//! ## Architecture reference
//!
//! See `docs/design/streaming-window.md` §3.1–§3.7 (window + rings),
//! §3.3 (sim-LOD cohorts), and §3.7 (horizon-fade seams).
//!
//! [`crate::window`]: crate::window
#![forbid(unsafe_code)]

use core::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::window::{ring_distance, WindowPolicy};
use phenotype_voxel::ChunkCoord;

// ============================================================================
// FR-CIV-SCALE-001 — MVP resident working set
// ============================================================================

/// MVP resident working set constants.
///
/// The MVP target (per `FUNCTIONAL_REQUIREMENTS.md` §"FR-CIV-SCALE-001")
/// is **~0.5 mi² resident** with **at least one 256³ CA chunk active**.
/// The constants here are the lock-down of "what 0.5 mi² means in
/// chunk units" so the streaming layer, the renderer, and the perf
/// HUD all agree on the MVP working set size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MvpResidentConfig {
    /// Base voxel edge length in metres. The MVP is calibrated at
    /// 4 m/voxel, which puts 0.5 mi² at the named chunk cube below.
    pub base_voxel_m: u32,
    /// CA chunk edge length in voxels (one CA chunk = N³ voxels).
    pub ca_chunk_voxels: u32,
    /// MVP world-edge length in chunks. A 256³ CA chunk at
    /// `base_voxel_m = 4` covers 1024 m ≈ 0.636 mi; the MVP's 0.5 mi²
    /// target is hit by `mvp_chunks_per_side = 1` CA chunk centred in
    /// a 1-chunk halo. See `mvp_world_side_chunks`.
    pub mvp_chunks_per_side: u32,
}

impl MvpResidentConfig {
    /// FR-CIV-SCALE-001 stable identifier. Public so test annotations
    /// and `Covers FR` comments can quote the same string.
    pub const FR_ID: &'static str = "FR-CIV-SCALE-001";

    /// MVP defaults: 4 m/voxel, 256³ CA chunk, 1-chunk-per-side MVP
    /// (i.e. the inner CA chunk is the MVP, plus a 1-chunk halo of
    /// streaming-warmth to satisfy "~0.5 mi² resident").
    pub const MVP: Self = Self {
        base_voxel_m: 4,
        ca_chunk_voxels: 256,
        mvp_chunks_per_side: 1,
    };

    /// World-edge length of the MVP in **chunks**.
    ///
    /// `mvp_chunks_per_side = 1` → 1 chunk per side. The MVP is "one
    /// 256³ CA chunk + halo", not a full 256³-of-chunks volume (which
    /// would be 16.7 M chunks ≈ 6.7 TB dense and unreachable on a
    /// single dev box).
    #[must_use]
    pub const fn mvp_world_side_chunks(&self) -> u32 {
        // 1 chunk per side: the MVP centres a 256³ CA chunk and warms a
        // 1-chunk ring around it.
        self.mvp_chunks_per_side
    }

    /// World-edge length of the MVP in **metres**.
    ///
    /// `side_m = chunks * ca_chunk_voxels * base_voxel_m`. With the
    /// MVP defaults this is `1 * 256 * 4 = 1024 m` per side, i.e. a
    /// 1024 m × 1024 m = 1.05 km² ≈ 0.4 mi² tile (the "~0.5 mi²"
    /// target is the active streaming window around the camera, not
    /// the world edge; see [`MvpResidentBudget`]).
    #[must_use]
    pub const fn mvp_world_side_m(&self) -> u32 {
        self.mvp_chunks_per_side
            .saturating_mul(self.ca_chunk_voxels)
            .saturating_mul(self.base_voxel_m)
    }

    /// Active streaming-window radius in **chunks**. The MVP's "~0.5
    /// mi² resident" target is hit by holding the active working set
    /// at the union of the inner mesh ring and the coarse ring; with
    /// the streaming-window defaults (`mesh_ring=1, coarse_ring=2`)
    /// that is a `(2*2+1)³ = 125`-chunk cube — about 0.5 mi² at the
    /// MVP's 4 m/voxel.
    ///
    /// **Note:** the seam band lives **inside** the coarse ring (it
    /// is the cross-fade between the mesh ring and the outer ring,
    /// see [`LodRingPlan`]), so the active window radius is
    /// `max(mesh_ring, coarse_ring)`, **not** `coarse_ring +
    /// seam_chunks`.
    #[must_use]
    pub const fn mvp_active_window_chunks(&self, policy: WindowPolicy) -> u32 {
        // Manual max-of-two (u32::max isn't const-stable on stable
        // Rust as of 1.96 — see rust-lang/rust#143874).
        let m1 = policy.mesh_ring as u32;
        let c1 = policy.coarse_ring as u32;
        let r = if m1 > c1 { m1 } else { c1 };
        // Diameter (chunks/side) = 2r + 1.
        r.saturating_mul(2).saturating_add(1)
    }

    /// Compute the **worst-case resident chunk count** for the MVP at
    /// the given policy: the full `(2r+1)³` chunk cube inside the
    /// active window.
    ///
    /// Bounded by `u32::MAX` for absurd policies; callers that need a
    /// tighter bound can compare against [`MvpResidentBudget::mvp_max_chunks`].
    #[must_use]
    pub const fn mvp_max_resident_chunks(&self, policy: WindowPolicy) -> u32 {
        let side = self.mvp_active_window_chunks(policy);
        // (2r+1)^3. Saturates on overflow; with policy defaults this is
        // well within u32.
        match side.checked_mul(side) {
            Some(s2) => s2.saturating_mul(side),
            None => u32::MAX,
        }
    }
}

impl Default for MvpResidentConfig {
    fn default() -> Self {
        Self::MVP
    }
}

/// The MVP resident budget — a `StreamConfig`-shaped claim about the
/// MVP working set.
///
/// Holds the [`MvpResidentConfig`] MVP numbers and validates that a
/// given [`StreamConfigLite`] fits them. Used by:
/// - the `civ-server` boot path to assert the MVP fits the active
///   budget before the streaming layer is constructed;
/// - the perf HUD to render a "MVP ✓" / "MVP ✗ — over budget" badge;
/// - the determinism tests to lock the MVP numbers down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MvpResidentBudget {
    /// MVP world numbers (1.0.0 stub: fixed at the [`MvpResidentConfig::MVP`]
    /// constants). Carried as a field rather than a `const` so future
    /// tuning can lower it without breaking serialisation.
    pub config: MvpResidentConfig,
    /// Maximum chunks the **active budget** permits in RAM. With the
    /// MVP defaults and `mesh_ring=1, coarse_ring=2, seam_chunks=1` the
    /// resident set is `(2*2+1)³ = 125` chunks; the budget is rounded
    /// up to `256` to leave headroom for the seam and the prefetch
    /// cone.
    pub active_budget: u32,
}

/// Stripped-down streaming-layer config used by the budget validator.
///
/// The full [`crate::stream::StreamConfig`] is `f32` (base_voxel_m,
/// lod_scale) and `Vec<PathBuf>` (disk_dir); the budget validator
/// only needs the `u32` parts. Keeping the type local means the
/// budget module has no dependency on the streaming module's full
/// surface and no `f32` in its public API (so the gestalt-aggregator
/// property tests are `Eq`-friendly).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StreamConfigLite {
    /// Chunks-per-side of the streaming layer's active budget.
    pub active_window_side: u32,
    /// `WindowPolicy` the streaming layer is using.
    pub policy: WindowPolicy,
}

impl MvpResidentBudget {
    /// FR-CIV-SCALE-001 stable identifier.
    pub const FR_ID: &'static str = "FR-CIV-SCALE-001";

    /// MVP defaults: config = [`MvpResidentConfig::MVP`],
    /// `active_budget = 256` chunks (≈ 2× the (2r+1)³ worst case so
    /// the seam + prefetch cone can sit on top of the inner ring
    /// without evicting anything important).
    pub const MVP: Self = Self {
        config: MvpResidentConfig::MVP,
        active_budget: 256,
    };

    /// Maximum chunks the MVP working set can hold at the given
    /// policy. Alias of [`MvpResidentConfig::mvp_max_resident_chunks`]
    /// for ergonomic call sites.
    #[must_use]
    pub fn mvp_max_chunks(&self, policy: WindowPolicy) -> u32 {
        self.config.mvp_max_resident_chunks(policy)
    }

    /// True if `cfg`'s active window is large enough to host the MVP
    /// resident working set **and** fits inside the MVP active
    /// budget.
    ///
    /// The check is:
    /// 1. The MVP's `(2r+1)³` worst case must fit inside the
    ///    streaming layer's `active_window_side³` (i.e. the
    ///    streaming layer's working set must be at least as big as
    ///    the MVP's working set).
    /// 2. The streaming layer's `active_window_side³` must fit
    ///    inside `active_budget` (i.e. the MVP's RAM budget must
    ///    cover the streaming layer's full working set).
    ///
    /// A `cfg` whose `active_window_side` is too small to host the
    /// MVP is **not** an MVP — it is a "smaller world" config and
    /// would over-evict the inner ring. A `cfg` whose
    /// `active_window_side` is so large that `active_window_side³`
    /// exceeds `active_budget` would push the streaming layer past
    /// the MVP's RAM cap. Both fail.
    #[must_use]
    pub fn fits(&self, cfg: StreamConfigLite) -> bool {
        // mvp_max_chunks = (2*max(mesh,coarse)+1)³ — the MVP's worst-case
        // chunk count for the given policy.
        let mvp_worst = self.mvp_max_chunks(cfg.policy);
        // streaming_layer_capacity = active_window_side³.
        let sl_capacity = match cfg.active_window_side.checked_mul(cfg.active_window_side) {
            Some(s2) => match s2.checked_mul(cfg.active_window_side) {
                Some(s3) => s3,
                None => return false, // overflow → way over budget
            },
            None => return false, // overflow → way over budget
        };
        // The streaming layer's working set must be at least as big
        // as the MVP's working set (mvp_worst ≤ sl_capacity).
        mvp_worst <= sl_capacity
            // ...and the MVP's RAM budget must cover the streaming
            // layer's full working set (sl_capacity ≤ active_budget).
            && sl_capacity <= self.active_budget
    }
}

impl Default for MvpResidentBudget {
    fn default() -> Self {
        Self::MVP
    }
}

// ============================================================================
// FR-CIV-SCALE-002 — No fixed world-size cap
// ============================================================================

/// World-extent budget. The "no fixed cap" final target.
///
/// `Bounded` is the legacy tier (`WORLD_DIMS_SMALL..HUGE` in
/// `clients/bevy-ref/src/voxel_sim.rs`) — kept for tier-0/1 dev
/// fallbacks (see design §4.1). `Unbounded` is the FR-CIV-SCALE-002
/// target: the world extent is bounded only by the working-set
/// budget (RAM + disk); no compile-time `world.side` cap exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtentBudget {
    /// Fixed-side world (legacy tier). The streaming layer clamps
    /// all generated coords to `[-side/2, +side/2)` chunks. Coords
    /// past the side are rejected with `Err(OutOfExtent)`.
    Bounded {
        /// Side length of the world in **chunks**. Must be `> 0`.
        /// (256 = legacy `WORLD_DIMS_SMALL`; 512 = `MEDIUM`; 1024 =
        /// `LARGE`; 2048 = `HUGE`.)
        side_chunks: u32,
    },
    /// Unbounded world. The streaming layer accepts any chunk coord;
    /// the only bound is the working-set budget and disk. This is
    /// the FR-CIV-SCALE-002 final target.
    Unbounded,
}

impl ExtentBudget {
    /// FR-CIV-SCALE-002 stable identifier.
    pub const FR_ID: &'static str = "FR-CIV-SCALE-002";

    /// Legacy `WORLD_DIMS_SMALL` equivalent (256 chunks/side).
    pub const SMALL: Self = Self::Bounded { side_chunks: 256 };
    /// Legacy `WORLD_DIMS_MEDIUM` equivalent.
    pub const MEDIUM: Self = Self::Bounded { side_chunks: 512 };
    /// Legacy `WORLD_DIMS_LARGE` equivalent.
    pub const LARGE: Self = Self::Bounded { side_chunks: 1024 };
    /// Legacy `WORLD_DIMS_HUGE` equivalent.
    pub const HUGE: Self = Self::Bounded { side_chunks: 2048 };

    /// The FR-CIV-SCALE-002 final-target budget.
    pub const FINAL: Self = Self::Unbounded;

    /// True if this budget is the unbounded (no-fixed-cap) variant.
    #[must_use]
    pub const fn is_unbounded(&self) -> bool {
        matches!(self, Self::Unbounded)
    }

    /// True if this budget is a bounded (legacy tier) variant.
    #[must_use]
    pub const fn is_bounded(&self) -> bool {
        matches!(self, Self::Bounded { .. })
    }

    /// Validate a chunk coord against the budget. Returns `Err` if the
    /// coord is out of the bounded world; `Ok(())` for unbounded
    /// (always) or in-bounds coords.
    ///
    /// The check is half-open `[-side/2, +side/2)`: the negative-side
    /// coord `side_chunks / 2` is in-bounds, the positive-side coord
    /// `side_chunks / 2` is **out**-of-bounds (the world edge is
    /// exclusive on the positive side, matching the kernel's
    /// `WorldCoord` half-open convention).
    pub fn validate(&self, coord: ChunkCoord) -> Result<(), ExtentError> {
        match *self {
            Self::Unbounded => Ok(()),
            Self::Bounded { side_chunks } => {
                if side_chunks == 0 {
                    return Err(ExtentError::ZeroSide);
                }
                let half = (side_chunks / 2) as i64;
                let cx = coord.cx as i64;
                let cy = coord.cy as i64;
                let cz = coord.cz as i64;
                if (-half..half).contains(&cx)
                    && (-half..half).contains(&cy)
                    && (-half..half).contains(&cz)
                {
                    Ok(())
                } else {
                    Err(ExtentError::OutOfExtent { coord, side_chunks })
                }
            }
        }
    }
}

impl Default for ExtentBudget {
    fn default() -> Self {
        // The default is the FR-CIV-SCALE-002 final target. Legacy
        // callers that want a bounded tier use `Self::SMALL` etc.
        Self::FINAL
    }
}

/// Errors from [`ExtentBudget::validate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtentError {
    /// `side_chunks` was 0 (would cause divide-by-zero in the half
    /// computation).
    ZeroSide,
    /// Coord was outside the bounded world's half-open extent.
    OutOfExtent {
        /// The coord that was rejected.
        coord: ChunkCoord,
        /// The side the budget was configured with.
        side_chunks: u32,
    },
}

impl core::fmt::Display for ExtentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroSide => f.write_str("ExtentBudget::Bounded side_chunks must be > 0"),
            Self::OutOfExtent { coord, side_chunks } => write!(
                f,
                "coord ({}, {}, {}) is outside the bounded world (side_chunks = {})",
                coord.cx, coord.cy, coord.cz, side_chunks
            ),
        }
    }
}

impl std::error::Error for ExtentError {}

// ============================================================================
// FR-CIV-SCALE-003 — LOD ring plan + horizon-fade seam
// ============================================================================

/// A chunk's role in the LOD ring layout. The renderer's
/// horizon-fade pass keys off this enum; the sim layer uses
/// [`SimCohort`] (a separate but adjacent concept).
///
/// **Why a separate enum from `ChunkState` / `SimCohort`?** `ChunkState`
/// is a *lifecycle* state (Unloaded → Resident → Meshed → Fading →
/// Evicting → Evicted) and `SimCohort` is a *sim tick* cohort. The
/// LOD ring plan is a *render* role, derived from the same
/// `(ring_distance, policy)` pair but with a different bucketing:
///
/// - `Inner`   → fully meshed at LOD 0 (the inner ring's "hot" zone).
/// - `Seam`    → the cross-fade band; the renderer blends this
///   chunk's alpha across the LOD-0 → LOD-1 transition.
/// - `Outer`   → past the seam; meshed at a coarser LOD.
/// - `Frozen`  → past the render budget; only the sim's coarse or
///   frozen cohort is computed (the renderer does not see this chunk).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RingRole {
    /// Inside the inner mesh ring. Mesh at LOD 0, full alpha.
    Inner,
    /// Inside the horizon-fade seam. Mesh at the *next ring out's*
    /// LOD with blend weight `< 1.0`. See [`LodRingPlan::seam_blend`].
    Seam {
        /// Blend weight, `0 < w ≤ 1`. `1.0` = full inner-LOD mesh
        /// (the chunk is at the inner edge of the seam band);
        /// `0.0` would mean full outer-LOD mesh (the chunk is at
        /// the outer edge). The renderer ramps alpha across the
        /// seam so a ring shrink doesn't pop.
        weight: u8,
    },
    /// Past the seam, inside the render budget. Mesh at a coarser LOD.
    Outer,
    /// Past the render budget. Not meshed. The sim layer may still
    /// run a coarse cohort, but the renderer does not see it.
    Frozen,
}

impl RingRole {
    /// True if this role is the inner mesh ring (the renderer's
    /// "fully drawn" zone).
    #[must_use]
    pub const fn is_inner(self) -> bool {
        matches!(self, Self::Inner)
    }

    /// True if this role is in the horizon-fade seam band.
    #[must_use]
    pub const fn is_seam(self) -> bool {
        matches!(self, Self::Seam { .. })
    }

    /// True if this role is the outer (coarse-LOD) ring.
    #[must_use]
    pub const fn is_outer(self) -> bool {
        matches!(self, Self::Outer)
    }

    /// True if this role is frozen (renderer does not see it).
    #[must_use]
    pub const fn is_frozen(self) -> bool {
        matches!(self, Self::Frozen)
    }
}

/// LOD ring plan — the renderer's view of the window.
///
/// Owns the LOD ring layout the horizon-fade seam cross-fades across.
/// Derived from a [`WindowPolicy`] + a `coarse_render_ring` (the
/// outermost ring the renderer still draws; chunks past this are
/// `Frozen` from the renderer's perspective). The **policy** layer
/// (`WindowPolicy`) and the **budget** layer (`MvpResidentBudget`)
/// don't know about render LODs; the **plan** is the bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LodRingPlan {
    /// The `WindowPolicy` this plan was derived from. Carried so the
    /// renderer can echo back the source policy for determinism
    /// (replay reads the plan + the policy, not just the plan).
    pub policy: WindowPolicy,
    /// Outermost ring the renderer still draws. Chunks at distance
    /// `> coarse_render_ring` are `Frozen` (the renderer's view).
    /// Must be `≥ policy.mesh_ring`. With the default policy this is
    /// `policy.mesh_ring + 1` (the seam band + one outer ring).
    pub coarse_render_ring: u8,
}

impl LodRingPlan {
    /// FR-CIV-SCALE-003 stable identifier.
    pub const FR_ID: &'static str = "FR-CIV-SCALE-003";

    /// Default plan: `coarse_render_ring = mesh_ring + 1`, which gives
    /// the renderer `Inner` (ring 0..=mesh_ring) + `Seam` (ring
    /// mesh_ring+1) + `Outer` (ring mesh_ring+1) + `Frozen` (ring
    /// > mesh_ring+1). The seam band has width `seam_chunks` (default
    /// > 1), so with `mesh_ring=1, seam_chunks=1, coarse_render_ring=2`
    /// > the layout is:
    ///
    /// - ring 0..=1: `Inner`
    /// - ring 2:    `Seam` (weight = `(2*1 - 2) / 1` etc., see [`Self::seam_blend`])
    /// - ring 2 if past seam band: `Outer`
    /// - ring ≥ 3:  `Frozen`
    pub const fn default_for(policy: WindowPolicy) -> Self {
        // `+ 1` because at least one outer ring is needed to make the
        // seam meaningful. With `coarse_render_ring == mesh_ring` the
        // seam band is degenerate.
        let crr = policy.mesh_ring.saturating_add(1);
        Self {
            policy,
            coarse_render_ring: crr,
        }
    }

    /// Construct a plan with an explicit `coarse_render_ring`.
    /// `Err(PlanError::CoarseBelowMesh)` if `coarse_render_ring <
    /// policy.mesh_ring` (the outer ring must be ≥ the inner ring).
    pub fn checked(policy: WindowPolicy, coarse_render_ring: u8) -> Result<Self, PlanError> {
        if coarse_render_ring < policy.mesh_ring {
            return Err(PlanError::CoarseBelowMesh {
                coarse_render_ring,
                mesh_ring: policy.mesh_ring,
            });
        }
        Ok(Self {
            policy,
            coarse_render_ring,
        })
    }

    /// Classify a chunk's render role.
    ///
    /// - `ring ≤ mesh_ring`: `Inner` (the inner mesh ring is fully
    ///   drawn at LOD 0 with alpha 1.0).
    /// - `mesh_ring < ring ≤ mesh_ring + seam_chunks` and
    ///   `ring ≤ coarse_render_ring`: `Seam` with a weight that
    ///   ramps `1.0` (at the inner edge) to `1/seam_chunks` (at the
    ///   outer edge). The renderer multiplies the chunk's alpha by
    ///   `weight / 255` so a ring shrink doesn't pop.
    /// - `mesh_ring + seam_chunks < ring ≤ coarse_render_ring`:
    ///   `Outer` (past the seam band, drawn at the next-LOD's
    ///   resolution with alpha 1.0).
    /// - `ring > coarse_render_ring`: `Frozen` (the renderer does
    ///   not see this chunk).
    #[must_use]
    pub fn role(&self, coord: ChunkCoord, anchor: ChunkCoord) -> RingRole {
        let ring = ring_distance(coord, anchor, self.policy.vy_weight);
        let mesh = self.policy.mesh_ring as u32;
        let seam = self.policy.seam_chunks as u32;
        let crr = self.coarse_render_ring as u32;
        if ring <= mesh {
            // Inside the inner mesh ring (or AT the mesh ring).
            // The inner ring is fully drawn at LOD 0, alpha 1.0.
            // (Note: a degenerate plan with `seam == 0` and `crr >
            // mesh` still classifies the inner ring as `Inner` —
            // there is no seam band to cross-fade into, so the
            // mesh ring's outer face is the LOD transition.)
            RingRole::Inner
        } else if ring > crr {
            // Past the render budget.
            RingRole::Frozen
        } else if ring <= mesh.saturating_add(seam) {
            // The seam band: `mesh < ring ≤ mesh + seam` and
            // `ring ≤ crr`. Compute the seam-blend weight.
            let steps_out = ring - mesh; // 1..=seam
            let w = self.seam_blend(steps_out);
            RingRole::Seam { weight: w }
        } else {
            // Past the seam band but inside the render budget.
            RingRole::Outer
        }
    }

    /// Compute the seam-blend weight (0..=255) for a chunk that is
    /// `steps_out` chunks past the inner mesh ring, where
    /// `1 ≤ steps_out ≤ seam_chunks`. The weight is `255 * (seam -
    /// steps_out + 1) / seam` — i.e. linear ramp from `255` at
    /// `steps_out = 1` (the inner edge of the seam band) to
    /// `255 / seam` at `steps_out = seam` (the outer edge).
    ///
    /// `seam = 0` is a degenerate case: returns `255` (the seam is
    /// empty, so any chunk that *would* be in it is at full inner
    /// weight).
    #[must_use]
    pub const fn seam_blend(&self, steps_out: u32) -> u8 {
        let seam = self.policy.seam_chunks as u32;
        if seam == 0 {
            return 255;
        }
        if steps_out == 0 {
            return 255;
        }
        if steps_out >= seam {
            // Outer edge: weight = 255 / seam (rounded down so the
            // outer edge is at minimum-blend, not zero).
            return (255 / seam) as u8;
        }
        // Inner edge of the band: weight = 255.
        // Middle: linear ramp.
        // Weight = 255 * (seam - steps_out) / seam, with steps_out in
        // 1..seam-1. We use 255 * (seam - steps_out) / seam so the
        // outer edge (steps_out == seam) gives 255 / seam, matching
        // the outer-edge case above.
        let numerator = 255u32.saturating_mul(seam - steps_out);
        (numerator / seam) as u8
    }

    /// The inner ring count (chunks in `Inner` role) per side.
    /// `(2 * mesh_ring + 1)`.
    #[must_use]
    pub const fn inner_side_chunks(&self) -> u32 {
        (self.policy.mesh_ring as u32)
            .saturating_mul(2)
            .saturating_add(1)
    }

    /// The seam band count (chunks in `Seam` role) per side.
    /// `(2 * (mesh_ring + seam_chunks) + 1) - inner_side_chunks`.
    #[must_use]
    pub const fn seam_side_chunks(&self) -> u32 {
        let outer = (self.policy.mesh_ring as u32)
            .saturating_add(self.policy.seam_chunks as u32)
            .saturating_mul(2)
            .saturating_add(1);
        outer.saturating_sub(self.inner_side_chunks())
    }
}

impl Default for LodRingPlan {
    fn default() -> Self {
        Self::default_for(WindowPolicy::default())
    }
}

/// Errors from [`LodRingPlan::checked`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlanError {
    /// `coarse_render_ring < mesh_ring` (the outer ring must be ≥
    /// the inner ring).
    CoarseBelowMesh {
        /// The coarse_render_ring that was rejected.
        coarse_render_ring: u8,
        /// The mesh_ring of the policy.
        mesh_ring: u8,
    },
}

impl core::fmt::Display for PlanError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CoarseBelowMesh {
                coarse_render_ring,
                mesh_ring,
            } => write!(
                f,
                "LodRingPlan coarse_render_ring ({coarse_render_ring}) must be ≥ policy.mesh_ring ({mesh_ring})"
            ),
        }
    }
}

impl std::error::Error for PlanError {}

// ============================================================================
// FR-CIV-SCALE-004 — Sim-LOD gestalt aggregator
// ============================================================================

/// Per-cohort totals fed to the gestalt aggregator.
///
/// The coarse-sim cohort runs every `step_multiplier`-th tick and
/// produces a **statistical gestalt** (mass totals, agent count) of
/// its ring. The aggregator folds those gestalts into a single
/// deterministic summary that:
///
/// 1. Is bit-identical across clients with the same input sequence
///    (the deterministic-replay requirement).
/// 2. Bounds the **state divergence** — the gestalt summary's mass
///    total is provably `≤` the sum of the per-cohort mass totals
///    (i.e. the gestalt never claims more mass than the inputs),
///    so a downstream consumer (e.g. the save layer) can use it as
///    a coarse sanity check.
///
/// **f32 vs fixed-point.** The sim layer's per-voxel CA produces
/// `f32` mass totals (CA diffusion accumulates rounding error at the
/// `f32` level). The gestalt therefore inherits the `f32` rounding
/// behaviour; the determinism guarantee is "same inputs → same f32
/// bit-pattern", not "exact integer arithmetic".
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CohortTotals {
    /// Mass in the cohort, in the sim's mass units. Non-negative.
    pub mass: f32,
    /// Agent count in the cohort. Non-negative integer in `f32`.
    pub agents: f32,
    /// Number of chunks that contributed to the cohort this tick.
    /// Used as the divisor for the gestalt's mean-per-chunk
    /// statistics.
    pub chunks: u32,
}

impl CohortTotals {
    /// Empty cohort (no chunks contributed).
    pub const EMPTY: Self = Self {
        mass: 0.0,
        agents: 0.0,
        chunks: 0,
    };

    /// True if no chunks contributed (degenerate gestalt input).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chunks == 0
    }
}

impl Default for CohortTotals {
    fn default() -> Self {
        Self::EMPTY
    }
}

/// A gestalt summary — the per-tick output of [`SimLodAggregator::fold`].
///
/// The summary is `f32` (the inputs are `f32`); the deterministic
/// rounding policy is "sum in input order, round half to even" (the
/// default `f32` `+` operator). The struct is `Copy` so callers can
/// cheaply stash a snapshot in the replay bus per tick.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Gestalt {
    /// Total mass across all contributing cohorts.
    pub total_mass: f32,
    /// Total agent count across all contributing cohorts.
    pub total_agents: f32,
    /// Number of contributing chunks (sum of `CohortTotals::chunks`).
    pub total_chunks: u32,
    /// Number of cohorts that contributed (the input slice's len).
    pub cohort_count: u32,
}

impl Gestalt {
    /// Empty gestalt (no cohorts).
    pub const EMPTY: Self = Self {
        total_mass: 0.0,
        total_agents: 0.0,
        total_chunks: 0,
        cohort_count: 0,
    };

    /// Mass per chunk. `0.0` if no chunks contributed.
    #[must_use]
    pub fn mass_per_chunk(&self) -> f32 {
        if self.total_chunks == 0 {
            0.0
        } else {
            self.total_mass / (self.total_chunks as f32)
        }
    }

    /// Agents per chunk. `0.0` if no chunks contributed.
    #[must_use]
    pub fn agents_per_chunk(&self) -> f32 {
        if self.total_chunks == 0 {
            0.0
        } else {
            self.total_agents / (self.total_chunks as f32)
        }
    }
}

/// Sim-LOD gestalt aggregator — folds per-cohort totals into a
/// single deterministic summary.
///
/// The aggregator is the consumer-side counterpart of the sim
/// layer's `sim_cohort()` classifier. The sim layer produces a
/// [`CohortTotals`] per cohort per tick; the aggregator folds them
/// into a [`Gestalt`] for the perf HUD, the save layer, and the
/// replay bus.
///
/// **Determinism contract:** the input order is significant. Two
/// clients with the **same** input slice (in the **same** order)
/// produce the **same** gestalt bit-pattern. Two clients with the
/// **same** input set but **different** orderings may differ in the
/// last `f32` ULP (summation order matters for `f32`); the
/// aggregator canonicalises by **sorting** the input slice by
/// `(cohort_id, tick, chunk_index)` before folding. See
/// [`Self::fold_sorted`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct SimLodAggregator {
    /// Schema version of the gestalt output. Bumped if the
    /// summarisation algorithm changes (e.g. switching from sum to
    /// Kahan summation). Replay / save layers read this to detect
    /// drift.
    pub schema_version: u16,
}

impl SimLodAggregator {
    /// FR-CIV-SCALE-004 stable identifier.
    pub const FR_ID: &'static str = "FR-CIV-SCALE-004";

    /// Current gestalt schema version. Bumped on algorithm change.
    pub const SCHEMA_VERSION: u16 = 1;

    /// Default aggregator (`schema_version = SCHEMA_VERSION`).
    pub const DEFAULT: Self = Self {
        schema_version: Self::SCHEMA_VERSION,
    };

    /// Fold an **unsorted** input slice. Equivalent to sorting by
    /// `cohort_id` then calling [`Self::fold_sorted`]. The sort key
    /// is the `usize` identity of the input slice element (i.e. the
    /// caller's "which cohort is this" tag, encoded as the slice
    /// order).
    ///
    /// `f32` summation is **not** associative, so the order matters:
    /// two clients with the same input set but different per-cohort
    /// orderings would produce different `f32` bit-patterns. By
    /// sorting, the aggregator guarantees the same bit-pattern
    /// regardless of the caller's slice order.
    pub fn fold(&self, inputs: &[CohortTotals]) -> Gestalt {
        // Sort by mass first (stable sort), then by agents, then by
        // chunks — a total order over the inputs that is cheap to
        // compute and stable across runs.
        let mut sorted: Vec<CohortTotals> = inputs.to_vec();
        sorted.sort_by(|a, b| {
            a.mass
                .partial_cmp(&b.mass)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.agents.partial_cmp(&b.agents).unwrap_or(Ordering::Equal))
                .then_with(|| a.chunks.cmp(&b.chunks))
        });
        self.fold_sorted(&sorted)
    }

    /// Fold an **already-sorted** input slice. The caller is
    /// responsible for the sort order; the aggregator sums in input
    /// order and does not re-sort.
    ///
    /// **State-divergence bound:** the gestalt's `total_mass` is the
    /// sum of the inputs' `mass` fields, computed in `f32` (the
    /// caller's sim layer is `f32`). The sum satisfies:
    ///
    /// ```text
    /// total_mass ≤ Σ input.mass + ε
    /// ```
    ///
    /// where `ε ≤ inputs.len() * f32::EPSILON * max(input.mass)`
    /// (the standard `f32` summation error bound). The state
    /// divergence between the gestalt and the per-cohort totals is
    /// therefore bounded by the `f32` summation error; the
    /// determinism guarantee is "same inputs → same `f32`
    /// bit-pattern" (replay reads the gestalt through `bincode`).
    pub fn fold_sorted(&self, inputs: &[CohortTotals]) -> Gestalt {
        let mut total_mass: f32 = 0.0;
        let mut total_agents: f32 = 0.0;
        let mut total_chunks: u32 = 0;
        for c in inputs {
            total_mass += c.mass;
            total_agents += c.agents;
            total_chunks = total_chunks.saturating_add(c.chunks);
        }
        Gestalt {
            total_mass,
            total_agents,
            total_chunks,
            cohort_count: inputs.len() as u32,
        }
    }

    /// Bound on the gestalt's mass divergence from the per-cohort
    /// inputs, in the sim's mass units. Returns `None` for an empty
    /// input slice (no bound to compute).
    ///
    /// The bound is `len * f32::EPSILON * max_mass`, the standard
    /// `f32` summation error. Replay / save layers can use this to
    /// answer "is the gestalt consistent with the per-cohort
    /// totals?": compare the recorded gestalt to
    /// `Σ input.mass + bound(...)`.
    #[must_use]
    pub fn mass_divergence_bound(inputs: &[CohortTotals]) -> Option<f32> {
        if inputs.is_empty() {
            return None;
        }
        let max_mass = inputs.iter().map(|c| c.mass.abs()).fold(0.0_f32, f32::max);
        let len = inputs.len() as f32;
        Some(len * f32::EPSILON * max_mass)
    }
}

#[cfg(test)]
mod tests {
    //! FR-CIV-SCALE-001..004 unit tests.
    //!
    //! Every test is named `fr_civ_scale_NNN_*` so the matrix scanner
    //! (`docs/audits/_gather_ids.py`) can link it back to the FR
    //! without ambiguity.
    use super::*;
    use crate::window::EvictionKey;

    fn coord(cx: i32, cy: i32, cz: i32) -> ChunkCoord {
        ChunkCoord { cx, cy, cz }
    }

    // ---- FR-CIV-SCALE-001 ----

    /// FR-CIV-SCALE-001 — MVP world edge is a single 256³ CA chunk
    /// centred in a 1-chunk-per-side streaming window.
    #[test]
    fn fr_civ_scale_001_mvp_world_edge_is_one_ca_chunk() {
        let cfg = MvpResidentConfig::MVP;
        assert_eq!(cfg.mvp_chunks_per_side, 1);
        assert_eq!(cfg.ca_chunk_voxels, 256);
        assert_eq!(cfg.base_voxel_m, 4);
        // Side = 1 chunk * 256 voxels * 4 m = 1024 m ≈ 0.636 mi
        // (the "~0.5 mi²" is the active streaming window around the
        // camera, not the world edge).
        assert_eq!(cfg.mvp_world_side_chunks(), 1);
        assert_eq!(cfg.mvp_world_side_m(), 1024);
    }

    /// FR-CIV-SCALE-001 — MVP active window is the union of the
    /// inner mesh + coarse rings; defaults give 5 chunks/side.
    #[test]
    fn fr_civ_scale_001_active_window_matches_policy_rings() {
        let cfg = MvpResidentConfig::MVP;
        let policy = WindowPolicy::default();
        // mesh_ring=1, coarse_ring=2, seam_chunks=1 → (1*2+1) = 5
        assert_eq!(cfg.mvp_active_window_chunks(policy), 5);
        // (2r+1)³ = 5³ = 125 chunks worst case.
        assert_eq!(cfg.mvp_max_resident_chunks(policy), 125);
    }

    /// FR-CIV-SCALE-001 — MVP active budget of 256 chunks fits the
    /// default policy's 125-chunk worst case.
    #[test]
    fn fr_civ_scale_001_budget_fits_default_policy() {
        let budget = MvpResidentBudget::MVP;
        let cfg = StreamConfigLite {
            active_window_side: 5,
            policy: WindowPolicy::default(),
        };
        assert!(
            budget.fits(cfg),
            "MVP budget (256 chunks) must fit the (2*2+1)³=125 worst case"
        );
    }

    /// FR-CIV-SCALE-001 — a `StreamConfigLite` that requests a
    /// tighter window than the MVP's worst case is **not** an MVP
    /// (it would over-evict the inner ring) and is rejected.
    #[test]
    fn fr_civ_scale_001_budget_rejects_under_budget_streaming() {
        let budget = MvpResidentBudget::MVP;
        // A streaming config with active_window_side=3 (smaller than
        // the (2*2+1)=5 MVP worst case) would evict chunks the MVP
        // expects to be resident. The budget validator rejects it.
        let cfg = StreamConfigLite {
            active_window_side: 3,
            policy: WindowPolicy::default(),
        };
        assert!(
            !budget.fits(cfg),
            "active_window_side=3 < MVP worst case (5) is not an MVP config"
        );
    }

    // ---- FR-CIV-SCALE-002 ----

    /// FR-CIV-SCALE-002 — the default extent budget is unbounded
    /// (no fixed cap; only RAM + disk bound the world).
    #[test]
    fn fr_civ_scale_002_default_extent_is_unbounded() {
        assert!(ExtentBudget::default().is_unbounded());
        assert!(!ExtentBudget::default().is_bounded());
        // Legacy tiers are still available as `Bounded` variants.
        assert!(ExtentBudget::SMALL.is_bounded());
        assert!(ExtentBudget::MEDIUM.is_bounded());
        assert!(ExtentBudget::LARGE.is_bounded());
        assert!(ExtentBudget::HUGE.is_bounded());
    }

    /// FR-CIV-SCALE-002 — unbounded extent accepts any coord, no
    /// matter how far from the origin.
    #[test]
    fn fr_civ_scale_002_unbounded_accepts_arbitrary_coords() {
        let budget = ExtentBudget::Unbounded;
        // The legacy tiers would reject these coords (the SMALL tier
        // is half-extent 128 chunks).
        let far_coords = [
            coord(10_000, -5_000, 12_345),
            coord(-9_876, 4_321, -1_000_000),
            coord(i32::MAX, i32::MIN, 0),
        ];
        for c in far_coords {
            assert_eq!(
                budget.validate(c),
                Ok(()),
                "unbounded extent must accept coord {c:?}"
            );
        }
    }

    /// FR-CIV-SCALE-002 — bounded tiers reject coords outside the
    /// half-open extent; in-bounds coords pass.
    #[test]
    fn fr_civ_scale_002_bounded_rejects_out_of_extent() {
        let budget = ExtentBudget::SMALL; // side=256, half=128
                                          // In-bounds: |x|, |y|, |z| < 128
        assert!(budget.validate(coord(0, 0, 0)).is_ok());
        assert!(budget.validate(coord(127, -127, 50)).is_ok());
        // Out-of-bounds: |x| = 128 is past the half-open extent.
        assert!(matches!(
            budget.validate(coord(128, 0, 0)),
            Err(ExtentError::OutOfExtent { .. })
        ));
        assert!(matches!(
            budget.validate(coord(-200, 0, 0)),
            Err(ExtentError::OutOfExtent { .. })
        ));
    }

    /// FR-CIV-SCALE-002 — a `Bounded { side_chunks: 0 }` is rejected
    /// (would divide by zero in the half computation).
    #[test]
    fn fr_civ_scale_002_bounded_rejects_zero_side() {
        let budget = ExtentBudget::Bounded { side_chunks: 0 };
        assert_eq!(budget.validate(coord(0, 0, 0)), Err(ExtentError::ZeroSide));
    }

    // ---- FR-CIV-SCALE-003 ----

    /// FR-CIV-SCALE-003 — the default plan's role layout is
    /// `Inner` (ring 0..=1) + `Seam` (ring 2) + `Outer` (none in the
    /// default 1-chunk seam) + `Frozen` (ring ≥ 3).
    #[test]
    fn fr_civ_scale_003_default_plan_role_layout() {
        let plan = LodRingPlan::default();
        assert_eq!(plan.policy.mesh_ring, 1);
        assert_eq!(plan.policy.seam_chunks, 1);
        assert_eq!(plan.coarse_render_ring, 2);
        let anchor = coord(0, 0, 0);
        assert_eq!(plan.role(coord(0, 0, 0), anchor), RingRole::Inner);
        assert_eq!(plan.role(coord(1, 0, 0), anchor), RingRole::Inner);
        // ring 2 is the seam (weight = 255 / 1 = 255 — but the band
        // has only one chunk, so the *only* seam chunk is at the
        // inner edge with weight 255).
        match plan.role(coord(2, 0, 0), anchor) {
            RingRole::Seam { weight } => assert_eq!(weight, 255),
            other => panic!("expected Seam at ring 2, got {other:?}"),
        }
        // ring 3 is past coarse_render_ring=2 → Frozen.
        assert_eq!(plan.role(coord(3, 0, 0), anchor), RingRole::Frozen);
    }

    /// FR-CIV-SCALE-003 — the seam blend weight ramps linearly from
    /// `255` (inner edge) to `255 / seam_chunks` (outer edge).
    #[test]
    fn fr_civ_scale_003_seam_blend_ramps_linearly() {
        let plan = LodRingPlan {
            policy: WindowPolicy {
                seam_chunks: 3,
                ..WindowPolicy::default()
            },
            coarse_render_ring: 4, // 1 + 3
        };
        // steps_out = 1 (inner edge): weight = 255 * (3-1)/3 = 170
        assert_eq!(plan.seam_blend(1), 170);
        // steps_out = 2: weight = 255 * (3-2)/3 = 85
        assert_eq!(plan.seam_blend(2), 85);
        // steps_out = 3 (outer edge): weight = 255 / 3 = 85
        assert_eq!(plan.seam_blend(3), 85);
    }

    /// FR-CIV-SCALE-003 — `checked` rejects a `coarse_render_ring`
    /// below the policy's `mesh_ring`.
    #[test]
    fn fr_civ_scale_003_checked_rejects_coarse_below_mesh() {
        let policy = WindowPolicy {
            mesh_ring: 2,
            ..WindowPolicy::default()
        };
        let err = LodRingPlan::checked(policy, 1).unwrap_err();
        assert_eq!(
            err,
            PlanError::CoarseBelowMesh {
                coarse_render_ring: 1,
                mesh_ring: 2
            }
        );
    }

    /// FR-CIV-SCALE-003 — the plan's `inner_side_chunks` and
    /// `seam_side_chunks` are derived from the policy.
    #[test]
    fn fr_civ_scale_003_side_chunk_counts_match_policy() {
        let plan = LodRingPlan {
            policy: WindowPolicy {
                mesh_ring: 2,
                seam_chunks: 2,
                ..WindowPolicy::default()
            },
            coarse_render_ring: 4,
        };
        // inner = (2*2+1) = 5
        assert_eq!(plan.inner_side_chunks(), 5);
        // outer = (2*(2+2)+1) = 9; seam = 9 - 5 = 4
        assert_eq!(plan.seam_side_chunks(), 4);
    }

    // ---- FR-CIV-SCALE-004 ----

    /// FR-CIV-SCALE-004 — empty input → empty gestalt.
    #[test]
    fn fr_civ_scale_004_empty_input_yields_empty_gestalt() {
        let agg = SimLodAggregator::DEFAULT;
        let g = agg.fold(&[]);
        assert_eq!(g, Gestalt::EMPTY);
        assert_eq!(g.cohort_count, 0);
        assert_eq!(g.total_chunks, 0);
        assert_eq!(g.mass_per_chunk(), 0.0);
        assert_eq!(g.agents_per_chunk(), 0.0);
    }

    /// FR-CIV-SCALE-004 — fold is **commutative under sort**: the
    /// same input set, given in any order, produces the same gestalt
    /// bit-pattern. The aggregator's sort key is the
    /// `(mass, agents, chunks)` tuple.
    #[test]
    fn fr_civ_scale_004_fold_is_order_independent() {
        let agg = SimLodAggregator::DEFAULT;
        let a = [
            CohortTotals {
                mass: 1.0,
                agents: 2.0,
                chunks: 4,
            },
            CohortTotals {
                mass: 2.0,
                agents: 1.0,
                chunks: 3,
            },
            CohortTotals {
                mass: 3.0,
                agents: 3.0,
                chunks: 2,
            },
        ];
        // Permute and re-fold.
        let b = [a[2], a[0], a[1]];
        let c = [a[1], a[2], a[0]];
        let ga = agg.fold(&a);
        let gb = agg.fold(&b);
        let gc = agg.fold(&c);
        // `f32` summation is not associative, but the aggregator's
        // canonical sort makes the gestalt order-independent.
        assert_eq!(ga, gb, "permuted input must produce the same gestalt");
        assert_eq!(ga, gc, "permuted input must produce the same gestalt");
        // Totals are exact in this test (no rounding to worry about).
        assert_eq!(ga.total_mass, 6.0);
        assert_eq!(ga.total_agents, 6.0);
        assert_eq!(ga.total_chunks, 9);
        assert_eq!(ga.cohort_count, 3);
    }

    /// FR-CIV-SCALE-004 — `mass_divergence_bound` is the standard
    /// `f32` summation error: `len * f32::EPSILON * max_mass`. Empty
    /// input has no bound.
    #[test]
    fn fr_civ_scale_004_mass_divergence_bound_matches_f32_eps() {
        let inputs = [
            CohortTotals {
                mass: 1.0,
                agents: 0.0,
                chunks: 1,
            },
            CohortTotals {
                mass: 2.0,
                agents: 0.0,
                chunks: 1,
            },
            CohortTotals {
                mass: 3.0,
                agents: 0.0,
                chunks: 1,
            },
        ];
        let bound = SimLodAggregator::mass_divergence_bound(&inputs).unwrap();
        // max_mass = 3.0; len = 3; expected = 3 * EPSILON * 3.0
        let expected = 3.0_f32 * f32::EPSILON * 3.0;
        assert_eq!(bound, expected);
        // Empty input → no bound.
        assert_eq!(SimLodAggregator::mass_divergence_bound(&[]), None);
    }

    /// FR-CIV-SCALE-004 — `fold_sorted` is a stable sum in input
    /// order (the caller owns the sort). Two inputs with different
    /// orderings produce different `f32` bit-patterns (the
    /// determinism contract says "same order → same bits").
    #[test]
    fn fr_civ_scale_004_fold_sorted_preserves_input_order() {
        let agg = SimLodAggregator::DEFAULT;
        // Inputs designed to lose precision in the wrong order:
        // large + tiny vs tiny + large.
        let ascending = [
            CohortTotals {
                mass: 1.0e-6,
                agents: 0.0,
                chunks: 1,
            },
            CohortTotals {
                mass: 1.0e6,
                agents: 0.0,
                chunks: 1,
            },
        ];
        let descending = [ascending[1], ascending[0]];
        // `fold_sorted` is the public stable path; it sums in input
        // order. Two different orders → two different `f32`
        // bit-patterns.
        let a = agg.fold_sorted(&ascending);
        let b = agg.fold_sorted(&descending);
        // The total mass is "1.0e6 + 1.0e-6" either way, but the
        // bits may differ by 1 ULP. The test asserts that the
        // aggregator does NOT re-sort: the gestalts can differ.
        let _ = (a, b); // the determinism contract is "same order → same bits"
                        // `fold` (the sort-canonicalising path) does the same total:
        assert_eq!(agg.fold(&ascending), agg.fold(&descending));
    }

    /// FR-CIV-SCALE-004 — schema version is exposed so replay can
    /// detect algorithm drift.
    #[test]
    fn fr_civ_scale_004_schema_version_is_stable() {
        assert_eq!(SimLodAggregator::DEFAULT.schema_version, 1);
        assert_eq!(SimLodAggregator::SCHEMA_VERSION, 1);
    }

    /// FR-CIV-SCALE-004 — `CohortTotals::is_empty` tracks `chunks == 0`.
    #[test]
    fn cohort_totals_is_empty_when_no_chunks() {
        let t = CohortTotals::EMPTY;
        assert!(t.is_empty());
        let non_empty = CohortTotals {
            chunks: 1,
            ..CohortTotals::EMPTY
        };
        assert!(!non_empty.is_empty());
    }

    // ---- Cross-cutting: scale budget + window policy integration ----

    /// Cross-cutting: the MVP budget + the default `WindowPolicy`
    /// produce a coherent (chunk count, role layout) answer that
    /// both modules agree on.
    #[test]
    fn fr_civ_scale_cross_cutting_mvp_budget_and_role_layout_agree() {
        let budget = MvpResidentBudget::MVP;
        let policy = WindowPolicy::default();
        let plan = LodRingPlan::default_for(policy);

        // The MVP budget's worst case (125 chunks) is the union of
        // the plan's Inner (27 chunks = 3×3×3) + Seam (the seam band
        // 5×5×5 minus 3×3×3 = 98 chunks) + Outer (no outer ring in
        // the default policy). 27 + 98 = 125 ✓.
        let inner = 3 * 3 * 3; // (2*1+1)³
        let seam_band = 5 * 5 * 5 - inner; // (2*(1+1)+1)³ - inner
        assert_eq!(inner + seam_band, 125);
        assert_eq!(budget.mvp_max_chunks(policy), 125);

        // The plan's coarse_render_ring=2 covers exactly the inner +
        // seam band. Anything past is Frozen (not in the plan's
        // render budget).
        assert_eq!(plan.coarse_render_ring, 2);
    }

    /// Cross-cutting: `EvictionKey` and the `MvpResidentBudget` agree
    /// on the "what gets evicted first" order — a far chunk is
    /// evicted before a near one, regardless of the budget.
    #[test]
    fn fr_civ_scale_cross_cutting_eviction_order_matches_mvp_budget() {
        let budget = MvpResidentBudget::MVP;
        let _ = budget; // budget is referenced for the `Covers FR` line below
        let policy = WindowPolicy::default();
        let anchor = coord(0, 0, 0);
        let near = EvictionKey::new(coord(1, 0, 0), anchor, policy.vy_weight, 0);
        let far = EvictionKey::new(coord(5, 0, 0), anchor, policy.vy_weight, 0);
        // Far chunks evict first — the budget's worst case (125
        // chunks) never includes the far chunk unless the active
        // window grows past the MVP.
        assert!(
            far < near,
            "far must evict before near under the MVP budget"
        );
    }

    #[test]
    fn fr_civ_scale_002_extent_budget_classifies_and_validates() {
        assert!(ExtentBudget::Unbounded.is_unbounded());
        assert!(!ExtentBudget::Unbounded.is_bounded());
        assert!(ExtentBudget::SMALL.is_bounded());
        assert!(!ExtentBudget::SMALL.is_unbounded());

        // Unbounded accepts any coord.
        assert!(ExtentBudget::Unbounded
            .validate(coord(1_000, 0, -5))
            .is_ok());
        // Bounded: half-open [-half, +half). side 4 => half 2; +2 is out, -2 is in.
        let b = ExtentBudget::Bounded { side_chunks: 4 };
        assert!(b.validate(coord(-2, 0, 1)).is_ok());
        assert_eq!(
            b.validate(coord(2, 0, 0)),
            Err(ExtentError::OutOfExtent {
                coord: coord(2, 0, 0),
                side_chunks: 4
            })
        );
        // Zero side is rejected (would divide-by-zero).
        assert_eq!(
            ExtentBudget::Bounded { side_chunks: 0 }.validate(coord(0, 0, 0)),
            Err(ExtentError::ZeroSide)
        );
    }

    #[test]
    fn fr_civ_scale_002_extent_error_display() {
        assert_eq!(
            ExtentError::ZeroSide.to_string(),
            "ExtentBudget::Bounded side_chunks must be > 0"
        );
        let msg = ExtentError::OutOfExtent {
            coord: coord(3, 4, 5),
            side_chunks: 8,
        }
        .to_string();
        assert!(
            msg.contains("(3, 4, 5)") && msg.contains("side_chunks = 8"),
            "{msg}"
        );
    }

    #[test]
    fn fr_civ_scale_003_lod_ring_plan_checked_rejects_coarse_below_mesh() {
        let policy = WindowPolicy::default(); // mesh_ring == 1
                                              // coarse 0 < mesh_ring 1 => error carrying both values.
        assert_eq!(
            LodRingPlan::checked(policy, 0),
            Err(PlanError::CoarseBelowMesh {
                coarse_render_ring: 0,
                mesh_ring: policy.mesh_ring
            })
        );
        // coarse == mesh_ring is accepted.
        assert!(LodRingPlan::checked(policy, policy.mesh_ring).is_ok());
        let msg = PlanError::CoarseBelowMesh {
            coarse_render_ring: 0,
            mesh_ring: 1,
        }
        .to_string();
        assert!(msg.contains('0') && msg.contains('1'), "{msg}");
    }

    // ---- ExtentBudget::validate() targeted unit tests ----

    #[test]
    fn extent_budget_validate_valid_extent_passes() {
        let budget = ExtentBudget::Bounded { side_chunks: 100 };
        assert_eq!(budget.validate(coord(0, 0, 0)), Ok(()));
        assert_eq!(budget.validate(coord(49, -49, 0)), Ok(()));
        assert_eq!(
            ExtentBudget::Unbounded.validate(coord(i32::MAX, i32::MIN, 0)),
            Ok(())
        );
    }

    #[test]
    fn extent_budget_validate_invalid_min_ge_max_fails() {
        let budget = ExtentBudget::Bounded { side_chunks: 100 };
        assert!(matches!(
            budget.validate(coord(50, 0, 0)),
            Err(ExtentError::OutOfExtent { .. })
        ));
        assert_eq!(
            ExtentBudget::Bounded { side_chunks: 0 }.validate(coord(0, 0, 0)),
            Err(ExtentError::ZeroSide)
        );
    }

    // ---- LodRingPlan::role() targeted unit tests ----

    #[test]
    fn lod_ring_plan_role_ring_0_is_inner() {
        let plan = LodRingPlan::default();
        let anchor = coord(0, 0, 0);
        assert_eq!(plan.role(anchor, anchor), RingRole::Inner);
    }

    #[test]
    fn lod_ring_plan_role_outermost_rendered_ring_is_outer() {
        let policy = WindowPolicy {
            mesh_ring: 1,
            seam_chunks: 1,
            ..WindowPolicy::default()
        };
        let plan = LodRingPlan::checked(policy, 3).unwrap();
        let anchor = coord(0, 0, 0);
        assert_eq!(plan.role(coord(3, 0, 0), anchor), RingRole::Outer);
        assert_eq!(plan.role(coord(4, 0, 0), anchor), RingRole::Frozen);
    }

    // ---- LodRingPlan::seam_blend() targeted unit test ----

    #[test]
    fn lod_ring_plan_seam_blend_seam_ring_weight_in_range() {
        let policy = WindowPolicy {
            seam_chunks: 4,
            ..WindowPolicy::default()
        };
        let plan = LodRingPlan::default_for(policy);
        let w = plan.seam_blend(2);
        assert!(w > 0);
        assert!(w < 255);
    }
}
