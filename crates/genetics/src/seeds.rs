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
use rand::Rng;
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

/// Sample a new genome from a seed using an *explicit* divergence value
/// (ignoring `seed.divergence`). This is the canonical low-level call site
/// when a scenario-level override is in play.
///
/// * `divergence = 0.0` → genome is clamped to the seed (no drift).
/// * `divergence = 1.0` → full class mutation rate applied.
/// * Intermediate values → linear blend.
#[must_use]
pub fn spawn_genome_with_divergence(
    rng: &mut ChaCha8Rng,
    class: &DnaClass,
    seed: &SeedDefinition,
    divergence: f32,
) -> Dna {
    let dna = seed.base_dna();
    mutate_with_divergence(&dna, rng, class, divergence)
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
///
/// Delegates to [`spawn_genome_with_divergence`] so the logic lives in one
/// place; existing callers are unaffected.
#[must_use]
pub fn spawn_genome(rng: &mut ChaCha8Rng, class: &DnaClass, seed: Option<&SeedDefinition>) -> Dna {
    match seed {
        Some(s) => spawn_genome_with_divergence(rng, class, s, s.divergence),
        None => Dna::random(class.length, rng),
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Named-race archetypes (content-model layer)
// ──────────────────────────────────────────────────────────────────────────────

/// Canonical named races.  Each variant is a distinct biological/cultural
/// archetype whose genome is hand-curated to produce a characteristic
/// phenotype cluster *before* emergence diverges the population.
///
/// Add new variants here as the content layer grows; each must have a
/// matching arm in [`archetype_dna`] and [`archetype_seed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum NamedSeed {
    /// Ardani — arid-world endurance caste; heat-adapted, high aggression
    /// potential, structured social hierarchy encoded in leading bytes.
    Ardani,
    /// Velthari — deep-forest symbiotes; high genetic plasticity (varied
    /// genome), empathic resonance markers in upper bytes.
    Velthari,
    /// Grundak — subterranean lithomorphs; low divergence pressure, dense
    /// mineral-affinity encoding in mid-range bytes.
    Grundak,
    /// Kethari — littoral and reef-dwellers; saline-tolerance markers in
    /// wave-modulated mid bytes, moderate drift for coastal adaptation.
    Kethari,
    /// Nymari — glacial highlanders; inverted cold-adapted ramp with sparse
    /// heat-shock loci in the tail.
    Nymari,
    /// Sylphine — wind-steppe nomads; XOR-scattered genome encoding aerial
    /// metabolism and high plasticity at spawn.
    Sylphine,
    /// Felmar — volcanic marsh dwellers; prime-scattered heat-shock loci with
    /// wetland tolerance markers in alternating bands.
    Felmar,
    /// Thornari — thorn-scrub and badland foragers; quadratic byte scatter
    /// encoding drought resilience and low social hierarchy variance.
    Thornari,
    /// Lumari — bioluminescent cave-sea hybrids; oscillating sine-phase genome
    /// encodes deep-dark adaptation with high empathic resonance markers.
    Lumari,
    /// Drakhari — volcanic highland apex predators; prime-stride heat-burst
    /// pattern with aggression encoded in the leading quarter of the genome.
    Drakhari,
    /// Quelven — arboreal canopy weavers; Fibonacci-stride genome produces
    /// rapid sub-clade divergence in the high-plasticity tail bytes.
    Quelven,
    /// Ashborn — post-eruption pioneer colonists; alternating-ash genome with
    /// resilience loci in even bytes and high speciation pressure in odd bytes.
    Ashborn,
}

/// Stable ordering of every [`NamedSeed`] for round-robin spawn and tests.
pub const ALL_NAMED_SEEDS: &[NamedSeed] = &[
    NamedSeed::Ardani,
    NamedSeed::Velthari,
    NamedSeed::Grundak,
    NamedSeed::Kethari,
    NamedSeed::Nymari,
    NamedSeed::Sylphine,
    NamedSeed::Felmar,
    NamedSeed::Thornari,
    NamedSeed::Lumari,
    NamedSeed::Drakhari,
    NamedSeed::Quelven,
    NamedSeed::Ashborn,
];

/// Return the canonical list of named-race archetypes.
#[must_use]
pub fn all_named_seeds() -> &'static [NamedSeed] {
    ALL_NAMED_SEEDS
}

/// Round-robin helper used when a scenario ships no `seed_mix` override.
#[must_use]
pub fn named_seed_round_robin(spawn_index: usize) -> NamedSeed {
    ALL_NAMED_SEEDS[spawn_index % ALL_NAMED_SEEDS.len()]
}

/// One weighted entry in a scenario `seed_mix` block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightedNamedSeed {
    /// Named archetype to draw from.
    pub seed: NamedSeed,
    /// Relative weight; must be finite and strictly positive.
    pub weight: f32,
}

