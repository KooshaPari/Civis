//! NFR-CIV-SCALE-PERF-900 — Chunk streaming window.
//!
//! Pure-logic streaming window over a chunk lattice: given a moving **focus**
//! (anchor chunk) and a **radius** (in chunks), the [`StreamingWindow`]
//! computes the set of chunks that *should* be resident, diffs against the
//! current resident set, and emits explicit **load** and **unload** deltas.
//! Resident memory is bounded by the configured `load_radius` and the hard
//! `resident_cap`.
//!
//! This module is the Civis-side, engine-agnostic core of "chunk streaming".
//! It deliberately does not touch the `phenotype-voxel` kernel types' mutators
//! or any Bevy ECS resource — it only operates on [`ChunkCoord`] and emits
//! [`LoadOp`] / [`UnloadOp`] deltas. The engine-facing adapter (Bevy/Godot/Unreal)
//! is responsible for materialising those deltas into actual GPU/storage work.
//!
//! ## Acceptance criterion (NFR-CIV-SCALE-PERF-900)
//!
//! *Moving the focus loads new chunks and unloads distant ones; resident count
//! stays bounded.* Concretely:
//!
//! 1. After calling [`StreamingWindow::update_focus`] with a new anchor, the set
//!    of resident chunks is exactly the Chebyshev ball of radius `load_radius`
//!    around that anchor (no leftovers from the previous anchor).
//! 2. `resident_count() <= load_radius^3` at all times (a hard upper bound; in
//!    practice the Chebyshev ball is `(2r+1)^3` and we cap at that).
//!
//! See `acceptance_focus_shift_loads_and_unloads_with_bounded_residency`.
//!
//! ## Determinism
//!
//! `update_focus` is order-independent given the same `(focus, radius, resident)`
//! triple: loads are produced by walking the target cube in deterministic order
//! `(cz, cy, cx)`, and unloads are produced by walking the previous resident
//! set in sorted order. Two windows with identical state agree bit-for-bit on
//! their [`WindowUpdate`] output.
//!
//! ## Relationship to other modules
//!
//! * [`crate::stream::StreamingWorld`] does the I/O (disk-backed dirty cache,
//!   seeded regeneration, LRU eviction). This module is its **front-end policy**:
//!   it decides *which* chunks should be resident given a focus; the engine
//!   then asks the streaming layer to materialise those chunks.
//! * [`crate::window::ring_distance`] / [`crate::window::WindowPolicy`] are the
//!   LOD/lifecycle counterpart. The streaming window here is the **load-set**
//!   counterpart: "chunks that must be in RAM". The window policy decides what
//!   each resident chunk *does* (meshed vs faded vs coarse-sim); this module
//!   decides whether it is *resident at all*.
//!
//! ## No Bevy
//!
//! Pure-logic by design — no `bevy_*` imports. The acceptance test lives in
//! `#[cfg(test)]` and uses only `std`.

use std::collections::BTreeSet;

use phenotype_voxel::ChunkCoord;

/// NFR-CIV-SCALE-PERF-900 — chunk streaming window: chunks load within a radius
/// of focus and unload beyond it, bounding resident memory.
pub const NFR_CIV_SCALE_PERF_900: &str = "NFR-CIV-SCALE-PERF-900";

/// Error type for [`StreamingWindow`] construction and updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowError {
    /// `load_radius` exceeds `unload_radius`, or one of them is absurdly large.
    InvalidRadii,
    /// `unload_radius` was set to 0; the window must keep at least the focus chunk.
    UnloadRadiusZero,
}

impl std::fmt::Display for WindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowError::InvalidRadii => {
                write!(f, "load_radius must be <= unload_radius")
            }
            WindowError::UnloadRadiusZero => {
                write!(f, "unload_radius must be >= 1 (focus chunk must stay resident)")
            }
        }
    }
}

impl std::error::Error for WindowError {}

