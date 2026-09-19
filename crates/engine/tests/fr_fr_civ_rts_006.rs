//! Tests for FR-CIV-RTS-006
//!
//! Epic: FR-CIV-RTS
//!
//! This test file verifies FR FR-CIV-RTS-006: Structure Construction & Management.
//! BuildingType enum exists with expected structure-related variants.

#[cfg(test)]
mod fr_fr_civ_rts_006 {
    /// BuildingType enum exists and can be constructed.
    #[test]
    fn building_type_exists() {
        use civ_engine::BuildingType;
        let _house = BuildingType::House;
        let _farm = BuildingType::Farm;
        let _barracks = BuildingType::Barracks;
        let _temple = BuildingType::Temple;
        let _market = BuildingType::Market;
        let _mine = BuildingType::Mine;
        let _city_center = BuildingType::CityCenter;
    }

    /// Building struct exists with position and HP.
    #[test]
    fn building_has_position_and_hp() {
        use civ_engine::{Building, BuildingType, Position, Fixed};
        let b = Building {
            building_type: BuildingType::House,
            position: Position { x: 0, y: 0 },
            hp: Fixed::from_num(100),
            max_hp: Fixed::from_num(100),
        };
        assert_eq!(b.position.x, 0);
        assert_eq!(b.position.y, 0);
        assert_eq!(b.hp, Fixed::from_num(100));
    }

    /// WorldState supports faction_resources for construction supply tracking.
    #[test]
    fn faction_resources_tracked() {
        let mut ws = civ_engine::WorldState::default();
        ws.faction_resources
            .insert(1, civ_engine::Resources::default());
        assert!(ws.faction_resources.contains_key(&1));
    }
}
