use super::*;
use std::{fs, io::Read};

#[tokio::test]
async fn jsonrpc_corrupt_save_load_preserves_active_session() {
    let sim = Arc::new(Mutex::new(Simulation::with_seed(42)));
    {
        let mut active = sim.lock().await;
        active.tick();
        active.state.population = 987_654;
        active.rng_mut().set_word_pos(137);
    }
    let (_dir, state) = test_app_state(sim.clone(), 1, 1, false);
    let path = state.saves_dir.join("slot-1.civsave.zst");
    let saved = Simulation::with_seed(7);
    CivSaveBundle::save_archive(&path, &saved).expect("write valid save");
    assert_eq!(
        CivSaveBundle::load_archive(&path).expect("valid control loads").state.tick,
        0
    );

    let compressed = fs::read(&path).expect("read archive");
    let tar_bytes = zstd::stream::decode_all(compressed.as_slice()).expect("decode archive");
    let mut input = tar::Archive::new(tar_bytes.as_slice());
    let mut output = tar::Builder::new(Vec::new());
    let mut corrupted = false;
    for entry in input.entries().expect("tar entries") {
        let mut entry = entry.expect("tar entry");
        let name = entry.path().expect("entry path").into_owned();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).expect("read entry");
        if name.file_name().is_some_and(|name| name == "world_state.json") {
            assert!(!bytes.is_empty());
            bytes[0] ^= 1;
            corrupted = true;
        }
        let mut header = entry.header().clone();
        header.set_cksum();
        output.append(&header, bytes.as_slice()).expect("append entry");
    }
    assert!(corrupted, "must corrupt the actual world-state component");
    let tar_bytes = output.into_inner().expect("finish tar");
    fs::write(&path, zstd::stream::encode_all(tar_bytes.as_slice(), 3).expect("compress"))
        .expect("write corrupt archive");
    assert!(matches!(
        CivSaveBundle::load_archive(&path),
        Err(civ_engine::SaveBundleError::HashMismatch { .. })
    ));

    let (before_state, before_world, before_rng) = {
        let mut active = sim.lock().await;
        (
            active.state.clone(),
            serde_json::to_value(active.snapshot()).expect("world snapshot"),
            active.rng_mut().clone(),
        )
    };
    let generation = state.scene_generation.load(Ordering::SeqCst);
    let mut role = None;
    let response = handle_jsonrpc_text(
        r#"{"jsonrpc":"2.0","id":1005,"method":"save.load","params":{"slot_name":"slot-1"}}"#,
        &state,
        &mut role,
        test_subscription_filter(),
        "corrupt-load",
    )
    .await;
    let response: serde_json::Value = serde_json::from_str(&response).expect("RPC response");
    assert_eq!(response["id"], 1005);
    assert_eq!(response["error"]["code"], -32603);
    assert!(response.get("result").is_none(), "{response}");
    let mut active = sim.lock().await;
    assert_eq!(active.state, before_state);
    assert_eq!(serde_json::to_value(active.snapshot()).unwrap(), before_world);
    let after_rng = active.rng_mut();
    assert_eq!(after_rng.get_seed(), before_rng.get_seed());
    assert_eq!(after_rng.get_stream(), before_rng.get_stream());
    assert_eq!(after_rng.get_word_pos(), before_rng.get_word_pos());
    assert_eq!(state.tick.load(Ordering::SeqCst), before_state.tick);
    assert_eq!(state.scene_generation.load(Ordering::SeqCst), generation);
}
