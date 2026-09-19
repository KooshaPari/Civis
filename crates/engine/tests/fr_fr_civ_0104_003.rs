//! Tests for FR-CIV-0104-003
//!
//! Epic: FR-CIV
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-0104-003: ABLATION_MODE Flag Propagates
//! Any HALT violation sets ablation_mode = true on the run permanently.

#[cfg(test)]
mod fr_fr_civ_0104_003 {
    use civ_engine::constraints::{
        ConstraintState, ConstraintSetResult, ConstraintCheck, MinimalConstraintParams,
        ViolationSeverity,
    };
    use civ_engine::Fixed;

    /// After a HALT violation, ablation_mode stays true even when subsequent
    /// checks pass.
    #[test]
    fn halt_sets_ablation_mode_permanently() {
        let params = MinimalConstraintParams::default();
        let mut state = ConstraintState::default();
        assert!(!state.ablation_mode, "Initially not in ablation mode");

        // First tick: HALT violation (coupling enabled).
        let halt_result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Violated {
                severity: ViolationSeverity::Halt,
                reason: "Coupling lock violated".to_string(),
            },
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        state.update(1, Fixed::from_num(7) / Fixed::from_num(10), &params, &halt_result);
        assert!(
            state.ablation_mode,
            "ablation_mode should be true after HALT"
        );

        // Second tick: all checks pass.
        let ok_result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        state.update(2, Fixed::from_num(7) / Fixed::from_num(10), &params, &ok_result);
        assert!(
            state.ablation_mode,
            "ablation_mode should remain true even after recovery"
        );
    }

    /// Warning and Critical violations do NOT set ablation_mode.
    #[test]
    fn non_halt_violations_do_not_set_ablation() {
        let params = MinimalConstraintParams::default();
        let mut state = ConstraintState::default();

        let critical_result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Violated {
                severity: ViolationSeverity::Critical,
                reason: "test".to_string(),
            },
        };
        state.update(1, Fixed::from_num(7) / Fixed::from_num(10), &params, &critical_result);
        assert!(
            !state.ablation_mode,
            "Critical should not set ablation_mode"
        );

        let warning_result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Violated {
                severity: ViolationSeverity::Warning,
                reason: "test".to_string(),
            },
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        state.update(2, Fixed::from_num(7) / Fixed::from_num(10), &params, &warning_result);
        assert!(
            !state.ablation_mode,
            "Warning should not set ablation_mode"
        );
    }

    /// Multiple HALT violations keep ablation_mode true (idempotent).
    #[test]
    fn multiple_halts_keep_ablation_mode() {
        let params = MinimalConstraintParams::default();
        let mut state = ConstraintState::default();

        for tick in 1..=5 {
            let result = ConstraintSetResult {
                c1_bounded_coercion: ConstraintCheck::Ok,
                c2_subsistence_floor: ConstraintCheck::Violated {
                    severity: ViolationSeverity::Halt,
                    reason: format!("HALT at tick {}", tick),
                },
                c3_transparent_ledger: ConstraintCheck::Ok,
                c4_adaptive_climate: ConstraintCheck::Ok,
                c5_coalition_compatible: ConstraintCheck::Ok,
            };
            state.update(tick, Fixed::from_num(7) / Fixed::from_num(10), &params, &result);
            assert!(state.ablation_mode, "Should be true at tick {}", tick);
        }
    }
}
