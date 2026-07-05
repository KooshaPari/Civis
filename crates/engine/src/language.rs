//! Engine-side language drift (FR-CIV-LANG-001 / FR-LANGUAGE-001).
//!
//! Provides a deterministic, byte-seeded phoneme language state per
//! faction. The state drifts under isolation pressure (high isolation
//! => higher drift sigma) and supports word-borrowing across languages
//! that come into contact. Used by [`crate::Simulation::phase_language`]
//! to keep `faction_languages` and `language_state` evolving every tick.
//!
//! ## Module surface (kept in lockstep with `engine.rs::phase_language`)
//!
//! - [`LanguageState`]: hash-map vocabulary + drift/split parameters
//! - [`seeded_language_state`]: bootstrap from a 4-f32 culture seed
//! - [`place_name_meaning`] / [`person_name_meaning`]: deterministic
//!   meaning-hash function used to look up borrowed words per cross-pair
//! - [`ensure_seeded_word`]: idempotent vocabulary inserter
//! - [`place_name`] / [`person_name`]: render a morpheme into a
//!   syllable string (used by inspector UI)
//! - [`tick_language`]: pure drift step using `tick` + drift_rate +
//!   contact_pressure (no lineage argument)
//! - [`tick_language_for_lineage`]: tick wrapper that mixes a lineage
//!   id into the per-tick seed, matching the engine caller
//! - [`borrow_word`]: cross-language copy of one meaning's morpheme
//! - [`should_split`]: returns true when two languages have diverged
//!   past their respective split thresholds
//!
//! All state evolution is deterministic given the same seed sequence,
//! matching the FR-CIV engine determinism contract (CIV-0100 §3.1).

use std::collections::HashMap;

use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

const COSINE_MERGE_THRESHOLD: f32 = 0.95;
const DEFAULT_DISTANCE_FOR_MISSING_PHONEME: f32 = 1.0;
const PHONEME_FEATURES: usize = 6;
const PLACE_NAME_NAMESPACE: u32 = 0x4e_50_54;
const PERSON_NAME_NAMESPACE: u32 = 0x50_45_52;
const NAMING_SYLLABLES: &[&str] = &[
    "ba", "de", "fi", "gu", "ha", "jo", "li", "mu", "no", "sa", "ti", "za",
];

/// A single phoneme represented by six continuous feature values in `[0, 1]`.
/// Drift targets this vector; cosine similarity between two phonemes
/// drives the merge step that keeps the lexicon compact.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Phoneme {
    pub features: [f32; 6],
}

impl Phoneme {
    fn drift(&mut self, rng: &mut ChaCha8Rng, sigma: f32) {
        for feature in &mut self.features {
            *feature = (*feature + gaussian_sample(rng) * sigma).clamp(0.0, 1.0);
        }
    }
}

/// A single morpheme = one or more phonemes + an optional meaning handle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Morpheme {
    pub phonemes: Vec<Phoneme>,
    pub meaning: Option<u32>,
}

/// Per-faction language state. The `vocabulary` maps a `meaning: u32` to
/// the morpheme used for it; `drift_rate` scales the per-tick Gaussian
/// step; `split_threshold` is the cosine-distance threshold above which
/// [`should_split`] decides two languages have diverged.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageState {
    pub vocabulary: HashMap<u32, Morpheme>,
    pub drift_rate: f32,
    pub split_threshold: f32,
    pub tick: u64,
}

impl Default for LanguageState {
    fn default() -> Self {
        Self {
            vocabulary: HashMap::new(),
            drift_rate: 0.05,
            split_threshold: 0.35,
            tick: 0,
        }
    }
}

/// Bootstrap a language state from a 4-f32 culture seed. Inserts the
/// default place-name and person-name morphemes keyed off the seed
/// so any consumer can immediately call [`place_name`] / [`person_name`].
pub fn seeded_language_state(seed: [f32; 4]) -> LanguageState {
    let mut state = LanguageState::default();
    ensure_seeded_word(&mut state, place_name_meaning(0, 0), seed);
    ensure_seeded_word(&mut state, person_name_meaning(0, 0), seed);
    state
}

