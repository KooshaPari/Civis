//! CIV-0104 Minimal Constraint Set — constitutional rails enforced each tick.
//!
//! The five constraints are not policy levers; they are hardcoded invariant
//! checks. Violations emit structured events. See
//! `docs/specs/CIV-0104-minimal-constraint-set-theorem.md` for the full
//! theorem statement and proof.

use crate::Fixed;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

/// Severity level of a constraint violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Non-fatal; logged but simulation continues.
    Warning,
    /// Degraded operation; simulation continues with correction signals.
    Critical,
    /// Hard failure; sets ablation_mode permanently.
    Halt,
}

// ---------------------------------------------------------------------------
// Individual constraint check result
// ---------------------------------------------------------------------------

/// Result of checking a single constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintCheck {
    /// Constraint satisfied.
    Ok,
    /// Constraint violated at the given severity with a human-readable reason.
    Violated {
        /// Severity of the violation.
        severity: ViolationSeverity,
        /// Human-readable description.
        reason: String,
    },
}

impl ConstraintCheck {
    /// Returns `true` if the check passed.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, ConstraintCheck::Ok)
    }
}

// ---------------------------------------------------------------------------
// Aggregate result
// ---------------------------------------------------------------------------

/// Aggregate result of running all five constraint checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstraintSetResult {
    /// Individual check for C1: Bounded Coercion.
    pub c1_bounded_coercion: ConstraintCheck,
    /// Individual check for C2: Subsistence Floor.
    pub c2_subsistence_floor: ConstraintCheck,
    /// Individual check for C3: Transparent Transfer Ledger.
    pub c3_transparent_ledger: ConstraintCheck,
    /// Individual check for C4: Adaptive Climate Response.
    pub c4_adaptive_climate: ConstraintCheck,
    /// Individual check for C5: Coalition-Compatible Strategy.
    pub c5_coalition_compatible: ConstraintCheck,
}

impl ConstraintSetResult {
    /// `true` when every individual check is `Ok`.
    #[must_use]
    pub fn all_satisfied(&self) -> bool {
        self.c1_bounded_coercion.is_ok()
            && self.c2_subsistence_floor.is_ok()
            && self.c3_transparent_ledger.is_ok()
            && self.c4_adaptive_climate.is_ok()
            && self.c5_coalition_compatible.is_ok()
    }

    /// The most severe violation across all five checks, or `None` if all OK.
    #[must_use]
    pub fn most_severe(&self) -> Option<ViolationSeverity> {
        let checks = [
            &self.c1_bounded_coercion,
            &self.c2_subsistence_floor,
            &self.c3_transparent_ledger,
            &self.c4_adaptive_climate,
            &self.c5_coalition_compatible,
        ];
        checks
            .iter()
            .filter_map(|c| match c {
                ConstraintCheck::Violated { severity, .. } => Some(*severity),
                ConstraintCheck::Ok => None,
            })
            .max()
    }

    /// Returns an iterator over all five individual results.
    #[must_use]
    pub fn iter_checks(&self) -> impl Iterator<Item = &ConstraintCheck> {
        [
            &self.c1_bounded_coercion,
            &self.c2_subsistence_floor,
            &self.c3_transparent_ledger,
            &self.c4_adaptive_climate,
            &self.c5_coalition_compatible,
        ]
        .into_iter()
    }
}

// ---------------------------------------------------------------------------
// Parameters (immutable — FR-CIV-0104-008)
// ---------------------------------------------------------------------------

/// Parameters for the C1: Bounded Coercion check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundedCoercionParams {
    /// Maximum enforcement under ideal governance. Valid range [0.3, 0.8]; default 0.6.
    pub e_base: Fixed,
    /// Legitimacy sensitivity of ceiling. Valid range [2.0, 8.0]; default 4.0.
    pub kappa_l: Fixed,
    /// Maximum tolerable selectivity. Valid range [0.0, 0.3]; default 0.2.
    pub sel_max: Fixed,
}

