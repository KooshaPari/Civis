use civ_voxel_bridge::hud::ToolPalette;
use civ_voxel_bridge::lod::{drain_dirty_chunks, ChunkDirty};
use civ_voxel_bridge::scale_budget::MvpResidentConfig;

#[test]
fn presentation_adapters_use_kernel_contracts() {
    let palette = ToolPalette::new();
    assert_eq!(palette.schema_version, "0.1.0-hub-palette");

    let mut dirty = vec![ChunkDirty {
        chunk_pos: (1, 2, 3),
        is_active: true,
    }];
    assert_eq!(drain_dirty_chunks(&mut dirty).len(), 1);
    assert!(dirty.is_empty());

    let config = MvpResidentConfig::MVP;
    assert!(config.ca_chunk_voxels > 0);
}
