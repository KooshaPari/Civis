//! FR-UX-003 — a timeline scrubber SHALL display tick history and allow
//! rewind to any stored tick.
//!
//! Matrix check: `render::timeline_scrubber_rewind`.

use civ_render::timeline::Timeline;

/// Recorded ticks are listed, any retained tick can be rewound to, and
/// evicted ticks are reported as unavailable.
#[test]
fn timeline_scrubber_rewind() {
    let mut tl = Timeline::new(4);
    for tick in 100..106 {
        tl.record(tick, vec![u8::try_from(tick).unwrap()]);
    }

    // History displays only the retained window (FIFO eviction).
    assert_eq!(tl.available_ticks(), vec![102, 103, 104, 105]);
    assert_eq!(tl.oldest_tick(), Some(102));
    assert_eq!(tl.newest_tick(), Some(105));
    assert_eq!(tl.cursor(), 105);

    // Rewind to any stored tick.
    let payload = tl.seek(103).expect("tick 103 is retained");
    assert_eq!(payload, &[103u8]);
    assert_eq!(tl.cursor(), 103);

    // Scrub thumb tracks position across the window.
    assert!((tl.scrub_fraction() - 1.0 / 3.0).abs() < 1e-5);

    // A rewound-to tick must not advance the window.
    assert_eq!(tl.available_ticks(), vec![102, 103, 104, 105]);

    // Evicted tick is unavailable and leaves the cursor untouched.
    assert!(tl.seek(100).is_none());
    assert_eq!(tl.cursor(), 103);
}
