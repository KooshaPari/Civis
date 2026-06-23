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
    let sim = Arc::new(Mutex::new(initial_simulation(&saves_dir, autoload).await));

    run_ws_bridge(
        WsBridgeConfig {
            addr,
            max_clients,
            require_role: false,
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
/// starts from [`Simulation::default`].
async fn initial_simulation(saves_dir: &std::path::Path, autoload: bool) -> Simulation {
    if !autoload {
        return Simulation::default();
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