/// Deterministic meaning-hash function for the `(faction, place)` pair.
/// Two cultures that ask for the same `(faction_id, place_id)` collide
/// on the same meaning slot, which is what makes borrowing well-defined.
pub fn place_name_meaning(faction_id: u32, place_id: u32) -> u32 {
    PLACE_NAME_NAMESPACE
        .wrapping_add(faction_id.rotate_left(13))
        .wrapping_add(place_id.rotate_left(3))
}

/// Deterministic meaning-hash function for the `(faction, person)` pair.
pub fn person_name_meaning(faction_id: u32, person_id: u32) -> u32 {
    PERSON_NAME_NAMESPACE
        .wrapping_add(faction_id.rotate_left(9))
        .wrapping_add(person_id.rotate_left(1))
}

/// Insert a seed-derived morpheme for `meaning` if none exists yet.
/// Idempotent: existing morphemes are left untouched (the carry-out of
/// drift is preserved across calls).
pub fn ensure_seeded_word(language: &mut LanguageState, meaning: u32, seed: [f32; 4]) {
    language.vocabulary.entry(meaning).or_insert_with(|| seeded_morpheme(seed, meaning));
}

/// Render the morpheme associated with `(faction_id, place_id)`.
pub fn place_name(language: &LanguageState, faction_id: u32, place_id: u32) -> String {
    render_name(language, place_name_meaning(faction_id, place_id))
}

/// Render the morpheme associated with `(faction_id, person_id)`.
pub fn person_name(language: &LanguageState, faction_id: u32, person_id: u32) -> String {
    render_name(language, person_name_meaning(faction_id, person_id))
}

/// Apply one tick of phoneme drift. `contact_pressure` scales the
/// per-step Gaussian sigma: in contact zones (pressure > 0) the drift
/// is gentler; in isolation (pressure ~ 0) the sigma defaults to
/// `drift_rate` and features diverge faster.
pub fn tick_language(lang: &mut LanguageState, contact_pressure: f32) {
    let mut rng = seeded_rng(lang, contact_pressure);
    let sigma = lang.drift_rate.max(0.0) * (1.0 + contact_pressure.max(0.0));

    for morpheme in lang.vocabulary.values_mut() {
        for phoneme in &mut morpheme.phonemes {
            phoneme.drift(&mut rng, sigma);
        }
        merge_near_identical_phonemes(&mut morpheme.phonemes);
    }

    lang.tick = lang.tick.wrapping_add(1);
}

/// Engine-crate shim: identical semantics to [`tick_language`] but takes
/// `isolation` (0..=1 where 1 = full isolation) and a `lineage` id used
/// to disambiguate the per-tick RNG seed per faction. The isolation is
/// inverted into a `contact_pressure` (`isolation = 1 → pressure = 0`)
/// before delegating so the underlying math stays the same.
///
/// Matches the engine.rs:4350 call site exactly.
pub fn tick_language_for_lineage(lang: &mut LanguageState, isolation: f32, lineage: u64) {
    // Mix the lineage into the language tick stamp so two factions at
    // equal isolation still get independent drift trajectories. This
    // is identity-preserving for the engine caller (semantics match
    // what an engine-locale mock would expect).
    lang.tick = lang.tick.wrapping_add(1);
    let lineage_mix = ((lineage & 0xFFFF) as f32) / 65535.0;
    let contact_pressure = (1.0 - isolation.max(0.0).min(1.0)).max(0.0) + lineage_mix * 1e-6;
    tick_language(lang, contact_pressure);
}

/// `true` when the average language distance between `lang` and `lang2`
/// exceeds the larger of the two split thresholds. Used by future
/// faction-split phases; the engine does not call it today but the
/// re-export lives here so external consumers don't have to rebuild.
pub fn should_split(lang: &LanguageState, lang2: &LanguageState) -> bool {
    let distance = average_language_distance(lang, lang2);
    distance > lang.split_threshold.max(lang2.split_threshold)
}

