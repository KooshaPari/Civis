//! Notification queue — FR-CIV-NOTIFY-900.
//!
//! Records significant simulation events with a severity level. The queue is a
//! capacity-bounded ring: pushing past capacity drops the oldest notification,
//! preserving the most recent `capacity` entries. Notifications can be
//! dismissed by id and consumed/iterated with a severity filter.
//!
//! This is pure-logic (no Bevy rendering). All four clients (web, Bevy, Godot,
//! Unreal) can read the queue and project it onto whatever surface they like.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Severity of a notification. Higher variants are more urgent.
///
/// Ordering is meaningful: `Info < Warning < Critical < Fatal`. The
/// `severity_at_least` filter and `Severity::iter` rely on this order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Informational — neutral events worth surfacing.
    Info,
    /// Warning — something the player should pay attention to.
    Warning,
    /// Critical — requires action soon.
    Critical,
    /// Fatal — the simulation has halted or the action has irrecoverable
    /// consequences for the current tick.
    Fatal,
}

impl Severity {
    /// All severities, lowest to highest. Used by `NotificationQueue` for
    /// exhaustive filters and convenience iteration.
    pub const ALL: [Severity; 4] = [
        Severity::Info,
        Severity::Warning,
        Severity::Critical,
        Severity::Fatal,
    ];

    /// Returns true if `self` is at least as severe as `threshold`.
    /// `Info >= Info` is true; `Warning >= Info` is true; `Info >= Warning`
    /// is false.
    #[must_use]
    pub const fn at_least(self, threshold: Severity) -> bool {
        (self as u8) >= (threshold as u8)
    }
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Info
    }
}

/// A single notification entry.
///
/// `id` is monotonically increasing across the lifetime of the
/// `NotificationQueue` — used as a stable handle for `dismiss`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    /// Stable id assigned at push time. Never reused, even after dismiss.
    pub id: u64,
    /// Severity bucket.
    pub severity: Severity,
    /// Short, human-readable label (e.g. `"colony.founded"` or a localized
    /// string). Kept as `String` so callers can pass arbitrary messages.
    pub message: String,
    /// Simulation tick at which the event occurred. Optional because some
    /// callers may not have a tick handle available.
    pub tick: Option<u64>,
}

impl Notification {
    /// Construct a notification without an id — `push` will assign one.
    #[must_use]
    pub fn new(severity: Severity, message: impl Into<String>, tick: Option<u64>) -> Self {
        Self {
            id: 0,
            severity,
            message: message.into(),
            tick,
        }
    }
}

/// Capacity-bounded ring buffer of notifications.
///
/// Pushing past `capacity` drops the oldest entry; the queue never grows
/// beyond `capacity`. Ordering is FIFO by insertion (oldest first,
/// newest last). Dismiss is by id and is O(n) — fine for the sizes the HUD
/// typically cares about (single-digit to low-double-digit capacity).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationQueue {
    /// Maximum number of notifications retained. `0` is permitted and
    /// behaves as an inert sink (every push drops itself).
    capacity: usize,
    /// Ring storage. Invariant: `buf.len() <= capacity`. When `buf.len()
    /// == capacity`, `head` points at the oldest entry's slot.
    buf: Vec<Notification>,
    /// Index of the oldest entry when `buf.len() == capacity`. Otherwise
    /// unused (the buffer is treated as a plain vec when not full).
    head: usize,
    /// Monotonic id source for `push`.
    next_id: u64,
}

