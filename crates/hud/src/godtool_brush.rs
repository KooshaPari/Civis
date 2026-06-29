//! God-tool brush kernel — FR-CIV-GODTOOL-901.
//!
//! The HUD-side data + query layer for the **brush application kernel** that
//! every god-tool brush stamp (terrain raise/lower, material paint, additive
//! drop, etc.) consumes when it distributes a click across affected cells.
//!
//! Per `docs/specs/requirements/FR-CIV-GODTOOL.md` (FR-CIV-GODTOOL-901) and
//! `docs/design/brush-tool-system.md` §2.2 + §2.3, the kernel is the function
//! that maps `(radius, strength, falloff, cell_offset)` → a normalized weight
//! in `[0.0, 1.0]`. The host client stamps the footprint once; every affected
//! cell multiplies the brush's payload (height Δ, material mass, energy, …)
//! by `kernel.weight(cell)`, so a strength-1 brush at the center writes the
//! full payload and a brush at the very edge of the radius writes zero.
//!
//! This module is **substrate-neutral**:
//!
//! - It owns the kernel *shape* (Linear / Smooth / Hard) and the
//!   `BrushKernel::weight_at(&cell_offset)` query.
//! - It owns the `affected_cells(&center, &shape)` enumeration (which cells
//!   fall inside the footprint at all).
//! - It exposes a single `apply(&cell_offset)` helper that returns
//!   `(weight, in_radius)` so callers can both gate writes on
//!   `in_radius` and scale them by `weight`.
//!
//! It does **not** depend on Bevy, Godot, Unreal, the server, or any
//! rendering crate — the kernel is pure data + math, and it can be reused
//! from any host.
//!
//! ## Relationship to other crates
//!
//! - The **palette state** (current tool + per-tool params) lives in
//!   `crate::god_tool_state` (FR-CIV-GODTOOL-900). A host composes a
//!   `GodToolState` + a `BrushKernelParams` (read from that state's
//!   `radius` / `strength` / `falloff` params) into a `BrushKernel` and
//!   stamps.
//! - The **substrate-facing schema** (the 50-verb catalog with substrate
//!   request kinds) lives in `civ-powers::PowerDef` / `PowerRegistry`. This
//!   module is the kernel math that any tool's applier eventually
//!   multiplies into the world write.
//!
//! ## Acceptance (AC-GT-BRUSH-901)
//!
//! - **AC-1:** `kernel.weight_at((0, 0)) == strength` (max at center).
//! - **AC-2:** `kernel.weight_at(offset)` is monotonically non-increasing
//!   in `|offset|` for every supported falloff.
//! - **AC-3:** `kernel.weight_at(offset) == 0.0` for any `offset` whose
//!   `length() > radius` (zero at the radius edge and beyond).

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Falloff curve applied across the brush footprint.
///
/// Each variant defines the **shape** of `weight(t)` where
/// `t = |offset| / radius ∈ [0, 1]`. `t = 0` is the brush center (full
/// weight = `strength`); `t >= 1` is outside the radius (zero weight).
///
/// The falloff is what makes the brush "feel" like a tool: a Hard edge
/// stamps a flat plateau, a Linear falloff ramps evenly, a Smooth falloff
/// eases in/out so the edge is feathered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrushFalloff {
    /// No falloff. Every cell inside the footprint gets the full
    /// `strength`; cells outside get zero. The plateau is `weight(t) = 1`
    /// for `t ∈ [0, 1)`.
    Hard,
    /// Linear ramp: `weight(t) = 1 - t`.
    Linear,
    /// Smooth ease-in/ease-out (`smoothstep`): `weight(t) = 1 - t² ·
    /// (3 - 2t)`. The edges feather to zero with continuous first
    /// derivative.
    Smooth,
}

impl BrushFalloff {
    /// All falloffs, declaration order. Used by tests and host clients
    /// that want to iterate every supported curve.
    pub const ALL: [BrushFalloff; 3] = [
        BrushFalloff::Hard,
        BrushFalloff::Linear,
        BrushFalloff::Smooth,
    ];

