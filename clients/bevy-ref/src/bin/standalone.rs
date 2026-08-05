//! Civis Bevy standalone sandbox — composes library plugins and shared terrain/atmosphere modules.

use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
#[cfg(feature = "models")]
use civ_bevy_ref::animation::ActorAnimationPlugin;
#[cfg(feature = "models")]
use civ_bevy_ref::gltf_models::GltfModelsPlugin;
#[cfg(feature = "egui")]
use civ_bevy_ref::graphics_settings::GraphicsSettingsPlugin;
#[cfg(feature = "gi")]
use civ_bevy_ref::lighting_gi::SolariGiPlugin;
#[cfg(feature = "voxel")]
use civ_bevy_ref::ocean::OceanPlugin;
#[cfg(feature = "egui")]
use civ_bevy_ref::settings_ui::{AntiAliasing, GameSettings, SettingsPlugin};
use civ_bevy_ref::{
    atmosphere::{animate_water, setup_atmosphere, update_lighting, DayNightCycle, WaterSurface},
    camera::{camera_input, update_camera, CameraRig},
    decorations::spawn_decorations,
    gpu_features::GpuFeaturesPlugin,
    live_attach::LiveAttachPlugin,
    native_backend::native_render_plugin,
    post_fx::PostFxSettings,
    resolve_attach_mode_from_env,
    terrain::{terrain_mesh, WORLD_SIZE},
    AttachMode,
};

fn main() {
    civ_bevy_ref::install_crash_handler();

    // Apply AAA Graphics API combo from settings.ron (or Windows DX12 default).
    #[cfg(feature = "egui")]
    {
        civ_bevy_ref::settings_ui::GameSettings::apply_boot_render_engine();
    }
    #[cfg(not(feature = "egui"))]
    if std::env::var_os(civ_bevy_ref::native_backend::BACKEND_ENV).is_none() {
        #[cfg(target_os = "windows")]
        {
            std::env::set_var(civ_bevy_ref::native_backend::BACKEND_ENV, "dx12");
        }
    }

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
                .set(bevy::asset::AssetPlugin {
                    // Resolve assets for both a source checkout and a copied
                    // standalone build. Bevy otherwise anchors the asset root
                    // to `target/*`, which makes a playable dev build report
                    // every texture/UI asset as missing.
                    file_path: standalone_asset_root(),
                    ..default()
                })
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
        .add_plugins(civ_bevy_ref::save_load_ui::SaveLoadUiPlugin)
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
        app.add_plugins((SettingsPlugin, GraphicsSettingsPlugin))
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

    // Gameplay HUD + live-stream overlays require LiveStreamScene from LiveAttach.
    // Register them only in server attach mode (see block below).

    // Shell overlays that work in both sandbox and live attach.
    #[cfg(feature = "egui")]
    {
        app.add_plugins((
            civ_bevy_ref::tutorial::TutorialPlugin,
            civ_bevy_ref::controls_help::ControlsHelpPlugin,
            civ_bevy_ref::perf_hud::PerfHudPlugin,
            civ_bevy_ref::AgentNeedsPlugin,
        ));
    }

    // Ambient + SFX audio (feature-gated).
    // SettingsPlugin / SolariGi / GltfModels / ActorAnimation are registered once above.
    #[cfg(feature = "audio")]
    app.add_plugins(civ_bevy_ref::audio::CivisAudioPlugin);
    // GPU particle VFX for events (feature-gated).
    #[cfg(feature = "vfx")]
    app.add_plugins(civ_bevy_ref::vfx::VfxPlugin);

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

    if attach_mode == AttachMode::Server {
        app.add_plugins(LiveAttachPlugin);
        // Live HUD / god tools need LiveBridge + LiveStreamScene from LiveAttach.
        #[cfg(feature = "egui")]
        {
            app.add_plugins((
                civ_bevy_ref::gameplay_hud::GameplayHudPlugin,
                civ_bevy_ref::faction_hud::FactionHudPlugin,
                civ_bevy_ref::world_faction_glyphs::WorldFactionGlyphsPlugin,
                civ_bevy_ref::god_panel::GodPanelPlugin,
                civ_bevy_ref::god_actions::GodActionsPlugin,
                civ_bevy_ref::holocron_panel::HolocronPanelPlugin,
            ));
        }
    }

    // Bounded native launch smoke: `CIVIS_SMOKE_FRAMES=N` exits after N Update ticks
    // (preflight already printed). Used by `just civis-3d-standalone-smoke`.
    if let Some(frames) = smoke_frames_from_env() {
        app.insert_resource(SmokeExitAfter { frames })
            .add_systems(Update, exit_after_smoke_frames);
    }

    app.run();
}

