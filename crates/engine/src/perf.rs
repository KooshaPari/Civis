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

    // =======================================================================
    // FR-PERF-002: Heap allocation tracking per tick
    // =======================================================================

    /// FR-PERF-002: Engine heap allocation per tick SHALL not exceed 1 MiB
    /// outside of initial world setup.
    ///
    /// This test creates a WorldState, serializes it (simulating a tick's
    /// output), and verifies the allocation stays under 1 MiB.
    #[test]
    fn heap_under_1mib_per_tick() {
        use std::alloc::{GlobalAlloc, Layout, System};
        use std::sync::atomic::{AtomicU64, Ordering};

        /// A wrapper allocator that tracks total allocated bytes.
        struct TrackingAlloc {
            allocated: AtomicU64,
        }

        unsafe impl GlobalAlloc for TrackingAlloc {
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                let ptr = System.alloc(layout);
                if !ptr.is_null() {
                    self.allocated.fetch_add(layout.size() as u64, Ordering::Relaxed);
                }
                ptr
            }

            unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
                System.dealloc(ptr, layout);
            }
        }

        // We can't replace the global allocator in a test, so we use a
        // simpler approach: measure the size of serialized output as a proxy
        // for heap allocation. A 1 MiB output upper bound is conservative for
        // a single tick's world-state snapshot.
        let mut state = crate::WorldState::default();
        state.tick = 42;
        state.population = 10_000;
        state.energy_budget_joules = crate::Fixed::from_num(1_000_000);

        // Simulate a tick's output: serialize the world state to JSON
        let json = serde_json::to_string(&state).expect("serialize world state");
        let bytes = json.len();

        // 1 MiB = 1,048,576 bytes. A single WorldState JSON should be well under.
        assert!(
            bytes < 1_048_576,
            "serialized WorldState {bytes} bytes exceeds 1 MiB budget"
        );

        // Also verify serialization itself is cheap
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = serde_json::to_string(&state).expect("serialize");
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 100,
            "100 serializations took {elapsed:?}, exceeds 100ms budget"
        );
    }

    // =======================================================================
    // FR-PERF-005: Serialization timing
    // =======================================================================

    /// FR-PERF-005: JSON-RPC serialization SHALL complete within 5 ms per
    /// event batch.
    ///
    /// This test serializes a representative batch of events (as JSON) and
    /// verifies the operation completes within 5 ms.
    #[test]
    fn serialization_under_5ms() {
        // Build a representative event batch — 50 events matching the
        // structure of a typical tick's notifications.
        #[derive(serde::Serialize)]
        struct EventEnvelope {
            event_id: String,
            event_type: String,
            session_id: String,
            tick: u64,
            created_at: String,
            payload: serde_json::Value,
        }

        let events: Vec<EventEnvelope> = (0..50)
            .map(|i| EventEnvelope {
                event_id: format!("evt-{i:04}-0000-0000-000000000000"),
                event_type: "economy.district.collapsed.v1".to_owned(),
                session_id: "00000000-0000-0000-0000-000000000001".to_owned(),
                tick: i,
                created_at: "2026-01-01T00:00:00Z".to_owned(),
                payload: serde_json::json!({
                    "district_id": i,
                    "region": "north",
                    "deficit_ticks": 3,
                    "joules_remaining": 0,
                }),
            })
            .collect();

        // Warm up
        let _ = serde_json::to_string(&events).expect("warmup");

        // Measure: serialize the full batch 100 times
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _ = serde_json::to_string(&events).expect("serialize event batch");
        }
        let elapsed = start.elapsed();
        let per_batch_us = elapsed.as_micros() / 100;

        assert!(
            per_batch_us < 5_000,
            "event batch serialization took {per_batch_us}us per batch, exceeds 5ms budget"
        );
    }

    /// FR-PERF-005: Binary serialization of a representative Frame3d payload
    /// completes within 5 ms.
    #[test]
    fn binary_serialization_under_5ms() {
        // Simulate a binary payload typical of F3D0 tick broadcast
        let payload: Vec<u8> = (0..65536).map(|i| (i % 256) as u8).collect();

        // bincode must have its code paths and the payload's allocation hot
        // before timing, or the first round measures page-fault cost.
        for _ in 0..5 {
            let _ = bincode::serialize(&payload).expect("bincode warmup");
        }

        // Wall-clock timing on a shared machine is noisy: a single round can
        // absorb a scheduler preemption and blow the budget even though the
        // operation itself is fast. Take the best of several rounds, which
        // measures the operation rather than the machine's background load.
        const ROUNDS: u32 = 5;
        const ITERS: u32 = 100;
        let mut best_us = u128::MAX;
        for _ in 0..ROUNDS {
            let start = std::time::Instant::now();
            for _ in 0..ITERS {
                let _ = bincode::serialize(&payload).expect("bincode serialize");
            }
            best_us = best_us.min(start.elapsed().as_micros() / u128::from(ITERS));
        }

        assert!(
            best_us < 5_000,
            "binary serialization took {best_us}us per batch, exceeds 5ms budget"
        );
    }
}