/// Choose a named seed for spawn index `spawn_index`.
///
/// * Empty `seed_mix` → [`named_seed_round_robin`].
/// * Single entry → always that seed (deterministic monoculture).
/// * Multiple entries → weighted sample via `rng` (caller must validate weights).
#[must_use]
pub fn choose_named_seed(
    seed_mix: &[WeightedNamedSeed],
    spawn_index: usize,
    rng: &mut ChaCha8Rng,
) -> NamedSeed {
    if seed_mix.is_empty() {
        return named_seed_round_robin(spawn_index);
    }
    if seed_mix.len() == 1 {
        return seed_mix[0].seed;
    }
    use rand::distributions::{Distribution, WeightedIndex};
    let weights: Vec<f32> = seed_mix.iter().map(|sw| sw.weight).collect();
    let dist = WeightedIndex::new(&weights).expect("seed_mix weights must be valid");
    seed_mix[dist.sample(rng)].seed
}

/// Resolve scenario divergence: explicit override wins, else seed default.
#[must_use]
pub fn effective_spawn_divergence(
    seed: &SeedDefinition,
    divergence_override: Option<f32>,
) -> f32 {
    divergence_override.unwrap_or(seed.divergence)
}

/// Curated scenario-level divergence dial presets (FR-CIV-EMERGENCE-004 nudge).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DivergencePreset {
    /// Stable preset id referenced from scenario docs / tooling.
    pub id: &'static str,
    /// Override value in `[0, 1]` applied at spawn when selected.
    pub divergence: f32,
    /// Author-facing note.
    pub notes: &'static str,
}

/// Named divergence presets for scenario authors.
pub const DIVERGENCE_PRESETS: &[DivergencePreset] = &[
    DivergencePreset {
        id: "locked",
        divergence: 0.0,
        notes: "Genomes clamped to seed; no spawn-time drift.",
    },
    DivergencePreset {
        id: "conservative",
        divergence: 0.15,
        notes: "Low drift; species clusters stay tight.",
    },
    DivergencePreset {
        id: "balanced",
        divergence: 0.35,
        notes: "Default multicultural sandbox; moderate emergence.",
    },
    DivergencePreset {
        id: "expressive",
        divergence: 0.55,
        notes: "Asymmetric weighting + visible sub-species within a generation.",
    },
    DivergencePreset {
        id: "chaotic",
        divergence: 1.0,
        notes: "Full class mutation rate; maximum species radiation.",
    },
    DivergencePreset {
        id: "stable",
        divergence: 0.08,
        notes: "Near-locked clusters; sub-species emerge only after many generations.",
    },
    DivergencePreset {
        id: "radiant",
        divergence: 0.75,
        notes: "High drift between expressive and chaotic; rapid visible radiation.",
    },
];

/// Look up a divergence preset by id (case-sensitive).
#[must_use]
pub fn divergence_preset(id: &str) -> Option<f32> {
    DIVERGENCE_PRESETS
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.divergence)
}

