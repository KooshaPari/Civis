//! Civis Bevy standalone sandbox — composes library plugins and shared terrain/atmosphere modules.

use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
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
    // Persist any panic to `civ-panic.log` in the working dir so a crash is
    // captured even when launched from a shortcut with no console (per the
    // "fail loudly, never silently" stance). Chains to the default hook.
    {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let loc = info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
                .unwrap_or_else(|| "<unknown>".to_string());
            let msg = info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic>".to_string());
            use std::io::Write as _;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("civ-panic.log")
            {
                let _ = f.write_all(format!("PANIC at {loc}: {msg}\n").as_bytes());
            }
            default_hook(info);
        }));
    }

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
        // Civis app/window icon (graphite + neon voxel-world glyph). Sets the
        // embedded icon on the primary winit window at startup.
        .add_plugins(civ_bevy_ref::window_icon::WindowIconPlugin)
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

    // 2D procedural/SVG alternate map view (M key + far-zoom auto-engage).
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::map2d::Map2dPlugin);

    // Perception layer: CS2-style info-view overlays (Tab) + click-to-inspect.
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::info_views::InfoViewsPlugin);
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::inspect::InspectPlugin);

    // Event-feed / toast notifications.
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::notifications::NotificationsPlugin);

    // Terrain sculpting brush (raise/lower/flatten); bevy-only, no egui needed.
    #[cfg(feature = "bevy")]
    app.add_plugins(civ_bevy_ref::terraform_brush::TerraformBrushPlugin);

    // Material brush palette + voxel paint (Powder-Toy-style); bevy+egui.
    #[cfg(feature = "egui")]
    app.add_plugins(civ_bevy_ref::material_brush_ui::MaterialBrushPlugin);

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

    // Headless verification hook: when CIVIS_AUTOSHOT=<path> is set, capture one
    // screenshot after a short warm-up (so chunk meshes / GLTF scenes are loaded
    // and the camera has framed the world) and then exit. This lets a debug
    // worker confirm voxel terrain visibility by pixels without manual F9.
    if let Ok(path) = std::env::var("CIVIS_AUTOSHOT") {
        app.insert_resource(AutoShot {
            path,
            timer: Timer::from_seconds(4.0, TimerMode::Once),
            taken: false,
        })
        .add_systems(Update, auto_screenshot);
    }

    app.run();
}

#[derive(Resource)]
struct AutoShot {
    path: String,
    timer: Timer,
    taken: bool,
}

fn auto_screenshot(
    mut commands: Commands,
    time: Res<Time>,
    mut shot: ResMut<AutoShot>,
    mut exit: MessageWriter<AppExit>,
) {
    if shot.taken {
        // Give the capture a couple frames to flush to disk, then quit.
        shot.timer.tick(time.delta());
        if shot.timer.is_finished() {
            exit.write(AppExit::Success);
        }
        return;
    }
    if shot.timer.tick(time.delta()).just_finished() {
        let path = shot.path.clone();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        shot.taken = true;
        shot.timer = Timer::from_seconds(1.5, TimerMode::Once);
    }
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
