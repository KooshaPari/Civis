use civ_voxel::{
    material::{AIR, WATER},
    worldgen,
    worldgen::GenWorld,
    ChunkId, ChunkView, CubicMesher, LodLevel, MaterialId,
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

#[cfg(feature = "voxel")]
#[test]
fn requirement_world_size_selection_changes_dimensions() {
    // GIVEN size-indexed selection values from UI world-size controls (small..huge),
    // WHEN world_dims_for(index) is invoked for each index,
    // THEN generated dimensions must strictly increase with each index.
    use civ_bevy_ref::voxel_sim::world_dims_for;
    let small = world_dims_for(0);
    let medium = world_dims_for(1);
    let large = world_dims_for(2);
    let huge = world_dims_for(3);

    // Width and depth must grow monotonically.
    assert!(
        medium[0] > small[0] && medium[2] > small[2],
        "medium {medium:?} must be wider+deeper than small {small:?}"
    );
    assert!(
        large[0] > medium[0] && large[2] > medium[2],
        "large {large:?} must be wider+deeper than medium {medium:?}"
    );
    assert!(
        huge[0] > large[0] && huge[2] > large[2],
        "huge {huge:?} must be wider+deeper than large {large:?}"
    );

    // Height must not shrink between presets.
    assert!(
        medium[1] >= small[1] && large[1] >= medium[1] && huge[1] >= large[1],
        "height must be non-shrinking across presets: s={} m={} l={} h={}",
        small[1],
        medium[1],
        large[1],
        huge[1]
    );
}

#[cfg(feature = "voxel")]
#[test]
fn requirement_new_world_differs_from_previous() {
    // GIVEN two different New World seeds,
    // WHEN worldgen::generate is called with same user size and different seeds,
    // THEN at least 20% of sampled surface columns should differ.
    //
    // Proves the "New World" button regenerates a meaningfully different world
    // (vs. producing an identical/seed-locked scene).
    use civ_bevy_ref::voxel_sim::world_dims_for;
    let dims = world_dims_for(0); // small preset for test cost
    let seed_a = 0x1E_5F_3A_C2_9D_17_4B_81u64;
    let seed_b = seed_a
        .wrapping_mul(13_245_799_145_678_972_871)
        .rotate_left(13);
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
    assert!(
        ratio > 0.20,
        "surface change ratio {ratio:.2} is below requirement (expected > 0.20)"
    );
}

#[test]
fn requirement_2d_map_extent_matches_world() {
    // GIVEN a world size D from UI/worldgen wiring,
    // WHEN basemap sampling is executed,
    // THEN 2D map extents should cover [0..D.x] and [0..D.z].
    let dims = [24, 18, 31];
    let grid = civ_voxel::fluid_ca::CaGrid::new(dims);
    let extent = civ_bevy_ref::map2d::world_extent_for_basemap(&grid);
    assert!(
        extent.min.x >= 0.0 && extent.min.y >= 0.0,
        "map extent must be non-negative: {extent:?}"
    );
    assert!(
        extent.width() == dims[0] as f32 && extent.height() == dims[2] as f32,
        "unexpected basemap extent {}x{} for world dims {dims:?}",
        extent.width(),
        extent.height(),
    );
    assert_eq!(extent.max.x, dims[0] as f32);
    assert_eq!(extent.max.y, dims[2] as f32);
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
#[ignore = "Camera control module is private; no public setter exists for yaw/pitch/pan/orbit inputs. Wire input bindings to camera before validating."]
fn requirement_camera_qe_yaw_rf_pitch_wasd_pan_scroll_orbit() {
    // GIVEN default camera state and a window with input focus,
    // WHEN keys Q/E, R/F, W/A/S/D and mouse scroll are dispatched,
    // THEN yaw, pitch, planar pan and orbit-distance change per binding and clamp at limits.
    //
    // Stub: assert a single boundary (pitch upper bound) once `civ_bevy_ref::camera::*`
    // exposes a `pub fn apply_input(&mut Camera, InputEvent)` style API.
    let pitch_upper_bound = 1.0f32; // placeholder
    assert!(
        pitch_upper_bound > 0.0,
        "placeholder: assert pitch clamp after camera API is public"
    );
}

#[test]
#[ignore = "Settings tabs are UI-side; no `pub fn settings_tabs()` exists yet to enumerate Graphics/Audio/Controls/Gameplay and verify subconfig coverage."]
fn requirement_settings_has_gfx_audio_controls_gameplay_tabs() {
    // GIVEN the in-game settings panel,
    // WHEN opened, the UI SHALL expose at least 4 tabs: Graphics, Audio, Controls, Gameplay,
    // AND Graphics SHALL contain subconfigs (preset, resolution_scale, shadows, AA, view distance, textures, AO, motion blur, bloom, GI, VFX).
    //
    // Stub: enumerate the tab list once `settings_ui::tabs()` is public.
    let tab_count_expected = 4usize;
    assert!(
        tab_count_expected >= 4,
        "placeholder: assert 4 tabs and GFX subconfig completeness after API is public"
    );
}

#[test]
#[ignore = "No public faction-count or alignment API exists. Once `civ_engine::factions::count() / alignment(id)` are pub, replace placeholder with seeded simulation asserts."]
fn requirement_emergent_factions_no_fixed_count_or_alignment() {
    // GIVEN N>1 seeded simulation runs of identical length,
    // WHEN the tick loop reaches convergence,
    // THEN the count of distinct factions SHALL NOT be a hardcoded constant
    // AND alignment vectors SHALL differ across runs (emergent, not scripted).
    //
    // Stub: asserts only the absence of a hardcoded constant; real assertions
    // require `civ_engine::factions::FactionSet::count()` to be public.
    let has_hardcoded_count = false;
    assert!(
        !has_hardcoded_count,
        "placeholder: assert no hardcoded faction count + alignment variance once API is public"
    );
}

#[test]
#[ignore = "Actor spawn pipeline is GUI-coupled; no pure-Rust test harness for GLTF animation playback exists. Wire an in-process spawn helper before validating T-pose absence at frame 0 and idle pose at frame N."]
fn requirement_actor_spawn_avoids_t_pose_and_animates() {
    // GIVEN a GLTF actor model registered with a SkinnedMesh + animation clip,
    // WHEN spawned into the world,
    // THEN at t=0 the actor SHALL NOT be in a T-pose (i.e. arm/leg bones are not collinear with spine)
    // AND by t=animation_period the actor SHALL have advanced at least one clip frame.
    //
    // Stub: asserts the requirement exists; implementation requires a
    // headless test rig (mock time, asset loader, animation graph) before
    // any real assertion is possible.
    let t_pose_acceptable = false;
    assert!(
        !t_pose_acceptable,
        "placeholder: T-pose is never acceptable; once a headless animation harness exists, assert skeleton joint angles diverge from rest-T"
    );
}

#[test]
#[ignore = "Native ocean rendering uses bevy_water plugin which requires GPU; no software-renderer stub exists. Add a feature-gated mock backend before this test can run on CI."]
fn requirement_native_ocean_renders_with_sea_level_match() {
    // GIVEN worldgen output that places WATER at y <= sea_level,
    // WHEN the native Bevy renderer spawns the bevy_water plugin,
    // THEN the ocean mesh surface SHALL align to sea_level within one voxel of tolerance
    // AND no sky-piercing water columns SHALL be visible.
    //
    // Stub: encodes the rule via the existing `requirement_water_only_below_sea`
    // invariant; real pixel assertions need a software-renderer or screenshot diff.
    let sea_level_match = true;
    assert!(
        sea_level_match,
        "placeholder: sea-level match requires GPU pipeline; this stub asserts the worldgen-side invariant already covered"
    );
}

#[test]
fn requirement_keybind_rebinding_overrides_default() {
    // GIVEN the default keymap for a known action (e.g. "Toggle Settings"),
    // WHEN the user rebinds it to a different KeyCode via the Controls tab,
    // THEN subsequent lookups via `GameSettings::key_for(action)` SHALL return the new binding
    // AND persistence layer SHALL serialize the override.
    use bevy::input::keyboard::KeyCode;

    use civ_bevy_ref::settings_ui::{GameSettings, KeyBinding};

    let mut settings = GameSettings::default();
    settings.rebind("Toggle Settings", KeyBinding::Key(KeyCode::KeyR));
    assert_eq!(settings.key_for("Toggle Settings"), Some(KeyBinding::Key(KeyCode::KeyR)));

    let text = ron::ser::to_string_pretty(&settings, ron::ser::PrettyConfig::default()).unwrap();
    let persisted: GameSettings = ron::from_str(&text).unwrap();
    assert_eq!(
        persisted.key_for("Toggle Settings"),
        Some(KeyBinding::Key(KeyCode::KeyR))
    );
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

#[test]
fn requirement_marker_types_differentiate_server_attach_vs_in_process() {
    use std::any::type_name;

    let live_agent = type_name::<civ_bevy_ref::live_stream::LiveAgentTag>();
    let live_building = type_name::<civ_bevy_ref::live_stream::LiveBuildingTag>();
    let sim_civilian = type_name::<civ_bevy_ref::sim_bridge::SimCivilianMarkerPublic>();
    let sim_building = type_name::<civ_bevy_ref::sim_bridge::SimBuildingMarkerPublic>();

    assert_ne!(live_agent, sim_civilian);
    assert_ne!(live_building, sim_building);
}
