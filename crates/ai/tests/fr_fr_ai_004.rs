//! FR-AI-004 — Personality drift SHALL accumulate stochastically each N ticks.

#[test]
fn drift_accumulates_stochastically() {
    use civ_ai::personality::{PersonalityDrift, PersonalityKind, PersonalityProfile};

    let profile = PersonalityProfile::from_kind(PersonalityKind::Balanced);
    let drift = PersonalityDrift::new(10, 0.1); // every 10 ticks, max 0.1 change

    // Apply drift at a non-drift tick — should be unchanged.
    let unchanged = drift.apply(&profile, 5, 0.5);
    assert_eq!(unchanged, profile, "no drift at non-interval tick");

    // Apply drift at a drift tick with positive rng → multipliers increase.
    let drifted_pos = drift.apply(&profile, 10, 1.0);
    assert!(drifted_pos.resource_mult > profile.resource_mult,
        "positive drift should increase resource_mult");
    assert!(drifted_pos.strategic_mult > profile.strategic_mult);
    assert!(drifted_pos.safety_mult > profile.safety_mult);
    assert!(drifted_pos.diplomatic_mult > profile.diplomatic_mult);

    // Apply drift at a drift tick with negative rng → multipliers decrease.
    let drifted_neg = drift.apply(&profile, 20, -1.0);
    assert!(drifted_neg.resource_mult < profile.resource_mult,
        "negative drift should decrease resource_mult");
    assert!(drifted_neg.strategic_mult < profile.strategic_mult);
    assert!(drifted_neg.safety_mult < profile.safety_mult);
    assert!(drifted_neg.diplomatic_mult < profile.diplomatic_mult);
}

#[test]
fn drift_clamps_to_bounds() {
    use civ_ai::personality::{PersonalityDrift, PersonalityKind, PersonalityProfile};

    let mut profile = PersonalityProfile::from_kind(PersonalityKind::Balanced);
    // Push resource_mult near the lower bound.
    profile.resource_mult = 0.15;

    let drift = PersonalityDrift::new(1, 0.1); // every tick
    let result = drift.apply(&profile, 1, -1.0);

    // After drift: 0.15 + (-0.1) = 0.05, but clamped to 0.1.
    assert!(result.resource_mult >= 0.1,
        "resource_mult ({}) should be clamped to >= 0.1", result.resource_mult);
    assert!(result.resource_mult <= 3.0);
}

#[test]
fn repeated_drift_accumulates() {
    use civ_ai::personality::{PersonalityDrift, PersonalityKind, PersonalityProfile};

    let profile = PersonalityProfile::from_kind(PersonalityKind::Balanced);
    let drift = PersonalityDrift::new(1, 0.05); // every tick

    let mut current = profile.clone();
    for tick in 1..=10 {
        current = drift.apply(&current, tick, 1.0);
    }
    // After 10 ticks of positive drift (0.05 each): 1.0 + 10 * 0.05 = 1.5.
    assert!((current.resource_mult - 1.5).abs() < 1e-10,
        "expected ~1.5, got {}", current.resource_mult);
}
