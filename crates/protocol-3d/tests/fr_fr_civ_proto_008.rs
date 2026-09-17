//! Tests for FR-CIV-PROTO-008
//!
//! Epic: FR-CIV-PROTO
//! Status: IMPLEMENTED
//!
//! Covers the 3D civilian-needs payload and the agent-appearance frame. The
//! appearance payload is carried by `AgentAppearanceFrame` (a batch of
//! `AgentAppearanceUpdate`s), not a standalone `AgentAppearance3d` type.

#[cfg(test)]
mod fr_fr_civ_proto_008 {
    use civ_protocol_3d::{AgentAppearanceFrame, CivilianNeeds3d, Frame3d};

    #[test]
    fn verify_fr_civ_proto_008_basic() {
        // Needs default to zero and are normalized to 0.0..=1.0.
        let needs = CivilianNeeds3d::default();
        assert_eq!(needs.food, 0.0);
        for (label, value) in [
            ("food", needs.food),
            ("shelter", needs.shelter),
            ("safety", needs.safety),
            ("social", needs.social),
            ("rest", needs.rest),
        ] {
            assert!(
                (0.0..=1.0).contains(&value),
                "{label} must be normalized, got {value}"
            );
        }

        // The struct is plain data: fields round-trip through serde unchanged.
        let populated = CivilianNeeds3d {
            food: 0.25,
            shelter: 0.5,
            safety: 0.75,
            social: 1.0,
            rest: 0.0,
        };
        let encoded = serde_json::to_string(&populated).expect("needs serialize");
        let decoded: CivilianNeeds3d = serde_json::from_str(&encoded).expect("needs deserialize");
        assert_eq!(decoded, populated);

        // An empty appearance frame is valid and reports its tick through the
        // Frame3d enum.
        let frame = Frame3d::AgentAppearance(AgentAppearanceFrame {
            tick: 5,
            updates: vec![],
        });
        assert_eq!(frame.tick(), 5);
    }
}
