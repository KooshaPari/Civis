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
    let replays_dir: PathBuf = std::env::var("CIVIS_REPLAYS_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("replays"));
    // `CIV_AUTOLOAD=1` seeds the bridge from the freshest on-disk save
    // (slot > autosave > manual, mtime desc within tier). Off by default so
    // CI runs stay reproducible against a fresh seeded simulation.
    let autoload = std::env::var("CIV_AUTOLOAD")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"));
    let map_seed = std::env::var("CIVIS_MAP_SEED")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(42);
    let sim = Arc::new(Mutex::new(
        initial_simulation(&saves_dir, autoload, map_seed).await,
    ));
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
            tracing::warn!(?err, ?saves_dir, map_seed, "CIV_AUTOLOAD enabled but saves_dir is unreadable; starting from a seeded simulation");
            return Simulation::with_seed(map_seed);
        }
    }) else {
        tracing::info!(
            ?saves_dir,
            map_seed,
            "CIV_AUTOLOAD enabled but no saves found; starting from a seeded simulation"
        );
        return Simulation::with_seed(map_seed);
    };

    match CivSaveBundle::load(&path) {
        Ok(loaded) => {
            tracing::info!(path = %path.display(), tick = loaded.state.tick, "loaded most recent save on launch");
            loaded
        }
        Err(err) => {
            tracing::warn!(?err, path = %path.display(), map_seed, "failed to load most recent save; falling back to a seeded simulation");
            Simulation::with_seed(map_seed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn autoload_fallbacks_preserve_configured_map_seed() {
        const MAP_SEED: u64 = 987_654_321;

        let unreadable_parent = tempdir().expect("unreadable fixture parent");
        let unreadable = unreadable_parent.path().join("not-a-directory");
        std::fs::write(&unreadable, b"file").expect("create non-directory fixture");
        assert!(std::fs::read_dir(&unreadable).is_err());

        let empty = tempdir().expect("empty saves directory");

        let corrupt = tempdir().expect("corrupt saves directory");
        let corrupt_save = corrupt.path().join("slot-1.civsave.zst");
        std::fs::write(&corrupt_save, b"not a save archive").expect("write corrupt save");
        assert_eq!(
            most_recent_save_path(corrupt.path()).expect("inspect corrupt fixture"),
            Some(corrupt_save)
        );

        for (case, saves_dir) in [
            ("unreadable directory", unreadable.as_path()),
            ("no save", empty.path()),
            ("corrupt save", corrupt.path()),
        ] {
            let simulation = initial_simulation(saves_dir, true, MAP_SEED).await;
            assert_eq!(simulation.state.rng_seed, MAP_SEED, "{case} fallback");
        }
    }
}
