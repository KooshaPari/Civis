//! Tests for FR-CIV-LANG-006 — Language and naming (word borrowing)
//!
//! Epic: FR-CIV-LANG
//! Verifies that words can be borrowed between language states.

#[cfg(test)]
mod fr_fr_civ_lang_006 {
    /// FR-CIV-LANG-006: borrow_word adds a lexeme to the target language.
    #[test]
    fn borrow_word_adds_lexeme_to_target() {
        let mut target = civ_engine::language::seeded_language_state([0.0; 4]);
        let source = civ_engine::language::seeded_language_state([1.0; 4]);
        civ_engine::language::borrow_word(
            &mut target,
            &source,
            civ_engine::language::WordKind::Place,
        );
        assert_eq!(target.lexemes.len(), 1);
        assert_eq!(target.lexemes[0], "borrow:place");
    }

    /// FR-CIV-LANG-006: borrow_word for Person kind.
    #[test]
    fn borrow_word_person_kind() {
        let mut target = civ_engine::language::seeded_language_state([0.0; 4]);
        let source = civ_engine::language::seeded_language_state([1.0; 4]);
        civ_engine::language::borrow_word(
            &mut target,
            &source,
            civ_engine::language::WordKind::Person,
        );
        assert_eq!(target.lexemes[0], "borrow:person");
    }

    /// FR-CIV-LANG-006: Multiple borrows accumulate lexemes.
    #[test]
    fn multiple_borrows_accumulate() {
        let mut target = civ_engine::language::seeded_language_state([0.0; 4]);
        let source = civ_engine::language::seeded_language_state([1.0; 4]);
        civ_engine::language::borrow_word(
            &mut target,
            &source,
            civ_engine::language::WordKind::Place,
        );
        civ_engine::language::borrow_word(
            &mut target,
            &source,
            civ_engine::language::WordKind::Person,
        );
        assert_eq!(target.lexemes.len(), 2);
    }
}