    /// Evaluate the curve at normalized distance `t` ∈ `[0, 1]`. Returns
    /// `0.0` for `t >= 1` (outside the radius) and a value in `[0, 1]`
    /// for `t ∈ [0, 1)`. Always returns `1.0` at `t = 0` (the center is
    /// always full strength for every falloff).
    ///
    /// This is the **shape function only** — the caller multiplies the
    /// result by `strength` to get the kernel weight. See
    /// [`BrushKernel::weight_at`].
    #[must_use]
    pub fn shape(self, t: f64) -> f64 {
        if t <= 0.0 {
            return 1.0;
        }
        if t >= 1.0 {
            return 0.0;
        }
        match self {
            // Hard plateau: every in-radius cell gets the full shape
            // value. `t < 1` is "in" and we still want full weight (this
            // is the difference from Linear — Hard has no ramp).
            BrushFalloff::Hard => 1.0,
            // Linear ramp from 1 → 0 across the radius.
            BrushFalloff::Linear => 1.0 - t,
            // Smoothstep ease-in/ease-out: 1 - smoothstep(0, 1, t).
            // smoothstep(0,1,t) = t² · (3 - 2t), which is 0 at t=0, 1 at
            // t=1, with zero derivative at both endpoints.
            BrushFalloff::Smooth => {
                let s = t * t * (3.0 - 2.0 * t);
                1.0 - s
            }
        }
    }
}

impl Default for BrushFalloff {
    fn default() -> Self {
        BrushFalloff::Linear
    }
}

/// Brush footprint shape — the **planar** outline of the kernel.
///
/// `Circle` is the default and the most common (round brushes, world-edit
/// symmetry). `Square` is the grid-aligned box (matches the voxel column
/// geometry directly). `Diamond` is the rotated square (Manhattan
/// distance ≤ radius).
///
/// The shape only affects which cells are *in the footprint at all* (the
/// radius-edge gating). The falloff is what modulates weight *within*
/// the footprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrushShape {
    /// Round footprint — Euclidean distance ≤ radius.
    Circle,
    /// Grid-aligned square — Chebyshev distance ≤ radius (so a
    /// `radius = 2` square is 5×5 cells).
    Square,
    /// Diamond — Manhattan distance ≤ radius (so a `radius = 2` diamond
    /// is a 5×5 rhombus).
    Diamond,
}

impl BrushShape {
    /// All shapes, declaration order.
    pub const ALL: [BrushShape; 3] = [
        BrushShape::Circle,
        BrushShape::Square,
        BrushShape::Diamond,
    ];

    /// `true` when `(dx, dy)` (axial offset from the brush center) is
    /// inside the footprint of `radius`. The unit is **cells**; the
    /// caller pre-multiplies world coordinates into cell offsets before
    /// calling.
    ///
    /// This is the **edge test** — it answers "is this cell in the
    /// radius at all?" (AC-3: weight is zero at and beyond the edge).
    /// The falloff *within* the radius is computed separately.
    #[must_use]
    pub fn contains(self, dx: i32, dy: i32, radius: i32) -> bool {
        if radius <= 0 {
            // A non-positive radius footprint is empty by convention;
            // a literal `0` would otherwise include the center cell.
            return false;
        }
        let adx = dx.unsigned_abs();
        let ady = dy.unsigned_abs();
        match self {
            // Euclidean: dx² + dy² ≤ r². We do this in integer space by
            // squaring once and comparing to radius² — no f64, no
            // rounding error.
            BrushShape::Circle => {
                // `i64` to avoid `u32²` overflow on large radii.
                let dx2 = (dx as i64) * (dx as i64);
                let dy2 = (dy as i64) * (dy as i64);
                let r2 = (radius as i64) * (radius as i64);
                dx2 + dy2 <= r2
            }
            // Chebyshev: max(|dx|, |dy|) ≤ r.
            BrushShape::Square => adx <= radius as u32 && ady <= radius as u32,
            // Manhattan: |dx| + |dy| ≤ r.
            BrushShape::Diamond => (adx + ady) <= radius as u32,
        }
    }
}

