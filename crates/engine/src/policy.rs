//! Policy module — consumption calculations and the `Policy` trait (FR-CORE-005).
//!
//! Two related but distinct concepts live here:
//!
//! 1. **Scenario economy policy** ([`PolicyInput`], [`effective_consumption`]):
//!    per-tick joule drain knob. Drained in `phase_economy`.
//! 2. **Control policy** ([`Policy`], [`ControlSignals`], [`policy_from_kind`]):
//!    high-level policy kind installed on a [`Simulation`] via
//!    [`crate::engine::Simulation::set_policy`]. Read inside
//!    [`crate::engine::Simulation::phase_policy`] each tick, immediately before
//!    `phase_economy` (FR-CORE-005). The result of `Policy::evaluate` is
//!    exposed on the simulation as
//!    [`crate::engine::Simulation::last_control_signals`].

use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

/// Defaults matching `scenarios/baseline.yaml`.
pub const DEFAULT_ECONOMY_POLICY: PolicyInput = PolicyInput {
    base_consumption_joules: 5_000_000_000.0,
    scarcity_multiplier: 1.0,
};

#[derive(Debug, Clone, Copy)]
pub struct PolicyInput {
    pub base_consumption_joules: f64,
    pub scarcity_multiplier: f64,
}

pub fn effective_consumption(input: PolicyInput) -> f64 {
    input.base_consumption_joules * input.scarcity_multiplier.max(0.0)
}

// ============================================================================
// FR-CORE-005 — Policy trait + ControlSignals
// ============================================================================

/// Per-tick control signal output of a [`Policy`].
///
/// Default is the empty signal — every map is empty — which means a policy
/// that returns `ControlSignals::default()` is observationally a no-op.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ControlSignals {
    /// Per-resource production multipliers (resource_name -> multiplier).
    /// Empty for the default / no-op policy.
    pub production_multipliers: BTreeMap<String, f64>,
    /// Per-resource allocation weights (resource_name -> weight).
    /// Empty for the default / no-op policy.
    pub allocation_weights: BTreeMap<String, f64>,
    /// Per-institution tax rate in basis points (institution_id -> bps).
    /// Empty for the default / no-op policy.
    pub tax_rates: BTreeMap<u32, u32>,
}

/// High-level control policy. Reads `&WorldState` and emits
/// [`ControlSignals`] each tick. The default `evaluate` is a no-op.
///
/// We pass `&WorldState` (engine state) rather than `&Simulation` so policies
/// cannot reach into the ECS world, RNG, or voxel substrate — only the
/// deterministic scalar state. This keeps policies pure functions of state and
/// preserves the replay-determinism guarantee that FR-CORE-005 inherits from
/// CIV-0001.
pub trait Policy: std::fmt::Debug + Send + Sync {
    /// Compute the control signals for `state` at the current tick.
    ///
    /// Default impl returns [`ControlSignals::default()`] — policies that
    /// only need to identify themselves can rely on the default.
    fn evaluate(&self, _state: &crate::engine::WorldState) -> ControlSignals {
        ControlSignals::default()
    }

    /// Stable name for this policy kind. Used in scenario YAML, replay logs,
    /// and `Debug` formatting. The default is `"noop"`.
    fn name(&self) -> &'static str {
        "noop"
    }
}

/// No-op policy — `evaluate` returns the default [`ControlSignals`].
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopPolicy;

impl Policy for NoopPolicy {}

/// Capitalist policy — currently observationally a no-op (just identifies
/// itself). Future revisions will bias production_multipliers and
/// allocation_weights toward market-rate goods. The trait is in place first
/// so scenario YAML can already request this kind.
#[derive(Debug, Clone, Copy, Default)]
pub struct CapitalistPolicy;

impl Policy for CapitalistPolicy {
    fn name(&self) -> &'static str {
        "capitalist"
    }
}

/// Subsistence-first policy — currently observationally a no-op (just
/// identifies itself). Future revisions will bias allocation_weights toward
/// food/energy and away from luxury goods.
#[derive(Debug, Clone, Copy, Default)]
pub struct SubsistenceFirstPolicy;

impl Policy for SubsistenceFirstPolicy {
    fn name(&self) -> &'static str {
        "subsistence_first"
    }
}

/// Map a scenario-YAML `kind` string to a concrete [`Policy`]. Unknown kinds
/// fall back to [`NoopPolicy`] (defensive: we never fail a scenario load on a
/// typo in the policy field).
pub fn policy_from_kind(kind: &str) -> Box<dyn Policy> {
    match kind {
        "capitalist" => Box::new(CapitalistPolicy),
        "subsistence_first" => Box::new(SubsistenceFirstPolicy),
        _ => Box::new(NoopPolicy),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumption_non_negative() {
        let c = effective_consumption(PolicyInput {
            base_consumption_joules: 10.0,
            scarcity_multiplier: -1.0,
        });
        assert_eq!(c, 0.0);
    }

    #[test]
    fn noop_policy_evaluate_returns_default() {
        let policy = NoopPolicy;
        let signals = policy.evaluate(&crate::engine::WorldState::default());
        assert_eq!(signals, ControlSignals::default());
        assert!(signals.production_multipliers.is_empty());
        assert!(signals.allocation_weights.is_empty());
        assert!(signals.tax_rates.is_empty());
    }

    #[test]
    fn noop_policy_name() {
        assert_eq!(NoopPolicy.name(), "noop");
    }

    #[test]
    fn capitalist_policy_name() {
        assert_eq!(CapitalistPolicy.name(), "capitalist");
    }

    #[test]
    fn subsistence_first_policy_name() {
        assert_eq!(SubsistenceFirstPolicy.name(), "subsistence_first");
    }

    #[test]
    fn capitalist_policy_evaluate_is_default() {
        let policy = CapitalistPolicy;
        let signals = policy.evaluate(&crate::engine::WorldState::default());
        assert_eq!(signals, ControlSignals::default());
    }

    #[test]
    fn subsistence_first_policy_evaluate_is_default() {
        let policy = SubsistenceFirstPolicy;
        let signals = policy.evaluate(&crate::engine::WorldState::default());
        assert_eq!(signals, ControlSignals::default());
    }

    #[test]
    fn policy_from_kind_capitalist() {
        let p = policy_from_kind("capitalist");
        assert_eq!(p.name(), "capitalist");
    }

    #[test]
    fn policy_from_kind_subsistence_first() {
        let p = policy_from_kind("subsistence_first");
        assert_eq!(p.name(), "subsistence_first");
    }

    #[test]
    fn policy_from_kind_unknown_falls_back_to_noop() {
        let p = policy_from_kind("nonsense");
        assert_eq!(p.name(), "noop");
    }

    #[test]
    fn policy_from_kind_empty_string_falls_back_to_noop() {
        let p = policy_from_kind("");
        assert_eq!(p.name(), "noop");
    }
}
