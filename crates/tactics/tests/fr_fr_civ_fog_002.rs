//! Tests for FR-CIV-FOG-002
//!
//! Epic: FR-CIV-FOG
//! Status: SPEC-ONLY
//!
//! FR-CIV-FOG-002: FogOfWar visibility can be queried per faction.

use civ_tactics::FogOfWar;

#[cfg(test)]
mod fr_fr_civ_fog_002 {
    use super::*;

    /// FR-CIV-FOG-002: New fog has no visible cells for any faction.
    #[test]
    fn verify_fr_civ_fog_002_basic() {
        let fog = FogOfWar::new(16, None);
        assert!(!fog.is_visible(0, (0, 0)));
    }

    /// FR-CIV-FOG-002: Visibility query for unknown faction returns false.
    #[test]
    fn fog_visibility_unknown_faction_returns_false() {
        let fog = FogOfWar::new(16, None);
        assert!(!fog.is_visible(999, (5, 5)));
    }
}
