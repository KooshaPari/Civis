//! Tests for FR-CIV-DET-003 — Determinism (fixed-point math)
//!
//! Epic: FR-CIV-DET
//! Verifies fixed-point arithmetic produces deterministic cross-platform results.

#[cfg(test)]
mod fr_fr_civ_det_003 {
    /// FR-CIV-DET-003: Fixed-point addition is exact.
    #[test]
    fn fixed_addition_exact() {
        let a = civ_engine::Fixed::from_num(100);
        let b = civ_engine::Fixed::from_num(200);
        assert_eq!(a + b, civ_engine::Fixed::from_num(300));
    }

    /// FR-CIV-DET-003: Fixed-point subtraction is exact.
    #[test]
    fn fixed_subtraction_exact() {
        let a = civ_engine::Fixed::from_num(500);
        let b = civ_engine::Fixed::from_num(200);
        assert_eq!(a - b, civ_engine::Fixed::from_num(300));
    }

    /// FR-CIV-DET-003: Zero constant is correctly defined.
    #[test]
    fn fixed_zero_is_zero() {
        assert_eq!(civ_engine::Fixed::ZERO, civ_engine::Fixed::from_num(0));
    }

    /// FR-CIV-DET-003: One constant is correctly defined.
    #[test]
    fn fixed_one_is_one() {
        assert_eq!(civ_engine::Fixed::ONE, civ_engine::Fixed::from_num(1));
    }

    /// FR-CIV-DET-003: SCALE constant matches 1000 for joule conversion.
    #[test]
    fn scale_matches_joule_conversion() {
        assert_eq!(civ_engine::SCALE, 1_000);
    }

    /// FR-CIV-DET-003: Max and min operations are deterministic.
    #[test]
    fn fixed_max_min_deterministic() {
        let a = civ_engine::Fixed::from_num(10);
        let b = civ_engine::Fixed::from_num(20);
        assert_eq!(a.max(b), b);
        assert_eq!(a.min(b), a);
    }
}
