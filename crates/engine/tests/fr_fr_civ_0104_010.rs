//! Tests for FR-CIV-0104-010
//!
//! Epic: FR-CIV
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-0104-010: Recovery Window Tracking
//! ticks_below_recovery_threshold increments when L < lambda_rec
//! and resets when L recovers above lambda_rec.

#[cfg(test)]
mod fr_fr_civ_0104_010 {
    use civ_engine::constraints::{
        ConstraintCheck, ConstraintSetResult, ConstraintState, MinimalConstraintParams,
    };
    use civ_engine::Fixed;

    /// Counter increments when legitimacy is below lambda_rec.
    #[test]
    fn counter_increments_below_lambda_rec() {
        let params = MinimalConstraintParams::default();
        let mut state = ConstraintState::default();
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };

        let low_legitimacy = Fixed::from_num(1) / Fixed::from_num(10); // 0.1 < 0.35
        for tick in 1..=10 {
            let snap = state.update(tick, low_legitimacy, &params, &result);
            assert_eq!(
                snap.ticks_below_recovery_threshold, tick,
                "Counter should increment at tick {}",
                tick
            );
        }
    }

    /// Counter resets to 0 when legitimacy recovers above lambda_rec.
    #[test]
    fn counter_resets_on_recovery() {
        let params = MinimalConstraintParams::default();
        let mut state = ConstraintState::default();
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };

        let low_legitimacy = Fixed::from_num(1) / Fixed::from_num(10); // 0.1
        let high_legitimacy = Fixed::from_num(7) / Fixed::from_num(10); // 0.7

        // Drive L below lambda_rec for 10 ticks.
        for tick in 1..=10 {
            state.update(tick, low_legitimacy, &params, &result);
        }
        assert_eq!(state.ticks_below_recovery_threshold, 10);

        // Recover above lambda_rec.
        let snap = state.update(11, high_legitimacy, &params, &result);
        assert_eq!(
            snap.ticks_below_recovery_threshold, 0,
            "Counter should reset on recovery"
        );
        assert_eq!(state.ticks_below_recovery_threshold, 0);
    }

    /// Full scenario: below for 10 ticks, recovery, below again for 5, recovery.
    #[test]
    fn full_cycle_below_and_above() {
        let params = MinimalConstraintParams::default();
        let mut state = ConstraintState::default();
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };

        let low = Fixed::from_num(1) / Fixed::from_num(10);
        let high = Fixed::from_num(7) / Fixed::from_num(10);
        let mut tick = 0u64;

        // Phase 1: below for 10 ticks.
        for _ in 0..10 {
            tick += 1;
            state.update(tick, low, &params, &result);
        }
        assert_eq!(state.ticks_below_recovery_threshold, 10);

        // Phase 2: recovery (1 tick).
        tick += 1;
        state.update(tick, high, &params, &result);
        assert_eq!(state.ticks_below_recovery_threshold, 0);

        // Phase 3: below for 5 ticks.
        for _ in 0..5 {
            tick += 1;
            let snap = state.update(tick, low, &params, &result);
            assert!(
                snap.ticks_below_recovery_threshold > 0,
                "Should be below threshold at tick {}",
                tick
            );
        }
        assert_eq!(state.ticks_below_recovery_threshold, 5);

        // Phase 4: recovery again.
        tick += 1;
        let snap = state.update(tick, high, &params, &result);
        assert_eq!(snap.ticks_below_recovery_threshold, 0);
    }

    /// Legitimacy exactly at lambda_rec is NOT below (>= counts as recovery).
    #[test]
    fn exactly_at_lambda_rec_is_not_below() {
        let params = MinimalConstraintParams::default();
        let mut state = ConstraintState::default();
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };

        // First go below.
        let low = Fixed::from_num(1) / Fixed::from_num(10);
        state.update(1, low, &params, &result);
        assert_eq!(state.ticks_below_recovery_threshold, 1);

        // Then exactly at lambda_rec (0.35).
        let at_threshold = Fixed::from_num(35) / Fixed::from_num(100);
        let snap = state.update(2, at_threshold, &params, &result);
        assert_eq!(
            snap.ticks_below_recovery_threshold, 0,
            "At lambda_rec should count as recovery"
        );
    }
}
