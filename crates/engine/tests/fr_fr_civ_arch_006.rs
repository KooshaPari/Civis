//! Tests for FR-CIV-ARCH-006 — Architecture and crate design
//!
//! Epic: FR-CIV-ARCH
//! Verifies that the engine crate exposes a coherent public API surface:
//! core simulation types, re-exported subsystem types, and module structure.

#[cfg(test)]
mod fr_fr_civ_arch_006 {
    /// FR-CIV-ARCH-006: The engine crate re-exports core simulation types
    /// (WorldState, Simulation, Fixed, etc.) at the crate root.
    #[test]
    fn world_state_is_public_and_defaultable() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
        assert!(ws.population == 0 || ws.population > 0); // population is u64, always non-negative
    }

    /// FR-CIV-ARCH-006: Simulation::with_seed creates a valid simulation.
    #[test]
    fn simulation_construction_with_seed() {
        let sim = civ_engine::Simulation::with_seed(42);
        assert_eq!(sim.state.tick, 0);
    }

    /// FR-CIV-ARCH-006: Fixed-point math type is accessible and usable.
    #[test]
    fn fixed_point_type_is_accessible() {
        let a = civ_engine::Fixed::from_num(100);
        let b = civ_engine::Fixed::from_num(50);
        let sum = a + b;
        assert_eq!(sum, civ_engine::Fixed::from_num(150));
    }

    /// FR-CIV-ARCH-006: Step function advances the world state.
    #[test]
    fn step_function_is_public() {
        let ws = civ_engine::WorldState::default();
        let next = civ_engine::step(ws, civ_engine::Fixed::from_num(0));
        assert_eq!(next.tick, 1);
    }

    /// FR-CIV-ARCH-006: Key re-exported types compile (building, building type).
    #[test]
    fn building_types_are_reexported() {
        use civ_engine::{Building, BuildingType};
        let b = Building {
            building_type: BuildingType::CityCenter,
            hp: civ_engine::Fixed::from_num(100),
            max_hp: civ_engine::Fixed::from_num(100),
            position: civ_engine::Position { x: 0, y: 0 },
        };
        assert_eq!(b.building_type, BuildingType::CityCenter);
    }

    /// FR-CIV-ARCH-006: SCALE constant defines fixed-point scaling.
    #[test]
    fn scale_constant_is_positive() {
        assert!(civ_engine::SCALE > 0);
        assert_eq!(civ_engine::SCALE, 1_000);
    }
}
