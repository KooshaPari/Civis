//! NFR-P-08 — `healthz` process summary for the nightly memory regression.

use civ_engine::Simulation;
use civ_server::spawn_ws_bridge;
use std::sync::Arc;

// NFR-P-08 — GET /healthz exposes the process summary (tick, connected
// clients, and the ws delivery counters) as stable JSON numbers for the
// nightly regression reader.
#[tokio::test]
async fn nfr_p_08_healthz_exposes_process_summary_json() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(1)));
    let addr = spawn_ws_bridge(sim, 4).await;

    let response = reqwest::get(format!("http://{addr}/healthz"))
        .await
        .expect("healthz request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let body: serde_json::Value = response.json().await.expect("healthz json");
    assert!(
        body.get("tick").and_then(|v| v.as_u64()).is_some(),
        "healthz must expose tick, got {body}"
    );
    assert!(
        body.get("clients").and_then(|v| v.as_u64()).is_some(),
        "healthz must expose connected client count, got {body}"
    );
    for key in [
        "tick_batches_sent",
        "tick_messages_sent",
        "ws_client_disconnects",
    ] {
        assert!(
            body.get(key).and_then(|v| v.as_u64()).is_some(),
            "healthz must expose {key} for the nightly regression, got {body}"
        );
    }
    // No client ever connected in this test, so the disconnect counter is zero.
    assert_eq!(
        body["ws_client_disconnects"], 0,
        "no client connected, disconnects must be 0"
    );
}
