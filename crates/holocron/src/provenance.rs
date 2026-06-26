//! Verb provenance: where the verb is reachable from and what it does.

use serde::{Deserialize, Serialize};

/// Where a verb can be invoked from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provenance {
    /// Reachable from the MCP server (`crates/civis-mcp`).
    Mcp,
    /// Reachable from the JSON-RPC server (`crates/server`).
    JsonRpc,
    /// Reachable from the WebSocket bridge (`crates/server/src/ws_bridge`).
    WebSocket,
    /// Reachable from the egui HUD (`crates/hud`).
    Hud,
    /// Internal-only verb (debug, diagnostics). Not user-facing.
    Internal,
}

impl Provenance {
    /// Stable string label used in the registry catalog.
    pub fn label(self) -> &'static str {
        match self {
            Provenance::Mcp => "MCP",
            Provenance::JsonRpc => "JSON-RPC",
            Provenance::WebSocket => "WebSocket",
            Provenance::Hud => "HUD",
            Provenance::Internal => "Internal",
        }
    }
}