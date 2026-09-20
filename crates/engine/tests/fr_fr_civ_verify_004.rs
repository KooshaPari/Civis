//! Tests for FR-CIV-VERIFY-004
//! Epic: FR-CIV-VERIFY. Catalog check (no JSON-RPC drift).
#[cfg(test)]
mod fr_fr_civ_verify_004 {
    #[test]
    fn world_state_schema_is_stable() {
        // FR-CIV-VERIFY-004: catalog surface must not drift.
        // Engine-side: WorldState must be serializable with stable schema.
        let ws = civ_engine::WorldState::default();
        let json = serde_json::to_string(&ws).expect("must serialize");
        let ws2: civ_engine::WorldState = serde_json::from_str(&json).expect("must deserialize");
        assert_eq!(ws.factions.len(), ws2.factions.len());
        assert_eq!(ws.tick, ws2.tick);
    }
}
