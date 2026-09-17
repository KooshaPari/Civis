//! Tests for FR-CIV-WAR-013
//!
//! Epic: FR-CIV-WAR
//! Status: CODE-ONLY-no-spec
//!
//! FR-CIV-WAR-013: Bridge to tactical — operational layer hands off to war bridge.

use civ_tactics::{CombatEngagement, DamageEvent};
use civ_voxel::WorldCoord;

#[cfg(test)]
mod fr_fr_civ_war_013 {
    use super::*;

    /// FR-CIV-WAR-013: Combat engagement struct is constructible with valid fields.
    #[test]
    fn verify_fr_civ_war_013_basic() {
        let engagement = CombatEngagement {
            shooter_id: 100,
            target_id: 200,
            shooter_faction: 1,
            target_faction: 2,
            damage: DamageEvent {
                center: WorldCoord { x: 10, y: 20, z: 5 },
                radius_voxels: 3,
                energy: 200,
            },
            target_index: 0,
        };
        assert_eq!(engagement.shooter_faction, 1);
        assert_eq!(engagement.target_faction, 2);
        assert!(engagement.damage.estimated_casualties() > 0);
    }
}
