//! Streaming window policy — the "scale ladder" foundation.
//!
//! Lifts the chunk-streaming layer from a single-radius LRU page-in toward
//! the project's "no fixed world cap" target (FR-CIV-SCALE-001..008). The
//! world is treated as an **unbounded** volume; the *window* is a
//! camera-anchored AABB split into concentric **rings**, and each chunk's
//! role (rendered? meshed? simulated at full cadence? frozen?) is a
//! pure function of its **ring distance** from the anchor.
//!
//! This module is **pure**: no IO, no engine types, no GPU. It is the
//! single source of truth for:
//!
//! - [`ring_distance`] — Chebyshev distance with a vertical weight
//!   (worlds are mostly flat heightfields; vertical distance costs more).
//! - [`WindowPolicy`] — the named, serialisable config (ring radii, seam
//!   width, sim-LOD cadence, prefetch policy).
//! - [`ChunkState`] — the lifecycle state machine for a chunk
//!   (`Unloaded` → `Resident` → `Meshed` → `Fading` → `Evicting` → `Evicted`).
//! - [`SimCohort`] — derived from ring distance; full sim / coarse sim / frozen.
//! - [`EvictionKey`] / [`EvictionKey::new`] — the comparator the streaming
//!   layer uses to decide which chunk to drop under pressure. Default is
//!   ring-distance with LRU as a same-ring tie-breaker.
//!
//! Wiring it into [`crate::stream::StreamingWorld`] and
//! `clients/bevy-ref/src/voxel_stream.rs` is a follow-up slice. This slice
//! lands the types and the property tests so the policy can be evolved
//! without re-deriving the math every time.
//!
//! ## Determinism
//!
//! Every public function is a pure function of `(coord, anchor, policy)`.
//! Two clients with the same seed and the same `(camera_anchor, policy)`
//! MUST produce the same ring assignment, the same `ChunkState`
//! transition set, and the same [`EvictionKey`] ordering. Replay builds
//! can reconstruct the working set bit-identically from the anchor
//! trace alone — no side-effectful load order leaks in.
//!
//! ## Architecture reference
//!
//! See `docs/design/streaming-window.md` for the full design (rings,
//! sim-LOD cohorts, prefetch cones, seam blending) and the alternatives
//! considered (fixed-grid tiers, clipmap rings, octree cut, LRU vs
//! ring-distance eviction).

#![forbid(unsafe_code)]

use phenotype_voxel::ChunkCoord;
use serde::{Deserialize, Serialize};

pub mod io;
pub mod plan;
pub mod ring_iter;
pub use ring_iter::RingIter;

/// Lifecycle state for a chunk in the streaming window.
///
/// The state machine is intentionally small and **derivable** — every
/// state is a pure function of `(ring_distance, mesh_ring, sim_ring,
/// fading_after)`. The streaming layer is free to track its own internal
/// state (e.g. LRU position, last-touched tick), but the *visible* state
/// the policy reports is one of these variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkState {
    /// Not in the resident set. Will regen from seed if requested.
    Unloaded,
    /// In the resident set, not yet meshed (or meshed and already
    /// despawned, e.g. a back-facing chunk).
    Resident,
    /// In the resident set, mesh is alive (Bevy entity spawned).
    Meshed,
    /// Mesh is alive but alpha is being lowered for a ring shrink. The
    /// streaming layer holds the chunk `Resident` for `ticks_remaining`
    /// more ticks so the renderer's blend ramp can complete.
    Fading {
        /// 1..=`WindowPolicy::fade_ticks` ticks left in the fade ramp.
        ticks_remaining: u8,
    },
    /// Marked for eviction this tick; mesh despawn scheduled.
    Evicting,
    /// Removed from `resident`; persisted to disk if dirty. Terminal
    /// (a coord can re-enter the cycle via `Resident` after regen).
    Evicted,
}

impl ChunkState {
    /// True if a chunk in this state holds a live mesh in the renderer.
    #[must_use]
    pub const fn has_mesh(self) -> bool {
        matches!(self, Self::Meshed | Self::Fading { .. })
    }

