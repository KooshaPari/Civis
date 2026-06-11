//! Prefetch planning + scale report helpers for the streaming window.
//!
//! Lifts the camera/sim velocity vectors from "inert input" to
//! first-class inputs to the [`crate::window::WindowPolicy`], so the
//! streaming layer can warm chunks *before* the camera arrives:
//!
//! - [`prefetch_set`] — given a `(anchor, velocity_chunks_per_tick,
//!   policy)`, returns the deterministic set of chunk coords the
//!   streaming layer should page in over the next `ticks` window. Pure
//!   function: no IO, no engine, no GPU. Implements FR-CIV-SCALE-005.
//! - [`ScaleReport`] — running tracker for `max_resident_chunks` and a
//!   bounded P99 estimator for tick time (in microseconds), giving the
//!   scale benchmark a stable, replay-safe summary. Implements
//!   FR-CIV-SCALE-008.
//!
//! Both types are pure and `#[derive]`-only — no engine coupling. The
//! streaming layer is responsible for calling them on the right tick
//! boundary and for routing the prefetch set into the same
//! [`crate::window::EvictionKey`] comparator the active set uses.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use crate::window::{ring_distance, RingIter, WindowPolicy};
use phenotype_voxel::ChunkCoord;

/// Number of ticks the prefetch planner looks ahead by default.
///
/// The streaming layer is free to override; the constant exists so
/// tests and docs have a single source of truth. `4` matches the
/// `sim_lod_step = 2` default in [`WindowPolicy`] — a chunk the
/// camera is approaching in 2-4 ticks is exactly the band the coarse
/// sim cohort is about to read.
pub const DEFAULT_PREFETCH_TICKS: u32 = 4;

/// Camera/sim velocity, in chunks per tick, in chunk coordinates.
///
/// The three axes are independent (e.g. `{x: 0, y: 0, z: 1}` for a
/// camera flying north at one chunk per tick). Fractional components
/// are not supported; the planner rounds down (i.e. a velocity of
/// `0.7` is treated as `0`, no prefetch). The cell matches the
/// `vy_weight` in [`WindowPolicy`]: vertical chunks are weighted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VelocityChunksPerTick {
    /// East/west velocity. Positive = +X (east).
    pub x: i32,
    /// Up/down velocity. Positive = +Y (up). Scaled by `vy_weight`.
    pub y: i32,
    /// North/south velocity. Positive = +Z (north).
    pub z: i32,
}

impl Default for VelocityChunksPerTick {
    /// Zero velocity — prefetch planner degenerates to "nothing to warm".
    fn default() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}

impl VelocityChunksPerTick {
    /// True if all components are zero (no prefetch needed).
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.x == 0 && self.y == 0 && self.z == 0
    }

    /// Render a deterministic, lexicographic key for the prefetch set
    /// so two calls with the same inputs produce the same set. The
    /// key is `(dx, dy, dz)` in the *anchor frame*; callers that want
    /// to sort the resulting set should sort by this key.
    #[must_use]
    pub const fn into_iter_chunks(self, anchor: ChunkCoord, ticks: u32) -> ChunkOffsetIter {
        ChunkOffsetIter {
            anchor,
            vx: self.x,
            vy: self.y,
            vz: self.z,
            ticks,
            current_tick: 1,
        }
    }
}

/// Iterator over chunk offsets at each tick along a velocity vector.
///
/// Yields `(tick, ChunkCoord)` where `ChunkCoord` is the chunk the
/// anchor will sit on at that tick, given the velocity.
pub struct ChunkOffsetIter {
    anchor: ChunkCoord,
    vx: i32,
    vy: i32,
    vz: i32,
    ticks: u32,
    current_tick: u32,
}

impl Iterator for ChunkOffsetIter {
    type Item = (u32, ChunkCoord);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_tick > self.ticks {
            return None;
        }
        let t = self.current_tick;
        let coord = ChunkCoord {
            cx: self.anchor.cx.saturating_add(self.vx.saturating_mul(t as i32)),
            cy: self.anchor.cy.saturating_add(self.vy.saturating_mul(t as i32)),
            cz: self.anchor.cz.saturating_add(self.vz.saturating_mul(t as i32)),
        };
        self.current_tick += 1;
        Some((t, coord))
    }
}

