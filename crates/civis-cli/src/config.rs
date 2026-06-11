//! `.env` + `CIV_*` env-var helpers for the verification harness.
//!
//! No URL / port is hard-coded; every consumer reads from this module.
//! Falls back to the project-level defaults declared in
//! [`crate::census`] when an env var is missing.

use std::env;

use crate::census::CensusConfig;

/// Load `.env` (if present) into the current process. Idempotent — safe to
/// call from every bin's `main`. Errors are non-fatal (the file may be
/// absent in CI) but they are surfaced via `tracing::warn!` so missing
/// configuration is visible in agent logs.
pub fn load_dotenv() {
    if let Err(err) = dotenvy::dotenv() {
        if !matches!(err, dotenvy::Error::Io(ref e) if e.kind() == std::io::ErrorKind::NotFound) {
            tracing::warn!("dotenvy::dotenv failed: {err}");
        }
    }
}

/// Build a [`CensusConfig`] from environment variables.
///
/// | Env var | Default | Notes |
/// |---------|---------|-------|
/// | `CIV_WS_HOST` | `127.0.0.1` | civ-server WebSocket host |
/// | `CIV_SERVER_PORT` | `3000` | civ-server WebSocket port |
/// | `CIV_WS_PATH` | `/ws` | path component |
/// | `CIV_CENSUS_TIMEOUT_MS` | `5000` | per-request timeout |
///
/// Behaviour matches the live Bevy client's `default_live_ws_url()` so the
/// harness and the renderer target the same server.
pub fn census_config_from_env() -> CensusConfig {
    let host = env::var("CIV_WS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("CIV_SERVER_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(3000);
    let path = env::var("CIV_WS_PATH").unwrap_or_else(|_| "/ws".to_string());
    let timeout_ms = env::var("CIV_CENSUS_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(5000);
    CensusConfig {
        host,
        port,
        path,
        timeout_ms,
    }
}

/// Output directory for `civis-verify` frame captures. Default
/// `target/verify-frames/` so the artefacts stay out of the source tree.
pub fn verify_output_dir_from_env() -> std::path::PathBuf {
    env::var("CIV_VERIFY_OUT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("target/verify-frames"))
}

/// Number of frames to let the Bevy render loop settle before capture.
/// Default 60 frames (~1 s at 60 Hz). Higher values are useful when waiting
/// for `civ-server` to stream the first `sim.snapshot` over the WS bridge.
pub fn verify_settle_frames_from_env() -> u32 {
    env::var("CIV_VERIFY_SETTLE_FRAMES")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn census_config_defaults_match_live_client() {
        // Sanity: the defaults here must mirror `civ-bevy-ref` constants.
        // (`DEFAULT_WS_HOST`, `DEFAULT_WS_PORT`, `DEFAULT_WS_PATH`.)
        let cfg = census_config_from_env();
        if env::var("CIV_WS_HOST").is_err() {
            assert_eq!(cfg.host, "127.0.0.1");
        }
        if env::var("CIV_SERVER_PORT").is_err() {
            assert_eq!(cfg.port, 3000);
        }
        if env::var("CIV_WS_PATH").is_err() {
            assert_eq!(cfg.path, "/ws");
        }
        assert!(cfg.timeout_ms >= 100, "timeout must be sane");
    }

    #[test]
    fn verify_output_dir_default_does_not_pollute_repo() {
        // We never want frame captures inside `docs/`, `crates/`, or `web/`.
        let dir = verify_output_dir_from_env();
        let s = dir.to_string_lossy();
        assert!(s.contains("target"), "expected target/ prefix, got {s}");
    }
}