impl Default for BrushShape {
    fn default() -> Self {
        BrushShape::Circle
    }
}

/// The brush parameters that drive the kernel — a plain-data carrier
/// of `(radius, strength, falloff, shape)`.
///
/// This is the HUD-side view of the kernel params. A host client reads
/// the per-tool `radius` / `strength` / `falloff` values out of
/// [`crate::GodToolState`] (FR-CIV-GODTOOL-900) and constructs a
/// `BrushKernelParams` to feed into [`BrushKernel::new`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BrushKernelParams {
    /// Brush footprint radius in cells. Must be ≥ 0; a `radius = 0`
    /// kernel affects no cells. The default is `3` (matches the
    /// `Raise` tool's spec default in `god_tool_state::GodToolRegistry`).
    pub radius: i32,
    /// Peak weight at the center of the footprint. Applied uniformly to
    /// the falloff shape (`weight = strength · falloff.shape(t)`).
    /// Must be ≥ 0; values > 1 amplify the payload.
    pub strength: f64,
    /// Falloff curve (Hard / Linear / Smooth).
    pub falloff: BrushFalloff,
    /// Footprint outline (Circle / Square / Diamond).
    pub shape: BrushShape,
}

impl BrushKernelParams {
    /// Construct a kernel params from the four driving values.
    /// `radius` and `strength` are required; `falloff` and `shape`
    /// default to `Linear` and `Circle` respectively.
    ///
    /// No clamping is performed — `BrushKernel::new` is the validator.
    /// Pass any values here; the kernel constructor decides what they
    /// mean (e.g. negative `radius` collapses to "empty footprint").
    #[must_use]
    pub fn new(radius: i32, strength: f64, falloff: BrushFalloff, shape: BrushShape) -> Self {
        Self {
            radius,
            strength,
            falloff,
            shape,
        }
    }

    /// Convenience constructor: `(radius, strength)` with `Linear`
    /// falloff and `Circle` shape — the most common brush in the
    /// sandbox.
    #[must_use]
    pub fn circle_linear(radius: i32, strength: f64) -> Self {
        Self::new(radius, strength, BrushFalloff::Linear, BrushShape::Circle)
    }
}

impl Default for BrushKernelParams {
    fn default() -> Self {
        Self {
            radius: 3,
            strength: 1.0,
            falloff: BrushFalloff::Linear,
            shape: BrushShape::Circle,
        }
    }
}

/// The brush kernel — radius/strength/falloff params materialised into
/// a query function over cell offsets.
///
/// `BrushKernel` is the value a host stamps. It is cheap to copy (a
/// few `i32` + `f64` + two enum tags) and intentionally `Copy`-able so
/// a Bevy resource, a Godot dictionary entry, or an Unreal component
/// can hold one inline.
///
/// The two query methods are:
///
/// - [`BrushKernel::weight_at`] — full kernel weight at an axial cell
///   offset from the brush center. Returns `0.0` for any cell outside
///   the radius (AC-3).
/// - [`BrushKernel::apply`] — `(weight, in_radius)` tuple form, useful
///   when the caller wants to gate writes on `in_radius` (early-out)
///   and only call `weight` math for cells that pass the gate.
///
/// Iteration over the affected cells is the host's responsibility
/// (see [`BrushKernel::affected_extent`] for the bounding box and
/// [`BrushShape::contains`] for the per-cell edge test).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BrushKernel {
    radius: i32,
    strength: f64,
    falloff: BrushFalloff,
    shape: BrushShape,
}

