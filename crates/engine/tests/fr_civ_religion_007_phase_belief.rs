//! FR-CIV-RELIGION-007 — regression test for `phase_belief` integration.
//!
//! Spec: docs/design/RELIGION_EMERGENCE.md §7 — the per-tick `phase_belief` phase
//! must (a) be dispatched from `Simulation::tick` (no panic / no skip), (b) call
//! `apply_big_gods_response` once per settlement, (c) produce bounded profiles
//! (`monitoring` / `mythic_coherence` / `uncertainty_reduction` in `[0, 1]`),
//! and (d) make those profiles readable through `last_religion_sample()`.

use civ_engine::{
    last_religion_sample, ReligiousProfile, Simulation, SubstrateGradients,
};

#[test]
fn phase_belief_does_not_panic_on_empty_simulation() {
    // Default Simulation has no settlements. phase_belief should be a no-op
    // graceful path — no panic, no events, empty sample.
    let mut sim = Simulation::default();
    sim.tick();
    assert!(
        last_religion_sample(&sim).is_empty(),
        "empty simulation must produce an empty religion sample",
    );
}

#[test]
fn phase_belief_advances_age_for_every_settlement() {
    // Seed three settlements, run one tick, assert every settlement produced
    // a ReligiousProfile whose age_ticks advanced.
    let mut sim = Simulation::default();
    for sid in 1..=3u32 {
        sim.upsert_settlement(sid, /*population=*/ 10_000);
    }
    sim.tick();
    let sample = last_religion_sample(&sim);
    assert_eq!(sample.len(), 3, "every settlement must produce a profile");
    for (sid, profile) in &sample {
        assert_eq!(profile.settlement_id, *sid);
        assert_eq!(
            profile.age_ticks, 1,
            "settlement {sid} profile must show age_ticks=1 after one tick",
        );
    }
}

#[test]
fn phase_belief_keeps_profile_scalars_in_unit_interval() {
    // Property: even after 100 ticks with non-zero substrate gradients,
    // the three religion scalars must stay in [0, 1].
    let mut sim = Simulation::default();
    sim.upsert_settlement(1, 50_000);
    for _ in 0..100 {
        sim.set_substrate_gradients_for(1, SubstrateGradients {
            grad_T: 0.5,
            grad_M: 0.5,
            grad_B: 0.5,
            kinship_density: 0.5,
            unrest: 0.0,
            migration_rate: 0.0,
            language_distance: 0.0,
        });
        sim.tick();
    }
    let (_, profile) = last_religion_sample(&sim).into_iter().next().unwrap();
    assert!((0.0..=1.0).contains(&profile.monitoring), "monitoring out of range: {}", profile.monitoring);
    assert!((0.0..=1.0).contains(&profile.mythic_coherence), "mythic_coherence out of range: {}", profile.mythic_coherence);
    assert!((0.0..=1.0).contains(&profile.uncertainty_reduction), "uncertainty_reduction out of range: {}", profile.uncertainty_reduction);
}

#[test]
fn phase_belief_is_deterministic_given_substrate_snapshot() {
    // Two simulations seeded identically must produce byte-identical profiles.
    let build = || {
        let mut sim = Simulation::default();
        sim.upsert_settlement(7, 12_345);
        sim.set_substrate_gradients_for(7, SubstrateGradients {
            grad_T: 0.3,
            grad_M: 0.7,
            grad_B: 0.1,
            kinship_density: 0.4,
            unrest: 0.2,
            migration_rate: 0.05,
            language_distance: 0.15,
        });
        sim
    };
    let mut a = build();
    let mut b = build();
    a.tick();
    b.tick();
    let pa = &last_religion_sample(&a)[0].1;
    let pb = &last_religion_sample(&b)[0].1;
    assert_eq!(pa.monitoring, pb.monitoring, "monitoring diverged");
    assert_eq!(pa.mythic_coherence, pb.mythic_coherence, "mythic_coherence diverged");
    assert_eq!(pa.uncertainty_reduction, pb.uncertainty_reduction, "uncertainty_reduction diverged");
    assert_eq!(pa.age_ticks, pb.age_ticks, "age_ticks diverged");
}

#[test]
fn phase_belief_persists_profiles_across_ticks() {
    // The religious_profiles map must accumulate — running tick twice should
    // keep age_ticks=2, not reset to 1.
    let mut sim = Simulation::default();
    sim.upsert_settlement(1, 1_000);
    sim.tick();
    sim.tick();
    let (_, profile) = last_religion_sample(&sim).into_iter().next().unwrap();
    assert_eq!(
        profile.age_ticks, 2,
        "profile age_ticks must persist across ticks",
    );
}

#[test]
fn religious_profile_default_satisfies_invariants() {
    // The default ReligiousProfile must already satisfy the REL-INV-1..7 tripwires
    // (scalars in [0, 1], age_ticks=0, no NaN/Inf).
    let p = ReligiousProfile::default();
    assert_eq!(p.age_ticks, 0);
    assert!(p.monitoring.is_finite() && (0.0..=1.0).contains(&p.monitoring));
    assert!(p.mythic_coherence.is_finite() && (0.0..=1.0).contains(&p.mythic_coherence));
    assert!(p.uncertainty_reduction.is_finite() && (0.0..=1.0).contains(&p.uncertainty_reduction));
    assert!(p.population >= 0);
}
