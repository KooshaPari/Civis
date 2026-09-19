//! Tests for FR-CIV-0104-007
//!
//! Epic: FR-CIV
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-0104-007: Baseline Stable Under Full Constraint Set
//! BASELINE_HYBRID_STABLE scenario with all five constraints active maintains
//! L > lambda_rec = 0.35 for many ticks under standard conditions.

#[cfg(test)]
mod fr_fr_civ_0104_007 {
    use civ_engine::constraints::{
        ConstraintCheck, ConstraintSetResult, ConstraintState, MinimalConstraintParams,
    };
    use civ_engine::Fixed;

    /// With all constraints satisfied, legitimacy stays above lambda_rec
    /// and no threshold-crossed events occur.
    #[test]
    fn baseline_stable_no_below_events() {
        let params = MinimalConstraintParams::default();
        let mut state = ConstraintState::default();
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };

        // Simulate 500 ticks at healthy legitimacy 0.7.
        let healthy_legitimacy = Fixed::from_num(7) / Fixed::from_num(10);
        for tick in 1..=500 {
            let snap = state.update(tick, healthy_legitimacy, &params, &result);
            assert!(
                snap.legitimacy >= params.legitimacy_recovery_threshold,
                "Legitimacy {} should stay above lambda_rec {} at tick {}",
                snap.legitimacy,
                params.legitimacy_recovery_threshold,
                tick
            );
            assert!(
                snap.all_satisfied,
                "All constraints should be satisfied at tick {}",
                tick
            );
            assert_eq!(
                snap.ticks_below_recovery_threshold, 0,
                "Should not be below recovery threshold at tick {}",
                tick
            );
        }
    }

    /// When ablation_mode triggers, it persists even though later ticks are fine.
    #[test]
    fn baseline_with_intermittent_halt_stays_ablation() {
        let params = MinimalConstraintParams::default();
        let mut state = ConstraintState::default();

        // Normal for first 10 ticks.
        let ok = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        for tick in 1..=10 {
            state.update(tick, Fixed::from_num(7) / Fixed::from_num(10), &params, &ok);
        }
        assert!(!state.ablation_mode);

        // HALT at tick 11.
        let halt = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Violated {
                severity: civ_engine::constraints::ViolationSeverity::Halt,
                reason: "test halt".to_string(),
            },
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        state.update(
            11,
            Fixed::from_num(7) / Fixed::from_num(10),
            &params,
            &halt,
        );
        assert!(state.ablation_mode);

        // Recovery for next 100 ticks.
        for tick in 12..=112 {
            let snap = state.update(tick, Fixed::from_num(7) / Fixed::from_num(10), &params, &ok);
            assert!(
                state.ablation_mode,
                "ablation_mode must stay true at tick {}",
                tick
            );
            assert!(snap.legitimacy >= params.legitimacy_recovery_threshold);
        }
    }
}