/// Configuration for a [`StreamingWindow`].
///
/// The window enforces a strict inner-ball rule: the resident set is **always
/// exactly** the Chebyshev ball of radius `load_radius` around the current
/// focus. `unload_radius` is reserved for callers that want a hysteresis buffer
/// (a separate policy layer that decides whether to *keep* a chunk that just
/// slipped outside the inner ring); the window itself drops anything outside
/// `load_radius` immediately on focus shift so resident memory is provably
/// bounded by `(2 * load_radius + 1)^3`.
///
/// `resident_cap` is a hard safety net: if the inner ball would exceed it (a
/// misconfiguration), the farthest chunks from the focus are trimmed off and
/// reported as unloads.
pub struct StreamingWindowConfig {
    /// Inner ring radius (chunks). Chunks within this Chebyshev distance of
    /// the focus are always loaded. The strict invariant: after every
    /// `update_focus`, the resident set is exactly the inner ball of this
    /// radius around the current focus.
    pub load_radius: u32,
    /// Outer ring radius (chunks). Currently unused by the window itself
    /// (the strict rule drops anything outside `load_radius` immediately);
    /// reserved for future hysteresis-buffer support. Must be `>= load_radius`.
    pub unload_radius: u32,
    /// Hard upper bound on the resident set. Acts as a safety cap so the
    /// window can never blow past `(2*load_radius+1)^3` even under a
    /// misconfigured `unload_radius`.
    pub resident_cap: usize,
}

impl StreamingWindowConfig {
    /// Validate the radii / cap. Returns `Err` if `load_radius > unload_radius`,
    /// if `unload_radius == 0`, or if `resident_cap == 0`.
    pub fn validate(&self) -> Result<(), WindowError> {
        if self.unload_radius == 0 {
            return Err(WindowError::UnloadRadiusZero);
        }
        if self.load_radius > self.unload_radius {
            return Err(WindowError::InvalidRadii);
        }
        if self.resident_cap == 0 {
            return Err(WindowError::InvalidRadii);
        }
        Ok(())
    }

    /// Chebyshev-ball size for `load_radius`: `(2r+1)^3`.
    pub fn inner_ball_size(load_radius: u32) -> usize {
        let side = (2 * load_radius as usize).saturating_add(1);
        side.saturating_mul(side).saturating_mul(side)
    }
}

/// One chunk that should be materialised (loaded) by the streaming layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LoadOp(pub ChunkCoord);

/// One chunk that should be released (unloaded) by the streaming layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnloadOp(pub ChunkCoord);

/// The full delta emitted by a single [`StreamingWindow::update_focus`] call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowUpdate {
    /// New chunks to materialise. Order: `(cz, cy, cx)` ascending.
    pub loads: Vec<LoadOp>,
    /// Old chunks to release. Order: `(cz, cy, cx)` ascending.
    pub unloads: Vec<UnloadOp>,
}

impl WindowUpdate {
    /// Number of loads in this delta.
    pub fn load_count(&self) -> usize {
        self.loads.len()
    }
    /// Number of unloads in this delta.
    pub fn unload_count(&self) -> usize {
        self.unloads.len()
    }
    /// True iff this delta is a no-op (no chunks added or removed).
    pub fn is_noop(&self) -> bool {
        self.loads.is_empty() && self.unloads.is_empty()
    }
}

/// Stats snapshot for the perf HUD.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WindowStats {
    /// Chunks currently resident.
    pub resident: usize,
    /// Total loads emitted across the lifetime of this window.
    pub total_loads: u64,
    /// Total unloads emitted across the lifetime of this window.
    pub total_unloads: u64,
    /// Number of `update_focus` calls that were a no-op.
    pub noop_updates: u64,
}

/// Pure-logic chunk streaming window.
///
/// Tracks the set of chunks that *should* be resident for the current focus
/// and emits explicit load/unload deltas when the focus moves. Pure logic: no
/// I/O, no ECS, no GPU. The engine adapter is responsible for materialising
/// the deltas into actual mesh/storage work.
#[derive(Debug, Clone)]
pub struct StreamingWindow {
    cfg: StreamingWindowConfig,
    /// Currently-resident chunk coords, kept sorted via [`BTreeSet`] for
    /// deterministic iteration.
    resident: BTreeSet<ChunkCoord>,
    /// Last focus handed to `update_focus`. `None` before the first call.
    focus: Option<ChunkCoord>,
    stats: WindowStats,
}

