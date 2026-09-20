//! Tests for FR-CIV-0104-006
//!
//! Epic: FR-CIV
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-0104-006: Ablation Suite Directional Signatures
//! Each ablation scenario produces expected directional failure signatures.

#[cfg(test)]
mod fr_fr_civ_0104_006 {
    use civ_engine::constraints::{
        run_all_checks, MinimalConstraintParams, ViolationSeverity,
    };
    use civ_engine::Fixed;
    use std::collections::BTreeMap;

    /// ABL-C1: Removing coercion ceiling (high enforcement) causes C1 violation.
    #[test]
    fn ablation_c1_high_enforcement_causes_violation() {
        let params = MinimalConstraintParams::default();
        let result = run_all_checks(
            Fixed::from_num(95) / Fixed::from_num(100), // extreme enforcement
            Fixed::from_num(3) / Fixed::from_num(10),   // low legitimacy
            Fixed::from_num(4) / Fixed::from_num(10),   // low governance
            Fixed::from_num(2) / Fixed::from_num(10),   // high selectivity
            &BTreeMap::new(),
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
            !result.c1_bounded_coercion.is_ok(),
            "ABL-C1: enforcement too high should violate C1"
        );
    }

    /// ABL-C2: Enabling coupling causes C2 HALT violation.
    #[test]
    fn ablation_c2_coupling_causes_halt() {
        let params = MinimalConstraintParams::default();
        let result = run_all_checks(
            Fixed::from_num(3) / Fixed::from_num(10),
            Fixed::from_num(7) / Fixed::from_num(10),
            Fixed::from_num(8) / Fixed::from_num(10),
            Fixed::from_num(1) / Fixed::from_num(10),
            &BTreeMap::new(),
            true, // coupling enabled — this is the ablation
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
            !result.c2_subsistence_floor.is_ok(),
            "ABL-C2: coupling enabled should violate C2"
        );
        // Verify it's a HALT (structural violation).
        match &result.c2_subsistence_floor {
            civ_engine::constraints::ConstraintCheck::Violated { severity, .. } => {
                assert_eq!(*severity, ViolationSeverity::Halt);
            }
            _ => panic!("Expected HALT for ABL-C2"),
        }
    }

    /// ABL-C3: Excessive opacity causes C3 violation.
    #[test]
    fn ablation_c3_opacity_causes_violation() {
        let params = MinimalConstraintParams::default();
        let result = run_all_checks(
            Fixed::from_num(3) / Fixed::from_num(10),
            Fixed::from_num(7) / Fixed::from_num(10),
            Fixed::from_num(8) / Fixed::from_num(10),
            Fixed::from_num(1) / Fixed::from_num(10),
            &BTreeMap::new(),
            false,
            Fixed::from_num(5) / Fixed::from_num(10),  // opacity 0.5 >> 0.15
            Fixed::from_num(5) / Fixed::from_num(10),  // poor ledger compliance
            Fixed::from_num(5) / Fixed::from_num(100),
            Fixed::from_num(2) / Fixed::from_num(10),
            Fixed::from_num(1) / Fixed::from_num(10),
            Fixed::from_num(5) / Fixed::from_num(10),
            Fixed::from_num(4) / Fixed::from_num(10),
            5,
            &params,
        );
        assert!(
            !result.c3_transparent_ledger.is_ok(),
            "ABL-C3: high opacity should violate C3"
        );
    }

    /// ABL-C4: Zero adaptation with high climate damage causes C4 HALT.
    #[test]
    fn ablation_c4_no_adaptation_causes_halt() {
        let params = MinimalConstraintParams::default();
        let result = run_all_checks(
            Fixed::from_num(3) / Fixed::from_num(10),
            Fixed::from_num(7) / Fixed::from_num(10),
            Fixed::from_num(8) / Fixed::from_num(10),
            Fixed::from_num(1) / Fixed::from_num(10),
            &BTreeMap::new(),
            false,
            Fixed::from_num(5) / Fixed::from_num(100),
            Fixed::from_num(95) / Fixed::from_num(100),
            Fixed::ZERO,                                // zero adaptation
            Fixed::from_num(5) / Fixed::from_num(10),  // high scarcity
            Fixed::from_num(3) / Fixed::from_num(10),  // high damage > 0.25
            Fixed::from_num(5) / Fixed::from_num(10),
            Fixed::from_num(4) / Fixed::from_num(10),
            5,
            &params,
        );
        assert!(
            !result.c4_adaptive_climate.is_ok(),
            "ABL-C4: no adaptation + high damage should violate C4"
        );
    }

    /// ABL-C5: Low coalition stability causes C5 violation.
    #[test]
    fn ablation_c5_unstable_coalition_causes_violation() {
        let params = MinimalConstraintParams::default();
        let result = run_all_checks(
            Fixed::from_num(3) / Fixed::from_num(10),
            Fixed::from_num(7) / Fixed::from_num(10),
            Fixed::from_num(8) / Fixed::from_num(10),
            Fixed::from_num(1) / Fixed::from_num(10),
            &BTreeMap::new(),
            false,
            Fixed::from_num(5) / Fixed::from_num(100),
            Fixed::from_num(95) / Fixed::from_num(100),
            Fixed::from_num(5) / Fixed::from_num(100),
            Fixed::from_num(2) / Fixed::from_num(10),
            Fixed::from_num(1) / Fixed::from_num(10),
            Fixed::from_num(99) / Fixed::from_num(100), // C0 near 1.0
            Fixed::from_num(98) / Fixed::from_num(100), // L0 near 1.0
            5,
            &params,
        );
        assert!(
            !result.c5_coalition_compatible.is_ok(),
            "ABL-C5: unstable coalition should violate C5"
        );
    }
}
