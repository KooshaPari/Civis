use civ_voxel::{
    material::{AIR, WATER},
    worldgen,
    worldgen::GenWorld,
    ChunkId, CubicMesher, ChunkView, LodLevel, MaterialId,
};

fn linear_index(dims: [usize; 3], x: usize, y: usize, z: usize) -> usize {
    x + y * dims[0] + z * dims[0] * dims[1]
}

fn surface_y(world: &GenWorld, x: usize, z: usize) -> Option<usize> {
    if x >= world.dims[0] || z >= world.dims[2] {
        return None;
    }
    for y in (0..world.dims[1]).rev() {
        let mat = world.cells[linear_index(world.dims, x, y, z)];
        if mat != AIR {
            return Some(y);
        }
    }
    None
}

fn sample_chunk(world: &GenWorld, origin: [usize; 3], chunk_edge: usize) -> Vec<MaterialId> {
    let mut chunk = vec![AIR; chunk_edge * chunk_edge * chunk_edge];
    let end_x = (origin[0] + chunk_edge).min(world.dims[0]);
    let end_y = (origin[1] + chunk_edge).min(world.dims[1]);
    let end_z = (origin[2] + chunk_edge).min(world.dims[2]);
    for x in origin[0]..end_x {
        for y in origin[1]..end_y {
            for z in origin[2]..end_z {
                let cx = x - origin[0];
                let cy = y - origin[1];
                let cz = z - origin[2];
                let chunk_idx = cx + cy * chunk_edge + cz * chunk_edge * chunk_edge;
                chunk[chunk_idx] = world.cells[linear_index(world.dims, x, y, z)];
            }
        }
    }
    chunk
}

#[test]
#[ignore = "Missing public API: world size mapping is currently private (civ_voxel::worldgen::world_dims_for). Export mapping and replace placeholder assertions."]
fn requirement_world_size_selection_changes_dimensions() {
    // GIVEN size-indexed selection values from UI world-size controls (small..large),
    // WHEN world_dims_for(index) is invoked for each index,
    // THEN generated dimensions must strictly increase with each index.
    //
    // This is currently blocked until `world_dims_for` is public (or replaced by a
    // dedicated public world-size mapping API).
    let expected_increasing: bool = true;
    assert!(expected_increasing, "placeholder: world_dims_for(index) should be public to validate this requirement");
}

#[test]
fn requirement_new_world_differs_from_previous() {
    // GIVEN two different New World seeds,
    // WHEN worldgen::generate is called with same user size and different seeds,
    // THEN at least 20% of sampled surface columns should differ.
    let dims = [64, 48, 64];
    let seed_a = 0x1E_5F_3A_C2_9D_17_4B_81u64;
    let seed_b = seed_a.wrapping_mul(13_245_799_145_678_972_871).rotate_left(13);
    let world_a = worldgen::generate(dims, seed_a);
    let world_b = worldgen::generate(dims, seed_b);

    let mut diffs = 0usize;
    for x in 0..dims[0] {
        for z in 0..dims[2] {
            if surface_y(&world_a, x, z) != surface_y(&world_b, x, z) {
                diffs += 1;
            }
        }
    }
    let total = dims[0] * dims[2];
    let ratio = diffs as f32 / total as f32;
    assert!(ratio > 0.20, "surface change ratio {ratio:.2} is below requirement (expected > 0.20)");
}

#[test]
#[ignore = "Missing public API: no pub basemap/world-extent mapper yet (map2d::build_basemap_image is private, and basemap extents are not exposed)."]
fn requirement_2d_map_extent_matches_world() {
    // GIVEN a world size D from UI/worldgen wiring,
    // WHEN basemap sampling is executed,
    // THEN 2D map extents should cover [0..D.x] and [0..D.z].
    // Once `map2d::build_basemap_image` becomes pub (or equivalent API is added),
    // assert that raster coverage includes all world X/Z coordinates.
    let map_samples_expected = true;
    assert!(map_samples_expected, "placeholder: assert full-map raster coverage once map API is public");
}

#[test]
fn requirement_water_only_below_sea() {
    // GIVEN a generated world with deterministic sea level derived from its height,
    // WHEN all WATER voxels are inspected,
    // THEN every WATER cell must be at or below computed sea level.
    let dims = [64, 48, 64];
    let seed = 0xA4_12_33_FF_22_99_71_11u64;
    let world = worldgen::generate(dims, seed);
    let sea_level = (dims[1] * 40) / 100;
    for x in 0..dims[0] {
        for y in 0..dims[1] {
            for z in 0..dims[2] {
                let mat = world.cells[linear_index(world.dims, x, y, z)];
                if mat == WATER {
                    assert!(
                        y <= sea_level,
                        "found WATER at y={y} above sea level={sea_level} (x={x}, z={z})"
                    );
                }
            }
        }
    }
}

#[test]
fn requirement_terrain_is_continuous_not_blobs() {
    // GIVEN a generated solid-ish chunk from worldgen output,
    // WHEN it is meshed with CubicMesher,
    // THEN vertex count should be high enough to represent a continuous terrain surface.
    let dims = [64, 48, 64];
    let seed = 0x4F_5B_22_11_77_44_C3_D2u64;
    let world = worldgen::generate(dims, seed);

    // Start with the ground chunk where bedrock + early strata should be dense.
    let chunk_edge = 16usize;
    let chunk = sample_chunk(&world, [0, 0, 0], chunk_edge);
    let view = ChunkView {
        id: ChunkId(0),
        voxels: &chunk,
    };
    let mesh = CubicMesher::mesh_cubic(view, LodLevel(0)).expect("mesh should be computable");
    assert!(
        mesh.vertices.len() > 1_024,
        "mesh vertices {} too low for a continuous terrain chunk",
        mesh.vertices.len()
    );
    assert!(
        mesh.indices.len() % 6 == 0,
        "faces must stay in full quads (indices divisible by 6)"
    );
}
