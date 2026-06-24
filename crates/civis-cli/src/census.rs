//! Pure JSON-RPC census dispatcher + transport config.
//!
//! This module intentionally owns *no* network I/O. The `civis-census` binary
//! wraps a `CensusClient` (see `bin/census.rs`) around the helpers here so
//! the dispatch logic stays unit-testable without a live `civ-server`.
//!
//! The shape mirrors [`civ_server::jsonrpc`] — the harness reuses the canonical
//! method enum, parse helpers, and dispatch table so the wire contract stays
//! in lockstep with the server.

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[cfg(test)]
use serde_json::Value;

use civ_server::jsonrpc::{
    self, DispatchContext, JsonRpcMethod, JsonRpcParseError, JsonRpcRequest, JsonRpcResponse,
    RequestId, JSONRPC_VERSION,
};

/// Wire shape for a JSON-RPC 2.0 request the harness sends to civ-server.
///
/// `civ_server::jsonrpc::JsonRpcRequest` is intentionally not `Serialize` (the
/// bridge is request-side only), so the harness mirrors just the fields it
/// needs when building outbound text frames.
#[derive(Debug, serde::Serialize)]
struct OutboundRequest<'a> {
    /// JSON-RPC version string (always `"2.0"`).
    jsonrpc: &'a str,
    /// Correlates the response.
    id: RequestId,
    /// Wire method name (matches [`JsonRpcMethod::as_str`]).
    method: &'a str,
    /// Method parameters; `{}` for parameter-less calls (forward-compatible
    /// with the server's `parse_request`, which accepts both shapes).
    params: serde_json::Value,
}

/// Transport config for the `civis-census` WebSocket client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CensusConfig {
    /// WebSocket host (`CIV_WS_HOST`, default `127.0.0.1`).
    pub host: String,
    /// WebSocket port (`CIV_SERVER_PORT`, default `3000`).
    pub port: u16,
    /// WebSocket path component (`CIV_WS_PATH`, default `/ws`).
    pub path: String,
    /// Per-request timeout in milliseconds (`CIV_CENSUS_TIMEOUT_MS`, default `5000`).
    pub timeout_ms: u64,
}

impl Default for CensusConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            path: "/ws".to_string(),
            timeout_ms: 5000,
        }
    }
}

impl CensusConfig {
    /// `ws://host:port/path` URL built from this config.
    #[must_use]
    pub fn ws_url(&self) -> String {
        let normalized_path = if self.path.starts_with('/') {
            self.path.clone()
        } else {
            format!("/{}", self.path)
        };
        format!("ws://{}:{}{}", self.host, self.port, normalized_path)
    }

    /// Resolve the per-request timeout as a [`Duration`].
    #[must_use]
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_ms)
    }
}

/// A validated `sim.status` JSON-RPC response payload (the only census endpoint
/// this harness is allowed to gate on; deeper fields are deferred to
/// `sim.snapshot` for L5 dashboards).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SimStatusResult {
    /// Server tick at response time.
    pub tick: u64,
    /// World population, when the bridge could read it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub population: Option<u64>,
    /// Whether the bridge had a live simulation attached. Derived locally
    /// (`population.is_some()`) — the server omits the field, so the
    /// harness fills it in during deserialisation.
    pub live: bool,
}

impl<'de> Deserialize<'de> for SimStatusResult {
    /// Deserialise the wire payload, treating `live` as `population.is_some()`.
    ///
    /// `sim.status` on the bridge returns either `{tick}` (no live sim) or
    /// `{tick, population}` (live). Anything else is a malformed reply.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            tick: u64,
            #[serde(default)]
            population: Option<u64>,
        }
        let wire = Wire::deserialize(deserializer)?;
        let live = wire.population.is_some();
        Ok(Self {
            tick: wire.tick,
            population: wire.population,
            live,
        })
    }
}

impl SimStatusResult {
    /// Parse a JSON-RPC `sim.status` success response into a typed result.
    pub fn from_response(response: &JsonRpcResponse) -> Result<Self, CensusError> {
        let result = response
            .result
            .as_ref()
            .ok_or_else(|| CensusError::MissingField("result".to_string()))?;
        serde_json::from_value(result.clone()).map_err(CensusError::Decode)
    }
}