impl Default for BoundedCoercionParams {
    fn default() -> Self {
        Self {
            e_base: Fixed::from_num(6) / Fixed::from_num(10),
            kappa_l: Fixed::from_num(4),
            sel_max: Fixed::from_num(2) / Fixed::from_num(10),
        }
    }
}

/// Parameters for the C2: Subsistence Floor check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubsistenceFloorParams {
    /// Minimum fraction of population receiving subsistence. Valid [0.85, 1.0]; default 0.92.
    pub b_min: Fixed,
    /// Maximum scarcity under which the floor is still computed. Valid [0.5, 0.9]; default 0.75.
    pub s_max: Fixed,
}

impl Default for SubsistenceFloorParams {
    fn default() -> Self {
        Self {
            b_min: Fixed::from_num(92) / Fixed::from_num(100),
            s_max: Fixed::from_num(75) / Fixed::from_num(100),
        }
    }
}

/// Parameters for the C3: Transparent Transfer Ledger check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransparentLedgerParams {
    /// Maximum opacity fraction. Valid [0.0, 0.20]; default 0.15.
    pub o_max: Fixed,
    /// Minimum fraction of transfers that must be logged. Valid [0.85, 1.0]; default 0.92.
    pub ledger_completeness_floor: Fixed,
}

impl Default for TransparentLedgerParams {
    fn default() -> Self {
        Self {
            o_max: Fixed::from_num(15) / Fixed::from_num(100),
            ledger_completeness_floor: Fixed::from_num(92) / Fixed::from_num(100),
        }
    }
}

/// Parameters for the C4: Adaptive Climate Response check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveClimateParams {
    /// Minimum adaptation investment at zero scarcity. Valid [0.02, 0.10]; default 0.04.
    pub a_min_base: Fixed,
    /// Scarcity coefficient. Valid [0.01, 0.05]; default 0.025.
    pub a_scarcity_coefficient: Fixed,
    /// Maximum climate damage fraction. Valid [0.15, 0.40]; default 0.25.
    pub cd_max: Fixed,
}

impl Default for AdaptiveClimateParams {
    fn default() -> Self {
        Self {
            a_min_base: Fixed::from_num(4) / Fixed::from_num(100),
            a_scarcity_coefficient: Fixed::from_num(25) / Fixed::from_num(1000),
            cd_max: Fixed::from_num(25) / Fixed::from_num(100),
        }
    }
}

/// Parameters for the C5: Coalition-Compatible Strategy check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoalitionStrategyParams {
    /// C0 ceiling; must be < 1.0.
    pub c0_ceiling: Fixed,
    /// L0 ceiling; must be < 1.0.
    pub l0_ceiling: Fixed,
    /// Minimum coalition member count for meaningful C0. Valid [2, 10]; default 3.
    pub coalition_min_members: u32,
}

impl Default for CoalitionStrategyParams {
    fn default() -> Self {
        Self {
            c0_ceiling: Fixed::from_num(95) / Fixed::from_num(100),
            l0_ceiling: Fixed::from_num(95) / Fixed::from_num(100),
            coalition_min_members: 3,
        }
    }
}

/// Top-level immutable parameter bundle (FR-CIV-0104-008: cannot be modified at runtime).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinimalConstraintParams {
    /// C1 parameters.
    pub c1: BoundedCoercionParams,
    /// C2 parameters.
    pub c2: SubsistenceFloorParams,
    /// C3 parameters.
    pub c3: TransparentLedgerParams,
    /// C4 parameters.
    pub c4: AdaptiveClimateParams,
    /// C5 parameters.
    pub c5: CoalitionStrategyParams,
    /// Legitimacy recovery threshold lambda_rec. Default 0.35.
    pub legitimacy_recovery_threshold: Fixed,
    /// Legitimacy floor L_min. Default 0.20.
    pub legitimacy_floor: Fixed,
    /// Recovery window (ticks below lambda_rec before exponential decay). Default 50.
    pub recovery_window: u64,
}

