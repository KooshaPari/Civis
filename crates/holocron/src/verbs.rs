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

/// The full static catalog of MCP godverbs.
///
/// Adding a new verb to the MCP surface means adding a matching entry here.
/// Holocron does not auto-discover verbs at runtime in Phase 1 — it consumes
/// this static catalog and presents it through the panel + cmd-K.
///
/// Per `HOLOCRON_KEYCAP_UI.md` Phase 1: "substrate catalog, no runtime enumeration".
pub const MCP_VERBS: &[VerbDescriptor] = &[
    // ===== Civic =====
    VerbDescriptor {
        id: "civ_lay_tax",
        name: "Lay Tax",
        group: VerbGroup::Civic,
        aliases: &["tax", "set_tax"],
        hotkey: Some('T'),
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Reversible,
        description: "Set the tax rate for the city.",
    },
    VerbDescriptor {
        id: "civ_pardon_prisoner",
        name: "Pardon Prisoner",
        group: VerbGroup::Civic,
        aliases: &["pardon", "release_prisoner"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Reversible,
        description: "Release a named prisoner from the dungeons.",
    },
    VerbDescriptor {
        id: "civ_proclaim_law",
        name: "Proclaim Law",
        group: VerbGroup::Civic,
        aliases: &["law", "decree"],
        hotkey: Some('L'),
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Reversible,
        description: "Issue a new binding law across the city.",
    },
    VerbDescriptor {
        id: "civ_repeal_law",
        name: "Repeal Law",
        group: VerbGroup::Civic,
        aliases: &["repeal", "unlaw"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Reversible,
        description: "Strike an existing law from the books.",
    },
    VerbDescriptor {
        id: "civ_pardon_citizen",
        name: "Pardon Citizen",
        group: VerbGroup::Civic,
        aliases: &["pardon_citizen", "forgive"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Reversible,
        description: "Clear a citizen's criminal record.",
    },
    VerbDescriptor {
        id: "civ_inspect_law",
        name: "Inspect Law",
        group: VerbGroup::Civic,
        aliases: &["show_law", "law_detail"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Cosmetic,
        description: "Show the text and effects of a specific law.",
    },

    // ===== Economic =====
    VerbDescriptor {
        id: "civ_adjust_wages",
        name: "Adjust Wages",
        group: VerbGroup::Economic,
        aliases: &["wages", "pay_workers"],
        hotkey: Some('W'),
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Reversible,
        description: "Raise or lower the base wages paid to laborers.",
    },
    VerbDescriptor {
        id: "civ_grant_subsidy",
        name: "Grant Subsidy",
        group: VerbGroup::Economic,
        aliases: &["subsidy", "bailout"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Reversible,
        description: "Pay a one-time grant to a specific building or faction.",
    },
    VerbDescriptor {
        id: "civ_impose_tariff",
        name: "Impose Tariff",
        group: VerbGroup::Economic,
        aliases: &["tariff", "trade_tax"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Reversible,
        description: "Add a tariff on a specific trade good.",
    },
    VerbDescriptor {
        id: "civ_lift_tariff",
        name: "Lift Tariff",
        group: VerbGroup::Economic,
        aliases: &["lift_tariff"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Reversible,
        description: "Remove an existing tariff on a trade good.",
    },
    VerbDescriptor {
        id: "civ_inspect_market",
        name: "Inspect Market",
        group: VerbGroup::Economic,
        aliases: &["market", "show_market"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Cosmetic,
        description: "Show current prices and supply for each trade good.",
    },

    // ===== Divine =====
    VerbDescriptor {
        id: "civ_disaster_banish",
        name: "Banish Disaster",
        group: VerbGroup::Divine,
        aliases: &["calm", "stop_disaster", "banish"],
        hotkey: Some('B'),
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Reversible,
        description: "Ends the current disaster immediately.",
    },
    VerbDescriptor {
        id: "civ_bless_citizens",
        name: "Bless Citizens",
        group: VerbGroup::Divine,
        aliases: &["bless", "fervor"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Reversible,
        description: "Increase morale and religious fervor across the city.",
    },
    VerbDescriptor {
        id: "civ_smite_unfaithful",
        name: "Smite Unfaithful",
        group: VerbGroup::Divine,
        aliases: &["smite", "lightning"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Irreversible,
        description: "Strikes down a citizen of low faith. Permanent.",
    },
    VerbDescriptor {
        id: "civ_inspect_disaster",
        name: "Inspect Disaster",
        group: VerbGroup::Divine,
        aliases: &["disaster_detail", "show_disaster"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Cosmetic,
        description: "Show the cause, severity, and projected end of the current disaster.",
    },

    // ===== Debug =====
    VerbDescriptor {
        id: "civ_save_snapshot",
        name: "Save Snapshot",
        group: VerbGroup::Debug,
        aliases: &["save", "checkpoint"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Cosmetic,
        description: "Save the current sim state to disk.",
    },
    VerbDescriptor {
        id: "civ_load_snapshot",
        name: "Load Snapshot",
        group: VerbGroup::Debug,
        aliases: &["load", "restore"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Cosmetic,
        description: "Restore a previously saved sim state.",
    },
    VerbDescriptor {
        id: "civ_inspect_sim",
        name: "Inspect Sim",
        group: VerbGroup::Debug,
        aliases: &["sim_status", "show_sim"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Cosmetic,
        description: "Show tick, citizen count, and emergent event counts.",
    },
    VerbDescriptor {
        id: "civ_world_tick",
        name: "World Tick",
        group: VerbGroup::Debug,
        aliases: &["tick", "advance_tick"],
        hotkey: None,
        provenance: Provenance::Mcp,
        risk_tier: RiskTier::Cosmetic,
        description: "Advance the world by one tick.",
    },
];

/// Number of MCP verbs in the static catalog.
///
/// Used by tests and by the HolocronPanel UI to display "N verbs available".
pub const MCP_VERB_COUNT: usize = MCP_VERBS.len();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_unique_ids() {
        let mut seen = std::collections::HashSet::new();
        for v in MCP_VERBS {
            assert!(seen.insert(v.id), "duplicate verb id: {}", v.id);
        }
    }

    #[test]
    fn catalog_ids_start_with_civ() {
        for v in MCP_VERBS {
            assert!(
                v.id.starts_with("civ_"),
                "verb id {} does not start with civ_",
                v.id
            );
        }
    }

    #[test]
    fn catalog_has_no_empty_names() {
        for v in MCP_VERBS {
            assert!(!v.name.trim().is_empty(), "verb {} has empty name", v.id);
        }
    }

    #[test]
    fn catalog_has_no_empty_descriptions() {
        for v in MCP_VERBS {
            assert!(
                !v.description.trim().is_empty(),
                "verb {} has empty description",
                v.id
            );
        }
    }

    #[test]
    fn count_matches_len() {
        assert_eq!(MCP_VERB_COUNT, MCP_VERBS.len());
    }

    #[test]
    fn covers_all_groups() {
        let civic = MCP_VERBS.iter().filter(|v| v.group == VerbGroup::Civic).count();
        let economic = MCP_VERBS.iter().filter(|v| v.group == VerbGroup::Economic).count();
        let divine = MCP_VERBS.iter().filter(|v| v.group == VerbGroup::Divine).count();
        let debug = MCP_VERBS.iter().filter(|v| v.group == VerbGroup::Debug).count();
        assert!(civic >= 3, "need at least 3 civic verbs, got {}", civic);
        assert!(economic >= 3, "need at least 3 economic verbs, got {}", economic);
        assert!(divine >= 3, "need at least 3 divine verbs, got {}", divine);
        assert!(debug >= 3, "need at least 3 debug verbs, got {}", debug);
    }
}