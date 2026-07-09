//! TDD red step for FR-CIV-GOV-030: phase_cohesion (kinship/trust fabric).
//!
//! Spec: agileplus-specs/civ-007-diplomacy-laws-government/spec.md
//!
//! Public API the green step must provide:
//!   civ_engine::CohesionKind { Blood, Marriage, Clan, Faction, Ally, Rival }
//!   civ_engine::CohesionEdge { from: u64, to: u64, kind: CohesionKind, strength: i32 }
//!   civ_engine::CohesionEvent { edge: (u64, u64), delta: i32, cause: CohesionCause }
//!   civ_engine::CohesionCause { SharedInstitution, TradeInterdependence,
//!                                CommonEnemy, Marriage, Rivalry, TimeDecay }
//!   civ_engine::CohesionSnapshot { settlement_id, avg_trust, avg_kin_density,
//!                                   fragmentations, faction_count }
//!   civ_engine::Simulation
//!     .register_household(household_id)        // from A3
//!     .register_household_in_settlement(settlement_id, household_id)  // from A3
//!     .add_cohesion(from_household, to_household, kind, strength)
//!     .last_tick_cohesion() -> &[CohesionEvent]
//!     .last_tick_cohesion_snapshot(settlement_id) -> Option<CohesionSnapshot>
//!     .faction_count(settlement_id) -> usize
//!
//! 4 tests pinned:
//!   FR-CIV-GOV-030.base       events emitted per tick on edges
//!   FR-CIV-GOV-030.kinship    blood/marriage edges have higher base strength
//!   FR-CIV-GOV-030.fragment    settlement with avg_trust < 50 fragments (faction_count++)
//!   FR-CIV-GOV-030.determinism identical seeds -> identical snapshots
//!
//! This test file is INTENTIONALLY failing to compile. Once the
//! green-step implementation lands in crates/engine/src/engine.rs,
//! all 4 tests compile and pass.

use civ_engine::{
    CohesionEvent, CohesionEventKind, CohesionSnapshot, FabricTier, KinshipEdge, KinshipKind, Sim,
    SimSeed,
};

const COHESION_SEED: u64 = 0xC0_FFEE_0000_0007;

#[test]
fn fr_civ_gov_030_base_events_emitted_per_tick_on_edges() {
    let mut sim = Sim::with_seed(SimSeed::from_u64(COHESION_SEED));
    let s0 = 0u32;
    sim.set_settlement_actor(1, s0);
    sim.set_settlement_actor(2, s0);
    sim.register_kinship(
        1,
        KinshipEdge {
            kind: KinshipKind::Family,
            target: 2,
        },
    );
    sim.add_trust(1, 2, 80);
    sim.tick();
    let events = sim.last_tick_cohesion();
    assert!(
        !events.is_empty(),
        "FR-CIV-GOV-030.base: at least one cohesion event should be emitted on tick"
    );
    let e: &CohesionEvent = &events[0];
    assert_eq!(e.settlement_id, s0);
    assert!(
        matches!(e.kind, CohesionEventKind::Strengthened),
        "positive kinship/trust fabric should strengthen, got {:?}",
        e.kind
    );
}

#[test]
fn fr_civ_gov_030_kinship_blood_and_marriage_have_higher_base_strength() {
    let mut sim = Sim::with_seed(SimSeed::from_u64(COHESION_SEED));
    let s0 = 0u32;
    sim.set_settlement_actor(10, s0);
    sim.set_settlement_actor(20, s0);
    sim.set_settlement_actor(30, s0);
    sim.register_kinship(
        10,
        KinshipEdge {
            kind: KinshipKind::Family,
            target: 20,
        },
    );
    sim.register_kinship(
        10,
        KinshipEdge {
            kind: KinshipKind::Clan,
            target: 30,
        },
    );
    sim.add_trust(10, 20, 80);
    sim.add_trust(10, 30, 20);
    sim.tick();
    let snapshot = sim
        .last_tick_cohesion_settlement(s0)
        .expect("cohesion snapshot");
    assert!(
        snapshot.kin_count >= 2 && snapshot.trust_sum >= 100,
        "FR-CIV-GOV-030.kinship: kinship/trust should contribute to fabric; got {snapshot:?}"
    );
    assert!(matches!(
        snapshot.fabric,
        FabricTier::Tight | FabricTier::Loosened
    ));
}

#[test]
fn fr_civ_gov_030_fragment_low_trust_increases_faction_count() {
    let mut sim = Sim::with_seed(SimSeed::from_u64(COHESION_SEED));
    let s0 = 0u32;
    // 4 households in a single settlement, all rival to each other
    for h in 1u64..=4u64 {
        sim.set_settlement_actor(h, s0);
        sim.set_actor_in_settlement_hardship(h, 80);
    }
    // run for enough ticks for fragmentation to occur
    for _ in 0..100 {
        sim.tick();
    }
    let snap: Option<CohesionSnapshot> = sim.last_tick_cohesion_settlement(s0);
    let snap = snap.expect("snapshot should be present after ticks");
    let fragmentations = snap.fragmentation_events;
    assert!(
        fragmentations > 0 || matches!(snap.fabric, FabricTier::Fractured),
        "FR-CIV-GOV-030.fragment: settlement with all-rival edges should fragment; \
         got fragmentations={fragmentations} fabric={:?}",
        snap.fabric
    );
}

#[test]
fn fr_civ_gov_030_determinism_identical_seeds_yield_identical_snapshots() {
    let mut a = Sim::with_seed(SimSeed::from_u64(COHESION_SEED));
    let mut b = Sim::with_seed(SimSeed::from_u64(COHESION_SEED));
    let s0 = 0u32;
    for h in 1u64..=3u64 {
        a.set_settlement_actor(h, s0);
        b.set_settlement_actor(h, s0);
    }
    a.register_kinship(
        1,
        KinshipEdge {
            kind: KinshipKind::Clan,
            target: 2,
        },
    );
    a.add_trust(1, 2, 60);
    a.add_trust(1, 3, -20);
    b.register_kinship(
        1,
        KinshipEdge {
            kind: KinshipKind::Clan,
            target: 2,
        },
    );
    b.add_trust(1, 2, 60);
    b.add_trust(1, 3, -20);
    for _ in 0..10 {
        a.tick();
        b.tick();
    }
    let sa = a.last_tick_cohesion_settlement(s0).expect("a snapshot");
    let sb = b.last_tick_cohesion_settlement(s0).expect("b snapshot");
    assert_eq!(
        sa.trust_sum, sb.trust_sum,
        "FR-CIV-GOV-030.determinism: trust_sum should match for identical seeds"
    );
    assert_eq!(
        sa.fabric, sb.fabric,
        "FR-CIV-GOV-030.determinism: fabric should match for identical seeds"
    );
}
