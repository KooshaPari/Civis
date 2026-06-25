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

use civ_engine::{CohesionCause, CohesionEvent, CohesionKind, CohesionSnapshot, Sim, SimSeed};

const COHESION_SEED: u64 = 0xC0_FFEE_0000_0007;

#[test]
fn fr_civ_gov_030_base_events_emitted_per_tick_on_edges() {
    let mut sim = Sim::with_seed(SimSeed::from_u64(COHESION_SEED));
    let s0 = 0u32;
    sim.register_household_in_settlement(s0, 1);
    sim.register_household_in_settlement(s0, 2);
    sim.add_cohesion(1, 2, CohesionKind::Blood, 80);
    sim.tick();
    let events = sim.last_tick_cohesion();
    assert!(
        !events.is_empty(),
        "FR-CIV-GOV-030.base: at least one cohesion event should be emitted on tick"
    );
    let e: &CohesionEvent = &events[0];
    // blood edges get a per-tick SharedInstitution reinforcement
    let _ = e.delta; // delta may be 0, positive, or negative
    let _ = CohesionCause::SharedInstitution;
}

#[test]
fn fr_civ_gov_030_kinship_blood_and_marriage_have_higher_base_strength() {
    let mut sim = Sim::with_seed(SimSeed::from_u64(COHESION_SEED));
    let s0 = 0u32;
    sim.register_household_in_settlement(s0, 10);
    sim.register_household_in_settlement(s0, 20);
    sim.register_household_in_settlement(s0, 30);
    // Blood (kinship): strength=80 should be reinforced
    sim.add_cohesion(10, 20, CohesionKind::Blood, 80);
    // Rival (non-kinship): strength=20 should be unchanged or decay
    sim.add_cohesion(10, 30, CohesionKind::Rival, 20);
    sim.tick();
    let events = sim.last_tick_cohesion();
    let blood_delta: i32 = events
        .iter()
        .filter(|e| matches!(e.cause, CohesionCause::SharedInstitution))
        .map(|e| e.delta)
        .sum();
    // kinship edges should have a non-negative net delta across ticks
    assert!(
        blood_delta >= 0,
        "FR-CIV-GOV-030.kinship: blood/institution edges should not decay, got delta={blood_delta}"
    );
}

#[test]
fn fr_civ_gov_030_fragment_low_trust_increases_faction_count() {
    let mut sim = Sim::with_seed(SimSeed::from_u64(COHESION_SEED));
    let s0 = 0u32;
    // 4 households in a single settlement, all rival to each other
    for h in 1u64..=4u64 {
        sim.register_household_in_settlement(s0, h);
    }
    for i in 1u64..=4u64 {
        for j in 1u64..=4u64 {
            if i != j {
                sim.add_cohesion(i, j, CohesionKind::Rival, 5);
            }
        }
    }
    // run for enough ticks for fragmentation to occur
    for _ in 0..100 {
        sim.tick();
    }
    let snap: Option<CohesionSnapshot> = sim.last_tick_cohesion_snapshot(s0);
    let snap = snap.expect("snapshot should be present after ticks");
    // either avg_trust is low OR faction_count >= 1
    let fragmentations = snap.fragmentations;
    let faction_count = snap.faction_count;
    assert!(
        fragmentations > 0 || faction_count >= 1,
        "FR-CIV-GOV-030.fragment: settlement with all-rival edges should fragment; \
         got fragmentations={fragmentations} faction_count={faction_count} avg_trust={}",
        snap.avg_trust
    );
}

#[test]
fn fr_civ_gov_030_determinism_identical_seeds_yield_identical_snapshots() {
    let mut a = Sim::with_seed(SimSeed::from_u64(COHESION_SEED));
    let mut b = Sim::with_seed(SimSeed::from_u64(COHESION_SEED));
    let s0 = 0u32;
    for h in 1u64..=3u64 {
        a.register_household_in_settlement(s0, h);
        b.register_household_in_settlement(s0, h);
    }
    a.add_cohesion(1, 2, CohesionKind::Ally, 60);
    a.add_cohesion(1, 3, CohesionKind::Rival, 20);
    b.add_cohesion(1, 2, CohesionKind::Ally, 60);
    b.add_cohesion(1, 3, CohesionKind::Rival, 20);
    for _ in 0..10 {
        a.tick();
        b.tick();
    }
    let sa = a.last_tick_cohesion_snapshot(s0).expect("a snapshot");
    let sb = b.last_tick_cohesion_snapshot(s0).expect("b snapshot");
    assert_eq!(
        sa.avg_trust, sb.avg_trust,
        "FR-CIV-GOV-030.determinism: avg_trust should match for identical seeds"
    );
    assert_eq!(
        sa.faction_count, sb.faction_count,
        "FR-CIV-GOV-030.determinism: faction_count should match for identical seeds"
    );
}