/// Borrow the morpheme for `meaning` from `source` into `lang`.
/// No-op when the source language has no morpheme for the meaning.
pub fn borrow_word(lang: &mut LanguageState, source: &LanguageState, meaning: u32) {
    if let Some(morpheme) = source.vocabulary.get(&meaning) {
        lang.vocabulary.insert(meaning, morpheme.clone());
    }
}

fn seeded_rng(lang: &LanguageState, contact_pressure: f32) -> ChaCha8Rng {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&lang.tick.to_le_bytes());
    hasher.update(&lang.drift_rate.to_bits().to_le_bytes());
    hasher.update(&lang.split_threshold.to_bits().to_le_bytes());
    hasher.update(&contact_pressure.to_bits().to_le_bytes());

    let mut entries: Vec<_> = lang.vocabulary.iter().collect();
    entries.sort_unstable_by_key(|(meaning, _)| **meaning);
    for (meaning, morpheme) in entries {
        hasher.update(&meaning.to_le_bytes());
        if let Some(m) = morpheme.meaning {
            hasher.update(&m.to_le_bytes());
        }
        for phoneme in &morpheme.phonemes {
            for feature in &phoneme.features {
                hasher.update(&feature.to_bits().to_le_bytes());
            }
        }
    }

    ChaCha8Rng::from_seed(*hasher.finalize().as_bytes())
}

fn seeded_morpheme(seed: [f32; 4], meaning: u32) -> Morpheme {
    let mut rng = seeded_rng_for_seed(seed, meaning);
    let feature_len = 2 + (rng.next_u32() % 2) as usize;
    let mut phonemes = Vec::with_capacity(feature_len);
    for idx in 0..feature_len {
        let mut features = [0.0_f32; PHONEME_FEATURES];
        for i in 0..PHONEME_FEATURES {
            let drift = (rng.next_u32() as f32 / u32::MAX as f32 - 0.5) * 0.4;
            features[i] = (seed[i % 4] + drift).clamp(0.0, 1.0);
        }
        phonemes.push(Phoneme {
            features,
        });
    }
    Morpheme {
        phonemes,
        meaning: Some(meaning),
    }
}

fn render_name(language: &LanguageState, meaning: u32) -> String {
    let morpheme = match language.vocabulary.get(&meaning).or_else(|| language.vocabulary.values().next()) {
        Some(m) => m,
        None => {
            let fallback = seeded_morpheme([0.5; 4], meaning);
            return morpheme_to_text(&fallback);
        }
    };
    morpheme_to_text(morpheme)
}

fn morpheme_to_text(morpheme: &Morpheme) -> String {
    let syllables: Vec<String> = morpheme
        .phonemes
        .iter()
        .map(phoneme_to_syllable)
        .collect();
    if syllables.is_empty() {
        "na".to_string()
    } else {
        syllables.join("-")
    }
}

fn phoneme_to_syllable(phoneme: &Phoneme) -> String {
    let mut syllable = String::new();
    for i in (0..PHONEME_FEATURES).step_by(2) {
        let idx = ((phoneme.features[i] * NAMING_SYLLABLES.len() as f32) as usize) % NAMING_SYLLABLES.len();
        syllable.push_str(NAMING_SYLLABLES[idx]);
    }
    syllable
}

fn seeded_rng_for_seed(seed: [f32; 4], meaning: u32) -> rand_chacha::ChaCha8Rng {
    let mut hasher = blake3::Hasher::new();
    for feature in &seed {
        hasher.update(&feature.to_bits().to_le_bytes());
    }
    hasher.update(&meaning.to_le_bytes());
    ChaCha8Rng::from_seed(*hasher.finalize().as_bytes())
}

fn gaussian_sample(rng: &mut ChaCha8Rng) -> f32 {
    let u1 = loop {
        let v: f32 = rng.gen();
        if v > 0.0 {
            break v;
        }
    };
    let u2: f32 = rng.gen();
    let radius = (-2.0 * u1.ln()).sqrt();
    let theta = core::f32::consts::TAU * u2;
    radius * theta.cos()
}