    /// True if a chunk in this state occupies RAM (counted against the
    /// active budget).
    #[must_use]
    pub const fn is_resident(self) -> bool {
        matches!(
            self,
            Self::Resident | Self::Meshed | Self::Fading { .. } | Self::Evicting
        )
    }
}

/// Sim-LOD cohort, derived from ring distance. A chunk's cohort
/// determines which tick path the simulator takes (full per-voxel CA,
/// coarse statistical gestalt, or frozen).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SimCohort {
    /// Every tick, per-voxel CA, full agent tick.
    FullSim,
    /// Every `step_multiplier`-th tick, statistical gestalt only.
    CoarseSim {
        /// Tick-rate divisor vs. full sim. 2 = every other tick, 4 =
        /// every 4th tick, etc. Configurable per ring in a follow-up slice.
        step_multiplier: u8,
    },
    /// No sim tick; mass is conserved trivially (no writes, no decay).
    Frozen,
}

/// Streaming-window policy.
///
/// All fields are `u8` (or `i8` for the signed forward-cone threshold) so
/// the struct is `Copy`, serialisable, and round-trips bit-identically
/// through `bincode` for replay/manifest persistence. Defaults are tuned
/// to match `WORLD_DIMS_SMALL`'s working set (see
/// `docs/design/streaming-window.md` §3.5 / §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowPolicy {
    /// Innermost ring fully meshed at LOD 0. Chunks at ring `≤
    /// mesh_ring` are `Meshed` (or `Fading` on a shrink).
    pub mesh_ring: u8,
    /// Innermost ring running full-sim cadence.
    pub sim_ring: u8,
    /// Outermost ring still on coarse-sim. Must be `≥ sim_ring`. Chunks
    /// past this are `Frozen`.
    pub coarse_ring: u8,
    /// Width of the horizon-fade seam between adjacent rings, in chunks.
    /// `0` = hard LOD boundary (no fade). See design §3.7.
    pub seam_chunks: u8,
    /// Vertical weight for the ring-distance metric. World-space
    /// `dy` is multiplied by this before being compared with `dx`/`dz`,
    /// so a single step up/down "costs" `vy_weight` horizontal steps.
    /// Default `2` for flat heightfields. `1` = uniform Chebyshev cube.
    pub vy_weight: u8,
    /// Coarse-sim tick divisor (e.g. `2` = every other tick).
    pub sim_lod_step: u8,
    /// How many rings past `mesh_ring` the prefetch cone reaches. `0`
    /// disables prefetch. See design §3.5.
    pub prefetch_ring: u8,
    /// Forward-cone half-angle for prefetch, in Q0.7 signed fixed-point
    /// (`-128..=127`, maps to `-1.0..~+0.99`). Chunks outside the cone
    /// (i.e. behind the camera) are skipped from prefetch. `0` =
    /// hemisphere (anything in front of the camera).
    pub forward_cone_cos_theta: i8,
    /// Fade ramp length for the `Fading` state, in ticks. Chunks
    /// dropping out of the mesh ring are held `Resident` for this long
    /// while the renderer ramps alpha 1.0 → 0.0. `0` = no fade (instant
    /// despawn on ring exit, i.e. the pre-existing behaviour).
    pub fade_ticks: u8,
}

impl Default for WindowPolicy {
    /// Defaults match the MVP: 1-ring fully meshed (≈ 0.5 mi² at
    /// `base_voxel_m = 4.0`), 1-ring full-sim, 2-ring coarse-sim, no
    /// prefetch, no fade (instant despawn). The defaults are
    /// deliberately conservative — turning prefetch and fade on in a
    /// follow-up is a config flip, not a code change.
    fn default() -> Self {
        Self {
            mesh_ring: 1,
            sim_ring: 1,
            coarse_ring: 2,
            seam_chunks: 1,
            vy_weight: 2,
            sim_lod_step: 2,
            prefetch_ring: 0,
            forward_cone_cos_theta: 0,
            fade_ticks: 0,
        }
    }
}

