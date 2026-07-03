//! Per-phase tick timing and budget enforcement (FR-CORE-007).
//!
//! This is **observability only** — timings are wall-clock and therefore
//! non-deterministic, so they are kept entirely out of the replay log, the
//! integrity hash chain, and save bundles. Nothing here may feed back into
//! simulation state, or replays would diverge across machines.

/// Wall-clock duration (microseconds) recorded for one named tick phase.
pub type PhaseTiming = (&'static str, u64);

/// Transient per-tick timing record. Cleared and refilled every [`Simulation::tick`].
///
/// [`Simulation::tick`]: crate::Simulation::tick
#[derive(Debug, Clone, Default)]
pub struct TickProfile {
    /// `(phase_name, micros)` in execution order for the most recent tick.
    pub phases: Vec<PhaseTiming>,
    /// Total tick wall-clock in microseconds.
    pub total_micros: u64,
}

impl TickProfile {
    /// Reset for a new tick.
    pub fn clear(&mut self) {
        self.phases.clear();
        self.total_micros = 0;
    }

    /// Record one phase's duration, accumulating the tick total.
    pub fn record(&mut self, phase: &'static str, micros: u64) {
        self.phases.push((phase, micros));
        self.total_micros = self.total_micros.saturating_add(micros);
    }

    /// The single slowest phase this tick, if any were recorded.
    #[must_use]
    pub fn slowest(&self) -> Option<PhaseTiming> {
        self.phases
            .iter()
            .copied()
            .max_by_key(|&(_, micros)| micros)
    }
}

/// Phases whose duration met or exceeded `budget_micros`, in input order.
///
/// Pure and deterministic over its inputs (the *timings* are non-deterministic,
/// but the over-budget selection is a pure function of them), so it is unit
/// testable without running the engine.
#[must_use]
pub fn phases_over_budget(timings: &[PhaseTiming], budget_micros: u64) -> Vec<PhaseTiming> {
    timings
        .iter()
        .copied()
        .filter(|&(_, micros)| micros >= budget_micros)
        .collect()
}

/// True when the total tick wall-clock met or exceeded `budget_micros`.
#[must_use]
pub fn tick_over_budget(profile: &TickProfile, budget_micros: u64) -> bool {
    profile.total_micros >= budget_micros
}

/// Rolling-window aggregator for per-tick total durations.
///
/// Maintains a fixed-size circular buffer of tick durations and computes
/// statistics (P99, max, mean) for performance budget validation.
///
/// Pure: accepts durations as input; does NOT call wall-clock internally.
/// Must be populated via explicit `push(dur_micros)` calls by the caller.
/// This ensures it remains deterministic-friendly and testable.
#[derive(Debug, Clone, Default)]
pub struct TickDurationAggregator {
    /// Circular buffer of tick durations in microseconds.
    durations: Vec<u64>,
    /// Index of next write position (round-robin).
    write_idx: usize,
}

impl TickDurationAggregator {
    /// Create a new aggregator with window size `capacity`.
    ///
    /// # Panics
    /// Panics if `capacity` is 0.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "window capacity must be > 0");
        Self {
            durations: vec![0; capacity],
            write_idx: 0,
        }
    }

    /// Append a tick duration (in microseconds). Overwrites the oldest if full.
    pub fn push(&mut self, dur_micros: u64) {
        self.durations[self.write_idx] = dur_micros;
        self.write_idx = (self.write_idx + 1) % self.durations.len();
    }

    /// Approximate 99th percentile of recorded durations.
    ///
    /// Returns the value below which ~99% of samples fall.
    /// If no durations recorded yet, returns 0.
    #[must_use]
    pub fn p99(&self) -> u64 {
        if self.durations.is_empty() {
            return 0;
        }
        let mut sorted = self.durations.clone();
        sorted.sort_unstable();
        let idx = (sorted.len() * 99) / 100;
        sorted[idx.min(sorted.len() - 1)]
    }

    /// Maximum recorded duration in the window.
    #[must_use]
    pub fn max(&self) -> u64 {
        self.durations.iter().copied().max().unwrap_or(0)
    }

    /// Mean (average) recorded duration.
    #[must_use]
    pub fn mean(&self) -> u64 {
        if self.durations.is_empty() {
            0
        } else {
            self.durations.iter().sum::<u64>() / self.durations.len() as u64
        }
    }

    /// Number of durations currently held.
    #[must_use]
    pub fn len(&self) -> usize {
        self.durations.len()
    }

    /// True if no durations have been recorded yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.durations.is_empty()
    }

    /// Clear all recorded durations.
    pub fn clear(&mut self) {
        self.durations.fill(0);
        self.write_idx = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> TickProfile {
        let mut p = TickProfile::default();
        p.record("production", 120);
        p.record("economy", 800);
        p.record("planet", 40);
        p
    }

    #[test]
    fn record_accumulates_total() {
        let p = sample();
        assert_eq!(p.total_micros, 960);
        assert_eq!(p.phases.len(), 3);
    }

    #[test]
    fn clear_resets_phases_and_total() {
        let mut p = sample();
        p.clear();
        assert!(p.phases.is_empty());
        assert_eq!(p.total_micros, 0);
    }

    #[test]
    fn slowest_picks_max_duration_phase() {
        assert_eq!(sample().slowest(), Some(("economy", 800)));
        assert_eq!(TickProfile::default().slowest(), None);
    }

    #[test]
    fn phases_over_budget_selects_only_offenders() {
        let p = sample();
        // Budget 500us: only the 800us economy phase is over.
        let over = phases_over_budget(&p.phases, 500);
        assert_eq!(over, vec![("economy", 800)]);
        // Budget above everything: none.
        assert!(phases_over_budget(&p.phases, 10_000).is_empty());
        // Budget at zero: all phases (>= 0).
        assert_eq!(phases_over_budget(&p.phases, 0).len(), 3);
    }

    #[test]
    fn tick_over_budget_compares_total() {
        let p = sample(); // total 960
        assert!(tick_over_budget(&p, 960), "boundary is inclusive");
        assert!(tick_over_budget(&p, 500));
        assert!(!tick_over_budget(&p, 961));
    }

    #[test]
    fn aggregator_new_creates_sized_buffer() {
        let agg = TickDurationAggregator::new(10);
        assert_eq!(agg.len(), 10);
        assert!(agg.is_empty() == false); // buffer exists but not written
    }

    #[test]
    #[should_panic(expected = "window capacity must be > 0")]
    fn aggregator_new_zero_capacity_panics() {
        let _ = TickDurationAggregator::new(0);
    }

    #[test]
    fn aggregator_push_records_durations() {
        let mut agg = TickDurationAggregator::new(5);
        agg.push(100);
        agg.push(200);
        agg.push(150);
        // All slots hold values, so max should be 200
        assert_eq!(agg.max(), 200);
    }

    #[test]
    fn aggregator_p99_with_single_value() {
        let mut agg = TickDurationAggregator::new(1);
        agg.push(500);
        assert_eq!(agg.p99(), 500);
    }

    #[test]
    fn aggregator_p99_with_known_values() {
        let mut agg = TickDurationAggregator::new(100);
        // Push 100 values: 1..100
        for i in 1..=100 {
            agg.push(i as u64);
        }
        // P99 should be around value 99 (99th percentile of 1..100)
        let p99 = agg.p99();
        assert!(p99 >= 98 && p99 <= 100, "p99={}, expected ~99", p99);
    }

    #[test]
    fn aggregator_max_tracks_maximum() {
        let mut agg = TickDurationAggregator::new(10);
        agg.push(50);
        agg.push(200);
        agg.push(75);
        assert_eq!(agg.max(), 200);
    }

    #[test]
    fn aggregator_mean_computes_average() {
        let mut agg = TickDurationAggregator::new(3);
        agg.push(100);
        agg.push(200);
        agg.push(300);
        // Mean of 100, 200, 300 = 200
        assert_eq!(agg.mean(), 200);
    }

    #[test]
    fn aggregator_circular_buffer_overwrites() {
        let mut agg = TickDurationAggregator::new(3);
        agg.push(10);
        agg.push(20);
        agg.push(30);
        // Buffer full: [10, 20, 30]
        assert_eq!(agg.max(), 30);
        assert_eq!(agg.mean(), 20);

        // Next push overwrites first slot
        agg.push(50);
        // Buffer now: [50, 20, 30]
        assert_eq!(agg.max(), 50);
        assert_eq!(agg.mean(), (50 + 20 + 30) / 3); // 33
    }

    #[test]
    fn aggregator_clear_resets() {
        let mut agg = TickDurationAggregator::new(5);
        agg.push(100);
        agg.push(200);
        assert_eq!(agg.max(), 200);

        agg.clear();
        assert_eq!(agg.max(), 0);
        assert_eq!(agg.mean(), 0);
        assert_eq!(agg.p99(), 0);
    }

    #[test]
    fn aggregator_p99_zero_capacity_input() {
        // Edge case: if somehow an empty aggregator is queried
        let agg = TickDurationAggregator::new(5); // created but never pushed to
        // All slots are zero, so p99 of [0, 0, 0, 0, 0] should be 0
        assert_eq!(agg.p99(), 0);
    }
}
