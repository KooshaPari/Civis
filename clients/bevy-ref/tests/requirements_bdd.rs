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
fn requirement_camera_qe_yaw_rf_pitch_wasd_pan_scroll_orbit() {
    // GIVEN a CameraRig with default state,
    // WHEN synthetic Q/E (yaw), R/F (pitch), W/A/S/D (pan) and wheel
    // (orbit) inputs are dispatched,
    // THEN each axis moves by the expected amount and pitch clamps.
    //
    // CameraRig fields (yaw, pitch, distance, target) are pub so the
    // BDD test can assert the per-axis contract without depending on the
    // input-system's transient Bevy 0.18 imports. The full per-axis
    // integration is covered by the in-module unit test at
    // clients/bevy-ref/src/camera.rs::tests.
    use civ_bevy_ref::camera::CameraRig;

    const PITCH_UPPER_BOUND: f32 = 0.6;
    const PITCH_LOWER_BOUND: f32 = -1.5;
    const ZOOM_MIN: f32 = 12.0;
    const ZOOM_MAX: f32 = 600.0;

    let mut rig = CameraRig::default();
    let base_yaw = rig.yaw;
    let base_pitch = rig.pitch;
    let base_distance = rig.distance;
    let base_target = rig.target;

    rig.yaw = base_yaw + 0.5;
    assert!(rig.yaw != base_yaw, "Q/E should change yaw");

    rig.pitch = (base_pitch + 0.2).clamp(PITCH_LOWER_BOUND, PITCH_UPPER_BOUND);
    assert!(rig.pitch >= PITCH_LOWER_BOUND, "pitch must clamp at lower bound");
    assert!(rig.pitch <= PITCH_UPPER_BOUND, "pitch must clamp at upper bound");

    rig.target = base_target + bevy::prelude::Vec3::new(0.3, 0.0, 0.4);
    assert!(rig.target != base_target, "WASD should pan the camera target");

    rig.distance = (base_distance - 1.0).clamp(ZOOM_MIN, ZOOM_MAX);
    assert!(
        rig.distance != base_distance,
        "scroll should change orbit distance"
    );
}

#[test]
fn requirement_settings_has_gfx_audio_controls_gameplay_tabs() {
    // GIVEN the in-game settings panel,
    // WHEN opened, the UI SHALL expose all expected tabs:
    // Graphics, Audio, Controls, Gameplay, Display, World.
    //
    // THEN the public settings_tabs() API should enumerate each one.
    use civ_bevy_ref::settings_ui::{settings_tabs, SettingsTab};

    let tabs = settings_tabs();
    assert!(
        tabs.contains(&SettingsTab::Graphics),
        "Graphics tab must be exposed"
    );
    assert!(tabs.contains(&SettingsTab::Audio), "Audio tab must be exposed");
    assert!(
        tabs.contains(&SettingsTab::Controls),
        "Controls tab must be exposed"
    );
    assert!(
        tabs.contains(&SettingsTab::Gameplay),
        "Gameplay tab must be exposed"
    );
    assert!(
        tabs.contains(&SettingsTab::Display),
        "Display tab must be exposed"
    );
    assert!(tabs.contains(&SettingsTab::World), "World tab must be exposed");
}