impl WindowPolicy {
    /// Construct a policy with explicit invariants validated. Returns
    /// `Err(PolicyError)` on out-of-range or inconsistent values. Use
    /// `Self::default()` or `Self { ... }` directly when the caller has
    /// already validated.
    #[allow(clippy::too_many_arguments)]
    pub fn checked(
        mesh_ring: u8,
        sim_ring: u8,
        coarse_ring: u8,
        seam_chunks: u8,
        vy_weight: u8,
        sim_lod_step: u8,
        prefetch_ring: u8,
        forward_cone_cos_theta: i8,
        fade_ticks: u8,
    ) -> Result<Self, PolicyError> {
        if vy_weight == 0 {
            return Err(PolicyError::ZeroVyWeight);
        }
        if sim_lod_step == 0 {
            return Err(PolicyError::ZeroSimLodStep);
        }
        if sim_ring > coarse_ring {
            return Err(PolicyError::SimRingAboveCoarseRing);
        }
        if !(-128..=127).contains(&forward_cone_cos_theta) {
            return Err(PolicyError::ForwardConeOutOfRange);
        }
        Ok(Self {
            mesh_ring,
            sim_ring,
            coarse_ring,
            seam_chunks,
            vy_weight,
            sim_lod_step,
            prefetch_ring,
            forward_cone_cos_theta,
            fade_ticks,
        })
    }

    /// Classify a chunk's render-LOD state against the policy. This is
    /// the function the renderer's per-frame plan calls.
    ///
    /// The result is a **function** of `(coord, anchor, policy)` —
    /// no side channels. Two clients with the same policy and the same
    /// anchor derive the same state.
    #[must_use]
    pub const fn classify(&self, coord: ChunkCoord, anchor: ChunkCoord) -> ChunkState {
        let ring = ring_distance(coord, anchor, self.vy_weight);
        if ring <= self.mesh_ring as u32 {
            ChunkState::Meshed
        } else if ring <= (self.mesh_ring as u32).saturating_add(self.seam_chunks as u32) {
            // The seam band: chunks that are past the mesh ring but within
            // `seam_chunks` of it are marked `Fading` so the renderer can
            // ramp their alpha. With `fade_ticks = 0` the renderer is
            // expected to despawn immediately; the state is still
            // semantically `Fading` for one tick.
            if self.fade_ticks == 0 {
                ChunkState::Resident
            } else {
                ChunkState::Fading {
                    ticks_remaining: self.fade_ticks,
                }
            }
        } else {
            ChunkState::Unloaded
        }
    }

    /// Derive the sim cohort from ring distance. A pure function of
    /// `(coord, anchor, policy)`.
    #[must_use]
    pub const fn sim_cohort(&self, coord: ChunkCoord, anchor: ChunkCoord) -> SimCohort {
        let ring = ring_distance(coord, anchor, self.vy_weight);
        if ring <= self.sim_ring as u32 {
            SimCohort::FullSim
        } else if ring <= self.coarse_ring as u32 {
            SimCohort::CoarseSim {
                step_multiplier: self.sim_lod_step,
            }
        } else {
            SimCohort::Frozen
        }
    }

