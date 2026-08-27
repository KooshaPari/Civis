// TODO(cleanup-surgeon): stub module — `language` types were removed by an
// earlier lane. `engine.rs:65` still imports `crate::language`. Restore the
// original or rewrite callers.
//
// This stub re-exports the engine-side `LanguageState` placeholder and the
// `ensure_seeded_word` / `borrow_word` / `tick_language_for_lineage` /
// `seeded_language_state` / `place_name` / `person_name` / `place_name_meaning`
// / `person_name_meaning` / `faction_isolation_pressure` functions consumed by
// `phase_language`. They return safe defaults so the engine compiles.

pub use crate::engine::LanguageState;
use std::collections::{BTreeMap, BTreeSet};

/// Stub kind of seedable word. Mirrors the old `WordKind` enum; restore when
/// the language module is re-stitched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordKind {
    Place,
    Person,
}

/// Build a `LanguageState` seeded from `signature` (or zero if `None`).
#[must_use]
pub fn seeded_language_state(signature: [f32; 4]) -> LanguageState {
    LanguageState {
        seed_signature: signature,
        ..LanguageState::default()
    }
}

/// Ensure the `state` knows about a word for `kind` at `meaning`. Stub: no-op.
pub fn ensure_seeded_word(state: &mut LanguageState, kind: WordKind, meaning: [f32; 4]) {
    let key = match kind {
        WordKind::Place => "place",
        WordKind::Person => "person",
    };
    state.lexemes.push(format!(
        "{key}:{}",
        meaning[0] + meaning[1] + meaning[2] + meaning[3]
    ));
}

/// Borrow a word from `source` into `target`. Stub: no-op.
pub fn borrow_word(target: &mut LanguageState, _source: &LanguageState, kind: WordKind) {
    let key = match kind {
        WordKind::Place => "borrow:place",
        WordKind::Person => "borrow:person",
    };
    target.lexemes.push(key.to_string());
}

/// Advance one language lineage tick under `isolation` pressure. Stub: no-op.
pub fn tick_language_for_lineage(state: &mut LanguageState, isolation: f32, _lineage_id: u64) {
    state.drift_rate = (state.drift_rate + isolation * 0.01).clamp(0.0, 1.0);
}

/// Stub: derive a `place_name_meaning` for `faction_id` at `meaning`.
#[must_use]
pub fn place_name_meaning(faction_id: u32, meaning: u32) -> WordKind {
    let _ = (faction_id, meaning);
    WordKind::Place
}

/// Stub: derive a `person_name_meaning` for `faction_id` at `meaning`.
#[must_use]
pub fn person_name_meaning(faction_id: u32, meaning: u32) -> WordKind {
    let _ = (faction_id, meaning);
    WordKind::Person
}

/// Stub: derive a place name for `(state, faction_id, place_id)`.
#[must_use]
pub fn place_name(_state: &LanguageState, faction_id: u32, place_id: u32) -> String {
    format!("place-{faction_id}-{place_id}")
}

/// Stub: derive a person name for `(state, faction_id, person_id)`.
#[must_use]
pub fn person_name(_state: &LanguageState, faction_id: u32, person_id: u32) -> String {
    format!("person-{faction_id}-{person_id}")
}

/// Per-pair isolation pressure (0..1). Higher = more isolated.
#[must_use]
pub fn faction_isolation_pressure(
    faction_id: u32,
    _dominant: &BTreeMap<u64, u32>,
    _member_counts: &BTreeMap<u64, u32>,
    _contacts: &BTreeSet<(u64, u64)>,
) -> f32 {
    let _ = faction_id;
    0.5
}

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRule {
    pub pattern: String,
    pub replacement: String,
    pub frequency: f32,
}

impl Default for GrammarRule {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            replacement: String::new(),
            frequency: 1.0,
        }
    }
}

/// A full language with phonemes, grammar, vocabulary, and drift.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub phonemes: Vec<String>,
    pub grammar_rules: Vec<GrammarRule>,
    pub vocabulary: HashMap<String, String>,
    pub drift_factor: f32,
    pub intelligibility_baseline: f32,
    pub created_tick: u64,
}

impl Default for Language {
    fn default() -> Self {
        Self {
            name: String::new(),
            phonemes: Vec::new(),
            grammar_rules: Vec::new(),
            vocabulary: HashMap::new(),
            drift_factor: 0.0,
            intelligibility_baseline: 1.0,
            created_tick: 0,
        }
    }
}

/// Create a new language with given name, phonemes, and creation tick.
#[must_use]
pub fn create_language(name: &str, phonemes: Vec<String>, tick: u64) -> Language {
    Language {
        name: name.to_string(),
        phonemes,
        grammar_rules: Vec::new(),
        vocabulary: HashMap::new(),
        drift_factor: 0.01,
        intelligibility_baseline: 1.0,
        created_tick: tick,
    }
}

