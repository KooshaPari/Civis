//! Tests for FR-NET-001 — Network State Synchronization
//!
//! Epic: FR-NET
//! The engine SHALL expose a WorldState that tracks tick, population,
//! and energy budget. Network clients synchronize to this state.

#[cfg(test)]
mod fr_fr_net_001 {
    /// FR-NET-001: WorldState initializes with tick zero and default energy.
    #[test]
    fn world_state_initializes_at_tick_zero() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0, "initial tick must be zero");
        assert!(ws.energy_budget_joules > civ_engine::Fixed::ZERO, "initial energy must be positive");
        assert_eq!(ws.population, 0, "initial population is zero");
    }

    /// FR-NET-001: Step function advances tick by exactly one.
    #[test]
    fn step_advances_tick_by_one() {
        let ws = civ_engine::WorldState::default();
        let ws2 = civ_engine::step(ws, civ_engine::Fixed::from_num(0));
        assert_eq!(ws2.tick, 1, "tick must advance by exactly one");
    }

    /// FR-NET-001: Tick counter monotonically increases across multiple steps.
    #[test]
    fn tick_monotonically_increases() {
        let mut ws = civ_engine::WorldState::default();
        for i in 1..=10 {
            ws = civ_engine::step(ws, civ_engine::Fixed::from_num(0));
            assert_eq!(ws.tick, i);
        }
    }
}