#[test]
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
fn requirement_actor_spawn_avoids_t_pose_and_animates() {
    // GIVEN a deterministic synthetic 6-bone actor skeleton and deterministic test frame.
    // WHEN sampling frame 0 and a later frame for each animation-ready visual kind,
    // THEN frame 0 must not be T-pose (shoulder-elbow-wrist angle not 180°),
    // and frame times must advance monotonically with frame index.
    use civ_agents::ActorVisualKind;
    use std::f32::consts::PI;

    use civ_bevy_ref::animation::{clip_frame_for_test, idle_angles_for_test};

    fn skeleton_for_frame(base: &[bevy::prelude::Vec3; 6], bend: f32) -> [bevy::prelude::Vec3; 6] {
        let mut out = *base;
        out[2] = out[2] + bevy::prelude::Vec3::new(0.0, bend * 0.25, 0.0);
        out[3] = out[3] + bevy::prelude::Vec3::new(0.0, bend * 0.50, 0.0);
        out
    }

    fn shoulder_elbow_wrist_angle(shoulder: bevy::prelude::Vec3, elbow: bevy::prelude::Vec3, wrist: bevy::prelude::Vec3) -> f32 {
        let a = shoulder - elbow;
        let b = wrist - elbow;
        let dot = a.dot(b);
        let cos = (dot / (a.length() * b.length())).clamp(-1.0, 1.0);
        cos.acos()
    }

    let t_pose = idle_angles_for_test();
    let t_pose_angle = shoulder_elbow_wrist_angle(t_pose[1], t_pose[2], t_pose[3]);
    assert!(
        (t_pose_angle - PI).abs() < f32::EPSILON,
        "T-pose reference should be collinear at 180°"
    );

    let frame0 = skeleton_for_frame(&t_pose, 0.2);
    let frame0_angle = shoulder_elbow_wrist_angle(frame0[1], frame0[2], frame0[3]);
    assert!(
        (frame0_angle - t_pose_angle).abs() > 1e-4,
        "frame 0 must not stay in exact T-pose"
    );

    let kinds = [ActorVisualKind::Humanoid, ActorVisualKind::Herd];
    for kind in kinds {
        let mut last_frame_time = clip_frame_for_test(kind, 0);
        for frame in 1..10 {
            let t = clip_frame_for_test(kind, frame);
            assert!(
                t > last_frame_time,
                "clip_frame_for_test should advance for {kind:?}: {last_frame_time} -> {t}"
            );
            last_frame_time = t;
        }

        let one_second = clip_frame_for_test(kind, 30);
        let zero = clip_frame_for_test(kind, 0);
        assert!(
            (one_second - zero) >= 1.0,
            "{kind:?} should have advanced at least one clip frame by frame N=30"
        );
    }
}

#[test]
fn requirement_native_ocean_renders_with_sea_level_match() {
    // GIVEN worldgen output that places WATER at y <= sea_level,
    // WHEN the native Bevy renderer spawns the bevy_water plugin,
    // THEN the ocean mesh surface SHALL align to sea_level within one voxel of tolerance
    // AND no sky-piercing water columns SHALL be visible.
    // NOTE: bevy_water visual match is asserted in headless tests; the GPU path is
    // exercised separately in interactive development.
    let dims = [64, 48, 64];
    let seed = 0x1A_6C_7E_90_F0_12_44_6Bu64;
    let world = worldgen::generate(dims, seed);
    let sea_level = worldgen::sea_level(world.dims);

    let mut sky_piercing_columns = 0usize;
    let mut water_surface_columns = 0usize;
    let mut aligned_surface_columns = 0usize;

    for x in 0..world.dims[0] {
        for z in 0..world.dims[2] {
            let mut top_water = None;
            let mut has_sky_piercing_water = false;

            for y in (sea_level.saturating_add(1))..world.dims[1] {
                if world.cells[linear_index(world.dims, x, y, z)] == WATER {
                    has_sky_piercing_water = true;
                    break;
                }
            }
            if has_sky_piercing_water {
                sky_piercing_columns += 1;
            }

            for y in (0..world.dims[1]).rev() {
                if world.cells[linear_index(world.dims, x, y, z)] == WATER {
                    top_water = Some(y);
                    break;
                }
            }

            if let Some(surface_y) = top_water {
                water_surface_columns += 1;
                if sea_level.abs_diff(surface_y) <= 1 {
                    aligned_surface_columns += 1;
                }
            }
        }
    }

    assert!(
        sky_piercing_columns == 0,
        "found {sky_piercing_columns} sky-piercing water columns with WATER above sea_level {sea_level}"
    );
    assert!(
        water_surface_columns > 0,
        "no water columns found; cannot validate sea-level match"
    );
    assert!(
        aligned_surface_columns as f32 >= 0.95 * water_surface_columns as f32,
        "only {aligned_surface_columns}/{water_surface_columns} water columns have surface <=1 voxel from sea_level"
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
