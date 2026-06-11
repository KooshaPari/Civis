//! `civis-cli` — programmatic verification harness for Civis.
//!
//! This crate is the *only* source of truth for visual + entity-count
//! evidence. Agents that need to claim "the renderer produced X" or "the sim
//! has N agents" must invoke one of the bins / tools defined here, capture
//! the resulting JSON, and reference the numbers in their report. The
//! [`fr-ax-dx-ux-maturity-audit.md`](../docs/development-guide/fr-ax-dx-ux-maturity-audit.md)
//! policy ("only pixel numbers + entity counts count as evidence") is enforced
//! by routing every visual claim through this crate.
//!
//! Four subcommands are exported as bins:
//!
//! | Bin | Purpose | Transport |
//! |-----|---------|-----------|
//! | `civis-verify` | Launch a windowed Bevy 0.18 client, wait N ticks, capture frame to PNG | Bevy `Screenshot` entity + `save_to_disk` observer |
//! | `civis-pixels` | Sample RGB grid points on a PNG, emit JSON statistics (mean RGB, pct near-black, pct gray, distinct hue count) | local file (PNG only) |
//! | `civis-census` | Query entity counts / sim stats via the WS JSON-RPC bridge | civ-server `ws://host:port/ws` |
//! | `civis-mcp`   | Thin JSON-RPC server exposing `verify`/`pixels`/`census` as MCP-shaped tools | stdin/stdout newline-delimited JSON |
//!
//! ## Public layout
//!
//! - [`pixels`] — pure pixel-statistics functions (no I/O). Unit-tested on
//!   synthetic images so the gate is deterministic.
//! - [`census`] — pure JSON-RPC dispatcher + response struct decoders; no
//!   network — the bin provides the transport.
//! - [`verify`] — types only (no Bevy runtime); the bin wires Bevy.
//! - [`config`] — `.env` + `CIV_*` env-var helpers, shared by every bin.
//!
//! ## Why a CLI instead of a `tools/` subcommand on `civ-server`?
//!
//! `civ-server` must stay focused on the live tick loop. A second binary
//! process avoids an extra plugin lifecycle inside the server and keeps the
//! verifier reusable against replay, snapshot, or mod-hosted simulations in
//! future phases (see PR `feat/verify-harness` alternatives section).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod census;
pub mod config;
pub mod pixels;

#[cfg(feature = "bevy")]
pub mod verify;

/// Library version of the harness (semver-compatible; `0.1.0` until the
/// Bevy 0.18 / RON schema freezes).
pub const HARNESS_VERSION: &str = "0.1.0";

/// Resolve the path of the workspace root that contains this crate.
///
/// Walks upward from `CARGO_MANIFEST_DIR` to its parent; this is the
/// directory a `cargo run -p civis-cli` invocation considers CWD-equivalent.
/// Used by the MCP shim to anchor relative output paths.
#[must_use]
pub fn workspace_root() -> Option<std::path::PathBuf> {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().map(std::path::Path::to_path_buf)
}

/// Re-export of the canonical JSON-RPC method names used by [`census`]. Useful
/// for test fixtures and MCP tool shims.
pub mod jsonrpc {
    pub use civ_server::jsonrpc::JsonRpcMethod;
}
