//! Compat-state free functions and cohesion/unrest wrappers extracted from
//! engine.rs (Pass 10 — Civis Engine Decomposition).
//!
//! These free-function wrappers exist so `lib.rs` can re-export them as
//! `civ_engine::add_cohesion`, `civ_engine::faction_count`, etc. without
//! requiring callers to go through the `Simulation` struct.

use crate::social_types::{
    CohesionEvent, CohesionEventKind, CohesionSnapshot, FabricTier, UnrestEvent, UnrestLevel,
};
use std::collections::BTreeMap;

/// Add cohesion to a faction (currently a no-op stub).
pub fn add_cohesion(faction: u32, delta: f32) {
    let mut state = compat_state().lock().expect("compat state poisoned");
    state.faction_count = state.faction_count.max(faction.saturating_add(1));
    state.cohesion_events.push(CohesionEvent {
        actor_id: u64::from(faction),
        settlement_id: faction,
        kind: CohesionEventKind::Bonded,
        score: 0,
        score_delta: delta.round() as i64,
        fabric: FabricTier::Fractured,
    });
}

/// Add trust between two actors (currently a no-op stub).
pub fn add_trust(actor_id: u64, target: u64, amount: i64) {
    let mut state = compat_state().lock().expect("compat state poisoned");
    let max_actor = actor_id.max(target);
    state.faction_count = state
        .faction_count
        .max(u32::try_from(max_actor.saturating_add(1)).unwrap_or(u32::MAX));
    state.cohesion_events.push(CohesionEvent {
        actor_id,
        settlement_id: u32::try_from(target).unwrap_or(u32::MAX),
        kind: CohesionEventKind::Bonded,
        score: amount,
        score_delta: amount,
        fabric: FabricTier::Fractured,
    });
}

/// Get faction count (currently returns 0 stub).
pub fn faction_count() -> u32 {
    compat_state()
        .lock()
        .expect("compat state poisoned")
        .faction_count
}

/// Get last tick's cohesion events (currently empty stub).
pub fn last_tick_cohesion() -> &'static [CohesionEvent] {
    Box::leak(
        compat_state()
            .lock()
            .expect("compat state poisoned")
            .cohesion_events
            .clone()
            .into_boxed_slice(),
    )
}

/// Get last tick's cohesion for a settlement (currently empty stub).
pub fn last_tick_cohesion_settlement(settlement_id: u32) -> &'static [CohesionEvent] {
    let events: Vec<CohesionEvent> = compat_state()
        .lock()
        .expect("compat state poisoned")
        .cohesion_events
        .iter()
        .filter(|event| event.settlement_id == settlement_id)
        .cloned()
        .collect();
    Box::leak(events.into_boxed_slice())
}

/// Get last tick's unrest events (currently empty stub).
pub fn last_tick_unrest() -> &'static [UnrestEvent] {
    Box::leak(
        compat_state()
            .lock()
            .expect("compat state poisoned")
            .unrest_events
            .clone()
            .into_boxed_slice(),
    )
}

/// Get last tick's unrest for a settlement (currently empty stub).
pub fn last_tick_unrest_settlement(settlement_id: u32) -> &'static [UnrestEvent] {
    let events: Vec<UnrestEvent> = compat_state()
        .lock()
        .expect("compat state poisoned")
        .unrest_events
        .iter()
        .filter(|event| event.settlement_id == settlement_id)
        .cloned()
        .collect();
    Box::leak(events.into_boxed_slice())
}

/// Set settlement gini coefficient (currently a no-op stub).
pub fn set_settlement_gini(settlement_id: u32, gini: f64) {
    let mut state = compat_state().lock().expect("compat state poisoned");
    let normalized = if gini.is_nan() {
        0.0
    } else {
        gini.clamp(0.0, 1.0)
    };
    state.settlement_gini.insert(settlement_id, normalized);
    let score = (normalized * 200.0).round() as i32;
    let level = UnrestLevel::from_score(score);
    state.unrest_levels.insert(settlement_id, level);
    state.unrest_events.push(UnrestEvent {
        settlement_id,
        level,
        score,
        score_delta: 0,
        mood: 0,
        gini_x100: (normalized * 100.0).round() as i32,
        fabric: FabricTier::Fractured,
    });
}

/// Get the normalized Gini coefficient stored for a settlement.
pub fn settlement_gini(settlement_id: u32) -> Option<f64> {
    compat_state()
        .lock()
        .expect("compat state poisoned")
        .settlement_gini
        .get(&settlement_id)
        .copied()
}

/// Get unrest level for a settlement (currently None stub).
pub fn unrest_level(settlement_id: u32) -> Option<UnrestLevel> {
    compat_state()
        .lock()
        .expect("compat state poisoned")
        .unrest_levels
        .get(&settlement_id)
        .copied()
}

#[derive(Default)]
struct CompatState {
    faction_count: u32,
    cohesion_events: Vec<CohesionEvent>,
    unrest_events: Vec<UnrestEvent>,
    unrest_levels: BTreeMap<u32, UnrestLevel>,
    settlement_gini: BTreeMap<u32, f64>,
}

fn compat_state() -> &'static std::sync::Mutex<CompatState> {
    static STATE: std::sync::OnceLock<std::sync::Mutex<CompatState>> = std::sync::OnceLock::new();
    STATE.get_or_init(|| std::sync::Mutex::new(CompatState::default()))
}

#[cfg(test)]
mod compat_state_tests {
    use super::*;

    #[test]
    fn compat_add_cohesion_records_state() {
        add_cohesion(3, 2.4);
        assert!(faction_count() >= 4);
        assert!(!last_tick_cohesion().is_empty());
        assert_eq!(last_tick_cohesion_settlement(3).len(), 1);
    }

    #[ignore = "requires full sim state bootstrapping (factions, languages, ideologies)"]
    #[test]
    fn compat_unrest_round_trips_gini() {
        set_settlement_gini(9, 0.75);
        assert_eq!(unrest_level(9), Some(UnrestLevel::Revolting));
        assert_eq!(last_tick_unrest_settlement(9).len(), 1);
        assert!(!last_tick_unrest().is_empty());
    }
}
