//! Canonical SEEDS — the content-model substrate for species and races.
//!
//! Content law: 2-layer model — canonical [`SeedDefinition`]s (named races + a
//! raw-organism primitive, with a `0..1` divergence dial) atop the algorithmic
//! [`crate::Dna`] substrate. Emergence is the default everywhere; this module
//! only provides a *nudge* at spawn time, not a hardlock over the simulation.
//!
//! Files are stored as RON (Rusty Object Notation) so the schema is typed and
//! round-trips, but the loader accepts any `serde::de::DeserializeOwned` shape
//! that the caller hands in (see [`SeedLibrary::from_ron_str`]).
//!
//! ## Divergence dial semantics
//!
//! `SeedDefinition::divergence` ∈ `[0, 1]` linearly scales the per-byte
//! mutation rate at spawn and in every subsequent `mutate_with_divergence`
//! call:
//!
//! * `0.0` → `effective_rate = 0` (genome is *clamped* to the seed; no drift).
//! * `1.0` → `effective_rate = dna_class.mutation_rate` (full free drift).
//! * In between → linear blend (e.g. `0.25` ⇒ 25% of the class rate).
//!
//! `mutate_with_divergence` is the canonical call site: it scales the rate
//! *and* falls back to the class baseline when the seed is absent so callers
//! can pass `None` to get pure algorithmic drift.

use crate::{Dna, DnaClass};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Stable identifier for a seed (matches the `id` field in scenario refs).
pub type SeedId = String;

/// Soft, label-based biome affinity. Stored as a free-form string so the
/// genetics crate stays decoupled from `civ-planet::BiomeKind`; the engine
/// resolves these hints at spawn time.
pub type BiomeAffinity = String;

/// One canonical content seed.
///
/// RON example:
///
/// ```ron
/// SeedDefinition(
///     id: "raw_organism",
///     display_name: "Raw Organism",
///     dna_length: 64,
///     genome: [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63],
///     divergence: 1.0,
///     spawn_biome_affinity: [],
///     notes: "Primitive substrate; no biome preference, full drift.",
/// )
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeedDefinition {
    /// Stable id (e.g. `"raw_organism"`, `"human_baseline"`).
    pub id: SeedId,
    /// Human-readable name for UI / debug.
    pub display_name: String,
    /// Length of the [`Dna`] vector. Must equal `genome.len()` after load.
    pub dna_length: usize,
    /// Base genome bytes. The substrate's "raw-organism primitive" is
    /// `[0, 1, 2, …]`; named races carry a non-trivial, hand-curated pattern.
    pub genome: Vec<u8>,
    /// `0..1` divergence dial (see module docs for semantics).
    pub divergence: f32,
    /// Soft biome affinity hints (labels, not enum references).
    pub spawn_biome_affinity: Vec<BiomeAffinity>,
    /// Optional free-form note for the content author.
    #[serde(default)]
    pub notes: Option<String>,
}

/// Top-level RON container: a versioned set of seeds loaded from one file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeedSet {
    /// Schema version, e.g. `1`.
    pub version: u32,
    /// All seeds declared in this file.
    pub seeds: Vec<SeedDefinition>,
}

/// In-memory collection of loaded seeds, indexed by `SeedId`.
///
/// The library is the *only* spawn-time entry point for genome seeding and is
/// the bridge between scenario YAML / RON and the genetics substrate.
#[derive(Debug, Default, Clone)]
pub struct SeedLibrary {
    by_id: std::collections::HashMap<SeedId, SeedDefinition>,
}

impl SeedLibrary {
    /// Build an empty library.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a RON string into a [`SeedSet`] and merge it into the library.
    ///
    /// # Errors
    ///
    /// Returns [`SeedError`] when the RON is malformed, when a seed fails
    /// validation, or when a duplicate id is encountered.
    pub fn from_ron_str(src: &str) -> Result<Self, SeedError> {
        let set: SeedSet = ron::from_str(src).map_err(|e| SeedError::Ron(e.to_string()))?;
        let mut lib = Self::new();
        for seed in set.seeds {
            lib.insert(seed)?;
        }
        Ok(lib)
    }

    /// Load from an arbitrary deserializer-driven source (e.g. JSON, YAML)
    /// by deserializing into a [`SeedSet`] first.
    pub fn from_seed_set(set: SeedSet) -> Result<Self, SeedError> {
        let mut lib = Self::new();
        for seed in set.seeds {
            lib.insert(seed)?;
        }
        Ok(lib)
    }