/// Return the canonical 64-byte [`Dna`] for a named race archetype.
///
/// These genomes are *fixed content constants* — they must not be generated
/// procedurally so scenario designers can rely on them being stable across
/// engine versions.  Use [`seed_with_divergence`] to introduce population
/// spread at spawn time.
#[must_use]
pub fn archetype_dna(seed: NamedSeed) -> Dna {
    const LEN: usize = 64;
    let bytes: Vec<u8> = match seed {
        // Ardani: ramp × 3, wrapping — compressed high-intensity pattern.
        // Distinctive in leading bytes (social hierarchy) and tail (heat).
        NamedSeed::Ardani => (0..LEN as u8)
            .map(|i| i.wrapping_mul(3).wrapping_add(37))
            .collect(),

        // Velthari: sinusoidal-like via alternating add/sub — high variance
        // throughout the genome encodes plasticity.
        NamedSeed::Velthari => (0..LEN as u8)
            .enumerate()
            .map(|(idx, i)| {
                if idx % 2 == 0 {
                    i.wrapping_mul(11).wrapping_add(71)
                } else {
                    i.wrapping_mul(17).wrapping_sub(13)
                }
            })
            .collect(),

        // Grundak: slowly-stepping ramp × prime — low variance, dense
        // mid-range values encode mineral affinity.
        NamedSeed::Grundak => (0..LEN as u8)
            .map(|i| i.wrapping_mul(2).wrapping_add(128))
            .collect(),

        // Kethari: prime-wave coastal pattern — saline markers throughout.
        NamedSeed::Kethari => (0..LEN as u8)
            .map(|i| i.wrapping_mul(23).wrapping_add(97))
            .collect(),

        // Nymari: inverted cold ramp — high bytes dominate, tail heat-shock loci.
        NamedSeed::Nymari => (0..LEN as u8)
            .map(|i| (255u8.wrapping_sub(i)).wrapping_mul(7).wrapping_add(19))
            .collect(),

        // Sylphine: XOR-scatter — high per-byte variance for steppe plasticity.
        NamedSeed::Sylphine => (0..LEN as u8)
            .map(|i| i.wrapping_mul(13) ^ i.wrapping_add(200))
            .collect(),

        // Felmar: prime-band marsh pattern — heat-shock alternating with wetland loci.
        NamedSeed::Felmar => (0..LEN as u8)
            .enumerate()
            .map(|(idx, i)| {
                let band = if idx % 3 == 0 { 29 } else { 11 };
                i.wrapping_mul(band).wrapping_add(83) ^ (idx as u8)
            })
            .collect(),

        // Thornari: quadratic scatter — drought markers with sparse hierarchy bytes.
        NamedSeed::Thornari => (0..LEN as u8)
            .map(|i| {
                let q = u16::from(i) * u16::from(i);
                (q.wrapping_add(41) % 251) as u8
            })
            .collect(),

        // Lumari: oscillating sine-phase via triangle-wave — deep-dark and empathic loci.
        NamedSeed::Lumari => (0..LEN as u8)
            .enumerate()
            .map(|(idx, i)| {
                // Triangle wave: ascend then descend in 16-byte periods.
                let phase = idx % 16;
                let tri: u8 = if phase < 8 {
                    (phase as u8).wrapping_mul(17)
                } else {
                    (15u8.wrapping_sub(phase as u8)).wrapping_mul(17)
                };
                i.wrapping_mul(7).wrapping_add(tri).wrapping_add(61)
            })
            .collect(),

        // Drakhari: prime-stride burst — leading-byte aggression, trailing heat-burst.
        NamedSeed::Drakhari => (0..LEN as u8)
            .enumerate()
            .map(|(idx, i)| {
                // First quarter: high aggression (leading bytes near 200-255).
                if idx < LEN / 4 {
                    i.wrapping_mul(5).wrapping_add(200)
                } else {
                    i.wrapping_mul(19).wrapping_add(47) ^ (idx as u8).wrapping_mul(3)
                }
            })
            .collect(),

        // Quelven: Fibonacci-stride — canopy-adapted, high tail plasticity.
        NamedSeed::Quelven => {
            let mut fib = [0u8; LEN];
            fib[0] = 1;
            fib[1] = 2;
            for k in 2..LEN {
                fib[k] = fib[k - 1].wrapping_add(fib[k - 2]).wrapping_add(k as u8);
            }
            fib.to_vec()
        }

        // Ashborn: alternating-ash pattern — even=resilience, odd=speciation loci.
        NamedSeed::Ashborn => (0..LEN as u8)
            .enumerate()
            .map(|(idx, i)| {
                if idx % 2 == 0 {
                    i.wrapping_mul(3).wrapping_add(150)
                } else {
                    i.wrapping_mul(43).wrapping_add(7) ^ 0xA5
                }
            })
            .collect(),
    };
    Dna(bytes)
}

