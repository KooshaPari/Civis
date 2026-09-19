//! Tests for FR-CIV-CORE-012
//!
//! Epic: FR-CIV-CORE
//!
//! This test file verifies FR FR-CIV-CORE-012: Fixed-Point Arithmetic.
//! No floating-point in money, resources, or energy; all i64 internally.

#[cfg(test)]
mod fr_fr_civ_core_012 {
    /// Fixed type wraps i64, not f64.
    #[test]
    fn fixed_is_i64_backed() {
        let f = civ_engine::Fixed::from_num(100i64);
        assert_eq!(f.to_bits(), 100_000, "scale factor is 1000");
    }

    /// Fixed arithmetic is deterministic: a + b == b + a.
    #[test]
    fn fixed_addition_commutative() {
        let a = civ_engine::Fixed::from_num(10i64);
        let b = civ_engine::Fixed::from_num(20i64);
        assert_eq!(a + b, b + a);
    }

    /// Energy budget uses Fixed, not f64.
    #[test]
    fn energy_budget_is_fixed() {
        let mut sim = civ_engine::Simulation::with_seed(1);
        sim.tick();
        let budget = sim.state.energy_budget_joules;
        // Fixed is Clone + Copy, not a float
        let _ = budget.to_bits(); // i64
        assert!(budget >= civ_engine::Fixed::ZERO, "energy must be non-negative");
    }

    /// Fixed::ZERO == Fixed::default().
    #[test]
    fn fixed_zero_is_default() {
        assert_eq!(civ_engine::Fixed::ZERO, civ_engine::Fixed::default());
    }
}
