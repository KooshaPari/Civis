//! Authoring spawn helpers (FR-CIV-UX-006) — normalized map coords to ECS entities.

use hecs::{Entity, World};

use crate::{Building, BuildingType, Fixed, MilitaryUnit, Position, UnitType};

/// Map normalized terrain coords (0..1) to the engine hex grid.
pub fn norm_to_grid(x: f32, y: f32) -> Position {
    Position {
        x: (x.clamp(0.0, 1.0) * 127.0).round() as i32 - 64,
        y: (y.clamp(0.0, 1.0) * 127.0).round() as i32 - 64,
    }
}

/// Map grid position back to normalized coords for spectator pins.
pub fn grid_to_norm(pos: Position) -> (f32, f32) {
    (
        ((pos.x + 64) as f32 / 127.0).clamp(0.0, 1.0),
        ((pos.y + 64) as f32 / 127.0).clamp(0.0, 1.0),
    )
}

/// Wire label for military units (Knight → Vehicle for spawn palette).
pub fn unit_type_label(unit_type: UnitType) -> &'static str {
    match unit_type {
        UnitType::Soldier => "Soldier",
        UnitType::Archer => "Archer",
        UnitType::Knight => "Vehicle",
        UnitType::Scout => "Scout",
    }
}

/// Spawn a military unit at normalized coords (vehicle palette → Knight).
pub fn spawn_military_at(
    world: &mut World,
    faction: u32,
    x: f32,
    y: f32,
    unit_type: UnitType,
) -> Entity {
    world.spawn((MilitaryUnit {
        unit_type,
        strength: Fixed::from_num(10),
        morale: Fixed::from_num(1),
        position: norm_to_grid(x, y),
        faction_id: faction,
    },))
}

/// Spawn an airport (civic hub) building at normalized coords.
pub fn spawn_airport_at(world: &mut World, x: f32, y: f32) -> Entity {
    world.spawn((Building {
        building_type: BuildingType::CityCenter,
        hp: Fixed::from_num(500),
        max_hp: Fixed::from_num(500),
        position: norm_to_grid(x, y),
    },))
}

/// Spawn a harbor / trade port (`Market`) at normalized coords.
pub fn spawn_port_at(world: &mut World, x: f32, y: f32) -> Entity {
    world.spawn((Building {
        building_type: BuildingType::Market,
        hp: Fixed::from_num(350),
        max_hp: Fixed::from_num(350),
        position: norm_to_grid(x, y),
    },))
}

/// Spawn a hangar / barracks (`Barracks`) at normalized coords.
pub fn spawn_hangar_at(world: &mut World, x: f32, y: f32) -> Entity {
    world.spawn((Building {
        building_type: BuildingType::Barracks,
        hp: Fixed::from_num(400),
        max_hp: Fixed::from_num(400),
        position: norm_to_grid(x, y),
    },))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    #[test]
    fn norm_to_grid_maps_center() {
        let p = norm_to_grid(0.5, 0.5);
        assert!((p.x).abs() <= 1);
        assert!((p.y).abs() <= 1);
    }

    #[test]
    fn spawn_military_and_airport_insert_components() {
        let mut world = World::new();
        let _mil = spawn_military_at(&mut world, 1, 0.2, 0.8, UnitType::Knight);
        let _air = spawn_airport_at(&mut world, 0.7, 0.3);
        let _port = spawn_port_at(&mut world, 0.3, 0.7);
        let _hangar = spawn_hangar_at(&mut world, 0.5, 0.5);
        assert_eq!(world.query::<&MilitaryUnit>().iter().count(), 1);
        assert_eq!(world.query::<&Building>().iter().count(), 3);
    }
}
