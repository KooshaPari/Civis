//! Tests for FR-CIV-WAR-011
//!
//! Epic: FR-CIV-WAR
//! Status: CODE-ONLY-no-spec
//!
//! FR-CIV-WAR-011: Maneuver — operational movement driven by objectives + supply.

use civ_tactics::OperationalMovementConfig;

#[cfg(test)]
mod fr_fr_civ_war_011 {
    use super::*;

    /// FR-CIV-WAR-011: Operational movement config defaults are sane.
    #[test]
    fn verify_fr_civ_war_011_basic() {
        let config = OperationalMovementConfig::default();
        assert!(config.cadence_ticks > 0, "cadence must be positive");
    }

    /// FR-CIV-WAR-011: Path search radius is positive.
    #[test]
    fn movement_config_search_radius_positive() {
        let config = OperationalMovementConfig::default();
        assert!(config.path_search_radius > 0, "search radius must be positive");
    }
}
