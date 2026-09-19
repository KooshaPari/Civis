//! Tests for FR-CIV-0104-004
//!
//! Epic: FR-CIV
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-0104-004: StabilityMetrics Written Each Tick
//! One stability_snapshots row written per tick; append-only.

#[cfg(test)]
mod fr_fr_civ_0104_004 {
    use civ_engine::constraints::{
        ConstraintCheck, ConstraintSetResult, ConstraintState, MinimalConstraintParams,
    };
    use civ_engine::Fixed;

    /// Running 100 ticks produces exactly 100 snapshots.
    #[test]
    fn hundred_ticks_produces_hundred_snapshots() {
        let params = MinimalConstraintParams::default();
        let mut state = ConstraintState::default();
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };

        for tick in 1..=100 {
            state.update(tick, Fixed::from_num(7) / Fixed::from_num(10), &params, &result);
        }

        assert_eq!(
            state.stability_snapshots.len(),
            100,
            "Should have exactly 100 snapshots"
        );
    }

    /// Snapshots are append-only (each new snapshot is at the end).
    #[test]
    fn snapshots_are_append_only() {
        let params = MinimalConstraintParams::default();
        let mut state = ConstraintState::default();
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };

        for tick in 1..=10 {
            let snap = state.update(tick, Fixed::from_num(7) / Fixed::from_num(10), &params, &result);
            assert_eq!(snap.tick, tick);
        }

        // Verify ticks are monotonically increasing.
        for (i, snap) in state.stability_snapshots.iter().enumerate() {
            assert_eq!(snap.tick, (i as u64) + 1);
        }
    }

    /// Each snapshot records the correct tick number.
    #[test]
    fn snapshot_records_correct_tick() {
        let params = MinimalConstraintParams::default();
        let mut state = ConstraintState::default();
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };

        let snap = state.update(42, Fixed::from_num(7) / Fixed::from_num(10), &params, &result);
        assert_eq!(snap.tick, 42);
        assert_eq!(state.stability_snapshots[0].tick, 42);
    }

    /// Default state starts with empty snapshots.
    #[test]
    fn default_state_has_no_snapshots() {
        let state = ConstraintState::default();
        assert!(state.stability_snapshots.is_empty());
    }
}
