//! NFR-CIV-SCALE-901 — the active working set is resident; the rest streams
//! from disk.
//!
//! Matrix check: `streaming_memory_budget`.
//! Acceptance contract: resident chunk count ≤ configured budget during a
//! full-map pan, and evicted chunks reload when the camera returns.
//!
//! `StreamingWindow` enforces the budget structurally: after every
//! `update_focus` the resident set is exactly the inner Chebyshev ball of
//! `load_radius`, so residency is bounded by `(2r+1)^3` no matter how far the
//! camera travels.

use civ_voxel::{ChunkCoord, StreamingWindow, StreamingWindowConfig};

const fn coord(cx: i32, cy: i32, cz: i32) -> ChunkCoord {
    ChunkCoord { cx, cy, cz }
}

fn window(radius: u32) -> StreamingWindow {
    StreamingWindow::with_radius(radius).expect("valid radius")
}

/// Panning across the map keeps residency within the configured bound.
#[test]
fn streaming_memory_budget() {
    const RADIUS: u32 = 2;
    let bound = StreamingWindowConfig::inner_ball_size(RADIUS);

    let mut w = window(RADIUS);
    assert_eq!(
        w.resident_count(),
        0,
        "a fresh window holds nothing until the focus is set"
    );

    let mut max_resident = 0usize;
    for step in 0..200i32 {
        let update = w.update_focus(coord(step * 3, 0, 0));
        let resident = w.resident_count();
        max_resident = max_resident.max(resident);

        assert!(
            resident <= bound,
            "resident {resident} exceeded the (2r+1)^3 bound {bound} at step {step}"
        );
        // Load/unload bookkeeping stays observable at every step.
        let _ = (update.load_count(), update.unload_count());
    }

    // The window is not vacuous: it really did hold the inner ball at peak.
    assert_eq!(
        max_resident, bound,
        "the resident window should reach exactly (2r+1)^3"
    );
    assert!(
        w.config().resident_cap >= bound,
        "configured cap must accommodate the inner ball"
    );
}

/// Returning to a previously visited focus restores the same resident set.
#[test]
fn evicted_chunks_reload_on_return() {
    let mut w = window(1);

    w.update_focus(coord(0, 0, 0));
    let at_origin: Vec<ChunkCoord> = w.resident_coords().collect();
    assert!(!at_origin.is_empty());
    assert!(w.contains(coord(0, 0, 0)), "the focus chunk is resident");

    // Travel far enough that the original chunks are evicted.
    w.update_focus(coord(500, 500, 500));
    assert!(
        !w.contains(coord(0, 0, 0)),
        "the origin chunk must be evicted after travelling away"
    );

    // Returning reloads exactly the same set.
    w.update_focus(coord(0, 0, 0));
    let back: Vec<ChunkCoord> = w.resident_coords().collect();
    assert_eq!(
        back, at_origin,
        "returning to a focus restores the identical resident set"
    );
}

/// A full-map pan never leaves the budget, and stale chunks are dropped.
#[test]
fn stale_chunks_are_dropped_at_every_step() {
    let mut w = window(1);
    let bound = StreamingWindowConfig::inner_ball_size(1);

    for step in 0..50i32 {
        w.update_focus(coord(step * 10, step * 10, 0));
        assert_eq!(
            w.resident_count(),
            bound,
            "resident set equals the inner ball at step {step}"
        );
        // Nothing from two steps ago can still be resident here.
        let stale = coord((step - 2) * 10, (step - 2) * 10, 0);
        if step >= 2 {
            assert!(
                !w.contains(stale),
                "a chunk two focuses back must not linger"
            );
        }
    }
}

/// Degenerate configurations are rejected rather than silently unbounded.
#[test]
fn budget_config_is_validated() {
    let zero_cap = StreamingWindowConfig {
        load_radius: 1,
        unload_radius: 2,
        resident_cap: 0,
    };
    assert!(zero_cap.validate().is_err(), "resident_cap = 0 must be rejected");

    let inverted = StreamingWindowConfig {
        load_radius: 5,
        unload_radius: 2,
        resident_cap: 64,
    };
    assert!(inverted.validate().is_err(), "inverted radii must be rejected");

    let good = StreamingWindowConfig {
        load_radius: 2,
        unload_radius: 3,
        resident_cap: 1024,
    };
    assert!(good.validate().is_ok());
    assert_eq!(StreamingWindowConfig::inner_ball_size(0), 1, "r=0 is the focus chunk");
    assert_eq!(StreamingWindowConfig::inner_ball_size(1), 27);
    assert_eq!(StreamingWindowConfig::inner_ball_size(2), 125);
}
