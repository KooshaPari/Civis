//! `civ-institutions` — civic institutions for the Civis simulation.
//!
//! A **civic institution** is a population-gated, per-settlement social
//! construct (Temple, Garrison, future Council / Court / School). Institutions
//! spawn when a settlement's population crosses an unlock threshold and
//! upgrade when it crosses a higher threshold.
//!
//! ## Population thresholds
//!
//! Each [`InstitutionKind`] defines two thresholds:
//!
//! - **Unlock** (L1 spawn): the smallest population at which the institution
//!   first appears.
//! - **L2 upgrade**: the population at which the institution upgrades from
//!   level 1 → level 2.
//!
//! Thresholds are exported as `pub const` so the engine's
//! [`phase_institutions`](https://docs.rs/civ_engine) logic, the
//! `civ-server` ws_bridge, and the Bevy reference client can all agree on
//! the exact cut-offs without code duplication.
//!
//! ## One-shot event semantics
//!
//! The owning engine is expected to track, for each
//! `(settlement_id, kind, level)` triple, whether it has already emitted an
//! `InstitutionEvent` for that triple. This guarantees that transient
//! population dips (settlement drops below the unlock threshold for one
//! tick) do not produce duplicate spawn events, and that L1 → L2 upgrades
//! emit exactly once.
//!
//! ## Spec coverage
//!
//! - **FR-CIV-GOV-001**: Temple spawns when a settlement crosses
//!   [`TEMPLE_UNLOCK_POPULATION`]; Garrison spawns when a settlement
//!   crosses [`GARRISON_UNLOCK_POPULATION`].
//! - **FR-CIV-GOV-002**: Civic events stream is exposed read-only via
//!   [`civ_engine::Simulation::last_tick_institution_events`].
//! - **FR-CIV-GOV-003**: L1 → L2 upgrade fires when a settlement crosses
//!   [`TEMPLE_L2_POPULATION`] (resp. [`GARRISON_L2_POPULATION`]) and is
//!   one-shot per `(settlement_id, kind, level)`.
//! - **FR-INST-001**: Each civilization SHALL have a governance type.
//! - **FR-INST-002**: Capture score accumulates each tick.
//! - **FR-INST-003**: Threshold events fire at 0.75 capture.
//! - **FR-INST-004**: Collapse triggers governance transition.
//! - **FR-INST-005**: Time-series data for post-run analysis.
//! - **FR-INST-006**: Citizen lifecycle driven by institutions + economy.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod capture;
pub mod collapse;
pub mod events;
pub mod faction_split;
pub mod governance;
pub mod legitimacy;

pub use capture::{CaptureConfig, CaptureScore, CAPTURE_THRESHOLD_BP};
pub use collapse::{check_collapse, transition_target, CollapseCause, CollapseTransitionEvent};
pub use events::{check_capture_threshold, CaptureThresholdEvent, CAPTURE_THRESHOLD_EVENT};
pub use faction_split::{
    maybe_split_faction, splinter_id, splinter_name, Faction, FactionSplitEvent,
    InstitutionCohesion, DEFAULT_COHESION_THRESHOLD, MAX_COHESION, MIN_COHESION,
};
pub use governance::GovernanceType;
pub use legitimacy::{
    GovernanceOutcome, InstitutionLegitimacy, DEFAULT_LEGITIMACY, LEGITIMACY_COLLAPSE_THRESHOLD,
    MAX_LEGITIMACY, MIN_LEGITIMACY,
};

use serde::{Deserialize, Serialize};

/// Temple institution — religious / civic center. Spawns when a settlement
/// grows large enough to support a permanent religious functionary.
///
/// Used by:
/// - `civ_engine::Simulation::phase_institutions` — emits the `Spawned` event
/// - `civ-server` ws_bridge — surfaces to the Bevy client
/// - Religion / mood research modules — pulls belief signals from this
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum InstitutionKind {
    /// Religious / civic center.
    Temple,
    /// Military / guard post.
    Garrison,
}

impl InstitutionKind {
    /// Total number of institution kinds currently modeled. Useful for
    /// pre-allocating capacity in `BTreeMap` lookups.
    pub const COUNT: usize = 2;

    /// Returns the index of this kind in a stable, sorted iteration order.
    /// Index 0 = `Temple`, index 1 = `Garrison`.
    pub fn index(self) -> usize {
        match self {
            InstitutionKind::Temple => 0,
            InstitutionKind::Garrison => 1,
        }
    }

