//! Behavioral coverage for FR-NFR-S-01 — max simultaneous WebSocket
//! clients: the server must sustain > 100 concurrent WebSocket clients at
//! 10 ticks/sec with no dropped frames.
//!
//! Acceptance signal from
//! `docs/traceability/fr-nfr-s-01/fr-nfr-s-01-intent.md` and
//! `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3: a load test spins up
//! 100+ concurrent `tokio-tungstenite` clients against `civ-server`, the
//! bridge ticks at 10 Hz, and no client misses frames.
//!
//! This is that load test. [`CLIENTS`] (= 101, i.e. strictly > 100)
//! concurrent WebSocket clients connect to a live `civ-server` ws bridge;
//! each one must observe a non-empty, monotonically non-decreasing tick
//! stream spanning multiple ticks, and the server-side health endpoint
//! must confirm all 101 clients attached with zero disconnects while the
//! 10 Hz tick loop keeps advancing under the load. The observation window
//! is scaled down from the spec's 60 s CI job to keep the test fast; the
//! client count (the actual NFR threshold) is not scaled.
//!
//! The spec-side lint of the §10.3 NFR row lives in
//! `crates/engine/tests/fr_nfr_s_01.rs`.

use std::sync::Arc;
use std::time::{Duration, Instant};

use civ_engine::Simulation;
use civ_protocol_3d::Frame3d;
use civ_server::spawn_ws_bridge;
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio::time::{timeout, timeout_at, Instant as TokioInstant};
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message,
    MaybeTlsStream, WebSocketStream,
};

/// NFR requires **> 100** concurrent clients — 101 is the smallest
/// integer that satisfies the strict inequality, and it doubles as the
/// bridge's capacity gate (connections are rejected at `>= max_clients`).
const CLIENTS: usize = 101;
/// Per-client observation window. The bridge ticks every 100 ms (10 Hz).
const OBSERVE_WINDOW: Duration = Duration::from_millis(2000);
/// Minimum sustained tick rate asserted under load (spec: 10 Hz; this
/// bound leaves 2x headroom for loaded CI runners while still catching a
/// broadcast loop that stalls under > 100 clients).
const MIN_SUSTAINED_TICKS_PER_SEC: f64 = 5.0;

type ClientSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Connect one client, then collect every `Frame3d` tick value observed
/// during [`OBSERVE_WINDOW`]. Returns the observations *and* the live
/// socket, so the connection remains open while server-side health is
/// asserted (dropping it early would race the disconnect counter).
async fn spawn_observer(client_id: usize, url: String) -> (Vec<u64>, ClientSocket) {
    let (mut socket, _) = timeout(Duration::from_secs(10), connect_async(&url))
        .await
        .unwrap_or_else(|_| panic!("client {client_id}: handshake timeout"))
        .unwrap_or_else(|e| panic!("client {client_id}: handshake failed: {e}"));

    let deadline = TokioInstant::now() + OBSERVE_WINDOW;
    let mut observed: Vec<u64> = Vec::new();
    loop {
        let next = timeout_at(deadline, socket.next()).await;
        let frame = match next {
            Ok(Some(Ok(frame))) => frame,
            Ok(Some(Err(e))) => panic!("client {client_id}: frame error: {e}"),
            Ok(None) => panic!("client {client_id}: connection closed mid-window"),
            Err(_) => break, // observation window elapsed
        };
        if let Message::Text(text) = frame {
            // Tick frames are JSON encodings of `Frame3d`; JSON-RPC
            // notifications (e.g. `scene.reset`) are not tick frames and
            // are ignored here.
            if let Ok(decoded) = serde_json::from_str::<Frame3d>(&text) {
                observed.push(decoded.tick());
            }
        }
    }
    (observed, socket)
}

/// Fetch `/healthz` as JSON with a bounded timeout.
async fn get_healthz(addr: std::net::SocketAddr) -> serde_json::Value {
    timeout(Duration::from_secs(5), async {
        reqwest::get(format!("http://{addr}/healthz"))
            .await
            .expect("healthz request")
            .json()
            .await
            .expect("healthz json")
    })
    .await
    .expect("healthz timeout")
}

