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

pub use seeds::{
    all_named_seeds, archetype_dna, archetype_seed, divergence_preset, effective_mutation_rate,
    example_seed_set, mutate_with_divergence, named_archetype_seed_set, named_seed_round_robin,
    raw_organism_primitive, seed_with_divergence, spawn_genome, spawn_genome_with_divergence,
    BiomeAffinity, DivergencePreset, NamedSeed, SeedDefinition, SeedError, SeedId, SeedLibrary,
    SeedSet, ALL_NAMED_SEEDS, DIVERGENCE_PRESETS,
};

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
}