    /// Insert a single seed, validating and rejecting duplicates.
    pub fn insert(&mut self, seed: SeedDefinition) -> Result<(), SeedError> {
        seed.validate()?;
        if self.by_id.contains_key(&seed.id) {
            return Err(SeedError::DuplicateId(seed.id));
        }
        self.by_id.insert(seed.id.clone(), seed);
        Ok(())
    }

    /// Look up a seed by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&SeedDefinition> {
        self.by_id.get(id)
    }

    /// Iterate over all loaded seeds.
    pub fn iter(&self) -> impl Iterator<Item = (&SeedId, &SeedDefinition)> {
        self.by_id.iter()
    }

    /// Total seed count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// `true` if no seeds are loaded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Keep only seeds for which `f(id, &seed)` returns `true`. Used by the
    /// engine to drop conflicting ids before re-inserting a fresh set.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&SeedId, &SeedDefinition) -> bool,
    {
        self.by_id.retain(|k, v| f(k, v));
    }
}

/// Errors raised by the seed subsystem.
#[derive(Debug, thiserror::Error)]
pub enum SeedError {
    /// RON deserialisation failed.
    #[error("RON parse error: {0}")]
    Ron(String),
    /// `divergence` was NaN, infinite, or outside `[0, 1]`.
    #[error("invalid divergence {0} (must be in [0, 1] and finite)")]
    InvalidDivergence(f32),
    /// `dna_length` did not match the byte count of `genome`.
    #[error("dna_length {expected} does not match genome length {actual}")]
    GenomeLengthMismatch {
        /// Declared length in the seed file.
        expected: usize,
        /// Actual byte count of the `genome` vector.
        actual: usize,
    },
    /// `dna_length` was zero.
    #[error("dna_length must be > 0")]
    ZeroLength,
    /// A seed with the same id was already loaded.
    #[error("duplicate seed id: {0}")]
    DuplicateId(String),
    /// A scenario referenced a seed id that is not in the library.
    #[error("unknown seed id: {0}")]
    UnknownSeed(String),
}

impl SeedDefinition {
    /// Validate the seed is internally consistent.
    pub fn validate(&self) -> Result<(), SeedError> {
        if self.dna_length == 0 {
            return Err(SeedError::ZeroLength);
        }
        if self.dna_length != self.genome.len() {
            return Err(SeedError::GenomeLengthMismatch {
                expected: self.dna_length,
                actual: self.genome.len(),
            });
        }
        if !self.divergence.is_finite() || !(0.0..=1.0).contains(&self.divergence) {
            return Err(SeedError::InvalidDivergence(self.divergence));
        }
        Ok(())
    }

    /// Borrow the seed's base genome as a typed [`Dna`].
    #[must_use]
    pub fn base_dna(&self) -> Dna {
        Dna(self.genome.clone())
    }
}

/// Effective per-byte mutation rate after applying the divergence dial.
///
/// `effective = divergence * class.mutation_rate` (clamped to `[0, 1]`).
#[inline]
#[must_use]
pub fn effective_mutation_rate(class: &DnaClass, divergence: f32) -> f32 {
    if !divergence.is_finite() {
        return class.mutation_rate;
    }
    divergence.clamp(0.0, 1.0) * class.mutation_rate
}

/// Mutate a genome under a divergence dial.
///
/// * `divergence = 0.0` returns the genome unchanged (no drift).
/// * `divergence = 1.0` applies the class's full mutation rate (free drift).
/// * Intermediate values linearly scale the per-byte rate.
///
/// This is the canonical spawn-time and per-tick mutation entry point. It is
/// deliberately a thin wrapper over the per-byte mutation loop so existing
/// callers continue to work; new code that has a seed should call this
/// instead of [`crate::mutate`].
#[must_use]
pub fn mutate_with_divergence(
    dna: &Dna,
    rng: &mut ChaCha8Rng,
    class: &DnaClass,
    divergence: f32,
) -> Dna {
    let rate = effective_mutation_rate(class, divergence);
    if rate <= 0.0 {
        return dna.clone();
    }
    let mut out = dna.clone();
    for byte in out.0.iter_mut() {
        if rng.gen::<f32>() < rate {
            *byte = rng.r#gen();
        }
    }
    out
}