impl Default for MinimalConstraintParams {
    fn default() -> Self {
        Self {
            c1: BoundedCoercionParams::default(),
            c2: SubsistenceFloorParams::default(),
            c3: TransparentLedgerParams::default(),
            c4: AdaptiveClimateParams::default(),
            c5: CoalitionStrategyParams::default(),
            legitimacy_recovery_threshold: Fixed::from_num(35) / Fixed::from_num(100),
            legitimacy_floor: Fixed::from_num(20) / Fixed::from_num(100),
            recovery_window: 50,
        }
    }
}

// ---------------------------------------------------------------------------
// Stability metrics snapshot
// ---------------------------------------------------------------------------

/// Snapshot of stability metrics at a given tick (FR-CIV-0104-004).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StabilityMetrics {
    /// Current tick number.
    pub tick: u64,
    /// Current legitimacy value [0, 1] in fixed-point.
    pub legitimacy: Fixed,
    /// Legitimacy recovery threshold lambda_rec.
    pub legitimacy_recovery_threshold: Fixed,
    /// Legitimacy floor L_min.
    pub legitimacy_floor: Fixed,
    /// Number of consecutive ticks below lambda_rec.
    pub ticks_below_recovery_threshold: u64,
    /// Whether ablation mode is active.
    pub ablation_mode: bool,
    /// All-satisfied flag from this tick's check.
    pub all_satisfied: bool,
}

// ---------------------------------------------------------------------------
// Constraint check functions
// ---------------------------------------------------------------------------

/// Check C1: Bounded Coercion.
///
/// Returns `Ok(())` if enforcement intensity is below the computable ceiling.
#[must_use]
pub fn check_bounded_coercion(
    enforcement_intensity: Fixed,
    legitimacy: Fixed,
    governance_integrity: Fixed,
    selectivity: Fixed,
    params: &BoundedCoercionParams,
) -> ConstraintCheck {
    // Sigmoid damping: sigma_L(L) = 1 / (1 + exp(-kappa_l * (L - lambda_rec)))
    // Simplified: when L is high, sigma_L ≈ 1; when L is low, sigma_L drops.
    let lambda_rec = Fixed::from_num(35) / Fixed::from_num(100);
    let diff = legitimacy - lambda_rec;
    // Approximate sigmoid with a clamped linear ramp for deterministic behavior:
    // sigma = clamp(0.5 + kappa_l * diff / 4, 0, 1)
    let sigma = {
        let raw = Fixed::from_num(1) / Fixed::from_num(2)
            + params.kappa_l * diff / Fixed::from_num(4);
        if raw < Fixed::ZERO {
            Fixed::ZERO
        } else if raw > Fixed::from_num(1) {
            Fixed::from_num(1)
        } else {
            raw
        }
    };

    // Ceiling: E* = E_base * G * (1 - Sel) * sigma_L(L)
    let one = Fixed::from_num(1);
    let ceiling = params.e_base * governance_integrity * (one - selectivity) * sigma;

    if enforcement_intensity > ceiling {
        ConstraintCheck::Violated {
            severity: ViolationSeverity::Critical,
            reason: format!(
                "Enforcement intensity {} exceeds ceiling {} (E_base={}, G={}, Sel={}, sigma={})",
                enforcement_intensity, ceiling, params.e_base, governance_integrity, selectivity, sigma
            ),
        }
    } else {
        ConstraintCheck::Ok
    }
}

/// Check C2: Subsistence Floor and Coupling Lock.
///
/// Verifies all cohorts receive essentials above B_min, and coupling is disabled.
#[must_use]
pub fn check_subsistence_floor(
    cohort_delivery_rates: &BTreeMap<u32, Fixed>,
    coupling_enabled: bool,
    params: &SubsistenceFloorParams,
) -> ConstraintCheck {
    // Coupling lock: coupling must always be false.
    if coupling_enabled {
        return ConstraintCheck::Violated {
            severity: ViolationSeverity::Halt,
            reason: "Coupling lock violated: score-based denial of essentials is forbidden"
                .to_string(),
        };
    }

    // Check each cohort delivery rate >= B_min.
    for (cohort_id, &rate) in cohort_delivery_rates {
        if rate < params.b_min {
            return ConstraintCheck::Violated {
                severity: ViolationSeverity::Critical,
                reason: format!(
                    "Cohort {} delivery rate {} is below subsistence floor {}",
                    cohort_id, rate, params.b_min
                ),
            };
        }
    }

    ConstraintCheck::Ok
}