    /// True if `coord` is in the prefetch cone (i.e. within
    /// `mesh_ring + prefetch_ring` and in front of the camera, where
    /// "in front" is defined by the forward-cone threshold).
    ///
    /// `forward_q7` is the unit look direction in Q0.7 signed
    /// fixed-point (`-128..=127` per axis). The check is
    /// `dot(forward_q7, to_chunk) > cos_theta * |to_chunk|` evaluated
    /// in integer space — no f32 in the policy path, so replay stays
    /// deterministic.
    ///
    /// Returns `false` if `prefetch_ring == 0` (prefetch disabled) or
    /// the chunk is past the prefetch ring.
    #[must_use]
    pub const fn in_prefetch_cone(
        &self,
        coord: ChunkCoord,
        anchor: ChunkCoord,
        forward_q7: [i32; 3],
    ) -> bool {
        if self.prefetch_ring == 0 {
            return false;
        }
        let ring = ring_distance(coord, anchor, self.vy_weight);
        if ring <= self.mesh_ring as u32 {
            return true; // already in the inner ring; cone test is moot
        }
        if ring > (self.mesh_ring as u32).saturating_add(self.prefetch_ring as u32) {
            return false;
        }
        // Weighted direction vector (in the same metric the ring uses).
        let dx = coord.cx - anchor.cx;
        let dy = (coord.cy - anchor.cy) * (self.vy_weight as i32);
        let dz = coord.cz - anchor.cz;
        // Integer dot in Q7 * raw-chunks. The forward vector is also
        // Q7, so the product is Q14. We compare against
        // `cos_theta_q7 * |dir|` (L1-norm, conservative lower bound
        // for |dir|) scaled to Q14. For a hemisphere (`cos_theta = 0`)
        // the test degenerates to `dot > 0` — anything in front of
        // the camera qualifies. For a tighter cone (`cos > 0`) we
        // require the dot to exceed `cos * L1 * 128`.
        let dot_q14 = forward_q7[0] * dx + forward_q7[1] * dy + forward_q7[2] * dz;
        let l1 = dx.abs() + dy.abs() + dz.abs();
        let cos_q7 = self.forward_cone_cos_theta as i32;
        if cos_q7 > 0 {
            dot_q14 > cos_q7.saturating_mul(l1).saturating_mul(128)
        } else {
            dot_q14 > 0
        }
    }
}

/// Errors from [`WindowPolicy::checked`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyError {
    /// `vy_weight` was 0 (would cause divide-by-zero in the comparator).
    ZeroVyWeight,
    /// `sim_lod_step` was 0 (would cause divide-by-zero in the cohort).
    ZeroSimLodStep,
    /// `sim_ring > coarse_ring` (the full-sim band must be a sub-band of
    /// the coarse-sim band).
    SimRingAboveCoarseRing,
    /// `forward_cone_cos_theta` was outside the Q0.7 signed range.
    ForwardConeOutOfRange,
}

impl core::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            Self::ZeroVyWeight => "vy_weight must be ≥ 1",
            Self::ZeroSimLodStep => "sim_lod_step must be ≥ 1",
            Self::SimRingAboveCoarseRing => "sim_ring must be ≤ coarse_ring",
            Self::ForwardConeOutOfRange => "forward_cone_cos_theta must be in -128..=127",
        };
        f.write_str(s)
    }
}

impl std::error::Error for PolicyError {}

/// Chebyshev distance with a vertical weight.
///
/// `ring_distance(coord, anchor, vy_weight) = max(|Δx|, |Δy| * vy_weight, |Δz|)`.
///
/// Worlds are mostly flat heightfields; a vertical step costs more than a
/// horizontal step, so the policy's inner ring doesn't fill the entire Y
/// axis as the camera flies. With `vy_weight = 1` the metric is a pure
/// Chebyshev cube; with `vy_weight = 2` a 1-chunk vertical step is
/// equivalent to 2 horizontal steps.
///
/// `const fn` so the renderer (e.g. a `const` LOD table) and the policy
/// share one definition. `vy_weight = 0` is treated as `1` so callers
/// that bypass [`WindowPolicy::checked`] (which rejects it) cannot
/// divide by zero.
#[must_use]
pub const fn ring_distance(coord: ChunkCoord, anchor: ChunkCoord, vy_weight: u8) -> u32 {
    let w = if vy_weight == 0 { 1u32 } else { vy_weight as u32 };
    let dx = (coord.cx - anchor.cx).unsigned_abs();
    let dz = (coord.cz - anchor.cz).unsigned_abs();
    let dy = (coord.cy - anchor.cy).unsigned_abs() * w;
    // Manual max-of-three so this stays `const fn` (u32::max isn't
    // const-stable on stable Rust as of 1.96 — see rust-lang/rust#143874).
    let m = if dx > dz { dx } else { dz };
    if m > dy { m } else { dy }
}