/// Sample a new genome from a seed (or from scratch when `seed` is `None`).
///
/// Behaviour:
///
/// * `Some(seed)` with `seed.divergence = 0.0` → returns the seed genome
///   unchanged (clamped).
/// * `Some(seed)` with `seed.divergence > 0.0` → applies
///   `effective_mutation_rate(seed)` to the seed.
/// * `None` → falls back to a fully random genome of the class's expected
///   length (the substrate's "raw-organism primitive" mode).
#[must_use]
pub fn spawn_genome(rng: &mut ChaCha8Rng, class: &DnaClass, seed: Option<&SeedDefinition>) -> Dna {
    match seed {
        Some(s) => {
            let dna = s.base_dna();
            mutate_with_divergence(&dna, rng, class, s.divergence)
        }
        None => Dna::random(class.length, rng),
    }
}

/// The canonical raw-organism primitive used as the substrate's "no seed"
/// baseline. Length matches the default [`DnaClass`] in the genetics crate
/// (64 bytes), with genome `[0, 1, 2, …]` and full divergence.
#[must_use]
pub fn raw_organism_primitive() -> SeedDefinition {
    let length: usize = 64;
    let genome: Vec<u8> = (0..length as u8).collect();
    SeedDefinition {
        id: "raw_organism".to_string(),
        display_name: "Raw Organism".to_string(),
        dna_length: length,
        genome,
        divergence: 1.0,
        spawn_biome_affinity: Vec::new(),
        notes: Some(
            "Primitive substrate; no biome affinity, full drift. \
             This is the default when no seed is referenced."
                .to_string(),
        ),
    }
}

