//! FR-CIV-UNREST-001/002/003 TDD red step for `phase_unrest`
//!
//! Pins the public API the green step must provide. This file is INTENTIONALLY
//! failing to compile until the green step lands in
//! `crates/engine/src/engine.rs`.
//!
//! Phase A5 of the civ-007-diplomacy-laws-government epic. Builds on:
//!   - PR #775: phase_institutions + phase_social_mood (mood source)
//!   - PR #784: phase_stratification (Gini source)
//!   - PR #787: phase_cohesion (fabric source)
//!
//! `phase_unrest` converts per-settlement (mood, gini, fabric) into
//! unrest-level events (Stable, Restless, Rioting, Revolting) and exposes
//! them as a per-tick stream for downstream phases (war, laws, diplomacy).
//!
//! Spec: agileplus-specs/civ-007-diplomacy-laws-government/spec.md
//! Branch: feat/phase-unrest

use civ_engine::{
    CohesionSnapshot, FabricTier, MoodSnapshot, Sim, SimSeed, StratificationReport,
};

const UNREST_SEED: u64 = 0xA5_A5_00_03;

// --- Public API the green step must provide -------------------------------

/// Per-settlement unrest level after `phase_unrest` runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnrestLevel {
    /// No unrest signals. Citizens are content.
    Stable,
    /// Early warning signs: minor discontent, no events yet.
    Restless,
    /// Active civil disorder: riots and protests.
    Rioting,
    /// Open revolt: government is being challenged.
    Revolting,
}

impl UnrestLevel {
    /// Map a numeric unrest score [0, 400] to a level.
    pub fn from_score(score: i32) -> Self {
        match score {
            s if s < 50 => UnrestLevel::Stable,
            s if s < 150 => UnrestLevel::Restless,
            s if s < 300 => UnrestLevel::Rioting,
            _ => UnrestLevel::Revolting,
        }
    }
}

/// Per-tick unrest event.
#[derive(Debug, Clone, PartialEq)]
pub struct UnrestEvent {
    pub settlement_id: u32,
    pub level: UnrestLevel,
    pub score: i32,
    pub score_delta: i32,
    pub mood: i32,
    pub gini_x100: i32,
    pub fabric: FabricTier,
}

/// Per-settlement unrest snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct UnrestSnapshot {
    pub settlement_id: u32,
    pub level: UnrestLevel,
    pub score: i32,
    pub events_count: u32,
}

fn make_sim(seed: u64) -> Sim {
    Sim::with_seed(SimSeed::from_u64(seed))
}

// --- Test 1: FR-CIV-UNREST-001.base ----------------------------------------
/// Happy-path settlement (positive mood, low gini, tight fabric) is Stable.
#[test]
fn fr_civ_unrest_001_base_happy_path_is_stable() {
    let mut sim = make_sim(UNREST_SEED);
    sim.set_settlement_population(0, 200);
    sim.set_settlement_food_stocked(0, 500);
    sim.set_settlement_housing_capacity(0, 250);
    sim.set_settlement_crime_pressure(0, 10);
    // Wealth spread is tight (low gini expected)
    for hid in 0..50u64 {
        sim.register_household(hid);
        sim.register_household_in_settlement(0, hid);
        sim.set_household_wealth(hid, 100 + (hid as i64 % 10));
        sim.set_household_power(hid, 10);
    }
    // Tight fabric: actor with strong kinship + trust + low hardship
    for aid in 0..20u64 {
        sim.set_settlement_actor(aid, 0);
        sim.set_actor_in_settlement_hardship(aid, 5);
        sim.set_actor_in_settlement_institutions(aid, true, true);
        sim.register_kinship(
            aid,
            civ_engine::KinshipEdge {
                kind: civ_engine::KinshipKind::Family,
                target: (aid + 1) % 20,
            },
        );
        sim.register_kinship(
            aid,
            civ_engine::KinshipEdge {
                kind: civ_engine::KinshipKind::Clan,
                target: (aid + 5) % 20,
            },
        );
        sim.add_trust(aid, (aid + 1) % 20, 30);
    }

    sim.tick();
    let snap = sim
        .last_tick_unrest_settlement(0)
        .expect("settlement 0 should have unrest snapshot");
    assert_eq!(snap.settlement_id, 0);
    assert_eq!(snap.level, UnrestLevel::Stable, "happy settlement should be Stable");
    assert!(snap.score < 50, "happy settlement score should be < 50, got {}", snap.score);
}

// --- Test 2: FR-CIV-UNREST-002.composite ---------------------------------
/// Unrest combines mood + gini + fabric into a single score.
#[test]
fn fr_civ_unrest_002_composite_score_uses_mood_gini_fabric() {
    let mut sim = make_sim(UNREST_SEED);
    sim.set_settlement_population(0, 100);
    sim.set_settlement_food_stocked(0, 10); // -> low mood
    sim.set_settlement_housing_capacity(0, 10); // -> low mood
    sim.set_settlement_crime_pressure(0, 50); // -> low mood

    // High gini: one rich actor, many poor
    sim.register_household(0);
    sim.register_household_in_settlement(0, 0);
    sim.set_household_wealth(0, 100_000);
    sim.set_household_power(0, 100);
    for hid in 1..50u64 {
        sim.register_household(hid);
        sim.register_household_in_settlement(0, hid);
        sim.set_household_wealth(hid, 1);
        sim.set_household_power(hid, 1);
    }
    // High hardship, no institutions -> low fabric
    for aid in 0..10u64 {
        sim.set_settlement_actor(aid, 0);
        sim.set_actor_in_settlement_hardship(aid, 200);
        sim.set_actor_in_settlement_institutions(aid, false, false);
    }

    sim.tick();
    let snap = sim
        .last_tick_unrest_settlement(0)
        .expect("settlement 0 should have unrest snapshot");
    assert_eq!(snap.settlement_id, 0);
    // score must combine: low mood, high gini, fractured fabric
    assert!(snap.score >= 200, "composite should be very high, got {}", snap.score);
    assert_eq!(snap.level, UnrestLevel::Revolting);
    assert!(snap.events_count > 0);
}

