//! Tests for FR-CIV-LANG-010 — Language and naming (advanced language simulation)
//!
//! Epic: FR-CIV-LANG
//! Verifies the extended language simulation: family trees, vocabulary drift,
//! lingua franca mechanics, script evolution, and translation difficulty.

#[cfg(test)]
mod fr_fr_civ_lang_010 {
    use civ_engine::language::*;

    /// FR-CIV-LANG-010: Language creation sets expected defaults.
    #[test]
    fn create_language_sets_defaults() {
        let lang = create_language("Proto-World", vec!["a".into(), "b".into()], 5);
        assert_eq!(lang.name, "Proto-World");
        assert_eq!(lang.phonemes.len(), 2);
        assert_eq!(lang.drift_factor, 0.01);
        assert_eq!(lang.created_tick, 5);
    }

    /// FR-CIV-LANG-010: Language evolution is deterministic.
    #[test]
    fn evolve_language_is_deterministic() {
        let lang = create_language("Test", vec![], 10);
        let a = evolve_language(&lang, 100);
        let b = evolve_language(&lang, 100);
        assert_eq!(a.intelligibility_baseline, b.intelligibility_baseline);
        assert_eq!(a.drift_factor, b.drift_factor);
    }

    /// FR-CIV-LANG-010: Loan words transfer vocabulary between languages.
    #[test]
    fn loan_word_transfers_vocabulary() {
        let src = add_vocabulary(&create_language("Src", vec![], 0), "water", "agua");
        let tgt = create_language("Tgt", vec![], 0);
        let result = loan_word(&tgt, &src, "water");
        assert_eq!(result.vocabulary.get("water").unwrap(), "agua");
    }

    /// FR-CIV-LANG-010: Mutual intelligibility returns 1.0 for same language.
    #[test]
    fn intelligibility_same_language() {
        let lang = create_language("Shared", vec![], 0);
        let score = compute_mutual_intelligibility(&lang, &lang);
        assert!((score - 1.0).abs() < f32::EPSILON);
    }

    /// FR-CIV-LANG-010: Language family tree tracks divergences.
    #[test]
    fn family_tree_divergence_tracking() {
        let mut tree = LanguageFamilyTree::new("Proto-World");
        let child = tree.diverge(0, "Proto-A", 100);
        assert_eq!(tree.families[child].parent_id, Some(0));
        assert_eq!(tree.families[child].divergence_tick, 100);
        assert!(tree.are_related(child, 0));
    }

    /// FR-CIV-LANG-010: Vocabulary drift is deterministic.
    #[test]
    fn vocabulary_drift_is_deterministic() {
        let lang = add_vocabulary(&create_language("Drift", vec![], 0), "fire", "ignis");
        let drift = VocabularyDrift {
            drift_rate: 1.0,
            mutation_probability: 0.5,
            loss_probability: 0.0,
        };
        let a = drift_vocabulary(&lang, &drift, 42);
        let b = drift_vocabulary(&lang, &drift, 42);
        assert_eq!(a.vocabulary, b.vocabulary);
    }

    /// FR-CIV-LANG-010: Script evolution follows thresholds.
    #[test]
    fn script_evolution_respects_thresholds() {
        assert_eq!(
            evolve_script(ScriptEvolution::Pictographic, 0.25),
            ScriptEvolution::Ideographic
        );
        assert_eq!(
            evolve_script(ScriptEvolution::Pictographic, 0.24),
            ScriptEvolution::Pictographic
        );
        assert_eq!(
            evolve_script(ScriptEvolution::Syllabic, 0.75),
            ScriptEvolution::Alphabetic
        );
    }

    /// FR-CIV-LANG-010: Translation difficulty is 0 for identical languages.
    #[test]
    fn translation_difficulty_identical_is_zero() {
        let lang = create_language("Same", vec![], 0);
        assert!((translation_difficulty(&lang, &lang) - 0.0).abs() < f32::EPSILON);
    }

    /// FR-CIV-LANG-010: Lingua franca adoption rate respects dominance.
    #[test]
    fn lingua_franca_zero_dominance_means_zero_adoption() {
        let lingua = LinguaFranca {
            language_id: 0,
            dominance: 0.0,
            adoption_rate: 1.0,
            resistance: 0.0,
        };
        let rate = compute_adoption_rate(&lingua, 1000, 1.0);
        assert!((rate - 0.0).abs() < f32::EPSILON);
    }
}