    /// Returns the human-readable name of this institution kind.
    pub fn as_str(self) -> &'static str {
        match self {
            InstitutionKind::Temple => "Temple",
            InstitutionKind::Garrison => "Garrison",
        }
    }
}

/// A persisted civic institution record for a single settlement. There is at
/// most **one** active institution record per `(settlement_id, kind)` pair,
/// tracked at the highest level the settlement has ever reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Institution {
    /// Which kind of institution this record represents.
    pub kind: InstitutionKind,
    /// Current level. `1` = spawned (L1), `2` = first upgrade (L2). Higher
    /// levels may be added in future specs.
    pub level: u8,
}

/// Population threshold at which a settlement unlocks (spawns) a Temple.
/// Settlements below this population have no Temple.
pub const TEMPLE_UNLOCK_POPULATION: u32 = 50;

/// Population threshold at which a Temple upgrades from L1 to L2.
pub const TEMPLE_L2_POPULATION: u32 = 200;

/// Population threshold at which a settlement unlocks (spawns) a Garrison.
/// Settlements below this population have no Garrison.
pub const GARRISON_UNLOCK_POPULATION: u32 = 120;

/// Population threshold at which a Garrison upgrades from L1 to L2.
pub const GARRISON_L2_POPULATION: u32 = 400;

// ---------------------------------------------------------------------------
// FR-INST-005 — Time-series storage for institutional metrics
// ---------------------------------------------------------------------------

/// A single time-series data point for institutional metrics.
///
/// Stored in the metrics DB for post-run analysis (FR-INST-005).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionTimeSeriesPoint {
    /// Tick when this data point was recorded.
    pub tick: u64,
    /// Civilization identifier.
    pub civilization_id: u32,
    /// Governance type at this tick.
    pub governance_type: GovernanceType,
    /// Capture score at this tick (basis points).
    pub capture_bp: i64,
    /// Legitimacy at this tick (fixed-point, 0–1000 scale).
    pub legitimacy_x1000: i32,
    /// Population driving institutional thresholds.
    pub population: u32,
}

/// Append-only time-series log for institutional metrics per civilization.
///
/// The owning simulation pushes a data point each tick. Consumers
/// (analytics, replay export, dashboard) read the series for trend
/// analysis.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionTimeSeries {
    /// Civilization identifier this series belongs to.
    pub civilization_id: u32,
    /// Chronological data points.
    pub points: Vec<InstitutionTimeSeriesPoint>,
}

impl InstitutionTimeSeries {
    /// Creates an empty series for a civilization.
    pub fn new(civilization_id: u32) -> Self {
        Self {
            civilization_id,
            points: Vec::new(),
        }
    }

    /// Appends a data point to the series.
    pub fn push(&mut self, point: InstitutionTimeSeriesPoint) {
        self.points.push(point);
    }

    /// Returns the number of recorded data points.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Returns true when no data points have been recorded.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Returns a reference to the most recent data point, if any.
    pub fn latest(&self) -> Option<&InstitutionTimeSeriesPoint> {
        self.points.last()
    }
}

// ---------------------------------------------------------------------------
// FR-INST-006 — Citizen lifecycle driven by institutional and economic state
// ---------------------------------------------------------------------------

/// Citizen lifecycle event types driven by institutional and economic state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleEvent {
    /// A new citizen is born. Triggered when institutional stability and
    /// economic conditions support population growth.
    Birth,
    /// A citizen migrates to another settlement. Triggered when local
    /// conditions deteriorate relative to alternatives.
    Migration,
    /// A citizen dies. Can be triggered by economic scarcity, institutional
    /// collapse, or natural causes.
    Death,
}

/// Economic condition signal fed into lifecycle decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EconomicCondition {
    /// Joule surplus per capita (basis points of subsistence threshold).
    /// Positive = above subsistence, negative = deficit.
    pub joule_surplus_bp: i64,
    /// Whether the treasury has sufficient reserves for public services.
    pub treasury_healthy: bool,
}

/// Institutional stability signal fed into lifecycle decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionalCondition {
    /// Current governance type.
    pub governance_type: GovernanceType,
    /// Current legitimacy (0–1000 fixed-point).
    pub legitimacy_x1000: i32,
    /// Whether the institution has collapsed this tick.
    pub collapsed: bool,
}

