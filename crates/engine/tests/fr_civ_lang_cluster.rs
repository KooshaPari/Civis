//! FR-CIV-LANG-001 / 003 / 005 / 009 — emergent language system.
//!
//! Requirement text (`FUNCTIONAL_REQUIREMENTS.md:472-480`):
//!
//! - FR-CIV-LANG-001: "All player-visible strings SHALL store `semantic_key` +
//!   per-civ lexeme IDs, never locale-baked bytes in sim state."
//! - FR-CIV-LANG-003: "Phonology engine SHALL apply diachronic sound-change
//!   rules each generation tick."
//! - FR-CIV-LANG-005: "Civ split/contact SHALL branch lexicons (clone+diverge)
//!   and borrow lexemes through receiver phonotactic filters."
//! - FR-CIV-LANG-009: "Background name generation SHALL remain DF-tier cheap;
//!   no LLM calls on the language engine hot path."
//!
//! These replace the auto-generated placeholder tests
//! (`crates/engine/tests/fr_fr_civ_lang_001.rs` and siblings), whose entire body
//! was `let ws = civ_engine::WorldState::default(); assert!(ws.tick == 0);` —
//! it never touched the language system, and the five files differed only in
//! their names.
//!
//! ## Not covered here
//!
//! **FR-CIV-LANG-002** ("Render path SHALL toggle English (system fonts) vs
//! native civ script atlas per civ/view setting") has **no implementation to
//! test**. A workspace-wide search for the fonts/atlas toggle finds only an
//! unrelated PBR material comment in `clients/bevy-ref/src/pbr_materials.rs`.
//! Writing an oracle would mean inventing the behaviour, so this file does not
//! claim it. It should be treated as unimplemented, not as covered.

use civ_engine::language::{
    add_vocabulary, compute_mutual_intelligibility, create_language, ensure_seeded_word,
    evolve_language, loan_word, person_name, place_name, seeded_language_state,
    tick_language_for_lineage, tick_language_system, LanguageFamilyTree, WordKind,
};
use civ_engine::LanguageState;

// ---------------------------------------------------------------------------
// FR-CIV-LANG-001 — semantic keys in sim state, not rendered strings
// ---------------------------------------------------------------------------

/// Covers FR-CIV-LANG-001.
///
/// The sim state must key words by a *semantic* key, and the rendered,
/// player-visible string must be derived at render time rather than baked into
/// state. Concretely: seeding a word records a semantic-key-tagged lexeme, and
/// the string `place_name` would render never appears in the state.
#[test]
fn fr_civ_lang_001_state_stores_semantic_keys_not_rendered_strings() {
    let mut state = seeded_language_state([0.25, 0.5, 0.75, 1.0]);
    assert!(
        state.lexemes.is_empty(),
        "a freshly seeded language has no lexemes yet"
    );

    ensure_seeded_word(&mut state, WordKind::Place, [1.0, 0.0, 0.0, 0.0]);

    assert_eq!(state.lexemes.len(), 1, "one seeded word recorded");
    let lexeme = &state.lexemes[0];
    assert!(
        lexeme.starts_with("place:"),
        "the lexeme must carry its semantic key as a prefix, got {lexeme:?}"
    );

    // The rendered string for the same concept must NOT be present in state.
    let rendered = place_name(&state, 3, 7);
    assert!(
        !state.lexemes.contains(&rendered),
        "the locale-rendered string {rendered:?} must not be baked into sim state; \
         lexemes are {lexemes:?}",
        lexemes = state.lexemes
    );

    // And the two kinds must stay distinguishable by key.
    ensure_seeded_word(&mut state, WordKind::Person, [0.0, 1.0, 0.0, 0.0]);
    assert_eq!(state.lexemes.len(), 2);
    assert!(
        state.lexemes.iter().any(|l| l.starts_with("person:")),
        "a person word must be keyed as a person, got {:?}",
        state.lexemes
    );
}

/// Covers FR-CIV-LANG-001.
///
/// Determinism: the same semantic inputs must produce the same lexeme, or the
/// simulation would diverge across replays.
#[test]
fn fr_civ_lang_001_seeding_is_deterministic() {
    let meaning = [0.1, 0.2, 0.3, 0.4];
    let mut a = seeded_language_state([1.0, 1.0, 1.0, 1.0]);
    let mut b = seeded_language_state([1.0, 1.0, 1.0, 1.0]);
    ensure_seeded_word(&mut a, WordKind::Place, meaning);
    ensure_seeded_word(&mut b, WordKind::Place, meaning);
    assert_eq!(a.lexemes, b.lexemes, "seeding must be deterministic");
}

