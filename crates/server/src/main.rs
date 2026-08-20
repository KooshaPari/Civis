use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use civ_engine::{CivSaveBundle, Simulation};
use civ_server::{most_recent_save_path, run_ws_bridge, TickBroadcastFormat, WsBridgeConfig};
use civ_observability::{init_observability, ObservabilityConfig};
use opentelemetry::trace::TracerProvider as _;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
#[tokio::main]
async fn main() {
    let provider = init_observability(ObservabilityConfig {
        service_name: "civ-server".to_string(),
        otlp_endpoint: None, // reads from OTEL_EXPORTER_OTLP_ENDPOINT env
        prometheus_port: None, // default 9090
    });
    let tracer = provider.tracer("civ-server");
    // Initialize tracing with OpenTelemetry layer
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    let port = std::env::var("CIV_SERVER_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);
    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], port));
    let max_clients = std::env::var("CIVIS_WS_MAX_CLIENTS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(16);
    let saves_dir: PathBuf = std::env::var("CIVIS_SAVES_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("saves"));
    let replays_dir: PathBuf = std::env::var("CIVIS_REPLAYS_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("replays"));
    // `CIV_AUTOLOAD=1` seeds the bridge from the freshest on-disk save
    // (slot > autosave > manual, mtime desc within tier). Off by default so
    // CI runs stay reproducible against a fresh `Simulation::default()`.
    let autoload = std::env::var("CIV_AUTOLOAD")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"));
    let map_seed = std::env::var("CIVIS_MAP_SEED")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(42);
    let sim = Arc::new(tokio::sync::Mutex::new(
        initial_simulation(&saves_dir, autoload, map_seed).await,
    ));
    // require_role defaults to true (deny-by-default); operators may disable
    // via the CIVIS_REQUIRE_ROLE=false env var in permissive local-only setups.
    let require_role = std::env::var("CIVIS_REQUIRE_ROLE")
        .map(|v| !v.eq_ignore_ascii_case("false"))
        .unwrap_or(true);

    run_ws_bridge(
        WsBridgeConfig {
            addr,
            max_clients,
            require_role,
            tick_broadcast_format: TickBroadcastFormat::from_env(),
            saves_dir,
            replays_dir,
        },
        sim,
    )
    .await;
}

/// Build the bridge's initial [`Simulation`] (P5 / CIV-1000 §13.5).
///
/// When `autoload` is true and `saves_dir` contains a recognizable save, the
/// freshest entry is loaded via [`CivSaveBundle::load`]. Otherwise the engine
/// starts from [`Simulation::with_seed`], using `CIVIS_MAP_SEED` (default 42).
async fn initial_simulation(
    saves_dir: &std::path::Path,
    autoload: bool,
    map_seed: u64,
) -> Simulation {
    if !autoload {
        return Simulation::with_seed(map_seed);
    }
    let Some(path) = (match most_recent_save_path(saves_dir) {
        Ok(path) => path,
        Err(err) => {
            tracing::warn!(?err, ?saves_dir, "CIV_AUTOLOAD enabled but saves_dir is unreadable; starting from Simulation::default()");
            return Simulation::default();
        }
    }) else {
        tracing::info!(
            ?saves_dir,
            "CIV_AUTOLOAD enabled but no saves found; starting from Simulation::default()"
        );
        return Simulation::default();
    };

    match CivSaveBundle::load(&path) {
        Ok(loaded) => {
            tracing::info!(path = %path.display(), tick = loaded.state.tick, "loaded most recent save on launch");
            loaded
        }
        Err(err) => {
            tracing::warn!(?err, path = %path.display(), "failed to load most recent save; falling back to Simulation::default()");
            Simulation::default()
        }
    }
}
