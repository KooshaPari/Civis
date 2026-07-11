//! FR-EMERGENCE anti-theater integration coverage.

use civ_engine::Simulation;

const RUN_SEEDS: &[u64] = &[7, 31, 97];

#[test]
fn fr_emergence_anti_theater_seeded_runs_expose_civic_structure() {
    for &seed in RUN_SEEDS {
        let mut sim = Simulation::with_seed(seed);
        sim.set_settlement_population(0, 60);
        sim.set_settlement_food_stocked(0, 1_000);
        sim.set_settlement_housing_capacity(0, 60);
        sim.set_settlement_crime_pressure(0, 10);

        sim.advance_ticks(2);

        assert_eq!(sim.snapshot().tick, 2, "seed {seed}");
        assert!(
            sim.last_tick_mood(0).is_some(),
            "seed {seed} did not expose a settlement mood snapshot"
        );
        assert!(
            !sim.faction_doctrines().is_empty(),
            "seed {seed} did not expose faction doctrine state"
        );
    }
}

#[test]
fn fr_emergence_anti_theater_legends_status_matches_live_simulation() {
    let mut sim = Simulation::with_seed(42);
    sim.advance_ticks(8);

    let status = sim.legends_query("status", None, None, None);

    assert_eq!(status.tick, sim.snapshot().tick);
    assert_eq!(status.node_count, sim.legends_graph().node_count());
    assert!(status.query_api_version > 0);
}