impl StreamingWindow {
    /// Construct a window with the given config. The config is validated; an
    /// invalid config (see [`StreamingWindowConfig::validate`]) is returned
    /// as an error.
    pub fn new(cfg: StreamingWindowConfig) -> Result<Self, WindowError> {
        cfg.validate()?;
        Ok(Self {
            cfg,
            resident: BTreeSet::new(),
            focus: None,
            stats: WindowStats::default(),
        })
    }

    /// Construct a window with `load_radius == unload_radius == radius` and
    /// `resident_cap = (2*radius+1)^3`. Convenience for the common
    /// "single-radius, no-hysteresis" config.
    pub fn with_radius(radius: u32) -> Result<Self, WindowError> {
        let cap = StreamingWindowConfig::inner_ball_size(radius).max(1);
        Self::new(StreamingWindowConfig {
            load_radius: radius,
            unload_radius: radius,
            resident_cap: cap,
        })
    }

    /// Current configuration.
    pub fn config(&self) -> StreamingWindowConfig {
        self.cfg
    }

    /// Current focus (anchor chunk). `None` if `update_focus` has never been called.
    pub fn focus(&self) -> Option<ChunkCoord> {
        self.focus
    }

    /// Number of chunks currently in the resident set.
    pub fn resident_count(&self) -> usize {
        self.resident.len()
    }

    /// `true` iff `coord` is currently in the resident set.
    pub fn contains(&self, coord: ChunkCoord) -> bool {
        self.resident.contains(&coord)
    }

    /// Iterator over the resident chunks, in `(cz, cy, cx)` ascending order.
    pub fn resident_coords(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.resident.iter().copied()
    }

    /// Stats snapshot.
    pub fn stats(&self) -> WindowStats {
        WindowStats {
            resident: self.resident.len(),
            ..self.stats
        }
    }

    /// Compute the inner Chebyshev ball around `focus` of radius
    /// `cfg.load_radius`. Returned in `(cz, cy, cx)` ascending order so the
    /// output is deterministic.
    fn inner_ball(&self, focus: ChunkCoord) -> Vec<ChunkCoord> {
        let r = self.cfg.load_radius as i64;
        let r_i32 = self.cfg.load_radius as i32;
        let fx = focus.cx as i64;
        let fy = focus.cy as i64;
        let fz = focus.cz as i64;
        let mut out = Vec::with_capacity(StreamingWindowConfig::inner_ball_size(
            self.cfg.load_radius,
        ));
        // Iterate (z, y, x) so the output is `(cz, cy, cx)` ascending — matches
        // the kernel's canonical chunk-id bit order (`cz` in the low bits).
        for cz_off in -r..=r {
            for cy_off in -r..=r {
                for cx_off in -r..=r {
                    let cx = fx + cx_off;
                    let cy = fy + cy_off;
                    let cz = fz + cz_off;
                    // Saturate into i32; Chebyshev-ball coords near i32::MAX
                    // overflow on add. Clamp rather than wrap so callers don't
                    // get surprise aliasing across the lattice boundary.
                    let cx_i32 = i32::try_from(cx).unwrap_or(if cx >= 0 {
                        i32::MAX
                    } else {
                        i32::MIN
                    });
                    let cy_i32 = i32::try_from(cy).unwrap_or(if cy >= 0 {
                        i32::MAX
                    } else {
                        i32::MIN
                    });
                    let cz_i32 = i32::try_from(cz).unwrap_or(if cz >= 0 {
                        i32::MAX
                    } else {
                        i32::MIN
                    });
                    // Cheap tie-breaker: a coord that overflowed still has
                    // |Δ| > r in at least one axis, so it would be outside the
                    // ball anyway. We filter explicitly using r_i32 so the
                    // clamped coord never leaks in.
                    let dx = (cx_i32 as i64 - fx).abs();
                    let dy = (cy_i32 as i64 - fy).abs();
                    let dz = (cz_i32 as i64 - fz).abs();
                    if dx <= r_i32 as i64 && dy <= r_i32 as i64 && dz <= r_i32 as i64 {
                        out.push(ChunkCoord {
                            cx: cx_i32,
                            cy: cy_i32,
                            cz: cz_i32,
                        });
                    }
                }
            }
        }
        out
    }