/// Errors the census client can surface to the operator. Pure-Rust so the MCP
/// shim can map them to a JSON-RPC `error` object without io::Error.
#[derive(Debug, thiserror::Error)]
pub enum CensusError {
    /// The server returned a JSON-RPC error object.
    #[error("sim.status failed: {code} {message}")]
    ServerError {
        /// JSON-RPC error code.
        code: i32,
        /// Human-readable message.
        message: String,
    },
    /// The response was missing an expected top-level field.
    #[error("missing field `{0}` in sim.status response")]
    MissingField(String),
    /// Could not decode the success payload into [`SimStatusResult`].
    #[error("decode sim.status payload: {0}")]
    Decode(#[source] serde_json::Error),
    /// A `sim.status` request could not be constructed (should not happen
    /// for the static `sim.status` call but kept for symmetry with the
    /// generic dispatch path).
    #[error("build sim.status request: {0}")]
    Build(String),
}

/// Build a JSON-RPC 2.0 `sim.status` request payload (text frame ready).
#[must_use]
pub fn build_sim_status_request(id: RequestId) -> String {
    let req = OutboundRequest {
        jsonrpc: JSONRPC_VERSION,
        id,
        method: JsonRpcMethod::SimStatus.as_str(),
        params: serde_json::json!({}),
    };
    serde_json::to_string(&req).expect("sim.status request serializes")
}

/// Build a JSON-RPC 2.0 `sim.snapshot` request payload (text frame ready).
#[must_use]
pub fn build_sim_snapshot_request(id: RequestId) -> String {
    let req = OutboundRequest {
        jsonrpc: JSONRPC_VERSION,
        id,
        method: JsonRpcMethod::SimSnapshot.as_str(),
        params: serde_json::json!({}),
    };
    serde_json::to_string(&req).expect("sim.snapshot request serializes")
}

/// Decode a JSON-RPC text frame into a typed [`JsonRpcResponse`], producing
/// the same `parse_error_response` shape as `civ-server` so the harness stays
/// in lockstep with the bridge's error codes.
pub fn decode_response(text: &str) -> Result<JsonRpcResponse, JsonRpcParseError> {
    let req = jsonrpc::parse_request(text).ok();
    if let Some(req) = req {
        return Ok(JsonRpcResponse {
            jsonrpc: req.jsonrpc,
            id: req.id,
            result: None,
            error: None,
        });
    }
    // The server may have sent a response (not a request) — parse it directly.
    serde_json::from_str::<JsonRpcResponse>(text).map_err(|_| JsonRpcParseError::Parse)
}

/// Validate a [`JsonRpcResponse`] payload as a successful `sim.status` reply.
pub fn validate_sim_status(response: &JsonRpcResponse) -> Result<SimStatusResult, CensusError> {
    if let Some(err) = &response.error {
        return Err(CensusError::ServerError {
            code: err.code,
            message: err.message.clone(),
        });
    }
    SimStatusResult::from_response(response)
}

/// Build a `sim.status`-shaped [`DispatchPlan`] for unit tests. Mirrors the
/// server's `dispatch_request` result so the harness can exercise the parser
/// without a live bridge.
#[must_use]
pub fn dispatch_sim_status_for_test(ctx: &DispatchContext) -> JsonRpcResponse {
    jsonrpc::dispatch_request(
        JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id: RequestId::Number(1),
            method: JsonRpcMethod::SimStatus,
            params: Some(serde_json::json!({})),
        },
        ctx.clone(),
    )
    .response
}

/// Re-export the wire types most consumers will need.
pub mod wire {
    pub use civ_server::jsonrpc::{
        error_code, JsonRpcError, JsonRpcMethod, JsonRpcParseError, JsonRpcRequest,
        JsonRpcResponse, RequestId, JSONRPC_VERSION,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_server::jsonrpc::DispatchEffect;
    use std::collections::BTreeMap;

    #[test]
    fn census_config_default_url_is_localhost_3000() {
        let cfg = CensusConfig::default();
        assert_eq!(cfg.ws_url(), "ws://127.0.0.1:3000/ws");
    }

    #[test]
    fn census_config_url_normalises_path() {
        let cfg = CensusConfig {
            host: "civis".to_string(),
            port: 9000,
            path: "ws".to_string(),
            timeout_ms: 1000,
        };
        assert_eq!(cfg.ws_url(), "ws://civis:9000/ws");
    }

    #[test]
    fn census_config_timeout_is_milliseconds() {
        let cfg = CensusConfig {
            timeout_ms: 1500,
            ..CensusConfig::default()
        };
        assert_eq!(cfg.timeout(), Duration::from_millis(1500));
    }

    #[test]
    fn sim_status_request_serializes_to_wire() {
        let frame = build_sim_status_request(RequestId::Number(7));
        let parsed: serde_json::Value = serde_json::from_str(&frame).expect("json");
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 7);
        assert_eq!(parsed["method"], "sim.status");
    }