// --- Test 3: FR-CIV-UNREST-003.events ------------------------------------
/// `last_tick_unrest() -> &[UnrestEvent]` exposes per-event stream.
#[test]
fn fr_civ_unrest_003_last_tick_unrest_returns_event_stream() {
    let mut sim = make_sim(UNREST_SEED);
    sim.set_settlement_population(0, 100);
    sim.set_settlement_food_stocked(0, 0);
    sim.set_settlement_housing_capacity(0, 0);
    sim.set_settlement_crime_pressure(0, 200);
    sim.register_household(0);
    sim.register_household_in_settlement(0, 0);
    sim.set_household_wealth(0, 1);
    sim.set_household_power(0, 0);
    sim.set_settlement_actor(0, 0);
    sim.set_actor_in_settlement_hardship(0, 300);
    sim.set_actor_in_settlement_institutions(0, false, false);

    sim.tick();
    let events = sim.last_tick_unrest();
    assert!(!events.is_empty(), "should emit at least one unrest event");
    assert_eq!(events[0].settlement_id, 0);
    assert!(matches!(events[0].level, UnrestLevel::Rioting | UnrestLevel::Revolting));
}

// --- Test 4: FR-CIV-UNREST-004.transitions ------------------------------
/// Unrest can escalate or de-escalate across ticks.
#[test]
fn fr_civ_unrest_004_level_can_deescalate_across_ticks() {
    let mut sim = make_sim(UNREST_SEED);
    sim.set_settlement_population(0, 100);
    sim.set_settlement_food_stocked(0, 0); // bad mood
    sim.set_settlement_housing_capacity(0, 0);
    sim.set_settlement_crime_pressure(0, 200);
    sim.register_household(0);
    sim.register_household_in_settlement(0, 0);
    sim.set_household_wealth(0, 1);
    sim.set_household_power(0, 0);
    sim.set_settlement_actor(0, 0);
    sim.set_actor_in_settlement_hardship(0, 300);
    sim.set_actor_in_settlement_institutions(0, false, false);

    sim.tick();
    let snap_high = sim
        .last_tick_unrest_settlement(0)
        .expect("settlement 0 unrest snapshot")
        .level;

    // Now improve mood and reduce hardship
    sim.set_settlement_food_stocked(0, 1000);
    sim.set_settlement_housing_capacity(0, 200);
    sim.set_settlement_crime_pressure(0, 0);
    sim.set_actor_in_settlement_hardship(0, 0);
    sim.set_actor_in_settlement_institutions(0, true, true);

    sim.tick();
    let snap_low = sim
        .last_tick_unrest_settlement(0)
        .expect("settlement 0 unrest snapshot")
        .level;

    let rank_high = match snap_high {
        UnrestLevel::Stable => 0,
        UnrestLevel::Restless => 1,
        UnrestLevel::Rioting => 2,
        UnrestLevel::Revolting => 3,
    };
    let rank_low = match snap_low {
        UnrestLevel::Stable => 0,
        UnrestLevel::Restless => 1,
        UnrestLevel::Rioting => 2,
        UnrestLevel::Revolting => 3,
    };
    assert!(
        rank_low < rank_high,
        "improving mood/hardship/institutions should lower unrest (high={:?}, low={:?})",
        snap_high,
        snap_low,
    );
}

// --- Test 5: FR-CIV-UNREST-005.determinism ------------------------------
/// Identical seeds produce identical unrest streams.
#[test]
fn fr_civ_unrest_005_determinism_identical_seeds_match() {
    fn run(seed: u64) -> Vec<UnrestEvent> {
        let mut sim = make_sim(seed);
        sim.set_settlement_population(0, 100);
        sim.set_settlement_food_stocked(0, 50);
        sim.set_settlement_housing_capacity(0, 50);
        sim.set_settlement_crime_pressure(0, 80);
        sim.register_household(0);
        sim.register_household_in_settlement(0, 0);
        sim.set_household_wealth(0, 100);
        sim.set_household_power(0, 10);
        sim.set_settlement_actor(0, 0);
        sim.set_actor_in_settlement_hardship(0, 100);
        sim.set_actor_in_settlement_institutions(0, true, false);
        for _ in 0..4 {
            sim.tick();
        }
        sim.last_tick_unrest().to_vec()
    }
    let a = run(UNREST_SEED);
    let b = run(UNREST_SEED);
    assert_eq!(a, b, "identical seeds must produce identical unrest streams");
    assert!(!a.is_empty());
}

// Compile-time check: ensure the existing simulation API surface still
// matches what the green step will need to consume.
#[allow(dead_code)]
fn _api_check_types(s: MoodSnapshot, _: StratificationReport, _: CohesionSnapshot) {
    let _ = s.settlement_id;
}