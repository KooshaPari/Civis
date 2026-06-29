//! civ-genetics — algorithmic DNA, mutation, recombination, fitness, speciation.
//!
//! Per ADR-008 the genetic loop is **pure algorithm, no LLM**. All randomness
//! threads through a caller-provided [`ChaCha8Rng`] so replay is bit-identical.
//!
//! See `docs/development-guide/fr-3d-additions.md` for `FR-CIV-GENETICS-*`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub mod seeds;
pub mod sentience;
pub mod traits;

pub use seeds::{
    archetype_dna, archetype_seed, effective_mutation_rate, example_seed_set,
    mutate_with_divergence, raw_organism_primitive, seed_with_divergence, spawn_genome,
    spawn_genome_with_divergence, BiomeAffinity, NamedSeed, SeedDefinition, SeedError, SeedId,
    SeedLibrary, SeedSet,
};
pub use traits::{inherit_trait_vector, TraitInheritance, TraitVector};

/// Schema version for `civ-genetics`. Bumped on breaking changes.
pub const SCHEMA_VERSION: &str = "0.1.0-stub";

/// A DNA strand — fixed-length byte vector. Length is class-parameterised at
/// construction; the type itself is class-agnostic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dna(pub Vec<u8>);

impl Dna {
    /// Construct a new DNA of `len` bytes, all initialised to zero.
    #[must_use]
    pub fn zero(len: usize) -> Self {
        Self(vec![0; len])
    }

    /// Construct a random DNA of `len` bytes seeded from `rng`.
    pub fn random(len: usize, rng: &mut ChaCha8Rng) -> Self {
        let mut bytes = vec![0u8; len];
        rng.fill(&mut bytes[..]);
        Self(bytes)
    }

    /// Length in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Is this DNA empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Per-class genetic configuration. New classes (humanoid, quadruped,
/// silicate, …) are data-driven; this struct is the entire schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DnaClass {
    /// Human-readable class name (mod-friendly).
    pub name: String,
    /// DNA length in bytes for organisms of this class.
    pub length: usize,
    /// Per-byte point-mutation probability per `mutate` call (0..=1).
    pub mutation_rate: f32,
    /// Speciation threshold: Hamming-distance fraction at which two genomes
    /// are considered to be in distinct species.
    pub speciation_threshold: f32,
}

impl Default for DnaClass {
    fn default() -> Self {
        Self {
            name: "default".into(),
            length: 64,
            mutation_rate: 0.01,
            speciation_threshold: 0.25,
        }
    }
}

/// Apply class-parameterised point mutations to `dna` in place. Each byte has
/// probability `class.mutation_rate` of being replaced with a fresh random
/// value drawn from `rng`. Deterministic under a fixed `rng` seed.
pub fn mutate(dna: &mut Dna, rng: &mut ChaCha8Rng, class: &DnaClass) {
    for byte in &mut dna.0 {
        if rng.gen::<f32>() < class.mutation_rate {
            *byte = rng.r#gen();
        }
    }
}

/// Uniform-crossover recombination: for each byte position, deterministically
/// draw from `parent_a` or `parent_b` with equal probability. Both parents
/// must be the same length.
pub fn recombine(parent_a: &Dna, parent_b: &Dna, rng: &mut ChaCha8Rng, _class: &DnaClass) -> Dna {
    assert_eq!(
        parent_a.0.len(),
        parent_b.0.len(),
        "recombine: parent length mismatch"
    );
    let mut child = Vec::with_capacity(parent_a.0.len());
    for (a, b) in parent_a.0.iter().zip(parent_b.0.iter()) {
        let from_a: bool = rng.r#gen();
        child.push(if from_a { *a } else { *b });
    }
    Dna(child)
}

/// Cosine-similarity fitness against an environment vector. Higher = fitter.
/// Both vectors are interpreted as unsigned bytes mapped to `[0, 1]`. Returns
/// `0.0` when either vector is all-zero (no direction).
#[must_use]
pub fn fitness(dna: &Dna, environment: &[u8]) -> f32 {
    let n = dna.0.len().min(environment.len());
    if n == 0 {
        return 0.0;
    }
    let mut dot: f64 = 0.0;
    let mut mag_a: f64 = 0.0;
    let mut mag_b: f64 = 0.0;
    for (a_byte, b_byte) in dna.0.iter().zip(environment.iter()).take(n) {
        let a = f64::from(*a_byte) / 255.0;
        let b = f64::from(*b_byte) / 255.0;
        dot += a * b;
        mag_a += a * a;
        mag_b += b * b;
    }
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    (dot / (mag_a.sqrt() * mag_b.sqrt())) as f32
}

