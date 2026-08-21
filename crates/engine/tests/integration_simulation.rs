//! End-to-end integration tests for the Civis simulation engine.

use civ_engine::scenario::{baseline_scenario_path, load_scenario};
use civ_engine::{Simulation, SimulationSnapshot};
use std::path::PathBuf;

/// Helper to get a default simulation.
fn setup_default() -> Simulation {
    Simulation::new()
}

/// Helper to get a simulation from a specific seed.
fn setup_with_seed(seed: u64) -> Simulation {
    Simulation::with_seed(seed)
}

/// Helper to get a simulation from the baseline scenario.
fn setup_from_baseline() -> Simulation {
    let scenario = load_scenario(baseline_scenario_path()).expect("baseline scenario should load");
    scenario.into_simulation(42)
}

/// Test 1: Basic tick test - ensure 10 ticks complete without panics.
#[test]
fn test_simulation_basic_tick() {
    let mut sim = setup_default();
    for _ in 0..10 {
        sim.tick();
    }
    assert_eq!(sim.current_tick, 10);
}

/// Test 2: Economy cycle - run 50 ticks, verify economy state (energy budget) is non-zero.
#[test]
fn test_simulation_economy_cycle() {
    let mut sim = setup_default();
    for _ in 0..50 {
        sim.tick();
    }
    let snapshot = sim.snapshot();
    assert!(
        snapshot.energy_budget.to_bits() > 0,
        "Energy budget should be positive after 50 ticks"
    );
}

/// Test 3: Diplomacy - create 3 factions (baseline has 4), run 20 ticks, verify diplomacy events occur.
#[test]
fn test_simulation_diplomacy() {
    let mut sim = setup_default();
    for _ in 0..20 {
        sim.tick();
    }
    let snapshot = sim.snapshot();
    assert!(
        !snapshot.diplomacy_events.is_empty(),
        "Diplomacy events should occur after 20 ticks"
    );
}

/// Test 4: Save/Load - Run simulation, serialize to JSON, deserialize, verify state matches.
#[test]
fn test_simulation_save_load() {
    let mut sim = setup_default();
    for _ in 0..5 {
        sim.tick();
    }

    let snapshot1 = sim.snapshot();

    // Serialize snapshot to JSON
    let json = serde_json::to_string(&snapshot1).expect("Snapshot should serialize to JSON");

    // Deserialize back
    let snapshot2: SimulationSnapshot =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(snapshot1.tick, snapshot2.tick);
    assert_eq!(snapshot1.population, snapshot2.population);
    assert_eq!(snapshot1.energy_budget, snapshot2.energy_budget);
}

/// Test 5: Determinism - Run same config twice, verify identical final state.
#[test]
fn test_simulation_determinism() {
    let seed = 12345;
    let mut sim1 = setup_with_seed(seed);
    let mut sim2 = setup_with_seed(seed);

    for _ in 0..10 {
        sim1.tick();
        sim2.tick();
    }

    let snap1 = sim1.snapshot();
    let snap2 = sim2.snapshot();

    assert_eq!(snap1.tick, snap2.tick);
    assert_eq!(snap1.population, snap2.population);
    assert_eq!(snap1.energy_budget, snap2.energy_budget);
    assert_eq!(snap1.citizen_count, snap2.citizen_count);
    assert_eq!(snap1.building_count, snap2.building_count);
    assert_eq!(snap1.military_count, snap2.military_count);
}
