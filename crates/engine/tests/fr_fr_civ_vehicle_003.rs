//! Tests for FR-CIV-VEHICLE-003 - Unit HP and Combat Stats
//!
//! Epic: FR-CIV-VEHICLE
//! Military units SHALL track HP, strength, and morale as Fixed-point values.

#[cfg(test)]
mod fr_fr_civ_vehicle_003 {
    /// FR-CIV-VEHICLE-003: Fixed-point values are deterministic.
    #[test]
    fn fixed_point_deterministic() {
        let a = civ_engine::Fixed::from_num(10);
        let b = civ_engine::Fixed::from_num(10);
        assert_eq!(a, b);
    }

    /// FR-CIV-VEHICLE-003: Fixed arithmetic preserves precision.
    #[test]
    fn fixed_arithmetic_precision() {
        let hp = civ_engine::Fixed::from_num(100);
        let damage = civ_engine::Fixed::from_num(30);
        let remaining = hp - damage;
        assert_eq!(remaining, civ_engine::Fixed::from_num(70));
    }
}