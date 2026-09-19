//! Tests for FR-CIV-VEHICLE-001 - Military Unit Types
//!
//! Epic: FR-CIV-VEHICLE
//! Military units SHALL have defined types (Soldier, Archer, Knight, Scout).

#[cfg(test)]
mod fr_fr_civ_vehicle_001 {
    /// FR-CIV-VEHICLE-001: UnitType label for Knight is Vehicle.
    #[test]
    fn knight_labeled_as_vehicle() {
        assert_eq!(civ_engine::unit_type_label(civ_engine::UnitType::Knight), "Vehicle");
    }

    /// FR-CIV-VEHICLE-001: All unit type labels are non-empty.
    #[test]
    fn all_unit_labels_nonempty() {
        assert!(!civ_engine::unit_type_label(civ_engine::UnitType::Soldier).is_empty());
        assert!(!civ_engine::unit_type_label(civ_engine::UnitType::Archer).is_empty());
        assert!(!civ_engine::unit_type_label(civ_engine::UnitType::Scout).is_empty());
    }
}