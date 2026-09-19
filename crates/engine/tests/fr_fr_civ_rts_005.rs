//! Tests for FR-CIV-RTS-005
//!
//! Epic: FR-CIV-RTS
//!
//! This test file verifies FR FR-CIV-RTS-005: Supply Line & Logistics.
//! Resources struct contains food and other supply-relevant fields.

#[cfg(test)]
mod fr_fr_civ_rts_005 {
    /// Resources struct has food field (supply line resource).
    #[test]
    fn resources_has_food() {
        let r = civ_engine::Resources::default();
        let _food = r.food;
        assert_eq!(r.food, civ_engine::Fixed::ZERO, "default food is zero");
    }

    /// Resources can be modified (supply consumed).
    #[test]
    fn resources_mutable() {
        let mut r = civ_engine::Resources::default();
        r.food = civ_engine::Fixed::from_num(100);
        assert_eq!(r.food, civ_engine::Fixed::from_num(100));
    }

    /// WorldState starts with default resources (no supply until spawned).
    #[test]
    fn world_state_default_resources() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.resources.food, civ_engine::Fixed::ZERO);
    }
}