/// Check C3: Transparent Transfer Ledger.
///
/// Verifies opacity is below ceiling and ledger completeness above floor.
#[must_use]
pub fn check_transparent_ledger(
    opacity: Fixed,
    ledger_write_rate: Fixed,
    params: &TransparentLedgerParams,
) -> ConstraintCheck {
    if opacity > params.o_max {
        return ConstraintCheck::Violated {
            severity: ViolationSeverity::Critical,
            reason: format!(
                "Opacity {} exceeds maximum {} (shadow capture R0 risk)",
                opacity, params.o_max
            ),
        };
    }

    if ledger_write_rate < params.ledger_completeness_floor {
        return ConstraintCheck::Violated {
            severity: ViolationSeverity::Warning,
            reason: format!(
                "Ledger write rate {} below completeness floor {}",
                ledger_write_rate, params.ledger_completeness_floor
            ),
        };
    }

    ConstraintCheck::Ok
}

/// Check C4: Adaptive Climate Response.
///
/// Verifies adaptation investment meets scarcity-adjusted floor.
#[must_use]
pub fn check_adaptive_climate_response(
    adaptation_investment: Fixed,
    scarcity_pressure: Fixed,
    climate_damage: Fixed,
    params: &AdaptiveClimateParams,
) -> ConstraintCheck {
    if climate_damage > params.cd_max {
        return ConstraintCheck::Violated {
            severity: ViolationSeverity::Halt,
            reason: format!(
                "Climate damage {} exceeds maximum {} (C2 infeasibility threshold)",
                climate_damage, params.cd_max
            ),
        };
    }

    // A_min = A_min_base + A_scarcity_coefficient * S
    let a_min = params.a_min_base + params.a_scarcity_coefficient * scarcity_pressure;

    if adaptation_investment < a_min {
        return ConstraintCheck::Violated {
            severity: ViolationSeverity::Critical,
            reason: format!(
                "Adaptation investment {} below minimum {} (S={})",
                adaptation_investment, a_min, scarcity_pressure
            ),
        };
    }

    ConstraintCheck::Ok
}

/// Check C5: Coalition-Compatible External Strategy.
///
/// Verifies coalition stability number C0 < 1 and leakage number L0 < 1.
#[must_use]
pub fn check_coalition_compatible_strategy(
    coalition_stability_number: Fixed,
    leakage_reproduction_number: Fixed,
    coalition_member_count: u32,
    params: &CoalitionStrategyParams,
) -> ConstraintCheck {
    if coalition_member_count < params.coalition_min_members {
        return ConstraintCheck::Violated {
            severity: ViolationSeverity::Warning,
            reason: format!(
                "Coalition member count {} below minimum {}",
                coalition_member_count, params.coalition_min_members
            ),
        };
    }

    if coalition_stability_number > params.c0_ceiling {
        return ConstraintCheck::Violated {
            severity: ViolationSeverity::Critical,
            reason: format!(
                "Coalition stability C0={} exceeds ceiling {}",
                coalition_stability_number, params.c0_ceiling
            ),
        };
    }

    if leakage_reproduction_number > params.l0_ceiling {
        return ConstraintCheck::Violated {
            severity: ViolationSeverity::Critical,
            reason: format!(
                "Leakage reproduction L0={} exceeds ceiling {}",
                leakage_reproduction_number, params.l0_ceiling
            ),
        };
    }

    ConstraintCheck::Ok
}

// ---------------------------------------------------------------------------
// Aggregated check_all
// ---------------------------------------------------------------------------

// FR-CIV-0104-001
/// Run all five constraint checks and return the aggregate result.
#[must_use]
pub fn check_all(result: &ConstraintSetResult) -> &ConstraintSetResult {
    // The result is pre-computed; this function is the entry point that
    // Phase 2 calls. We return the result by reference for inspection.
    result
}