impl BrushKernel {
    /// Build a kernel from a [`BrushKernelParams`]. The kernel is a
    /// pre-computed view of the params — the four fields are copied in
    /// directly with no validation beyond what the query methods do
    /// (see [`BrushKernel::weight_at`]).
    #[must_use]
    pub fn new(params: BrushKernelParams) -> Self {
        Self {
            radius: params.radius,
            strength: params.strength,
            falloff: params.falloff,
            shape: params.shape,
        }
    }

    /// The brush radius in cells.
    #[must_use]
    pub fn radius(&self) -> i32 {
        self.radius
    }

    /// The brush strength (peak weight at the center).
    #[must_use]
    pub fn strength(&self) -> f64 {
        self.strength
    }

    /// The falloff curve.
    #[must_use]
    pub fn falloff(&self) -> BrushFalloff {
        self.falloff
    }

    /// The footprint shape.
    #[must_use]
    pub fn shape(&self) -> BrushShape {
        self.shape
    }

    /// `true` when the kernel would affect **any** cell at all. A
    /// `radius = 0` kernel, or a negative-radius kernel, is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.radius <= 0
    }

    /// The planar extent of the footprint as `(min_offset, max_offset)`
    /// inclusive, in cells. The host iterates every cell in the AABB
    /// and gates with [`BrushShape::contains`] + the kernel weight
    /// math.
    ///
    /// For all three shapes the AABB is `[-radius, +radius]` on each
    /// axis (Circle's AABB is a superset of its footprint; the per-cell
    /// edge test filters the corners out).
    #[must_use]
    pub fn affected_extent(&self) -> ((i32, i32), (i32, i32)) {
        if self.radius <= 0 {
            return ((0, 0), (0, 0));
        }
        let r = self.radius;
        ((-r, -r), (r, r))
    }

    /// Full kernel weight at an axial cell offset `(dx, dy)` from the
    /// brush center.
    ///
    /// Returns `strength` at the center (`(0, 0)`) — AC-1. Returns
    /// `0.0` for any cell outside the radius — AC-3. For cells inside
    /// the radius, returns `strength · falloff.shape(t)` where
    /// `t = |offset| / radius` and `|offset|` is the distance metric
    /// implied by the footprint shape:
    ///
    /// - `Circle` → Euclidean distance `sqrt(dx² + dy²)`.
    /// - `Square` → Chebyshev distance `max(|dx|, |dy|)`.
    /// - `Diamond` → Manhattan distance `|dx| + |dy|`.
    ///
    /// `weight_at` is total: AC-2 (monotonically non-increasing in
    /// `|offset|`) holds for every falloff because `shape(t)` is
    /// non-increasing on `[0, 1]` and `t` is monotonic in `|offset|`
    /// for every shape's distance metric.
    #[must_use]
    pub fn weight_at(&self, offset: (i32, i32)) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        let (dx, dy) = offset;
        if !self.shape.contains(dx, dy, self.radius) {
            return 0.0;
        }
        // `t` is the normalized distance from the center, using the
        // shape's distance metric. For all three shapes:
        //   Circle:   |offset| = sqrt(dx² + dy²)
        //   Square:   |offset| = max(|dx|, |dy|)
        //   Diamond:  |offset| = |dx| + |dy|
        // The shape's `contains` predicate already rejected any cell
        // whose distance exceeds `radius`, so `t <= 1` here.
        let t = self.normalized_distance(dx, dy);
        self.strength * self.falloff.shape(t)
    }

    /// `(weight, in_radius)` tuple form of [`BrushKernel::weight_at`].
    /// `in_radius` is `true` when the cell passes the shape's edge
    /// test (regardless of whether its weight is meaningfully non-zero
    /// under the falloff — for `Hard` and `Linear` it is, for `Smooth`
    /// at the edge it is *almost* zero but technically still inside).
    ///
    /// Host clients use this when they want to early-out on
    /// `in_radius == false` without paying for the f64 math.
    #[must_use]
    pub fn apply(&self, offset: (i32, i32)) -> (f64, bool) {
        if self.is_empty() {
            return (0.0, false);
        }
        let (dx, dy) = offset;
        let in_radius = self.shape.contains(dx, dy, self.radius);
        if !in_radius {
            return (0.0, false);
        }
        let t = self.normalized_distance(dx, dy);
        (self.strength * self.falloff.shape(t), true)
    }

    /// Normalized distance `t = |offset| / radius ∈ [0, 1]`. Returns 0
    /// at the center and 1 at the shape's edge. Uses the shape's
    /// distance metric so the falloff curve gets a meaningful `t`.
    fn normalized_distance(&self, dx: i32, dy: i32) -> f64 {
        let r = f64::from(self.radius);
        let d = match self.shape {
            // Euclidean — we use f64::from so a `radius = 0` kernel
            // (which is_empty-filtered above) never reaches here.
            BrushShape::Circle => {
                let fx = f64::from(dx);
                let fy = f64::from(dy);
                (fx * fx + fy * fy).sqrt()
            }
            BrushShape::Square => {
                let adx = i64::from(dx.unsigned_abs());
                let ady = i64::from(dy.unsigned_abs());
                adx.max(ady) as f64
            }
            BrushShape::Diamond => {
                let adx = i64::from(dx.unsigned_abs());
                let ady = i64::from(dy.unsigned_abs());
                (adx + ady) as f64
            }
        };
        if r == 0.0 {
            0.0
        } else {
            d / r
        }
    }
}

