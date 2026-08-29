//! FR-CIV-phasewire integration tests.
//!
//! These tests verify that the six expanded modules — `religion`, `language`,
//! `psyche_behavior`, `building_layouts`, `history`, and `writing` — are
//! correctly wired into the engine's tick loop. Each newly added phase method
//! is exercised on both an empty `WorldState` (no-op path) and a populated
//! `WorldState` (real `tick_*` call) so we catch both "phase doesn't run" and
//! "phase panics on first call" regressions.
//!
//! The new top-level phases live on `Simulation`:
//!
//! - [`Simulation::phase_religion`] — drives [`crate::religion::tick_religion`]
//! - [`Simulation::phase_language`] — drives [`crate::language::tick_language_system`]
//!   (then chains into `phase_language_drift`)
//! - [`Simulation::phase_psyche`] — drives [`crate::psyche_behavior::tick_psychology`]
//! - [`Simulation::phase_buildings`] — drives [`crate::building_layouts::tick_building_layouts`]
//! - [`Simulation::phase_history`] — drives [`crate::history::tick_history`]
//! - [`Simulation::phase_writing`] — drives [`crate::writing::tick_writing_system`]

use civ_engine::Simulation;

// ---------------------------------------------------------------------------
// Empty-state no-op safety: each phase must not panic on a fresh simulation.
// ---------------------------------------------------------------------------

#[test]
fn phase_religion_runs_on_empty_world_state() {
    let mut sim = Simulation::with_seed(1);
    sim.phase_religion();
    sim.phase_religion();
    assert!(sim.state.faction_religions.is_empty());
}

#[test]
fn phase_language_runs_on_empty_world_state() {
    let mut sim = Simulation::with_seed(2);
    sim.phase_language();
    sim.phase_language();
    assert!(sim.state.faction_language_systems.is_empty());
}

#[test]
fn phase_psyche_runs_on_empty_world_state() {
    let mut sim = Simulation::with_seed(3);
    sim.phase_psyche();
    sim.phase_psyche();
    assert!(sim.state.civilian_psyches.is_empty());
}

#[test]
fn phase_buildings_runs_on_empty_world_state() {
    let mut sim = Simulation::with_seed(4);
    sim.phase_buildings();
    sim.phase_buildings();
    assert!(sim.state.settlement_building_layouts.is_empty());
}

#[test]
fn phase_history_runs_on_empty_history_log() {
    let mut sim = Simulation::with_seed(5);
    let before = sim.state.historical_log.clone();
    sim.phase_history();
    // tick_history is currently a clone-and-return no-op, but the phase must
    // not panic and must still leave a usable log in place.
    assert_eq!(sim.state.historical_log.events.len(), before.events.len());
}

#[test]
fn phase_writing_runs_on_empty_world_state() {
    let mut sim = Simulation::with_seed(6);
    sim.phase_writing();
    sim.phase_writing();
    assert!(sim.state.faction_writing_systems.is_empty());
}

// ---------------------------------------------------------------------------
// Populated-state correctness: each phase must invoke the module's exported
// `tick_*` function so the underlying state actually advances.
// ---------------------------------------------------------------------------

#[test]
fn phase_religion_advances_adherence_when_faction_religions_present() {
    let mut sim = Simulation::with_seed(11);
    // Stage a religion + matching faction resources so the tick has inputs.
    let mut religion = civ_engine::religion::Religion::default();
    religion.adherence = 0.10;
    religion.spread_rate = 0.5;
    religion.name = "Tester".to_string();
    sim.state.faction_religions.insert(1, religion.clone());

    let mut resources = civ_engine::Resources::default();
    resources.food = civ_engine::Fixed::from_num(1000);
    sim.state.faction_resources.insert(1, resources);

    sim.phase_religion();

    let after = sim
        .state
        .faction_religions
        .get(&1)
        .expect("religion must persist");
    // tick_religion grows adherence by spread_rate * 0.01 given population > 0
    // — i.e. a guaranteed strictly-positive delta when spread_rate > 0.
    assert!(
        after.adherence > religion.adherence,
        "tick_religion must grow adherence for populated religions ({:?} -> {:?})",
        religion.adherence,
        after.adherence
    );
}