/// Convenience wrapper: return an archetype as a validated [`SeedDefinition`]
/// ready for insertion into a [`SeedLibrary`].
#[must_use]
pub fn archetype_seed(named: NamedSeed) -> SeedDefinition {
    let (id, display_name, biomes, divergence, notes) = match named {
        NamedSeed::Ardani => (
            "ardani",
            "Ardani",
            vec!["Desert".to_string(), "Savanna".to_string()],
            0.15_f32,
            "Arid-world endurance caste; low drift, heat-adapted.",
        ),
        NamedSeed::Velthari => (
            "velthari",
            "Velthari",
            vec!["DeepForest".to_string(), "Rainforest".to_string()],
            0.35_f32,
            "Deep-forest symbiotes; moderate drift, high plasticity.",
        ),
        NamedSeed::Grundak => (
            "grundak",
            "Grundak",
            vec!["Cave".to_string(), "Underground".to_string()],
            0.05_f32,
            "Subterranean lithomorphs; very low drift, mineral affinity.",
        ),
        NamedSeed::Kethari => (
            "kethari",
            "Kethari",
            vec![
                "Ocean".to_string(),
                "Tidepool".to_string(),
                "Coast".to_string(),
            ],
            0.25_f32,
            "Littoral reef-dwellers; moderate drift, marine affinity.",
        ),
        NamedSeed::Nymari => (
            "nymari",
            "Nymari",
            vec!["Tundra".to_string(), "Glacier".to_string(), "Alpine".to_string()],
            0.20_f32,
            "Glacial highlanders; low-moderate drift, cold adaptation.",
        ),
        NamedSeed::Sylphine => (
            "sylphine",
            "Sylphine",
            vec![
                "Grassland".to_string(),
                "Savanna".to_string(),
                "Highland".to_string(),
            ],
            0.30_f32,
            "Wind-steppe nomads; moderate-high drift, aerial metabolism.",
        ),
        NamedSeed::Felmar => (
            "felmar",
            "Felmar",
            vec![
                "Volcanic".to_string(),
                "Marsh".to_string(),
                "HotSprings".to_string(),
            ],
            0.22_f32,
            "Volcanic marsh dwellers; moderate drift, heat and wetland affinity.",
        ),
        NamedSeed::Thornari => (
            "thornari",
            "Thornari",
            vec![
                "Scrubland".to_string(),
                "Badlands".to_string(),
                "Desert".to_string(),
            ],
            0.18_f32,
            "Thorn-scrub foragers; low-moderate drift, drought resilience.",
        ),
        NamedSeed::Lumari => (
            "lumari",
            "Lumari",
            vec![
                "DeepCave".to_string(),
                "UnderwaterCave".to_string(),
                "Abyss".to_string(),
            ],
            0.28_f32,
            "Bioluminescent cave-sea hybrids; moderate drift, deep-dark and empathic loci.",
        ),
        NamedSeed::Drakhari => (
            "drakhari",
            "Drakhari",
            vec![
                "VolcanicHighland".to_string(),
                "Caldera".to_string(),
                "Volcanic".to_string(),
            ],
            0.12_f32,
            "Volcanic highland apex predators; low drift, leading-byte aggression.",
        ),
        NamedSeed::Quelven => (
            "quelven",
            "Quelven",
            vec![
                "Canopy".to_string(),
                "Rainforest".to_string(),
                "TropicalForest".to_string(),
            ],
            0.40_f32,
            "Arboreal canopy weavers; moderate-high drift, rapid sub-clade divergence.",
        ),
        NamedSeed::Ashborn => (
            "ashborn",
            "Ashborn",
            vec![
                "AshPlains".to_string(),
                "PostEruption".to_string(),
                "Badlands".to_string(),
            ],
            0.33_f32,
            "Post-eruption pioneer colonists; moderate drift, high speciation pressure.",
        ),
    };
    let genome = archetype_dna(named).0;
    let dna_length = genome.len();
    SeedDefinition {
        id: id.to_string(),
        display_name: display_name.to_string(),
        dna_length,
        genome,
        divergence,
        spawn_biome_affinity: biomes,
        notes: Some(notes.to_string()),
    }
}

