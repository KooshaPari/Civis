use std::collections::HashMap;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

const COSINE_MERGE_THRESHOLD: f32 = 0.95;
const DEFAULT_DISTANCE_FOR_MISSING_PHONEME: f32 = 1.0;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Morpheme {
    pub phonemes: Vec<Phoneme>,
    pub meaning: Option<u32>,
}

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

pub fn should_split(lang: &LanguageState, lang2: &LanguageState) -> bool {
    let distance = average_language_distance(lang, lang2);
    distance > lang.split_threshold.max(lang2.split_threshold)
}

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
}
