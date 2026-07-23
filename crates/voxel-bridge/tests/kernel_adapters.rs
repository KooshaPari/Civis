use civ_voxel::{ChunkCoord, WorldGen};
use civ_voxel_bridge::material::STONE;
use civ_voxel_bridge::worldgen::HeightFieldGen;

#[test]
fn worldgen_and_material_adapters_use_kernel_contracts() {
    let generator = HeightFieldGen {
        seed: 7,
        base_voxel_m: 4.0,
        sea_level_m: 0.0,
    };
    let chunk = generator.generate(ChunkCoord {
        cx: 0,
        cy: 0,
        cz: 0,
    });

    assert_eq!(STONE, civ_voxel::STONE);
    assert_eq!(chunk.voxels.len(), 16 * 16 * 16);
}