fn standalone_asset_root() -> String {
    if let Some(root) = std::env::var_os("CIVIS_ASSET_ROOT") {
        return root.to_string_lossy().into_owned();
    }

    let cwd = std::env::current_dir().ok();
    let exe = std::env::current_exe().ok();
    let candidates = asset_root_candidates(cwd.as_deref(), exe.as_deref());

    candidates
        .into_iter()
        .find(|path| {
            path.is_dir()
                && path.join("ui").join("title-bg.png").is_file()
                && path.join("icons").join("tool_spawn.png").is_file()
        })
        .unwrap_or_else(|| std::path::PathBuf::from("assets"))
        .to_string_lossy()
        .into_owned()
}

fn asset_root_candidates(
    cwd: Option<&std::path::Path>,
    exe: Option<&std::path::Path>,
) -> Vec<std::path::PathBuf> {
    let mut candidates = Vec::new();
    if let Some(cwd) = cwd {
        candidates.push(cwd.join("clients").join("bevy-ref").join("assets"));
        candidates.push(cwd.join("assets"));
    }
    if let Some(exe) = exe {
        let exe_dir = exe.parent().unwrap_or(exe);
        // Packaged layout: civ-standalone.exe beside the source-style client
        // tree, without requiring CIVIS_ASSET_ROOT at runtime.
        candidates.push(exe_dir.join("clients").join("bevy-ref").join("assets"));
        candidates.push(exe_dir.join("assets"));
    }
    candidates
}

#[cfg(test)]
mod asset_root_tests {
    use super::asset_root_candidates;

    #[test]
    fn packaged_client_tree_is_checked_beside_executable() {
        let package_dir = std::path::PathBuf::from("dist");
        let executable = package_dir.join("civ-standalone.exe");
        let candidates = asset_root_candidates(None, Some(&executable));

        assert_eq!(
            candidates[0],
            package_dir.join("clients").join("bevy-ref").join("assets")
        );
        assert_eq!(candidates[1], package_dir.join("assets"));
    }

    #[test]
    fn checkout_tree_precedes_packaged_fallbacks() {
        let checkout = std::path::PathBuf::from("checkout");
        let package_dir = std::path::PathBuf::from("dist");
        let executable = package_dir.join("civ-standalone.exe");
        let candidates = asset_root_candidates(Some(&checkout), Some(&executable));

        assert_eq!(
            candidates[0],
            checkout.join("clients").join("bevy-ref").join("assets")
        );
        assert_eq!(candidates[1], checkout.join("assets"));
        assert_eq!(
            candidates[2],
            package_dir.join("clients").join("bevy-ref").join("assets")
        );
    }
}

/// How many Update frames to run before exiting (native smoke).
#[derive(Resource, Debug, Clone, Copy)]
struct SmokeExitAfter {
    frames: u32,
}

fn smoke_frames_from_env() -> Option<u32> {
    let raw = std::env::var("CIVIS_SMOKE_FRAMES").ok()?;
    let n: u32 = raw.trim().parse().ok()?;
    (n > 0).then_some(n)
}

fn exit_after_smoke_frames(budget: Res<SmokeExitAfter>, mut frames: Local<u32>) {
    *frames += 1;
    if *frames >= budget.frames {
        eprintln!(
            "[smoke] civ-standalone exiting after {} Update frame(s)",
            budget.frames
        );
        // The interactive runner tears down the primary Egui context after
        // AppExit is observed, which can race bevy_egui's multipass output
        // finalization. Smoke mode only needs a bounded launch assertion.
        std::process::exit(0);
    }
}

#[cfg(feature = "egui")]
fn sync_post_fx_from_settings(settings: Res<GameSettings>, mut post_fx: ResMut<PostFxSettings>) {
    let graphics = &settings.graphics;
    post_fx.aces = graphics.tonemapping_enabled;
    post_fx.tonemapping = graphics.tonemapping_enabled;
    post_fx.color_grading = graphics.color_grading_enabled;
    post_fx.bloom = graphics.bloom;
    post_fx.ssao = graphics.ssao_enabled;
    post_fx.ssr = graphics.ssr_enabled;
    post_fx.volumetric_fog = graphics.volumetric_fog_enabled;
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
