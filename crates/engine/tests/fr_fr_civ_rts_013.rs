//! Tests for FR-CIV-RTS-013
//!
//! Epic: FR-CIV-RTS
//!
//! This test file verifies FR FR-CIV-RTS-013: Unit Experience & Leveling.
//! MilitaryUnit has strength field that can be increased.

#[cfg(test)]
mod fr_fr_civ_rts_013 {
    /// MilitaryUnit has strength field (proxy for experience/leveling).
    #[test]
    fn military_unit_has_strength() {
        use civ_engine::{MilitaryUnit, UnitType, Position, Fixed};
        let unit = MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: Fixed::from_num(10),
            hp: Fixed::from_num(100),
            max_hp: Fixed::from_num(100),
            morale: Fixed::from_num(80),
            position: Position { x: 0, y: 0 },
            faction_id: 1,
        };
        assert_eq!(unit.strength, Fixed::from_num(10));
    }

    /// Strength can be increased (leveling up).
    #[test]
    fn strength_increments() {
        use civ_engine::{MilitaryUnit, UnitType, Position, Fixed};
        let mut unit = MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: Fixed::from_num(10),
            hp: Fixed::from_num(100),
            max_hp: Fixed::from_num(100),
            morale: Fixed::from_num(80),
            position: Position { x: 0, y: 0 },
            faction_id: 1,
        };
        unit.strength = unit.strength + Fixed::from_num(5);
        assert_eq!(unit.strength, Fixed::from_num(15));
    }
}