/// Evolve a language by one tick with deterministic drift via hash.
#[must_use]
pub fn evolve_language(lang: &Language, tick: u64) -> Language {
    let mut hasher = DefaultHasher::new();
    lang.name.hash(&mut hasher);
    tick.hash(&mut hasher);
    let hash = hasher.finish();
    let drift = (hash as f32 / u64::MAX as f32) * lang.drift_factor;
    Language {
        intelligibility_baseline: (lang.intelligibility_baseline - drift).max(0.1),
        drift_factor: (lang.drift_factor + drift * 0.01).min(0.5),
        ..lang.clone()
    }
}

/// Loan a word from source language into target language.
#[must_use]
pub fn loan_word(target: &Language, source: &Language, meaning: &str) -> Language {
    let mut new_target = target.clone();
    if let Some(word) = source.vocabulary.get(meaning) {
        new_target
            .vocabulary
            .insert(meaning.to_string(), word.clone());
    }
    new_target
}

/// Compute mutual intelligibility between two languages (0.0 = none, 1.0 = identical).
#[must_use]
pub fn compute_mutual_intelligibility(a: &Language, b: &Language) -> f32 {
    if a.name == b.name {
        return 1.0;
    }
    let shared = a
        .vocabulary
        .keys()
        .filter(|k| b.vocabulary.contains_key(*k))
        .count();
    let total = a.vocabulary.len().max(b.vocabulary.len()).max(1);
    shared as f32 / total as f32
}

/// Add a new vocabulary word to a language.
#[must_use]
pub fn add_vocabulary(lang: &Language, meaning: &str, word: &str) -> Language {
    let mut new_lang = lang.clone();
    new_lang
        .vocabulary
        .insert(meaning.to_string(), word.to_string());
    new_lang
}

/// Advance a language system by one tick.
#[must_use]
pub fn tick_language_system(lang: &Language, tick: u64) -> Language {
    evolve_language(lang, tick)
}

#[cfg(test)]
mod language_extended_tests {
    use super::*;

    #[test]
    fn create_language_basic() {
        let lang = create_language("Common", vec!["a".into(), "b".into(), "c".into()], 10);
        assert_eq!(lang.name, "Common");
        assert_eq!(lang.phonemes.len(), 3);
        assert_eq!(lang.created_tick, 10);
    }

    #[test]
    fn create_language_empty() {
        let lang = create_language("Empty", vec![], 0);
        assert!(lang.phonemes.is_empty());
        assert_eq!(lang.drift_factor, 0.01);
    }

    #[test]
    fn evolve_language_deterministic() {
        let lang = create_language("Test", vec![], 5);
        let a = evolve_language(&lang, 100);
        let b = evolve_language(&lang, 100);
        assert_eq!(a.intelligibility_baseline, b.intelligibility_baseline);
    }

    #[test]
    fn evolve_language_changes_intelligibility() {
        let lang = create_language("Test", vec![], 5);
        let evolved = evolve_language(&lang, 100);
        assert_ne!(
            evolved.intelligibility_baseline,
            lang.intelligibility_baseline
        );
    }

    #[test]
    fn loan_word_transfers_meaning() {
        let src = add_vocabulary(&create_language("Src", vec![], 0), "water", "agua");
        let tgt = create_language("Tgt", vec![], 0);
        let result = loan_word(&tgt, &src, "water");
        assert_eq!(result.vocabulary.get("water").unwrap(), "agua");
    }

    #[test]
    fn loan_word_missing_meaning() {
        let src = create_language("Src", vec![], 0);
        let tgt = create_language("Tgt", vec![], 0);
        let result = loan_word(&tgt, &src, "nonexistent");
        assert!(result.vocabulary.is_empty());
    }

    #[test]
    fn compute_mutual_intelligibility_same() {
        let lang = create_language("Test", vec![], 0);
        assert!((compute_mutual_intelligibility(&lang, &lang) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_mutual_intelligibility_disjoint() {
        let a = add_vocabulary(&create_language("A", vec![], 0), "x", "1");
        let b = add_vocabulary(&create_language("B", vec![], 0), "y", "2");
        let score = compute_mutual_intelligibility(&a, &b);
        assert!((score - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_mutual_intelligibility_partial() {
        let a = add_vocabulary(
            &add_vocabulary(&create_language("A", vec![], 0), "x", "1"),
            "y",
            "2",
        );
        let b = add_vocabulary(&create_language("B", vec![], 0), "x", "3");
        let score = compute_mutual_intelligibility(&a, &b);
        assert!(score > 0.0 && score < 1.0);
    }

    #[test]
    fn add_vocabulary_extends() {
        let lang = create_language("Test", vec![], 0);
        let lang = add_vocabulary(&lang, "fire", "ignis");
        let lang = add_vocabulary(&lang, "water", "aqua");
        assert_eq!(lang.vocabulary.len(), 2);
    }

    #[test]
    fn tick_language_system_delegates() {
        let lang = create_language("Test", vec![], 5);
        let ticked = tick_language_system(&lang, 100);
        let evolved = evolve_language(&lang, 100);
        assert_eq!(
            ticked.intelligibility_baseline,
            evolved.intelligibility_baseline
        );
    }

    #[test]
    fn grammar_rule_default() {
        let rule = GrammarRule::default();
        assert!(rule.pattern.is_empty());
        assert_eq!(rule.frequency, 1.0);
    }
}