/// Construct a `ConstraintSetResult` by running all five checks.
#[must_use]
pub fn run_all_checks(
    enforcement_intensity: Fixed,
    legitimacy: Fixed,
    governance_integrity: Fixed,
    selectivity: Fixed,
    cohort_delivery_rates: &BTreeMap<u32, Fixed>,
    coupling_enabled: bool,
    opacity: Fixed,
    ledger_write_rate: Fixed,
    adaptation_investment: Fixed,
    scarcity_pressure: Fixed,
    climate_damage: Fixed,
    coalition_stability_number: Fixed,
    leakage_reproduction_number: Fixed,
    coalition_member_count: u32,
    params: &MinimalConstraintParams,
) -> ConstraintSetResult {
    ConstraintSetResult {
        c1_bounded_coercion: check_bounded_coercion(
            enforcement_intensity,
            legitimacy,
            governance_integrity,
            selectivity,
            &params.c1,
        ),
        c2_subsistence_floor: check_subsistence_floor(
            cohort_delivery_rates,
            coupling_enabled,
            &params.c2,
        ),
        c3_transparent_ledger: check_transparent_ledger(opacity, ledger_write_rate, &params.c3),
        c4_adaptive_climate: check_adaptive_climate_response(
            adaptation_investment,
            scarcity_pressure,
            climate_damage,
            &params.c4,
        ),
        c5_coalition_compatible: check_coalition_compatible_strategy(
            coalition_stability_number,
            leakage_reproduction_number,
            coalition_member_count,
            &params.c5,
        ),
    }
}

/// Per-tick constraint state tracked alongside the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintState {
    /// Whether a HALT violation has been observed (permanent once set).
    pub ablation_mode: bool,
    /// Number of consecutive ticks where legitimacy < lambda_rec.
    pub ticks_below_recovery_threshold: u64,
    /// Append-only log of per-tick stability snapshots.
    pub stability_snapshots: Vec<StabilityMetrics>,
}

impl Default for ConstraintState {
    fn default() -> Self {
        Self {
            ablation_mode: false,
            ticks_below_recovery_threshold: 0,
            stability_snapshots: Vec::new(),
        }
    }
}

impl ConstraintState {
    /// Update recovery tracking after a check. Returns the new snapshot.
    pub fn update(
        &mut self,
        tick: u64,
        legitimacy: Fixed,
        params: &MinimalConstraintParams,
        result: &ConstraintSetResult,
    ) -> StabilityMetrics {
        // If any check returned Halt, set ablation_mode permanently.
        if !self.ablation_mode {
            self.ablation_mode = result.iter_checks().any(|c| {
                matches!(
                    c,
                    ConstraintCheck::Violated {
                        severity: ViolationSeverity::Halt,
                        ..
                    }
                )
            });
        }

        // Update recovery window counter.
        if legitimacy < params.legitimacy_recovery_threshold {
            self.ticks_below_recovery_threshold += 1;
        } else {
            self.ticks_below_recovery_threshold = 0;
        }

        let snapshot = StabilityMetrics {
            tick,
            legitimacy,
            legitimacy_recovery_threshold: params.legitimacy_recovery_threshold,
            legitimacy_floor: params.legitimacy_floor,
            ticks_below_recovery_threshold: self.ticks_below_recovery_threshold,
            ablation_mode: self.ablation_mode,
            all_satisfied: result.all_satisfied(),
        };

        self.stability_snapshots.push(snapshot.clone());
        snapshot
    }
}

/// Compute the state hash contribution from a `ConstraintSetResult`.
///
/// Each violated constraint contributes its severity ordinal to the hash.
#[must_use]
pub fn constraint_hash_contribution(result: &ConstraintSetResult) -> u64 {
    let mut hash: u64 = 0;
    for (i, check) in result.iter_checks().enumerate() {
        let severity_val = match check {
            ConstraintCheck::Ok => 0u64,
            ConstraintCheck::Violated { severity, .. } => match severity {
                ViolationSeverity::Warning => 1,
                ViolationSeverity::Critical => 2,
                ViolationSeverity::Halt => 3,
            },
        };
        // Simple deterministic combination: shift by position and XOR.
        hash ^= severity_val << (i * 2);
    }
    hash
}
