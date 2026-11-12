use civ_engine::{CivSaveBundle, Simulation};

#[test]
fn saved_market_prices_survive_reload() {
    let dir = tempfile::tempdir().unwrap();
    let mut sim = Simulation::with_seed(42);
    sim.market_state.prices.insert("food".to_owned(), 4321);
    CivSaveBundle::save_dir(dir.path(), &sim).unwrap();
    let loaded = CivSaveBundle::load_dir(dir.path()).unwrap();
    assert_eq!(loaded.market_state, sim.market_state);
}

#[test]
fn saved_research_queue_survives_reload() {
    let dir = tempfile::tempdir().unwrap();
    let mut sim = Simulation::with_seed(42);
    sim.research_cache_mut()
        .queued
        .push_back("pottery".to_owned());
    CivSaveBundle::save_dir(dir.path(), &sim).unwrap();
    let loaded = CivSaveBundle::load_dir(dir.path()).unwrap();
    assert_eq!(loaded.research_cache(), sim.research_cache());
}

#[test]
fn saved_market_and_research_continue_after_reload() {
    let dir = tempfile::tempdir().unwrap();
    let mut original = Simulation::with_seed(42);
    original.market_state.prices.insert("food".to_owned(), 4321);
    original
        .research_cache_mut()
        .queued
        .push_back("pottery".to_owned());
    CivSaveBundle::save_dir(dir.path(), &original).unwrap();
    let mut loaded = CivSaveBundle::load_dir(dir.path()).unwrap();
    original.tick();
    loaded.tick();
    assert_eq!(loaded.market_state, original.market_state);
    assert_eq!(loaded.research_cache(), original.research_cache());
}
