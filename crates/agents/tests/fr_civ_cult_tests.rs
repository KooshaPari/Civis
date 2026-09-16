//! FR traceability tests for FR-CIV-CULT-001, FR-CIV-CULT-002, FR-CIV-CULT-003.
//!
//! These integration tests exercise the public culture API in `civ-agents`
//! to close the IMPL-NO-TEST gap identified in the traceability matrix.
//!
//! - FR-CIV-CULT-001: Culture system (entity creation, identity, defaults)
//! - FR-CIV-CULT-002: Cultural traits (diffusion mechanics, distance metric)
//! - FR-CIV-CULT-003: Cultural identity (ideology convergence, divergence over isolation)

use civ_agents::culture::{
    cultural_distance, language_divergence_from_isolation, ContactEdge,
    CultureProfile, TraitVector,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn seeded_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

// ===========================================================================
// FR-CIV-CULT-001: Culture system (entity + identity)
// ===========================================================================

/// Covers FR-CIV-CULT-001.
/// FR-CIV-CULT-001 — CultureProfile::new() initializes traits and language
/// from the provided seed vector, establishing the culture entity's identity.
#[test]
fn cult001_new_profile_initializes_traits_from_seed() {
    let seed: TraitVector = [0.1, 0.3, 0.7, 0.9];
    let profile = CultureProfile::new(seed);
    assert_eq!(
        profile.traits, seed,
        "traits must match the seed vector"
    );
    assert_eq!(
        profile.language, seed,
        "language must initially mirror the seed vector"
    );
}

/// Covers FR-CIV-CULT-001.
/// FR-CIV-CULT-001 — A newly created culture profile starts with zero
/// contact and kinship, representing an isolated population before any
/// diffusion step.
#[test]
fn cult001_new_profile_defaults_contact_and_kinship_to_zero() {
    let profile = CultureProfile::new([0.5; 4]);
    assert_eq!(profile.contact, 0.0, "contact should default to 0.0");
    assert_eq!(profile.kinship, 0.0, "kinship should default to 0.0");
}

/// Covers FR-CIV-CULT-001.
/// FR-CIV-CULT-001 — Two profiles with different seed vectors have distinct
/// identities (different traits), confirming each culture entity is unique.
#[test]
fn cult001_different_seeds_produce_distinct_profiles() {
    let a = CultureProfile::new([0.0, 0.0, 0.0, 0.0]);
    let b = CultureProfile::new([1.0, 1.0, 1.0, 1.0]);
    assert_ne!(a.traits, b.traits, "different seeds must yield different traits");
    assert_ne!(
        a.language, b.language,
        "different seeds must yield different language vectors"
    );
    assert!(
        cultural_distance(a.traits, b.traits) > 0.5,
        "different seeds should have high cultural distance"
    );
}

// ===========================================================================
// FR-CIV-CULT-002: Cultural traits (diffusion mechanics)
// ===========================================================================

/// Covers FR-CIV-CULT-002.
/// FR-CIV-CULT-002 — Cultural distance between identical trait vectors is
/// zero, confirming the identity property of the distance metric.
#[test]
fn cult002_distance_identical_vectors_is_zero() {
    let a: TraitVector = [0.5, 0.5, 0.5, 0.5];
    let b: TraitVector = [0.5, 0.5, 0.5, 0.5];
    let d = cultural_distance(a, b);
    assert!(
        d.abs() < 1e-6,
        "distance between identical vectors should be 0, got {d}"
    );
}

/// Covers FR-CIV-CULT-002.
/// FR-CIV-CULT-002 — Cultural distance between maximally different vectors
/// ([0,0,0,0] vs [1,1,1,1]) equals 1.0, confirming the metric is bounded
/// to [0, 1].
#[test]
fn cult002_distance_maximally_different_is_one() {
    let a: TraitVector = [0.0, 0.0, 0.0, 0.0];
    let b: TraitVector = [1.0, 1.0, 1.0, 1.0];
    let d = cultural_distance(a, b);
    assert!(
        (d - 1.0).abs() < 1e-6,
        "distance between [0,0,0,0] and [1,1,1,1] should be 1.0, got {d}"
    );
}

/// Covers FR-CIV-CULT-002.
/// FR-CIV-CULT-002 — Cultural distance is symmetric: d(a, b) == d(b, a).
#[test]
fn cult002_distance_is_symmetric() {
    let a: TraitVector = [0.2, 0.8, 0.3, 0.6];
    let b: TraitVector = [0.9, 0.1, 0.7, 0.4];
    let d_ab = cultural_distance(a, b);
    let d_ba = cultural_distance(b, a);
    assert!(
        (d_ab - d_ba).abs() < 1e-6,
        "distance must be symmetric: d(a,b)={d_ab}, d(b,a)={d_ba}"
    );
}

/// Covers FR-CIV-CULT-002.
/// FR-CIV-CULT-002 — Drift populations with contact edges converge their
/// cultural traits: after diffusion, cultural distance between connected
/// populations decreases.
#[test]
fn cult002_contact_drift_converges_cultural_traits() {
    let mut rng = seeded_rng(42);
    let mut profiles = vec![
        CultureProfile::new([0.0, 0.0, 0.0, 0.0]),
        CultureProfile::new([1.0, 1.0, 1.0, 1.0]),
    ];
    let dist_before = cultural_distance(profiles[0].traits, profiles[1].traits);

    let edges = [ContactEdge {
        from: 0,
        to: 1,
        weight: 1.0,
    }];
    civ_agents::culture::drift_populations(
        &mut profiles,
        &edges,
        &mut rng,
        0.0,   // zero mutation so only diffusion acts
        0.5,   // moderate diffusion rate
        0.25,  // creole threshold
    );

    let dist_after = cultural_distance(profiles[0].traits, profiles[1].traits);
    assert!(
        dist_after < dist_before,
        "connected populations should converge: dist before={dist_before}, after={dist_after}"
    );
}

/// Covers FR-CIV-CULT-002.
/// FR-CIV-CULT-002 — Kinship resistance reduces the effective contact
/// intensity: a high-kinship profile absorbs less cultural influence.
#[test]
fn cult002_kinship_resists_cultural_diffusion() {
    let mut rng_low_kin = seeded_rng(77);
    let mut rng_high_kin = seeded_rng(77);

    // Profile A with no kinship resistance.
    let mut profiles_no_resist = vec![
        CultureProfile {
            traits: [0.0, 0.0, 0.0, 0.0],
            language: [0.0, 0.0, 0.0, 0.0],
            phonemes: civ_agents::PhonemeInventory::from_trait_seed([0.0; 4]),
            contact: 0.0,
            kinship: 0.0, // no resistance
        },
        CultureProfile::new([1.0, 1.0, 1.0, 1.0]),
    ];

    // Profile A with high kinship resistance.
    let mut profiles_high_resist = vec![
        CultureProfile {
            traits: [0.0, 0.0, 0.0, 0.0],
            language: [0.0, 0.0, 0.0, 0.0],
            phonemes: civ_agents::PhonemeInventory::from_trait_seed([0.0; 4]),
            contact: 0.0,
            kinship: 0.9, // strong resistance
        },
        CultureProfile::new([1.0, 1.0, 1.0, 1.0]),
    ];

    let edges = [ContactEdge {
        from: 1,
        to: 0,
        weight: 1.0,
    }];

    civ_agents::culture::drift_populations(
        &mut profiles_no_resist,
        &edges,
        &mut rng_low_kin,
        0.0,
        1.0,
        0.25,
    );
    civ_agents::culture::drift_populations(
        &mut profiles_high_resist,
        &edges,
        &mut rng_high_kin,
        0.0,
        1.0,
        0.25,
    );

    let shift_no_resist = cultural_distance(
        profiles_no_resist[0].traits,
        [0.0, 0.0, 0.0, 0.0],
    );
    let shift_high_resist = cultural_distance(
        profiles_high_resist[0].traits,
        [0.0, 0.0, 0.0, 0.0],
    );

    assert!(
        shift_high_resist < shift_no_resist,
        "high kinship should resist diffusion: shift_no_resist={shift_no_resist}, \
         shift_high_resist={shift_high_resist}"
    );
}

// ===========================================================================
// FR-CIV-CULT-003: Cultural identity (ideology convergence / divergence)
// ===========================================================================

/// Covers FR-CIV-CULT-003.
/// FR-CIV-CULT-003 — Language divergence increases monotonically with
/// isolation ticks, confirming that isolated populations diverge over time.
#[test]
fn cult003_divergence_monotone_in_isolation_ticks() {
    let a = CultureProfile::new([0.3, 0.4, 0.3, 0.4]);
    let b = CultureProfile::new([0.7, 0.6, 0.7, 0.6]);
    let ticks = [0u32, 10, 50, 200, 1000];
    let mut prev = 0.0f32;
    for &t in &ticks {
        let d = language_divergence_from_isolation(&a, &b, t);
        assert!(
            d >= prev - 1e-6,
            "divergence must be non-decreasing at ticks={t}: got {d}, prev was {prev}"
        );
        prev = d;
    }
    assert!(
        prev > 0.0,
        "long isolation should produce strictly positive divergence"
    );
}

/// Covers FR-CIV-CULT-003.
/// FR-CIV-CULT-003 — Identical cultures have zero divergence at zero
/// isolation ticks, confirming the identity axiom of cultural convergence.
#[test]
fn cult003_identical_cultures_no_divergence_at_zero_isolation() {
    let a = CultureProfile::new([0.5, 0.5, 0.5, 0.5]);
    let b = CultureProfile::new([0.5, 0.5, 0.5, 0.5]);
    let d = language_divergence_from_isolation(&a, &b, 0);
    assert_eq!(
        d, 0.0,
        "identical cultures should have zero divergence at t=0"
    );
}

/// Covers FR-CIV-CULT-003.
/// FR-CIV-CULT-003 — Culturally distant populations have higher baseline
/// language divergence than culturally close ones at the same isolation level,
/// confirming that cultural distance drives identity divergence.
#[test]
fn cult003_divergence_correlates_with_cultural_distance() {
    let near_a = CultureProfile::new([0.5, 0.5, 0.5, 0.5]);
    let near_b = CultureProfile::new([0.52, 0.48, 0.51, 0.49]);
    let far_a = CultureProfile::new([0.0, 0.0, 0.0, 0.0]);
    let far_b = CultureProfile::new([1.0, 1.0, 1.0, 1.0]);

    let near_div = language_divergence_from_isolation(&near_a, &near_b, 100);
    let far_div = language_divergence_from_isolation(&far_a, &far_b, 100);
    assert!(
        far_div > near_div,
        "culturally distant pairs must have higher divergence: far={far_div}, near={near_div}"
    );
}

/// Covers FR-CIV-CULT-003.
/// FR-CIV-CULT-003 — Divergence is bounded in [0, 1] for all isolation
/// tick values, confirming the convergence metric is well-formed.
#[test]
fn cult003_divergence_bounded_in_unit_interval() {
    let a = CultureProfile::new([0.0, 0.0, 0.0, 0.0]);
    let b = CultureProfile::new([1.0, 1.0, 1.0, 1.0]);
    for ticks in [0u32, 1, 50, 200, 1000] {
        let d = language_divergence_from_isolation(&a, &b, ticks);
        assert!(
            (0.0..=1.0).contains(&d),
            "divergence must be in [0,1] at ticks={ticks}, got {d}"
        );
    }
}
