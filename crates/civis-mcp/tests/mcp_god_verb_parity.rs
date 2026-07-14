//! MCP disaster/life verb parity with Live/WS panel (GAP-MCP-PARITY-001).

use civis_mcp::god_verb_parity::{
    build_life_spawn_god_action_params, mcp_tool_for_ws_god_verb, ws_disaster_life_verbs,
    WS_LEGACY_GOD_VERBS,
};
use civis_mcp::{tool_names, TOOL_NAMES};

#[test]
fn dedicated_mcp_tools_registered_for_ws_disaster_life_verbs() {
    let registered = tool_names();
    for verb in ws_disaster_life_verbs() {
        let tool = mcp_tool_for_ws_god_verb(verb).expect("catalog entry");
        assert!(
            registered.iter().any(|name| name == tool),
            "registered router missing `{tool}` for Live/WS verb `{verb}`"
        );
        assert!(
            TOOL_NAMES.contains(&tool),
            "TOOL_NAMES missing `{tool}` for Live/WS verb `{verb}`"
        );
    }
}

#[test]
fn dedicated_mcp_tools_registered_for_ws_legacy_god_verbs() {
    let registered = tool_names();
    for verb in WS_LEGACY_GOD_VERBS {
        let tool = mcp_tool_for_ws_god_verb(verb).expect("catalog entry");
        assert!(
            registered.iter().any(|name| name == tool),
            "registered router missing `{tool}` for legacy verb `{verb}`"
        );
    }
}

#[test]
fn sim_spawn_organism_params_use_god_action_life_verbs() {
    let single = build_life_spawn_god_action_params(1, 0.1, 0.9, Some(1), None);
    assert_eq!(single["action"], "life.spawn_organism");

    let herd = build_life_spawn_god_action_params(8, 0.1, 0.9, Some(1), Some(99));
    assert_eq!(herd["action"], "life.spawn_herd");
    assert_eq!(herd["count"], 8);
    assert_eq!(herd["seed_civilian_id"], 99);
}
