//! FR-CIV-UNREST-001/002/003 integration coverage for `phase_unrest`.

use civ_engine::{KinshipEdge, KinshipKind, Simulation, UnrestLevel};

const UNREST_SEED: u64 = 0xA5_A5_00_03;

#[test]
fn fr_civ_unrest_001_happy_settlement_is_stable() {
    let mut sim = Simulation::with_seed(UNREST_SEED);
    sim.set_settlement_population(0, 200);
    sim.set_settlement_food_stocked(0, 40_000);
    sim.set_settlement_housing_capacity(0, 40_000);
    sim.set_settlement_crime_pressure(0, 0);
    sim.set_settlement_gini(0, 0.0);
    sim.set_settlement_actor(1, 0);
    sim.set_settlement_actor(2, 0);
    sim.set_actor_in_settlement_hardship(1, 0);
    sim.set_actor_in_settlement_hardship(2, 0);
    sim.register_kinship(
        1,
        KinshipEdge {
            kind: KinshipKind::Family,
            target: 2,
        },
    );
    sim.add_trust(1, 2, 100);

    sim.tick();

    let snapshot = sim
        .last_tick_unrest_settlement(0)
        .expect("settlement 0 should have an unrest snapshot");
    assert_eq!(snapshot.settlement_id, 0);
    assert_eq!(snapshot.level, UnrestLevel::Stable);
    assert!(snapshot.score < 50, "stable score was {}", snapshot.score);
}

#[test]
fn fr_civ_unrest_002_high_inequality_and_hardship_trigger_unrest() {
    let mut sim = Simulation::with_seed(UNREST_SEED);
    sim.set_settlement_population(0, 100);
    sim.set_settlement_food_stocked(0, 0);
    sim.set_settlement_housing_capacity(0, 0);
    sim.set_settlement_crime_pressure(0, 200);
    sim.set_settlement_gini(0, 1.0);
    sim.set_settlement_actor(1, 0);
    sim.set_actor_in_settlement_hardship(1, 300);

    sim.tick();

    let snapshot = sim
        .last_tick_unrest_settlement(0)
        .expect("settlement 0 should have an unrest snapshot");
    assert!(
        snapshot.score >= 300,
        "high-unrest score was {}",
        snapshot.score
    );
    assert_eq!(snapshot.level, UnrestLevel::Revolting);
    assert!(snapshot.events_count > 0);
    assert!(
        sim.last_tick_unrest()
            .iter()
            .any(|event| event.settlement_id == 0 && event.level == UnrestLevel::Revolting),
        "a Stable-to-Revolting transition must be observable in the event stream"
    );
}

#[test]
fn fr_civ_unrest_003_improved_conditions_deescalate_unrest() {
    let mut sim = Simulation::with_seed(UNREST_SEED);
    sim.set_settlement_population(0, 100);
    sim.set_settlement_food_stocked(0, 0);
    sim.set_settlement_housing_capacity(0, 0);
    sim.set_settlement_crime_pressure(0, 200);
    sim.set_settlement_gini(0, 1.0);
    sim.set_settlement_actor(1, 0);
    sim.set_actor_in_settlement_hardship(1, 300);
    sim.tick();
    let high = sim
        .last_tick_unrest_settlement(0)
        .expect("high-unrest snapshot")
        .level;

    sim.set_settlement_food_stocked(0, 40_000);
    sim.set_settlement_housing_capacity(0, 40_000);
    sim.set_settlement_crime_pressure(0, 0);
    sim.set_settlement_gini(0, 0.0);
    sim.set_actor_in_settlement_hardship(1, 0);
    sim.tick();
    let low = sim
        .last_tick_unrest_settlement(0)
        .expect("deescalated snapshot")
        .level;

    assert!(
        low < high,
        "improved conditions should lower unrest (high={high:?}, low={low:?})"
    );
}
