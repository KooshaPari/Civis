//! Civis Bevy standalone sandbox — composes library plugins and shared terrain/atmosphere modules.

use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use civ_bevy_ref::{
    atmosphere::{
        animate_water, setup_atmosphere, update_lighting, DayNightCycle, WaterSurface,
    },
    camera::{camera_input, update_camera, CameraRig},
    decorations::spawn_decorations,
    gpu_features::GpuFeaturesPlugin,
    live_attach::LiveAttachPlugin,
    native_backend::native_render_plugin,
    resolve_attach_mode_from_env,
    terrain::{terrain_height, terrain_mesh, WORLD_SIZE},
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
                setup_sandbox_terrain.run_if(in_sandbox_attach_mode),
                spawn_decorations.run_if(in_sandbox_attach_mode),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (camera_input, update_camera, animate_water, update_lighting),
        );

    if attach_mode == AttachMode::Standalone {
        #[cfg(feature = "pbr-textures")]
        app.add_plugins(civ_bevy_ref::materials::BiomeMaterialsPlugin);
    }

    if attach_mode == AttachMode::Server {
        app.add_plugins(LiveAttachPlugin);
    }

    app.run();
}

fn in_sandbox_attach_mode(mode: Res<AttachMode>) -> bool {
    *mode == AttachMode::Standalone
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 90.0, 150.0).looking_at(Vec3::new(0.0, 12.0, 0.0), Vec3::Y),
    ));
}

#[cfg(feature = "pbr-textures")]
fn setup_sandbox_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    biome_materials: Res<civ_bevy_ref::materials::BiomeMaterials>,
) {
    let terrain = terrain_mesh();
    let centre_h = terrain_height(WORLD_SIZE * 0.5, WORLD_SIZE * 0.5);
    let biome = civ_bevy_ref::terrain::pbr_biome_at_height(centre_h);
    commands.spawn((
        Mesh3d(meshes.add(terrain)),
        MeshMaterial3d(biome_materials.handle(biome).clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    spawn_sandbox_water(&mut commands, &mut meshes, &mut materials);
}

#[cfg(not(feature = "pbr-textures"))]
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

    spawn_sandbox_water(&mut commands, &mut meshes, &mut materials);
}

fn spawn_sandbox_water(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {

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
