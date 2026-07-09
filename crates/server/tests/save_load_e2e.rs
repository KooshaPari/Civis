//! End-to-end save/load round-trip test for civ-server (FR-CIV-TEST-021).

use civ_engine::{CivSaveBundle, Simulation};
use tempfile::TempDir;

fn snapshot(sim: &Simulation) -> (u64, u64, u64, u64, u64) {
    (
        sim.state.tick,
        sim.state.population,
        sim.state.belief,
        sim.state.cohesion,
        sim.state.unrest,
    )
}

#[test]
fn save_load_round_trip_preserves_world_state() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("roundtrip.civsave.zst");

    let mut sim = Simulation::default();
    for _ in 0..5 {
        sim.tick();
    }

    let before = snapshot(&sim);
    CivSaveBundle::save_archive(&path, &sim).expect("save should succeed");

    let loaded = CivSaveBundle::load(&path).expect("load should succeed");
    let after = snapshot(&loaded);

    assert_eq!(before, after, "simulation state must survive save/load");
}

#[test]
fn post_load_ticks_are_deterministic() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("determinism.civsave.zst");

    let mut sim = Simulation::default();
    for _ in 0..3 {
        sim.tick();
    }

    CivSaveBundle::save_archive(&path, &sim).expect("save");

    let mut first = CivSaveBundle::load(&path).expect("load first");
    let mut second = CivSaveBundle::load(&path).expect("load second");

    for _ in 0..2 {
        first.tick();
        second.tick();
    }

    assert_eq!(snapshot(&first), snapshot(&second));
}
