//! Tests for FR-CIV-MCP-004 — God action tool parity
//!
//! Epic: FR-CIV-MCP
//! Verifies that god action verbs have dedicated MCP tools.

#[cfg(test)]
mod fr_fr_civ_mcp_004 {
    /// FR-CIV-MCP-004: Every WS legacy god verb has a dedicated MCP tool.
    #[test]
    fn legacy_god_verbs_have_mcp_tools() {
        use civis_mcp::god_verb_parity::{mcp_tool_for_ws_god_verb, WS_LEGACY_GOD_VERBS};
        for verb in WS_LEGACY_GOD_VERBS {
            assert!(mcp_tool_for_ws_god_verb(verb).is_some(), "missing MCP tool for {verb}");
        }
    }

    /// FR-CIV-MCP-004: Life/disaster verbs have dedicated MCP tools.
    #[test]
    fn life_disaster_verbs_have_mcp_tools() {
        use civis_mcp::god_verb_parity::{mcp_tool_for_ws_god_verb, ws_disaster_life_verbs};
        for verb in ws_disaster_life_verbs() {
            assert!(mcp_tool_for_ws_god_verb(verb).is_some(), "missing MCP tool for {verb}");
        }
    }

    /// FR-CIV-MCP-004: life.spawn_organism single-agent params are correct.
    #[test]
    fn life_spawn_single_params() {
        use civis_mcp::god_verb_parity::build_life_spawn_god_action_params;
        let params = build_life_spawn_god_action_params(1, 0.25, 0.75, Some(2), Some(42));
        assert_eq!(params["action"], "life.spawn_organism");
        assert_eq!(params["target_faction"], 2);
    }
}
