// Language module — emergent linguistic simulation for the Civis engine.
//
// Provides language creation, evolution, vocabulary management, loan-words,
// mutual intelligibility, language family trees, vocabulary drift, lingua
// franca adoption, script evolution, and translation difficulty.

pub use crate::engine::LanguageState;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Stub kind of seedable word. Mirrors the old `WordKind` enum; restore when
/// the language module is re-stitched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordKind {
    Place,
    Person,
}

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

// ---------------------------------------------------------------------------
// Legacy stub functions (seeded_language_state, ensure_seeded_word, etc.)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Core language functions
// ---------------------------------------------------------------------------

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

// ===========================================================================
// 1. Language Family Tree
// ===========================================================================

/// A single node in a language family tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageFamily {
    /// Human-readable name (e.g. "Proto-Indo-European").
    pub name: String,
    /// `None` for the root proto-language.
    pub parent_id: Option<usize>,
    /// Indices of direct child families.
    pub children: Vec<usize>,
    /// Tick at which this language diverged from its parent.
    pub divergence_tick: u64,
}

/// A tree of related languages tracking proto-language to divergence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageFamilyTree {
    pub families: Vec<LanguageFamily>,
    /// Index of the root proto-language node.
    pub root_id: usize,
}

impl LanguageFamilyTree {
    /// Create a new tree with a single root proto-language at tick 0.
    #[must_use]
    pub fn new(root_name: &str) -> Self {
        let root = LanguageFamily {
            name: root_name.to_string(),
            parent_id: None,
            children: Vec::new(),
            divergence_tick: 0,
        };
        Self {
            families: vec![root],
            root_id: 0,
        }
    }

    /// Diverge a child language from `parent_id` at the given `tick`.
    /// Returns the child's id (index into `families`).
    ///
    /// # Panics
    /// Panics if `parent_id` is out of bounds.
    pub fn diverge(&mut self, parent_id: usize, child_name: &str, tick: u64) -> usize {
        assert!(
            parent_id < self.families.len(),
            "parent_id {parent_id} out of bounds (len {})",
            self.families.len()
        );
        let child_id = self.families.len();
        self.families[parent_id].children.push(child_id);
        self.families.push(LanguageFamily {
            name: child_name.to_string(),
            parent_id: Some(parent_id),
            children: Vec::new(),
            divergence_tick: tick,
        });
        child_id
    }

    /// Return the ancestor chain from `id` up to (and including) the root.
    ///
    /// # Panics
    /// Panics if `id` is out of bounds.
    #[must_use]
    pub fn get_lineage(&self, id: usize) -> Vec<usize> {
        assert!(
            id < self.families.len(),
            "id {id} out of bounds (len {})",
            self.families.len()
        );
        let mut chain = vec![id];
        let mut current = id;
        while let Some(parent) = self.families[current].parent_id {
            chain.push(parent);
            current = parent;
        }
        chain
    }

    /// Return `true` if two language ids share a common ancestor
    /// (i.e. are in the same family tree).
    ///
    /// # Panics
    /// Panics if either id is out of bounds.
    #[must_use]
    pub fn are_related(&self, a: usize, b: usize) -> bool {
        assert!(
            a < self.families.len(),
            "a {a} out of bounds (len {})",
            self.families.len()
        );
        assert!(
            b < self.families.len(),
            "b {b} out of bounds (len {})",
            self.families.len()
        );
        if a == b {
            return true;
        }
        // Walk a's lineage upward, check for overlap with b's lineage.
        let lineage_a: BTreeSet<usize> = self.get_lineage(a).into_iter().collect();
        self.get_lineage(b).iter().any(|id| lineage_a.contains(id))
    }
}

// ===========================================================================
// 2. Vocabulary Drift
// ===========================================================================

/// Parameters controlling how vocabulary mutates over generations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyDrift {
    /// Base rate at which words drift per tick (0.0-1.0).
    pub drift_rate: f32,
    /// Probability that any given word undergoes character mutation (0.0-1.0).
    pub mutation_probability: f32,
    /// Probability that any given word is lost entirely (0.0-1.0).
    pub loss_probability: f32,
}

