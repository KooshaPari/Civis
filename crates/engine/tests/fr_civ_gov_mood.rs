//! TDD red step for `phase_social_mood` — FR-CIV-GOV-010.
//!
//! Per `agileplus-specs/civ-007-diplomacy-laws-government/spec.md`,
//! social mood is computed per settlement per tick from:
//!   - food surplus (more food = higher mood)
//!   - housing capacity (less overcrowding = higher mood)
//!   - crime pressure (less crime = higher mood)
//!   - institution bonuses (Temple/Garrison add fixed bonuses)
//!
//! Mood is a signed i32 in `[-1000, +1000]`. `mood_delta` is signed:
//!   - delta > 0  -> settlement mood improved this tick
//!   - delta == 0 -> no change
//!   - delta < 0  -> settlement mood worsened this tick
//!
//! `mood` saturates at [-1000, +1000] and `mood_delta` is the difference
//! between the new mood and the prior tick's stored mood (or 0 if no prior).
//!
//! All mood scores are computed deterministically from the inputs:
//!   food_score   = clamp(stocked_food / FOOD_DIVISOR, -200, +200)
//!   housing_score = clamp(housing_capacity / HOUSING_DIVISOR, -200, +200)
//!   crime_score  = max(0, 300 - 4 * crime_pressure)            (always >= 0)
//!   temple_bonus   = +TEMPLE_BONUS if settlement has Temple
//!   garrison_bonus = +GARRISON_BONUS if settlement has Garrison
//!
//! These formulas are the deterministic reference for the green-step
//! implementation. They were chosen to be simple, integer, and stable
//! under fixed-point math (no floating-point, no NaN/Inf risk).

use civ_engine::{MoodSnapshot, Simulation};

pub const MOOD_SEED: u64 = 0xC1C0DA7A_5EED_F00D;

/// Divisor for the food-score formula. Tuned so that 200 stocked food = +1 score.
pub const FOOD_DIVISOR: i32 = 200;

/// Divisor for the housing-score formula. Tuned so that 200 housing = +1 score.
pub const HOUSING_DIVISOR: i32 = 200;

/// Maximum negative housing score (saturated when housing is severely short).
pub const HOUSING_NEG_CAP: i32 = -200;

/// Maximum positive housing score (saturated when housing is overprovisioned).
pub const HOUSING_POS_CAP: i32 = 200;

/// Maximum positive food score.
pub const FOOD_POS_CAP: i32 = 200;

/// Maximum negative food score (severely starved).
pub const FOOD_NEG_CAP: i32 = -200;

/// Linear coefficient on crime_pressure for the crime_score formula.
pub const CRIME_COEFFICIENT: i32 = 4;

/// Constant offset for the crime_score formula (clamped at 0 minimum).
pub const CRIME_CONSTANT: i32 = 300;

/// Mood bonus from a Temple L1 institution.
pub const TEMPLE_BONUS: i32 = 50;

/// Mood bonus from a Garrison L1 institution.
pub const GARRISON_BONUS: i32 = 30;

/// Mood floor (very-depressed settlements).
pub const MOOD_FLOOR: i32 = -1000;

/// Mood ceiling (euphoric settlements).
pub const MOOD_CEILING: i32 = 1000;

#[test]
fn fr_civ_gov_010_base_emits_one_mood_snapshot_per_settlement_per_tick() {
    let mut sim = Simulation::with_seed(MOOD_SEED);
    sim.set_settlement_population(0, 60);
    sim.set_settlement_food_stocked(0, 1_000);
    sim.set_settlement_housing_capacity(0, 60);
    sim.set_settlement_crime_pressure(0, 0);
    sim.advance_ticks(1);

    let snapshot = sim
        .last_tick_mood(0)
        .expect("settlement 0 should have a mood snapshot after 1 tick");
    assert_eq!(snapshot.settlement_id, 0, "snapshot keyed to settlement 0");
    assert_eq!(
        snapshot.mood_delta, 0,
        "first tick has no prior mood, so delta is 0"
    );
    assert!(
        snapshot.mood >= MOOD_FLOOR && snapshot.mood <= MOOD_CEILING,
        "mood must be in [{MOOD_FLOOR}, {MOOD_CEILING}], got {}",
        snapshot.mood
    );
}

#[test]
fn fr_civ_gov_010_food_score_scales_with_stocked_food() {
    let mut sim = Simulation::with_seed(MOOD_SEED);
    sim.set_settlement_population(0, 100);
    sim.set_settlement_housing_capacity(0, 100);
    sim.set_settlement_crime_pressure(0, 0);
    sim.set_settlement_food_stocked(0, 2_000);
    sim.advance_ticks(1);
    let snap_a = sim.last_tick_mood(0).unwrap();
    assert_eq!(
        snap_a.food_score, 10,
        "2000 stocked food / 200 divisor = 10 food_score"
    );

    sim.set_settlement_food_stocked(0, 60_000);
    sim.advance_ticks(1);
    let snap_b = sim.last_tick_mood(0).unwrap();
    assert_eq!(
        snap_b.food_score, FOOD_POS_CAP,
        "60000 / 200 = 300, but saturated at FOOD_POS_CAP=200"
    );

    sim.set_settlement_food_stocked(0, -2_000);
    sim.advance_ticks(1);
    let snap_c = sim.last_tick_mood(0).unwrap();
    assert_eq!(
        snap_c.food_score, FOOD_NEG_CAP,
        "-2000 / 200 = -10, but saturated at FOOD_NEG_CAP=-200"
    );
}

