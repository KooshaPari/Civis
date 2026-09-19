//! Tests for FR-CIV-CORE-014
//!
//! Epic: FR-CIV-CORE
//!
//! This test file verifies FR FR-CIV-CORE-014: Event Logging.
//! Every state-mutating action emits event to log.

#[cfg(test)]
mod fr_fr_civ_core_014 {
    /// After at least one tick, the replay log must have events recorded.
    #[test]
    fn tick_produces_events() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        assert!(
            sim.replay_log().events.is_empty(),
            "fresh sim has no events"
        );
        sim.tick();
        assert!(
            !sim.replay_log().events.is_empty(),
            "after tick, replay log must have events"
        );
    }

    /// Each tick appends at least a Tick marker event.
    #[test]
    fn each_tick_records_marker() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        for t in 1..=3 {
            sim.tick();
            let log = sim.replay_log();
            // The last event should be a Tick marker for the current tick
            let last = log.events.last().expect("at least one event");
            assert!(
                matches!(last, civ_engine::replay::ReplayEvent::Tick { tick } if *tick == t),
                "last event at tick {} should be Tick marker, got {:?}",
                t,
                last
            );
        }
    }
}
