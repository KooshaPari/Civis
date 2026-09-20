//! Tests for FR-CIV-ROAD-902 - Infrastructure Provenance
//!
//! Epic: FR-CIV-FRAME
//! Infrastructure SHALL track provenance (emergent vs user-placed).

#[cfg(test)]
mod fr_fr_civ_road_902 {
    /// FR-CIV-ROAD-902: InfraProvenance variants are distinct.
    #[test]
    fn provenance_variants_distinct() {
        use civ_traffic::InfraProvenance;
        assert_ne!(InfraProvenance::Emergent, InfraProvenance::UserPlaced);
    }

    /// FR-CIV-ROAD-902: Schema version is set.
    #[test]
    fn schema_version_is_set() {
        assert!(!civ_traffic::SCHEMA_VERSION.is_empty());
    }
}