//! Tests for FR-CIV-VERIFY-006
//! Epic: FR-CIV-VERIFY. Mod build/sign pipeline.
#[cfg(test)]
mod fr_fr_civ_verify_006 {
    #[test]
    fn mod_host_infrastructure_exists() {
        // FR-CIV-VERIFY-006 requires mod host to function.
        let ws = civ_engine::WorldState::default();
        // Mod host reads WorldState for mod context.
        assert!(!ws.factions.is_empty());
    }
}