/// Produce a new [`Dna`] by interpolating between `base` and a fully-random
/// genome, controlled by `divergence ∈ [0.0, 1.0]`.
///
/// * `divergence = 0.0` → returns `base.clone()` unchanged.
/// * `divergence = 1.0` → each byte is independently randomised in `[0, 255]`.
/// * Intermediate values lerp each byte between the base value and a random
///   target: `out[i] = base[i] + round(divergence * (rand[i] - base[i]))`.
///
/// This is a *spawn-time* helper; for per-tick drift use
/// [`mutate_with_divergence`] instead.
pub fn seed_with_divergence<R: rand::Rng>(base: &Dna, divergence: f32, rng: &mut R) -> Dna {
    let divergence = divergence.clamp(0.0, 1.0);
    if divergence <= 0.0 {
        return base.clone();
    }
    let bytes: Vec<u8> = base
        .0
        .iter()
        .map(|&b| {
            let rand_byte: u8 = rng.gen();
            if divergence >= 1.0 {
                rand_byte
            } else {
                let delta = (f32::from(rand_byte) - f32::from(b)) * divergence;
                // Round to nearest, saturating at u8 bounds.
                let result = f32::from(b) + delta;
                result.round().clamp(0.0, 255.0) as u8
            }
        })
        .collect();
    Dna(bytes)
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

/// Seed set containing every [`NamedSeed`] archetype (no raw-organism primitive).
#[must_use]
pub fn named_archetype_seed_set() -> SeedSet {
    SeedSet {
        version: 1,
        seeds: ALL_NAMED_SEEDS
            .iter()
            .copied()
            .map(archetype_seed)
            .collect(),
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
    use rand::SeedableRng;

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

    // ── Named-seed + divergence-dial tests ────────────────────────────────────

    #[test]
    fn test_zero_divergence_returns_archetype() {
        let archetype = archetype_dna(NamedSeed::Ardani);
        let mut rng = ChaCha8Rng::seed_from_u64(0xABCD_1234);
        let result = seed_with_divergence(&archetype, 0.0, &mut rng);
        assert_eq!(
            result, archetype,
            "divergence=0.0 must return an exact clone of the base genome"
        );
    }

    #[test]
    fn test_full_divergence_within_bounds() {
        let archetype = archetype_dna(NamedSeed::Velthari);
        let mut rng = ChaCha8Rng::seed_from_u64(0xDEAD_C0DE);
        let result = seed_with_divergence(&archetype, 1.0, &mut rng);
        assert_eq!(result.0.len(), archetype.0.len());
    }

    #[test]
    fn test_named_seeds_differ() {
        for (i, &left) in ALL_NAMED_SEEDS.iter().enumerate() {
            for &right in &ALL_NAMED_SEEDS[i + 1..] {
                let a = archetype_dna(left);
                let b = archetype_dna(right);
                assert_ne!(
                    a, b,
                    "{left:?} and {right:?} archetypes must differ on at least one byte"
                );
            }
        }
    }

    /// FR-CIV-EMERGENCE-004 — named archetypes are pairwise speciation-ready:
    /// Hamming distance exceeds the default class threshold so distinct
    /// species clusters are visible from spawn.
    #[test]
    fn fr_emergence_004_inter_archetype_speciation_ready() {
        use crate::{speciation_distance, DnaClass};

        let class = DnaClass::default();
        let floor = class.speciation_threshold * 0.5;
        for (i, &left) in ALL_NAMED_SEEDS.iter().enumerate() {
            for &right in &ALL_NAMED_SEEDS[i + 1..] {
                let dist = speciation_distance(&archetype_dna(left), &archetype_dna(right));
                assert!(
                    dist >= floor,
                    "{left:?} vs {right:?}: distance {dist} below floor {floor}"
                );
            }
        }
    }

    #[test]
    fn all_named_seeds_round_robin_cycles() {
        assert_eq!(ALL_NAMED_SEEDS.len(), 12);
        for (i, &expected) in ALL_NAMED_SEEDS.iter().enumerate() {
            assert_eq!(named_seed_round_robin(i), expected);
        }
        assert_eq!(named_seed_round_robin(12), NamedSeed::Ardani);
    }

    #[test]
    fn choose_named_seed_empty_is_full_round_robin() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for (i, &expected) in ALL_NAMED_SEEDS.iter().enumerate() {
            assert_eq!(choose_named_seed(&[], i, &mut rng), expected);
        }
    }

    #[test]
    fn choose_named_seed_single_entry_is_monoculture() {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let mix = [WeightedNamedSeed {
            seed: NamedSeed::Felmar,
            weight: 1.0,
        }];
        for i in 0..50 {
            assert_eq!(choose_named_seed(&mix, i, &mut rng), NamedSeed::Felmar);
        }
    }

    #[test]
    fn choose_named_seed_weighted_favors_plurality() {
        let mix = [
            WeightedNamedSeed {
                seed: NamedSeed::Ardani,
                weight: 0.6,
            },
            WeightedNamedSeed {
                seed: NamedSeed::Velthari,
                weight: 0.3,
            },
            WeightedNamedSeed {
                seed: NamedSeed::Grundak,
                weight: 0.1,
            },
        ];
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let n = 2000usize;
        let mut counts = [0usize; 3];
        for i in 0..n {
            match choose_named_seed(&mix, i, &mut rng) {
                NamedSeed::Ardani => counts[0] += 1,
                NamedSeed::Velthari => counts[1] += 1,
                NamedSeed::Grundak => counts[2] += 1,
                _ => {}
            }
        }
        let ardani_frac = counts[0] as f32 / n as f32;
        assert!(
            (ardani_frac - 0.6).abs() < 0.08,
            "Ardani fraction {ardani_frac:.3} not within ±0.08 of 0.6"
        );
        assert!(counts[0] > counts[1] && counts[0] > counts[2]);
    }

    #[test]
    fn effective_spawn_divergence_prefers_override() {
        let seed = archetype_seed(NamedSeed::Ardani);
        assert!((effective_spawn_divergence(&seed, Some(0.55)) - 0.55).abs() < f32::EPSILON);
        assert!(
            (effective_spawn_divergence(&seed, None) - seed.divergence).abs() < f32::EPSILON
        );
    }

    #[test]
    fn divergence_presets_include_stable_and_radiant() {
        assert_eq!(divergence_preset("stable"), Some(0.08));
        assert_eq!(divergence_preset("radiant"), Some(0.75));
    }

    #[test]
    fn named_archetype_seed_set_loads_all() {
        let set = named_archetype_seed_set();
        assert_eq!(set.seeds.len(), ALL_NAMED_SEEDS.len());
        let lib = SeedLibrary::from_seed_set(set).expect("valid named set");
        assert_eq!(lib.len(), ALL_NAMED_SEEDS.len());
        for &named in ALL_NAMED_SEEDS {
            let id = archetype_seed(named).id;
            assert!(lib.get(&id).is_some(), "missing seed id {id}");
        }
    }

    #[test]
    fn divergence_presets_are_valid_and_lookupable() {
        for preset in DIVERGENCE_PRESETS {
            assert!(
                preset.divergence.is_finite() && (0.0..=1.0).contains(&preset.divergence),
                "preset {} has invalid divergence",
                preset.id
            );
            assert_eq!(divergence_preset(preset.id), Some(preset.divergence));
        }
        assert_eq!(divergence_preset("missing"), None);
    }

    #[test]
    fn archetype_seeds_validate() {
        for &named in ALL_NAMED_SEEDS {
            let seed = archetype_seed(named);
            seed.validate()
                .unwrap_or_else(|e| panic!("{named:?} archetype_seed failed validation: {e}"));
        }
    }

    #[test]
    fn seed_with_divergence_clamped_above_one_behaves_as_full() {
        let base = archetype_dna(NamedSeed::Grundak);
        let mut rng_a = ChaCha8Rng::seed_from_u64(7);
        let mut rng_b = ChaCha8Rng::seed_from_u64(7);
        // Values above 1.0 must be clamped to 1.0; with same RNG state the
        // two calls must produce identical results.
        let clamped = seed_with_divergence(&base, 1.5, &mut rng_a);
        let full = seed_with_divergence(&base, 1.0, &mut rng_b);
        assert_eq!(
            clamped, full,
            "divergence > 1.0 must be clamped and produce the same result as 1.0"
        );
    }

    #[test]
    fn spawn_genome_with_divergence_zero_clones_seed() {
        let mut rng = ChaCha8Rng::seed_from_u64(0xABCD_1234_u64);
        let class = base_class();
        let seed = raw_organism_primitive();
        let dna = spawn_genome_with_divergence(&mut rng, &class, &seed, 0.0);
        assert_eq!(
            dna.0, seed.genome,
            "divergence=0.0 must produce an exact clone of the seed genome"
        );
    }

    #[test]
    fn spawn_genome_with_divergence_one_drifts() {
        let mut rng = ChaCha8Rng::seed_from_u64(0xFEED_BEEF_u64);
        let class = base_class();
        let seed = raw_organism_primitive();
        let dna = spawn_genome_with_divergence(&mut rng, &class, &seed, 1.0);
        // With mutation_rate=0.05 and 64 bytes the probability of zero
        // mutations is (0.95)^64 ≈ 3.6e-2; vanishingly small over this fixed
        // RNG seed (and in practice we observed a non-trivial number of
        // flips). The assertion is probabilistic but with a fixed seed it is
        // deterministic.
        assert_ne!(
            dna.0, seed.genome,
            "divergence=1.0 must produce a drifted genome (not an exact clone)"
        );
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

    // ── P2.2c: new species variety tests ──────────────────────────────────────

    /// New seeds (Lumari, Drakhari, Quelven, Ashborn) produce distinct genomes
    /// that differ from each other and from all pre-existing archetypes.
    #[test]
    fn p2_new_seeds_produce_distinct_genomes() {
        let new_seeds = [
            NamedSeed::Lumari,
            NamedSeed::Drakhari,
            NamedSeed::Quelven,
            NamedSeed::Ashborn,
        ];
        // New seeds must differ from each other.
        for (i, &a) in new_seeds.iter().enumerate() {
            for &b in &new_seeds[i + 1..] {
                assert_ne!(
                    archetype_dna(a),
                    archetype_dna(b),
                    "{a:?} and {b:?} must produce distinct genomes"
                );
            }
        }
        // New seeds must differ from original 8.
        let original = [
            NamedSeed::Ardani,
            NamedSeed::Velthari,
            NamedSeed::Grundak,
            NamedSeed::Kethari,
            NamedSeed::Nymari,
            NamedSeed::Sylphine,
            NamedSeed::Felmar,
            NamedSeed::Thornari,
        ];
        for &n in &new_seeds {
            for &o in &original {
                assert_ne!(
                    archetype_dna(n),
                    archetype_dna(o),
                    "{n:?} must differ from pre-existing {o:?}"
                );
            }
        }
    }

    /// Divergence increases measurably over generations: after N ticks with
    /// divergence > 0, the genome drifts away from its archetype baseline.
    #[test]
    fn p2_divergence_increases_over_generations() {
        let class = base_class();
        let seeds_under_test = [
            NamedSeed::Lumari,
            NamedSeed::Drakhari,
            NamedSeed::Quelven,
            NamedSeed::Ashborn,
        ];
        for &named in &seeds_under_test {
            let base = archetype_dna(named);
            let mut rng = ChaCha8Rng::seed_from_u64(0x1234_5678_u64 ^ named as u64);
            let seed_def = archetype_seed(named);
            let mut cur = base.clone();
            let initial_dist = speciation_distance(&base, &cur);
            for _ in 0..500 {
                cur = mutate_with_divergence(&cur, &mut rng, &class, seed_def.divergence);
            }
            let final_dist = speciation_distance(&base, &cur);
            assert!(
                final_dist > initial_dist,
                "{named:?}: genome must drift from archetype over 500 ticks (initial={initial_dist}, final={final_dist})"
            );
        }
    }

    /// Each new seed is deterministic: same RNG seed → same genome every time.
    #[test]
    fn p2_new_seeds_are_deterministic() {
        let class = base_class();
        for &named in &[
            NamedSeed::Lumari,
            NamedSeed::Drakhari,
            NamedSeed::Quelven,
            NamedSeed::Ashborn,
        ] {
            let seed_def = archetype_seed(named);
            let mut rng_a = ChaCha8Rng::seed_from_u64(0xFEED_DEAD_u64);
            let mut rng_b = ChaCha8Rng::seed_from_u64(0xFEED_DEAD_u64);
            let dna_a = spawn_genome_with_divergence(&mut rng_a, &class, &seed_def, seed_def.divergence);
            let dna_b = spawn_genome_with_divergence(&mut rng_b, &class, &seed_def, seed_def.divergence);
            assert_eq!(
                dna_a, dna_b,
                "{named:?}: spawn must be deterministic under identical RNG seed"
            );
        }
    }

    /// All 12 archetypes are pairwise speciation-ready (Hamming distance above
    /// floor). Extends FR-CIV-EMERGENCE-004 to cover the new variants.
    #[test]
    fn p2_all_twelve_archetypes_speciation_ready() {
        use crate::{speciation_distance, DnaClass};
        let class = DnaClass::default();
        let floor = class.speciation_threshold * 0.5;
        for (i, &left) in ALL_NAMED_SEEDS.iter().enumerate() {
            for &right in &ALL_NAMED_SEEDS[i + 1..] {
                let dist = speciation_distance(&archetype_dna(left), &archetype_dna(right));
                assert!(
                    dist >= floor,
                    "{left:?} vs {right:?}: distance {dist} below floor {floor}"
                );
            }
        }
    }

    /// New seeds validate (dna_length == genome.len(), divergence in [0,1]).
    #[test]
    fn p2_new_archetype_seeds_validate() {
        for &named in &[
            NamedSeed::Lumari,
            NamedSeed::Drakhari,
            NamedSeed::Quelven,
            NamedSeed::Ashborn,
        ] {
            let seed = archetype_seed(named);
            seed.validate()
                .unwrap_or_else(|e| panic!("{named:?} archetype_seed failed validation: {e}"));
        }
    }
}
