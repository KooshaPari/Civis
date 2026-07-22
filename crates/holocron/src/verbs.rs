//! Static MCP verb catalog for Holocron.
//!
//! This module declares the canonical list of godverbs Holocron knows about,
//! derived from the `civ_*` MCP verbs exposed by `crates/civis-mcp/src/server.rs`.
//!
//! **Source of truth:** the swarm ships the MCP verbs; this catalog mirrors
//! them under a stable shape (`VerbDescriptor`). When a new MCP verb is
//! added, append a corresponding `VerbDescriptor` entry here.

use crate::descriptor::VerbDescriptor;
use crate::group::VerbGroup;
use crate::provenance::Provenance;
use crate::risk::RiskTier;

/// Build the full static catalog of MCP godverbs.
///
/// Adding a new verb to the MCP surface means adding a matching entry here.
/// Holocron does not auto-discover verbs at runtime in Phase 1 — it consumes
/// this static catalog and presents it through the panel + cmd-K.
///
/// Per `HOLOCRON_KEYCAP_UI.md` Phase 1: "substrate catalog, no runtime enumeration".
pub fn build_mcp_catalog() -> Vec<VerbDescriptor> {
    vec![
        // ===== Civic =====
        VerbDescriptor {
            id: "civ_lay_tax".to_string(),
            name: "Lay Tax".to_string(),
            summary: "Set the tax rate for the city".to_string(),
            group: VerbGroup::Civic,
            aliases: &["tax", "set_tax"],
            hotkey: Some('T'),
            provenance: Provenance::Mcp,
            risk: RiskTier::Reversible,
            description: "Set the tax rate for the city.".to_string(),
            mcp_tool: Some("civ_lay_tax".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_pardon_prisoner".to_string(),
            name: "Pardon Prisoner".to_string(),
            summary: "Release a named prisoner".to_string(),
            group: VerbGroup::Civic,
            aliases: &["pardon", "release_prisoner"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Reversible,
            description: "Release a named prisoner from the dungeons.".to_string(),
            mcp_tool: Some("civ_pardon_prisoner".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_proclaim_law".to_string(),
            name: "Proclaim Law".to_string(),
            summary: "Issue a new binding law".to_string(),
            group: VerbGroup::Civic,
            aliases: &["law", "decree"],
            hotkey: Some('L'),
            provenance: Provenance::Mcp,
            risk: RiskTier::Reversible,
            description: "Issue a new binding law across the city.".to_string(),
            mcp_tool: Some("civ_proclaim_law".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_repeal_law".to_string(),
            name: "Repeal Law".to_string(),
            summary: "Strike an existing law".to_string(),
            group: VerbGroup::Civic,
            aliases: &["repeal", "unlaw"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Reversible,
            description: "Strike an existing law from the books.".to_string(),
            mcp_tool: Some("civ_repeal_law".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_pardon_citizen".to_string(),
            name: "Pardon Citizen".to_string(),
            summary: "Clear a citizen's criminal record".to_string(),
            group: VerbGroup::Civic,
            aliases: &["pardon_citizen", "forgive"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Reversible,
            description: "Clear a citizen's criminal record.".to_string(),
            mcp_tool: Some("civ_pardon_citizen".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_inspect_law".to_string(),
            name: "Inspect Law".to_string(),
            summary: "Show law details".to_string(),
            group: VerbGroup::Civic,
            aliases: &["show_law", "law_detail"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Cosmetic,
            description: "Show the text and effects of a specific law.".to_string(),
            mcp_tool: Some("civ_inspect_law".to_string()),
            use_count: 0,
        },

        // ===== Economic =====
        VerbDescriptor {
            id: "civ_adjust_wages".to_string(),
            name: "Adjust Wages".to_string(),
            summary: "Adjust worker wages".to_string(),
            group: VerbGroup::Economic,
            aliases: &["wages", "pay_workers"],
            hotkey: Some('W'),
            provenance: Provenance::Mcp,
            risk: RiskTier::Reversible,
            description: "Raise or lower the base wages paid to laborers.".to_string(),
            mcp_tool: Some("civ_adjust_wages".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_grant_subsidy".to_string(),
            name: "Grant Subsidy".to_string(),
            summary: "Grant a subsidy to a building".to_string(),
            group: VerbGroup::Economic,
            aliases: &["subsidy", "bailout"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Reversible,
            description: "Pay a one-time grant to a specific building or faction.".to_string(),
            mcp_tool: Some("civ_grant_subsidy".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_impose_tariff".to_string(),
            name: "Impose Tariff".to_string(),
            summary: "Add a trade tariff".to_string(),
            group: VerbGroup::Economic,
            aliases: &["tariff", "trade_tax"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Reversible,
            description: "Add a tariff on a specific trade good.".to_string(),
            mcp_tool: Some("civ_impose_tariff".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_lift_tariff".to_string(),
            name: "Lift Tariff".to_string(),
            summary: "Remove a trade tariff".to_string(),
            group: VerbGroup::Economic,
            aliases: &["lift_tariff"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Reversible,
            description: "Remove an existing tariff on a trade good.".to_string(),
            mcp_tool: Some("civ_lift_tariff".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_inspect_market".to_string(),
            name: "Inspect Market".to_string(),
            summary: "View market data".to_string(),
            group: VerbGroup::Economic,
            aliases: &["market", "show_market"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Cosmetic,
            description: "Show current prices and supply for each trade good.".to_string(),
            mcp_tool: Some("civ_inspect_market".to_string()),
            use_count: 0,
        },

        // ===== Divine =====
        VerbDescriptor {
            id: "civ_disaster_banish".to_string(),
            name: "Banish Disaster".to_string(),
            summary: "End the current disaster".to_string(),
            group: VerbGroup::Divine,
            aliases: &["calm", "stop_disaster", "banish"],
            hotkey: Some('B'),
            provenance: Provenance::Mcp,
            risk: RiskTier::Reversible,
            description: "Ends the current disaster immediately.".to_string(),
            mcp_tool: Some("civ_disaster_banish".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_bless_citizens".to_string(),
            name: "Bless Citizens".to_string(),
            summary: "Boost morale and faith".to_string(),
            group: VerbGroup::Divine,
            aliases: &["bless", "fervor"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Reversible,
            description: "Increase morale and religious fervor across the city.".to_string(),
            mcp_tool: Some("civ_bless_citizens".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_smite_unfaithful".to_string(),
            name: "Smite Unfaithful".to_string(),
            summary: "Strike down unfaithful citizens".to_string(),
            group: VerbGroup::Divine,
            aliases: &["smite", "lightning"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Irreversible,
            description: "Strikes down a citizen of low faith. Permanent.".to_string(),
            mcp_tool: Some("civ_smite_unfaithful".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_inspect_disaster".to_string(),
            name: "Inspect Disaster".to_string(),
            summary: "View disaster details".to_string(),
            group: VerbGroup::Divine,
            aliases: &["disaster_detail", "show_disaster"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Cosmetic,
            description: "Show the cause, severity, and projected end of the current disaster.".to_string(),
            mcp_tool: Some("civ_inspect_disaster".to_string()),
            use_count: 0,
        },

        // ===== Debug =====
        VerbDescriptor {
            id: "civ_save_snapshot".to_string(),
            name: "Save Snapshot".to_string(),
            summary: "Save world state".to_string(),
            group: VerbGroup::Debug,
            aliases: &["save", "checkpoint"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Cosmetic,
            description: "Save the current sim state to disk.".to_string(),
            mcp_tool: Some("civ_save_snapshot".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_load_snapshot".to_string(),
            name: "Load Snapshot".to_string(),
            summary: "Restore world state".to_string(),
            group: VerbGroup::Debug,
            aliases: &["load", "restore"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Cosmetic,
            description: "Restore a previously saved sim state.".to_string(),
            mcp_tool: Some("civ_load_snapshot".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_inspect_sim".to_string(),
            name: "Inspect Sim".to_string(),
            summary: "View sim status".to_string(),
            group: VerbGroup::Debug,
            aliases: &["sim_status", "show_sim"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Cosmetic,
            description: "Show tick, citizen count, and emergent event counts.".to_string(),
            mcp_tool: Some("civ_inspect_sim".to_string()),
            use_count: 0,
        },
        VerbDescriptor {
            id: "civ_world_tick".to_string(),
            name: "World Tick".to_string(),
            summary: "Advance world time".to_string(),
            group: VerbGroup::Debug,
            aliases: &["tick", "advance_tick"],
            hotkey: None,
            provenance: Provenance::Mcp,
            risk: RiskTier::Cosmetic,
            description: "Advance the world by one tick.".to_string(),
            mcp_tool: Some("civ_world_tick".to_string()),
            use_count: 0,
        },
    ]
}

/// The full static catalog of MCP godverbs (lazy-initialized).
pub const STATIC_CATALOG: &str = "Use build_mcp_catalog() instead";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_unique_ids() {
        let verbs = build_mcp_catalog();
        let mut seen = std::collections::HashSet::new();
        for v in verbs {
            assert!(seen.insert(v.id.clone()), "duplicate verb id: {}", v.id);
        }
    }

    #[test]
    fn catalog_ids_start_with_civ() {
        let verbs = build_mcp_catalog();
        for v in verbs {
            assert!(
                v.id.starts_with("civ_"),
                "verb id {} does not start with civ_",
                v.id
            );
        }
    }

    #[test]
    fn catalog_has_no_empty_names() {
        let verbs = build_mcp_catalog();
        for v in verbs {
            assert!(!v.name.trim().is_empty(), "verb {} has empty name", v.id);
        }
    }

    #[test]
    fn catalog_has_no_empty_descriptions() {
        let verbs = build_mcp_catalog();
        for v in verbs {
            assert!(
                !v.description.trim().is_empty(),
                "verb {} has empty description",
                v.id
            );
        }
    }

    #[test]
    fn covers_all_groups() {
        let verbs = build_mcp_catalog();
        let civic = verbs.iter().filter(|v| v.group == VerbGroup::Civic).count();
        let economic = verbs.iter().filter(|v| v.group == VerbGroup::Economic).count();
        let divine = verbs.iter().filter(|v| v.group == VerbGroup::Divine).count();
        let debug = verbs.iter().filter(|v| v.group == VerbGroup::Debug).count();
        assert!(civic >= 3, "need at least 3 civic verbs, got {}", civic);
        assert!(economic >= 3, "need at least 3 economic verbs, got {}", economic);
        assert!(divine >= 3, "need at least 3 divine verbs, got {}", divine);
        assert!(debug >= 3, "need at least 3 debug verbs, got {}", debug);
    }
}
