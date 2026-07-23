//! Live/WS god-verb catalog parity (GAP-MCP-PARITY-001 / WBS P3.1).
//!
//! Source of truth for substrate life/disaster verbs:
//! `crates/server/src/ws_bridge.rs` `DispatchEffect::GodAction` match arms.

use serde_json::{json, Value};

/// Legacy Bevy god-panel verbs handled by the WS bridge.
pub const WS_LEGACY_GOD_VERBS: &[&str] = &["smite", "bless", "earthquake", "plague", "miracle"];

/// Substrate life verbs on the Live/WS panel.
pub const WS_LIFE_VERBS: &[&str] = &["life.spawn_organism", "life.spawn_herd"];

/// Substrate disaster verbs on the Live/WS panel.
pub const WS_DISASTER_VERBS: &[&str] = &["disaster.wildfire", "disaster.flood"];

/// Every disaster/life verb the Live/WS panel exposes via `sim.god_action`.
pub fn ws_disaster_life_verbs() -> impl Iterator<Item = &'static str> {
    WS_LIFE_VERBS
        .iter()
        .chain(WS_DISASTER_VERBS.iter())
        .copied()
}

/// Map a Live/WS god verb to its dedicated MCP tool name.
pub fn mcp_tool_for_ws_god_verb(verb: &str) -> Option<&'static str> {
    match verb {
        "smite" => Some("civis_god_action_smite"),
        "bless" => Some("civis_god_action_bless"),
        "earthquake" => Some("civis_god_action_earthquake"),
        "plague" => Some("civis_god_action_plague"),
        "miracle" => Some("civis_god_action_miracle"),
        "life.spawn_organism" => Some("civis_god_action_life_spawn_organism"),
        "life.spawn_herd" => Some("civis_god_action_life_spawn_herd"),
        "disaster.wildfire" => Some("civis_god_action_disaster_wildfire"),
        "disaster.flood" => Some("civis_god_action_disaster_flood"),
        _ => None,
    }
}

/// Build `sim.god_action` params for grouped organism spawning.
///
/// Routes through the same substrate verbs as the Live panel:
/// `life.spawn_organism` for a single agent, `life.spawn_herd` otherwise.
pub fn build_life_spawn_god_action_params(
    count: u32,
    x: f32,
    y: f32,
    faction: Option<u32>,
    seed_civilian_id: Option<u64>,
) -> Value {
    let n = count.max(1);
    let mut obj = serde_json::Map::new();
    if n == 1 {
        obj.insert("action".to_owned(), json!("life.spawn_organism"));
        obj.insert("x".to_owned(), json!(x));
        obj.insert("y".to_owned(), json!(y));
    } else {
        obj.insert("action".to_owned(), json!("life.spawn_herd"));
        obj.insert("count".to_owned(), json!(n));
    }
    obj.insert("target_faction".to_owned(), json!(faction.unwrap_or(0)));
    if let Some(id) = seed_civilian_id {
        obj.insert("seed_civilian_id".to_owned(), json!(id));
    }
    Value::Object(obj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_disaster_life_verbs_match_panel_catalog() {
        let verbs: Vec<_> = ws_disaster_life_verbs().collect();
        assert_eq!(
            verbs,
            vec![
                "life.spawn_organism",
                "life.spawn_herd",
                "disaster.wildfire",
                "disaster.flood",
            ]
        );
    }

    #[test]
    fn every_ws_disaster_life_verb_has_dedicated_mcp_tool() {
        for verb in ws_disaster_life_verbs() {
            assert!(
                mcp_tool_for_ws_god_verb(verb).is_some(),
                "missing MCP tool for Live/WS verb `{verb}`"
            );
        }
    }

    #[test]
    fn every_ws_legacy_god_verb_has_dedicated_mcp_tool() {
        for verb in WS_LEGACY_GOD_VERBS {
            assert!(
                mcp_tool_for_ws_god_verb(verb).is_some(),
                "missing MCP tool for Live/WS legacy verb `{verb}`"
            );
        }
    }

    #[test]
    fn life_spawn_params_single_uses_spawn_organism() {
        let params = build_life_spawn_god_action_params(1, 0.25, 0.75, Some(2), Some(42));
        assert_eq!(params["action"], "life.spawn_organism");
        assert!((params["x"].as_f64().unwrap() - 0.25).abs() < 1e-9);
        assert!((params["y"].as_f64().unwrap() - 0.75).abs() < 1e-9);
        assert_eq!(params["target_faction"], 2);
        assert_eq!(params["seed_civilian_id"], 42);
    }

    #[test]
    fn life_spawn_params_herd_uses_spawn_herd() {
        let params = build_life_spawn_god_action_params(5, 0.5, 0.5, None, None);
        assert_eq!(params["action"], "life.spawn_herd");
        assert_eq!(params["count"], 5);
        assert_eq!(params["target_faction"], 0);
        assert!(params.get("x").is_none());
        assert!(params.get("seed_civilian_id").is_none());
    }
}
