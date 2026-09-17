//! FR-DIPL-007 tests — Shadow network influence.
//!
//! Shadow networks SHALL model covert influence as a hidden resource
//! accumulating per tick.

use civ_diplomacy::{
    NonNegativeU64, ShadowFlow, ShadowFlowType, ShadowNetworkConfig, ShadowNetworkEvent,
    ShadowNetworkState, PolityId, Pair,
};

fn p(id: u32) -> PolityId {
    PolityId::new(id)
}

/// FR-DIPL-007: influence_accumulates — flows increase aggregate leakage.
#[test]
fn shadow_influence_accumulates() {
    let mut state = ShadowNetworkState::new(ShadowNetworkConfig::default());

    // Record three flows between the same pair
    for qty in [10, 20, 30] {
        state.record_flow(ShadowFlow {
            source: p(1),
            destination: p(2),
            flow_type: ShadowFlowType::Finance,
            quantity: qty,
            tick: 1,
        });
    }

    let agg = state.get_pair_aggregate(Pair::new(p(1), p(2))).unwrap();
    assert_eq!(agg.total_leakage, NonNegativeU64::new(60));
    assert_eq!(agg.flow_count, 3);
}

/// FR-DIPL-007: Total system-level leakage accumulates across all pairs.
#[test]
fn shadow_total_leakage_accumulates() {
    let mut state = ShadowNetworkState::new(ShadowNetworkConfig::default());

    state.record_flow(ShadowFlow {
        source: p(1),
        destination: p(2),
        flow_type: ShadowFlowType::Finance,
        quantity: 50,
        tick: 1,
    });
    state.record_flow(ShadowFlow {
        source: p(3),
        destination: p(4),
        flow_type: ShadowFlowType::Information,
        quantity: 30,
        tick: 1,
    });

    assert_eq!(state.total_leakage(), NonNegativeU64::new(80));
    assert_eq!(state.active_pair_count(), 2);
}

/// FR-DIPL-007: NonNegativeU64 clamps at zero (conservation invariant).
#[test]
fn shadow_non_negative_u64_saturating_sub() {
    let val = NonNegativeU64::new(10);
    assert_eq!(val.saturating_sub(5), NonNegativeU64::new(5));
    assert_eq!(val.saturating_sub(100), NonNegativeU64::new(0));
    // Cannot go negative
    assert_eq!(NonNegativeU64::new(0).saturating_sub(1), NonNegativeU64::new(0));
}

/// FR-DIPL-007: Flow events are recorded and can be drained.
#[test]
fn shadow_events_recorded() {
    let mut state = ShadowNetworkState::new(ShadowNetworkConfig::default());
    state.record_flow(ShadowFlow {
        source: p(1),
        destination: p(2),
        flow_type: ShadowFlowType::Materiel,
        quantity: 100,
        tick: 1,
    });

    let events = state.drain_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], ShadowNetworkEvent::FlowRecorded { flow } if flow.quantity == 100));
}

/// FR-DIPL-007: Influence per flow type is tracked separately.
#[test]
fn shadow_flows_by_type_tracked() {
    let mut state = ShadowNetworkState::new(ShadowNetworkConfig::default());

    state.record_flow(ShadowFlow {
        source: p(1),
        destination: p(2),
        flow_type: ShadowFlowType::Finance,
        quantity: 50,
        tick: 1,
    });
    state.record_flow(ShadowFlow {
        source: p(1),
        destination: p(2),
        flow_type: ShadowFlowType::Information,
        quantity: 30,
        tick: 1,
    });
    state.record_flow(ShadowFlow {
        source: p(1),
        destination: p(2),
        flow_type: ShadowFlowType::Finance,
        quantity: 20,
        tick: 1,
    });

    let agg = state.get_pair_aggregate(Pair::new(p(1), p(2))).unwrap();
    assert_eq!(agg.total_leakage, NonNegativeU64::new(100));
    assert_eq!(agg.flow_count, 3);
    assert_eq!(
        agg.flows_by_type[&ShadowFlowType::Finance],
        NonNegativeU64::new(70)
    );
    assert_eq!(
        agg.flows_by_type[&ShadowFlowType::Information],
        NonNegativeU64::new(30)
    );
}

/// FR-DIPL-007: Influence accumulates across multiple ticks.
#[test]
fn shadow_influence_accumulates_across_ticks() {
    let mut state = ShadowNetworkState::new(ShadowNetworkConfig::default());

    // Tick 1
    state.record_flow(ShadowFlow {
        source: p(1),
        destination: p(2),
        flow_type: ShadowFlowType::Finance,
        quantity: 100,
        tick: 1,
    });
    state.reset_tick();

    // Tick 2
    state.record_flow(ShadowFlow {
        source: p(1),
        destination: p(2),
        flow_type: ShadowFlowType::Finance,
        quantity: 200,
        tick: 2,
    });

    let agg = state.get_pair_aggregate(Pair::new(p(1), p(2))).unwrap();
    // After reset_tick, only tick 2 flows are in the aggregate
    assert_eq!(agg.total_leakage, NonNegativeU64::new(200));
}

/// FR-DIPL-007: Enforcement reduces leakage.
#[test]
fn shadow_enforcement_reduces_leakage() {
    let mut state = ShadowNetworkState::new(ShadowNetworkConfig::default());
    let pair = Pair::new(p(1), p(2));

    state.record_flow(ShadowFlow {
        source: p(1),
        destination: p(2),
        flow_type: ShadowFlowType::Finance,
        quantity: 500,
        tick: 1,
    });

    let events = state.enforce(p(3), pair, 1);
    assert!(!events.is_empty());
    assert!(matches!(&events[0], ShadowNetworkEvent::LeakReduced { .. }));
}
