//! Tests for FR-CIV-PERF-016
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-016: Fixed-point arithmetic.

#[cfg(test)]
mod fr_fr_civ_perf_016 {
    /// Verify FR-CIV-PERF-016: Fixed-point type arithmetic.
    #[test]
    fn verify_fr_civ_perf_016_basic() {
        use civ_engine::Fixed;
        let a = Fixed::from_num(100i64);
        let b = Fixed::from_num(200i64);
        assert_eq!(a + b, Fixed::from_num(300i64), "addition");
        assert_eq!(b - a, Fixed::from_num(100i64), "subtraction");
    }

    /// Verify Fixed-point multiplication.
    #[test]
    fn fixed_multiplication() {
        use civ_engine::Fixed;
        let a = Fixed::from_num(10i64);
        let b = Fixed::from_num(20i64);
        assert_eq!(a * b, Fixed::from_num(200i64));
    }

    /// Verify Fixed-point division.
    #[test]
    fn fixed_division() {
        use civ_engine::Fixed;
        let a = Fixed::from_num(100i64);
        let b = Fixed::from_num(10i64);
        assert_eq!(a / b, Fixed::from_num(10i64));
    }

    /// Verify division by zero returns zero.
    #[test]
    fn fixed_division_by_zero() {
        use civ_engine::Fixed;
        let a = Fixed::from_num(100i64);
        let b = Fixed::ZERO;
        assert_eq!(a / b, Fixed::ZERO);
    }
}
