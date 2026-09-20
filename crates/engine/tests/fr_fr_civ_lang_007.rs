//! Tests for FR-CIV-LANG-007 — Language and naming (drift / tick)
//!
//! Epic: FR-CIV-LANG
//! Verifies that language drift advances deterministically under isolation pressure.

#[cfg(test)]
mod fr_fr_civ_lang_007 {
    /// FR-CIV-LANG-007: tick_language_for_lineage increases drift rate.
    #[test]
    fn tick_language_increases_drift_rate() {
        let mut state = civ_engine::language::seeded_language_state([0.0; 4]);
        assert_eq!(state.drift_rate, 0.0);
        civ_engine::language::tick_language_for_lineage(&mut state, 0.5, 1);
        assert!(state.drift_rate > 0.0);
    }

    /// FR-CIV-LANG-007: drift rate is clamped to [0, 1].
    #[test]
    fn drift_rate_clamped_to_one() {
        let mut state = civ_engine::language::seeded_language_state([0.0; 4]);
        // Apply high isolation many times
        for _ in 0..200 {
            civ_engine::language::tick_language_for_lineage(&mut state, 1.0, 1);
        }
        assert!(state.drift_rate <= 1.0);
    }

    /// FR-CIV-LANG-007: zero isolation does not change drift.
    #[test]
    fn zero_isolation_no_change() {
        let mut state = civ_engine::language::seeded_language_state([0.0; 4]);
        civ_engine::language::tick_language_for_lineage(&mut state, 0.0, 1);
        assert_eq!(state.drift_rate, 0.0);
    }

    /// FR-CIV-LANG-007: place_name generates deterministic names.
    #[test]
    fn place_name_is_deterministic() {
        let state = civ_engine::language::seeded_language_state([0.5; 4]);
        let n1 = civ_engine::language::place_name(&state, 1, 10);
        let n2 = civ_engine::language::place_name(&state, 1, 10);
        assert_eq!(n1, n2);
        assert_eq!(n1, "place-1-10");
    }

    /// FR-CIV-LANG-007: person_name generates deterministic names.
    #[test]
    fn person_name_is_deterministic() {
        let state = civ_engine::language::seeded_language_state([0.5; 4]);
        let n1 = civ_engine::language::person_name(&state, 2, 5);
        let n2 = civ_engine::language::person_name(&state, 2, 5);
        assert_eq!(n1, n2);
        assert_eq!(n1, "person-2-5");
    }
}
