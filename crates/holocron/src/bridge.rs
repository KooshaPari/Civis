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
    verb.mcp_tool.as_deref()
}

/// Sanity check: verify that every verb with a `mcp_tool` link has
/// a well-formed tool name (lowercase, snake_case, no spaces).
///
/// Returns a list of (verb_id, reason) for any verbs that fail.
pub fn validate_mcp_links(registry: &VerbRegistry) -> Vec<(String, String)> {
    let mut issues = Vec::new();
    for (id, desc) in registry.iter() {
        if let Some(tool) = desc.mcp_tool.as_deref() {
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
    use crate::provenance::Provenance;
    use crate::registry::VerbRegistry;

    fn make(id: &str, tool: Option<&str>) -> VerbDescriptor {
        let mut b = VerbDescriptor::builder(id, id, VerbGroup::Civic)
            .description("test")
            .provenance(Provenance::Mcp);
        if let Some(t) = tool {
            b = b.mcp_tool(t);
        }
        b.build()
    }

    #[test]
    fn mcp_tool_name_returns_link() {
        let v = make("a", Some("civ_a"));
        assert_eq!(mcp_tool_name(&v), Some("civ_a"));
    }

    #[test]
    fn mcp_tool_name_none_when_unlinked() {
        let v = make("a", None);
        assert_eq!(mcp_tool_name(&v), None);
    }

    #[test]
    fn validate_accepts_snake_case() {
        let mut reg = VerbRegistry::new();
        reg.register(make("a", Some("civ_world_inspect"))).unwrap();
        reg.register(make("b", Some("civ_law_propose_v2"))).unwrap();
        assert!(validate_mcp_links(&reg).is_empty());
    }

    #[test]
    fn validate_rejects_bad_names() {
        let mut reg = VerbRegistry::new();
        reg.register(make("a", Some("BadName"))).unwrap();
        reg.register(make("b", Some("has space"))).unwrap();
        let issues = validate_mcp_links(&reg);
        assert_eq!(issues.len(), 2);
    }
}