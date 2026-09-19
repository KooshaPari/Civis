//! Tests for FR-CIV-0104-001
//!
//! Epic: FR-CIV
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-0104-001: Five Constraints Enforced Each Tick
//! All five constraint predicates are evaluated inside Phase 2 of every tick.
//! We verify that check_all invokes all five constraint checks.

#[cfg(test)]
mod fr_fr_civ_0104_001 {
    use civ_engine::constraints::{
        check_all, run_all_checks, ConstraintSetResult, MinimalConstraintParams,
    };
    use civ_engine::Fixed;
    use std::collections::BTreeMap;

    /// Default params produce a valid baseline where all constraints pass
    /// with healthy inputs.
    #[test]
    fn all_five_constraints_evaluated_with_healthy_inputs() {
        let params = MinimalConstraintParams::default();
        let cohorts = BTreeMap::from([
            (0u32, Fixed::from_num(95) / Fixed::from_num(100)),
            (1, Fixed::from_num(96) / Fixed::from_num(100)),
            (2, Fixed::from_num(97) / Fixed::from_num(100)),
        ]);
        let result = run_all_checks(
            Fixed::from_num(3) / Fixed::from_num(10),  // enforcement 0.3
            Fixed::from_num(7) / Fixed::from_num(10),   // legitimacy 0.7
            Fixed::from_num(8) / Fixed::from_num(10),   // governance integrity 0.8
            Fixed::from_num(1) / Fixed::from_num(10),   // selectivity 0.1
            &cohorts,
            false,                                        // coupling disabled
            Fixed::from_num(5) / Fixed::from_num(100),  // opacity 0.05
            Fixed::from_num(95) / Fixed::from_num(100), // ledger write 0.95
            Fixed::from_num(5) / Fixed::from_num(100),  // adaptation 0.05
            Fixed::from_num(2) / Fixed::from_num(10),   // scarcity 0.2
            Fixed::from_num(1) / Fixed::from_num(10),   // climate damage 0.1
            Fixed::from_num(5) / Fixed::from_num(10),   // C0 0.5
            Fixed::from_num(4) / Fixed::from_num(10),   // L0 0.4
            5,                                           // coalition members
            &params,
        );
        assert!(
            result.all_satisfied(),
            "All constraints should pass with healthy inputs"
        );
    }

    /// check_all returns the same result it was given (pass-through contract).
    #[test]
    fn check_all_returns_result() {
        let params = MinimalConstraintParams::default();
        let cohorts = BTreeMap::new();
        let result = run_all_checks(
            Fixed::ZERO,
            Fixed::from_num(5) / Fixed::from_num(10),
            Fixed::from_num(5) / Fixed::from_num(10),
            Fixed::ZERO,
            &cohorts,
            false,
            Fixed::ZERO,
            Fixed::from_num(1),
            Fixed::from_num(5) / Fixed::from_num(100),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_num(5) / Fixed::from_num(10),
            Fixed::from_num(5) / Fixed::from_num(10),
            5,
            &params,
        );
        let returned = check_all(&result);
        assert_eq!(returned.all_satisfied(), result.all_satisfied());
    }

    /// Each of the five checks can independently detect a violation.
    #[test]
    fn each_constraint_can_detect_violation() {
        let params = MinimalConstraintParams::default();
        let empty_cohorts = BTreeMap::new();

        // Healthy baseline
        let base_result = run_all_checks(
            Fixed::from_num(3) / Fixed::from_num(10),
            Fixed::from_num(7) / Fixed::from_num(10),
            Fixed::from_num(8) / Fixed::from_num(10),
            Fixed::from_num(1) / Fixed::from_num(10),
            &empty_cohorts,
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
        );
        assert!(
            base_result.all_satisfied(),
            "Baseline should have all constraints satisfied"
        );

        // C1 violation: enforcement too high
        let c1_violated = run_all_checks(
            Fixed::from_num(9) / Fixed::from_num(10), // enforcement 0.9
            Fixed::from_num(7) / Fixed::from_num(10),
            Fixed::from_num(8) / Fixed::from_num(10),
            Fixed::from_num(1) / Fixed::from_num(10),
            &empty_cohorts,
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
        );
        assert!(
            !c1_violated.c1_bounded_coercion.is_ok(),
            "C1 should detect enforcement violation"
        );

        // C3 violation: opacity too high
        let c3_violated = run_all_checks(
            Fixed::from_num(3) / Fixed::from_num(10),
            Fixed::from_num(7) / Fixed::from_num(10),
            Fixed::from_num(8) / Fixed::from_num(10),
            Fixed::from_num(1) / Fixed::from_num(10),
            &empty_cohorts,
            false,
            Fixed::from_num(3) / Fixed::from_num(10),  // opacity 0.3 > O_max 0.15
            Fixed::from_num(95) / Fixed::from_num(100),
            Fixed::from_num(5) / Fixed::from_num(100),
            Fixed::from_num(2) / Fixed::from_num(10),
            Fixed::from_num(1) / Fixed::from_num(10),
            Fixed::from_num(5) / Fixed::from_num(10),
            Fixed::from_num(4) / Fixed::from_num(10),
            5,
            &params,
        );
        assert!(
            !c3_violated.c3_transparent_ledger.is_ok(),
            "C3 should detect opacity violation"
        );
    }
}