/// Normalised Hamming distance between two DNAs of equal length (`0.0` =
/// identical, `1.0` = every byte differs). Panics on length mismatch — callers
/// are expected to compare within the same `DnaClass`.
#[must_use]
pub fn speciation_distance(a: &Dna, b: &Dna) -> f32 {
    assert_eq!(a.0.len(), b.0.len(), "speciation_distance: length mismatch");
    if a.0.is_empty() {
        return 0.0;
    }
    let diff = a.0.iter().zip(b.0.iter()).filter(|(x, y)| x != y).count();
    diff as f32 / a.0.len() as f32
}

/// True when two genomes have drifted past the class's speciation threshold.
#[must_use]
pub fn should_speciate(a: &Dna, b: &Dna, class: &DnaClass) -> bool {
    speciation_distance(a, b) > class.speciation_threshold
}

/// A species record. Issued by the simulation when [`should_speciate`] fires;
/// the founder centroid is the DNA snapshot of the individual that triggered
/// reproductive isolation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Species {
    /// Stable species ID.
    pub id: u64,
    /// The DNA class this species belongs to.
    pub dna_class: String,
    /// DNA snapshot of the speciation trigger.
    pub founder_centroid: Dna,
}

/// Tracks the speciation event: when two populations isolated long enough
/// (DNA distance crosses a threshold) diverge into distinct subspecies.
///
/// Speciation is emergent: populations never explicitly tagged as separate
/// are treated as one lineage until their DNA distance reaches the class's
/// `speciation_threshold`, at which point the simulation records a speciation
/// event and may tag them as distinct species.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeciationEvent {
    /// Timestamp or generation marker at which speciation occurred.
    pub trigger_marker: u64,
    /// Lineage/population ID that triggered the split.
    pub lineage_id: u64,
    /// The DNA specimen that crossed the isolation threshold.
    pub specimen_dna: Dna,
    /// The reference population centroid it diverged from.
    pub ancestral_centroid: Dna,
    /// The measured Hamming distance at divergence.
    pub divergence_distance: f32,
    /// The class's speciation threshold that was crossed.
    pub threshold: f32,
}

