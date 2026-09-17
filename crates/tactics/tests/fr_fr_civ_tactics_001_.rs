//! Tests for FR-CIV-TACTICS-001-
//!
//! Epic: FR-CIV-TACTICS
//! Status: SPEC-ONLY
//!
//! FR-CIV-TACTICS-001-: DamageEvent voxel-destructible combat basics.

use civ_tactics::DamageEvent;
use civ_voxel::WorldCoord;

#[cfg(test)]
mod fr_fr_civ_tactics_001_ {
    use super::*;

    /// FR-CIV-TACTICS-001-: DamageEvent with zero energy produces zero casualties.
    #[test]
    fn verify_fr_civ_tactics_001__basic() {
        let de = DamageEvent {
            center: WorldCoord { x: 0, y: 0, z: 0 },
            radius_voxels: 0,
            energy: 0,
        };
        assert_eq!(de.estimated_casualties(), 0);
    }

    /// FR-CIV-TACTICS-001-: Larger radius and energy yield more casualties.
    #[test]
    fn damage_event_casualties_monotonic_in_radius() {
        let small = DamageEvent {
            center: WorldCoord { x: 0, y: 0, z: 0 },
            radius_voxels: 1,
            energy: 100,
        };
        let large = DamageEvent {
            center: WorldCoord { x: 0, y: 0, z: 0 },
            radius_voxels: 3,
            energy: 100,
        };
        assert!(large.estimated_casualties() > small.estimated_casualties());
    }
}
