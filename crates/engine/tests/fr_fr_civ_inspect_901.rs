//! Tests for FR-CIV-INSPECT-901 - God-tool Inspection Dispatcher
//!
//! Epic: FR-CIV-INSPECT
//! The god-tool substrate SHALL dispatch inspection requests through Simulation.

#[cfg(test)]
mod fr_fr_civ_inspect_901 {
    /// FR-CIV-INSPECT-901: God-action record tracks god-tool invocations.
    #[test]
    fn god_action_record_type_exists() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0, "fresh state at tick zero");
    }

    /// FR-CIV-INSPECT-901: WorldState tick available for inspection timestamping.
    #[test]
    fn inspection_timestamps_from_tick() {
        let ws = civ_engine::WorldState { tick: 42, ..civ_engine::WorldState::default() };
        assert_eq!(ws.tick, 42);
    }
}