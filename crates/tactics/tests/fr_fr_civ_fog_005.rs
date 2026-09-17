//! Tests for FR-CIV-FOG-005
//!
//! Epic: FR-CIV-FOG
//! Status: SPEC-ONLY
//!
//! FR-CIV-FOG-005: FogOfWar resets visibility when update() is called.

use civ_tactics::FogOfWar;
use civ_voxel::{MaterialId, VoxelWorld};

#[cfg(test)]
mod fr_fr_civ_fog_005 {
    use super::*;

    /// FR-CIV-FOG-005: Visibility is recomputed on update (not incremental).
    #[test]
    fn verify_fr_civ_fog_005_basic() {
        let mut fog = FogOfWar::new(32, None);
        let world = VoxelWorld::<MaterialId>::new(32);
        fog.update(&[], &world);
        assert!(!fog.is_visible(0, (0, 0)));
        fog.update(&[], &world);
        assert!(!fog.is_visible(0, (0, 0)));
    }
}
