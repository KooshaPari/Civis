//! FR-CIV-DIPLO-003: Shadow network system — covert flows of finance,
//! information, and materiel that persist under enforcement.
//!
//! # Invariants
//!
//! * Every shadow flow is recorded and logged via [`ShadowNetworkEvent`].
//! * **Leakage conservation**: the system-level leakage metric is
//!   non-negative after every tick — enforced by type-level wrapper
//!   [`NonNegativeU64`].
//! * **Enforcement intensity** accumulates with each enforcement action;
//!   exceeding the configured [`ShadowNetworkConfig::overreach_threshold`]
//!   triggers an overreach detection event and a legitimacy modifier delta.
//!
//! # Determinism
//!
//! All computation is integer-only over [`BTreeMap`]-backed collections.
//! Given the same sequence of [`record_flow`] and [`enforce`] calls, two
//! [`ShadowNetworkState`] instances produce identical state and event
//! vectors. No RNG, no floating-point, no wall-clock.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{Pair, PolityId};

// ---------------------------------------------------------------------------
// Non-negative quantity wrapper
// ---------------------------------------------------------------------------

/// A `u64` value guaranteed non-negative. This is the leakage conservation
/// enforcement type: the system-level [`ShadowNetworkState::total_leakage`]
/// is stored as this type, so a negative global invariant is impossible
/// at the type level.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct NonNegativeU64(u64);

impl NonNegativeU64 {
    /// Wrap a raw `u64` value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The inner value.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Saturating subtraction. Returns `NonNegativeU64(0)` if `rhs > self`.
    pub fn saturating_sub(self, rhs: u64) -> Self {
        Self(self.0.saturating_sub(rhs))
    }

    /// Saturating addition. Clamps at `u64::MAX`.
    pub fn saturating_add(self, rhs: u64) -> Self {
        Self(self.0.saturating_add(rhs))
    }
}

impl Default for NonNegativeU64 {
    fn default() -> Self {
        Self(0)
    }
}

// ---------------------------------------------------------------------------
// Flow type enum
// ---------------------------------------------------------------------------

/// Category of covert flow. Each variant maps to a semantic channel
/// through which resources, intelligence, or materiel move beneath
/// the diplomatic surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ShadowFlowType {
    /// Covert financial transfers (smuggled currency, shell accounts).
    Finance,
    /// Covert information exchanges (espionage, signals intelligence).
    Information,
    /// Covert materiel transfers (weapons, supplies, troops).
    Materiel,
}

// ---------------------------------------------------------------------------
// Flow record
// ---------------------------------------------------------------------------

/// A single recorded covert flow. Immutable once created; logged by the
/// system and returned as part of [`ShadowNetworkEvent::FlowRecorded`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ShadowFlow {
    /// Source polity initiating the covert flow.
    pub source: PolityId,
    /// Destination polity receiving the covert flow.
    pub destination: PolityId,
    /// Category of the flow.
    pub flow_type: ShadowFlowType,
    /// Non-negative quantity of the flow (integer units).
    pub quantity: u64,
    /// Simulation tick at which the flow was recorded.
    pub tick: u64,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Events emitted by the shadow network system each tick. Downstream
/// systems (legitimacy modifier, AI planner, scenario, JSON-RPC, replay
/// bus) consume these.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ShadowNetworkEvent {
    /// A new covert flow was recorded.
    FlowRecorded {
        /// The flow record.
        flow: ShadowFlow,
    },
    /// Leak reduction was applied to a pair during enforcement.
    LeakReduced {
        /// The pair whose aggregate leakage was reduced.
        pair: Pair,
        /// Amount reduced.
        amount: u64,
        /// Remaining aggregate leakage for the pair after reduction.
        remaining: u64,
        /// Tick of the enforcement action.
        tick: u64,
    },
    /// Enforcement intensity has exceeded the overreach threshold.
    ///
    /// The legitimacy modifier delta is negative — high enforcement
    /// overreach erodes the enforcing authority's legitimacy.
    OverreachDetected {
        /// Polity whose enforcement is over the limit.
        polity: PolityId,
        /// Current enforcement intensity.
        intensity: u32,
        /// Configured overreach threshold.
        threshold: u32,
        /// Legitimacy modifier delta to apply (always non-positive).
        legitimacy_delta: i32,
        /// Tick of the detection.
        tick: u64,
    },
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Tunable parameters for the shadow network system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowNetworkConfig {
    /// Enforcement intensity threshold above which overreach is detected.
    /// When any polity's per-tick enforcement count exceeds this value,
    /// an [`ShadowNetworkEvent::OverreachDetected`] event is emitted with
    /// a negative legitimacy modifier.
    pub overreach_threshold: u32,
    /// Maximum leak reduction per enforcement action per pair.
    /// A single enforce call can only reduce a pair's aggregate leakage
    /// by this amount.
    pub max_leak_reduction: u64,
    /// Legitimacy modifier applied per overreach detection event
    /// (negative value).
    pub overreach_legitimacy_delta: i32,
}

