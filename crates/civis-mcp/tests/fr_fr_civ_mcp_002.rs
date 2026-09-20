//! Tests for FR-CIV-MCP-002 — Tool surface verification
//!
//! Epic: FR-CIV-MCP
//! Verifies that the MCP tool surface is consistent and complete.

#[cfg(test)]
mod fr_fr_civ_mcp_002 {
    /// FR-CIV-MCP-002: TOOL_NAMES constant is non-empty.
    #[test]
    fn tool_names_constant_is_nonempty() {
        use civis_mcp::TOOL_NAMES;
        assert!(!TOOL_NAMES.is_empty());
    }

    /// FR-CIV-MCP-002: tool_names() returns sorted unique names matching TOOL_NAMES.
    #[test]
    fn tool_names_fn_matches_constant() {
        let names = civis_mcp::tool_names();
        let mut expected: Vec<&str> = civis_mcp::TOOL_NAMES.to_vec();
        expected.sort();
        let actual: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        assert_eq!(actual, expected);
    }

    /// FR-CIV-MCP-002: Required tools (health, snapshot, census) are present.
    #[test]
    fn required_tools_present() {
        let names = civis_mcp::tool_names();
        for required in &["civis_health", "civis_snapshot", "civis_census"] {
            assert!(names.iter().any(|n| n == required), "missing {required}");
        }
    }
}
