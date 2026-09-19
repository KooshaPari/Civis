//! Tests for FR-CIV-0104-002
//!
//! Epic: FR-CIV
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-0104-002: Violation Events Emitted
//! `constraint.violated.v1` is emitted for any WARNING, CRITICAL, or HALT violation.
//! We verify that each constraint produces the correct violation severity.

#[cfg(test)]
mod fr_fr_civ_0104_002 {
    use civ_engine::constraints::{
        check_bounded_coercion, check_subsistence_floor, check_transparent_ledger,
        check_adaptive_climate_response, check_coalition_compatible_strategy,
        BoundedCoercionParams, SubsistenceFloorParams, TransparentLedgerParams,
        AdaptiveClimateParams, CoalitionStrategyParams, ConstraintCheck, ViolationSeverity,
    };
    use civ_engine::Fixed;
    use std::collections::BTreeMap;

    /// C1 violation emits CRITICAL severity when enforcement exceeds ceiling.
    #[test]
    fn c1_violation_emits_critical_severity() {
        let params = BoundedCoercionParams::default();
        let result = check_bounded_coercion(
            Fixed::from_num(9) / Fixed::from_num(10), // enforcement 0.9
            Fixed::from_num(7) / Fixed::from_num(10), // legitimacy 0.7
            Fixed::from_num(8) / Fixed::from_num(10), // governance 0.8
            Fixed::from_num(1) / Fixed::from_num(10), // selectivity 0.1
            &params,
        );
        match result {
            ConstraintCheck::Violated { severity, .. } => {
                assert_eq!(severity, ViolationSeverity::Critical);
            }
            _ => panic!("Expected C1 violation"),
        }
    }

    /// C2 violation emits HALT when coupling is enabled (structural violation).
    #[test]
    fn c2_coupling_violation_emits_halt() {
        let params = SubsistenceFloorParams::default();
        let cohorts = BTreeMap::new();
        let result = check_subsistence_floor(&cohorts, true, &params);
        match result {
            ConstraintCheck::Violated { severity, .. } => {
                assert_eq!(severity, ViolationSeverity::Halt);
            }
            _ => panic!("Expected C2 coupling violation"),
        }
    }

    /// C2 violation emits CRITICAL when cohort delivery is below floor.
    #[test]
    fn c2_low_delivery_emits_critical() {
        let params = SubsistenceFloorParams::default();
        let cohorts = BTreeMap::from([
            (0u32, Fixed::from_num(8) / Fixed::from_num(10)), // 0.8 < B_min 0.92
        ]);
        let result = check_subsistence_floor(&cohorts, false, &params);
        match result {
            ConstraintCheck::Violated { severity, .. } => {
                assert_eq!(severity, ViolationSeverity::Critical);
            }
            _ => panic!("Expected C2 delivery violation"),
        }
    }

    /// C3 violation emits CRITICAL when opacity exceeds O_max.
    #[test]
    fn c3_high_opacity_emits_critical() {
        let params = TransparentLedgerParams::default();
        let result = check_transparent_ledger(
            Fixed::from_num(3) / Fixed::from_num(10),  // 0.3 > 0.15
            Fixed::from_num(95) / Fixed::from_num(100),
            &params,
        );
        match result {
            ConstraintCheck::Violated { severity, .. } => {
                assert_eq!(severity, ViolationSeverity::Critical);
            }
            _ => panic!("Expected C3 opacity violation"),
        }
    }

    /// C3 violation emits WARNING when ledger write rate is below floor.
    #[test]
    fn c3_low_ledger_rate_emits_warning() {
        let params = TransparentLedgerParams::default();
        let result = check_transparent_ledger(
            Fixed::from_num(5) / Fixed::from_num(100),  // opacity OK
            Fixed::from_num(8) / Fixed::from_num(10),   // 0.8 < 0.92
            &params,
        );
        match result {
            ConstraintCheck::Violated { severity, .. } => {
                assert_eq!(severity, ViolationSeverity::Warning);
            }
            _ => panic!("Expected C3 ledger warning"),
        }
    }

    /// C4 violation emits HALT when climate damage exceeds CD_max.
    #[test]
    fn c4_high_damage_emits_halt() {
        let params = AdaptiveClimateParams::default();
        let result = check_adaptive_climate_response(
            Fixed::from_num(5) / Fixed::from_num(100),
            Fixed::from_num(2) / Fixed::from_num(10),
            Fixed::from_num(3) / Fixed::from_num(10),  // 0.3 > 0.25
            &params,
        );
        match result {
            ConstraintCheck::Violated { severity, .. } => {
                assert_eq!(severity, ViolationSeverity::Halt);
            }
            _ => panic!("Expected C4 damage violation"),
        }
    }

    /// C4 violation emits CRITICAL when adaptation is below floor.
    #[test]
    fn c4_low_adaptation_emits_critical() {
        let params = AdaptiveClimateParams::default();
        let result = check_adaptive_climate_response(
            Fixed::from_num(1) / Fixed::from_num(100), // adaptation too low
            Fixed::from_num(5) / Fixed::from_num(10),  // scarcity raises floor
            Fixed::from_num(1) / Fixed::from_num(10),  // damage OK
            &params,
        );
        match result {
            ConstraintCheck::Violated { severity, .. } => {
                assert_eq!(severity, ViolationSeverity::Critical);
            }
            _ => panic!("Expected C4 adaptation violation"),
        }
    }

    /// C5 violation emits CRITICAL when C0 exceeds ceiling.
    #[test]
    fn c5_high_c0_emits_critical() {
        let params = CoalitionStrategyParams::default();
        let result = check_coalition_compatible_strategy(
            Fixed::from_num(96) / Fixed::from_num(100), // C0 > 0.95
            Fixed::from_num(5) / Fixed::from_num(10),
            5,
            &params,
        );
        match result {
            ConstraintCheck::Violated { severity, .. } => {
                assert_eq!(severity, ViolationSeverity::Critical);
            }
            _ => panic!("Expected C5 coalition violation"),
        }
    }

    /// C5 violation emits WARNING when coalition member count is too low.
    #[test]
    fn c5_low_members_emits_warning() {
        let params = CoalitionStrategyParams::default();
        let result = check_coalition_compatible_strategy(
            Fixed::from_num(5) / Fixed::from_num(10),
            Fixed::from_num(5) / Fixed::from_num(10),
            1, // below min 3
            &params,
        );
        match result {
            ConstraintCheck::Violated { severity, .. } => {
                assert_eq!(severity, ViolationSeverity::Warning);
            }
            _ => panic!("Expected C5 member count warning"),
        }
    }
}