#[test]
fn fr_civ_gov_010_crime_score_uses_linear_decreasing_formula() {
    let mut sim = Simulation::with_seed(MOOD_SEED);
    sim.set_settlement_population(0, 100);
    sim.set_settlement_housing_capacity(0, 100);
    sim.set_settlement_food_stocked(0, 0);
    sim.set_settlement_crime_pressure(0, 0);
    sim.advance_ticks(1);
    let snap_zero = sim.last_tick_mood(0).unwrap();
    assert_eq!(
        snap_zero.crime_score, 300,
        "0 crime_pressure -> CRIME_CONSTANT (300)"
    );

    sim.set_settlement_crime_pressure(0, 50);
    sim.advance_ticks(1);
    let snap_mid = sim.last_tick_mood(0).unwrap();
    assert_eq!(
        snap_mid.crime_score, 100,
        "300 - 4*50 = 100"
    );

    sim.set_settlement_crime_pressure(0, 200);
    sim.advance_ticks(1);
    let snap_high = sim.last_tick_mood(0).unwrap();
    assert_eq!(
        snap_high.crime_score, 0,
        "300 - 4*200 = -500, but clamped at 0 (crime_score never negative)"
    );
}

#[test]
fn fr_civ_gov_010_institution_bonuses_apply_when_settlement_has_temple_or_garrison() {
    let mut sim = Simulation::with_seed(MOOD_SEED);
    sim.set_settlement_population(0, 100);
    sim.set_settlement_housing_capacity(0, 100);
    sim.set_settlement_food_stocked(0, 0);
    sim.set_settlement_crime_pressure(0, 0);
    sim.advance_ticks(1);
    let snap_no_inst = sim.last_tick_mood(0).unwrap();
    assert_eq!(snap_no_inst.temple_bonus, 0);
    assert_eq!(snap_no_inst.garrison_bonus, 0);

    // Spawn a Temple by raising population past unlock threshold (50).
    sim.set_settlement_population(0, 100);
    sim.advance_ticks(1);
    let snap_temple = sim.last_tick_mood(0).unwrap();
    assert_eq!(
        snap_temple.temple_bonus, TEMPLE_BONUS,
        "Temple L1 should grant TEMPLE_BONUS to mood"
    );

    // Now spawn a Garrison (pop >= 120).
    sim.set_settlement_population(0, 150);
    sim.advance_ticks(1);
    let snap_both = sim.last_tick_mood(0).unwrap();
    assert_eq!(snap_both.temple_bonus, TEMPLE_BONUS);
    assert_eq!(
        snap_both.garrison_bonus, GARRISON_BONUS,
        "Garrison L1 should grant GARRISON_BONUS to mood"
    );
}

#[test]
fn fr_civ_gov_010_determinism_identical_seeds_produce_identical_snapshots() {
    fn run(seed: u64) -> Vec<MoodSnapshot> {
        let mut sim = Simulation::with_seed(seed);
        sim.set_settlement_population(0, 80);
        sim.set_settlement_population(1, 200);
        sim.set_settlement_food_stocked(0, 1_500);
        sim.set_settlement_food_stocked(1, 5_000);
        sim.set_settlement_housing_capacity(0, 80);
        sim.set_settlement_housing_capacity(1, 200);
        sim.set_settlement_crime_pressure(0, 10);
        sim.set_settlement_crime_pressure(1, 50);
        sim.advance_ticks(3);
        (0..2)
            .map(|id| {
                sim.last_tick_mood(id)
                    .expect("settlement should have mood snapshot")
            })
            .collect()
    }

    let a = run(MOOD_SEED);
    let b = run(MOOD_SEED);
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        assert_eq!(x.mood, y.mood, "settlement {i}: mood mismatch");
        assert_eq!(x.food_score, y.food_score);
        assert_eq!(x.housing_score, y.housing_score);
        assert_eq!(x.crime_score, y.crime_score);
        assert_eq!(x.temple_bonus, y.temple_bonus);
        assert_eq!(x.garrison_bonus, y.garrison_bonus);
        assert_eq!(x.mood_delta, y.mood_delta);
    }

    // Different seed must differ somewhere.
    let c = run(MOOD_SEED.wrapping_add(1));
    assert!(
        a.iter().zip(c.iter()).any(|(x, y)| x.mood != y.mood),
        "different seeds should not produce identical mood streams (sanity)"
    );
}
