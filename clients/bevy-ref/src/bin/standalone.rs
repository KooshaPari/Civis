//! Civis Bevy standalone sandbox — composes library plugins and shared terrain/atmosphere modules.

use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
#[cfg(feature = "voxel")]
use civ_bevy_ref::ocean::OceanPlugin;
use civ_bevy_ref::{
    post_fx::PostFxSettings,
    atmosphere::{animate_water, setup_atmosphere, update_lighting, DayNightCycle, WaterSurface},
    camera::{camera_input, update_camera, CameraRig},
    decorations::spawn_decorations,
    gpu_features::GpuFeaturesPlugin,
    live_attach::LiveAttachPlugin,
    native_backend::native_render_plugin,
    resolve_attach_mode_from_env,
    terrain::{terrain_mesh, WORLD_SIZE},
    AttachMode,
};
#[cfg(feature = "gi")]
use civ_bevy_ref::lighting_gi::SolariGiPlugin;
#[cfg(feature = "egui")]
use civ_bevy_ref::settings_ui::{AntiAliasing, GameSettings, SettingsPlugin};
#[cfg(feature = "models")]
use civ_bevy_ref::animation::ActorAnimationPlugin;
#[cfg(feature = "models")]
use civ_bevy_ref::gltf_models::GltfModelsPlugin;

