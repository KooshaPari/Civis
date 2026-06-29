//! Static MCP verb catalog for Holocron.

use crate::descriptor::VerbDescriptor;
use crate::group::VerbGroup;
use crate::provenance::Provenance;
use crate::risk::RiskTier;

/// The full static catalog of MCP godverbs.
pub static MCP_VERBS: std::sync::LazyLock<Vec<VerbDescriptor>> = std::sync::LazyLock::new(|| {
    vec![
        VerbDescriptor::new(
            "civ_lay_tax",
            "Lay Tax",
            "Set the tax rate for the city.",
            VerbGroup::Civic,
            RiskTier::Minor,
            Provenance::Mcp,
            &["tax", "set_tax"],
        ),
        VerbDescriptor::new(
            "civ_pardon_prisoner",
            "Pardon Prisoner",
            "Release a named prisoner from the dungeons.",
            VerbGroup::Civic,
            RiskTier::Minor,
            Provenance::Mcp,
            &["pardon", "release_prisoner"],
        ),
        VerbDescriptor::new(
            "civ_proclaim_law",
            "Proclaim Law",
            "Issue a new binding law across the city.",
            VerbGroup::Civic,
            RiskTier::Minor,
            Provenance::Mcp,
            &["law", "decree"],
        ),
        VerbDescriptor::new(
            "civ_repeal_law",
            "Repeal Law",
            "Strike an existing law from the books.",
            VerbGroup::Civic,
            RiskTier::Minor,
            Provenance::Mcp,
            &["repeal", "unlaw"],
        ),
        VerbDescriptor::new(
            "civ_pardon_citizen",
            "Pardon Citizen",
            "Clear a citizen's criminal record.",
            VerbGroup::Civic,
            RiskTier::Minor,
            Provenance::Mcp,
            &["pardon_citizen", "forgive"],
        ),
        VerbDescriptor::new(
            "civ_inspect_law",
            "Inspect Law",
            "Show the text and effects of a specific law.",
            VerbGroup::Civic,
            RiskTier::ReadOnly,
            Provenance::Mcp,
            &["show_law", "law_detail"],
        ),
        VerbDescriptor::new(
            "civ_adjust_wages",
            "Adjust Wages",
            "Raise or lower the base wages paid to laborers.",
            VerbGroup::Economic,
            RiskTier::Minor,
            Provenance::Mcp,
            &["wages", "pay_workers"],
        ),
        VerbDescriptor::new(
            "civ_grant_subsidy",
            "Grant Subsidy",
            "Pay a one-time grant to a specific building or faction.",
            VerbGroup::Economic,
            RiskTier::Minor,
            Provenance::Mcp,
            &["subsidy", "bailout"],
        ),
        VerbDescriptor::new(
            "civ_impose_tariff",
            "Impose Tariff",
            "Add a tariff on a specific trade good.",
            VerbGroup::Economic,
            RiskTier::Minor,
            Provenance::Mcp,
            &["tariff", "trade_tax"],
        ),
        VerbDescriptor::new(
            "civ_lift_tariff",
            "Lift Tariff",
            "Remove an existing tariff on a trade good.",
            VerbGroup::Economic,
            RiskTier::Minor,
            Provenance::Mcp,
            &["lift_tariff"],
        ),
        VerbDescriptor::new(
            "civ_inspect_market",
            "Inspect Market",
            "Show current prices and supply for each trade good.",
            VerbGroup::Economic,
            RiskTier::ReadOnly,
            Provenance::Mcp,
            &["market", "show_market"],
        ),
        VerbDescriptor::new(
            "civ_disaster_banish",
            "Banish Disaster",
            "Ends the current disaster immediately.",
            VerbGroup::Divine,
            RiskTier::Minor,
            Provenance::Mcp,
            &["calm", "stop_disaster", "banish"],
        ),
        VerbDescriptor::new(
            "civ_bless_citizens",
            "Bless Citizens",
            "Increase morale and religious fervor across the city.",
            VerbGroup::Divine,
            RiskTier::Minor,
            Provenance::Mcp,
            &["bless", "fervor"],
        ),
        VerbDescriptor::new(
            "civ_smite_unfaithful",
            "Smite Unfaithful",
            "Strikes down a citizen of low faith. Permanent.",
            VerbGroup::Divine,
            RiskTier::Critical,
            Provenance::Mcp,
            &["smite", "lightning"],
        ),
        VerbDescriptor::new(
            "civ_inspect_disaster",
            "Inspect Disaster",
            "Show the cause, severity, and projected end of the current disaster.",
            VerbGroup::Divine,
            RiskTier::ReadOnly,
            Provenance::Mcp,
            &["disaster_detail", "show_disaster"],
        ),
        VerbDescriptor::new(
            "civ_save_snapshot",
            "Save Snapshot",
            "Save the current sim state to disk.",
            VerbGroup::Debug,
            RiskTier::ReadOnly,
            Provenance::Mcp,
            &["save", "checkpoint"],
        ),
        VerbDescriptor::new(
            "civ_load_snapshot",
            "Load Snapshot",
            "Restore a previously saved sim state.",
            VerbGroup::Debug,
            RiskTier::ReadOnly,
            Provenance::Mcp,
            &["load", "restore"],
        ),
        VerbDescriptor::new(
            "civ_inspect_sim",
            "Inspect Sim",
            "Show tick, citizen count, and emergent event counts.",
            VerbGroup::Debug,
            RiskTier::ReadOnly,
            Provenance::Mcp,
            &["sim_status", "show_sim"],
        ),
        VerbDescriptor::new(
            "civ_world_tick",
            "World Tick",
            "Advance the world by one tick.",
            VerbGroup::Debug,
            RiskTier::ReadOnly,
            Provenance::Mcp,
            &["tick", "advance_tick"],
        ),
    ]
});