/// Deterministically mutate a language's vocabulary by one drift step.
///
/// Uses a seeded hasher based on the language name + tick for reproducibility.
/// For each vocabulary entry, the function decides (via the hash stream)
/// whether to mutate the word, lose it, or leave it unchanged.
#[must_use]
pub fn drift_vocabulary(lang: &Language, drift: &VocabularyDrift, tick: u64) -> Language {
    // When drift_rate is zero, nothing changes.
    if drift.drift_rate <= 0.0 {
        return lang.clone();
    }

    let mut hasher = DefaultHasher::new();
    lang.name.hash(&mut hasher);
    tick.hash(&mut hasher);
    let seed = hasher.finish();

    let mut new_vocabulary = HashMap::new();
    for (meaning, word) in &lang.vocabulary {
        // Derive a per-word hash from the seed + meaning.
        let mut word_hasher = DefaultHasher::new();
        seed.hash(&mut word_hasher);
        meaning.hash(&mut word_hasher);
        let h = word_hasher.finish();
        let bucket = (h as f32 / u64::MAX as f32) * drift.drift_rate;

        if bucket < drift.loss_probability {
            // Word is lost.
            continue;
        } else if bucket < drift.loss_probability + drift.mutation_probability {
            // Mutate: shift characters deterministically.
            let mutated = mutate_word(word, h);
            new_vocabulary.insert(meaning.clone(), mutated);
        } else {
            // Unchanged.
            new_vocabulary.insert(meaning.clone(), word.clone());
        }
    }

    Language {
        vocabulary: new_vocabulary,
        ..lang.clone()
    }
}

/// Deterministically mutate a word's characters using a hash seed.
/// Shifts each character by a small offset derived from the hash.
fn mutate_word(word: &str, seed: u64) -> String {
    let mut result = String::with_capacity(word.len());
    for (i, ch) in word.chars().enumerate() {
        let shift = ((seed >> (i % 8 * 4)) & 0xF) as i8 - 8; // range: -8..=7
        if ch.is_ascii_alphabetic() {
            let base = if ch.is_ascii_lowercase() { b'a' } else { b'A' };
            let offset = (ch as u8 - base) as i16 + shift as i16;
            let wrapped = ((offset % 26 + 26) % 26) as u8;
            result.push((base + wrapped) as char);
        } else {
            result.push(ch);
        }
    }
    result
}

/// Compute a Levenshtein-like edit distance between two words,
/// normalized to [0.0, 1.0] where 0.0 = identical and 1.0 = maximally different.
#[must_use]
pub fn word_distance(word_a: &str, word_b: &str) -> f32 {
    let a: Vec<char> = word_a.chars().collect();
    let b: Vec<char> = word_b.chars().collect();
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 && b_len == 0 {
        return 0.0;
    }

    // Standard Levenshtein DP.
    let mut dp = vec![vec![0u32; b_len + 1]; a_len + 1];
    for (i, row) in dp.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i as u32;
    }
    for j in 0..=b_len {
        dp[0][j] = j as u32;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    let max_len = a_len.max(b_len) as f32;
    dp[a_len][b_len] as f32 / max_len
}

// ===========================================================================
// 3. Lingua Franca Mechanics
// ===========================================================================

/// State for a lingua franca (dominant language) and its adoption dynamics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinguaFranca {
    /// Which language id is the lingua franca.
    pub language_id: usize,
    /// Current dominance level (0.0 = none, 1.0 = total).
    pub dominance: f32,
    /// Base adoption rate per tick (0.0-1.0).
    pub adoption_rate: f32,
    /// Resistance of other populations to adoption (0.0-1.0).
    pub resistance: f32,
}

