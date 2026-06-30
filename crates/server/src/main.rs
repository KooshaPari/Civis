use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use civ_engine::{CivSaveBundle, Simulation};
use civ_server::{most_recent_save_path, run_ws_bridge, TickBroadcastFormat, WsBridgeConfig};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
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
    let sim = Arc::new(Mutex::new(initial_simulation(&saves_dir, autoload, map_seed).await));

    // require_role defaults to true (deny-by-default); operators may disable
    // via the CIVIS_REQUIRE_ROLE=false env var in permissive local-only setups.
    let require_role = std::env::var("CIVIS_REQUIRE_ROLE")
        .map(|v| v.to_ascii_lowercase() != "false")
        .unwrap_or(true);
    run_ws_bridge(
        WsBridgeConfig {
            addr,
            max_clients,
            require_role,
            tick_broadcast_format: TickBroadcastFormat::from_env(),
            saves_dir,
            ..Default::default()
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
        // ponytail: keep the server's default boot seed aligned with watch terrain.
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
