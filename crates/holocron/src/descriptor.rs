//! `VerbDescriptor` — the canonical, substrate-faithful description of a godverb.
//!
//! Every godverb that the MCP / JSON-RPC / egui layers expose is reflected
//! here as a static `VerbDescriptor`. Holocron consumes the descriptor list
//! to power the panel, the Command-K launcher, and the context-aware
//! ranking. No runtime introspection of the MCP server is required.

use crate::{group::VerbGroup, provenance::Provenance, risk::RiskTier};
use serde::Serialize;

/// Static metadata describing a single godverb.
///
/// VerbDescriptors are constructed at startup and stored in the
/// [`VerbRegistry`](crate::registry::VerbRegistry). They are immutable for
/// the lifetime of the process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerbDescriptor {
    /// Stable, kebab-case identifier (e.g. `"spawn-citizen"`).
    pub id: String,
    /// Human-readable verb name shown in the panel / Command-K (e.g. `"Spawn Citizen"`).
    pub name: String,
    /// Single-line summary shown in the panel and as Command-K hint text.
    pub summary: String,
    /// Grouping for the panel and for context-aware ranking.
    pub group: VerbGroup,
    /// How risky the verb is (used by the panel to gate destructive actions).
    pub risk: RiskTier,
    /// Provenance — where the verb is reachable from.
    pub provenance: Provenance,
    /// Aliases for Command-K fuzzy search. Lower-case, no whitespace.
    pub aliases: &'static [&'static str],
    /// Longer description of what the verb does.
    pub description: String,
    /// Optional hotkey binding for the verb.
    pub hotkey: Option<char>,
    /// MCP tool name if this verb is backed by an MCP tool.
    pub mcp_tool: Option<String>,
    /// Number of times this verb has been used (for ranking).
    pub use_count: u64,
}

impl VerbDescriptor {
    /// Construct a new descriptor via builder pattern (recommended).
    pub fn builder(id: &str, name: &str, group: VerbGroup) -> VerbDescriptorBuilder {
        VerbDescriptorBuilder {
            id: id.to_string(),
            name: name.to_string(),
            summary: String::new(),
            group,
            risk: RiskTier::ReadOnly,
            provenance: Provenance::Mcp,
            aliases: &[],
            description: String::new(),
            hotkey: None,
            mcp_tool: None,
            use_count: 0,
        }
    }

    /// Tokenise this descriptor for fuzzy matching: lower-cased words
    /// derived from `name`, `id`, and `aliases`.
    pub fn search_tokens(&self) -> Vec<String> {
        let mut tokens = Vec::with_capacity(4 + self.aliases.len());
        tokens.push(self.name.to_lowercase());
        tokens.push(self.id.replace('-', " "));
        for alias in self.aliases {
            tokens.push((*alias).to_lowercase());
        }
        tokens
    }
}

/// Builder for constructing VerbDescriptor with a fluent API.
pub struct VerbDescriptorBuilder {
    id: String,
    name: String,
    summary: String,
    group: VerbGroup,
    risk: RiskTier,
    provenance: Provenance,
    aliases: &'static [&'static str],
    description: String,
    hotkey: Option<char>,
    mcp_tool: Option<String>,
    use_count: u64,
}

impl VerbDescriptorBuilder {
    /// Set the summary text.
    pub fn summary(mut self, summary: &str) -> Self {
        self.summary = summary.to_string();
        self
    }

    /// Set the risk tier.
    pub fn risk(mut self, risk: RiskTier) -> Self {
        self.risk = risk;
        self
    }

    /// Set the provenance.
    pub fn provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = provenance;
        self
    }

    /// Set the aliases.
    pub fn aliases(mut self, aliases: &'static [&'static str]) -> Self {
        self.aliases = aliases;
        self
    }

    /// Set the description.
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Set the optional hotkey.
    pub fn hotkey(mut self, hotkey: Option<char>) -> Self {
        self.hotkey = hotkey;
        self
    }

    /// Set the optional MCP tool name.
    pub fn mcp_tool(mut self, mcp_tool: &str) -> Self {
        self.mcp_tool = Some(mcp_tool.to_string());
        self
    }

    /// Set the use count.
    pub fn use_count(mut self, count: u64) -> Self {
        self.use_count = count;
        self
    }

    /// Build the final VerbDescriptor.
    pub fn build(self) -> VerbDescriptor {
        VerbDescriptor {
            id: self.id,
            name: self.name,
            summary: self.summary,
            group: self.group,
            risk: self.risk,
            provenance: self.provenance,
            aliases: self.aliases,
            description: self.description,
            hotkey: self.hotkey,
            mcp_tool: self.mcp_tool,
            use_count: self.use_count,
        }
    }
}
