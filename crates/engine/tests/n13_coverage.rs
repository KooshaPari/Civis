//! N13 emergence coupling: language divergence <-> diplomatic tension.
//!
//! FR-CIV-EMERGENCE-N13: language distance between faction populations drives
//! diplomatic tension (upward causation); war/conflict state accelerates
//! language drift by reducing cross-faction contact (downward causation).
//!
//! Tests define the expected contract so an N13 implementation can be
//! validated against them.

use civ_agents::culture::{language_distance, CultureProfile, ContactEdge, drift_populations};
use civ_agents::diplomacy::{DiplomacyMatrix, DiplomacySignal, RelationKind};
use civ_agents::ClusterId;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// Helper: maps language distance to a diplomacy signal.
// Contract: language distance d contributes d as combat_grievance
// (outgroup threat perception). This is the N13 upward-causation mapping.
fn language_tension_signal(distance: f32) -> DiplomacySignal {
    DiplomacySignal {
        combat_grievance: distance,
        ..Default::default()
    }
}

// -- N13a: high linguistic distance -> negative diplomatic score --------------

/// N13a nominal: high language distance (>=0.8) drives pairwise relation score
/// below -0.20 (Rivalry or worse) after 15 signal steps.
#[test]
fn n13a_high_language_distance_raises_tension() {
    let lang_a = [1.0_f32, 0.0, 1.0, 0.0];
    let lang_b = [0.0_f32, 1.0, 0.0, 1.0];
    let dist = language_distance(lang_a, lang_b);
    assert!(dist >= 0.7, "test requires high linguistic distance, got {dist}");

    let mut matrix = DiplomacyMatrix::new();
    let a = ClusterId(0);
    let b = ClusterId(1);

    let mut last_score = 0.0_f32;
    for _ in 0..15 {
        let outcome = matrix.apply_signal(a, b, language_tension_signal(dist));
        last_score = outcome.score;
    }

    assert!(
        last_score < -0.20,
        "15 steps of high language tension should push score into Rivalry or War, got {last_score}"
    );
    let relation = matrix.relation(a, b);
    assert!(
        matches!(relation, RelationKind::Rivalry | RelationKind::War),
        "expected Rivalry or War, got {relation:?}"
    );
}

/// N13a boundary: moderate language distance produces less tension than maximum
/// after the same number of signal steps.
#[test]
fn n13a_moderate_distance_less_tension_than_maximum() {
    let lang_high_a = [1.0_f32, 0.0, 1.0, 0.0];
    let lang_high_b = [0.0_f32, 1.0, 0.0, 1.0];
    let lang_mod_a = [0.5_f32, 0.5, 0.5, 0.5];
    let lang_mod_b = [0.8_f32, 0.2, 0.8, 0.2];

    let dist_high = language_distance(lang_high_a, lang_high_b);
    let dist_mod = language_distance(lang_mod_a, lang_mod_b);
    assert!(dist_high > dist_mod, "setup: dist_high must exceed dist_mod");

    let mut mx_high = DiplomacyMatrix::new();
    let mut mx_mod = DiplomacyMatrix::new();
    let a = ClusterId(0);
    let b = ClusterId(1);

    let mut score_high = 0.0_f32;
    let mut score_mod = 0.0_f32;
    for _ in 0..10 {
        score_high = mx_high.apply_signal(a, b, language_tension_signal(dist_high)).score;
        score_mod = mx_mod.apply_signal(a, b, language_tension_signal(dist_mod)).score;
    }

    assert!(
        score_high < score_mod,
        "high language distance must produce more tension than moderate: high={score_high}, mod={score_mod}"
    );
}

// -- N13b: shared language -> no tension increase -----------------------------

/// N13b nominal: zero language distance produces zero combat_grievance signal;
/// diplomatic score must not go negative after 10 steps with no other signals.
#[test]
fn n13b_identical_language_no_tension_increase() {
    let shared = [0.5_f32, 0.3, 0.7, 0.1];
    let dist = language_distance(shared, shared);
    assert!(dist < 1e-5, "identical language must have zero distance, got {dist}");

    let signal = language_tension_signal(dist);
    assert!(
        signal.combat_grievance < 1e-5,
        "zero language distance must produce zero combat_grievance, got {}",
        signal.combat_grievance
    );

    let mut matrix = DiplomacyMatrix::new();
    let a = ClusterId(2);
    let b = ClusterId(3);

    let mut last_score = 0.0_f32;
    for _ in 0..10 {
        let outcome = matrix.apply_signal(a, b, signal);
        last_score = outcome.score;
    }

    // Score may decay slightly toward zero via neutral relaxation but must not go negative.
    assert!(
        last_score >= -0.05,
        "shared language must not produce tension; score={last_score}"
    );
}

