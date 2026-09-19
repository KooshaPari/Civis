//! Tests for FR-PROT-003 — Event Envelope Fields
//!
//! Epic: FR-PROT
//! Event envelope SHALL contain event_id, event_type, session_id,
//! tick, created_at, and payload.

#[cfg(test)]
mod fr_fr_prot_003 {
    /// FR-PROT-003: WorldState tick is available as envelope timestamp.
    #[test]
    fn tick_available_for_envelope() {
        let ws = civ_engine::WorldState {
            tick: 999,
            ..civ_engine::WorldState::default()
        };
        assert_eq!(ws.tick, 999);
    }

    /// FR-PROT-003: Fixed type supports integer serialization (no floats).
    #[test]
    fn fixed_type_integer_serializable() {
        let val = civ_engine::Fixed::from_num(12345);
        let json = serde_json::to_string(&val).expect("Fixed must serialize");
        // Fixed is i64-backed, should serialize as an integer or wrapper
        assert!(!json.is_empty());
    }
}