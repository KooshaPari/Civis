//! Coverage tests for uncovered pub fns in `civis-mcp` (FR-CIV-TEST-008).
//!
//! The existing integration tests in `mcp_integration.rs` cover
//! `tool_names`, `tool_router`, and `pixels_tool_payload`.  This file
//! adds coverage for the two remaining uncovered pub surfaces:
//!
//! * `HARNESS_VERSION` — must be non-empty and match the package version
//!   format so MCP clients can correlate evidence packets with the build.
//! * `census_config_with_url()` — must return a `CensusConfig` whose WS
//!   URL is a valid non-empty string even when no env vars are set (the
//!   function must not panic or return an empty URL).

use civis_mcp::{HARNESS_VERSION, census_config_with_url};

/// `HARNESS_VERSION` must be non-empty and look like a semver string
/// (at least two dots, e.g. "0.1.0").  A blank or wrongly formatted
/// version would silently break the MCP evidence-packet correlation.
#[test]
fn harness_version_is_non_empty_semver_like() {
    assert!(
        !HARNESS_VERSION.is_empty(),
        "HARNESS_VERSION must not be empty"
    );
    // Semver has at least two dot separators: "MAJOR.MINOR.PATCH".
    let dot_count = HARNESS_VERSION.chars().filter(|&c| c == '.').count();
    assert!(
        dot_count >= 2,
        "HARNESS_VERSION '{HARNESS_VERSION}' does not look like a semver string (need >= 2 dots)"
    );
}

/// `census_config_with_url()` must succeed (no panic) even without env
/// vars, and the resulting config's WS URL must be a non-empty string
/// that starts with a recognized scheme.  This guards against regressions
/// where a refactor of `census_config_from_env` would cause the MCP shim
/// to hand a blank URL to the runtime and then hang on connect.
#[test]
fn census_config_with_url_returns_valid_config() {
    // Temporarily clear env vars so we exercise the default-fallback path.
    // We restore them via a guard pattern even on panic.
    struct EnvGuard {
        key: &'static str,
        old: Option<String>,
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.old {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
        }
    }

    let _g1 = EnvGuard {
        key: "CIVIS_WS_URL",
        old: std::env::var("CIVIS_WS_URL").ok(),
    };
    let _g2 = EnvGuard {
        key: "CENSUS_WS_URL",
        old: std::env::var("CENSUS_WS_URL").ok(),
    };
    std::env::remove_var("CIVIS_WS_URL");
    std::env::remove_var("CENSUS_WS_URL");

    // Must not panic.
    let config = census_config_with_url();
    let url = config.ws_url();
    assert!(!url.is_empty(), "ws_url() must return a non-empty string");
    // The default must be a WebSocket URL (ws:// or wss://).
    let is_ws_scheme = url.starts_with("ws://") || url.starts_with("wss://");
    assert!(
        is_ws_scheme,
        "ws_url() returned '{url}' which is not a ws:// or wss:// URL"
    );
}