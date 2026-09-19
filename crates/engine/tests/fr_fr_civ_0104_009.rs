//! Tests for FR-CIV-0104-009
//!
//! Epic: FR-CIV
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-0104-009: Constraint State Hash Contribution
//! ConstraintSetResult is included in state hash computation.
//! Changing one constraint result without changing other state must change the hash.

#[cfg(test)]
mod fr_fr_civ_0104_009 {
    use civ_engine::constraints::{
        constraint_hash_contribution, ConstraintCheck, ConstraintSetResult, ViolationSeverity,
    };

    /// All-OK produces a zero hash contribution.
    #[test]
    fn all_ok_produces_zero_hash() {
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        assert_eq!(
            constraint_hash_contribution(&result),
            0,
            "All-OK should produce zero hash"
        );
    }

    /// A single C1 violation changes the hash.
    #[test]
    fn c1_violation_changes_hash() {
        let ok = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        let c1_violated = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Violated {
                severity: ViolationSeverity::Critical,
                reason: "test".to_string(),
            },
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        assert_ne!(
            constraint_hash_contribution(&ok),
            constraint_hash_contribution(&c1_violated),
            "C1 violation must change hash"
        );
    }

    /// Different severity levels produce different hashes.
    #[test]
    fn different_severities_produce_different_hashes() {
        let warning = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Violated {
                severity: ViolationSeverity::Warning,
                reason: "w".to_string(),
            },
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        let critical = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Violated {
                severity: ViolationSeverity::Critical,
                reason: "c".to_string(),
            },
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        let halt = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Violated {
                severity: ViolationSeverity::Halt,
                reason: "h".to_string(),
            },
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };

        let hw = constraint_hash_contribution(&warning);
        let hc = constraint_hash_contribution(&critical);
        let hh = constraint_hash_contribution(&halt);
        assert_ne!(hw, hc, "Warning and Critical must differ");
        assert_ne!(hc, hh, "Critical and Halt must differ");
        assert_ne!(hw, hh, "Warning and Halt must differ");
    }

    /// Changing C5 (not C1) changes the hash, even when C1 is unchanged.
    #[test]
    fn changing_c5_independently_changes_hash() {
        let base = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        let c5_changed = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Ok,
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Ok,
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Violated {
                severity: ViolationSeverity::Critical,
                reason: "C5 breach".to_string(),
            },
        };
        assert_ne!(
            constraint_hash_contribution(&base),
            constraint_hash_contribution(&c5_changed),
            "C5 change must alter hash"
        );
    }

    /// Same result always hashes the same (deterministic).
    #[test]
    fn hash_is_deterministic() {
        let result = ConstraintSetResult {
            c1_bounded_coercion: ConstraintCheck::Violated {
                severity: ViolationSeverity::Critical,
                reason: "test".to_string(),
            },
            c2_subsistence_floor: ConstraintCheck::Ok,
            c3_transparent_ledger: ConstraintCheck::Violated {
                severity: ViolationSeverity::Warning,
                reason: "test2".to_string(),
            },
            c4_adaptive_climate: ConstraintCheck::Ok,
            c5_coalition_compatible: ConstraintCheck::Ok,
        };
        let h1 = constraint_hash_contribution(&result);
        let h2 = constraint_hash_contribution(&result);
        assert_eq!(h1, h2, "Hash must be deterministic");
    }
}
