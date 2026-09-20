//! Tests for FR-CIV-3D-014
//!
//! Epic: FR-CIV-3D
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-3D-014: View Layer Purity
//! The 3D client contains zero simulation logic; all state comes from core.
//! Engine-side: verify the engine provides all necessary state through
//! the WorldState snapshot — the client never computes its own.

#[cfg(test)]
mod fr_fr_civ_3d_014 {
    use civ_engine::WorldState;
    use civ_engine::Fixed;

    /// WorldState provides all required simulation state for rendering.
    #[test]
    fn worldstate_has_all_rendering_fields() {
        let ws = WorldState::default();
        // Population for demographic overlays.
        assert!(ws.population > 0);
        // Tick for time display.
        assert_eq!(ws.tick, 0);
        // Factions for nation colors.
        assert!(!ws.factions.is_empty());
        // Resources for economy display.
        assert!(ws.resources.food >= Fixed::ZERO);
        assert!(ws.resources.wood >= Fixed::ZERO);
        assert!(ws.resources.metal >= Fixed::ZERO);
        assert!(ws.resources.energy >= Fixed::ZERO);
    }

    /// WorldState equality is well-defined (client can diff snapshots).
    #[test]
    fn worldstate_equality_works() {
        let ws1 = WorldState::default();
        let ws2 = WorldState::default();
        assert_eq!(ws1, ws2, "Default WorldStates must be equal");
    }

    /// Step produces a new state (client receives new snapshots, not mutations).
    #[test]
    fn step_produces_new_state() {
        let ws1 = WorldState::default();
        let ws2 = civ_engine::step(ws1, Fixed::from_num(10));
        assert_ne!(ws1.tick, ws2.tick, "Step must advance tick");
    }
}