/// Compute the deterministic prefetch set the streaming layer should
/// warm for the next `ticks` ticks, given the camera's current
/// `anchor`, velocity, and the active `policy`.
///
/// The set is the union, per future tick, of the **outer ring** at
/// distance `mesh_ring + prefetch_ring` from the future-anchor. We
/// pick the **outer** ring (not the inner mesh ring) because:
/// - The inner ring is already paged in by [`WindowPolicy::classify`]
///   for the *current* anchor; the streaming layer does not need a
///   hint to keep it warm.
/// - The outer ring is where the future-anchor's inner ring will
///   transition out — the chunks we want to be already in RAM when
///   the camera arrives.
///
/// Output is sorted lexicographically by `(cx, cy, cz)` and
/// deduplicated, so two clients with the same inputs get the same
/// set. A zero velocity yields the empty set (no prefetch needed).
///
/// `ticks == 0` also yields the empty set (caller wants no prefetch).
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn prefetch_set(
    anchor: ChunkCoord,
    velocity: VelocityChunksPerTick,
    policy: &WindowPolicy,
    ticks: u32,
) -> Vec<ChunkCoord> {
    if velocity.is_zero() || ticks == 0 || policy.prefetch_ring == 0 {
        return Vec::new();
    }
    // The outer ring radius is `mesh_ring + prefetch_ring` in weighted
    // Chebyshev distance. A future-anchor's inner ring becomes
    // `mesh_ring` away from us when the future-anchor is at the
    // outer-ring boundary.
    let outer_ring = (policy.mesh_ring as u32).saturating_add(policy.prefetch_ring as u32);
    if outer_ring == 0 {
        return Vec::new();
    }
    // Walk future-anchors and accumulate their outer rings.
    let mut set: Vec<ChunkCoord> = Vec::new();
    for (_t, future_anchor) in velocity.into_iter_chunks(anchor, ticks) {
        for coord in RingIter::new(future_anchor, outer_ring, policy.vy_weight) {
            // Skip chunks that are already in the active inner ring of
            // the *current* anchor (they're paged in by classify).
            let current_ring = ring_distance(coord, anchor, policy.vy_weight);
            if current_ring <= policy.mesh_ring as u32 {
                continue;
            }
            set.push(coord);
        }
    }
    // Sort + dedup deterministically.
    set.sort_unstable_by_key(|c| (c.cx, c.cy, c.cz));
    set.dedup();
    set
}

/// Bounded sample for a P99 estimator.
///
/// A streaming P99 over an unbounded number of samples is a classic
/// reservoir-sampling problem; for the scale benchmark we don't need
/// exactness, we need **stability** (the same inputs give the same
/// estimate) and **bounded memory**. We use a fixed-size sample of
/// the most-recent N values, sorted on read. `N = 1024` is enough to
/// smooth out a 1% tail without allocating more than a few KiB.
pub const P99_SAMPLE_CAP: usize = 1024;

/// A running record of scale metrics for the perf HUD / scale
/// benchmark (FR-CIV-SCALE-008).
///
/// `max_resident_chunks` is the all-time max observed since
/// construction. `p99_tick_time_us` is the P99 of the most-recent
/// `P99_SAMPLE_CAP` tick-time samples, in microseconds; it is
/// `None` until at least one sample is recorded.
///
/// The report is `#[derive(Copy)]` where possible (the sample buffer
/// is owned and cannot be Copy, so the report itself is `Clone`).
#[derive(Debug, Clone)]
pub struct ScaleReport {
    /// Largest `resident.len()` observed since construction.
    pub max_resident_chunks: usize,
    /// Total ticks observed (one increment per `record_tick_time`
    /// call). Useful for averaging.
    pub total_ticks: u64,
    /// Rolling sample of the most-recent tick durations, in
    /// microseconds. Bounded to [`P99_SAMPLE_CAP`].
    sample_us: [u32; P99_SAMPLE_CAP],
    /// Number of valid samples in `sample_us` (≤ `P99_SAMPLE_CAP`).
    sample_len: usize,
    /// Index of the next slot to write in `sample_us` (wraps when
    /// full). Independent of `sample_len` once full.
    next_idx: usize,
}

impl Default for ScaleReport {
    fn default() -> Self {
        Self {
            max_resident_chunks: 0,
            total_ticks: 0,
            sample_us: [0u32; P99_SAMPLE_CAP],
            sample_len: 0,
            next_idx: 0,
        }
    }
}

