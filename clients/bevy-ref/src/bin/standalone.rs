//! Civis Bevy standalone sandbox — composes library plugins and shared terrain/atmosphere modules.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use civ_bevy_ref::{
    atmosphere::{
        animate_water, setup_atmosphere, update_lighting, DayNightCycle, SunLight, WaterSurface,
    },
    camera::{camera_input, update_camera, CameraRig},
    decorations::spawn_decorations,
    gpu_features::GpuFeaturesPlugin,
    native_backend::native_render_plugin,
    terrain::{terrain_mesh, WORLD_SIZE},
};

fn main() {
    App::new()
        .insert_resource(DayNightCycle::default())
        .insert_resource(CameraRig::default())
        .add_plugins(DefaultPlugins.set(native_render_plugin()))
        .add_plugins(GpuFeaturesPlugin)
        .add_plugins(civ_bevy_ref::sim_bridge::SimBridgePlugin)
        .add_plugins(civ_bevy_ref::game_ui::GameUiPlugin)
        .add_plugins(civ_bevy_ref::spawn_tools::SpawnToolsPlugin)
        .add_plugins(civ_bevy_ref::minimap::MinimapPlugin)
        .init_resource::<civ_bevy_ref::game_ui::GameUiSnapshot>()
        .add_systems(
            Startup,
            (setup_atmosphere, setup_world, spawn_decorations).chain(),
        )
        .add_systems(
            Update,
            (
                camera_input,
                update_camera,
                advance_day_night_cycle,
                animate_water,
                update_lighting,
            ),
        )
        .run();
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(60.0, 80.0, 60.0).looking_at(Vec3::new(128.0, 30.0, 128.0), Vec3::Y),
    ));

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

    let water_size = WORLD_SIZE * 1.05;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(water_size, 0.2, water_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.12, 0.35, 0.62, 0.55),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.2,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.1, 0.0),
        WaterSurface,
    ));
}

fn advance_day_night_cycle(time: Res<Time>, mut cycle: ResMut<DayNightCycle>) {
    const DAY_LENGTH_SECONDS: f32 = 10.0 * 60.0;
    cycle.time_of_day = (cycle.time_of_day + time.delta_secs() / DAY_LENGTH_SECONDS) % 1.0;
}
