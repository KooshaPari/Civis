//! Tests for FR-CIV-INSPECT-920 - Voxel Inspection
//!
//! Epic: FR-CIV-INSPECT
//! Voxel substrate SHALL be inspectable through god-tool dispatch.

#[cfg(test)]
mod fr_fr_civ_inspect_920 {
    /// FR-CIV-INSPECT-920: Default energy budget is inspectable.
    #[test]
    fn energy_budget_inspectable() {
        let ws = civ_engine::WorldState::default();
        assert!(ws.energy_budget_joules > civ_engine::Fixed::ZERO);
    }
}