/// Eviction comparator. Returns an ordering key: chunks with **smaller**
/// keys are evicted **first** under pressure.
///
/// The default policy is **ring-distance** (chunks far from the anchor
/// go first); within the same ring, the older `lru_pos` (smaller is
/// colder) goes first. This matches user expectations: a far chunk is
/// evicted before a near one even if the near one was touched earlier
/// in the frame, and a chunk inside the same ring keeps a stable
/// LRU order so the test suite can assert determinism.
///
/// LRU is the tie-breaker, not the primary signal — the design doc's
/// §3.6 / §4.4 explain why.
///
/// **Comparator direction**: the implementation uses `Ord` such that
/// `far < near` (i.e. a `BinaryHeap<EvictionKey>` keeps the hottest
/// chunks at the top and `pop()` returns the next eviction target). A
/// `Vec<EvictionKey>::sort()` puts eviction targets at the front of
/// the slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EvictionKey {
    /// Ring distance from the current anchor. Sort key (larger ring
    /// sorts smaller → evicts first).
    pub ring: u32,
    /// LRU position within the ring. Lower = colder. Tie-breaker
    /// (smaller lru_pos sorts smaller → evicts first within a ring).
    pub lru_pos: u32,
}

impl EvictionKey {
    /// Build an eviction key for a chunk.
    ///
    /// `lru_pos` is the chunk's position in the streaming layer's
    /// per-ring LRU. The streaming layer tracks this; the policy just
    /// folds it into the key.
    #[must_use]
    pub const fn new(coord: ChunkCoord, anchor: ChunkCoord, vy_weight: u8, lru_pos: u32) -> Self {
        Self {
            ring: ring_distance(coord, anchor, vy_weight),
            lru_pos,
        }
    }
}

impl Ord for EvictionKey {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Larger ring = evict first (chunks far from the anchor are the
        // first to go). Within the same ring, larger lru_pos is colder
        // (a higher LRU position means a more recently-touched chunk, so
        // the *smaller* lru_pos is colder, evicted first).
        //
        // So the eviction priority order is: (larger ring, smaller lru_pos).
        // That maps to a *descending* ring sort, then *ascending* lru_pos.
        // We invert the ring comparison.
        other
            .ring
            .cmp(&self.ring)
            .then(self.lru_pos.cmp(&other.lru_pos))
    }
}

impl PartialOrd for EvictionKey {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coord(cx: i32, cy: i32, cz: i32) -> ChunkCoord {
        ChunkCoord { cx, cy, cz }
    }

    // ---- ring_distance ----

    #[test]
    fn ring_distance_zero_at_anchor() {
        let a = coord(3, -1, 7);
        assert_eq!(ring_distance(a, a, 2), 0);
    }

    #[test]
    fn ring_distance_horizontal_chebyshev() {
        // (0,0,0) → (5,0,3) with vy_weight=2: max(5, 0*2, 3) = 5
        assert_eq!(ring_distance(coord(5, 0, 3), coord(0, 0, 0), 2), 5);
        assert_eq!(ring_distance(coord(0, 0, 0), coord(5, 0, 3), 2), 5);
    }

    #[test]
    fn ring_distance_vertical_weighted() {
        // (0,0,0) → (1,1,1) with vy_weight=2: max(1, 1*2, 1) = 2
        assert_eq!(ring_distance(coord(1, 1, 1), coord(0, 0, 0), 2), 2);
        // With vy_weight=1, it's max(1, 1, 1) = 1
        assert_eq!(ring_distance(coord(1, 1, 1), coord(0, 0, 0), 1), 1);
    }

    #[test]
    fn ring_distance_negative_coords() {
        let a = coord(-3, -2, -4);
        let b = coord(2, 1, 1);
        // dx=5, dy=3*2=6, dz=5 → 6
        assert_eq!(ring_distance(b, a, 2), 6);
    }

    #[test]
    fn ring_distance_zero_vy_weight_falls_back_to_one() {
        // The const fn degrades vy_weight=0 to 1 (we error in `checked`,
        // but `ring_distance` is the *math* primitive and must be
        // safe for callers that bypass the validator).
        assert_eq!(ring_distance(coord(2, 5, 0), coord(0, 0, 0), 0), 5);
    }

