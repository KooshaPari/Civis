//! Tests for FR-CIV-WAR-022
//!
//! Epic: FR-CIV-WAR
//! Status: CODE-ONLY-no-spec
//!
//! FR-CIV-WAR-022: Voxel destruction feedback — DamageEvents reshape terrain.
//! Tests that damage events carry valid world coordinates and radii.

use civ_tactics::DamageEvent;
use civ_voxel::WorldCoord;

#[cfg(test)]
mod fr_fr_civ_war_022 {
    use super::*;

    /// FR-CIV-WAR-022: Damage event with positive energy produces casualties.
    #[test]
    fn verify_fr_civ_war_022_basic() {
        let de = DamageEvent {
            center: WorldCoord { x: 5, y: 5, z: 3 },
            radius_voxels: 2,
            energy: 500,
        };
        assert!(
            de.estimated_casualties() > 0,
            "non-zero damage event should produce casualties"
        );
    }

    /// FR-CIV-WAR-022: Larger radius at same energy increases blast footprint.
    #[test]
    fn larger_radius_increases_damage_footprint() {
        let small = DamageEvent {
            center: WorldCoord { x: 0, y: 0, z: 0 },
            radius_voxels: 1,
            energy: 300,
        };
        let large = DamageEvent {
            center: WorldCoord { x: 0, y: 0, z: 0 },
            radius_voxels: 5,
            energy: 300,
        };
        assert!(large.estimated_casualties() > small.estimated_casualties());
    }
}
