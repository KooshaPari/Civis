//! Tests for FR-CIV-0104-011
//!
//! Epic: FR-CIV
//! FR-CIV-0104-011: ConstraintSetResult Self-Consistency
//!
//! `ConstraintSetResult` must satisfy the internal relation
//! `all_satisfied == results.iter().all(|r| r.is_ok())`, and `most_severe`
//! must equal the highest-severity member of `results`. We verify this via
//! `verify_self_consistency()` for healthy, mixed-severity, and all-Ok cases.

#[cfg(test)]
mod fr_fr_civ_0104_011 {
    use civ_engine::constraints::{
        run_all_checks, ConstraintCheck, ConstraintSetResult, MinimalConstraintParams,
        ViolationSeverity,
    };
    use civ_engine::Fixed;
    use std::collections::BTreeMap;

    /// Healthy inputs → all-satisfied + most_severe == None → consistent.
    #[test]
    fn healthy_inputs_are_self_consistent() {
        let params = MinimalConstraintParams::default();
        let cohorts = BTreeMap::from([
            (0u32, Fixed::from_num(95) / Fixed::from_num(100)),
            (1, Fixed::from_num(96) / Fixed::from_num(100)),
        ]);
        let result = run_all_checks(
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
        );
        assert!(result.all_satisfied());
        assert_eq!(result.most_severe(), None);
        assert!(
            result.verify_self_consistency(),
            "Healthy inputs must be self-consistent"
        );
    }

    /// Mixed severity: at least one violation; highest-severity must match
    /// `most_severe()`.
    #[test]
    fn mixed_severity_is_self_consistent() {
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Violated {
                severity: ViolationSeverity::Warning,
                reason: "c1".into(),
            },
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Violated {
                severity: ViolationSeverity::Critical,
                reason: "c3".into(),
            },
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Violated {
                severity: ViolationSeverity::Halt,
                reason: "c5".into(),
            },
        };
        assert!(!result.all_satisfied());
        assert_eq!(result.most_severe(), Some(ViolationSeverity::Halt));
        assert!(
            result.verify_self_consistency(),
            "Mixed-severity result must be self-consistent"
        );
    }

    /// Single Warning violation: most_severe == Warning; not all_satisfied.
    #[test]
    fn single_warning_is_self_consistent() {
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Violated {
                severity: ViolationSeverity::Warning,
                reason: "c4 warn".into(),
            },
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        assert!(!result.all_satisfied());
        assert_eq!(result.most_severe(), Some(ViolationSeverity::Warning));
        assert!(result.verify_self_consistency());
    }

    /// iter_checks length is exactly 5 (sanity for the iterator invariant).
    #[test]
    fn iter_checks_length_is_five() {
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        assert_eq!(result.iter_checks().count(), 5);
        assert!(result.verify_self_consistency());
    }
}