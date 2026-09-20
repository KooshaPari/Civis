//! Tests for FR-CIV-LANG-004 — Language and naming (word seeding)
//!
//! Epic: FR-CIV-LANG
//! Verifies that seeded language state creation and word seeding work correctly.

#[cfg(test)]
mod fr_fr_civ_lang_004 {
    /// FR-CIV-LANG-004: seeded_language_state stores the signature.
    #[test]
    fn seeded_language_state_preserves_signature() {
        let sig = [0.1, 0.2, 0.3, 0.4];
        let state = civ_engine::language::seeded_language_state(sig);
        assert_eq!(state.seed_signature, sig);
    }

    /// FR-CIV-LANG-004: ensure_seeded_word adds a lexeme for a place.
    #[test]
    fn ensure_seeded_word_adds_place_lexeme() {
        let mut state = civ_engine::language::seeded_language_state([0.0; 4]);
        civ_engine::language::ensure_seeded_word(
            &mut state,
            civ_engine::language::WordKind::Place,
            [1.0, 2.0, 3.0, 4.0],
        );
        assert_eq!(state.lexemes.len(), 1);
        assert!(state.lexemes[0].starts_with("place:"));
    }

    /// FR-CIV-LANG-004: ensure_seeded_word adds a lexeme for a person.
    #[test]
    fn ensure_seeded_word_adds_person_lexeme() {
        let mut state = civ_engine::language::seeded_language_state([0.0; 4]);
        civ_engine::language::ensure_seeded_word(
            &mut state,
            civ_engine::language::WordKind::Person,
            [1.0, 0.0, 0.0, 0.0],
        );
        assert_eq!(state.lexemes.len(), 1);
        assert!(state.lexemes[0].starts_with("person:"));
    }

    /// FR-CIV-LANG-004: place_name_meaning returns WordKind::Place.
    #[test]
    fn place_name_meaning_returns_place() {
        let wk = civ_engine::language::place_name_meaning(1, 2);
        assert_eq!(wk, civ_engine::language::WordKind::Place);
    }

    /// FR-CIV-LANG-004: person_name_meaning returns WordKind::Person.
    #[test]
    fn person_name_meaning_returns_person() {
        let wk = civ_engine::language::person_name_meaning(1, 2);
        assert_eq!(wk, civ_engine::language::WordKind::Person);
    }
}
