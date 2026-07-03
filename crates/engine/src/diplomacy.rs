//! Diplomacy module — macro-level diplomatic event processing for the simulation.
//!
//! This module implements the high-level diplomacy phase logic that runs
//! every 500 ticks, processing faction relation drift from proximity,
//! competition, trade, and combat interactions.

impl crate::engine::Simulation {
    /// Run macro-level diplomacy event processing (called every 500 ticks).
    ///
    /// This processes per-tick relation drift from proximity, competition, trade,
    /// religion, and combat interactions, emitting diplomacy events when faction
    /// relations cross significant thresholds.
    pub fn run_macro_diplomacy_event(&mut self) {
        self.tick_faction_relation_drift();
    }
}
