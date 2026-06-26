//! `VerbDescriptor` — the canonical, substrate-faithful description of a godverb.
//!
//! Every godverb that the MCP / JSON-RPC / egui layers expose is reflected
//! here as a static `VerbDescriptor`. Holocron consumes the descriptor list
//! to power the panel, the Command-K launcher, and the context-aware
//! ranking. No runtime introspection of the MCP server is required.

use crate::{group::VerbGroup, provenance::Provenance, risk::RiskTier};
use serde::{Deserialize, Serialize};

/// Static metadata describing a single godverb.
///
/// VerbDescriptors are constructed at startup and stored in the
/// [`VerbRegistry`](crate::registry::VerbRegistry). They are immutable for
/// the lifetime of the process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
}

impl VerbDescriptor {
    /// Construct a new descriptor. All fields are required.
    pub const fn new(
        id: &'static str,
        name: &'static str,
        summary: &'static str,
        group: VerbGroup,
        risk: RiskTier,
        provenance: Provenance,
        aliases: &'static [&'static str],
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            summary: summary.to_string(),
            group,
            risk,
            provenance,
            aliases,
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