    #[test]
    fn sim_snapshot_request_serializes_to_wire() {
        let frame = build_sim_snapshot_request(RequestId::String("snap-1".to_string()));
        let parsed: serde_json::Value = serde_json::from_str(&frame).expect("json");
        assert_eq!(parsed["method"], "sim.snapshot");
        assert_eq!(parsed["id"], "snap-1");
    }

    #[test]
    fn dispatch_sim_status_includes_population_when_ctx_provides_it() {
        let ctx = DispatchContext {
            tick: 12,
            population: Some(1_234),
            snapshot: None,
            require_role: false,
            speed_multiplier: 1,
            connection_role: None,
            saves_dir: None,
            emergence: None,
        };
        let response = dispatch_sim_status_for_test(&ctx);
        let result = validate_sim_status(&response).expect("valid");
        assert_eq!(result.tick, 12);
        assert_eq!(result.population, Some(1_234));
        assert!(result.live);
    }

    #[test]
    fn dispatch_sim_status_marks_live_false_without_population() {
        let ctx = DispatchContext {
            tick: 3,
            population: None,
            snapshot: None,
            require_role: false,
            speed_multiplier: 1,
            connection_role: None,
            saves_dir: None,
            emergence: None,
        };
        let response = dispatch_sim_status_for_test(&ctx);
        let result = validate_sim_status(&response).expect("valid");
        assert_eq!(result.tick, 3);
        assert_eq!(result.population, None);
        assert!(!result.live);
    }

    #[test]
    fn dispatch_sim_status_propagates_error_response() {
        let err_response = JsonRpcResponse::failure(
            RequestId::Number(1),
            civ_server::jsonrpc::JsonRpcError {
                code: -32601,
                message: "Method not found: sim.status".to_string(),
                data: None,
            },
        );
        let err = validate_sim_status(&err_response).unwrap_err();
        match err {
            CensusError::ServerError { code, .. } => assert_eq!(code, -32601),
            other => panic!("expected ServerError, got {other:?}"),
        }
    }

    #[test]
    fn decode_response_accepts_sim_status_wire_shape() {
        let wire = r#"{"jsonrpc":"2.0","id":1,"result":{"tick":42,"population":7}}"#;
        let response = decode_response(wire).expect("parse");
        let result = validate_sim_status(&response).expect("valid");
        assert_eq!(result.tick, 42);
        assert_eq!(result.population, Some(7));
    }

    #[test]
    fn dispatch_plan_effects_do_not_leak_into_response_payload() {
        let ctx = DispatchContext {
            tick: 1,
            population: None,
            snapshot: None,
            require_role: false,
            speed_multiplier: 1,
            connection_role: None,
            saves_dir: None,
            emergence: None,
        };
        let plan = jsonrpc::dispatch_request(
            JsonRpcRequest {
                jsonrpc: JSONRPC_VERSION.to_owned(),
                id: RequestId::Number(9),
                method: JsonRpcMethod::SimStatus,
                params: Some(serde_json::json!({})),
            },
            ctx,
        );
        assert_eq!(plan.effect, DispatchEffect::None);
        assert!(plan.response.error.is_none());
    }

    #[test]
    fn sim_status_request_omits_params_omission_in_wire() {
        // The server's `parse_request` accepts both `{}` and absent params, so
        // the harness always sends `params: {}` to stay forward-compatible.
        let frame = build_sim_status_request(RequestId::Number(3));
        let value: BTreeMap<String, Value> = serde_json::from_str(&frame).expect("json");
        assert!(value.contains_key("params"));
    }
}