/// Number of MCP verbs in the static catalog.
pub fn mcp_verb_count() -> usize {
    MCP_VERBS.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_unique_ids() {
        let mut seen = std::collections::HashSet::new();
        for v in MCP_VERBS.iter() {
            assert!(seen.insert(v.id.as_str()), "duplicate verb id: {}", v.id);
        }
    }

    #[test]
    fn catalog_ids_start_with_civ() {
        for v in MCP_VERBS.iter() {
            assert!(
                v.id.starts_with("civ_"),
                "verb id {} does not start with civ_",
                v.id
            );
        }
    }

    #[test]
    fn catalog_has_no_empty_names() {
        for v in MCP_VERBS.iter() {
            assert!(!v.name.trim().is_empty(), "verb {} has empty name", v.id);
        }
    }

    #[test]
    fn catalog_has_no_empty_summaries() {
        for v in MCP_VERBS.iter() {
            assert!(
                !v.summary.trim().is_empty(),
                "verb {} has empty summary",
                v.id
            );
        }
    }

    #[test]
    fn count_matches_len() {
        assert_eq!(mcp_verb_count(), MCP_VERBS.len());
    }

    #[test]
    fn covers_all_groups() {
        let civic = MCP_VERBS
            .iter()
            .filter(|v| v.group == VerbGroup::Civic)
            .count();
        let economic = MCP_VERBS
            .iter()
            .filter(|v| v.group == VerbGroup::Economic)
            .count();
        let divine = MCP_VERBS
            .iter()
            .filter(|v| v.group == VerbGroup::Divine)
            .count();
        let debug = MCP_VERBS
            .iter()
            .filter(|v| v.group == VerbGroup::Debug)
            .count();
        assert!(civic >= 3, "need at least 3 civic verbs, got {}", civic);
        assert!(
            economic >= 3,
            "need at least 3 economic verbs, got {}",
            economic
        );
        assert!(divine >= 3, "need at least 3 divine verbs, got {}", divine);
        assert!(debug >= 3, "need at least 3 debug verbs, got {}", debug);
    }
}
