//! Bridge from the Holocron registry to the existing MCP substrate.
//!
//! Each `VerbDescriptor` registered in Holocron can be linked to the
//! underlying MCP tool name. Firing a verb through Holocron dispatches
//! to `civis_mcp::server::dispatch_tool` so the substrate-faithful
//! invariant holds: MCP, JSON-RPC, egui, and Holocron all fire the
//! same path.
//!
//! This module is intentionally a thin layer — the Holocron registry
//! does not duplicate verb logic, only catalog metadata.

use crate::descriptor::VerbDescriptor;
use crate::registry::VerbRegistry;

/// Returns the MCP tool name for a Holocron verb, if linked.
///
/// Currently this is the convention: verb id == MCP tool name
/// (`civ_world_inspect`, `civ_law_propose`, ...). The MCP bridge in
/// the substrate-faithful phase will assert that every registered
/// MCP tool has a matching Holocron descriptor.
pub fn mcp_tool_name(verb: &VerbDescriptor) -> Option<&str> {
    (verb.provenance == crate::provenance::Provenance::Mcp).then_some(verb.id.as_str())
}

/// Sanity check: verify that every verb with a `mcp_tool` link has
/// a well-formed tool name (lowercase, snake_case, no spaces).
///
/// Returns a list of (verb_id, reason) for any verbs that fail.
pub fn validate_mcp_links(registry: &VerbRegistry) -> Vec<(String, String)> {
    let mut issues = Vec::new();
    for (id, desc) in registry.iter() {
        if let Some(tool) = mcp_tool_name(desc) {
            if tool.is_empty() {
                issues.push((id.to_string(), "empty mcp_tool".into()));
                continue;
            }
            if !tool
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            {
                issues.push((id.to_string(), format!("non-snake_case mcp_tool: {tool}")));
            }
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor::VerbDescriptor;
    use crate::group::VerbGroup;
    use crate::registry::VerbRegistry;

    fn make(id: &'static str) -> VerbDescriptor {
        VerbDescriptor::new(
            id,
            id,
            "test",
            VerbGroup::Civic,
            crate::risk::RiskTier::Minor,
            crate::provenance::Provenance::Mcp,
            &[],
        )
    }

    #[test]
    fn mcp_tool_name_returns_id_for_mcp_verbs() {
        let v = make("civ_a");
        assert_eq!(mcp_tool_name(&v), Some("civ_a"));
    }

    #[test]
    fn mcp_tool_name_none_when_unlinked() {
        let v = VerbDescriptor::new(
            "hud_a",
            "hud_a",
            "test",
            VerbGroup::Civic,
            crate::risk::RiskTier::ReadOnly,
            crate::provenance::Provenance::Hud,
            &[],
        );
        assert_eq!(mcp_tool_name(&v), None);
    }

    #[test]
    fn validate_accepts_snake_case() {
        let mut reg = VerbRegistry::empty();
        reg.register(make("civ_world_inspect"));
        reg.register(make("civ_law_propose_v2"));
        assert!(validate_mcp_links(&reg).is_empty());
    }

    #[test]
    fn validate_rejects_bad_names() {
        let mut reg = VerbRegistry::empty();
        reg.register(make("BadName"));
        reg.register(make("has space"));
        let issues = validate_mcp_links(&reg);
        assert_eq!(issues.len(), 2);
    }
}
