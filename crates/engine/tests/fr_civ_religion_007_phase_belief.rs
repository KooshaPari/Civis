//! FR-CIV-REL-007 regression coverage for the public belief helpers.

use civ_engine::religion::{
    apply_big_gods_response, last_religion_sample, substrate_gradients_for, ReligiousProfile,
    SubstrateGradients,
};
use civ_engine::Simulation;

#[test]
fn fr_civ_religion_007_response_clamps_profile_scalars() {
    let mut profile = ReligiousProfile {
        monitoring: 0.99,
        mythic_coherence: 0.99,
        uncertainty_reduction: 0.99,
        ..ReligiousProfile::default()
    };
    let gradients = SubstrateGradients {
        grad_T: 10.0,
        grad_B: 10.0,
        unrest: 10.0,
        ..SubstrateGradients::default()
    };

    apply_big_gods_response(&mut profile, &gradients, 42);

    assert_eq!(profile.created_tick, 42);
    assert!((0.0..=1.0).contains(&profile.monitoring));
    assert!((0.0..=1.0).contains(&profile.mythic_coherence));
    assert!((0.0..=1.0).contains(&profile.uncertainty_reduction));
}

#[test]
fn fr_civ_religion_007_response_is_deterministic() {
    let gradients = SubstrateGradients {
        grad_T: 0.5,
        grad_M: 0.3,
        grad_B: 0.7,
        kinship_density: 0.6,
        unrest: 0.2,
        migration_rate: 0.1,
        language_distance: 0.4,
    };
    let mut first = ReligiousProfile::default();
    let mut second = ReligiousProfile::default();

    apply_big_gods_response(&mut first, &gradients, 42);
    apply_big_gods_response(&mut second, &gradients, 42);

    assert_eq!(first, second);
}

#[test]
fn fr_civ_religion_007_substrate_and_sample_preserve_settlement_identity() {
    let gradients = substrate_gradients_for(7);
    let sample = last_religion_sample(7);

    assert!(gradients.grad_T.is_finite());
    assert!(gradients.grad_M.is_finite());
    assert!(gradients.grad_B.is_finite());
    assert_eq!(sample.settlement_id, 7);
}

#[test]
fn fr_civ_religion_007_seeded_simulation_ticks_before_phase_wiring_returns() {
    let mut sim = Simulation::with_seed(0x0E1_007);
    let before = sim.snapshot().tick;

    sim.tick();

    assert_eq!(sim.snapshot().tick, before + 1);
}
