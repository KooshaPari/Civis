//! Tests for FR-CIV-MCP-005 — Harness version and census config
//!
//! Epic: FR-CIV-MCP
//! Verifies the harness version and census config are well-formed.

#[cfg(test)]
mod fr_fr_civ_mcp_005 {
    /// FR-CIV-MCP-005: HARNESS_VERSION is non-empty semver.
    #[test]
    fn harness_version_is_semver() {
        use civis_mcp::HARNESS_VERSION;
        assert!(!HARNESS_VERSION.is_empty());
        let dots = HARNESS_VERSION.chars().filter(|&c| c == '.').count();
        assert!(dots >= 2, "expected semver, got {HARNESS_VERSION}");
    }

    /// FR-CIV-MCP-005: census_config_with_url returns valid default config.
    #[test]
    fn census_config_returns_valid_ws_url() {
        let config = civis_mcp::census_config_with_url();
        let url = config.ws_url();
        assert!(!url.is_empty());
        assert!(url.starts_with("ws://") || url.starts_with("wss://"));
    }
}