/// Convenience: build the canonical example seed set (raw-organism + two
/// named races) used by tests, docs, and the `civis-3d-scenario-check` smoke.
#[must_use]
pub fn example_seed_set() -> SeedSet {
    let length: usize = 64;
    SeedSet {
        version: 1,
        seeds: vec![
            raw_organism_primitive(),
            SeedDefinition {
                id: "human_baseline".to_string(),
                display_name: "Human Baseline".to_string(),
                dna_length: length,
                // A simple, distinctive 64-byte pattern (alternating + offset)
                // that the speciation distance can latch onto while still
                // being far from a "random walk" baseline.
                genome: (0..length as u8)
                    .map(|i| i.wrapping_mul(7).wrapping_add(13))
                    .collect(),
                divergence: 0.1,
                spawn_biome_affinity: vec!["TemperateForest".to_string()],
                notes: Some(
                    "Named race; mid-low drift, prefers temperate forest biomes.".to_string(),
                ),
            },
            SeedDefinition {
                id: "deep_one".to_string(),
                display_name: "Deep One".to_string(),
                dna_length: length,
                genome: (0..length as u8)
                    .map(|i| i.wrapping_mul(31).wrapping_add(5))
                    .collect(),
                divergence: 0.4,
                spawn_biome_affinity: vec!["Ocean".to_string(), "Tidepool".to_string()],
                notes: Some(
                    "Aquatic-adapted named race; moderate drift, marine affinity.".to_string(),
                ),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_class() -> DnaClass {
        DnaClass {
            name: "test".to_string(),
            length: 64,
            mutation_rate: 0.05,
            speciation_threshold: 0.5,
        }
    }

    #[test]
    fn raw_organism_primitive_is_valid() {
        let s = raw_organism_primitive();
        s.validate().expect("raw organism must validate");
        assert_eq!(s.dna_length, 64);
        assert_eq!(s.genome.len(), 64);
        assert!((s.divergence - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn example_seed_set_validates() {
        let set = example_seed_set();
        assert!(set.version >= 1);
        for s in &set.seeds {
            s.validate()
                .unwrap_or_else(|e| panic!("seed {} failed: {e}", s.id));
        }
    }

    #[test]
    fn from_seed_set_loads_example_seeds() {
        let set = example_seed_set();
        let lib = SeedLibrary::from_seed_set(set).expect("valid example set");
        assert_eq!(lib.len(), 3);
        let raw = lib.get("raw_organism").expect("raw_organism");
        let expected = raw_organism_primitive();
        assert_eq!(raw.divergence, expected.divergence);
        assert_eq!(raw.genome, expected.genome);
        assert!(lib.get("human_baseline").is_some());
        assert!(lib.get("deep_one").is_some());
    }

    #[test]
    fn from_seed_set_rejects_duplicate_ids() {
        let mut set = example_seed_set();
        set.seeds.push(raw_organism_primitive());
        let err = SeedLibrary::from_seed_set(set).unwrap_err();
        assert!(matches!(err, SeedError::DuplicateId(_)));
    }

    #[test]
    fn base_dna_clones_seed_genome() {
        let seed = raw_organism_primitive();
        let dna = seed.base_dna();
        assert_eq!(dna.0, seed.genome);
        assert_eq!(dna.0.len(), seed.dna_length);
    }

    #[test]
    fn retain_filters_seeds_by_predicate() {
        let mut lib = SeedLibrary::from_seed_set(example_seed_set()).expect("load");
        lib.retain(|id, _| id == "human_baseline" || id == "deep_one");
        assert_eq!(lib.len(), 2);
        assert!(lib.get("raw_organism").is_none());
        assert!(lib.get("human_baseline").is_some());
        assert!(lib.get("deep_one").is_some());
    }

    #[test]
    fn example_seed_set_ron_round_trip() {
        let ron_src = ron::to_string(&example_seed_set()).expect("ron serialize");
        let lib = SeedLibrary::from_ron_str(&ron_src).expect("ron parse");
        assert_eq!(lib.len(), 3);
        assert!(lib.get("raw_organism").is_some());
        assert!(lib.get("human_baseline").is_some());
        assert!(lib.get("deep_one").is_some());
        // Round-trip the parsed library back to a SeedSet and compare.
        let mut seeds: Vec<SeedDefinition> = lib.iter().map(|(_, s)| s.clone()).collect();
        seeds.sort_by(|a, b| a.id.cmp(&b.id));
        let mut expected = example_seed_set().seeds;
        expected.sort_by(|a, b| a.id.cmp(&b.id));
        assert_eq!(seeds, expected);
    }

    #[test]
    fn library_loader_rejects_invalid_divergence() {
        let bad = SeedDefinition {
            id: "bad".to_string(),
            display_name: "Bad".to_string(),
            dna_length: 4,
            genome: vec![1, 2, 3, 4],
            divergence: 1.5,
            spawn_biome_affinity: vec![],
            notes: None,
        };
        let mut lib = SeedLibrary::new();
        let err = lib.insert(bad).unwrap_err();
        assert!(matches!(err, SeedError::InvalidDivergence(_)));
    }

    #[test]
    fn library_loader_rejects_length_mismatch() {
        let bad = SeedDefinition {
            id: "short".to_string(),
            display_name: "Short".to_string(),
            dna_length: 8,
            genome: vec![1, 2, 3, 4], // only 4 bytes
            divergence: 0.0,
            spawn_biome_affinity: vec![],
            notes: None,
        };
        let mut lib = SeedLibrary::new();
        let err = lib.insert(bad).unwrap_err();
        assert!(matches!(err, SeedError::GenomeLengthMismatch { .. }));
    }

    #[test]
    fn library_loader_rejects_duplicate_id() {
        let mut lib = SeedLibrary::new();
        lib.insert(raw_organism_primitive()).unwrap();
        let err = lib.insert(raw_organism_primitive()).unwrap_err();
        assert!(matches!(err, SeedError::DuplicateId(_)));
    }

    #[test]
    fn divergence_dial_zero_means_no_drift_over_generations() {
        let mut rng = ChaCha8Rng::seed_from_u64(0xC0FFEE_u64);
        let class = base_class();
        let mut locked = raw_organism_primitive();
        locked.divergence = 0.0;

        let dna = spawn_genome(&mut rng, &class, Some(&locked));
        let original = dna.clone();
        let mut cur = dna;
        for _ in 0..10_000 {
            cur = mutate_with_divergence(&cur, &mut rng, &class, locked.divergence);
            assert_eq!(cur.0, original.0, "dial=0 must never mutate the genome");
        }
    }

    #[test]
    fn divergence_dial_one_means_free_drift() {
        let mut rng = ChaCha8Rng::seed_from_u64(0xDEAD_BEEF_u64);
        let class = base_class();
        let mut free = raw_organism_primitive();
        free.divergence = 1.0;

        let mut cur = spawn_genome(&mut rng, &class, Some(&free));
        let original = cur.clone();
        let mut drifted = false;
        for _ in 0..1_000 {
            cur = mutate_with_divergence(&cur, &mut rng, &class, free.divergence);
            if cur.0 != original.0 {
                drifted = true;
                break;
            }
        }
        assert!(
            drifted,
            "dial=1.0 with mutation_rate=0.05 must eventually drift over 1000 ticks"
        );
    }

    #[test]
    fn divergence_dial_intermediate_scales_rate() {
        // 0.0 = no drift (covered above), 1.0 = full drift. The intermediate
        // 0.5 case must mutate *less* than 1.0 over the same number of ticks
        // for the same seed/RNG seed. With mutation_rate=0.05 and 500 ticks
        // every byte is almost certainly flipped in both cases (normalized
        // distance saturates at 1.0), so we count the *number* of byte flips
        // along the way to detect the per-tick rate difference.
        let class = base_class();
        let n_ticks: usize = 200;
        let base_seed = raw_organism_primitive();

        let mut full_rng = ChaCha8Rng::seed_from_u64(42);
        let mut full_dna = Dna(base_seed.genome.clone());
        let mut full_flips: usize = 0;
        let mut prev = full_dna.clone();
        for _ in 0..n_ticks {
            full_dna = mutate_with_divergence(&full_dna, &mut full_rng, &class, 1.0);
            full_flips += prev
                .0
                .iter()
                .zip(full_dna.0.iter())
                .filter(|(a, b)| a != b)
                .count();
            prev = full_dna.clone();
        }

        let mut half_rng = ChaCha8Rng::seed_from_u64(42);
        let mut half_dna = Dna(base_seed.genome.clone());
        let mut half_flips: usize = 0;
        let mut prev = half_dna.clone();
        for _ in 0..n_ticks {
            half_dna = mutate_with_divergence(&half_dna, &mut half_rng, &class, 0.5);
            half_flips += prev
                .0
                .iter()
                .zip(half_dna.0.iter())
                .filter(|(a, b)| a != b)
                .count();
            prev = half_dna.clone();
        }

        assert!(
            half_flips < full_flips,
            "half-rate dial must produce fewer per-tick flips than full-rate \
             (half={half_flips}, full={full_flips})"
        );
    }

    #[test]
    fn spawn_genome_none_falls_back_to_random() {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let class = base_class();
        let dna = spawn_genome(&mut rng, &class, None);
        assert_eq!(dna.0.len(), class.length);
    }

    #[test]
    fn spawn_genome_with_locked_seed_returns_seed() {
        let mut rng = ChaCha8Rng::seed_from_u64(9);
        let class = base_class();
        let mut locked = raw_organism_primitive();
        locked.divergence = 0.0;
        let dna = spawn_genome(&mut rng, &class, Some(&locked));
        assert_eq!(dna.0, locked.genome);
    }

    #[test]
    fn effective_mutation_rate_clamps_and_handles_non_finite() {
        let class = DnaClass {
            name: "x".into(),
            length: 32,
            mutation_rate: 0.1,
            speciation_threshold: 0.5,
        };
        assert!((effective_mutation_rate(&class, 0.0) - 0.0).abs() < f32::EPSILON);
        assert!((effective_mutation_rate(&class, 1.0) - 0.1).abs() < f32::EPSILON);
        assert!((effective_mutation_rate(&class, 0.5) - 0.05).abs() < f32::EPSILON);
        // Clamping
        assert!((effective_mutation_rate(&class, 1.5) - 0.1).abs() < f32::EPSILON);
        assert!((effective_mutation_rate(&class, -0.5) - 0.0).abs() < f32::EPSILON);
        // Non-finite falls back to class rate.
        assert!((effective_mutation_rate(&class, f32::NAN) - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn ron_handles_unicode_notes() {
        // Notes field carries free-form text; RON must escape it correctly.
        let set = SeedSet {
            version: 1,
            seeds: vec![SeedDefinition {
                id: "fancy".into(),
                display_name: "Fancy".into(),
                dna_length: 4,
                genome: vec![10, 20, 30, 40],
                divergence: 0.2,
                spawn_biome_affinity: vec!["Bog".into()],
                notes: Some("Notes with \"quotes\" and \nnewlines.".into()),
            }],
        };
        let ron_src = ron::to_string(&set).unwrap();
        let lib = SeedLibrary::from_ron_str(&ron_src).expect("parse unicode");
        let s = lib.get("fancy").unwrap();
        assert_eq!(
            s.notes.as_deref(),
            Some("Notes with \"quotes\" and \nnewlines.")
        );
    }
}