fn main() {
    civ_bevy_ref::install_crash_handler();

    let attach_mode = resolve_attach_mode_from_env();

    if let Err(message) = civ_bevy_ref::preflight::run_startup_preflight(attach_mode) {
        eprintln!("{message}");
        std::process::exit(1);
    }

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
        // Frame diagnostics: emit `FrameTime` + `SystemInformation` once per
        // second at INFO so the 90s frame-budget profile has a measurable
        // signal. See `docs/audits/frame-budget-baseline-2026-06-10.md`.
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugins(civ_bevy_ref::frame_budget::FrameBudgetPlugin)
        // Civis app/window icon (graphite + neon voxel-world glyph). Sets the
        // embedded icon on the primary winit window at startup.
        .add_plugins(civ_bevy_ref::window_icon::WindowIconPlugin)
        .add_plugins(civ_bevy_ref::sim_bridge::SimBridgePlugin)
        .add_plugins(civ_bevy_ref::post_fx::PostFxPlugin)
        .add_plugins(civ_bevy_ref::game_ui::GameUiPlugin)
        .add_plugins(civ_bevy_ref::emergence_dashboard::EmergenceDashboardPlugin)
        .add_plugins(civ_bevy_ref::tech_tree_ui::TechTreeUiPlugin)
        .add_plugins(civ_bevy_ref::diplomacy_ui::DiplomacyUiPlugin)
        .add_plugins(civ_bevy_ref::event_feed::EventFeedPlugin)
        .add_plugins(civ_bevy_ref::sandbox_event_feed::SandboxEventFeedPlugin)
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
    #[cfg(feature = "egui")]
    {
        app.add_plugins(SettingsPlugin)
            .add_systems(Startup, sync_post_fx_from_settings)
            .add_systems(
                Update,
                sync_post_fx_from_settings.run_if(resource_changed::<GameSettings>),
            );
    }

    #[cfg(feature = "models")]
    {
        app.add_plugins((GltfModelsPlugin, ActorAnimationPlugin));
    }

    #[cfg(feature = "gi")]
    {
        app.add_plugins(SolariGiPlugin);
    }

    if attach_mode == AttachMode::Standalone {
        #[cfg(feature = "pbr-textures")]
        app.add_plugins(civ_bevy_ref::materials::BiomeMaterialsPlugin);
    }

    // Perception layer: CS2-style terrain overlays + Tab nearby-counts HUD + inspect.
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::info_views::InfoViewsPlugin);
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::inspect::InspectPlugin);
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::entity_inspector::EntityInspectorPlugin);

    // Event-feed / toast notifications.
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::notifications::NotificationsPlugin);

    // Terrain sculpting brush (raise/lower/flatten); bevy-only, no egui needed.
    #[cfg(feature = "bevy")]
    app.add_plugins(civ_bevy_ref::terraform_brush::TerraformBrushPlugin);

    // God-game disaster actions (meteor/flood/quake/storm/wildfire) that mutate
    // the voxel world; bevy-only, gated systems handle egui/voxel internally.
    #[cfg(feature = "bevy")]
    app.add_plugins(civ_bevy_ref::disaster_tools::DisasterToolsPlugin);

    // Material brush palette + voxel paint (Powder-Toy-style); bevy+egui.
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::material_brush_ui::MaterialBrushPlugin);

    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::game_laws::GameLawsPlugin);

    // Gameplay HUD: faction leaderboard + victory progress + outcome banner (F9).
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::gameplay_hud::GameplayHudPlugin);

    // Settings / options panel (RON-persisted); bevy+egui.
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::settings_ui::SettingsPlugin);
    // Ambient + SFX audio (feature-gated).
    #[cfg(feature = "audio")]
    app.add_plugins(civ_bevy_ref::audio::CivisAudioPlugin);
    // GPU particle VFX for events (feature-gated).
    #[cfg(feature = "vfx")]
    app.add_plugins(civ_bevy_ref::vfx::VfxPlugin);
    // Real-time RT global illumination via bevy_solari (feature-gated).
    #[cfg(feature = "gi")]
    app.add_plugins(civ_bevy_ref::lighting_gi::SolariGiPlugin);

    // P-VM-3: real volumetric voxel material world (replaces the heightmap).
    // `voxel_stream` takes precedence: when enabled, the camera-driven streaming
    // sandbox owns the world instead of the bounded dense `VoxelSimPlugin`.
    #[cfg(all(feature = "voxel", not(feature = "voxel_stream")))]
    app.add_plugins(civ_bevy_ref::voxel_sim::VoxelSimPlugin);

    // OceanPlugin — wraps bevy_water::WaterPlugin.  Gated on `voxel` (which
    // pulls bevy_water).  Two modes:
    //
    // • voxel + voxel_stream  → full mode (OceanPlugin::default): WaterPlugin
    //   + WaterSettings + wave-plane spawn.  VoxelStreamPlugin does NOT spawn
    //   a water plane, so OceanPlugin owns the surface here.
    //
    // • voxel only (VoxelSimPlugin active) → thin mode (water_plugin_only):
    //   registers WaterPlugin shader infrastructure but skips the spawn because
    //   VoxelSimPlugin::spawn_bevy_water_plane already owns the wave surface.
    #[cfg(all(feature = "voxel", feature = "voxel_stream"))]
    app.add_plugins(OceanPlugin::default());
    #[cfg(all(feature = "voxel", not(feature = "voxel_stream")))]
    app.add_plugins(OceanPlugin::water_plugin_only());

    // FR-CIV-VOXEL-020: camera-driven chunk streaming over the 20mi voxel world.
    #[cfg(feature = "voxel_stream")]
    app.add_plugins(civ_bevy_ref::voxel_stream::VoxelStreamPlugin);

    // CC0 GLTF models: populate GameModels so sim_bridge swaps capsule/cuboid
    // primitives for real Knight/house scenes (per-asset primitive fallback).
    #[cfg(feature = "models")]
    app.add_plugins(civ_bevy_ref::gltf_models::GltfModelsPlugin);

    // Actor rigging: drive glTF skeletal animation from emergent motion so
    // agents idle / walk / run + face their heading instead of sliding statically.
    #[cfg(feature = "models")]
    app.add_plugins(civ_bevy_ref::animation::ActorAnimationPlugin);

    if attach_mode == AttachMode::Server {
        app.add_plugins(LiveAttachPlugin);
    }

    app.run();
}

#[cfg(feature = "egui")]
fn sync_post_fx_from_settings(
    settings: Res<GameSettings>,
    mut post_fx: ResMut<PostFxSettings>,
) {
    let graphics = &settings.graphics;
    post_fx.aces = graphics.anti_aliasing != AntiAliasing::Off;
    post_fx.bloom = graphics.bloom;
    post_fx.ssao = graphics.ambient_occlusion;
    post_fx.taa = graphics.anti_aliasing == AntiAliasing::TAA;
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
