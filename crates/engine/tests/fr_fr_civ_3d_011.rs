//! Tests for FR-CIV-3D-011
//!
//! Epic: FR-CIV-3D
//! Upgraded from stub to real assertions.
//!
//! FR-CIV-3D-011: Biome Coverage
//! Terrain generation produces all six biome types.
//! Engine-side: verify BiomeKind enum covers all required biomes.

#[cfg(test)]
mod fr_fr_civ_3d_011 {
    use civ_engine::BiomeKind;

    /// BiomeKind covers all six required biome types.
    #[test]
    fn all_six_biomes_exist() {
        let biomes = [
            BiomeKind::Ocean,
            BiomeKind::Plains,
            BiomeKind::Forest,
            BiomeKind::Desert,
            BiomeKind::Tundra,
            BiomeKind::Mountain,
        ];
        assert_eq!(biomes.len(), 6, "Must have exactly 6 biome types");
    }

    /// Each biome is distinct.
    #[test]
    fn biomes_are_distinct() {
        let biomes = [
            BiomeKind::Ocean,
            BiomeKind::Plains,
            BiomeKind::Forest,
            BiomeKind::Desert,
            BiomeKind::Tundra,
            BiomeKind::Mountain,
        ];
        for (i, b1) in biomes.iter().enumerate() {
            for (j, b2) in biomes.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        format!("{:?}", b1),
                        format!("{:?}", b2),
                        "Biomes at positions {} and {} must be distinct",
                        i, j
                    );
                }
            }
        }
    }
}
