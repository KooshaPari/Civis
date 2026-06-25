//! FR-CIV-GOV-001/002/003 — `phase_institutions` (temple + garrison levels).
//!
//! STATUS: TDD red step. Each test below references the public API the
//! green-step implementation in `crates/engine/src/engine.rs` must provide.
//! The tests deliberately do NOT use the `add_cohesion` / `agent_entity` /
//! `micro_actor_action_count` stubs (which are themselves typed
//! incorrectly — they take `u32`/`u64` arguments but their call sites pass
//! nothing, so the engine currently does not compile cleanly).
//!
//! Spec authority: `agileplus-specs/civ-007-diplomacy-laws-government/spec.md`
//!
//! Covered IDs (3):
//!   FR-CIV-GOV-001 — `phase_institutions` spawns a temple when a settlement
//!                    reaches `temple_unlock_population` and a garrison when
//!                    it reaches `garrison_unlock_population`.
//!   FR-CIV-GOV-002 — `last_tick_institution_events()` exposes the per-tick
//!                    event stream (spawned/upgraded/demolished) so the
//!                    HUD and ws_bridge can render the civil layer.
//!   FR-CIV-GOV-003 — upgrades are gated by settlement population thresholds
//!                    (Temple L1→L2 at N1, Garrison L1→L2 at N2) and emit
//!                    `InstitutionEvent::Upgraded` events.
use civ_engine::{InstitutionEvent, InstitutionKind, Simulation, GENESIS};

const TEMPLE_UNLOCK: u32 = 50;
const GARRISON_UNLOCK: u32 = 120;
const TEMPLE_L2_UNLOCK: u32 = 250;
const GARRISON_L2_UNLOCK: u32 = 500;

// ------------------------------------------------------------- FR-CIV-GOV-001
/// Covers FR-CIV-GOV-001.
/// `phase_institutions` MUST spawn a temple when a settlement population
/// crosses the `temple_unlock_population` threshold and a garrison when it
/// crosses `garrison_unlock_population`.
#[test]
fn fr_civ_gov_001_spawns_when_settlement_crosses_threshold() {
    let mut sim = Simulation::with_seed(GENESIS);
    // Pre-population: no institutions.
    assert!(
        sim.last_tick_institution_events()
            .iter()
            .all(|e| !matches!(e.kind, InstitutionKind::Temple | InstitutionKind::Garrison)),
        "phase_institutions emitted before any settlement reached the unlock threshold"
    );

    // Drive population past the temple threshold (but below garrison).
    sim.set_settlement_population(0, TEMPLE_UNLOCK + 1);
    sim.advance_ticks(1);

    let events = sim.last_tick_institution_events();
    assert!(
        events.iter().any(|e| matches!(
            e.kind,
            InstitutionKind::Temple if e.level == 1 && e.settlement_id == 0
        )),
        "Temple L1 must spawn at settlement 0 once population crosses {}; got events: {:?}",
        TEMPLE_UNLOCK,
        events
    );
    assert!(
        events.iter().all(|e| !matches!(e.kind, InstitutionKind::Garrison)),
        "Garrison must NOT spawn below {} population",
        GARRISON_UNLOCK
    );

    // Drive past the garrison threshold.
    sim.set_settlement_population(0, GARRISON_UNLOCK + 1);
    sim.advance_ticks(1);

    let events = sim.last_tick_institution_events();
    assert!(
        events.iter().any(|e| matches!(
            e.kind,
            InstitutionKind::Garrison if e.level == 1 && e.settlement_id == 0
        )),
        "Garrison L1 must spawn at settlement 0 once population crosses {}; got: {:?}",
        GARRISON_UNLOCK,
        events
    );
}

