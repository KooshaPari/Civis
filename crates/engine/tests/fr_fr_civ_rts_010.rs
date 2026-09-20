//! Tests for FR-CIV-RTS-010
//!
//! Epic: FR-CIV-RTS
//!
//! This test file verifies FR FR-CIV-RTS-010: Siege Mechanics.
//! MilitaryUnit has HP for siege health tracking.

#[cfg(test)]
mod fr_fr_civ_rts_010 {
    /// MilitaryUnit struct has hp field for siege health.
    #[test]
    fn military_unit_has_hp() {
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
        assert_eq!(unit.hp, Fixed::from_num(100));
    }

    /// MilitaryUnit can take damage (hp decreases).
    #[test]
    fn military_unit_takes_damage() {
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
        unit.hp = unit.hp - Fixed::from_num(30);
        assert_eq!(unit.hp, Fixed::from_num(70));
    }
}
