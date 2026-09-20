//! Attach-mode helpers for the Godot reference client.
//!
//! Implements the small, pure Rust pieces of FR-CIV-GODOT-ATTACH-001 through
//! `-004` (see `docs/development-guide/fr-godot-attach.md`):
//!
//! * `attach_mode_of(url)` — `FR-CIV-GODOT-ATTACH-004`: pick `server` for
//!   `ws://` / `wss://` URLs and `watch` for `http://` / `https://` URLs.
//! * `snapshot_throttle_ms_for(mode, default_ms)` — central throttle knob
//!   mirroring the GDScript `snapshot_throttle_ms` field
//!   (`FR-CIV-GODOT-ATTACH-003`).
//! * `terrain_url_for(watch_base, grid_size)` — compose the `GET /terrain`
//!   URL the Godot client calls against civ-watch when in server attach
//!   (`FR-CIV-GODOT-ATTACH-002`).
//! * `set_speed_rpc_payload(multiplier)` — JSON-RPC payload for the
//!   `sim.set_speed` call the Godot client fires on open
//!   (`FR-CIV-GODOT-ATTACH-001`).

/// String names of the canonical attach modes used by the Godot client
/// inspector + `_on_open` path.
pub const ATTACH_MODE_SERVER: &str = "server";
pub const ATTACH_MODE_WATCH: &str = "watch";
pub const ATTACH_MODE_STANDALONE: &str = "standalone";

/// Default throttle (ms) for `sim.snapshot` refreshes triggered by F3D0 /
/// legacy tick frames (`FR-CIV-GODOT-ATTACH-003`).
///
/// Matches the `snapshot_throttle_ms` default in `civis_ws_client.gd` and
/// matches Unreal's `SnapshotThrottleSec = 0.25` in
/// `clients/unreal-show/Source/CivShow/CivWsClient.cpp`.
pub const DEFAULT_SNAPSHOT_THROTTLE_MS: u64 = 250;

/// **FR-CIV-GODOT-ATTACH-004** — Resolve the attach mode implied by a URL
/// scheme. `ws://` and `wss://` ⇒ `"server"`; `http://` and `https://` ⇒
/// `"watch"`; anything else ⇒ `"standalone"`. Caller is expected to default
/// to `"standalone"` if it gets back something it doesn't know.
#[must_use]
pub fn attach_mode_of(url: &str) -> &'static str {
    let lower = url.trim().to_ascii_lowercase();
    if lower.starts_with("ws://") || lower.starts_with("wss://") {
        ATTACH_MODE_SERVER
    } else if lower.starts_with("http://") || lower.starts_with("https://") {
        ATTACH_MODE_WATCH
    } else {
        ATTACH_MODE_STANDALONE
    }
}

/// **FR-CIV-GODOT-ATTACH-003** — Returns the throttle (in milliseconds) the
/// snapshot-refresh path should use. `server` mode uses the F3D0-driven 250
/// ms throttle; `watch` mode uses a fixed 500 ms cadence (HTTP polls, no
/// push frames); `standalone` falls through to the caller's default.
#[must_use]
pub fn snapshot_throttle_ms_for(mode: &str, default_ms: u64) -> u64 {
    match mode {
        ATTACH_MODE_SERVER => DEFAULT_SNAPSHOT_THROTTLE_MS,
        ATTACH_MODE_WATCH => 500,
        _ => default_ms,
    }
}

/// **FR-CIV-GODOT-ATTACH-002** — Compose the `GET /terrain` URL the Godot
/// client should hit on civ-watch while attached to a server. The path is
/// `GET {base}/terrain?size={grid_size}` so watch's HTTP route can stay
/// single-shape regardless of caller.
#[must_use]
pub fn terrain_url_for(watch_base: &str, grid_size: u16) -> String {
    let trimmed = watch_base.trim_end_matches('/');
    format!("{trimmed}/terrain?size={grid_size}")
}

/// **FR-CIV-GODOT-ATTACH-001** — JSON-RPC payload the Godot client sends to
/// `civ-server` for the `sim.set_speed` call. The server drives its tick
/// loop from this; civilians move without manual tick.
///
/// Mirrors `CivisWsClient.set_speed(multiplier)` in GDScript.
#[must_use]
pub fn set_speed_rpc_payload(multiplier: i32) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "sim.set_speed",
        "params": { "multiplier": multiplier },
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-GODOT-ATTACH-004 — URL scheme maps to attach mode.
    #[test]
    fn attach_mode_of_recognises_schemes() {
        assert_eq!(attach_mode_of("ws://127.0.0.1:3000/ws"), ATTACH_MODE_SERVER);
        assert_eq!(attach_mode_of("wss://example.com/ws"), ATTACH_MODE_SERVER);
        assert_eq!(attach_mode_of("http://127.0.0.1:9090"), ATTACH_MODE_WATCH);
        assert_eq!(attach_mode_of("HTTPS://example.com"), ATTACH_MODE_WATCH);
        assert_eq!(attach_mode_of(""), ATTACH_MODE_STANDALONE);
        assert_eq!(attach_mode_of("ftp://example.com"), ATTACH_MODE_STANDALONE);
    }

    /// FR-CIV-GODOT-ATTACH-003 — server attach uses 250 ms throttle; watch
    /// uses 500 ms; everything else defers to caller.
    #[test]
    fn snapshot_throttle_ms_for_uses_mode_specific_values() {
        assert_eq!(snapshot_throttle_ms_for(ATTACH_MODE_SERVER, 0), 250);
        assert_eq!(snapshot_throttle_ms_for(ATTACH_MODE_WATCH, 0), 500);
        assert_eq!(snapshot_throttle_ms_for(ATTACH_MODE_STANDALONE, 999), 999);
    }

    /// FR-CIV-GODOT-ATTACH-002 — terrain URL strips a trailing slash before
    /// appending the `?size=N` query so callers can pass either
    /// `http://host:9090` or `http://host:9090/`.
    #[test]
    fn terrain_url_for_strips_trailing_slash() {
        assert_eq!(
            terrain_url_for("http://127.0.0.1:9090", 128),
            "http://127.0.0.1:9090/terrain?size=128"
        );
        assert_eq!(
            terrain_url_for("http://127.0.0.1:9090/", 64),
            "http://127.0.0.1:9090/terrain?size=64"
        );
    }

    /// FR-CIV-GODOT-ATTACH-001 — `sim.set_speed` RPC body includes the
    /// multiplier; round-trips through `serde_json`.
    #[test]
    fn set_speed_rpc_payload_carries_multiplier() {
        let body = set_speed_rpc_payload(3);
        let v: serde_json::Value = serde_json::from_str(&body).expect("parse json");
        assert_eq!(v["method"], "sim.set_speed");
        assert_eq!(v["params"]["multiplier"], 3);
        assert_eq!(v["jsonrpc"], "2.0");
    }
}