/// Compute the adoption rate for a lingua franca into a target population.
///
/// Factors:
/// - `lingua.dominance` amplifies adoption.
/// - Larger `target_population` increases inertia (slower adoption).
/// - Higher `trade_contact` accelerates adoption.
/// - `lingua.resistance` dampens adoption.
///
/// Returns a value clamped to [0.0, 1.0].
#[must_use]
pub fn compute_adoption_rate(
    lingua: &LinguaFranca,
    target_population: u32,
    trade_contact: f32,
) -> f32 {
    let pop_factor = 1.0 / (1.0 + target_population as f32 * 0.000_01);
    let raw = lingua.adoption_rate * lingua.dominance * (1.0 + trade_contact) * pop_factor;
    let resisted = raw * (1.0 - lingua.resistance);
    resisted.clamp(0.0, 1.0)
}

/// Blend vocabulary from a lingua franca into a target language.
///
/// `adoption_strength` (0.0-1.0) determines the fraction of vocabulary
/// entries that get replaced with lingua franca words for shared meanings.
#[must_use]
pub fn apply_lingua_franca(
    lang: &Language,
    lingua_lang: &Language,
    adoption_strength: f32,
) -> Language {
    let strength = adoption_strength.clamp(0.0, 1.0);
    let mut new_lang = lang.clone();

    for (meaning, lingua_word) in &lingua_lang.vocabulary {
        // Deterministic decision per meaning using a hash.
        let mut hasher = DefaultHasher::new();
        lang.name.hash(&mut hasher);
        meaning.hash(&mut hasher);
        let h = hasher.finish();
        let bucket = h as f32 / u64::MAX as f32;

        if bucket < strength {
            new_lang
                .vocabulary
                .insert(meaning.clone(), lingua_word.clone());
        }
    }

    new_lang
}

// ===========================================================================
// 4. Writing System Evolution
// ===========================================================================

/// Stage of writing-system evolution, from pictographs to alphabets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ScriptEvolution {
    /// Cave paintings, petroglyphs -- one glyph per concept.
    Pictographic,
    /// Logograms -- one symbol per word/morpheme (e.g. early Chinese).
    Ideographic,
    /// Each symbol represents a syllable (e.g. Japanese kana).
    Syllabic,
    /// Each symbol represents a phoneme (e.g. Latin alphabet).
    Alphabetic,
}

/// Progress a script to the next evolutionary stage when `complexity_factor`
/// exceeds internal thresholds.  Returns the new (possibly unchanged) stage.
///
/// Thresholds (deterministic):
/// - Ideographic: complexity >= 0.25
/// - Syllabic: complexity >= 0.50
/// - Alphabetic: complexity >= 0.75
#[must_use]
pub fn evolve_script(current: ScriptEvolution, complexity_factor: f32) -> ScriptEvolution {
    match current {
        ScriptEvolution::Pictographic if complexity_factor >= 0.25 => {
            ScriptEvolution::Ideographic
        }
        ScriptEvolution::Ideographic if complexity_factor >= 0.50 => ScriptEvolution::Syllabic,
        ScriptEvolution::Syllabic if complexity_factor >= 0.75 => ScriptEvolution::Alphabetic,
        other => other,
    }
}

/// Return the approximate number of unique symbols needed for this script
/// stage.  Used as a rough complexity metric.
///
/// Ordering: Pictographic > Ideographic > Syllabic > Alphabetic.
#[must_use]
pub fn script_complexity(script: ScriptEvolution) -> u32 {
    match script {
        ScriptEvolution::Pictographic => 10_000, // one glyph per concept
        ScriptEvolution::Ideographic => 5_000,   // logograms combine components
        ScriptEvolution::Syllabic => 200,        // ~100-300 syllables
        ScriptEvolution::Alphabetic => 40,       // 20-40 letters
    }
}

// ===========================================================================
// 5. Translation Difficulty
// ===========================================================================

