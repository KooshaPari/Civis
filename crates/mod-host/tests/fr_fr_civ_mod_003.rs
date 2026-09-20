//! Tests for FR-CIV-MOD-003 — Capability enforcement
//!
//! Epic: FR-CIV-MOD
//! Verifies capability set and world domain access control.

#[cfg(test)]
mod fr_fr_civ_mod_003 {
    /// FR-CIV-MOD-003: WorldDomain variants exist.
    #[test]
    fn world_domain_variants() {
        use civ_mod_host::WorldDomain;
        // Must compile - variants exist
        let _ = WorldDomain::Economy;
        let _ = WorldDomain::Military;
        let _ = WorldDomain::Diplomacy;
        let _ = WorldDomain::Citizens;
    }

    /// FR-CIV-MOD-003: CapabilitySet from empty permissions denies all domains.
    #[test]
    fn empty_perms_no_access() {
        use civ_mod_host::{CapabilitySet, WorldDomain, ModPermissions};
        let perms = ModPermissions::default();
        let cap = CapabilitySet::from_permissions(&perms);
        assert!(!cap.can_read_domain(WorldDomain::Economy));
        assert!(!cap.can_read_domain(WorldDomain::Military));
    }

    /// FR-CIV-MOD-003: allow_all grants all domains.
    #[test]
    fn allow_all_grants_all() {
        use civ_mod_host::{CapabilitySet, WorldDomain};
        let cap = CapabilitySet::allow_all();
        assert!(cap.can_read_domain(WorldDomain::Economy));
        assert!(cap.can_read_domain(WorldDomain::Climate));
        assert!(cap.can_emit_action(999));
    }
}
