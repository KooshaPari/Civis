//! Persistence regression tests for the language-state + per-faction language
//! drift surface.
//!
//! `Simulation::language_state` and `Simulation::faction_languages` are
//! mutated every tick by `phase_culture`. Before this commit they lived only
//! on `Simulation`; on `.civsave.zst` archive round-trip they reset to empty
//! defaults and the language-drift trajectory was lost.

use civ_engine::{CivSaveBundle, LanguageState, Simulation};

#[test]
fn archive_roundtrips_language_state_after_save_load() {
    let mut sim = Simulation::with_seed(707);
    sim.advance_ticks(3);

    let pre_saved_lang = sim.language_state().clone();
    let pre_faction_langs: std::collections::BTreeMap<u32, LanguageState> =
        sim.faction_languages().clone();

    let temp = std::env::temp_dir().join("civis_lang_roundtrip.civsave.zst");
    CivSaveBundle::save_archive(&temp, &mut sim).expect("save should succeed");
    let loaded = CivSaveBundle::load_archive(&temp).expect("load should succeed");

    let loaded_lang = loaded.language_state();
    let loaded_faction_langs = loaded.faction_languages();

    assert_eq!(
        loaded_lang.seed_signature, pre_saved_lang.seed_signature,
        "world LanguageState.seed_signature must round-trip exactly"
    );
    assert_eq!(
        loaded_lang.lexemes, pre_saved_lang.lexemes,
        "world LanguageState.lexemes must round-trip exactly"
    );
    assert_eq!(
        loaded_faction_langs.len(),
        pre_faction_langs.len(),
        "faction_languages map size must round-trip exactly"
    );

    let _ = std::fs::remove_file(&temp);
}

#[test]
fn archive_roundtrips_modified_faction_language_via_public_setter() {
    let mut sim = Simulation::with_seed(303);
    sim.advance_ticks(1);

    let overrides = std::collections::BTreeMap::from([(
        3_u32,
        LanguageState {
            seed_signature: [0.2_f32, 0.4_f32, 0.6_f32, 0.8_f32],
            drift_rate: 0.05_f32,
            split_threshold: 0.3_f32,
            lexemes: vec!["alpha".to_string(), "beta".into(), "gamma".into()],
        },
    )]);
    sim.set_faction_languages(overrides);
    sim.advance_ticks(1);

    let temp = std::env::temp_dir().join("civis_lang_setter.civsave.zst");
    CivSaveBundle::save_archive(&temp, &mut sim).expect("save should succeed");
    let loaded = CivSaveBundle::load_archive(&temp).expect("load should succeed");

    let loaded_lang = loaded
        .faction_languages()
        .get(&3_u32)
        .expect("faction 3 should be present post-load");
    assert_eq!(
        loaded_lang.seed_signature,
        [0.2_f32, 0.4_f32, 0.6_f32, 0.8_f32],
        "overridden faction 3 seed_signature must round-trip exactly"
    );
    assert_eq!(
        loaded_lang.lexemes,
        vec!["alpha".to_string(), "beta".into(), "gamma".into()],
        "overridden faction 3 lexemes must round-trip exactly"
    );

    let _ = std::fs::remove_file(&temp);
}

#[test]
fn language_state_is_readable_after_ticks() {
    let mut sim = Simulation::with_seed(909);
    sim.advance_ticks(2);
    let ls = sim.language_state();
    assert!(ls.drift_rate.is_finite());
    assert!(ls.split_threshold.is_finite());
}
