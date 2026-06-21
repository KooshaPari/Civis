//! N13 emergence coupling: language divergence <-> diplomatic tension.
//!
//! FR-CIV-EMERGENCE-N13: language distance between faction populations drives
//! diplomatic tension (upward causation); war/conflict state accelerates
//! language drift by reducing cross-faction contact (downward causation).

use civ_agents::culture::{language_distance, CultureProfile, ContactEdge, drift_populations};
use civ_agents::diplomacy::{DiplomacyMatrix, DiplomacySignal, RelationKind};
use civ_agents::ClusterId;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// N13 contract: language distance d maps to combat_grievance d.
fn language_tension_signal(distance: f32) -> DiplomacySignal {
    DiplomacySignal { combat_grievance: distance, ..Default::default() }
}

// -- N13a: high linguistic distance -> negative diplomatic score --------------

/// N13a nominal: high language distance drives pairwise score below -0.20.
#[test]
fn n13a_high_language_distance_raises_tension() {
    let dist = language_distance([1.0, 0.0, 1.0, 0.0], [0.0, 1.0, 0.0, 1.0]);
    assert!(dist >= 0.7, "requires high distance, got {dist}");

    let mut matrix = DiplomacyMatrix::new();
    let (a, b) = (ClusterId(0), ClusterId(1));
    let mut last = 0.0_f32;
    for _ in 0..15 { last = matrix.apply_signal(a, b, language_tension_signal(dist)).score; }

    assert!(last < -0.20, "15 high-tension steps must push score < -0.20, got {last}");
    let rel = matrix.relation(a, b);
    assert!(
        matches!(rel, RelationKind::Rivalry | RelationKind::War),
        "expected Rivalry or War, got {rel:?}"
    );
}

/// N13a boundary: moderate distance causes less tension than maximum distance.
#[test]
fn n13a_moderate_distance_less_tension_than_maximum() {
    let dist_high = language_distance([1.0, 0.0, 1.0, 0.0], [0.0, 1.0, 0.0, 1.0]);
    let dist_mod = language_distance([0.5, 0.5, 0.5, 0.5], [0.8, 0.2, 0.8, 0.2]);
    assert!(dist_high > dist_mod, "setup: dist_high > dist_mod required");

    let (a, b) = (ClusterId(0), ClusterId(1));
    let (mut mx_h, mut mx_m) = (DiplomacyMatrix::new(), DiplomacyMatrix::new());
    let (mut sh, mut sm) = (0.0_f32, 0.0_f32);
    for _ in 0..10 {
        sh = mx_h.apply_signal(a, b, language_tension_signal(dist_high)).score;
        sm = mx_m.apply_signal(a, b, language_tension_signal(dist_mod)).score;
    }
    assert!(sh < sm, "high distance must produce more tension: high={sh}, mod={sm}");
}

// -- N13b: shared language -> no tension increase ----------------------------

/// N13b: zero language distance produces zero combat_grievance; score stays >= -0.05.
#[test]
fn n13b_identical_language_no_tension_increase() {
    let shared = [0.5_f32, 0.3, 0.7, 0.1];
    let dist = language_distance(shared, shared);
    assert!(dist < 1e-5, "identical language must have zero distance, got {dist}");

    let signal = language_tension_signal(dist);
    assert!(signal.combat_grievance < 1e-5, "zero distance -> zero grievance");

    let (a, b) = (ClusterId(2), ClusterId(3));
    let mut matrix = DiplomacyMatrix::new();
    let mut score = 0.0_f32;
    for _ in 0..10 { score = matrix.apply_signal(a, b, signal).score; }
    assert!(score >= -0.05, "shared language must not produce tension; score={score}");
}

// -- N13c: downward causation -- war kinship increases language divergence ----

/// N13c nominal: high kinship (war isolation) reduces language diffusion,
/// causing faster divergence than a neutral (low kinship) pair.
#[test]
fn n13c_war_kinship_accelerates_language_divergence() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let (sa, sb) = ([0.5_f32, 0.5, 0.5, 0.5], [0.52_f32, 0.48, 0.51, 0.49]);

    let mut neutral = vec![
        CultureProfile { language: sa, kinship: 0.0, ..CultureProfile::new(sa) },
        CultureProfile { language: sb, kinship: 0.0, ..CultureProfile::new(sb) },
    ];
    let mut war = vec![
        CultureProfile { language: sa, kinship: 0.9, ..CultureProfile::new(sa) },
        CultureProfile { language: sb, kinship: 0.9, ..CultureProfile::new(sb) },
    ];
    let edges = vec![
        ContactEdge { from: 0, to: 1, weight: 0.5 },
        ContactEdge { from: 1, to: 0, weight: 0.5 },
    ];
    for _ in 0..20 {
        drift_populations(&mut neutral, &edges, &mut rng, 0.01, 0.3, 0.8);
        drift_populations(&mut war, &edges, &mut rng, 0.01, 0.3, 0.8);
    }
    let nd = language_distance(neutral[0].language, neutral[1].language);
    let wd = language_distance(war[0].language, war[1].language);
    assert!(wd >= nd, "war isolation must produce >= divergence vs neutral: war={wd}, neutral={nd}");
}

/// N13c boundary: kinship=1 (full isolation) must produce >= divergence vs kinship=0.
#[test]
fn n13c_full_isolation_diverges_more_than_full_contact() {
    let mut rng_c = ChaCha8Rng::seed_from_u64(99);
    let mut rng_i = ChaCha8Rng::seed_from_u64(99);
    let (sa, sb) = ([0.4_f32, 0.6, 0.4, 0.6], [0.6_f32, 0.4, 0.6, 0.4]);

    let mut contact = vec![
        CultureProfile { language: sa, kinship: 0.0, ..CultureProfile::new(sa) },
        CultureProfile { language: sb, kinship: 0.0, ..CultureProfile::new(sb) },
    ];
    let mut isolated = vec![
        CultureProfile { language: sa, kinship: 1.0, ..CultureProfile::new(sa) },
        CultureProfile { language: sb, kinship: 1.0, ..CultureProfile::new(sb) },
    ];
    let edges = vec![
        ContactEdge { from: 0, to: 1, weight: 1.0 },
        ContactEdge { from: 1, to: 0, weight: 1.0 },
    ];
    for _ in 0..30 {
        drift_populations(&mut contact, &edges, &mut rng_c, 0.02, 0.5, 0.7);
        drift_populations(&mut isolated, &edges, &mut rng_i, 0.02, 0.5, 0.7);
    }
    let cd = language_distance(contact[0].language, contact[1].language);
    let id = language_distance(isolated[0].language, isolated[1].language);
    assert!(id >= cd, "kinship=1 must produce >= divergence vs kinship=0: isolated={id}, contact={cd}");
}