impl NotificationQueue {
    /// Construct a queue with a fixed capacity. Capacity `0` is allowed and
    /// produces an inert sink — useful for tests that want to assert the
    /// overflow path always drops.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            buf: Vec::with_capacity(capacity),
            head: 0,
            next_id: 1,
        }
    }

    /// Configured capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Number of notifications currently stored. Always `<= capacity`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// True when no notifications are stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// True when the queue is at capacity (next push will evict the oldest).
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.capacity > 0 && self.buf.len() == self.capacity
    }

    /// Push a notification. Assigns a fresh id. If the queue is at
    /// capacity, the oldest notification is dropped — the *new* one is
    /// retained. Returns the assigned id.
    ///
    /// When `capacity == 0`, the input is discarded and the id still
    /// advances (ids are monotonic and never reused).
    pub fn push(&mut self, mut note: Notification) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        note.id = id;
        if self.capacity == 0 {
            return id;
        }
        if self.buf.len() < self.capacity {
            // Buffer not full — append directly.
            self.buf.push(note);
            return id;
        }
        // Buffer is full. Overwrite the slot at `head` (oldest), then
        // advance `head` to the next-oldest slot.
        let slot = self.head % self.capacity;
        self.buf[slot] = note;
        self.head = (self.head + 1) % self.capacity;
        id
    }

    /// Dismiss (remove) a notification by id. Returns true if a
    /// notification was removed.
    ///
    /// When the buffer is full and a middle element is removed, the ring
    /// is compacted by sliding later entries forward and rewinding `head`
    /// so the invariant `head == 0` when `buf.len() < capacity` holds.
    /// This keeps `iter()` simple (no wrap-around arithmetic on a
    /// non-full buffer).
    pub fn dismiss(&mut self, id: u64) -> bool {
        if let Some(idx) = self.position(id) {
            self.buf.remove(idx);
            // After a remove, `buf.len() < capacity`, so `head` must
            // reset to 0 to keep the invariant clean.
            self.head = 0;
            true
        } else {
            false
        }
    }

    /// Remove every notification with severity at least `threshold`.
    /// Returns the number of notifications removed.
    pub fn dismiss_severity_at_least(&mut self, threshold: Severity) -> usize {
        let before = self.buf.len();
        self.buf.retain(|n| !n.severity.at_least(threshold));
        let removed = before - self.buf.len();
        if removed > 0 {
            self.head = 0;
        }
        removed
    }

    /// Clear every notification. Ids continue to advance on subsequent
    /// pushes (ids are never reused).
    pub fn clear(&mut self) {
        self.buf.clear();
        self.head = 0;
    }

    /// Snapshot the queue in oldest-first order.
    #[must_use]
    pub fn snapshot(&self) -> Vec<Notification> {
        if self.buf.len() < self.capacity {
            // Not full: order is already oldest-first.
            self.buf.clone()
        } else {
            // Full: rotate so `head` becomes index 0.
            let mut out = Vec::with_capacity(self.buf.len());
            let cap = self.capacity;
            for i in 0..cap {
                out.push(self.buf[(self.head + i) % cap].clone());
            }
            out
        }
    }

    /// Iterate notifications in oldest-first order.
    pub fn iter(&self) -> impl Iterator<Item = &Notification> {
        self.snapshot().into_iter()
    }

    /// Iterate notifications at or above `threshold`, oldest-first.
    pub fn iter_severity_at_least<'a>(
        &'a self,
        threshold: Severity,
    ) -> impl Iterator<Item = &'a Notification> + 'a {
        // Materialise once so the returned iterator doesn't borrow `self`
        // mutably for the filter; cheaper than a custom streaming
        // iterator and the queue is small.
        let mut owned: Vec<&'a Notification> = Vec::new();
        if self.buf.len() < self.capacity {
            for n in &self.buf {
                if n.severity.at_least(threshold) {
                    owned.push(n);
                }
            }
        } else {
            let cap = self.capacity;
            for i in 0..cap {
                let n = &self.buf[(self.head + i) % cap];
                if n.severity.at_least(threshold) {
                    owned.push(n);
                }
            }
        }
        owned.into_iter()
    }

    /// Look up a notification by id without removing it.
    #[must_use]
    pub fn get(&self, id: u64) -> Option<&Notification> {
        self.position(id).map(|i| &self.buf[i])
    }

    // Internal: locate the slot index of `id` accounting for the ring
    // head offset when the buffer is full.
    fn position(&self, id: u64) -> Option<usize> {
        if self.buf.is_empty() {
            return None;
        }
        if self.buf.len() < self.capacity {
            self.buf.iter().position(|n| n.id == id)
        } else {
            let cap = self.capacity;
            for i in 0..cap {
                let slot = (self.head + i) % cap;
                if self.buf[slot].id == id {
                    return Some(slot);
                }
            }
            None
        }
    }
}

