//! Timeline scrubber (FR-UX-003, CIV-0300).
//!
//! A timeline scrubber SHALL display tick history and allow rewind to any
//! stored tick. This module keeps a bounded ring of tick snapshots plus the
//! scrub position, and exposes the seek operations the client needs.
//!
//! Snapshots are opaque byte payloads: the render layer never interprets world
//! state, it only stores and returns it, so the scrubber stays decoupled from
//! the engine's state encoding.

use serde::{Deserialize, Serialize};

/// Entry in the tick history ring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickSnapshot {
    /// Absolute tick number this snapshot was captured at.
    pub tick: u64,
    /// Opaque serialised world state for this tick.
    pub payload: Vec<u8>,
}

/// Timeline scrubber state (FR-UX-003).
///
/// Holds a ring of the most recent `capacity` snapshots. [`seek`](Self::seek)
/// rewinds playback to any retained tick; older snapshots are evicted FIFO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Timeline {
    snapshots: Vec<TickSnapshot>,
    capacity: usize,
    /// The tick currently being displayed.
    cursor: u64,
}

impl Timeline {
    /// Create a timeline retaining up to `capacity` snapshots.
    ///
    /// A capacity of 0 is raised to 1 so at least one snapshot is always
    /// retained.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            snapshots: Vec::new(),
            capacity: capacity.max(1),
            cursor: 0,
        }
    }

    /// Number of retained snapshots.
    #[must_use]
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Is the history empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// The tick currently displayed.
    #[must_use]
    pub fn cursor(&self) -> u64 {
        self.cursor
    }

    /// Oldest retained tick, if any.
    #[must_use]
    pub fn oldest_tick(&self) -> Option<u64> {
        self.snapshots.first().map(|s| s.tick)
    }

    /// Newest retained tick, if any.
    #[must_use]
    pub fn newest_tick(&self) -> Option<u64> {
        self.snapshots.last().map(|s| s.tick)
    }

    /// Record a new tick snapshot, evicting the oldest when full.
    ///
    /// Recording advances the cursor to the recorded tick.
    pub fn record(&mut self, tick: u64, payload: Vec<u8>) {
        if self.snapshots.len() == self.capacity {
            self.snapshots.remove(0);
        }
        self.snapshots.push(TickSnapshot { tick, payload });
        self.cursor = tick;
    }

    /// Rewind (or advance) to a stored tick, returning its payload.
    ///
    /// Returns `None` when the tick is not retained (already evicted or never
    /// recorded); the cursor is left unchanged in that case.
    pub fn seek(&mut self, tick: u64) -> Option<&[u8]> {
        if let Some(idx) = self.snapshots.iter().position(|s| s.tick == tick) {
            self.cursor = tick;
            Some(&self.snapshots[idx].payload)
        } else {
            None
        }
    }

    /// Ticks currently available to seek to, oldest first.
    #[must_use]
    pub fn available_ticks(&self) -> Vec<u64> {
        self.snapshots.iter().map(|s| s.tick).collect()
    }

    /// Fraction in `[0, 1]` of the way the cursor sits across the retained
    /// window, for positioning the scrubber thumb.
    #[must_use]
    pub fn scrub_fraction(&self) -> f32 {
        match (self.oldest_tick(), self.newest_tick()) {
            (Some(lo), Some(hi)) if hi > lo => {
                (self.cursor.saturating_sub(lo)) as f32 / (hi - lo) as f32
            }
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eviction_is_fifo() {
        let mut tl = Timeline::new(3);
        for t in 1..=5 {
            tl.record(t, vec![t as u8]);
        }
        assert_eq!(tl.available_ticks(), vec![3, 4, 5]);
        assert_eq!(tl.len(), 3);
    }

    #[test]
    fn seek_rewinds_to_stored_tick() {
        let mut tl = Timeline::new(8);
        for t in 0..5 {
            tl.record(t, vec![t as u8]);
        }
        assert_eq!(tl.cursor(), 4);
        let payload = tl.seek(2).expect("tick 2 retained");
        assert_eq!(payload, &[2u8]);
        assert_eq!(tl.cursor(), 2);
    }

    #[test]
    fn seek_evicted_tick_returns_none_and_preserves_cursor() {
        let mut tl = Timeline::new(2);
        tl.record(0, vec![0]);
        tl.record(1, vec![1]);
        tl.record(2, vec![2]);
        assert_eq!(tl.cursor(), 2);
        assert!(tl.seek(0).is_none());
        assert_eq!(tl.cursor(), 2);
    }

    #[test]
    fn scrub_fraction_spans_window() {
        let mut tl = Timeline::new(5);
        for t in 10..15 {
            tl.record(t, vec![]);
        }
        tl.seek(10).unwrap();
        assert!((tl.scrub_fraction() - 0.0).abs() < f32::EPSILON);
        tl.seek(14).unwrap();
        assert!((tl.scrub_fraction() - 1.0).abs() < f32::EPSILON);
    }
}