impl Default for ShadowNetworkConfig {
    fn default() -> Self {
        Self {
            overreach_threshold: 5,
            max_leak_reduction: 100,
            overreach_legitimacy_delta: -10,
        }
    }
}

// ---------------------------------------------------------------------------
// Per-pair aggregate
// ---------------------------------------------------------------------------

/// Aggregate shadow activity for a single actor pair. Tracked as a
/// running counter within the current tick and reset on tick boundary.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairShadowAggregate {
    /// Sum of all flow quantities for this pair in the current tick.
    pub total_leakage: NonNegativeU64,
    /// Breakdown of flows by type.
    pub flows_by_type: BTreeMap<ShadowFlowType, NonNegativeU64>,
    /// Number of individual flows recorded for this pair this tick.
    pub flow_count: u32,
}

// ---------------------------------------------------------------------------
// Shadow network state
// ---------------------------------------------------------------------------

/// The shadow network system state. Owns all tracked flows and the
/// per-tick event buffer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowNetworkState {
    /// Configuration in effect.
    pub config: ShadowNetworkConfig,
    /// Per-pair aggregate leakage, keyed by canonical [`Pair`].
    pair_aggregates: BTreeMap<Pair, PairShadowAggregate>,
    /// Per-poly enforcement intensity counter (resets each tick).
    enforcement_intensity: BTreeMap<PolityId, u32>,
    /// Total system-level leakage across all pairs in the current tick.
    /// Stored as [`NonNegativeU64`] to enforce the conservation invariant
    /// at the type level.
    total_leakage: NonNegativeU64,
    /// Cumulative legitimacy modifier deltas from overreach detections.
    legitimacy_modifier: i32,
    /// Event buffer, drained by [`Self::drain_events`].
    pending_events: Vec<ShadowNetworkEvent>,
    /// Monotonic flow id counter (for stable ordering in audit trails).
    next_flow_id: u64,
}

impl ShadowNetworkState {
    /// Construct an empty shadow network with the given `config`.
    pub fn new(config: ShadowNetworkConfig) -> Self {
        Self {
            config,
            pair_aggregates: BTreeMap::new(),
            enforcement_intensity: BTreeMap::new(),
            total_leakage: NonNegativeU64::new(0),
            legitimacy_modifier: 0,
            pending_events: Vec::new(),
            next_flow_id: 0,
        }
    }

    /// Record a single covert flow. Returns the event that was logged.
    ///
    /// FR-CIV-DIPLO-003: every shadow flow is logged.
    pub fn record_flow(&mut self, flow: ShadowFlow) -> ShadowNetworkEvent {
        let pair = Pair::new(flow.source, flow.destination);
        let quantity = flow.quantity;

        // Update per-pair aggregate.
        let agg = self.pair_aggregates.entry(pair).or_default();
        agg.total_leakage = agg.total_leakage.saturating_add(quantity);
        *agg.flows_by_type.entry(flow.flow_type).or_default() =
            NonNegativeU64::new(agg.flows_by_type.get(&flow.flow_type).map_or(0, |n| n.get()))
                .saturating_add(quantity);
        agg.flow_count += 1;

        // Update system-level leakage (non-negative by type).
        self.total_leakage = self.total_leakage.saturating_add(quantity);

        self.next_flow_id += 1;

        ShadowNetworkEvent::FlowRecorded { flow }
    }