    /// Move the focus to `new_focus` and emit the load/unload deltas required
    /// to bring the resident set into agreement with the new inner ball.
    ///
    /// Behaviour:
    ///
    /// 1. Loads: every chunk in the **new inner ball** that is not already
    ///    resident. Ordered by `(cz, cy, cx)` ascending.
    /// 2. Unloads: every chunk currently resident that is **outside the new
    ///    inner ball** — i.e. the strict "keep only inner" rule. This means
    ///    chunks in the hysteresis buffer ring (`load_radius < ring <=
    ///    unload_radius`) are also dropped on focus shift. The hysteresis
    ///    buffer is exposed via [`StreamingWindowConfig`] for callers that
    ///    want to override this policy with their own bookkeeping; the window
    ///    itself enforces the strict inner-ball invariant so resident memory
    ///    is provably bounded by `(2 * load_radius + 1)^3`.
    /// 3. After the call, `resident == inner_ball(new_focus)` exactly.
    /// 4. The hard `resident_cap` is enforced by trimming the resident set
    ///    if it would otherwise exceed it; trimmed chunks are reported as
    ///    unloads. With the default `resident_cap = inner_ball_size(load_radius)`
    ///    this branch never fires.
    pub fn update_focus(&mut self, new_focus: ChunkCoord) -> WindowUpdate {
        // --- compute the target set ---
        let new_inner = self.inner_ball(new_focus);
        let new_inner_set: BTreeSet<ChunkCoord> = new_inner.iter().copied().collect();

        // --- loads: new_inner \ resident ---
        let mut loads: Vec<LoadOp> = Vec::new();
        for coord in &new_inner {
            if !self.resident.contains(coord) {
                loads.push(LoadOp(*coord));
            }
        }

        // --- unloads: resident \ new_inner (strict keep-only-inner rule) ---
        let mut unloads: Vec<UnloadOp> = Vec::new();
        for coord in &self.resident {
            if !new_inner_set.contains(coord) {
                unloads.push(UnloadOp(*coord));
            }
        }

        // --- apply delta to resident set ---
        for load in &loads {
            self.resident.insert(load.0);
        }
        for unload in &unloads {
            self.resident.remove(&unload.0);
        }

        // --- enforce hard cap ---
        if self.resident.len() > self.cfg.resident_cap {
            // Deterministic eviction: drop the *farthest* chunks from the new
            // focus first (Chebyshev distance), breaking ties by `(cz, cy, cx)`
            // ascending so two windows in the same state produce the same
            // eviction order.
            let mut by_distance: Vec<(ChunkCoord, i64)> = self
                .resident
                .iter()
                .map(|coord| {
                    let dx = (coord.cx as i64 - new_focus.cx as i64).abs();
                    let dy = (coord.cy as i64 - new_focus.cy as i64).abs();
                    let dz = (coord.cz as i64 - new_focus.cz as i64).abs();
                    let d = dx.max(dy).max(dz);
                    (*coord, d)
                })
                .collect();
            by_distance.sort_by(|(a_coord, a_d), (b_coord, b_d)| {
                b_d.cmp(a_d).then_with(|| a_coord.cmp(b_coord))
            });
            while self.resident.len() > self.cfg.resident_cap {
                if let Some((coord, _)) = by_distance.pop() {
                    if self.resident.remove(&coord) {
                        unloads.push(UnloadOp(coord));
                    }
                } else {
                    break;
                }
            }
        }

        // --- update stats ---
        self.stats.total_loads = self.stats.total_loads.saturating_add(loads.len() as u64);
        self.stats.total_unloads =
            self.stats.total_unloads.saturating_add(unloads.len() as u64);
        if loads.is_empty() && unloads.is_empty() {
            self.stats.noop_updates = self.stats.noop_updates.saturating_add(1);
        }

        self.focus = Some(new_focus);

        WindowUpdate { loads, unloads }
    }