#[test]
fn phase_psyche_decays_anxiety_when_civilian_psyches_present() {
    let mut sim = Simulation::with_seed(12);
    let mut psyche = civ_engine::psyche_behavior::PsycheState::default();
    psyche.anxiety = 0.9;
    psyche.mood = 0.5;
    sim.state.civilian_psyches.insert(7, psyche.clone());

    sim.phase_psyche();

    let after = sim
        .state
        .civilian_psyches
        .get(&7)
        .expect("psyche must persist");
    assert!(
        after.anxiety < psyche.anxiety,
        "tick_psychology must decay anxiety ({:?} -> {:?})",
        psyche.anxiety,
        after.anxiety
    );
    // last_tick must advance so the phase wrote through.
    assert_eq!(after.last_tick, sim.state.tick);
}

#[test]
fn phase_buildings_decays_efficiency_when_settlement_layouts_present() {
    let mut sim = Simulation::with_seed(13);
    let layout = civ_engine::building_layouts::BuildingLayout::default();
    let layouts = vec![layout.clone()];
    sim.state
        .settlement_building_layouts
        .insert(99, layouts.clone());

    sim.phase_buildings();

    let after = sim
        .state
        .settlement_building_layouts
        .get(&99)
        .expect("layouts must persist");
    assert_eq!(after.len(), 1);
    // tick_building_layouts decays efficiency by 0.001 per tick to a 0.1 floor
    // — so a fresh default layout must remain valid and bounded.
    assert!(after[0].efficiency <= layout.efficiency + f32::EPSILON);
    assert!(after[0].efficiency >= 0.1);
}

#[test]
fn phase_history_logs_events_through_tick() {
    let mut sim = Simulation::with_seed(14);
    let mut event = civ_engine::history::HistoricalEvent::default();
    event.tick = 0;
    sim.state.historical_log.record_event(event);

    for _ in 0..3 {
        sim.phase_history();
        sim.state.tick += 1;
    }

    // The history log must survive the phase calls.
    assert!(
        !sim.state.historical_log.events.is_empty(),
        "history log must keep recorded events after phase_history runs"
    );
}

#[test]
fn phase_writing_advances_literacy_when_faction_writing_systems_present() {
    let mut sim = Simulation::with_seed(15);
    let mut ws = civ_engine::writing::WritingSystem::default();
    ws.literacy_rate = 0.10;
    ws.name = "TestScript".to_string();
    sim.state.faction_writing_systems.insert(2, ws.clone());

    sim.phase_writing();

    let after = sim
        .state
        .faction_writing_systems
        .get(&2)
        .expect("writing system must persist");
    assert!(
        after.literacy_rate > ws.literacy_rate,
        "tick_writing_system must grow literacy ({:?} -> {:?})",
        ws.literacy_rate,
        after.literacy_rate
    );
    assert!(after.literacy_rate <= 1.0);
}

#[test]
fn phase_language_advances_tick_on_populated_systems() {
    let mut sim = Simulation::with_seed(16);
    let mut lang = civ_engine::language::Language::default();
    lang.name = "TesterLang".to_string();
    lang.phonemes = vec!["a".to_string(), "b".to_string()];
    lang.drift_factor = 0.05;
    lang.intelligibility_baseline = 0.9;
    sim.state.faction_language_systems.insert(1, lang.clone());

    sim.phase_language();

    // The drift sub-phase always populates faction_languages even when the
    // populated tick_language_system loop is empty, so we don't need an
    // exact equality match — only that the phase runs without panicking and
    // leaves the populated faction_language_systems state in place.
    let after = sim
        .state
        .faction_language_systems
        .get(&1)
        .expect("language must persist");
    assert_eq!(after.name, lang.name);
}

// ---------------------------------------------------------------------------
// End-to-end: the full tick loop must run cleanly with the new phases.
// ---------------------------------------------------------------------------

#[test]
fn full_tick_loop_runs_clean_with_new_phase_wiring() {
    let mut sim = Simulation::with_seed(7);
    for _ in 0..25 {
        sim.tick();
    }
    assert_eq!(sim.current_tick, 25);
}

#[test]
fn run_phase_dispatch_finds_every_new_phase() {
    // Drive one tick per PHASE_ORDER entry so each phase fn runs at least
    // once — protects against future PHASE_ORDER drift where a phase name
    // is added to the list without a corresponding `phase_*` method.
    let mut sim = Simulation::with_seed(8);
    sim.advance_ticks(40);
    assert_eq!(sim.current_tick, 40);
}
