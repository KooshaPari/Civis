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
        self.phases.iter().copied().max_by_key(|&(_, micros)| micros)
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
}