    /// Apply enforcement to a pair, reducing their aggregate leakage.
    ///
    /// Returns events for the reduction and any overreach detection.
    ///
    /// FR-CIV-DIPLO-003: enforcement intensity feeds legitimacy modifier
    /// with overreach detection.
    pub fn enforce(
        &mut self,
        enforcer: PolityId,
        target: Pair,
        tick: u64,
    ) -> Vec<ShadowNetworkEvent> {
        let mut events = Vec::new();

        // Increment enforcement intensity for the enforcer.
        let intensity = self
            .enforcement_intensity
            .entry(enforcer)
            .or_insert(0);
        *intensity += 1;
        let current_intensity = *intensity;

        // Reduce aggregate leakage for the target pair.
        let reduction = self.config.max_leak_reduction;
        if let Some(agg) = self.pair_aggregates.get_mut(&target) {
            let before = agg.total_leakage.get();
            if before > 0 {
                agg.total_leakage = agg.total_leakage.saturating_sub(reduction);
                let remaining = agg.total_leakage.get();
                let actual_reduction = before - remaining;
                // Also reduce system-level leakage.
                self.total_leakage = self.total_leakage.saturating_sub(actual_reduction);
                events.push(ShadowNetworkEvent::LeakReduced {
                    pair: target,
                    amount: actual_reduction,
                    remaining,
                    tick,
                });
            }
        }

        // Overreach detection.
        if current_intensity > self.config.overreach_threshold {
            let delta = self.config.overreach_legitimacy_delta;
            self.legitimacy_modifier += delta;
            events.push(ShadowNetworkEvent::OverreachDetected {
                polity: enforcer,
                intensity: current_intensity,
                threshold: self.config.overreach_threshold,
                legitimacy_delta: delta,
                tick,
            });
        }

        self.pending_events.extend(events.clone());
        events
    }

    /// Reset per-tick counters (enforcement intensity, pair aggregates,
    /// system-level leakage). Call at the start of a new tick before
    /// recording flows.
    pub fn reset_tick(&mut self) {
        self.enforcement_intensity.clear();
        self.pair_aggregates.clear();
        self.total_leakage = NonNegativeU64::new(0);
    }

    /// Number of distinct actor pairs with active shadow flows this tick.
    pub fn active_pair_count(&self) -> usize {
        self.pair_aggregates.len()
    }

    /// Total system-level leakage for the current tick.
    pub fn total_leakage(&self) -> NonNegativeU64 {
        self.total_leakage
    }

    /// Cumulative legitimacy modifier from overreach detections.
    pub fn legitimacy_modifier(&self) -> i32 {
        self.legitimacy_modifier
    }

    /// Enforcement intensity for a specific polity in the current tick.
    pub fn enforcement_intensity(&self, polity: PolityId) -> u32 {
        self.enforcement_intensity.get(&polity).copied().unwrap_or(0)
    }

    /// Look up the aggregate shadow activity for a specific pair.
    pub fn get_pair_aggregate(&self, pair: Pair) -> Option<&PairShadowAggregate> {
        self.pair_aggregates.get(&pair)
    }

    /// Drain all pending [`ShadowNetworkEvent`]s accumulated since the
    /// last drain.
    pub fn drain_events(&mut self) -> Vec<ShadowNetworkEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Peek at pending events without consuming them.
    pub fn pending_events(&self) -> &[ShadowNetworkEvent] {
        &self.pending_events
    }