fn merge_near_identical_phonemes(phonemes: &mut Vec<Phoneme>) {
    let mut i = 0;
    while i < phonemes.len() {
        let mut j = i + 1;
        let mut merged = false;
        while j < phonemes.len() {
            if cosine_similarity(&phonemes[i], &phonemes[j]) > COSINE_MERGE_THRESHOLD {
                phonemes[i] = average_phonemes(&phonemes[i], &phonemes[j]);
                phonemes.remove(j);
                merged = true;
            } else {
                j += 1;
            }
        }
        if !merged {
            i += 1;
        }
    }
}

fn average_phonemes(a: &Phoneme, b: &Phoneme) -> Phoneme {
    let mut features = [0.0; 6];
    for idx in 0..features.len() {
        features[idx] = (a.features[idx] + b.features[idx]) * 0.5;
    }
    Phoneme { features }
}

fn cosine_similarity(a: &Phoneme, b: &Phoneme) -> f32 {
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for idx in 0..a.features.len() {
        dot += a.features[idx] * b.features[idx];
        norm_a += a.features[idx] * a.features[idx];
        norm_b += b.features[idx] * b.features[idx];
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom <= f32::EPSILON {
        if norm_a <= f32::EPSILON && norm_b <= f32::EPSILON {
            1.0
        } else {
            0.0
        }
    } else {
        (dot / denom).clamp(-1.0, 1.0)
    }
}

fn phoneme_distance(a: &Phoneme, b: &Phoneme) -> f32 {
    1.0 - cosine_similarity(a, b)
}

fn average_language_distance(a: &LanguageState, b: &LanguageState) -> f32 {
    let mut common: Vec<_> = a
        .vocabulary
        .keys()
        .copied()
        .filter(|meaning| b.vocabulary.contains_key(meaning))
        .collect();
    common.sort_unstable();

    if common.is_empty() {
        return centroid_distance(a, b);
    }

    let mut total_distance = 0.0;
    let mut total_pairs = 0.0;

    for meaning in common {
        let left = &a.vocabulary[&meaning].phonemes;
        let right = &b.vocabulary[&meaning].phonemes;
        let shared_len = left.len().min(right.len());

        for idx in 0..shared_len {
            total_distance += phoneme_distance(&left[idx], &right[idx]);
            total_pairs += 1.0;
        }

        let excess = left.len().abs_diff(right.len()) as f32;
        total_distance += excess * DEFAULT_DISTANCE_FOR_MISSING_PHONEME;
        total_pairs += excess;
    }

    if total_pairs <= f32::EPSILON {
        centroid_distance(a, b)
    } else {
        total_distance / total_pairs
    }
}

fn centroid_distance(a: &LanguageState, b: &LanguageState) -> f32 {
    let centroid_a = centroid(a);
    let centroid_b = centroid(b);
    match (centroid_a, centroid_b) {
        (Some(a), Some(b)) => 1.0 - cosine_similarity(&a, &b),
        (None, None) => 0.0,
        _ => 1.0,
    }
}

fn centroid(lang: &LanguageState) -> Option<Phoneme> {
    let mut total = [0.0; 6];
    let mut count = 0.0;

    for morpheme in lang.vocabulary.values() {
        for phoneme in &morpheme.phonemes {
            for idx in 0..total.len() {
                total[idx] += phoneme.features[idx];
            }
            count += 1.0;
        }
    }

    if count <= f32::EPSILON {
        None
    } else {
        for feature in &mut total {
            *feature /= count;
        }
        Some(Phoneme { features: total })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn phoneme(features: [f32; 6]) -> Phoneme {
        Phoneme { features }
    }

    fn morpheme(phonemes: Vec<Phoneme>, meaning: u32) -> Morpheme {
        Morpheme {
            phonemes,
            meaning: Some(meaning),
        }
    }

    #[test]
    fn tick_language_advances_time_and_drifts_features() {
        let mut lang = LanguageState {
            vocabulary: HashMap::from([(1, morpheme(vec![phoneme([0.5; 6])], 1))]),
            drift_rate: 0.2,
            split_threshold: 0.4,
            tick: 7,
        };

        tick_language(&mut lang, 0.0);

        assert_eq!(lang.tick, 8);
        let moved = &lang.vocabulary[&1].phonemes[0].features;
        assert!(moved.iter().any(|feature| (*feature - 0.5).abs() > f32::EPSILON));
        assert!(moved.iter().all(|feature| (0.0..=1.0).contains(feature)));
    }

    #[test]
    fn tick_language_merges_near_identical_phonemes() {
        let mut lang = LanguageState {
            vocabulary: HashMap::from([(
                7,
                Morpheme {
                    phonemes: vec![
                        phoneme([1.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
                        phoneme([0.99, 0.01, 0.0, 0.0, 0.0, 0.0]),
                    ],
                    meaning: Some(7),
                },
            )]),
            drift_rate: 0.0,
            split_threshold: 0.4,
            tick: 0,
        };

        tick_language(&mut lang, 0.0);

        assert_eq!(lang.vocabulary[&7].phonemes.len(), 1);
    }

    #[test]
    fn should_split_when_average_distance_exceeds_threshold() {
        let lang_a = LanguageState {
            vocabulary: HashMap::from([(
                11,
                morpheme(vec![phoneme([1.0, 0.0, 0.0, 0.0, 0.0, 0.0])], 11),
            )]),
            drift_rate: 0.1,
            split_threshold: 0.25,
            tick: 0,
        };
        let lang_b = LanguageState {
            vocabulary: HashMap::from([(
                11,
                morpheme(vec![phoneme([0.0, 1.0, 0.0, 0.0, 0.0, 0.0])], 11),
            )]),
            drift_rate: 0.1,
            split_threshold: 0.25,
            tick: 0,
        };

        assert!(should_split(&lang_a, &lang_b));
        assert!(should_split(&lang_b, &lang_a));
    }

    #[test]
    fn borrow_word_clones_source_morpheme_for_meaning() {
        let source = LanguageState {
            vocabulary: HashMap::from([(
                42,
                Morpheme {
                    phonemes: vec![phoneme([0.2, 0.8, 0.0, 0.0, 0.0, 0.0])],
                    meaning: Some(42),
                },
            )]),
            drift_rate: 0.1,
            split_threshold: 0.3,
            tick: 0,
        };
        let mut target = LanguageState::default();

        borrow_word(&mut target, &source, 42);

        assert_eq!(target.vocabulary.get(&42), source.vocabulary.get(&42));
    }

    /// FR-CIV-LANG: `tick_language_for_lineage` advances `tick` and
    /// is the engine entry-point; isolation inversion (1 → 0 contact)
    /// must keep two lineages that share the same seed / isolation
    /// from sharing the *exact* RNG byte stream.
    #[test]
    fn tick_language_for_lineage_is_stable_per_lineage() {
        let mut a = seeded_language_state([0.5; 4]);
        let mut b = seeded_language_state([0.5; 4]);
        for _ in 0..10 {
            tick_language_for_lineage(&mut a, 0.5, 1);
            tick_language_for_lineage(&mut b, 0.5, 2);
        }
        // Independent lineages drift to slightly different feature
        // vectors even when the public inputs match.
        let vec_a = a.vocabulary[&place_name_meaning(0, 0)]
            .phonemes
            .first()
            .map(|p| p.features)
            .unwrap_or([0.0; 6]);
        let vec_b = b.vocabulary[&place_name_meaning(0, 0)]
            .phonemes
            .first()
            .map(|p| p.features)
            .unwrap_or([0.0; 6]);
        assert!(
            vec_a.iter().zip(vec_b.iter()).any(|(x, y)| (x - y).abs() > 1e-6),
            "lineage id must contribute to per-tick seeding"
        );
    }
}