/// Happy path + edge cases for FR-NFR-S-01:
///
/// * 101 concurrent clients all complete the WebSocket handshake.
/// * Every client sees a non-empty, monotonic tick stream spanning at
///   least three distinct ticks (no stall, no reordering, no silent end).
/// * While all 101 sockets are still open, `/healthz` reports
///   `clients == 101` and `ws_client_disconnects == 0`.
/// * The 10 Hz tick loop keeps advancing under the load at a measured
///   rate of at least [`MIN_SUSTAINED_TICKS_PER_SEC`].
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn more_than_100_concurrent_clients_sustain_the_10hz_tick_stream() {
    let sim = Arc::new(tokio::sync::Mutex::new(Simulation::with_seed(20_260_924)));
    let addr = spawn_ws_bridge(sim, CLIENTS).await;
    let url = format!("ws://{addr}/ws");

    // Connect + observe concurrently; each task returns its socket so the
    // fleet stays connected while we assert server-side health.
    let mut tasks = Vec::with_capacity(CLIENTS);
    for client_id in 0..CLIENTS {
        tasks.push(tokio::spawn(spawn_observer(client_id, url.clone())));
    }
    let mut results = Vec::with_capacity(CLIENTS);
    for task in tasks {
        results.push(task.await.expect("observer task panicked"));
    }

    // --- client-side assertions ---------------------------------------
    for (client_id, (observed, _socket)) in results.iter().enumerate() {
        assert!(
            !observed.is_empty(),
            "client {client_id} received no tick frames in {OBSERVE_WINDOW:?} — stream stalled"
        );
        assert!(
            observed.windows(2).all(|w| w[0] <= w[1]),
            "client {client_id} saw tick order regress (dropped/reordered frames): {observed:?}"
        );
        let mut distinct = observed.clone();
        distinct.dedup();
        assert!(
            distinct.len() >= 3,
            "client {client_id} only saw {} distinct ticks in {OBSERVE_WINDOW:?} at 10 Hz: {observed:?}",
            distinct.len()
        );
    }

    // --- server-side assertions (all 101 sockets still held) -----------
    // Poll until the last handshake is registered server-side.
    let health1 = timeout(Duration::from_secs(5), async {
        loop {
            let health = get_healthz(addr).await;
            if health["clients"].as_u64() == Some(CLIENTS as u64) {
                return health;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("all 101 clients must register with the bridge");
    assert_eq!(
        health1["ws_client_disconnects"].as_u64(),
        Some(0),
        "no client may be dropped under > 100-client load"
    );
    let tick1 = health1["tick"].as_u64().expect("healthz tick");
    assert!(
        tick1 >= 10,
        "the 10 Hz loop should have advanced at least 10 ticks, got {tick1}"
    );
    let messages = health1["tick_messages_sent"]
        .as_u64()
        .expect("healthz tick_messages_sent");
    assert!(
        messages >= CLIENTS as u64,
        "every client should have received tick traffic, got {messages} messages for {CLIENTS} clients"
    );

    // Cadence under sustained load: sample the tick again after ~1.2 s
    // with all clients still attached.
    let started = Instant::now();
    tokio::time::sleep(Duration::from_millis(1200)).await;
    let health2 = get_healthz(addr).await;
    let elapsed = started.elapsed().as_secs_f64();
    assert_eq!(
        health2["clients"].as_u64(),
        Some(CLIENTS as u64),
        "clients must remain attached through the cadence window"
    );
    assert_eq!(
        health2["ws_client_disconnects"].as_u64(),
        Some(0),
        "disconnects must stay at zero for the whole run"
    );
    let tick2 = health2["tick"].as_u64().expect("healthz tick (2)");
    assert!(tick2 >= tick1, "tick must never move backwards");
    let rate = (tick2 - tick1) as f64 / elapsed;
    assert!(
        rate >= MIN_SUSTAINED_TICKS_PER_SEC,
        "tick loop must sustain >= {MIN_SUSTAINED_TICKS_PER_SEC} ticks/sec under {CLIENTS} \
         clients (spec: 10 Hz); measured {rate:.1} Hz ({tick1} -> {tick2} over {elapsed:.2}s)"
    );

    // Release all connections only after every assertion has run.
    drop(results);
}