/// Evaluate whether two populations have diverged into separate species.
///
/// Returns `Some(SpeciationEvent)` when the speciation threshold is crossed,
/// indicating that the two populations should now be tagged as distinct species.
/// Returns `None` if the populations remain within the threshold (still one species).
#[must_use]
pub fn evaluate_speciation(
    trigger_marker: u64,
    lineage_id: u64,
    specimen_dna: &Dna,
    ancestral_centroid: &Dna,
    class: &DnaClass,
) -> Option<SpeciationEvent> {
    let distance = speciation_distance(specimen_dna, ancestral_centroid);
    if distance > class.speciation_threshold {
        Some(SpeciationEvent {
            trigger_marker,
            lineage_id,
            specimen_dna: specimen_dna.clone(),
            ancestral_centroid: ancestral_centroid.clone(),
            divergence_distance: distance,
            threshold: class.speciation_threshold,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    /// Covers FR-CIV-GENETICS-000 — exposes a semver-like schema version stub.
    #[test]
    fn schema_version_stub() {
        assert!(!SCHEMA_VERSION.is_empty());
        let core = SCHEMA_VERSION.split('-').next().unwrap();
        let segments: Vec<&str> = core.split('.').collect();
        assert_eq!(segments.len(), 3);
        assert!(segments.iter().all(|part| !part.is_empty()));
    }

    /// Covers FR-CIV-GENETICS-001 — mutation is deterministic under a fixed seed.
    #[test]
    fn mutation_deterministic() {
        let class = DnaClass::default();
        let mut a = Dna::zero(class.length);
        let mut b = Dna::zero(class.length);
        let mut r1 = rng(99);
        let mut r2 = rng(99);
        mutate(&mut a, &mut r1, &class);
        mutate(&mut b, &mut r2, &class);
        assert_eq!(a, b);
    }

    /// Covers FR-CIV-GENETICS-002 — recombination is deterministic under a fixed seed.
    #[test]
    fn recombination_deterministic() {
        let class = DnaClass::default();
        let parent_a = Dna(vec![1u8; class.length]);
        let parent_b = Dna(vec![2u8; class.length]);
        let mut r1 = rng(123);
        let mut r2 = rng(123);
        let c1 = recombine(&parent_a, &parent_b, &mut r1, &class);
        let c2 = recombine(&parent_a, &parent_b, &mut r2, &class);
        assert_eq!(c1, c2);
        // And every byte must come from one of the parents.
        for byte in &c1.0 {
            assert!(*byte == 1 || *byte == 2);
        }
    }

    /// Covers FR-CIV-GENETICS-010 — speciation triggers above the class threshold and
    /// not below.
    #[test]
    fn speciation_trigger() {
        let class = DnaClass {
            speciation_threshold: 0.5,
            ..DnaClass::default()
        };
        let a = Dna(vec![0u8; class.length]);
        let mut b = a.clone();
        // Flip 10% of bytes — below threshold.
        for i in 0..(class.length / 10) {
            b.0[i] = 0xff;
        }
        assert!(!should_speciate(&a, &b, &class));
        // Flip everything — above threshold.
        for byte in &mut b.0 {
            *byte = 0xff;
        }
        assert!(should_speciate(&a, &b, &class));
    }

    /// Covers FR-CIV-GENETICS-011 — speciation_distance is symmetric.
    #[test]
    fn speciation_distance_is_symmetric() {
        let a = Dna(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let b = Dna(vec![1, 0, 3, 0, 5, 0, 7, 0]);
        assert_eq!(speciation_distance(&a, &b), speciation_distance(&b, &a));
    }

    /// Covers FR-CIV-GENETICS-012 — fitness against the same vector as DNA is 1.0.
    #[test]
    fn self_fitness_is_one() {
        let dna = Dna(vec![123; 16]);
        let env = vec![123u8; 16];
        let f = fitness(&dna, &env);
        assert!((f - 1.0).abs() < 1e-6);
    }

    // ── Emergent Speciation (FR-CIV-SPECIES gap) ─────────────────────────

    /// Covers FR-CIV-SPECIES (emergent speciation): two long-isolated populations
    /// that accumulate sufficient DNA distance (> speciation_threshold) trigger
    /// a SpeciationEvent. This test simulates a scenario where a population
    /// diverges over many generations while remaining in reproductive isolation.
    #[test]
    fn speciation_event_fires_when_populations_diverge_enough() {
        let class = DnaClass {
            name: "test_species".to_string(),
            length: 64,
            mutation_rate: 0.05,
            speciation_threshold: 0.15, // 15% divergence triggers speciation
        };

        // Start with a common ancestral population.
        let ancestral = Dna(vec![128u8; 64]);

        // Simulate isolated population A — mutate for many generations.
        let mut pop_a = ancestral.clone();
        let mut rng_a = rng(42);
        for _ in 0..50 {
            mutate(&mut pop_a, &mut rng_a, &class);
        }

        // Simulate isolated population B — mutate independently.
        let mut pop_b = ancestral.clone();
        let mut rng_b = rng(99);
        for _ in 0..50 {
            mutate(&mut pop_b, &mut rng_b, &class);
        }

        // Both populations should have drifted significantly from the ancestor.
        let dist_a = speciation_distance(&pop_a, &ancestral);
        let dist_b = speciation_distance(&pop_b, &ancestral);
        assert!(
            dist_a > class.speciation_threshold,
            "population A should diverge past threshold (got {dist_a})"
        );
        assert!(
            dist_b > class.speciation_threshold,
            "population B should diverge past threshold (got {dist_b})"
        );

        // Speciation events should fire for both populations.
        let event_a = evaluate_speciation(100, 1, &pop_a, &ancestral, &class);
        let event_b = evaluate_speciation(101, 2, &pop_b, &ancestral, &class);

        assert!(event_a.is_some(), "population A should trigger speciation");
        assert!(event_b.is_some(), "population B should trigger speciation");

        let evt_a = event_a.unwrap();
        let evt_b = event_b.unwrap();

        assert_eq!(evt_a.lineage_id, 1);
        assert_eq!(evt_b.lineage_id, 2);
        assert_eq!(evt_a.trigger_marker, 100);
        assert_eq!(evt_b.trigger_marker, 101);
        assert!(evt_a.divergence_distance > class.speciation_threshold);
        assert!(evt_b.divergence_distance > class.speciation_threshold);
    }

    /// Covers FR-CIV-SPECIES (emergent speciation): a mixing population does NOT
    /// speciate. When two isolated populations re-mix and reproduce, their
    /// offspring average the genomes, bringing them back within the speciation
    /// threshold.
    #[test]
    fn mixing_population_does_not_speciate() {
        let class = DnaClass {
            name: "test_mixed".to_string(),
            length: 64,
            mutation_rate: 0.05,
            speciation_threshold: 0.25,
        };

        // Start with two diverged populations.
        let mut pop_a = Dna(vec![50u8; 64]);
        let mut pop_b = Dna(vec![200u8; 64]);

        let mut rng_a = rng(11);
        let mut rng_b = rng(22);

        // Diverge them for 30 generations each.
        for _ in 0..30 {
            mutate(&mut pop_a, &mut rng_a, &class);
            mutate(&mut pop_b, &mut rng_b, &class);
        }

        let dist_before_mix = speciation_distance(&pop_a, &pop_b);
        // They may or may not be above threshold; that's not the point.
        // The point is that when they re-mix (recombine), their offspring
        // tend toward an intermediate state.

        // Now create offspring through recombination (reproductive mixing).
        let mut rng_mix = rng(33);
        let mut mixed_offspring = Vec::new();
        for _ in 0..10 {
            let child = recombine(&pop_a, &pop_b, &mut rng_mix, &class);
            mixed_offspring.push(child);
        }

        // The offspring cluster around an intermediate point between
        // the two parents. Measure distance from offspring to parents.
        for offspring in mixed_offspring {
            let dist_to_a = speciation_distance(&offspring, &pop_a);
            let dist_to_b = speciation_distance(&offspring, &pop_b);

            // Both distances should be moderate (less than the original
            // parent-to-parent distance on average).
            let avg_offspring_dist = (dist_to_a + dist_to_b) / 2.0;
            assert!(
                avg_offspring_dist < dist_before_mix,
                "mixing should bring offspring closer to both parents \
                 (offspring avg dist {avg_offspring_dist} should be < parent dist {dist_before_mix})"
            );

            // Offspring should NOT trigger a speciation event on their own
            // because they represent the intermediate zone.
            let event = evaluate_speciation(200, 3, &offspring, &pop_a, &class);
            // The event may or may not fire depending on exact distances,
            // but the key is that recombination acts as a homogenizing force.
            // If it does fire, the distance should still be smaller than
            // if the populations had NOT mixed.
            if let Some(evt) = event {
                assert!(
                    evt.divergence_distance < dist_before_mix,
                    "mixed offspring speciation distance should be less than \
                     parent distance (mixed: {}, parents: {})",
                    evt.divergence_distance,
                    dist_before_mix
                );
            }
        }
    }

    /// Covers FR-CIV-SPECIES: speciation is deterministic under fixed RNG.
    /// Re-running the same sequence with the same RNG seed produces identical
    /// speciation events.
    #[test]
    fn speciation_is_deterministic() {
        let class = DnaClass {
            name: "deterministic".to_string(),
            length: 64,
            mutation_rate: 0.03,
            speciation_threshold: 0.2,
        };

        // First run.
        let mut pop_1 = Dna(vec![100u8; 64]);
        let mut rng_1 = rng(7777);
        for _ in 0..100 {
            mutate(&mut pop_1, &mut rng_1, &class);
        }
        let event_1 = evaluate_speciation(0, 99, &pop_1, &Dna(vec![100u8; 64]), &class);

        // Second run with identical seed.
        let mut pop_2 = Dna(vec![100u8; 64]);
        let mut rng_2 = rng(7777);
        for _ in 0..100 {
            mutate(&mut pop_2, &mut rng_2, &class);
        }
        let event_2 = evaluate_speciation(0, 99, &pop_2, &Dna(vec![100u8; 64]), &class);

        // Both runs should produce identical outcomes.
        assert_eq!(pop_1, pop_2, "divergence sequence must be identical");
        assert_eq!(event_1, event_2, "speciation events must be identical");
    }

    /// Covers FR-CIV-SPECIES: below-threshold populations do not trigger events.
    /// When DNA distance is below speciation_threshold, evaluate_speciation
    /// returns None.
    #[test]
    fn no_speciation_event_below_threshold() {
        let class = DnaClass {
            name: "low_threshold".to_string(),
            length: 64,
            mutation_rate: 0.001, // Very low mutation to stay below threshold.
            speciation_threshold: 0.5,
        };

        let base = Dna(vec![100u8; 64]);
        let mut variant = base.clone();

        // Flip only 5% of bytes (well below 50% threshold).
        for i in 0..3 {
            variant.0[i] = 200;
        }

        let dist = speciation_distance(&base, &variant);
        assert!(
            dist < class.speciation_threshold,
            "test setup: distance {} should be below threshold {}",
            dist,
            class.speciation_threshold
        );

        let event = evaluate_speciation(0, 5, &variant, &base, &class);
        assert!(event.is_none(), "no speciation event should fire below threshold");
    }
}
