//! Emergent culture and language drift across population networks.
//!
//! Culture is represented as a small meme/trait vector that mutates and
//! diffuses through kinship/contact networks. Language is modeled as a
//! separate drift vector so isolated populations diverge while contact zones
//! creolize.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::language::{drift_phonemes, phoneme_inventory_distance, PhonemeInventory};

/// Fixed-size cultural meme vector.
///
/// Values are normalized to `[0, 1]` and are intentionally generic: the
/// emergent content comes from repeated mutation, diffusion, and contact.
pub type TraitVector = [f32; 4];

/// Population-level cultural state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CultureProfile {
    /// Cultural trait vector.
    pub traits: TraitVector,
    /// Language drift vector. Different enough vectors represent distinct
    /// dialects or languages.
    pub language: TraitVector,
    /// Drifted phoneme inventory for this cluster dialect.
    pub phonemes: PhonemeInventory,
    /// How much contact this population has with others in the current step.
    pub contact: f32,
    /// How much kinship insulation this population has. Higher values reduce
    /// drift from outside contact.
    pub kinship: f32,
}

impl CultureProfile {
    /// Constructs a profile with the same starting state for culture and
    /// language.
    pub fn new(seed: TraitVector) -> Self {
        Self {
            traits: seed,
            language: seed,
            phonemes: PhonemeInventory::from_trait_seed(seed),
            contact: 0.0,
            kinship: 0.0,
        }
    }
}

/// Contact graph edge between populations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ContactEdge {
    /// Source population index.
    pub from: usize,
    /// Target population index.
    pub to: usize,
    /// Contact intensity in `[0, 1]`.
    pub weight: f32,
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn mix_trait_vectors(a: TraitVector, b: TraitVector, weight: f32) -> TraitVector {
    let w = clamp01(weight);
    let mut out = [0.0; 4];
    for i in 0..4 {
        out[i] = clamp01(a[i] * (1.0 - w) + b[i] * w);
    }
    out
}

/// Mutates a cultural vector by small random jitter.
pub fn mutate_traits(rng: &mut impl Rng, traits: TraitVector, mutation_rate: f32) -> TraitVector {
    let rate = clamp01(mutation_rate);
    let mut out = traits;
    for value in &mut out {
        let delta = (rng.gen::<f32>() - 0.5) * 2.0 * rate;
        *value = clamp01(*value + delta);
    }
    out
}

/// Returns a normalized cultural distance in `[0, 1]`.
pub fn cultural_distance(a: TraitVector, b: TraitVector) -> f32 {
    let mut sum = 0.0;
    for i in 0..4 {
        let d = a[i] - b[i];
        sum += d * d;
    }
    (sum / 4.0).sqrt().min(1.0)
}

/// Returns a normalized language distance in `[0, 1]` from trait vectors alone.
pub fn language_distance(a: TraitVector, b: TraitVector) -> f32 {
    cultural_distance(a, b)
}

/// Cluster divergence: cultural distance drives language distance (FR-CIV-LANG).
#[must_use]
pub fn cluster_language_distance(a: &CultureProfile, b: &CultureProfile) -> f32 {
    let cult = cultural_distance(a.traits, b.traits);
    let phon = phoneme_inventory_distance(&a.phonemes, &b.phonemes);
    let raw = language_distance(a.language, b.language);
    clamp01((cult * 0.5 + phon * 0.3 + raw * 0.2).max(cult * 0.7))
}

/// Language divergence accelerated by isolation (FR-CIV-LANG-003 / FR-CIV-LANG-005).
///
/// Models how isolated clusters accumulate linguistic divergence beyond what cultural
/// drift alone produces: `isolation_ticks` acts as a multiplier on baseline distance so
/// populations that were once in contact and then separated grow further apart over time.
/// Returns a value in `[0, 1]`.
#[must_use]
pub fn language_divergence_from_isolation(
    a: &CultureProfile,
    b: &CultureProfile,
    isolation_ticks: u32,
) -> f32 {
    let base = cluster_language_distance(a, b);
    // Isolation accumulator: saturates at 1.0; half-saturation at 200 ticks.
    let isolation_factor = isolation_ticks as f32 / (isolation_ticks as f32 + 200.0);
    clamp01(base + (1.0 - base) * isolation_factor * 0.5)
}

