//! Tests for FR-CIV-VEHICLE-005 - Unit Type Classification
//!
//! Epic: FR-CIV-VEHICLE
//! Each unit SHALL have a distinct type classification.

#[cfg(test)]
mod fr_fr_civ_vehicle_005 {
    /// FR-CIV-VEHICLE-005: UnitType variants are distinct.
    #[test]
    fn unit_types_are_distinct() {
        let types = [
            civ_engine::UnitType::Soldier,
            civ_engine::UnitType::Archer,
            civ_engine::UnitType::Knight,
            civ_engine::UnitType::Scout,
        ];
        for (i, a) in types.iter().enumerate() {
            for (j, b) in types.iter().enumerate() {
                if i != j {
                    assert_ne!(format!("{:?}", a), format!("{:?}" , b));
                }
            }
        }
    }
}