//! Civis Bevy standalone sandbox — composes library plugins and shared terrain/atmosphere modules.

use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use civ_bevy_ref::{
    atmosphere::{animate_water, setup_atmosphere, update_lighting, DayNightCycle},
    camera::{camera_input, update_camera, CameraRig},
    decorations::spawn_decorations,
    gpu_features::GpuFeaturesPlugin,
    live_attach::LiveAttachPlugin,
    native_backend::native_render_plugin,
    resolve_attach_mode_from_env,
    terrain::terrain_mesh,
    AttachMode,
};

fn main() {
    let attach_mode = resolve_attach_mode_from_env();
    let window_title = match attach_mode {
        AttachMode::Standalone => "Civis — Bevy standalone".to_string(),
        AttachMode::Server => "Civis — Bevy standalone (live attach)".to_string(),
    };

    let mut app = App::new();
    app.insert_resource(DayNightCycle::default())
        .insert_resource(CameraRig::default())
        .insert_resource(attach_mode)
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: window_title,
                        ..default()
                    }),
                    ..default()
                })
                .set(native_render_plugin()),
        )
        .add_plugins(GpuFeaturesPlugin)
        .add_plugins(civ_bevy_ref::sim_bridge::SimBridgePlugin)
        .add_plugins(civ_bevy_ref::skybox::SkyboxPlugin)
        .add_plugins(civ_bevy_ref::post_fx::PostFxPlugin)
        .add_plugins(civ_bevy_ref::game_ui::GameUiPlugin)
        .add_plugins(civ_bevy_ref::tech_tree_ui::TechTreeUiPlugin)
        .add_plugins(civ_bevy_ref::event_feed::EventFeedPlugin)
        .add_plugins(civ_bevy_ref::menus::MenusPlugin)
        .add_plugins(civ_bevy_ref::spawn_tools::SpawnToolsPlugin)
        .add_plugins(civ_bevy_ref::minimap::MinimapPlugin)
        .init_resource::<civ_bevy_ref::game_ui::GameUiSnapshot>()
        .add_systems(Startup, setup_atmosphere)
        .add_systems(
            Startup,
            (
                setup_camera,
                // Heightmap terrain + decorations are the default-playable
                // fallback. Under the `voxel` feature VoxelSimPlugin owns the
                // world instead (see `heightmap_enabled`).
                setup_sandbox_terrain
                    .run_if(in_sandbox_attach_mode)
                    .run_if(heightmap_enabled),
                spawn_decorations
                    .run_if(in_sandbox_attach_mode)
                    .run_if(heightmap_enabled),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (camera_input, update_camera, animate_water, update_lighting),
        );

    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::diplomacy_ui::DiplomacyUiPlugin);

    // P-VM-3: real volumetric voxel material world (replaces the heightmap).
    #[cfg(feature = "voxel")]
    app.add_plugins(civ_bevy_ref::voxel_sim::VoxelSimPlugin);

    if attach_mode == AttachMode::Server {
        app.add_plugins(LiveAttachPlugin);
    }

    app.run();
}

fn in_sandbox_attach_mode(mode: Res<AttachMode>) -> bool {
    *mode == AttachMode::Standalone
}

/// True when the heightmap terrain should spawn — i.e. the `voxel` feature is
/// OFF. Under `voxel`, `VoxelSimPlugin` owns the world instead.
fn heightmap_enabled() -> bool {
    cfg!(not(feature = "voxel"))
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        // Far plane raised well past the sky dome (radius 4000) and star shell
        // (1500) so the skybox/stars are not frustum-culled at any zoom.
        Projection::Perspective(PerspectiveProjection {
            far: 10_000.0,
            ..default()
        }),
        Transform::from_xyz(0.0, 90.0, 150.0).looking_at(Vec3::new(0.0, 12.0, 0.0), Vec3::Y),
    ));
}

fn setup_sandbox_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let terrain = terrain_mesh();
    commands.spawn((
        Mesh3d(meshes.add(terrain)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.62, 0.38),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    // NOTE: the single water body is owned by `atmosphere::setup_atmosphere`
    // (a `WaterSurface` plane at `WATER_LEVEL`). Spawning another here would
    // double up the water; intentionally not spawned in this setup.
}
