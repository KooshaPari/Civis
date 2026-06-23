use civ_engine::Simulation;

fn run_ticks(sim: &mut Simulation, ticks: usize) {
    for _ in 0..ticks {
        sim.tick();
    }
}

#[test]
fn tick_loop_changes_population_and_forms_clusters() {
    // Citizen births in `phase_citizen_lifecycle` only fire on ticks where
    // `tick % 200 == 0` (the birth window), and even then per-civilian
    // `birth_chance` is ~0.003, so a 360-tick run with seed 2024 may
    // legitimately hit zero births. Population change is therefore observed
    // through the death-driven `phase_life` path, which begins producing
    // `state.population` deltas around tick ~352 once `LifeNeeds` decay
    // exceeds the `sickness_onset_ticks` threshold for a subset of agents.
    // Settlements are detected while civilians are still co-located (civ
    // count ~150+); running much past tick 400 lets agents scatter and
    // dissolves the emergent clusters we want to assert on.
    let mut sim = Simulation::with_seed(2024);
    let initial = sim.snapshot();
    let mut saw_population_change = false;

    for _ in 0..360 {
        sim.tick();
        if sim.snapshot().population != initial.population {
            saw_population_change = true;
        }
    }

    let final_snapshot = sim.snapshot();
    let settlement_count = sim.settlement_count();

    assert_eq!(final_snapshot.tick, initial.tick + 360);
    assert!(
        saw_population_change,
        "population should evolve at least once over repeated ticks"
    );
    assert!(
        settlement_count > 0,
        "expected emergent settlement clusters to form after repeated ticks"
    );
    assert!(
        sim.cluster_stocks().len() as u32 >= settlement_count,
        "cluster stock tracking should cover detected settlements"
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