/// Diffuses culture and language across a contact graph for one step.
///
/// Isolated populations drift through mutation. Contact-linked populations
/// exchange culture, and sufficiently divergent languages creolize by blending
/// toward one another.
pub fn drift_populations(
    profiles: &mut [CultureProfile],
    edges: &[ContactEdge],
    rng: &mut impl Rng,
    mutation_rate: f32,
    diffusion_rate: f32,
    creole_threshold: f32,
) {
    if profiles.is_empty() {
        return;
    }
    let base = profiles.to_vec();
    if edges.is_empty() {
        for (idx, profile) in profiles.iter_mut().enumerate() {
            profile.traits = mutate_traits(rng, base[idx].traits, mutation_rate);
            profile.language = mutate_traits(rng, base[idx].language, mutation_rate * 0.5);
            let _ = drift_phonemes(rng, &mut profile.phonemes, mutation_rate * 0.4, 50);
            profile.contact = 0.0;
        }
        return;
    }
    let mut incoming: Vec<Vec<(usize, f32)>> = vec![Vec::new(); profiles.len()];
    for edge in edges {
        if edge.from < profiles.len() && edge.to < profiles.len() {
            incoming[edge.to].push((edge.from, clamp01(edge.weight)));
        }
    }

    for (idx, profile) in profiles.iter_mut().enumerate() {
        let mut traits = mutate_traits(rng, base[idx].traits, mutation_rate);
        let mut language = mutate_traits(rng, base[idx].language, mutation_rate * 0.5);
        let mut total_weight = 0.0;

        for &(source, weight) in &incoming[idx] {
            let adjusted = clamp01(weight * (1.0 - base[idx].kinship));
            if adjusted <= 0.0 {
                continue;
            }
            total_weight += adjusted;
            traits = mix_trait_vectors(traits, base[source].traits, adjusted * diffusion_rate);
            language =
                mix_trait_vectors(language, base[source].language, adjusted * diffusion_rate);
        }

        if total_weight == 0.0 {
            // Isolated populations still drift, but only through mutation.
            profile.contact = 0.0;
        } else {
            profile.contact = clamp01(total_weight);
        }

        profile.traits = traits;
        profile.language = language;
        let _ = drift_phonemes(rng, &mut profile.phonemes, mutation_rate * 0.4, 50);
    }

    // Creolization is a second pass so contact zones blend the already-drifted
    // language state rather than the source state directly.
    let post = profiles.to_vec();
    for edge in edges {
        if edge.from >= profiles.len() || edge.to >= profiles.len() {
            continue;
        }
        let a = edge.from;
        let b = edge.to;
        let weight = clamp01(edge.weight);
        if weight <= 0.0 {
            continue;
        }

        let distance = language_distance(post[a].language, post[b].language);
        if distance >= creole_threshold {
            let blend = clamp01((weight + distance) * 0.5);
            profiles[a].language =
                mix_trait_vectors(post[a].language, post[b].language, blend * 0.5);
            profiles[b].language =
                mix_trait_vectors(post[b].language, post[a].language, blend * 0.5);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    #[test]
    fn mutation_moves_traits_but_keeps_bounds() {
        let mut rng = rng(1);
        let traits = [0.5, 0.5, 0.5, 0.5];
        let mutated = mutate_traits(&mut rng, traits, 0.2);
        assert!(mutated.iter().all(|v| (0.0..=1.0).contains(v)));
        assert_ne!(traits, mutated);
    }

    #[test]
    fn isolated_populations_diverge_over_time() {
        let mut rng = rng(7);
        let mut profiles = vec![
            CultureProfile::new([0.1, 0.1, 0.1, 0.1]),
            CultureProfile::new([0.9, 0.9, 0.9, 0.9]),
        ];
        for _ in 0..32 {
            drift_populations(&mut profiles, &[], &mut rng, 0.05, 0.25, 0.6);
        }
        assert_ne!(profiles[0].traits, profiles[1].traits);
        assert_ne!(profiles[0].language, profiles[1].language);
    }

    #[test]
    fn contact_diffuses_culture_and_creolizes_language() {
        let mut rng = rng(11);
        let mut profiles = vec![
            CultureProfile::new([0.0, 0.0, 0.0, 0.0]),
            CultureProfile::new([1.0, 1.0, 1.0, 1.0]),
        ];
        let edges = [ContactEdge {
            from: 0,
            to: 1,
            weight: 1.0,
        }];

        drift_populations(&mut profiles, &edges, &mut rng, 0.0, 0.75, 0.25);

        let culture_gap = cultural_distance(profiles[0].traits, profiles[1].traits);
        let language_gap = language_distance(profiles[0].language, profiles[1].language);
        assert!(culture_gap < 1.0);
        assert!(language_gap < 1.0);
    }

    #[test]
    fn kinship_resists_external_contact() {
        let mut rng = rng(23);
        let mut profiles = vec![
            CultureProfile {
                traits: [0.0, 0.0, 0.0, 0.0],
                language: [0.0, 0.0, 0.0, 0.0],
                contact: 0.0,
                kinship: 0.95,
                ..CultureProfile::new([0.0, 0.0, 0.0, 0.0])
            },
            CultureProfile {
                traits: [1.0, 1.0, 1.0, 1.0],
                language: [1.0, 1.0, 1.0, 1.0],
                contact: 0.0,
                kinship: 0.0,
                ..CultureProfile::new([1.0, 1.0, 1.0, 1.0])
            },
        ];
        let edges = [ContactEdge {
            from: 1,
            to: 0,
            weight: 1.0,
        }];

        drift_populations(&mut profiles, &edges, &mut rng, 0.0, 1.0, 0.25);

        assert!(cultural_distance(profiles[0].traits, [0.0, 0.0, 0.0, 0.0]) < 0.2);
        assert!(profiles[0].contact < 0.2);
    }

    #[test]
    fn cluster_language_distance_tracks_cultural_divergence() {
        let near_a = CultureProfile::new([0.5, 0.5, 0.5, 0.5]);
        let near_b = CultureProfile::new([0.52, 0.48, 0.51, 0.49]);
        let far_a = CultureProfile::new([0.0, 0.0, 0.0, 0.0]);
        let far_b = CultureProfile::new([1.0, 1.0, 1.0, 1.0]);
        let near = cluster_language_distance(&near_a, &near_b);
        let far = cluster_language_distance(&far_a, &far_b);
        assert!(far > near, "cultural divergence must raise language distance");
    }

    #[test]
    fn phoneme_vectors_diverge_with_isolated_drift() {
        let mut rng = rng(31);
        let mut profiles = vec![
            CultureProfile::new([0.2, 0.2, 0.2, 0.2]),
            CultureProfile::new([0.8, 0.8, 0.8, 0.8]),
        ];
        for _ in 0..24 {
            drift_populations(&mut profiles, &[], &mut rng, 0.06, 0.0, 0.85);
        }
        assert_ne!(profiles[0].phonemes, profiles[1].phonemes);
    }

    // --- Sub-feature 3: cluster language divergence mirrors culture/religion pattern ---

    #[test]
    fn language_divergence_rises_with_isolation_ticks() {
        let a = CultureProfile::new([0.3, 0.4, 0.3, 0.4]);
        let b = CultureProfile::new([0.7, 0.6, 0.7, 0.6]);
        let dist_0 = language_divergence_from_isolation(&a, &b, 0);
        let dist_100 = language_divergence_from_isolation(&a, &b, 100);
        let dist_500 = language_divergence_from_isolation(&a, &b, 500);
        assert!(
            dist_500 >= dist_100 && dist_100 >= dist_0,
            "language divergence must be monotone in isolation_ticks"
        );
        assert!(dist_500 > dist_0, "long isolation must produce strictly more divergence");
    }

    #[test]
    fn language_divergence_bounded_in_unit_interval() {
        let a = CultureProfile::new([0.0, 0.0, 0.0, 0.0]);
        let b = CultureProfile::new([1.0, 1.0, 1.0, 1.0]);
        for ticks in [0u32, 1, 50, 200, 1000, u32::MAX / 2] {
            let d = language_divergence_from_isolation(&a, &b, ticks);
            assert!((0.0..=1.0).contains(&d), "divergence must be in [0,1] at ticks={ticks}");
        }
    }

    #[test]
    fn language_divergence_correlates_with_cultural_distance() {
        // Near pair vs far pair at same isolation
        let near_a = CultureProfile::new([0.5, 0.5, 0.5, 0.5]);
        let near_b = CultureProfile::new([0.52, 0.48, 0.51, 0.49]);
        let far_a = CultureProfile::new([0.0, 0.0, 0.0, 0.0]);
        let far_b = CultureProfile::new([1.0, 1.0, 1.0, 1.0]);
        let near_div = language_divergence_from_isolation(&near_a, &near_b, 100);
        let far_div = language_divergence_from_isolation(&far_a, &far_b, 100);
        assert!(
            far_div > near_div,
            "culturally distant clusters must have larger language divergence"
        );
    }

    #[test]
    fn identical_clusters_no_divergence_without_drift() {
        let a = CultureProfile::new([0.5, 0.5, 0.5, 0.5]);
        let b = CultureProfile::new([0.5, 0.5, 0.5, 0.5]);
        let d0 = language_divergence_from_isolation(&a, &b, 0);
        assert_eq!(d0, 0.0, "identical clusters must have zero divergence at t=0");
    }
}
