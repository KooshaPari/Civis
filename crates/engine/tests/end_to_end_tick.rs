use civ_engine::Simulation;

fn run_ticks(sim: &mut Simulation, ticks: usize) {
    for _ in 0..ticks {
        sim.tick();
    }
}

#[test]
fn tick_loop_changes_population() {
    const TICKS: u64 = 360;
    // Citizen births in `phase_citizen_lifecycle` only fire on ticks where
    // `tick % 200 == 0` (the birth window), and even then per-civilian
    // `birth_chance` is low, so population change is observed across a long
    // run.
    let mut sim = Simulation::with_seed(2024);
    let initial = sim.snapshot();
    let mut saw_population_change = false;

    for _ in 0..TICKS {
        sim.tick();
        if sim.snapshot().population != initial.population {
            saw_population_change = true;
        }
    }

    let final_snapshot = sim.snapshot();

    assert_eq!(final_snapshot.tick, initial.tick + TICKS);
    assert!(
        saw_population_change,
        "population should evolve at least once over repeated ticks"
    );
}

#[test]
fn tick_loop_runs_without_panicking_for_multiple_seeds() {
    for seed in [1_u64, 7, 42, 2024] {
        let mut sim = Simulation::with_seed(seed);
        run_ticks(&mut sim, 32);

        let snapshot = sim.snapshot();
        assert_eq!(snapshot.tick, 32);
        assert!(snapshot.citizen_count > 0);
        assert!(snapshot.building_count > 0);
    }
}