    // ---- WindowPolicy::classify determinism ----

    #[test]
    fn classify_deterministic_for_same_inputs() {
        let policy = WindowPolicy::default();
        let a = coord(2, 0, 3);
        let anchor = coord(0, 0, 0);
        let s1 = policy.classify(a, anchor);
        let s2 = policy.classify(a, anchor);
        assert_eq!(s1, s2);
        assert_eq!(s1, ChunkState::Unloaded); // ring=3 > mesh_ring=1 + seam=1
        // Re-check the boundary: ring=2 (= mesh_ring+1) is in the
        // seam band; ring=3 is past it.
        let ring2 = coord(0, 1, 0); // dy=1*vy_weight=2 → ring 2
        assert_eq!(
            policy.classify(ring2, anchor),
            ChunkState::Resident,
            "ring=2 is the seam band"
        );
        let ring1 = coord(0, 0, 1); // ring 1
        assert_eq!(
            policy.classify(ring1, anchor),
            ChunkState::Meshed,
            "ring=1 is the mesh ring"
        );
    }

    #[test]
    fn classify_invariant_under_anchor_translation() {
        // Classify is translation-invariant: shifting both coord and
        // anchor by the same vector gives the same state.
        let policy = WindowPolicy::default();
        let c = coord(2, 0, 3);
        let a = coord(0, 0, 0);
        let s_at_origin = policy.classify(c, a);
        let s_shifted = policy.classify(coord(7, 4, -1), coord(5, 4, -4));
        assert_eq!(s_at_origin, s_shifted);
    }

    // ---- SimCohort ----

    #[test]
    fn sim_cohort_full_for_inner_ring() {
        let policy = WindowPolicy::default();
        let a = coord(0, 0, 0);
        assert_eq!(policy.sim_cohort(coord(0, 0, 0), a), SimCohort::FullSim);
        assert_eq!(policy.sim_cohort(coord(1, 0, 0), a), SimCohort::FullSim);
    }

    #[test]
    fn sim_cohort_coarse_for_middle_ring() {
        let policy = WindowPolicy::default();
        let a = coord(0, 0, 0);
        // ring 2 = coarse (coarse_ring = 2 default)
        assert_eq!(
            policy.sim_cohort(coord(0, 0, 2), a),
            SimCohort::CoarseSim { step_multiplier: 2 }
        );
    }

    #[test]
    fn sim_cohort_frozen_for_far_ring() {
        let policy = WindowPolicy::default();
        let a = coord(0, 0, 0);
        // ring 3 = frozen (coarse_ring = 2, so > 2 → Frozen)
        assert_eq!(policy.sim_cohort(coord(0, 0, 3), a), SimCohort::Frozen);
    }

    // ---- WindowPolicy::checked validation ----

    #[test]
    fn checked_rejects_zero_vy_weight() {
        let err = WindowPolicy::checked(1, 1, 2, 1, 0, 2, 0, 0, 0).unwrap_err();
        assert_eq!(err, PolicyError::ZeroVyWeight);
    }

    #[test]
    fn checked_rejects_zero_sim_lod_step() {
        let err = WindowPolicy::checked(1, 1, 2, 1, 2, 0, 0, 0, 0).unwrap_err();
        assert_eq!(err, PolicyError::ZeroSimLodStep);
    }

    #[test]
    fn checked_rejects_sim_above_coarse() {
        let err = WindowPolicy::checked(1, 3, 2, 1, 2, 2, 0, 0, 0).unwrap_err();
        assert_eq!(err, PolicyError::SimRingAboveCoarseRing);
    }

    #[test]
    fn checked_accepts_consistent_policy() {
        assert!(WindowPolicy::checked(1, 1, 2, 1, 2, 2, 0, 0, 0).is_ok());
        // sim_ring == coarse_ring is allowed: only full + frozen.
        assert!(WindowPolicy::checked(1, 1, 1, 1, 2, 2, 0, 0, 0).is_ok());
    }

