//! Tests for FR-CIV-LANG-008 — Language and naming (isolation pressure)
//!
//! Epic: FR-CIV-LANG
//! Verifies faction isolation pressure computation based on settlement contacts.

#[cfg(test)]
mod fr_fr_civ_lang_008 {
    use std::collections::{BTreeMap, BTreeSet};

    /// FR-CIV-LANG-008: Faction with no foreign contacts has maximal isolation.
    #[test]
    fn no_foreign_contacts_is_maximal_isolation() {
        let dominant = BTreeMap::from([(10, 1), (20, 2)]);
        let members = BTreeMap::from([(10, 12), (20, 12)]);
        let isolation = civ_engine::language::faction_isolation_pressure(
            1,
            &dominant,
            &members,
            &BTreeSet::new(),
        );
        assert_eq!(isolation, 1.0);
    }

    /// FR-CIV-LANG-008: Cross-faction contact reduces isolation.
    #[test]
    fn cross_faction_contact_reduces_isolation() {
        let dominant = BTreeMap::from([(10, 1), (20, 2)]);
        let members = BTreeMap::from([(10, 12), (20, 3)]);
        let contacts = BTreeSet::from([(10, 20)]);
        let isolated =
            civ_engine::language::faction_isolation_pressure(1, &dominant, &members, &BTreeSet::new());
        let connected =
            civ_engine::language::faction_isolation_pressure(1, &dominant, &members, &contacts);
        assert!(connected < isolated);
    }

    /// FR-CIV-LANG-008: Same-faction contact does not reduce isolation.
    #[test]
    fn same_faction_contact_ignored() {
        let dominant = BTreeMap::from([(10, 1), (11, 1), (20, 2)]);
        let members = BTreeMap::from([(10, 5), (11, 5), (20, 0)]);
        let contacts = BTreeSet::from([(10, 11), (10, 20)]);
        let isolation = civ_engine::language::faction_isolation_pressure(
            1,
            &dominant,
            &members,
            &contacts,
        );
        assert_eq!(isolation, 1.0);
    }

    /// FR-CIV-LANG-008: Faction with zero population has zero isolation.
    #[test]
    fn zero_population_returns_zero() {
        let dominant = BTreeMap::from([(10, 1)]);
        let members = BTreeMap::from([(10, 0)]);
        let isolation = civ_engine::language::faction_isolation_pressure(
            1,
            &dominant,
            &members,
            &BTreeSet::new(),
        );
        assert_eq!(isolation, 0.0);
    }
}