/// Compute translation difficulty between two languages.
///
/// Returns 0.0 (trivial / identical) to 1.0 (impossible).
///
/// Factors:
/// - Vocabulary overlap (inverse of distance).
/// - Grammar rule similarity.
/// - Phoneme set distance (Jaccard).
#[must_use]
pub fn translation_difficulty(a: &Language, b: &Language) -> f32 {
    if a.name == b.name {
        return 0.0;
    }

    // --- Vocabulary overlap ---
    let shared_vocab: u32 = a
        .vocabulary
        .keys()
        .filter(|k| b.vocabulary.contains_key(*k))
        .count() as u32;
    let vocab_total = (a.vocabulary.len().max(b.vocabulary.len())) as f32;
    let vocab_similarity = if vocab_total > 0.0 {
        shared_vocab as f32 / vocab_total
    } else {
        1.0 // both empty => same starting point
    };
    let vocab_difficulty = 1.0 - vocab_similarity;

    // --- Grammar rule similarity ---
    let shared_grammar: u32 = a
        .grammar_rules
        .iter()
        .filter(|rule_a| {
            b.grammar_rules
                .iter()
                .any(|rule_b| rule_a.pattern == rule_b.pattern && rule_a.replacement == rule_b.replacement)
        })
        .count() as u32;
    let grammar_total = a.grammar_rules.len().max(b.grammar_rules.len()) as f32;
    let grammar_similarity = if grammar_total > 0.0 {
        shared_grammar as f32 / grammar_total
    } else {
        1.0
    };
    let grammar_difficulty = 1.0 - grammar_similarity;

    // --- Phoneme Jaccard distance ---
    let set_a: BTreeSet<&str> = a.phonemes.iter().map(|s| s.as_str()).collect();
    let set_b: BTreeSet<&str> = b.phonemes.iter().map(|s| s.as_str()).collect();
    let intersection = set_a.intersection(&set_b).count() as f32;
    let union = set_a.union(&set_b).count() as f32;
    let phoneme_difficulty = if union > 0.0 {
        1.0 - (intersection / union)
    } else {
        0.0
    };

    // Weighted combination.
    let difficulty =
        vocab_difficulty * 0.50 + grammar_difficulty * 0.30 + phoneme_difficulty * 0.20;
    difficulty.clamp(0.0, 1.0)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod language_extended_tests {
    use super::*;

    // --- Original 12 tests ---

    #[test]
    fn create_language_basic() {
        let lang =
            create_language("Common", vec!["a".into(), "b".into(), "c".into()], 10);
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
        assert_eq!(result.vocabulary.get("water").expect("should have water"), "agua");
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

    // ===================================================================
    // Language Family Tree tests
    // ===================================================================

    #[test]
    fn family_tree_new_root() {
        let tree = LanguageFamilyTree::new("Proto-World");
        assert_eq!(tree.families.len(), 1);
        assert_eq!(tree.root_id, 0);
        assert_eq!(tree.families[0].name, "Proto-World");
        assert!(tree.families[0].parent_id.is_none());
    }

    #[test]
    fn family_tree_diverge() {
        let mut tree = LanguageFamilyTree::new("Proto-World");
        let child = tree.diverge(0, "Proto-Indo-European", 100);
        assert_eq!(child, 1);
        assert_eq!(tree.families.len(), 2);
        assert_eq!(tree.families[1].parent_id, Some(0));
        assert_eq!(tree.families[1].divergence_tick, 100);
        assert_eq!(tree.families[0].children, vec![1]);
    }

    #[test]
    fn family_tree_multiple_divergences() {
        let mut tree = LanguageFamilyTree::new("Proto-World");
        let a = tree.diverge(0, "Proto-A", 50);
        let b = tree.diverge(0, "Proto-B", 75);
        let a1 = tree.diverge(a, "Language-A1", 200);
        assert_eq!(a, 1);
        assert_eq!(b, 2);
        assert_eq!(a1, 3);
        assert_eq!(tree.families[0].children, vec![1, 2]);
        assert_eq!(tree.families[a].children, vec![3]);
    }

    #[test]
    fn family_tree_lineage() {
        let mut tree = LanguageFamilyTree::new("Proto-World");
        let a = tree.diverge(0, "Proto-A", 50);
        let a1 = tree.diverge(a, "A1", 200);
        let lineage = tree.get_lineage(a1);
        assert_eq!(lineage, vec![a1, a, 0]);
    }

    #[test]
    fn family_tree_are_related_direct() {
        let mut tree = LanguageFamilyTree::new("Proto-World");
        let a = tree.diverge(0, "Proto-A", 50);
        let b = tree.diverge(0, "Proto-B", 60);
        assert!(tree.are_related(a, b)); // share root
    }

    #[test]
    fn family_tree_are_related_self() {
        let tree = LanguageFamilyTree::new("Proto-World");
        assert!(tree.are_related(0, 0));
    }

    #[test]
    fn family_tree_are_related_distant() {
        let mut tree = LanguageFamilyTree::new("Root");
        let a = tree.diverge(0, "A", 10);
        let a1 = tree.diverge(a, "A1", 20);
        let a2 = tree.diverge(a1, "A2", 30);
        assert!(tree.are_related(a2, 0));
        assert!(tree.are_related(a2, a));
        assert!(tree.are_related(a2, a1));
    }

    // ===================================================================
    // Vocabulary Drift tests
    // ===================================================================

    #[test]
    fn drift_vocabulary_deterministic() {
        let lang = add_vocabulary(
            &create_language("DriftTest", vec![], 0),
            "fire",
            "ignis",
        );
        let drift = VocabularyDrift {
            drift_rate: 1.0,
            mutation_probability: 1.0,
            loss_probability: 0.0,
        };
        let a = drift_vocabulary(&lang, &drift, 42);
        let b = drift_vocabulary(&lang, &drift, 42);
        assert_eq!(a.vocabulary, b.vocabulary);
    }

    #[test]
    fn drift_vocabulary_high_loss_empties() {
        let mut lang = create_language("LoseAll", vec![], 0);
        lang.vocabulary.insert("a".into(), "x".into());
        lang.vocabulary.insert("b".into(), "y".into());
        let drift = VocabularyDrift {
            drift_rate: 1.0,
            mutation_probability: 0.0,
            loss_probability: 1.0,
        };
        let result = drift_vocabulary(&lang, &drift, 1);
        assert!(result.vocabulary.is_empty());
    }

    #[test]
    fn drift_vocabulary_zero_rate_preserves() {
        let lang = add_vocabulary(&create_language("NoDrift", vec![], 0), "fire", "ignis");
        let drift = VocabularyDrift {
            drift_rate: 0.0,
            mutation_probability: 0.5,
            loss_probability: 0.5,
        };
        let result = drift_vocabulary(&lang, &drift, 1);
        assert_eq!(result.vocabulary.len(), 1);
        assert_eq!(
            result.vocabulary.get("fire").expect("word exists"),
            "ignis"
        );
    }

    #[test]
    fn word_distance_identical() {
        assert!((word_distance("hello", "hello") - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn word_distance_empty() {
        assert!((word_distance("", "abc") - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn word_distance_symmetric() {
        let d1 = word_distance("kitten", "sitting");
        let d2 = word_distance("sitting", "kitten");
        assert!((d1 - d2).abs() < f32::EPSILON);
    }

    #[test]
    fn word_distance_substitution() {
        let d = word_distance("cat", "car");
        // Levenshtein distance = 1, max_len = 3, so 1/3
        assert!((d - 1.0 / 3.0).abs() < 1e-5);
    }

    // ===================================================================
    // Lingua Franca tests
    // ===================================================================

    #[test]
    fn adoption_rate_basic() {
        let lingua = LinguaFranca {
            language_id: 0,
            dominance: 1.0,
            adoption_rate: 0.5,
            resistance: 0.0,
        };
        let rate = compute_adoption_rate(&lingua, 1000, 1.0);
        assert!(rate > 0.0 && rate <= 1.0);
    }

    #[test]
    fn adoption_rate_high_resistance() {
        let lingua = LinguaFranca {
            language_id: 0,
            dominance: 1.0,
            adoption_rate: 1.0,
            resistance: 0.99,
        };
        let rate = compute_adoption_rate(&lingua, 1000, 1.0);
        assert!(rate < 0.05);
    }

    #[test]
    fn adoption_rate_zero_dominance() {
        let lingua = LinguaFranca {
            language_id: 0,
            dominance: 0.0,
            adoption_rate: 1.0,
            resistance: 0.0,
        };
        let rate = compute_adoption_rate(&lingua, 1000, 1.0);
        assert!((rate - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn adoption_rate_large_population_dampens() {
        let lingua = LinguaFranca {
            language_id: 0,
            dominance: 1.0,
            adoption_rate: 0.8,
            resistance: 0.0,
        };
        let small = compute_adoption_rate(&lingua, 100, 1.0);
        let large = compute_adoption_rate(&lingua, 10_000_000, 1.0);
        assert!(small > large);
    }

    #[test]
    fn apply_lingua_franca_strong_adoption() {
        let target = create_language("Target", vec![], 0);
        let mut lingua_lang = create_language("Lingua", vec![], 0);
        lingua_lang.vocabulary.insert("water".into(), "aqua".into());
        lingua_lang.vocabulary.insert("fire".into(), "fuoco".into());

        let result = apply_lingua_franca(&target, &lingua_lang, 1.0);
        // With strength 1.0 all shared meanings should be replaced.
        assert!(result.vocabulary.len() >= 2);
        // At minimum "water" and "fire" are present.
        assert!(result.vocabulary.contains_key("water") || result.vocabulary.contains_key("fire"));
    }

    #[test]
    fn apply_lingua_franca_zero_adoption() {
        let target = add_vocabulary(&create_language("Target", vec![], 0), "water", "wasser");
        let mut lingua_lang = create_language("Lingua", vec![], 0);
        lingua_lang.vocabulary.insert("water".into(), "aqua".into());
        lingua_lang.vocabulary.insert("fire".into(), "fuoco".into());

        let result = apply_lingua_franca(&target, &lingua_lang, 0.0);
        // With strength 0.0, nothing should be adopted.
        assert_eq!(
            result.vocabulary.get("water").expect("original preserved"),
            "wasser"
        );
    }

    // ===================================================================
    // Writing System Evolution tests
    // ===================================================================

    #[test]
    fn script_evolution_progression() {
        assert_eq!(
            evolve_script(ScriptEvolution::Pictographic, 0.0),
            ScriptEvolution::Pictographic
        );
        assert_eq!(
            evolve_script(ScriptEvolution::Pictographic, 0.25),
            ScriptEvolution::Ideographic
        );
        assert_eq!(
            evolve_script(ScriptEvolution::Ideographic, 0.50),
            ScriptEvolution::Syllabic
        );
        assert_eq!(
            evolve_script(ScriptEvolution::Syllabic, 0.75),
            ScriptEvolution::Alphabetic
        );
    }

    #[test]
    fn script_evolution_stays_at_max() {
        assert_eq!(
            evolve_script(ScriptEvolution::Alphabetic, 1.0),
            ScriptEvolution::Alphabetic
        );
    }

    #[test]
    fn script_complexity_decreases() {
        let p = script_complexity(ScriptEvolution::Pictographic);
        let i = script_complexity(ScriptEvolution::Ideographic);
        let s = script_complexity(ScriptEvolution::Syllabic);
        let a = script_complexity(ScriptEvolution::Alphabetic);
        // Pictographic needs most symbols, alphabetic fewest.
        assert!(p > i);
        assert!(i > s);
        assert!(s > a);
    }

    // ===================================================================
    // Translation Difficulty tests
    // ===================================================================

    #[test]
    fn translation_difficulty_identical() {
        let lang = create_language("Same", vec![], 0);
        assert!((translation_difficulty(&lang, &lang) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn translation_difficulty_identical_name() {
        let a = create_language("Shared", vec![], 0);
        let b = create_language("Shared", vec![], 0);
        assert!((translation_difficulty(&a, &b) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn translation_difficulty_identical_vocabulary() {
        let a = add_vocabulary(&add_vocabulary(&create_language("A", vec![], 0), "x", "1"), "y", "2");
        let b = add_vocabulary(&add_vocabulary(&create_language("B", vec![], 0), "x", "1"), "y", "2");
        let d = translation_difficulty(&a, &b);
        // High vocabulary overlap => low difficulty.
        assert!(d < 0.5);
    }

    #[test]
    fn translation_difficulty_no_overlap() {
        let a = add_vocabulary(&create_language("A", vec!["p".into()], 0), "fire", "ignis");
        let b = add_vocabulary(&create_language("B", vec!["q".into()], 0), "water", "aqua");
        let d = translation_difficulty(&a, &b);
        // No overlap, different phonemes => high difficulty.
        assert!(d > 0.5);
    }

    #[test]
    fn translation_difficulty_empty_languages() {
        let a = create_language("A", vec![], 0);
        let b = create_language("B", vec![], 0);
        let d = translation_difficulty(&a, &b);
        // Both empty => shared starting point but different names.
        // Vocabulary: both empty => similarity 1.0 => vocab_difficulty 0.
        // Grammar: both empty => similarity 1.0 => grammar_difficulty 0.
        // Phonemes: both empty => union 0 => difficulty 0.
        assert!((d - 0.0).abs() < f32::EPSILON);
    }

    // ===================================================================
    // Edge-case and integration tests
    // ===================================================================

    #[test]
    fn loan_word_multiple_transfers() {
        let src = add_vocabulary(
            &add_vocabulary(&create_language("Src", vec![], 0), "water", "agua"),
            "fire",
            "fuego",
        );
        let tgt = create_language("Tgt", vec![], 0);
        let tgt = loan_word(&tgt, &src, "water");
        let tgt = loan_word(&tgt, &src, "fire");
        assert_eq!(tgt.vocabulary.len(), 2);
        assert_eq!(tgt.vocabulary.get("water").expect("water"), "agua");
        assert_eq!(tgt.vocabulary.get("fire").expect("fire"), "fuego");
    }

    #[test]
    fn family_tree_deep_lineage() {
        let mut tree = LanguageFamilyTree::new("Root");
        let mut prev = 0;
        for i in 0..10 {
            prev = tree.diverge(prev, &format!("Gen-{i}"), i as u64 * 10);
        }
        let lineage = tree.get_lineage(prev);
        assert_eq!(lineage.len(), 11); // 10 children + root
        assert_eq!(*lineage.last().expect("root in lineage"), 0);
    }

    #[test]
    fn word_distance_both_empty() {
        assert!((word_distance("", "") - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn script_evolution_insufficient_complexity() {
        assert_eq!(
            evolve_script(ScriptEvolution::Pictographic, 0.24),
            ScriptEvolution::Pictographic
        );
        assert_eq!(
            evolve_script(ScriptEvolution::Ideographic, 0.49),
            ScriptEvolution::Ideographic
        );
    }

    #[test]
    fn seeded_language_state_has_signature() {
        let sig = [0.1, 0.2, 0.3, 0.4];
        let state = seeded_language_state(sig);
        assert_eq!(state.seed_signature, sig);
    }

    #[test]
    fn ensure_seeded_word_pushes_lexeme() {
        let mut state = seeded_language_state([0.0; 4]);
        ensure_seeded_word(&mut state, WordKind::Place, [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(state.lexemes.len(), 1);
        assert!(state.lexemes[0].starts_with("place:"));
    }

    #[test]
    fn borrow_word_pushes_lexeme() {
        let mut target = seeded_language_state([0.0; 4]);
        let source = seeded_language_state([0.0; 4]);
        borrow_word(&mut target, &source, WordKind::Person);
        assert_eq!(target.lexemes.len(), 1);
        assert_eq!(target.lexemes[0], "borrow:person");
    }
}
