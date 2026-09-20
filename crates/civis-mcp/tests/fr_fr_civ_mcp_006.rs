//! Tests for FR-CIV-MCP-006 — JSON-RPC request envelope
//!
//! Epic: FR-CIV-MCP
//! Verifies the JSON-RPC 2.0 envelope is correctly formed.

#[cfg(test)]
mod fr_fr_civ_mcp_006 {
    /// FR-CIV-MCP-006: build_rpc_request produces valid JSON-RPC 2.0.
    #[test]
    fn rpc_envelope_is_jsonrpc_2() {
        let frame = civis_mcp::build_rpc_request("sim.snapshot", serde_json::json!({"tick": 1}));
        let parsed: serde_json::Value = serde_json::from_str(&frame).expect("valid JSON");
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["method"], "sim.snapshot");
        assert_eq!(parsed["id"], 1);
    }

    /// FR-CIV-MCP-006: build_rpc_request works for empty params.
    #[test]
    fn rpc_envelope_empty_params() {
        let frame = civis_mcp::build_rpc_request("health", serde_json::json!({}));
        let parsed: serde_json::Value = serde_json::from_str(&frame).expect("valid JSON");
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["method"], "health");
    }
}
