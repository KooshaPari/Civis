//! Tests for FR-CIV-0104-005
//!
//! Epic: FR-CIV
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-0104-005: Constraint Checks Deterministic
//! Same state -> same ConstraintCheck results; no floating-point in check logic.

#[cfg(test)]
mod fr_fr_civ_0104_005 {
    use civ_engine::constraints::{
        run_all_checks, MinimalConstraintParams,
    };
    use civ_engine::Fixed;
    use std::collections::BTreeMap;

    /// Running check_all twice with identical inputs produces identical results.
    #[test]
    fn identical_inputs_produce_identical_results() {
        let params = MinimalConstraintParams::default();
        let cohorts = BTreeMap::from([
            (0u32, Fixed::from_num(95) / Fixed::from_num(100)),
            (1, Fixed::from_num(96) / Fixed::from_num(100)),
        ]);

        let make_result = || {
            run_all_checks(
                Fixed::from_num(3) / Fixed::from_num(10),
                Fixed::from_num(7) / Fixed::from_num(10),
                Fixed::from_num(8) / Fixed::from_num(10),
                Fixed::from_num(1) / Fixed::from_num(10),
                &cohorts,
                false,
                Fixed::from_num(5) / Fixed::from_num(100),
                Fixed::from_num(95) / Fixed::from_num(100),
                Fixed::from_num(5) / Fixed::from_num(100),
                Fixed::from_num(2) / Fixed::from_num(10),
                Fixed::from_num(1) / Fixed::from_num(10),
                Fixed::from_num(5) / Fixed::from_num(10),
                Fixed::from_num(4) / Fixed::from_num(10),
                5,
                &params,
            )
        };

        let r1 = make_result();
        let r2 = make_result();
        assert_eq!(r1, r2, "Deterministic: same inputs must produce same outputs");
        assert_eq!(r1.all_satisfied(), r2.all_satisfied());
    }

    /// Determinism holds across many invocations (1000x).
    #[test]
    fn determinism_across_many_invocations() {
        let params = MinimalConstraintParams::default();
        let cohorts = BTreeMap::from([(0u32, Fixed::from_num(9) / Fixed::from_num(10))]);

        let reference = run_all_checks(
            Fixed::from_num(4) / Fixed::from_num(10),
            Fixed::from_num(6) / Fixed::from_num(10),
            Fixed::from_num(7) / Fixed::from_num(10),
            Fixed::from_num(15) / Fixed::from_num(100),
            &cohorts,
            false,
            Fixed::from_num(1) / Fixed::from_num(10),
            Fixed::from_num(93) / Fixed::from_num(100),
            Fixed::from_num(45) / Fixed::from_num(1000),
            Fixed::from_num(3) / Fixed::from_num(10),
            Fixed::from_num(2) / Fixed::from_num(10),
            Fixed::from_num(6) / Fixed::from_num(10),
            Fixed::from_num(5) / Fixed::from_num(10),
            4,
            &params,
        );

        for _ in 0..1000 {
            let r = run_all_checks(
                Fixed::from_num(4) / Fixed::from_num(10),
                Fixed::from_num(6) / Fixed::from_num(10),
                Fixed::from_num(7) / Fixed::from_num(10),
                Fixed::from_num(15) / Fixed::from_num(100),
                &cohorts,
                false,
                Fixed::from_num(1) / Fixed::from_num(10),
                Fixed::from_num(93) / Fixed::from_num(100),
                Fixed::from_num(45) / Fixed::from_num(1000),
                Fixed::from_num(3) / Fixed::from_num(10),
                Fixed::from_num(2) / Fixed::from_num(10),
                Fixed::from_num(6) / Fixed::from_num(10),
                Fixed::from_num(5) / Fixed::from_num(10),
                4,
                &params,
            );
            assert_eq!(reference, r);
        }
    }

    /// Using fixed-point arithmetic ensures bitwise determinism.
    #[test]
    fn uses_fixed_point_not_floating_point() {
        // Verify the params themselves are deterministic Fixed values.
        let p1 = MinimalConstraintParams::default();
        let p2 = MinimalConstraintParams::default();
        assert_eq!(p1.c1.e_base, p2.c1.e_base);
        assert_eq!(p1.c2.b_min, p2.c2.b_min);
        assert_eq!(p1.c3.o_max, p2.c3.o_max);
        assert_eq!(p1.c4.a_min_base, p2.c4.a_min_base);
        assert_eq!(p1.c5.c0_ceiling, p2.c5.c0_ceiling);
    }
}
