//! FR-LAW — emergent customary law from recurring disputes.
//!
//! Covers:
//!   FR-LAW-001 — factions accrue customary laws after repeated disputes
//!   FR-LAW-002 — stronger institutions codify emerged laws
//!   FR-LAW-003 — faction law sets surface on `Simulation::snapshot`

use civ_engine::{DisputeKind, Simulation};

const LAW_SEED: u64 = 0;

#[test]
fn fr_law_disputatious_faction_develops_more_laws_than_peaceful() {
    let mut disputatious = Simulation::with_seed(LAW_SEED);
    for _ in 0..10 {
        disputatious.record_faction_dispute(0, DisputeKind::Theft);
        disputatious.record_faction_dispute(0, DisputeKind::Violence);
        disputatious.record_faction_dispute(0, DisputeKind::ResourceDispute);
    }
    disputatious.advance_ticks(1);

    let mut peaceful = Simulation::with_seed(LAW_SEED);
    peaceful.advance_ticks(10);

    let hot = disputatious
        .snapshot()
        .faction_laws
        .get(&0)
        .map(|s| s.laws.len())
        .unwrap_or(0);
    let calm = peaceful
        .snapshot()
        .faction_laws
        .get(&0)
        .map(|s| s.laws.len())
        .unwrap_or(0);

    assert!(
        hot > calm,
        "disputatious faction should develop more laws (hot={hot}) than peaceful (calm={calm})"
    );
    assert_eq!(hot, 3, "expected one customary law per dispute kind");
}

#[test]
fn fr_law_institutions_codify_emerged_norms() {
    let mut sim = Simulation::with_seed(LAW_SEED);
    sim.set_settlement_faction(0, 0);
    sim.set_settlement_population(0, civ_engine::GARRISON_L2_POPULATION + 1);
    for _ in 0..5 {
        sim.record_faction_dispute(0, DisputeKind::Theft);
    }
    sim.advance_ticks(2);

    let snap = sim.snapshot();
    let laws = snap
        .faction_laws
        .get(&0)
        .expect("faction 0 law snapshot");
    assert!(!laws.laws.is_empty(), "theft disputes should yield a law");
    assert!(
        laws.laws.iter().any(|law| law.codified),
        "institutions should codify laws when settlement strength is sufficient"
    );
}
