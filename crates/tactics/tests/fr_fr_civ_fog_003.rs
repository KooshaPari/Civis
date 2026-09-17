//! Tests for FR-CIV-FOG-003
//!
//! Epic: FR-CIV-FOG
//! Status: SPEC-ONLY
//!
//! FR-CIV-FOG-003: FogOfWar maintains per-faction independent visibility.

use civ_tactics::FogOfWar;
use civ_voxel::{MaterialId, VoxelWorld};

#[cfg(test)]
mod fr_fr_civ_fog_003 {
    use super::*;

    /// FR-CIV-FOG-003: After update with no units, all factions see nothing.
    #[test]
    fn verify_fr_civ_fog_003_basic() {
        let mut fog = FogOfWar::new(16, None);
        let world = VoxelWorld::<MaterialId>::new(16);
        fog.update(&[], &world);
        assert!(!fog.is_visible(1, (0, 0)));
        assert!(!fog.is_visible(2, (0, 0)));
    }
}