// ---------------------------------------------------------------------------
// FR-CIV-LANG-003 — diachronic drift each generation tick
// ---------------------------------------------------------------------------

/// Covers FR-CIV-LANG-003.
///
/// Drift is driven by isolation pressure: no isolation must contribute nothing,
/// and full isolation must advance the drift rate.
#[test]
fn fr_civ_lang_003_drift_tracks_isolation_pressure() {
    let mut calm = seeded_language_state([0.0; 4]);
    let before = calm.drift_rate;
    tick_language_for_lineage(&mut calm, 0.0, 1);
    assert_eq!(
        calm.drift_rate, before,
        "zero isolation must not advance the drift rate"
    );

    let mut isolated = seeded_language_state([0.0; 4]);
    let before = isolated.drift_rate;
    tick_language_for_lineage(&mut isolated, 1.0, 1);
    assert!(
        isolated.drift_rate > before,
        "full isolation must advance the drift rate (was {before}, now {})",
        isolated.drift_rate
    );
}

/// Covers FR-CIV-LANG-003.
///
/// The drift rate must stay in range no matter how long a lineage stays
/// isolated, or downstream comparisons stop being meaningful.
#[test]
fn fr_civ_lang_003_drift_rate_is_bounded_under_sustained_isolation() {
    let mut state = seeded_language_state([0.0; 4]);
    for tick in 0..500 {
        tick_language_for_lineage(&mut state, 1.0, 7);
        assert!(
            (0.0..=1.0).contains(&state.drift_rate),
            "drift_rate left [0,1] at tick {tick}: {}",
            state.drift_rate
        );
    }
    assert_eq!(
        state.drift_rate, 1.0,
        "sustained full isolation must saturate at the documented ceiling"
    );
}

/// Covers FR-CIV-LANG-003.
///
/// Independent lineages with identical inputs must drift identically, and a
/// language system tick must be a pure function of (language, tick).
#[test]
fn fr_civ_lang_003_tick_is_deterministic() {
    let lang = create_language("Proto", vec!["k".into(), "a".into()], 0);

    let a = tick_language_system(&lang, 42);
    let b = tick_language_system(&lang, 42);
    assert_eq!(
        a.intelligibility_baseline, b.intelligibility_baseline,
        "same (language, tick) must yield the same intelligibility"
    );
    assert_eq!(
        a.drift_factor, b.drift_factor,
        "same (language, tick) must yield the same drift factor"
    );

    // A different tick must be able to differ, otherwise the tick is ignored.
    let c = evolve_language(&lang, 43);
    let differs = c.intelligibility_baseline != a.intelligibility_baseline
        || c.drift_factor != a.drift_factor;
    assert!(
        differs,
        "tick 43 produced an identical result to tick 42; the tick argument is ignored"
    );
}