    // ---- EvictionKey ----

    #[test]
    fn eviction_key_far_ring_evicted_before_near() {
        // ring 5 evicted before ring 1, regardless of LRU position.
        let near = EvictionKey::new(coord(1, 0, 0), coord(0, 0, 0), 2, 999);
        let far = EvictionKey::new(coord(5, 0, 0), coord(0, 0, 0), 2, 0);
        assert!(far < near, "far (lru=0) must evict before near (lru=999)");
    }

    #[test]
    fn eviction_key_lru_tiebreaker_within_ring() {
        // Same ring: smaller lru_pos is evicted first.
        let cold = EvictionKey::new(coord(2, 0, 0), coord(0, 0, 0), 2, 0);
        let warm = EvictionKey::new(coord(2, 0, 0), coord(0, 0, 0), 2, 5);
        assert!(cold < warm);
        // Stable, total order.
        let mut keys = [warm, cold];
        keys.sort();
        assert_eq!(keys, [cold, warm]);
    }

    #[test]
    fn eviction_key_stable_under_repeated_construction() {
        // Determinism: building the same key twice yields the same value.
        let k1 = EvictionKey::new(coord(-3, 1, 2), coord(1, 0, 0), 2, 7);
        let k2 = EvictionKey::new(coord(-3, 1, 2), coord(1, 0, 0), 2, 7);
        assert_eq!(k1, k2);
    }

    // ---- ChunkState helpers ----

    #[test]
    fn chunk_state_has_mesh_and_residency() {
        assert!(!ChunkState::Unloaded.has_mesh());
        assert!(!ChunkState::Resident.has_mesh());
        assert!(ChunkState::Meshed.has_mesh());
        assert!(ChunkState::Fading { ticks_remaining: 1 }.has_mesh());
        assert!(!ChunkState::Evicting.has_mesh());
        assert!(!ChunkState::Evicted.has_mesh());

        assert!(!ChunkState::Unloaded.is_resident());
        assert!(ChunkState::Resident.is_resident());
        assert!(ChunkState::Meshed.is_resident());
        assert!(ChunkState::Fading { ticks_remaining: 1 }.is_resident());
        assert!(ChunkState::Evicting.is_resident());
        assert!(!ChunkState::Evicted.is_resident());
    }

    // ---- FR-CIV-SCALE slice 5 coverage ----

    #[test]
    fn fr_civ_scale_001_default_policy_guarantees_minimal_resident_area() {
        let policy = WindowPolicy::default();
        let anchor = coord(0, 0, 0);

        // At minimum, a non-empty residency window exists with a
        // 256^3-usable inner mesh band and a bounded seam band.
        assert_eq!(policy.classify(anchor, anchor), ChunkState::Meshed);
        assert_eq!(
            policy.classify(coord(1, 0, 0), anchor),
            ChunkState::Meshed,
            "mesh_ring=1 should keep adjacent ring-1 chunks resident+meshed"
        );
        assert_eq!(
            policy.classify(coord(2, 0, 0), anchor),
            ChunkState::Resident,
            "with seam_chunks=1 and fade_ticks=0, ring 2 is in the seam band and should be resident"
        );
        assert_eq!(
            policy.classify(coord(3, 0, 0), anchor),
            ChunkState::Unloaded,
            "ring 3 is outside mesh+seam and should stay unloaded"
        );
    }

    #[test]
    fn fr_civ_scale_002_no_fixed_world_cap_allows_arbitrary_anchor_coords() {
        let policy = WindowPolicy::default();
        let anchor = coord(10_000, 5_000, -12_000);
        let probe = coord(3_211, 4_212, -3_213);
        let anchor_2 = coord(-9_000, -4_500, 15_000);
        let probe_2 = coord(anchor_2.cx + 3_211, anchor_2.cy + 4_212, anchor_2.cz - 3_213);

        // Same relative offset from far-apart anchors yields same ring band.
        assert_eq!(
            policy.classify(probe, anchor),
            policy.classify(probe_2, anchor_2)
        );
        assert_eq!(policy.sim_cohort(probe, anchor), policy.sim_cohort(probe_2, anchor_2));
        assert_eq!(
            policy.in_prefetch_cone(probe, anchor, [32, 0, 0]),
            policy.in_prefetch_cone(probe_2, anchor_2, [32, 0, 0])
        );
    }