impl ScaleReport {
    /// Construct a fresh, empty report.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the current resident-chunk count. Updates
    /// `max_resident_chunks` if `current_resident` is a new high.
    pub fn record_resident(&mut self, current_resident: usize) {
        if current_resident > self.max_resident_chunks {
            self.max_resident_chunks = current_resident;
        }
    }

    /// Record a tick duration in microseconds. Saturating at `u32::MAX`
    /// is fine — the scale benchmark only cares about the relative
    /// shape, and a single tick > 4.3s is already a regression.
    pub fn record_tick_time_us(&mut self, us: u32) {
        self.sample_us[self.next_idx] = us;
        self.next_idx = (self.next_idx + 1) % P99_SAMPLE_CAP;
        if self.sample_len < P99_SAMPLE_CAP {
            self.sample_len += 1;
        }
        self.total_ticks = self.total_ticks.saturating_add(1);
    }

    /// P99 of the recorded tick-time samples, in microseconds.
    ///
    /// Returns `None` if no sample has been recorded yet. Otherwise
    /// sorts a **copy** of the live sample window and returns the
    /// element at index `ceil(0.99 * len) - 1` (the 99th-percentile
    /// sample). `len < 100` returns the maximum sample — with fewer
    /// than 100 samples the 99th percentile is the tail.
    #[must_use]
    pub fn p99_tick_time_us(&self) -> Option<u32> {
        if self.sample_len == 0 {
            return None;
        }
        let mut buf: Vec<u32> = self.sample_us[..self.sample_len].to_vec();
        buf.sort_unstable();
        // ceil(0.99 * n) - 1, with floor semantics for small n.
        let n = buf.len();
        let idx = if n < 100 {
            n - 1
        } else {
            // (99 * n) / 100 is integer-floor. For n = 100 → 99, n = 101
            // → 99, ..., n = 199 → 197, ..., n = 1000 → 990 — all in
            // range. We want the 99th percentile, which is the value
            // at rank ⌈0.99·n⌉, i.e. index ⌈0.99·n⌉ - 1.
            let rank = (99 * n).div_ceil(100);
            rank.saturating_sub(1).min(n - 1)
        };
        Some(buf[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coord(cx: i32, cy: i32, cz: i32) -> ChunkCoord {
        ChunkCoord { cx, cy, cz }
    }

    // ---- VelocityChunksPerTick ----

    #[test]
    fn velocity_zero_is_zero() {
        assert!(VelocityChunksPerTick::default().is_zero());
        assert!(VelocityChunksPerTick { x: 0, y: 0, z: 0 }.is_zero());
        assert!(!VelocityChunksPerTick { x: 1, y: 0, z: 0 }.is_zero());
        assert!(!VelocityChunksPerTick { x: 0, y: -1, z: 0 }.is_zero());
    }

    #[test]
    fn velocity_iter_yields_one_coord_per_tick() {
        let v = VelocityChunksPerTick { x: 1, y: 0, z: 0 };
        let items: Vec<(u32, ChunkCoord)> = v.into_iter_chunks(coord(0, 0, 0), 3).collect();
        assert_eq!(
            items,
            vec![
                (1, coord(1, 0, 0)),
                (2, coord(2, 0, 0)),
                (3, coord(3, 0, 0)),
            ]
        );
    }

    #[test]
    fn velocity_iter_zero_ticks_yields_nothing() {
        let v = VelocityChunksPerTick { x: 1, y: 0, z: 0 };
        let items: Vec<(u32, ChunkCoord)> = v.into_iter_chunks(coord(0, 0, 0), 0).collect();
        assert!(items.is_empty());
    }

    #[test]
    fn velocity_iter_saturates_on_overflow() {
        // saturating_add protects against panics on extreme coords.
        let v = VelocityChunksPerTick { x: i32::MAX, y: 0, z: 0 };
        let items: Vec<(u32, ChunkCoord)> = v.into_iter_chunks(coord(0, 0, 0), 2).collect();
        assert_eq!(items[0].1.cx, i32::MAX);
        assert_eq!(items[1].1.cx, i32::MAX); // saturates
    }

    // ---- prefetch_set ----

    /// FR-CIV-SCALE-005 — a camera flying in +X with `prefetch_ring = 2`
    /// warms the chunk band 2 rings ahead, sorted + deduplicated, in
    /// the chunk coordinate frame the policy uses.
    #[test]
    fn fr_civ_scale_005_velocity_driven_prefetch_warms_future_rings() {
        let policy = WindowPolicy {
            mesh_ring: 1,
            prefetch_ring: 2,
            ..WindowPolicy::default()
        };
        let velocity = VelocityChunksPerTick { x: 1, y: 0, z: 0 };
        let set = prefetch_set(coord(0, 0, 0), velocity, &policy, DEFAULT_PREFETCH_TICKS);
        // For each of ticks 1..=4 the future-anchor is at (t, 0, 0);
        // we want the outer ring at distance 3 (mesh_ring+prefetch=3)
        // from each future-anchor, but skipping the inner ring of the
        // *current* anchor. The set is non-empty and sorted.
        assert!(!set.is_empty(), "moving camera should warm the future ring");
        // All coords are at weighted ring > 1 from the current anchor.
        for c in &set {
            let r = ring_distance(*c, coord(0, 0, 0), policy.vy_weight);
            assert!(
                r > policy.mesh_ring as u32,
                "prefetch coord {c:?} (ring {r}) should be outside the current inner ring"
            );
        }
        // Sorted lexicographically.
        let mut sorted = set.clone();
        sorted.sort_unstable_by_key(|c| (c.cx, c.cy, c.cz));
        assert_eq!(set, sorted, "prefetch set must be sorted");
        // Deduped.
        let mut dedup = set.clone();
        dedup.dedup();
        assert_eq!(set.len(), dedup.len(), "prefetch set must be deduplicated");
    }

    /// A stationary camera (zero velocity) yields the empty set.
    #[test]
    fn prefetch_set_zero_velocity_is_empty() {
        let policy = WindowPolicy {
            prefetch_ring: 2,
            ..WindowPolicy::default()
        };
        let v = VelocityChunksPerTick::default();
        let set = prefetch_set(coord(0, 0, 0), v, &policy, 4);
        assert!(set.is_empty());
    }

    /// `prefetch_ring = 0` (prefetch disabled) yields the empty set
    /// regardless of velocity — the policy's own switch wins.
    #[test]
    fn prefetch_set_disabled_when_prefetch_ring_zero() {
        let policy = WindowPolicy::default(); // prefetch_ring = 0
        let v = VelocityChunksPerTick { x: 1, y: 0, z: 0 };
        let set = prefetch_set(coord(0, 0, 0), v, &policy, 4);
        assert!(set.is_empty());
    }

    /// `ticks = 0` yields the empty set (caller wants no prefetch).
    #[test]
    fn prefetch_set_zero_ticks_is_empty() {
        let policy = WindowPolicy {
            prefetch_ring: 2,
            ..WindowPolicy::default()
        };
        let v = VelocityChunksPerTick { x: 1, y: 0, z: 0 };
        let set = prefetch_set(coord(0, 0, 0), v, &policy, 0);
        assert!(set.is_empty());
    }

    /// Two calls with the same inputs produce the same set (replay-safe).
    #[test]
    fn prefetch_set_is_deterministic() {
        let policy = WindowPolicy {
            mesh_ring: 1,
            prefetch_ring: 2,
            ..WindowPolicy::default()
        };
        let v = VelocityChunksPerTick { x: 1, y: 0, z: 0 };
        let a = prefetch_set(coord(0, 0, 0), v, &policy, 4);
        let b = prefetch_set(coord(0, 0, 0), v, &policy, 4);
        assert_eq!(a, b);
    }

    /// Far-apart anchors with the same relative offset yield the same
    /// set shape (translation-invariance of the policy, mirrored for
    /// prefetch).
    #[test]
    fn prefetch_set_translation_invariant() {
        let policy = WindowPolicy {
            mesh_ring: 1,
            prefetch_ring: 2,
            ..WindowPolicy::default()
        };
        let v = VelocityChunksPerTick { x: 1, y: 0, z: 0 };
        let a = prefetch_set(coord(0, 0, 0), v, &policy, 4);
        let b = prefetch_set(coord(10_000, 0, -5_000), v, &policy, 4);
        assert_eq!(a.len(), b.len(), "far-apart anchors should yield the same set size");
        // The relative offset between the two sets is exactly the
        // anchor delta — verify by subtracting.
        let delta = coord(10_000, 0, -5_000);
        let translated: Vec<ChunkCoord> = a
            .iter()
            .map(|c| ChunkCoord {
                cx: c.cx + delta.cx,
                cy: c.cy + delta.cy,
                cz: c.cz + delta.cz,
            })
            .collect();
        let mut translated_sorted = translated.clone();
        translated_sorted.sort_unstable_by_key(|c| (c.cx, c.cy, c.cz));
        let mut b_sorted = b.clone();
        b_sorted.sort_unstable_by_key(|c| (c.cx, c.cy, c.cz));
        assert_eq!(translated_sorted, b_sorted);
    }

    // ---- ScaleReport ----

    /// FR-CIV-SCALE-008 — `max_resident_chunks` tracks the high water
    /// mark across many `record_resident` calls.
    #[test]
    fn fr_civ_scale_008_scale_report_max_resident_tracks_high_water() {
        let mut r = ScaleReport::new();
        r.record_resident(5);
        assert_eq!(r.max_resident_chunks, 5);
        r.record_resident(3); // lower — no change
        assert_eq!(r.max_resident_chunks, 5);
        r.record_resident(7);
        assert_eq!(r.max_resident_chunks, 7);
        r.record_resident(7); // equal — no change
        assert_eq!(r.max_resident_chunks, 7);
    }

    /// P99 is `None` until at least one sample is recorded.
    #[test]
    fn scale_report_p99_none_until_sample() {
        let r = ScaleReport::new();
        assert!(r.p99_tick_time_us().is_none());
    }

    /// With a single sample, P99 is that sample.
    #[test]
    fn scale_report_p99_single_sample_is_max() {
        let mut r = ScaleReport::new();
        r.record_tick_time_us(42);
        assert_eq!(r.p99_tick_time_us(), Some(42));
    }

    /// With 100 samples of equal value, P99 is that value.
    #[test]
    fn scale_report_p99_uniform_value() {
        let mut r = ScaleReport::new();
        for _ in 0..100 {
            r.record_tick_time_us(1_000);
        }
        assert_eq!(r.p99_tick_time_us(), Some(1_000));
        assert_eq!(r.total_ticks, 100);
    }

    /// With 100 samples `[1..=100]` µs, the P99 is the 99th-ranked
    /// value (99 µs).
    #[test]
    fn scale_report_p99_picks_99th_percentile() {
        let mut r = ScaleReport::new();
        for us in 1..=100u32 {
            r.record_tick_time_us(us);
        }
        // n = 100 → rank = ceil(99 * 100 / 100) = 99, index = 98 → 99 µs.
        assert_eq!(r.p99_tick_time_us(), Some(99));
    }

    /// The sample buffer wraps at `P99_SAMPLE_CAP` and the P99 stays
    /// stable — we only see the most-recent `P99_SAMPLE_CAP` samples.
    #[test]
    fn scale_report_wraps_at_sample_cap() {
        let mut r = ScaleReport::new();
        // Fill the buffer with 1 µs samples, then overwrite with 10 µs.
        for _ in 0..P99_SAMPLE_CAP {
            r.record_tick_time_us(1);
        }
        for _ in 0..P99_SAMPLE_CAP {
            r.record_tick_time_us(10);
        }
        assert_eq!(r.p99_tick_time_us(), Some(10));
        // Now overwrite half with 1 µs — P99 still 10.
        for _ in 0..(P99_SAMPLE_CAP / 2) {
            r.record_tick_time_us(1);
        }
        assert_eq!(r.p99_tick_time_us(), Some(10));
        // Overwrite all — P99 is 1.
        for _ in 0..P99_SAMPLE_CAP {
            r.record_tick_time_us(1);
        }
        assert_eq!(r.p99_tick_time_us(), Some(1));
    }

    /// Total-ticks counter saturates rather than overflowing.
    #[test]
    fn scale_report_total_ticks_saturates() {
        let mut r = ScaleReport::new();
        r.record_tick_time_us(1);
        r.total_ticks = u64::MAX;
        r.record_tick_time_us(1);
        assert_eq!(r.total_ticks, u64::MAX, "total_ticks should saturate");
    }
}