// -- N13c: downward causation -- war state increases language divergence ------

/// N13c nominal: war-level kinship insulation reduces cross-faction language
/// diffusion, causing faster language drift compared to a neutral (low kinship) pair.
///
/// Mechanism: war -> factions raise kinship barriers -> reduced incoming
/// language diffusion -> populations diverge faster.
#[test]
fn n13c_war_kinship_accelerates_language_divergence() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let seed_a = [0.5_f32, 0.5, 0.5, 0.5];
    let seed_b = [0.52_f32, 0.48, 0.51, 0.49];

    let mut neutral_profiles = vec![
        CultureProfile { language: seed_a, kinship: 0.0, ..CultureProfile::new(seed_a) },
        CultureProfile { language: seed_b, kinship: 0.0, ..CultureProfile::new(seed_b) },
    ];
    let mut war_profiles = vec![
        CultureProfile { language: seed_a, kinship: 0.9, ..CultureProfile::new(seed_a) },
        CultureProfile { language: seed_b, kinship: 0.9, ..CultureProfile::new(seed_b) },
    ];

    let edges = vec![
        ContactEdge { from: 0, to: 1, weight: 0.5 },
        ContactEdge { from: 1, to: 0, weight: 0.5 },
    ];

    for _ in 0..20 {
        drift_populations(&mut neutral_profiles, &edges, &mut rng, 0.01, 0.3, 0.8);
        drift_populations(&mut war_profiles, &edges, &mut rng, 0.01, 0.3, 0.8);
    }

    let neutral_drift = language_distance(neutral_profiles[0].language, neutral_profiles[1].language);
    let war_drift = language_distance(war_profiles[0].language, war_profiles[1].language);

    assert!(
        war_drift >= neutral_drift,
        "war kinship isolation must produce >= language divergence vs neutral: war={war_drift}, neutral={neutral_drift}"
    );
}

/// N13c boundary: kinship=0 (full contact) must produce <= language divergence
/// compared to kinship=1.0 (full isolation) after identical mutation runs.
#[test]
fn n13c_full_isolation_diverges_more_than_full_contact() {
    let mut rng_contact = ChaCha8Rng::seed_from_u64(99);
    let mut rng_isolated = ChaCha8Rng::seed_from_u64(99);

    let seed_a = [0.4_f32, 0.6, 0.4, 0.6];
    let seed_b = [0.6_f32, 0.4, 0.6, 0.4];

    let mut contact_profiles = vec![
        CultureProfile { language: seed_a, kinship: 0.0, ..CultureProfile::new(seed_a) },
        CultureProfile { language: seed_b, kinship: 0.0, ..CultureProfile::new(seed_b) },
    ];
    let mut isolated_profiles = vec![
        CultureProfile { language: seed_a, kinship: 1.0, ..CultureProfile::new(seed_a) },
        CultureProfile { language: seed_b, kinship: 1.0, ..CultureProfile::new(seed_b) },
    ];

    let edges = vec![
        ContactEdge { from: 0, to: 1, weight: 1.0 },
        ContactEdge { from: 1, to: 0, weight: 1.0 },
    ];

    for _ in 0..30 {
        drift_populations(&mut contact_profiles, &edges, &mut rng_contact, 0.02, 0.5, 0.7);
        drift_populations(&mut isolated_profiles, &edges, &mut rng_isolated, 0.02, 0.5, 0.7);
    }

    let contact_dist = language_distance(contact_profiles[0].language, contact_profiles[1].language);
    let isolated_dist = language_distance(isolated_profiles[0].language, isolated_profiles[1].language);

    assert!(
        isolated_dist >= contact_dist,
        "kinship=1 isolation must produce >= language divergence vs kinship=0 contact: isolated={isolated_dist}, contact={contact_dist}"
    );
}