    /// Process a batch of flows and enforcement actions for a single tick.
    /// Returns all events emitted during the tick.
    ///
    /// This is the main tick entry point: callers supply the flows to
    /// record and the enforcement actions to apply, and the method
    /// returns the ordered event stream.
    pub fn process_tick(
        &mut self,
        flows: &[ShadowFlow],
        enforcement_actions: &[(PolityId, Pair)],
        tick: u64,
    ) -> Vec<ShadowNetworkEvent> {
        let mut events = Vec::new();

        // Record all flows first.
        for flow in flows {
            events.push(self.record_flow(flow.clone()));
        }

        // Apply enforcement actions.
        for (enforcer, target) in enforcement_actions {
            events.extend(self.enforce(*enforcer, *target, tick));
        }

        events
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn p(id: u32) -> PolityId {
        PolityId::new(id)
    }

    fn pair(a: u32, b: u32) -> Pair {
        Pair::new(p(a), p(b))
    }

    // -- FR-CIV-DIPLO-003-01: Shadow flow struct and recording ---------------

    /// FR-CIV-DIPLO-003-01.
    ///
    /// A single finance flow is recorded, the pair aggregate is updated,
    /// and the system-level leakage increases by the flow quantity.
    #[test]
    fn record_flow_updates_pair_aggregate_and_total_leakage() {
        let mut state = ShadowNetworkState::new(ShadowNetworkConfig::default());
        let flow = ShadowFlow {
            source: p(1),
            destination: p(2),
            flow_type: ShadowFlowType::Finance,
            quantity: 50,
            tick: 1,
        };

        let event = state.record_flow(flow.clone());
        assert_eq!(
            event,
            ShadowNetworkEvent::FlowRecorded { flow }
        );

        let agg = state.get_pair_aggregate(pair(1, 2)).unwrap();
        assert_eq!(agg.total_leakage, NonNegativeU64::new(50));
        assert_eq!(
            agg.flows_by_type[&ShadowFlowType::Finance],
            NonNegativeU64::new(50)
        );
        assert_eq!(agg.flow_count, 1);
        assert_eq!(state.total_leakage(), NonNegativeU64::new(50));
    }

    /// FR-CIV-DIPLO-003-01.
    ///
    /// Multiple flows of different types to the same pair accumulate
    /// correctly.
    #[test]
    fn multiple_flows_accumulate_by_type() {
        let mut state = ShadowNetworkState::new(ShadowNetworkConfig::default());

        state.record_flow(ShadowFlow {
            source: p(1),
            destination: p(2),
            flow_type: ShadowFlowType::Finance,
            quantity: 30,
            tick: 1,
        });
        state.record_flow(ShadowFlow {
            source: p(1),
            destination: p(2),
            flow_type: ShadowFlowType::Information,
            quantity: 20,
            tick: 1,
        });
        state.record_flow(ShadowFlow {
            source: p(1),
            destination: p(2),
            flow_type: ShadowFlowType::Materiel,
            quantity: 10,
            tick: 1,
        });

        let agg = state.get_pair_aggregate(pair(1, 2)).unwrap();
        assert_eq!(agg.total_leakage, NonNegativeU64::new(60));
        assert_eq!(agg.flow_count, 3);
        assert_eq!(state.active_pair_count(), 1);
        assert_eq!(state.total_leakage(), NonNegativeU64::new(60));
    }

    // -- FR-CIV-DIPLO-003-02: Leakage conservation enforcement ---------------

    /// FR-CIV-DIPLO-003-02.
    ///
    /// Total leakage is always non-negative after enforcement, even when
    /// the reduction amount exceeds the current leakage.
    #[test]
    fn leakage_conservation_never_goes_negative() {
        let config = ShadowNetworkConfig {
            max_leak_reduction: 1000,
            ..Default::default()
        };
        let mut state = ShadowNetworkState::new(config);

        // Record a small flow.
        state.record_flow(ShadowFlow {
            source: p(1),
            destination: p(2),
            flow_type: ShadowFlowType::Finance,
            quantity: 5,
            tick: 1,
        });
        assert_eq!(state.total_leakage(), NonNegativeU64::new(5));

        // Enforce with a reduction larger than the leakage.
        let events = state.enforce(p(3), pair(1, 2), 1);
        assert_eq!(state.total_leakage(), NonNegativeU64::new(0));
        assert_eq!(events.len(), 1);
        match &events[0] {
            ShadowNetworkEvent::LeakReduced {
                amount, remaining, ..
            } => {
                assert_eq!(*amount, 5);
                assert_eq!(*remaining, 0);
            }
            other => panic!("expected LeakReduced, got {:?}", other),
        }
    }

    /// FR-CIV-DIPLO-003-02.
    ///
    /// Enforce on a pair with no flows is a no-op (no event emitted).
    #[test]
    fn enforce_on_empty_pair_emits_no_event() {
        let mut state = ShadowNetworkState::new(ShadowNetworkConfig::default());
        let events = state.enforce(p(1), pair(1, 2), 1);
        assert!(events.is_empty());
        assert_eq!(state.total_leakage(), NonNegativeU64::new(0));
    }

    // -- FR-CIV-DIPLO-003-03: Enforcement intensity and overreach ------------

    /// FR-CIV-DIPLO-003-03.
    ///
    /// Enforcement intensity accumulates and triggers overreach detection
    /// when the threshold is exceeded.
    #[test]
    fn enforcement_overreach_detection() {
        let config = ShadowNetworkConfig {
            overreach_threshold: 2,
            overreach_legitimacy_delta: -5,
            ..Default::default()
        };
        let mut state = ShadowNetworkState::new(config);

        // Intensity 1 — no overreach.
        let events = state.enforce(p(1), pair(2, 3), 1);
        assert!(!events.iter().any(|e| matches!(
            e,
            ShadowNetworkEvent::OverreachDetected { .. }
        )));
        assert_eq!(state.enforcement_intensity(p(1)), 1);

        // Intensity 2 — at threshold, no overreach yet.
        let events = state.enforce(p(1), pair(2, 3), 1);
        assert!(!events.iter().any(|e| matches!(
            e,
            ShadowNetworkEvent::OverreachDetected { .. }
        )));
        assert_eq!(state.enforcement_intensity(p(1)), 2);

        // Intensity 3 — above threshold, overreach detected.
        let events = state.enforce(p(1), pair(2, 3), 1);
        let overreach: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, ShadowNetworkEvent::OverreachDetected { .. }))
            .collect();
        assert_eq!(overreach.len(), 1);
        assert_eq!(state.legitimacy_modifier(), -5);
        assert_eq!(state.enforcement_intensity(p(1)), 3);
    }

    /// FR-CIV-DIPLO-003-03.
    ///
    /// Different polities have independent enforcement intensity counters.
    #[test]
    fn enforcement_intensity_is_per_polity() {
        let config = ShadowNetworkConfig {
            overreach_threshold: 1,
            ..Default::default()
        };
        let mut state = ShadowNetworkState::new(config);

        // p(1) enforces once — no overreach (intensity=1, threshold=1).
        let events1 = state.enforce(p(1), pair(2, 3), 1);
        assert!(!events1.iter().any(|e| matches!(
            e,
            ShadowNetworkEvent::OverreachDetected { .. }
        )));

        // p(2) enforces once — no overreach.
        let events2 = state.enforce(p(2), pair(2, 3), 1);
        assert!(!events2.iter().any(|e| matches!(
            e,
            ShadowNetworkEvent::OverreachDetected { .. }
        )));

        // p(1) enforces again — overreach (intensity=2 > threshold=1).
        let events3 = state.enforce(p(1), pair(2, 3), 2);
        assert!(events3.iter().any(|e| matches!(
            e,
            ShadowNetworkEvent::OverreachDetected { polity, .. } if *polity == p(1)
        )));

        // p(2) still at intensity=1, no overreach.
        assert_eq!(state.enforcement_intensity(p(2)), 1);
    }

    // -- FR-CIV-DIPLO-003-04: Flow logging completeness ----------------------

    /// FR-CIV-DIPLO-003-04.
    ///
    /// Every recorded flow produces exactly one event in the event buffer.
    #[test]
    fn every_flow_is_logged() {
        let mut state = ShadowNetworkState::new(ShadowNetworkConfig::default());
        for i in 0..10 {
            state.record_flow(ShadowFlow {
                source: p(1),
                destination: p(2),
                flow_type: ShadowFlowType::Materiel,
                quantity: (i + 1) * 10,
                tick: i,
            });
        }

        let events = state.drain_events();
        let flow_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, ShadowNetworkEvent::FlowRecorded { .. }))
            .collect();
        assert_eq!(flow_events.len(), 10);
    }

    // -- FR-CIV-DIPLO-003-05: Symmetric pair handling ------------------------

    /// FR-CIV-DIPLO-003-05.
    ///
    /// Flows from a->b and from b->a are stored under the same canonical
    /// pair (a < b), and both directions contribute to the same aggregate.
    #[test]
    fn bidirectional_flows_share_pair_aggregate() {
        let mut state = ShadowNetworkState::new(ShadowNetworkConfig::default());

        state.record_flow(ShadowFlow {
            source: p(2),
            destination: p(1),
            flow_type: ShadowFlowType::Finance,
            quantity: 30,
            tick: 1,
        });
        state.record_flow(ShadowFlow {
            source: p(1),
            destination: p(2),
            flow_type: ShadowFlowType::Information,
            quantity: 20,
            tick: 1,
        });

        let agg = state.get_pair_aggregate(pair(1, 2)).unwrap();
        assert_eq!(agg.total_leakage, NonNegativeU64::new(50));
        assert_eq!(agg.flow_count, 2);
        assert_eq!(
            agg.flows_by_type[&ShadowFlowType::Finance],
            NonNegativeU64::new(30)
        );
        assert_eq!(
            agg.flows_by_type[&ShadowFlowType::Information],
            NonNegativeU64::new(20)
        );
    }

    // -- FR-CIV-DIPLO-003-06: process_tick integration -----------------------

    /// FR-CIV-DIPLO-003-006.
    ///
    /// `process_tick` records flows and applies enforcement in order,
    /// returning a combined event stream.
    #[test]
    fn process_tick_records_flows_and_applies_enforcement() {
        let mut state = ShadowNetworkState::new(ShadowNetworkConfig::default());

        let flows = vec![
            ShadowFlow {
                source: p(1),
                destination: p(2),
                flow_type: ShadowFlowType::Finance,
                quantity: 40,
                tick: 1,
            },
            ShadowFlow {
                source: p(3),
                destination: p(4),
                flow_type: ShadowFlowType::Materiel,
                quantity: 60,
                tick: 1,
            },
        ];

        let enforcement = vec![(p(5), pair(1, 2))];

        let events = state.process_tick(&flows, &enforcement, 1);

        // Should have 2 FlowRecorded + 1 LeakReduced = 3 events.
        let flow_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, ShadowNetworkEvent::FlowRecorded { .. }))
            .collect();
        let reduce_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, ShadowNetworkEvent::LeakReduced { .. }))
            .collect();
        assert_eq!(flow_events.len(), 2);
        assert_eq!(reduce_events.len(), 1);

        // Pair (1,2) had 40 leaked, enforcement reduced by max_leak_reduction (100).
        let agg = state.get_pair_aggregate(pair(1, 2)).unwrap();
        assert_eq!(agg.total_leakage, NonNegativeU64::new(0));

        // Pair (3,4) was not enforced.
        let agg2 = state.get_pair_aggregate(pair(3, 4)).unwrap();
        assert_eq!(agg2.total_leakage, NonNegativeU64::new(60));
    }

    // -- FR-CIV-DIPLO-003-07: reset_tick clears state -----------------------

    /// FR-CIV-DIPLO-003-07.
    ///
    /// `reset_tick` clears per-tick counters but preserves cumulative
    /// legitimacy modifier.
    #[test]
    fn reset_tick_clears_counters_preserves_legitimacy() {
        let config = ShadowNetworkConfig {
            overreach_threshold: 0,
            overreach_legitimacy_delta: -10,
            ..Default::default()
        };
        let mut state = ShadowNetworkState::new(config);

        state.record_flow(ShadowFlow {
            source: p(1),
            destination: p(2),
            flow_type: ShadowFlowType::Finance,
            quantity: 100,
            tick: 1,
        });
        // Overreach immediately (threshold=0, first enforcement = intensity 1 > 0).
        state.enforce(p(3), pair(1, 2), 1);
        assert_eq!(state.legitimacy_modifier(), -10);
        assert_eq!(state.active_pair_count(), 1);

        state.reset_tick();

        // Counters reset.
        assert_eq!(state.active_pair_count(), 0);
        assert_eq!(state.total_leakage(), NonNegativeU64::new(0));
        assert_eq!(state.enforcement_intensity(p(3)), 0);
        // Legitimacy modifier persists across ticks.
        assert_eq!(state.legitimacy_modifier(), -10);
    }

    // -- NonNegativeU64 unit tests -------------------------------------------

    #[test]
    fn non_negative_saturating_sub_clamps_to_zero() {
        let v = NonNegativeU64::new(5);
        assert_eq!(v.saturating_sub(10), NonNegativeU64::new(0));
        assert_eq!(v.saturating_sub(3), NonNegativeU64::new(2));
    }

    #[test]
    fn non_negative_saturating_add_clamps_to_max() {
        let v = NonNegativeU64::new(u64::MAX - 5);
        assert_eq!(v.saturating_add(10), NonNegativeU64::new(u64::MAX));
        assert_eq!(v.saturating_add(3), NonNegativeU64::new(u64::MAX - 2));
    }

    // -- Determinism tests ---------------------------------------------------

    /// FR-CIV-DIPLO-003: two identical tick sequences produce identical
    /// state and event vectors.
    #[test]
    fn deterministic_tick_processing() {
        let build = || {
            let config = ShadowNetworkConfig {
                overreach_threshold: 2,
                overreach_legitimacy_delta: -5,
                max_leak_reduction: 25,
            };
            let mut state = ShadowNetworkState::new(config);

            let flows = vec![
                ShadowFlow {
                    source: p(1),
                    destination: p(2),
                    flow_type: ShadowFlowType::Finance,
                    quantity: 30,
                    tick: 1,
                },
                ShadowFlow {
                    source: p(2),
                    destination: p(1),
                    flow_type: ShadowFlowType::Information,
                    quantity: 20,
                    tick: 1,
                },
            ];
            let enforcement = vec![(p(3), pair(1, 2))];
            let _events = state.process_tick(&flows, &enforcement, 1);
            state
        };

        let s1 = build();
        let s2 = build();
        assert_eq!(s1, s2);
    }
}