/// Result of evaluating whether a lifecycle event should fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleDecision {
    /// The lifecycle event, if conditions warrant one.
    pub event: Option<LifecycleEvent>,
    /// Whether the citizen's productivity is reduced (e.g. due to stress).
    pub productivity_reduced: bool,
}

/// Evaluate a citizen lifecycle event based on institutional and economic state.
///
/// Returns a [`LifecycleDecision`] indicating whether a birth, migration, or
/// death event should occur, based on the combination of governance legitimacy
/// and economic health.
pub fn evaluate_lifecycle(
    institutional: &InstitutionalCondition,
    economic: &EconomicCondition,
) -> LifecycleDecision {
    // Collapsed institutions cause death or migration.
    if institutional.collapsed {
        let event = if economic.treasury_healthy {
            // Treasury healthy but institution collapsed: citizens migrate.
            Some(LifecycleEvent::Migration)
        } else {
            // Both collapsed: citizens die.
            Some(LifecycleEvent::Death)
        };
        return LifecycleDecision {
            event,
            productivity_reduced: true,
        };
    }

    // Low legitimacy reduces productivity and risks migration.
    if institutional.legitimacy_x1000 < 300 {
        let event = if economic.joule_surplus_bp < 0 {
            Some(LifecycleEvent::Migration)
        } else {
            None
        };
        return LifecycleDecision {
            event,
            productivity_reduced: true,
        };
    }

    // Negative economic surplus with decent legitimacy: migration risk.
    if economic.joule_surplus_bp < -1_000 {
        return LifecycleDecision {
            event: Some(LifecycleEvent::Migration),
            productivity_reduced: true,
        };
    }

    // Positive conditions with stable institutions: births.
    if economic.joule_surplus_bp > 500 && institutional.legitimacy_x1000 > 700 {
        return LifecycleDecision {
            event: Some(LifecycleEvent::Birth),
            productivity_reduced: false,
        };
    }

    // Default: no lifecycle event, no productivity reduction.
    LifecycleDecision {
        event: None,
        productivity_reduced: false,
    }
}

// ---------------------------------------------------------------------------
// FR-INST-001 — Civilization institutional state (ties everything together)
// ---------------------------------------------------------------------------

/// Full institutional state for a single civilization.
///
/// Combines governance type (FR-INST-001), capture score (FR-INST-002),
/// legitimacy tracking, and exposes the tick update interface that drives
/// collapse transitions (FR-INST-004) and time-series recording (FR-INST-005).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CivilizationState {
    /// Civilization identifier.
    pub civilization_id: u32,
    /// Current governance type (FR-INST-001).
    pub governance_type: GovernanceType,
    /// Capture score (FR-INST-002).
    pub capture: CaptureScore,
    /// Legitimacy state.
    pub legitimacy: InstitutionLegitimacy,
    /// Time-series log for this civilization (FR-INST-005).
    pub time_series: InstitutionTimeSeries,
}

impl CivilizationState {
    /// Creates a new civilization state with the given governance type.
    pub fn new(civilization_id: u32, governance_type: GovernanceType) -> Self {
        Self {
            civilization_id,
            governance_type,
            capture: CaptureScore::new(),
            legitimacy: InstitutionLegitimacy::default(),
            time_series: InstitutionTimeSeries::new(civilization_id),
        }
    }

    /// Advances institutional state by one tick.
    ///
    /// - Accumulates capture score based on resource `concentration` (basis points).
    /// - Checks for capture threshold events.
    /// - Checks for collapse and applies governance transition.
    /// - Records a time-series data point.
    ///
    /// Returns any events emitted during this tick.
    pub fn tick(
        &mut self,
        tick: u64,
        concentration_bp: i64,
        population: u32,
    ) -> TickResult {
        let mut events = TickResult::default();

        // FR-INST-002: Accumulate capture score.
        self.capture.accumulate(concentration_bp);

        // FR-INST-003: Check capture threshold event.
        if let Some(event) = check_capture_threshold(
            tick,
            self.civilization_id,
            self.capture.value_bp,
            self.capture.threshold_bp,
            false, // TODO: track already-fired state
        ) {
            events.capture_threshold = Some(event);
        }

        // FR-INST-004: Check for collapse and governance transition.
        if let Some(transition) = check_collapse(
            tick,
            self.civilization_id,
            self.governance_type,
            &self.legitimacy,
            &self.capture,
        ) {
            self.governance_type = transition.to_type;
            events.collapse_transition = Some(transition);
        }

        // FR-INST-005: Record time-series data point.
        self.time_series.push(InstitutionTimeSeriesPoint {
            tick,
            civilization_id: self.civilization_id,
            governance_type: self.governance_type,
            capture_bp: self.capture.value_bp,
            legitimacy_x1000: (self.legitimacy.value * 1000.0) as i32,
            population,
        });

        events
    }
}