impl Default for NotificationQueue {
    fn default() -> Self {
        Self::with_capacity(16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(msg: &str) -> Notification {
        Notification::new(Severity::Info, msg, None)
    }
    fn warn(msg: &str) -> Notification {
        Notification::new(Severity::Warning, msg, None)
    }
    fn crit(msg: &str) -> Notification {
        Notification::new(Severity::Critical, msg, None)
    }

    /// FR-CIV-NOTIFY-900 acceptance: pushing past capacity drops the oldest.
    #[test]
    fn pushing_past_capacity_drops_oldest() {
        let mut q = NotificationQueue::with_capacity(3);
        let id1 = q.push(info("a"));
        let id2 = q.push(info("b"));
        let id3 = q.push(info("c"));
        assert_eq!(q.len(), 3);
        assert!(q.is_full());

        // 4th push: should evict "a".
        let id4 = q.push(info("d"));
        assert_eq!(q.len(), 3);
        assert!(q.get(id1).is_none(), "oldest (a) must be evicted");
        assert!(q.get(id2).is_some());
        assert!(q.get(id3).is_some());
        assert!(q.get(id4).is_some());

        // Iteration is oldest-first among survivors.
        let snap = q.snapshot();
        assert_eq!(snap[0].id, id2);
        assert_eq!(snap[1].id, id3);
        assert_eq!(snap[2].id, id4);
    }

    /// FR-CIV-NOTIFY-900 acceptance: dismiss removes by id.
    #[test]
    fn dismiss_removes_by_id() {
        let mut q = NotificationQueue::with_capacity(4);
        let id_a = q.push(info("a"));
        let id_b = q.push(info("b"));
        let id_c = q.push(info("c"));
        assert_eq!(q.len(), 3);

        assert!(q.dismiss(id_b));
        assert_eq!(q.len(), 2);
        assert!(q.get(id_a).is_some());
        assert!(q.get(id_b).is_none());
        assert!(q.get(id_c).is_some());

        // Dismissing an unknown id is a no-op.
        assert!(!q.dismiss(9999));
        assert_eq!(q.len(), 2);
    }

    /// Dismiss from the middle of a full ring preserves FIFO order of the
    /// survivors and the `len() < capacity` invariant on the head.
    #[test]
    fn dismiss_in_full_ring_keeps_remaining_order() {
        let mut q = NotificationQueue::with_capacity(3);
        let _ = q.push(info("a"));
        let id_b = q.push(info("b"));
        let _ = q.push(info("c"));
        assert!(q.is_full());
        assert!(q.dismiss(id_b));
        assert_eq!(q.len(), 2);
        let snap = q.snapshot();
        assert_eq!(snap[0].message, "a");
        assert_eq!(snap[1].message, "c");
    }

    /// FR-CIV-NOTIFY-900 acceptance: severity filter works.
    #[test]
    fn severity_filter_works() {
        let mut q = NotificationQueue::with_capacity(8);
        let _ = q.push(info("i1"));
        let _ = q.push(warn("w1"));
        let _ = q.push(crit("c1"));
        let _ = q.push(warn("w2"));
        let _ = q.push(info("i2"));

        // Threshold Info: everything.
        assert_eq!(q.iter_severity_at_least(Severity::Info).count(), 5);
        // Threshold Warning: drops the two Info entries.
        let warns: Vec<&Notification> = q.iter_severity_at_least(Severity::Warning).collect();
        assert_eq!(warns.len(), 3);
        assert!(warns.iter().all(|n| n.severity.at_least(Severity::Warning)));
        // Threshold Critical: only the one Critical entry.
        let crits: Vec<&Notification> = q.iter_severity_at_least(Severity::Critical).collect();
        assert_eq!(crits.len(), 1);
        assert_eq!(crits[0].message, "c1");
        // Threshold Fatal: nothing.
        assert_eq!(q.iter_severity_at_least(Severity::Fatal).count(), 0);
    }

    /// Severity ordering is correct: `Info < Warning < Critical < Fatal`.
    #[test]
    fn severity_ordering_is_total() {
        assert!(Severity::Info.at_least(Severity::Info));
        assert!(Severity::Warning.at_least(Severity::Info));
        assert!(Severity::Critical.at_least(Severity::Warning));
        assert!(Severity::Fatal.at_least(Severity::Critical));
        assert!(!Severity::Info.at_least(Severity::Warning));
        assert!(!Severity::Warning.at_least(Severity::Critical));
    }

    /// `dismiss_severity_at_least` removes only matching entries and
    /// reports the count.
    #[test]
    fn dismiss_severity_at_least() {
        let mut q = NotificationQueue::with_capacity(8);
        let _ = q.push(info("i1"));
        let _ = q.push(warn("w1"));
        let _ = q.push(crit("c1"));
        let _ = q.push(info("i2"));
        let removed = q.dismiss_severity_at_least(Severity::Warning);
        assert_eq!(removed, 2);
        assert_eq!(q.len(), 2);
        for n in q.iter() {
            assert_eq!(n.severity, Severity::Info);
        }
    }

    /// `clear` empties the queue but ids keep advancing.
    #[test]
    fn clear_then_push_keeps_ids_monotonic() {
        let mut q = NotificationQueue::with_capacity(2);
        let _ = q.push(info("a"));
        let _ = q.push(info("b"));
        q.clear();
        assert!(q.is_empty());
        let id1 = q.push(info("c"));
        let id2 = q.push(info("d"));
        let id3 = q.push(info("e")); // evicts id1
        assert!(q.get(id1).is_none());
        assert!(q.get(id2).is_some());
        assert!(q.get(id3).is_some());
    }

    /// Capacity 0 is an inert sink — every push is dropped, length stays 0.
    #[test]
    fn zero_capacity_silently_drops() {
        let mut q = NotificationQueue::with_capacity(0);
        let id = q.push(info("a"));
        assert_eq!(q.len(), 0);
        assert!(q.is_empty());
        assert!(q.get(id).is_none());
    }

    /// Pushing through several wraps of a small ring keeps `snapshot()`
    /// in oldest-first order.
    #[test]
    fn repeated_wraps_preserve_oldest_first_order() {
        let mut q = NotificationQueue::with_capacity(2);
        let _ = q.push(info("a"));
        let _ = q.push(info("b"));
        let _ = q.push(info("c")); // evicts a
        let _ = q.push(info("d")); // evicts b
        let _ = q.push(info("e")); // evicts c
        let snap = q.snapshot();
        assert_eq!(snap.len(), 2);
        assert_eq!(snap[0].message, "d");
        assert_eq!(snap[1].message, "e");
    }

    /// `get` and `dismiss` both work after several ring wraps.
    #[test]
    fn get_and_dismiss_after_wraps() {
        let mut q = NotificationQueue::with_capacity(2);
        let _ = q.push(info("a"));
        let id_b = q.push(info("b"));
        let id_c = q.push(info("c")); // evicts a
        assert!(q.dismiss(id_b)); // b is still in the ring (oldest)
        assert_eq!(q.len(), 1);
        assert!(q.get(id_c).is_some());
    }
}