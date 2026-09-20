//! Tests for FR-CIV-3D-012
//!
//! Epic: FR-CIV-3D
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-3D-012: Protocol Agnosticism
//! The 3D client uses the same JSON-RPC WebSocket protocol as the 2D client.
//! Engine-side: verify the snapshot/state format is protocol-independent.

#[cfg(test)]
mod fr_fr_civ_3d_012 {
    use civ_engine::WorldState;

    /// WorldState serializes to JSON (the shared protocol format).
    #[test]
    fn worldstate_json_serializable() {
        let ws = WorldState::default();
        let json = serde_json::to_string(&ws).expect("WorldState must serialize to JSON");
        assert!(!json.is_empty(), "JSON output must not be empty");
    }

    /// WorldState can round-trip through JSON (protocol agnosticism).
    #[test]
    fn worldstate_json_roundtrip() {
        let ws = WorldState::default();
        let json = serde_json::to_string(&ws).expect("serialize");
        let ws2: WorldState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(ws.tick, ws2.tick);
        assert_eq!(ws.population, ws2.population);
        assert_eq!(ws.factions.len(), ws2.factions.len());
    }

    /// The same WorldState produces the same JSON regardless of invocation.
    #[test]
    fn identical_json_output() {
        let ws = WorldState::default();
        let json1 = serde_json::to_string(&ws).unwrap();
        let json2 = serde_json::to_string(&ws).unwrap();
        assert_eq!(json1, json2, "Same state must produce same JSON");
    }
}