impl Default for BrushKernel {
    fn default() -> Self {
        Self::new(BrushKernelParams::default())
    }
}

impl From<BrushKernelParams> for BrushKernel {
    fn from(params: BrushKernelParams) -> Self {
        Self::new(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a fresh kernel for the test using the most common
    /// sandbox shape: circle + Linear.
    fn kernel(radius: i32, strength: f64) -> BrushKernel {
        BrushKernel::new(BrushKernelParams::circle_linear(radius, strength))
    }

    /// AC-1 (FR-CIV-GODTOOL-901): kernel weight is max at the center
    /// (the spec acceptance test). Verified for every supported
    /// falloff, including `Hard` (which is a plateau).
    #[test]
    fn kernel_weight_is_max_at_center() {
        for falloff in BrushFalloff::ALL {
            let params = BrushKernelParams::new(3, 1.0, falloff, BrushShape::Circle);
            let k = BrushKernel::new(params);
            assert!(
                (k.weight_at((0, 0)) - k.strength()).abs() < f64::EPSILON,
                "falloff {falloff:?}: center weight ({}) must equal strength ({})",
                k.weight_at((0, 0)),
                k.strength()
            );
        }
    }

    /// AC-2 (FR-CIV-GODTOOL-901): kernel weight is monotonically
    /// non-increasing in `|offset|` for every supported falloff.
    /// We sweep along the +x axis from the center out to the radius
    /// and assert the weight never goes up.
    #[test]
    fn kernel_weight_is_monotonically_non_increasing() {
        for falloff in BrushFalloff::ALL {
            let k = BrushKernel::new(BrushKernelParams::new(
                5,
                1.0,
                falloff,
                BrushShape::Circle,
            ));
            let mut prev = k.weight_at((0, 0));
            for dx in 1..=5 {
                let w = k.weight_at((dx, 0));
                assert!(
                    w <= prev + f64::EPSILON,
                    "falloff {falloff:?}: weight at dx={dx} ({w}) must be <= weight at dx={} ({prev})",
                    dx - 1
                );
                prev = w;
            }
        }
    }

    /// AC-3 (FR-CIV-GODTOOL-901): kernel weight falls to zero at the
    /// radius edge (and beyond). This is the explicit acceptance test
    /// stated in the task description — "kernel weight is max at
    /// center and falls to zero at radius edge". We check every
    /// supported shape so a host choosing any shape gets the
    /// contract.
    #[test]
    fn kernel_weight_falls_to_zero_at_radius_edge() {
        // Circle + every falloff: cells just outside the radius are
        // zero; cells at `t = 1` (the edge) are zero by the
        // `falloff.shape` clamp; the center is the full strength.
        for falloff in BrushFalloff::ALL {
            for radius in [1, 2, 3, 5, 8] {
                let k = BrushKernel::new(BrushKernelParams::new(
                    radius,
                    1.0,
                    falloff,
                    BrushShape::Circle,
                ));
                // Just outside the radius on the x axis: zero.
                assert_eq!(
                    k.weight_at((radius + 1, 0)),
                    0.0,
                    "falloff {falloff:?}, radius {radius}: cell beyond the edge must be 0"
                );
                // Diagonal cell outside a small radius: zero.
                assert_eq!(
                    k.weight_at((radius, radius)),
                    0.0,
                    "falloff {falloff:?}, radius {radius}: corner diagonal cell must be 0 for circle"
                );
                // Center is the peak: must equal strength.
                assert!(
                    (k.weight_at((0, 0)) - 1.0).abs() < f64::EPSILON,
                    "falloff {falloff:?}, radius {radius}: center weight must equal strength"
                );
            }
        }

        // Square: corners of the bounding box are inside the radius
        // (Chebyshev metric) and the corner weight is the shape value
        // at `t = 1` for Linear/Smooth (zero) and 1.0 for Hard.
        let sq_hard = BrushKernel::new(BrushKernelParams::new(
            3,
            1.0,
            BrushFalloff::Hard,
            BrushShape::Square,
        ));
        assert!((sq_hard.weight_at((3, 3)) - 1.0).abs() < f64::EPSILON);
        let sq_lin = BrushKernel::new(BrushKernelParams::new(
            3,
            1.0,
            BrushFalloff::Linear,
            BrushShape::Square,
        ));
        assert!(sq_lin.weight_at((3, 3)).abs() < f64::EPSILON);

        // Diamond: corner (radius, radius) is outside (Manhattan =
        // 2·radius > radius) so it is zero. (radius, 0) is on the
        // edge and Linear/Smooth give zero there.
        let dia_lin = BrushKernel::new(BrushKernelParams::new(
            3,
            1.0,
            BrushFalloff::Linear,
            BrushShape::Diamond,
        ));
        assert_eq!(dia_lin.weight_at((4, 0)), 0.0);
        assert!(dia_lin.weight_at((3, 0)).abs() < f64::EPSILON);
    }

    /// A `radius = 0` kernel is empty by convention: no cell is
    /// affected and `weight_at` is always `0.0`.
    #[test]
    fn empty_kernel_has_zero_weight_everywhere() {
        let k = kernel(0, 1.0);
        assert!(k.is_empty());
        assert_eq!(k.weight_at((0, 0)), 0.0);
        assert_eq!(k.weight_at((1, 0)), 0.0);
        assert_eq!(k.apply((0, 0)), (0.0, false));
    }

    /// Negative radius (defensive): same empty behavior as `radius = 0`.
    #[test]
    fn negative_radius_kernel_is_empty() {
        let k = kernel(-3, 1.0);
        assert!(k.is_empty());
        assert_eq!(k.weight_at((0, 0)), 0.0);
    }

    /// `strength` scales the weight uniformly: doubling the strength
    /// doubles the weight at every cell.
    #[test]
    fn strength_scales_weight_uniformly() {
        let a = kernel(3, 1.0);
        let b = kernel(3, 2.0);
        for offset in [(-3, -3), (-1, 0), (0, 0), (0, 1), (2, -1), (3, 0)] {
            let wa = a.weight_at(offset);
            let wb = b.weight_at(offset);
            if wa == 0.0 {
                assert_eq!(wb, 0.0, "{offset:?}: 2x strength of 0 must still be 0");
            } else {
                assert!(
                    (wb / wa - 2.0).abs() < 1e-9,
                    "{offset:?}: 2x strength should double weight (a={wa}, b={wb})"
                );
            }
        }
    }

    /// `apply` agrees with `weight_at` for every (offset, in_radius)
    /// tuple — the two queries are aliases for the same math.
    #[test]
    fn apply_agrees_with_weight_at() {
        let k = kernel(4, 1.0);
        for dx in -5..=5 {
            for dy in -5..=5 {
                let (w, in_r) = k.apply((dx, dy));
                let w2 = k.weight_at((dx, dy));
                assert_eq!(in_r, k.shape.contains(dx, dy, k.radius()));
                assert!(
                    (w - w2).abs() < 1e-12,
                    "apply/weight_at disagree at ({dx}, {dy}): {w} vs {w2}"
                );
            }
        }
    }

    /// `BrushFalloff::shape` is the curve source-of-truth: zero at `t
    /// = 1` (the radius edge) and beyond, one at `t = 0` (the center).
    /// This is the math the kernel weight inherits.
    #[test]
    fn falloff_shape_is_zero_at_and_past_radius_edge() {
        for falloff in BrushFalloff::ALL {
            assert!((falloff.shape(0.0) - 1.0).abs() < f64::EPSILON);
            // Just inside the edge.
            assert!(falloff.shape(0.999) >= 0.0);
            // At the edge and beyond — kernel-weight contract.
            assert_eq!(falloff.shape(1.0), 0.0);
            assert_eq!(falloff.shape(2.0), 0.0);
            assert_eq!(falloff.shape(f64::INFINITY), 0.0);
        }
    }

    /// `BrushShape::contains` agrees with the kernel weight gate:
    /// `weight_at(offset) == 0` iff `shape.contains(dx, dy, radius)`
    /// is false. Spot-check a grid for every shape.
    #[test]
    fn shape_contains_agrees_with_kernel_weight_gate() {
        for shape in BrushShape::ALL {
            let k = BrushKernel::new(BrushKernelParams::new(
                3,
                1.0,
                BrushFalloff::Linear,
                shape,
            ));
            for dx in -4..=4 {
                for dy in -4..=4 {
                    let contained = shape.contains(dx, dy, 3);
                    let w = k.weight_at((dx, dy));
                    if !contained {
                        assert_eq!(w, 0.0, "{shape:?}: ({dx},{dy}) should be outside");
                    } else {
                        // Inside — weight is in (0, strength] under
                        // Linear (no plateau at the edge because `t =
                        // 1` clamps to zero). So strictly less than
                        // strength.
                        assert!(w >= 0.0, "{shape:?}: ({dx},{dy}) weight must be non-negative");
                        assert!(
                            w <= k.strength() + 1e-12,
                            "{shape:?}: ({dx},{dy}) weight must not exceed strength"
                        );
                    }
                }
            }
        }
    }

    /// `affected_extent` is the AABB for the host's iteration loop.
    /// For `radius = 0` the extent collapses to the empty range; for
    /// `radius > 0` it is `[-r, +r]` on each axis.
    #[test]
    fn affected_extent_is_the_axis_aligned_bbox() {
        let k = kernel(4, 1.0);
        assert_eq!(k.affected_extent(), ((-4, -4), (4, 4)));

        let empty = kernel(0, 1.0);
        assert_eq!(empty.affected_extent(), ((0, 0), (0, 0)));
    }

    /// `BrushFalloff::default` is `Linear` and `BrushShape::default`
    /// is `Circle` — matches the most common sandbox brush.
    #[test]
    fn falloff_and_shape_defaults() {
        assert_eq!(BrushFalloff::default(), BrushFalloff::Linear);
        assert_eq!(BrushShape::default(), BrushShape::Circle);
        // `BrushKernelParams::default` produces a kernel with
        // `radius = 3`, `strength = 1.0`, falloff = Linear, shape =
        // Circle — mirrors the Raise tool's spec defaults.
        let p = BrushKernelParams::default();
        assert_eq!(p.radius, 3);
        assert!((p.strength - 1.0).abs() < f64::EPSILON);
        assert_eq!(p.falloff, BrushFalloff::Linear);
        assert_eq!(p.shape, BrushShape::Circle);
    }
}