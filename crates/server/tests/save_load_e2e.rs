//! End-to-end save/load round-trip test for civ-server (FR-CIV-TEST-021).
//!
//! Verifies that a running simulation can be serialized and deserialized
//! without data loss, and that post-load ticks maintain determinism.

use civ_server::Server;
use civ_server::config::ServerConfig;
use civ_engine::WorldState;
use std::path::PathBuf;
use tempfile::TempDir;

fn test_config(data_dir: &PathBuf) -> ServerConfig {
    ServerConfig {
        data_dir: Some(data_dir.clone()),
        addr: "127.0.0.1:0".parse().unwrap(),
        ..Default::default()
    }
}

fn setup_server(data_dir: &PathBuf) -> Server {
    let config = test_config(data_dir);
    Server::new(config).expect("server should start cleanly")
}

#[test]
fn save_load_round_trip_preserves_world_state() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("saves");
    std::fs::create_dir_all(&data_dir).unwrap();

    let mut server = setup_server(&data_dir);

    // Tick a few times to build state
    for _ in 0..5 {
        server.tick().expect("tick should succeed");
    }

    // Save the world
    let save_id = "test-roundtrip";
    server.save_world(save_id).expect("save should succeed");

    // Capture pre-load state
    let pre_pop = server.world().map(|w| w.population).unwrap_or(0);
    let pre_belief = server.world().map(|w| w.belief).unwrap_or(0);

    // Load the saved world
    server.load_world(save_id).expect("load should succeed");

    // Post-load state must match
    let post_pop = server.world().map(|w| w.population).unwrap_or(0);
    let post_belief = server.world().map(|w| w.belief).unwrap_or(0);

    assert_eq!(
        pre_pop, post_pop,
        "population must survive save/load round-trip"
    );
    assert_eq!(
        pre_belief, post_belief,
        "belief must survive save/load round-trip"
    );
}

#[test]
fn post_load_ticks_are_deterministic() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("saves");
    std::fs::create_dir_all(&data_dir).unwrap();

    let mut server = setup_server(&data_dir);

    // Build some state
    for _ in 0..3 {
        server.tick().expect("tick should succeed");
    }

    // Save
    server.save_world("det-test").expect("save");

    // Tick a few times and capture state
    for _ in 0..2 {
        server.tick().expect("tick");
    }
    let post_first = server.world().map(|w| w.population).unwrap_or(0);

    // Reload
    server.load_world("det-test").expect("load");

    // Tick same number of times
    for _ in 0..2 {
        server.tick().expect("tick");
    }
    let post_second = server.world().map(|w| w.population).unwrap_or(0);

    assert_eq!(
        post_first, post_second,
        "post-load ticks must be deterministic"
    );
}

#[test]
fn save_load_rpc_round_trip() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("saves");
    std::fs::create_dir_all(&data_dir).unwrap();

    let mut server = setup_server(&data_dir);

    // Tick to create state
    server.tick().expect("tick");

    // Use JSON-RPC-style save command
    let save_result = server.handle_command("save", Some(&serde_json::json!({
        "name": "rpc-test"
    })));
    assert!(save_result.is_ok(), "save via RPC should succeed: {:?}", save_result.err());

    // Load via RPC
    let load_result = server.handle_command("load", Some(&serde_json::json!({
        "name": "rpc-test"
    })));
    assert!(load_result.is_ok(), "load via RPC should succeed: {:?}", load_result.err());
}

#[test]
fn save_load_preserves_emergence_state() {
    let tmp = TempDir::new().unwrap();
    let data_dir = tmp.path().join("saves");
    std::fs::create_dir_all(&data_dir).unwrap();

    let mut server = setup_server(&data_dir);

    // Run enough ticks to trigger emergence phase
    for _ in 0..10 {
        server.tick().expect("tick");
    }

    // Save
    server.save_world("emergence-test").expect("save");

    // Capture emergence metrics
    let pre_last = server.world()
        .and_then(|w| w.last_emergence_sample.clone())
        .unwrap_or_default();

    // Load
    server.load_world("emergence-test").expect("load");

    // Post-load emergence state must match
    let post_last = server.world()
        .and_then(|w| w.last_emergence_sample.clone())
        .unwrap_or_default();

    assert_eq!(
        pre_last.social_mood,
        post_last.social_mood,
        "emergence social_mood must round-trip"
    );
    assert_eq!(
        pre_last.cohesion,
        post_last.cohesion,
        "emergence cohesion must round-trip"
    );
}
