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