    #[test]
    fn fr_civ_scale_003_lod_seams_are_modelled_as_fading_band() {
        let policy = WindowPolicy {
            fade_ticks: 2,
            ..WindowPolicy::default()
        };
        let anchor = coord(0, 0, 0);
        // Ring distance is `max(|dx|, |dy|*vy_weight, |dz|)` with `vy_weight=2`.
        let near_chunk = coord(1, 0, 0); // ring 1 = mesh_ring
        let seam_chunk = coord(2, 0, 0); // ring 2 = mesh_ring + seam_chunks
        let far_chunk = coord(3, 0, 0); // ring 3 = past mesh + seam band

        assert_eq!(
            policy.classify(near_chunk, anchor),
            ChunkState::Meshed,
            "ring=1 (inside mesh_ring) should be meshed"
        );
        assert_eq!(
            policy.classify(seam_chunk, anchor),
            ChunkState::Fading {
                ticks_remaining: 2
            },
            "ring=2 should be fading seam band with fade_ticks set"
        );
        assert_eq!(
            policy.classify(far_chunk, anchor),
            ChunkState::Unloaded,
            "ring=3 (mesh+seam=2) should be unloaded"
        );
    }

    #[test]
    fn fr_civ_scale_004_sim_lod_transitions_from_full_to_coarse_to_frozen_by_distance() {
        let policy = WindowPolicy {
            sim_ring: 2,
            coarse_ring: 4,
            sim_lod_step: 4,
            ..WindowPolicy::default()
        };
        let anchor = coord(0, 0, 0);

        assert_eq!(
            policy.sim_cohort(coord(0, 0, 0), anchor),
            SimCohort::FullSim,
            "inside full-sim ring should stay high-fidelity"
        );
        // ring 3 is between sim_ring=2 and coarse_ring=4 → CoarseSim.
        assert_eq!(
            policy.sim_cohort(coord(3, 0, 0), anchor),
            SimCohort::CoarseSim { step_multiplier: 4 },
            "outside mesh/sim ring but within coarse_ring should coarse-step"
        );
        // ring 5 is past coarse_ring=4 → Frozen.
        assert_eq!(
            policy.sim_cohort(coord(5, 0, 0), anchor),
            SimCohort::Frozen,
            "outside coarse_ring should freeze state updates"
        );
    }

    // ---- Prefetch cone ----

    #[test]
    fn prefetch_disabled_when_prefetch_ring_zero() {
        let policy = WindowPolicy::default();
        // Default prefetch_ring = 0 → in_prefetch_cone always false.
        assert!(!policy.in_prefetch_cone(coord(5, 0, 0), coord(0, 0, 0), [0, 0, 128]));
    }

    #[test]
    fn prefetch_cone_in_front_of_camera() {
        // forward = +X (looking east). Chunks east of the anchor are in
        // the cone; chunks west are not.
        let policy = WindowPolicy {
            prefetch_ring: 2,
            ..WindowPolicy::default()
        };
        let anchor = coord(0, 0, 0);
        let forward = [128, 0, 0]; // +X, Q7
        // East chunks (ring 2-3, in front): in cone.
        assert!(policy.in_prefetch_cone(coord(3, 0, 0), anchor, forward));
        // West chunks: out of cone.
        assert!(!policy.in_prefetch_cone(coord(-3, 0, 0), anchor, forward));
        // South chunks: out of cone (dot = 0, hemisphere threshold).
        assert!(!policy.in_prefetch_cone(coord(0, 0, -3), anchor, forward));
        // Inner ring: in cone regardless of direction (already covered).
        assert!(policy.in_prefetch_cone(coord(1, 0, 0), anchor, forward));
        assert!(policy.in_prefetch_cone(coord(-1, 0, 0), anchor, forward));
    }
}