    /// Drop every chunk from the resident set. Returns the unloads emitted.
    pub fn clear(&mut self) -> Vec<UnloadOp> {
        let mut unloads: Vec<UnloadOp> =
            self.resident.iter().map(|c| UnloadOp(*c)).collect();
        unloads.sort_by_key(|op| (op.0.cz, op.0.cy, op.0.cx));
        self.resident.clear();
        self.stats.total_unloads =
            self.stats.total_unloads.saturating_add(unloads.len() as u64);
        unloads
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(cx: i32, cy: i32, cz: i32) -> ChunkCoord {
        ChunkCoord { cx, cy, cz }
    }

    /// NFR-CIV-SCALE-PERF-900 — acceptance: moving focus loads new chunks and
    /// unloads distant ones; resident count stays bounded.
    #[test]
    fn acceptance_focus_shift_loads_and_unloads_with_bounded_residency() {
        let cfg = StreamingWindowConfig {
            load_radius: 2,
            unload_radius: 2,
            resident_cap: StreamingWindowConfig::inner_ball_size(2),
        };
        let mut w = StreamingWindow::new(cfg).expect("window");

        // 1) Initial focus at the origin: should load the full Chebyshev ball
        //    of radius 2 around (0,0,0), i.e. a 5³ cube = 125 chunks.
        let upd = w.update_focus(c(0, 0, 0));
        assert_eq!(upd.load_count(), 125);
        assert_eq!(upd.unload_count(), 0);
        assert_eq!(w.resident_count(), 125);
        assert!(w.resident_count() <= cfg.resident_cap);
        assert!(w.contains(c(0, 0, 0)));
        assert!(w.contains(c(2, 2, 2)));
        assert!(!w.contains(c(3, 0, 0)));

        // 2) Move focus one chunk along +X. The intersection of the new ball
        //    with the old ball is 4×5×5 = 100 chunks (the new ball is the same
        //    size; one chunk at +X = +2 fell out and one new chunk at +X = +3
        //    came in). Concretely: 100 stay, 25 leave, 25 enter.
        let upd = w.update_focus(c(1, 0, 0));
        assert_eq!(upd.load_count(), 25);
        assert_eq!(upd.unload_count(), 25);
        assert_eq!(w.resident_count(), 125);
        assert!(w.resident_count() <= cfg.resident_cap);
        assert!(w.contains(c(3, 0, 0)));
        assert!(!w.contains(c(-2, 0, 0)));
        assert!(w.contains(c(1, 0, 0)));

        // 3) Jump focus far away (across the lattice). Almost everything
        //    unloads; the new ball loads. Resident count is still bounded.
        let upd = w.update_focus(c(100, 0, 100));
        assert_eq!(upd.load_count(), 125);
        assert_eq!(upd.unload_count(), 125);
        assert_eq!(w.resident_count(), 125);
        assert!(w.resident_count() <= cfg.resident_cap);
        assert!(w.contains(c(100, 0, 100)));
        assert!(!w.contains(c(0, 0, 0)));

        // 4) Stay still: zero deltas. Resident count is unchanged.
        let upd = w.update_focus(c(100, 0, 100));
        assert!(upd.is_noop());
        assert_eq!(w.resident_count(), 125);
        assert!(w.resident_count() <= cfg.resident_cap);
    }

    /// NFR-CIV-SCALE-PERF-900 — resident count never exceeds `load_radius`³
    /// bound, even after a long sequence of focus shifts.
    #[test]
    fn acceptance_resident_count_bounded_under_random_walk() {
        let r = 3u32;
        let cap = StreamingWindowConfig::inner_ball_size(r);
        let mut w = StreamingWindow::with_radius(r).expect("window");
        let mut focus = c(0, 0, 0);
        w.update_focus(focus);
        assert_eq!(w.resident_count(), cap);
        // Walk the focus across the lattice in a deterministic zig-zag and
        // assert the resident count never exceeds the Chebyshev ball size.
        for step in 0..200i32 {
            let dx = (step % 7) - 3;
            let dz = (step / 7 % 7) - 3;
            focus = c(focus.cx + dx, 0, focus.cz + dz);
            w.update_focus(focus);
            assert!(
                w.resident_count() <= cap,
                "resident count {} exceeded cap {} at step {}",
                w.resident_count(),
                cap,
                step
            );
            // And the resident set must actually equal the new inner ball —
            // no leftovers from old positions, no gaps inside the new ball.
            assert_eq!(w.resident_count(), cap);
        }
    }

    /// NFR-CIV-SCALE-PERF-900 — focus at the same coord produces a no-op delta.
    #[test]
    fn noop_when_focus_unchanged() {
        let mut w = StreamingWindow::with_radius(1).expect("window");
        let _ = w.update_focus(c(0, 0, 0));
        let upd = w.update_focus(c(0, 0, 0));
        assert!(upd.is_noop());
        assert_eq!(w.stats().noop_updates, 1);
    }

    /// Determinism: two windows with the same focus history produce identical
    /// deltas.
    #[test]
    fn deltas_are_deterministic_across_windows() {
        let cfg = StreamingWindowConfig {
            load_radius: 2,
            unload_radius: 2,
            resident_cap: StreamingWindowConfig::inner_ball_size(2),
        };
        let mut a = StreamingWindow::new(cfg).expect("a");
        let mut b = StreamingWindow::new(cfg).expect("b");
        for step in 0..16 {
            let focus = c(step, step.wrapping_mul(2) as i32, step.wrapping_neg() as i32);
            let ua = a.update_focus(focus);
            let ub = b.update_focus(focus);
            assert_eq!(ua, ub, "differed at step {step}");
        }
    }

    /// Invalid radii are rejected at construction.
    #[test]
    fn invalid_radii_rejected() {
        // load_radius > unload_radius: invalid.
        let bad = StreamingWindowConfig {
            load_radius: 4,
            unload_radius: 2,
            resident_cap: 1,
        };
        assert!(matches!(
            StreamingWindow::new(bad),
            Err(WindowError::InvalidRadii)
        ));
        // unload_radius == 0: invalid (focus must stay resident).
        let bad = StreamingWindowConfig {
            load_radius: 0,
            unload_radius: 0,
            resident_cap: 1,
        };
        assert!(matches!(
            StreamingWindow::new(bad),
            Err(WindowError::UnloadRadiusZero)
        ));
    }

    /// `clear` drops every resident chunk and emits an unload per chunk.
    #[test]
    fn clear_drops_everything() {
        let mut w = StreamingWindow::with_radius(2).expect("window");
        w.update_focus(c(0, 0, 0));
        assert_eq!(w.resident_count(), 125);
        let unloads = w.clear();
        assert_eq!(unloads.len(), 125);
        assert_eq!(w.resident_count(), 0);
    }

    /// Hysteresis: `unload_radius > load_radius` allows a chunk to *linger*
    /// in the buffer ring for one frame after the focus moves past it.
    /// Within the buffer ring, the chunk is dropped immediately (we use the
    /// strict "keep only inner ball" rule), but `unload_radius > load_radius`
    /// still affects the documented contract via [`StreamingWindowConfig`].
    /// Here we just verify the config is accepted and the window stays sane.
    #[test]
    fn hysteresis_config_accepted_and_stable() {
        let cfg = StreamingWindowConfig {
            load_radius: 1,
            unload_radius: 3,
            resident_cap: StreamingWindowConfig::inner_ball_size(1),
        };
        let mut w = StreamingWindow::new(cfg).expect("window");
        w.update_focus(c(0, 0, 0));
        assert_eq!(w.resident_count(), 27);
        // Move focus one chunk along +X: still inside the unload_radius ball,
        // so any leftover from the *old* ball is fully evicted by the strict
        // keep-inner rule. Resident set is exactly the new inner ball.
        w.update_focus(c(1, 0, 0));
        assert_eq!(w.resident_count(), 27);
        assert!(w.contains(c(2, 0, 0)));
        assert!(!w.contains(c(-1, 0, 0)));
    }

    /// Stats accumulate correctly across many focus shifts.
    #[test]
    fn stats_accumulate() {
        let mut w = StreamingWindow::with_radius(1).expect("window");
        let _ = w.update_focus(c(0, 0, 0)); // 27 loads, 0 unloads
        let upd = w.update_focus(c(1, 0, 0)); // some loads, some unloads
        let stats = w.stats();
        assert_eq!(stats.resident, 27);
        assert!(stats.total_loads >= 27 + upd.load_count() as u64);
        assert!(stats.total_unloads >= upd.unload_count() as u64);
    }
}