// ------------------------------------------------------------- FR-CIV-GOV-002
/// Covers FR-CIV-GOV-002.
/// `last_tick_institution_events()` MUST expose the per-tick event stream so
/// the HUD and ws_bridge can render the civil layer.
#[test]
fn fr_civ_gov_002_events_accessible_via_accessor() {
    let mut sim = Simulation::with_seed(GENESIS);

    // Initial tick: no events (no settlement has reached any threshold).
    sim.advance_ticks(1);
    assert_eq!(
        sim.last_tick_institution_events().len(),
        0,
        "fresh simulation must start with zero institution events"
    );

    // Cross the threshold; the next tick MUST emit at least one event.
    sim.set_settlement_population(0, TEMPLE_UNLOCK + 1);
    sim.advance_ticks(1);

    let events = sim.last_tick_institution_events();
    assert!(
        !events.is_empty(),
        "phase_institutions must emit at least one event after the threshold is crossed"
    );

    // The accessor MUST be re-entrant and deterministic across re-reads
    // (clients re-poll the accessor each frame).
    let again = sim.last_tick_institution_events();
    assert_eq!(
        events, again,
        "last_tick_institution_events must return the same slice on repeated reads"
    );

    // After a no-op tick, the accessor MUST reset to the new tick's events
    // (not replay the previous ones).
    sim.advance_ticks(1);
    assert_eq!(
        sim.last_tick_institution_events().len(),
        0,
        "no-op tick must reset the per-tick event stream"
    );
}

// ------------------------------------------------------------- FR-CIV-GOV-003
/// Covers FR-CIV-GOV-003.
/// Upgrades MUST be gated by settlement population thresholds (Temple
/// L1->L2 at `temple_l2_unlock_population`, Garrison L1->L2 at
/// `garrison_l2_unlock_population`) and MUST emit an `Upgraded` event.
#[test]
fn fr_civ_gov_003_upgrade_gated_by_population_threshold() {
    let mut sim = Simulation::with_seed(GENESIS);

    // Reach Temple L1.
    sim.set_settlement_population(0, TEMPLE_UNLOCK + 1);
    sim.advance_ticks(1);
    assert!(
        sim.last_tick_institution_events()
            .iter()
            .any(|e| matches!(e.kind, InstitutionKind::Temple if e.level == 1)),
        "Temple L1 must spawn before the upgrade test begins"
    );

    // Cross the L2 threshold.
    sim.set_settlement_population(0, TEMPLE_L2_UNLOCK + 1);
    sim.advance_ticks(1);

    let events = sim.last_tick_institution_events();
    let upgrade = events.iter().find(|e| {
        matches!(e, InstitutionEvent { kind: InstitutionKind::Temple, level: 2, .. })
    });
    assert!(
        upgrade.is_some(),
        "Temple L1 -> L2 upgrade must emit an InstitutionEvent at level=2; got: {:?}",
        events
    );

    // Same logic for Garrison.
    sim.set_settlement_population(0, GARRISON_L2_UNLOCK + 1);
    sim.advance_ticks(1);

    let events = sim.last_tick_institution_events();
    assert!(
        events.iter().any(|e| matches!(
            e.kind,
            InstitutionKind::Garrison if e.level == 2
        )),
        "Garrison L1 -> L2 upgrade must emit an InstitutionEvent at level=2; got: {:?}",
        events
    );

    // Same-spawn must NOT re-fire (the upgrade is one-shot, not a regression).
    sim.advance_ticks(1);
    assert!(
        sim.last_tick_institution_events()
            .iter()
            .all(|e| !matches!(e, InstitutionEvent { kind: InstitutionKind::Temple, level: 2, .. })),
        "Temple L2 must not re-emit on subsequent ticks once upgraded"
    );
}

// ------------------------------------------------------------- bonus determinism
/// Bonus check: institution events must be deterministic for a given seed.
/// Two simulations seeded identically and run the same number of ticks must
/// produce identical institution event sequences. This guards against the
/// `phase_institutions` implementation accidentally consuming non-seeded RNG.
#[test]
fn fr_civ_gov_determinism_same_seed_same_events() {
    let seed = GENESIS;

    let mut a = Simulation::with_seed(seed);
    a.set_settlement_population(0, TEMPLE_L2_UNLOCK + 1);
    a.advance_ticks(1);
    let a_events = a.last_tick_institution_events().to_vec();

    let mut b = Simulation::with_seed(seed);
    b.set_settlement_population(0, TEMPLE_L2_UNLOCK + 1);
    b.advance_ticks(1);
    let b_events = b.last_tick_institution_events().to_vec();

    assert_eq!(
        a_events, b_events,
        "two simulations with the same seed must produce identical institution events"
    );
}