/// Events emitted during a single tick of [`CivilizationState::tick`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TickResult {
    /// Capture threshold event, if emitted this tick.
    pub capture_threshold: Option<CaptureThresholdEvent>,
    /// Collapse transition event, if triggered this tick.
    pub collapse_transition: Option<CollapseTransitionEvent>,
}

#[cfg(test)]
#[allow(clippy::assertions_on_constants)]
mod tests {
    use super::*;

    #[test]
    fn institution_kind_index_is_stable() {
        assert_eq!(InstitutionKind::Temple.index(), 0);
        assert_eq!(InstitutionKind::Garrison.index(), 1);
    }

    #[test]
    fn institution_kind_count_is_2() {
        assert_eq!(InstitutionKind::COUNT, 2);
    }

    #[test]
    fn thresholds_are_strictly_ordered() {
        assert!(TEMPLE_UNLOCK_POPULATION < TEMPLE_L2_POPULATION);
        assert!(GARRISON_UNLOCK_POPULATION < GARRISON_L2_POPULATION);
    }

    #[test]
    fn temple_unlock_lower_than_garrison() {
        assert!(TEMPLE_UNLOCK_POPULATION < GARRISON_UNLOCK_POPULATION);
    }

    #[test]
    fn time_series_push_and_len() {
        let mut ts = InstitutionTimeSeries::new(1);
        assert!(ts.is_empty());
        ts.push(InstitutionTimeSeriesPoint {
            tick: 0,
            civilization_id: 1,
            governance_type: GovernanceType::Democracy,
            capture_bp: 0,
            legitimacy_x1000: 1000,
            population: 100,
        });
        assert_eq!(ts.len(), 1);
        assert!(!ts.is_empty());
        assert!(ts.latest().is_some());
    }

    #[test]
    fn civilization_state_new_defaults() {
        let state = CivilizationState::new(42, GovernanceType::Democracy);
        assert_eq!(state.civilization_id, 42);
        assert_eq!(state.governance_type, GovernanceType::Democracy);
        assert_eq!(state.capture.value_bp, 0);
        assert!(state.time_series.is_empty());
    }

    #[test]
    fn lifecycle_birth_on_good_conditions() {
        let inst = InstitutionalCondition {
            governance_type: GovernanceType::Democracy,
            legitimacy_x1000: 800,
            collapsed: false,
        };
        let econ = EconomicCondition {
            joule_surplus_bp: 600,
            treasury_healthy: true,
        };
        let decision = evaluate_lifecycle(&inst, &econ);
        assert_eq!(decision.event, Some(LifecycleEvent::Birth));
        assert!(!decision.productivity_reduced);
    }

    #[test]
    fn lifecycle_death_on_collapse() {
        let inst = InstitutionalCondition {
            governance_type: GovernanceType::Anarchy,
            legitimacy_x1000: 200,
            collapsed: true,
        };
        let econ = EconomicCondition {
            joule_surplus_bp: -500,
            treasury_healthy: false,
        };
        let decision = evaluate_lifecycle(&inst, &econ);
        assert_eq!(decision.event, Some(LifecycleEvent::Death));
        assert!(decision.productivity_reduced);
    }

    #[test]
    fn lifecycle_migration_on_institutional_failure() {
        let inst = InstitutionalCondition {
            governance_type: GovernanceType::Anarchy,
            legitimacy_x1000: 200,
            collapsed: true,
        };
        let econ = EconomicCondition {
            joule_surplus_bp: 100,
            treasury_healthy: true,
        };
        let decision = evaluate_lifecycle(&inst, &econ);
        assert_eq!(decision.event, Some(LifecycleEvent::Migration));
        assert!(decision.productivity_reduced);
    }
}