/// Covers FR-CIV-LANG-003.
///
/// Intelligibility decays under drift but must never fall through its floor.
#[test]
fn fr_civ_lang_003_intelligibility_decays_without_leaving_floor() {
    let mut lang = create_language("Proto", vec!["k".into()], 0);
    let start = lang.intelligibility_baseline;
    assert!(start > 0.1, "premise: starts above the floor");

    for tick in 1..=200 {
        lang = tick_language_system(&lang, tick);
        assert!(
            lang.intelligibility_baseline >= 0.1,
            "intelligibility fell below its floor at tick {tick}: {}",
            lang.intelligibility_baseline
        );
        assert!(
            lang.intelligibility_baseline <= 1.0,
            "intelligibility exceeded 1.0 at tick {tick}: {}",
            lang.intelligibility_baseline
        );
    }
    assert!(
        lang.intelligibility_baseline < start,
        "drift over 200 ticks must reduce intelligibility, not leave it at {start}"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-LANG-005 — branching lexicons and borrowing
// ---------------------------------------------------------------------------

/// Covers FR-CIV-LANG-005.
///
/// A civ split must clone the parent into a child that records its parent and
/// remains related to it.
#[test]
fn fr_civ_lang_005_split_creates_a_related_child() {
    let mut tree = LanguageFamilyTree::new("Proto");
    let root = tree.root_id;
    assert!(
        tree.families[root].parent_id.is_none(),
        "the root proto-language has no parent"
    );

    let child = tree.diverge(root, "Daughter", 120);
    assert_ne!(child, root, "diverge must add a new node");
    assert_eq!(
        tree.families[child].parent_id,
        Some(root),
        "the child must record its parent"
    );
    assert_eq!(
        tree.families[child].divergence_tick, 120,
        "the child must record when it diverged"
    );
    assert!(
        tree.families[root].children.contains(&child),
        "the parent must list the child"
    );
    assert!(
        tree.are_related(root, child),
        "a split child must remain related to its parent"
    );

    let lineage = tree.get_lineage(child);
    assert!(
        lineage.contains(&root),
        "the child's lineage must include the root, got {lineage:?}"
    );
}

/// Covers FR-CIV-LANG-005.
///
/// Borrowing must copy a lexeme only when the source actually has it; a
/// missing source entry must not fabricate one.
#[test]
fn fr_civ_lang_005_borrow_copies_only_existing_lexemes() {
    let source = add_vocabulary(&create_language("Source", vec!["k".into()], 0), "water", "akwa");
    let target = create_language("Target", vec!["t".into()], 0);
    assert!(
        target.vocabulary.get("water").is_none(),
        "premise: target does not know the word"
    );

    let borrowed = loan_word(&target, &source, "water");
    assert_eq!(
        borrowed.vocabulary.get("water").map(String::as_str),
        Some("akwa"),
        "borrowing must copy the source's lexeme for that meaning"
    );

    // A meaning the source lacks must not be invented in the receiver.
    let absent = loan_word(&target, &source, "fire");
    assert!(
        absent.vocabulary.get("fire").is_none(),
        "borrowing a meaning the source lacks must not fabricate a word"
    );

    // The source must be unaffected by a loan.
    assert_eq!(
        source.vocabulary.get("water").map(String::as_str),
        Some("akwa")
    );
}

/// Covers FR-CIV-LANG-005.
///
/// Shared vocabulary is what makes languages mutually intelligible; the metric
/// must reflect it and must be symmetric.
#[test]
fn fr_civ_lang_005_shared_lexemes_drive_mutual_intelligibility() {
    let a = add_vocabulary(&create_language("A", vec!["k".into()], 0), "water", "akwa");
    let b = add_vocabulary(&create_language("B", vec!["t".into()], 0), "water", "akwa");

    let ab = compute_mutual_intelligibility(&a, &b);
    assert!(
        ab > 0.0,
        "languages sharing a lexeme must have non-zero intelligibility, got {ab}"
    );
    assert_eq!(
        ab,
        compute_mutual_intelligibility(&b, &a),
        "intelligibility must be symmetric"
    );

    let unrelated = create_language("C", vec!["z".into()], 0);
    let ac = compute_mutual_intelligibility(&a, &unrelated);
    assert!(
        ab >= ac,
        "sharing a lexeme must not be less intelligible than sharing none"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-LANG-009 — name generation stays cheap and offline
// ---------------------------------------------------------------------------

/// Covers FR-CIV-LANG-009.
///
/// "No LLM calls on the hot path" is only observable indirectly, but it implies
/// the generator must be a pure function of its inputs: a network-backed
/// generator could not return bit-identical output on every call, nor ignore
/// the surrounding world state. Asserting purity is the falsifiable claim.
#[test]
fn fr_civ_lang_009_name_generation_is_pure_and_repeatable() {
    let state_a = seeded_language_state([0.1, 0.2, 0.3, 0.4]);
    let state_b = seeded_language_state([9.0, 8.0, 7.0, 6.0]);

    // Repeatable: same call, same result.
    assert_eq!(
        place_name(&state_a, 4, 11),
        place_name(&state_a, 4, 11),
        "repeated place-name generation must be identical"
    );
    assert_eq!(
        person_name(&state_a, 4, 11),
        person_name(&state_a, 4, 11),
        "repeated person-name generation must be identical"
    );

    // Independent of unrelated world state: an offline generator cannot be
    // influenced by a different language seed for the same name request.
    assert_eq!(
        place_name(&state_a, 4, 11),
        place_name(&state_b, 4, 11),
        "name generation must not depend on unrelated state; a network or LLM \
         call would not be so stable"
    );

    // Distinct inputs must still be distinguishable, or the function is a
    // constant and the assertion above is vacuous.
    assert_ne!(
        place_name(&state_a, 4, 11),
        place_name(&state_a, 4, 12),
        "different places must not collapse to the same name"
    );
    assert_ne!(
        person_name(&state_a, 4, 11),
        place_name(&state_a, 4, 11),
        "a person and a place must not share a rendered name"
    );
}
